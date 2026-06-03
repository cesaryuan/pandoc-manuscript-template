"""Generate MathType OLE bins, WMF previews, and placement metadata."""

import json
import os
import platform
import re
import shutil
import subprocess
from dataclasses import dataclass
from pathlib import Path

from .compound_file import CompoundFile


HELPER_PROJECT = Path("scripts/mathtype_ole_helper/MathTypeOleHelper.csproj")
HELPER_EXE = Path("scripts/mathtype_ole_helper/bin/Release/net48/MathTypeOleHelper.exe")
MATHTYPE_PROG_ID = "Equation.DSMT4"
MATHTYPE_MT6_RELATIVE_PATHS = (
    Path("System/64/MT6.dll"),
    Path("System/32/MT6.dll"),
    Path("MT6.dll"),
)
# Keep the sizing template in-repo so builds do not depend on a local MathType preferences path.
MATHTYPE_DEFAULT_PREFS_TEMPLATE = Path("scripts/mathtype/Times+Symbol 12.eqp")
BEGIN_ALIGNED_RE = re.compile(r"\\begin\s*\{\s*aligned\s*\}")
END_ALIGNED_RE = re.compile(r"\\end\s*\{\s*aligned\s*\}")


@dataclass(frozen=True)
class EquationRequest:
    """MathType generation inputs extracted from a marker-bound DOCX formula."""

    latex: str
    font_size_pt: float | None = None


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


@dataclass(frozen=True)
class MathTypeAvailability:
    """Human-readable result of checking whether MathType conversion can run."""

    reasons: tuple[str, ...]
    details: tuple[str, ...]

    @property
    def usable(self) -> bool:
        """Return True when no blocking MathType environment problem was found."""
        return not self.reasons

    def format_failure(self, heading: str = "MathType cannot be used") -> str:
        """Return a multi-line explanation suitable for build error output."""
        lines = [heading]
        if self.reasons:
            lines.append("Blocking reason(s):")
            lines.extend(f"  - {reason}" for reason in self.reasons)
        if self.details:
            lines.append("Detected detail(s):")
            lines.extend(f"  - {detail}" for detail in self.details)
        return "\n".join(lines)


def _read_hkcr_default(subkey: str) -> str | None:
    """Read a default HKCR value, preferring the 64-bit registry view."""
    if platform.system() != "Windows":
        return None

    try:
        import winreg
    except ImportError:
        return None

    flags = [getattr(winreg, "KEY_WOW64_64KEY", 0), 0]
    seen_flags: set[int] = set()
    for flag in flags:
        if flag in seen_flags:
            continue
        seen_flags.add(flag)
        try:
            with winreg.OpenKey(winreg.HKEY_CLASSES_ROOT, subkey, 0, winreg.KEY_READ | flag) as key:
                value, _value_type = winreg.QueryValueEx(key, "")
        except OSError:
            continue
        if isinstance(value, str) and value.strip():
            return value.strip()
    return None


def _registry_executable_path(command: str) -> Path | None:
    """Extract the executable path from a registry command value."""
    text = command.strip()
    match = re.match(r'^"([^"]+\.exe)"', text, flags=re.IGNORECASE)
    if match:
        return Path(match.group(1))

    # Registry command values are often unquoted even under Program Files.
    match = re.match(r"^(.+?\.exe)(?:\s|$)", text, flags=re.IGNORECASE)
    if match:
        return Path(match.group(1))
    return None


def _mathtype_install_roots(server_path: Path | None = None) -> list[Path]:
    """Return likely MathType install roots, preferring the registered OLE server."""
    roots: list[Path] = []
    if server_path is not None:
        server_dir = server_path.parent
        roots.append(server_dir)
        if server_dir.name.lower() == "system":
            roots.append(server_dir.parent)
        if server_dir.name.lower() in {"64", "32"} and server_dir.parent.name.lower() == "system":
            roots.append(server_dir.parent.parent)

    for env_name in ("ProgramFiles(x86)", "ProgramFiles", "ProgramW6432"):
        folder = os.environ.get(env_name)
        if folder:
            roots.append(Path(folder) / "MathType")

    unique_roots: list[Path] = []
    seen: set[str] = set()
    for root in roots:
        key = str(root).lower()
        if key not in seen:
            unique_roots.append(root)
            seen.add(key)
    return unique_roots


