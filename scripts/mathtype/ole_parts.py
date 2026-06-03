"""Generate MathType OLE bins, WMF previews, and placement metadata."""

import json
import subprocess
from dataclasses import dataclass
from pathlib import Path

from .compound_file import CompoundFile


HELPER_PROJECT = Path("scripts/mathtype_ole_helper/MathTypeOleHelper.csproj")
HELPER_EXE = Path("scripts/mathtype_ole_helper/bin/Debug/net48/MathTypeOleHelper.exe")


@dataclass
class GeneratedEquation:
    """Generated MathType object parts for a single DOCX math node."""

    latex: str
    ole_path: Path
    wmf_path: Path
    metadata_path: Path | None = None

    @property
    def baseline_from_bottom_pt(self) -> float | None:
        """Return MathType's baseline distance from the preview bottom."""
        if self.metadata_path is None or not self.metadata_path.exists():
            return None
        data = json.loads(self.metadata_path.read_text(encoding="utf-8-sig"))
        mathtype = data.get("mathtype")
        if not isinstance(mathtype, dict):
            return None
        value = mathtype.get("baseline_from_bottom_pt")
        if isinstance(value, (int, float)) and value > 0:
            return float(value)
        return None


def run(command: list[str], echo_stdout: bool = True) -> subprocess.CompletedProcess:
    """Run a command and echo its useful output for MathType logs."""
    result = subprocess.run(
        command,
        check=False,
        text=True,
        capture_output=True,
        encoding="utf-8",
        errors="replace",
    )
    if echo_stdout and result.stdout.strip():
        print(result.stdout.strip())
    if result.stderr.strip():
        print(result.stderr.strip())
    if result.returncode != 0:
        raise RuntimeError(f"command failed: {' '.join(command)}")
    return result


def build_helper() -> None:
    """Build the small .NET OLE helper used by MathType conversion."""
    run(["dotnet", "build", str(HELPER_PROJECT), "-v:quiet"])


def mathtype_tex_payload(latex: str) -> str:
    """Return TeX text in the math-delimited form accepted by MathType OLE.

    Pandoc emits math content without the surrounding delimiters. MathType's OLE
    SetData path rejects bare fragments such as ``f(x)`` or ``w_i`` with
    DV_E_FORMATETC, but accepts the same TeX wrapped as ``$...$``.
    """
    text = latex.strip()
    if text.startswith("$$") and text.endswith("$$"):
        return text
    if text.startswith("$") and text.endswith("$"):
        return text
    if text.startswith(r"\(") and text.endswith(r"\)"):
        return "$" + text[2:-2].strip() + "$"
    if text.startswith(r"\[") and text.endswith(r"\]"):
        return "$$" + text[2:-2].strip() + "$$"
    return "$" + text + "$"


def write_latex_input(path: Path, latex: str) -> None:
    """Write MathType-ready TeX as UTF-8; the helper converts it to UTF-16LE."""
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(mathtype_tex_payload(latex), encoding="utf-8")


def make_ole_from_format(
    format_name: str,
    input_path: Path,
    output_path: Path,
    binary: bool = False,
    preview_output: Path | None = None,
    metadata_output: Path | None = None,
) -> None:
    """Ask MathType OLE to create an Equation.DSMT4 compound file without Word."""
    command = [
        str(HELPER_EXE),
        "--method",
        "set-data",
        "--pre-verb",
        "2",
        "--format",
        format_name,
        "--input",
        str(input_path),
        "--output",
        str(output_path),
        "--encoding",
        "utf16le",
        "--no-verb",
    ]
    if binary:
        command.append("--binary")
    if preview_output is not None:
        command.extend(["--preview-output", str(preview_output)])
    if metadata_output is not None:
        command.extend(["--metadata-output", str(metadata_output)])
    run(command)


def inspect_ole(path: Path) -> CompoundFile:
    """Print the key MathType OLE stream evidence."""
    compound = CompoundFile(path.read_bytes())
    names = ", ".join(entry.name for entry in compound.entries if entry.name)
    native = compound.read_stream("Equation Native")
    print(f"[mathtype] {path}: streams={names}")
    print(f"[mathtype] Equation Native bytes={len(native)}, DSMT offset={native.find(b'DSMT')}")
    return compound


def generate_equation_parts(latex_values: list[str], output_dir: Path) -> list[GeneratedEquation]:
    """Generate OLE bins and WMF previews for all marker-bound LaTeX formulas."""
    output_dir.mkdir(parents=True, exist_ok=True)
    equations: list[GeneratedEquation] = []
    for index, latex in enumerate(latex_values, start=1):
        input_path = output_dir / f"eq_{index:03d}.tex"
        ole_path = output_dir / f"eq_{index:03d}.ole.bin"
        wmf_path = output_dir / f"eq_{index:03d}.wmf"
        metadata_path = output_dir / f"eq_{index:03d}.json"
        write_latex_input(input_path, latex)
        try:
            make_ole_from_format(
                "TeX Input Language",
                input_path,
                ole_path,
                preview_output=wmf_path,
                metadata_output=metadata_path,
            )
        except RuntimeError as exc:
            raise RuntimeError(
                f"TeX input failed for equation {index}; "
                "the MathType converter no longer calls Pandoc for fallback MathML"
            ) from exc
        compound = inspect_ole(ole_path)
        if compound.read_stream("Equation Native").find(b"DSMT") < 0:
            raise ValueError(f"generated OLE lacks DSMT marker: {ole_path}")
        if not wmf_path.exists() or wmf_path.read_bytes()[:4] != bytes.fromhex("d7cdc69a"):
            raise ValueError(f"generated WMF preview is missing placeable header: {wmf_path}")
        equations.append(GeneratedEquation(latex=latex, ole_path=ole_path, wmf_path=wmf_path, metadata_path=metadata_path))
    return equations