def find_mathtype_mt6_dll(server_path: Path | None = None) -> Path | None:
    """Find MT6.dll from MathType's registered server path or common install roots."""
    for root in _mathtype_install_roots(server_path):
        for relative_path in MATHTYPE_MT6_RELATIVE_PATHS:
            candidate = root / relative_path
            if candidate.exists():
                return candidate
    return None


def check_mathtype_availability() -> MathTypeAvailability:
    """Check OS, MathType OLE registration, server path, and helper tooling.

    This is intentionally a lightweight preflight for build.py. It catches the
    common hard failures before Pandoc does any work, while the actual converter
    still performs the real OLE generation for each equation.
    """
    reasons: list[str] = []
    details: list[str] = []

    system = platform.system()
    if system != "Windows":
        reasons.append(
            "MathType OLE conversion requires Windows because it uses COM/OLE "
            f"({MATHTYPE_PROG_ID}); detected system: {system or 'unknown'}."
        )
        return MathTypeAvailability(tuple(reasons), tuple(details))

    details.append("Windows detected.")

    server_path_for_dll: Path | None = None
    clsid = _read_hkcr_default(fr"{MATHTYPE_PROG_ID}\CLSID")
    if not clsid:
        reasons.append(
            f"MathType OLE class is not registered: HKCR\\{MATHTYPE_PROG_ID}\\CLSID "
            "was not found. Install MathType, or repair the MathType installation."
        )
    else:
        details.append(f"{MATHTYPE_PROG_ID} resolves to CLSID {clsid}.")
        server_value = _read_hkcr_default(fr"CLSID\{clsid}\LocalServer32") or _read_hkcr_default(
            fr"CLSID\{clsid}\LocalServer"
        )
        if not server_value:
            reasons.append(
                f"MathType CLSID {clsid} has no LocalServer32/LocalServer value. "
                "Repair or reinstall MathType so the OLE server is registered."
            )
        else:
            server_path = _registry_executable_path(server_value)
            if server_path is None:
                reasons.append(f"Could not parse MathType OLE server registry value: {server_value}")
            elif not server_path.exists():
                reasons.append(
                    f"MathType OLE server is registered but the executable is missing: {server_path}. "
                    "Repair or reinstall MathType."
                )
            else:
                server_path_for_dll = server_path
                details.append(f"MathType OLE server found: {server_path}.")

    dotnet_path = shutil.which("dotnet")
    helper_exists = HELPER_EXE.exists()
    if helper_exists:
        details.append(f"MathType OLE helper executable found: {HELPER_EXE}.")
    else:
        details.append(f"MathType OLE helper executable not found: {HELPER_EXE}.")

    if dotnet_path is not None:
        details.append("dotnet command found.")
    else:
        details.append("dotnet command not found.")

    if dotnet_path is None and not helper_exists:
        reasons.append(
            f"The `dotnet` command was not found and no prebuilt MathType OLE helper exists at {HELPER_EXE}."
        )
    elif dotnet_path is not None and not helper_exists:
        if not HELPER_PROJECT.exists():
            reasons.append(f"MathType OLE helper project is missing: {HELPER_PROJECT}.")
        else:
            details.append(f"MathType OLE helper project found: {HELPER_PROJECT}.")

    mt6_dll = find_mathtype_mt6_dll(server_path_for_dll)
    if mt6_dll is not None:
        details.append(f"MathType metadata DLL found: {mt6_dll}.")
    else:
        # The helper treats MT6.dll as optional baseline metadata. Keep it as
        # detail instead of a blocker so OLE conversion can still use WMF metrics.
        details.append(
            "Optional MathType metadata DLL was not found from the OLE server path or common install folders; "
            "baseline placement will use the WMF fallback if conversion succeeds."
        )

    return MathTypeAvailability(tuple(reasons), tuple(details))


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
    """Ensure the small .NET OLE helper exists for MathType conversion."""
    if HELPER_EXE.exists():
        print(f"[mathtype] helper executable found, skipping build: {HELPER_EXE}")
        return
    if shutil.which("dotnet") is None:
        raise RuntimeError(
            f"MathType OLE helper is missing and `dotnet` is unavailable; expected helper at {HELPER_EXE}"
        )
    if not HELPER_PROJECT.exists():
        raise FileNotFoundError(f"MathType OLE helper project is missing: {HELPER_PROJECT}")

    run(["dotnet", "build", str(HELPER_PROJECT), "-c", "Release", "-v:quiet"])
    if not HELPER_EXE.exists():
        raise FileNotFoundError(f"Release build did not create expected helper executable: {HELPER_EXE}")


def normalize_mathtype_latex(latex: str) -> str:
    """Rewrite LaTeX constructs that MathType's TeX input does not support."""
    # MathType rejects the AMS `aligned` environment, but accepts the closely
    # related `align` environment for the same multi-line equation content.
    text = BEGIN_ALIGNED_RE.sub(r"\\begin{align}", latex)
    return END_ALIGNED_RE.sub(r"\\end{align}", text)


def mathtype_tex_payload(latex: str) -> str:
    """Return TeX text in the math-delimited form accepted by MathType OLE.

    Pandoc emits math content without the surrounding delimiters. MathType's OLE
    SetData path rejects bare fragments such as ``f(x)`` or ``w_i`` with
    DV_E_FORMATETC, but accepts the same TeX wrapped as ``$...$``.
    """
    text = normalize_mathtype_latex(latex.strip())
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


def format_font_size_pt(value: float) -> str:
    """Format a point size for MathType preference files."""
    return f"{value:.2f}".rstrip("0").rstrip(".")


def write_sized_prefs_file(template_path: Path, output_path: Path, full_size_pt: float) -> None:
    """Clone a MathType `.eqp` file and patch only the Full size setting.

    The built-in MathType size model expresses most other sizes as percentages,
    so adjusting `Full` is enough to keep script/symbol sizes proportional when
    we need equations in tables to match smaller surrounding Word text.
    """
    text = template_path.read_text(encoding="utf-8", errors="replace")
    lines = text.splitlines()
    inside_sizes = False
    updated = False
    for index, line in enumerate(lines):
        stripped = line.strip()
        if stripped.startswith("[") and stripped.endswith("]"):
            inside_sizes = stripped.casefold() == "[sizes]"
            continue
        if inside_sizes and stripped.startswith("Full="):
            lines[index] = f"Full={format_font_size_pt(full_size_pt)} pt"
            updated = True
            break

    if not updated:
        raise ValueError(f"Could not find [Sizes]/Full entry in MathType preferences template: {template_path}")

    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def make_ole_from_format(
    format_name: str,
    input_path: Path,
    output_path: Path,
    binary: bool = False,
    preview_output: Path | None = None,
    metadata_output: Path | None = None,
    prefs_file: Path | None = None,
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
    if prefs_file is not None:
        command.extend(["--prefs-file", str(prefs_file)])
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


def generate_equation_parts(requests: list[EquationRequest], output_dir: Path) -> list[GeneratedEquation]:
    """Generate OLE bins and WMF previews for all marker-bound formulas."""
    output_dir.mkdir(parents=True, exist_ok=True)
    equations: list[GeneratedEquation] = []
    prefs_template = MATHTYPE_DEFAULT_PREFS_TEMPLATE if MATHTYPE_DEFAULT_PREFS_TEMPLATE.exists() else None
    prefs_cache: dict[float, Path] = {}
    needs_variable_sizes = any(request.font_size_pt is not None for request in requests)
    if needs_variable_sizes and prefs_template is None:
        print(
            "[mathtype] warning: MathType preference template not found; "
            "equations will fall back to MathType's current default size"
        )

    for index, request in enumerate(requests, start=1):
        latex = request.latex
        input_path = output_dir / f"eq_{index:03d}.tex"
        ole_path = output_dir / f"eq_{index:03d}.ole.bin"
        wmf_path = output_dir / f"eq_{index:03d}.wmf"
        metadata_path = output_dir / f"eq_{index:03d}.json"
        prefs_path: Path | None = None

        if request.font_size_pt is not None and prefs_template is not None:
            font_size_key = round(request.font_size_pt * 2) / 2
            prefs_path = prefs_cache.get(font_size_key)
            if prefs_path is None:
                prefs_path = output_dir / "prefs" / f"full-{format_font_size_pt(font_size_key).replace('.', '_')}pt.eqp"
                write_sized_prefs_file(prefs_template, prefs_path, font_size_key)
                prefs_cache[font_size_key] = prefs_path

        write_latex_input(input_path, latex)
        try:
            make_ole_from_format(
                "TeX Input Language",
                input_path,
                ole_path,
                preview_output=wmf_path,
                metadata_output=metadata_path,
                prefs_file=prefs_path,
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
