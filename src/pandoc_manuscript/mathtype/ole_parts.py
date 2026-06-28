"""Generate MathType OLE bins, WMF previews, and placement metadata."""

import hashlib
import json
import locale
import os
import platform
import re
import shutil
import subprocess
from collections.abc import Iterator
from dataclasses import dataclass
from pathlib import Path
from typing import Literal

from tqdm import tqdm

from ..runtime.logging import log_debug, log_info, log_warning, should_log
from ..runtime.paths import PMT_MATHTYPE_CACHE_DIR
from ..runtime.resources import package_resource_path, source_tree_root

from .compound_file import CompoundFile


def resource_path(path: str | Path) -> Path:
    """Resolve a MathType runtime resource bundled inside the package."""
    path = Path(path)
    if path.is_absolute():
        return path
    return package_resource_path(path)


HELPER_PROJECT = resource_path("mathtype/ole_helper/MathTypeOleHelper.csproj")
HELPER_EXE = resource_path("mathtype/ole_helper/bin/Release/net48/MathTypeOleHelper.exe")
MATHTYPE_RUST_SOURCE_EXE_NAME = "mathtype-rust.exe" if os.name == "nt" else "mathtype-rust"
MATHTYPE_RUST_PACKAGE_EXE_NAME = "mathtype-rust.exe"
MATHTYPE_RUST_PACKAGE_EXE = resource_path(Path("mathtype/bin") / MATHTYPE_RUST_PACKAGE_EXE_NAME)
MATHTYPE_PROG_ID = "Equation.DSMT4"
MATHTYPE_MT6_RELATIVE_PATHS = (
    Path("System/64/MT6.dll"),
    Path("System/32/MT6.dll"),
    Path("MT6.dll"),
)
# Keep the sizing template in-repo so builds do not depend on a local MathType preferences path.
MATHTYPE_DEFAULT_PREFS_TEMPLATE = resource_path("mathtype/Times+Symbol 12.eqp")
MATHTYPE_CACHE_DIR = PMT_MATHTYPE_CACHE_DIR
MATHTYPE_CACHE_VERSION = 1
BEGIN_ALIGNED_RE = re.compile(r"\\begin\s*\{\s*aligned\s*\}")
END_ALIGNED_RE = re.compile(r"\\end\s*\{\s*aligned\s*\}")
MathTypeSingleConversionMethod = Literal["rust", "set-data"]
MathTypeConversionMethod = Literal["rust", "set-data", "auto", "both"]
DEFAULT_MATHTYPE_CONVERSION_METHOD: MathTypeConversionMethod = "rust"


def source_tree_path(path: str | Path) -> Path | None:
    """Resolve a repository path only when pmt is running from a source checkout."""
    path = Path(path)
    if path.is_absolute():
        return path
    root = source_tree_root()
    if root is not None:
        return root / path
    return None


MATHTYPE_RUST_PROJECT = source_tree_path("scripts/mathtype-rust/Cargo.toml")
MATHTYPE_RUST_SOURCE_EXE = source_tree_path(Path("scripts/mathtype-rust/target/debug") / MATHTYPE_RUST_SOURCE_EXE_NAME)
MATHTYPE_RUST_EXE = MATHTYPE_RUST_SOURCE_EXE or MATHTYPE_RUST_PACKAGE_EXE
MATHTYPE_RUST_BUILD_CHECKED = False


def normalize_conversion_method(value: object | None) -> MathTypeConversionMethod:
    """Normalize style metadata for MathType equation conversion backends."""
    if value is None:
        return DEFAULT_MATHTYPE_CONVERSION_METHOD
    text = str(value).strip().casefold().replace("_", "-")
    aliases: dict[str, MathTypeConversionMethod] = {
        "rust": "rust",
        "mathtype-rust": "rust",
        "mtef": "rust",
        "sdk": "rust",
        "sdk-xform-ole": "rust",
        "set-data": "set-data",
        "setdata": "set-data",
        "tex": "set-data",
        "tex-input": "set-data",
        "mathtype": "set-data",
        "ole": "set-data",
        "auto": "auto",
        "fallback": "auto",
        "both": "both",
    }
    method = aliases.get(text)
    if method is None:
        allowed = "rust, set-data, auto, both"
        raise ValueError(f"mathtypeConversionMethod must be one of: {allowed}; got {value!r}")
    return method


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
    """Check OS, MathType OLE registration, and helper tooling.

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
            # Some MathType installs activate through COM even without these path hints.
            details.append(
                f"MathType CLSID {clsid} has no LocalServer32/LocalServer value; "
                "continuing because COM activation can still work through other registry entries."
            )
        else:
            server_path = _registry_executable_path(server_value)
            if server_path is None:
                details.append(f"Could not parse optional MathType OLE server registry value: {server_value}")
            elif not server_path.exists():
                details.append(
                    f"Optional MathType OLE server path from registry does not exist: {server_path}; "
                    "continuing with COM availability checks."
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


def decode_process_output(data: bytes) -> str:
    """Decode helper output without corrupting localized Windows diagnostics.

    Older helper executables may write redirected stderr using the active
    Windows code page instead of UTF-8, so fall back before replacing bytes.
    """
    if not data:
        return ""

    encodings = ["utf-8", locale.getpreferredencoding(False), "gb18030"]
    if os.name == "nt":
        encodings.extend(["mbcs", "oem"])

    seen: set[str] = set()
    for encoding in encodings:
        normalized = encoding.lower()
        if normalized in seen:
            continue
        seen.add(normalized)
        try:
            return data.decode(encoding)
        except (LookupError, UnicodeDecodeError):
            continue
    return data.decode("utf-8", errors="replace")


def run(
    command: list[str],
    echo_stdout: bool = True,
    stderr_as_warning: bool = True,
) -> subprocess.CompletedProcess[str]:
    """Run a command and echo its useful output for MathType logs."""
    result = subprocess.run(
        command,
        check=False,
        capture_output=True,
    )
    stdout = decode_process_output(result.stdout)
    stderr = decode_process_output(result.stderr)
    if echo_stdout and result.stdout.strip():
        log_info(stdout.strip())
    if stderr.strip():
        if stderr_as_warning or result.returncode != 0:
            log_warning(stderr.strip())
        else:
            log_info(stderr.strip())
    if result.returncode != 0:
        raise RuntimeError(f"command failed: {' '.join(command)}")
    return subprocess.CompletedProcess(command, result.returncode, stdout=stdout, stderr=stderr)


def build_helper() -> None:
    """Ensure the small .NET OLE helper exists for MathType conversion."""
    if HELPER_EXE.exists():
        log_info(f"[mathtype] helper executable found, skipping build: {HELPER_EXE}")
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


def build_mathtype_rust_converter() -> Path:
    """Ensure the Rust MTEF converter exists for the MathType Rust path."""
    global MATHTYPE_RUST_BUILD_CHECKED
    if MATHTYPE_RUST_PROJECT is not None and MATHTYPE_RUST_PROJECT.exists():
        if shutil.which("cargo") is None:
            if MATHTYPE_RUST_EXE.exists():
                log_warning(
                    "[mathtype] warning: cargo not found; using existing mathtype-rust executable "
                    f"without rebuilding: {MATHTYPE_RUST_EXE}"
                )
                return MATHTYPE_RUST_EXE
            raise RuntimeError(
                f"mathtype-rust requires `cargo`, or a prebuilt executable at {MATHTYPE_RUST_EXE}"
            )
        if not MATHTYPE_RUST_BUILD_CHECKED:
            log_info(f"[mathtype] building mathtype-rust converter: {MATHTYPE_RUST_PROJECT}")
            run(
                ["cargo", "build", "--manifest-path", str(MATHTYPE_RUST_PROJECT)],
                stderr_as_warning=False,
            )
            MATHTYPE_RUST_BUILD_CHECKED = True
        if not MATHTYPE_RUST_EXE.exists():
            raise FileNotFoundError(f"Cargo build did not create expected executable: {MATHTYPE_RUST_EXE}")
        return MATHTYPE_RUST_EXE

    if MATHTYPE_RUST_EXE.exists():
        log_debug(f"[mathtype] mathtype-rust executable found: {MATHTYPE_RUST_EXE}")
        return MATHTYPE_RUST_EXE
    if MATHTYPE_RUST_PROJECT is None or not MATHTYPE_RUST_PROJECT.exists():
        raise FileNotFoundError(
            f"mathtype-rust executable is missing: {MATHTYPE_RUST_EXE}. "
            "Install a wheel that bundles mathtype-rust, or run from a source checkout with scripts/mathtype-rust."
        )
    raise FileNotFoundError(f"mathtype-rust executable is missing: {MATHTYPE_RUST_EXE}")


def mathtype_rust_exe_digest_for_method(conversion_method: MathTypeConversionMethod) -> str | None:
    """Build and hash mathtype-rust when the selected backend can use it."""
    if conversion_method == "set-data":
        return None
    if conversion_method == "auto":
        try:
            return file_sha256(build_mathtype_rust_converter())
        except (RuntimeError, FileNotFoundError):
            return None
    return file_sha256(build_mathtype_rust_converter())


def normalize_mathtype_latex(latex: str) -> str:
    """Rewrite LaTeX constructs that MathType's TeX input does not translate natively."""
    # MathType does not translate the AMS `aligned` environment as a native
    # alignment object. Rewriting it to the closely related `align`
    # environment preserves the row content while avoiding the raw-TeX
    # fallback that otherwise appears in the generated MTEF.
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


def file_sha256(path: Path) -> str | None:
    """Return a file digest for cache invalidation, or None when the file is absent."""
    if not path.exists():
        return None
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def mathtype_ole_mtef_payload(path: Path) -> bytes:
    """Return the bare MTEF payload from a MathType OLE Equation Native stream."""
    native = CompoundFile(path.read_bytes()).read_stream("Equation Native")
    if len(native) < 28:
        raise ValueError(f"Equation Native stream is shorter than the 28-byte MathType OLE header: {path}")
    header_size = int.from_bytes(native[:2], byteorder="little")
    if header_size != 28:
        raise ValueError(f"unexpected MathType OLE header size in {path}: {header_size}")
    return native[header_size:]


def mathtype_ole_mtef_sha256(path: Path) -> str:
    """Return a digest for only the MTEF bytes inside a MathType OLE file."""
    return hashlib.sha256(mathtype_ole_mtef_payload(path)).hexdigest()


JSON_RESULT_COMPARISON_FIELDS = (
    ("width_pt",),
    ("height_pt",),
    ("mathtype", "width_pt"),
    ("mathtype", "height_pt"),
    ("mathtype", "baseline_from_bottom_pt"),
)


def rounded_json_result_value(value: object) -> object:
    """Round numeric JSON metadata values before backend comparison."""
    if isinstance(value, (int, float)) and not isinstance(value, bool):
        return round(value)
    return value


def json_result_comparison_payload(data: object) -> dict[str, object]:
    """Keep only rounded point-size metrics that should match across backends."""
    if not isinstance(data, dict):
        return {}

    payload: dict[str, object] = {}
    for field_path in JSON_RESULT_COMPARISON_FIELDS:
        source: object = data
        for key in field_path:
            if not isinstance(source, dict) or key not in source:
                break
            source = source[key]
        else:
            target = payload
            for key in field_path[:-1]:
                nested = target.setdefault(key, {})
                if not isinstance(nested, dict):
                    nested = {}
                    target[key] = nested
                target = nested
            target[field_path[-1]] = rounded_json_result_value(source)
    return payload


def json_result_sha256(path: Path) -> str:
    """Return a stable digest for comparable rounded MathType JSON metrics."""
    data = json.loads(path.read_text(encoding="utf-8-sig"))
    data = json_result_comparison_payload(data)
    canonical = json.dumps(data, ensure_ascii=True, sort_keys=True, separators=(",", ":")).encode("utf-8")
    return hashlib.sha256(canonical).hexdigest()


def file_group_sha256(paths: list[Path]) -> str | None:
    """Return one digest for a small ordered set of existing input files."""
    existing_paths = [path for path in paths if path.exists()]
    if not existing_paths:
        return None

    digest = hashlib.sha256()
    for path in sorted(existing_paths, key=lambda item: item.as_posix().casefold()):
        digest.update(path.as_posix().encode("utf-8"))
        digest.update(b"\0")
        digest.update(path.read_bytes())
        digest.update(b"\0")
    return digest.hexdigest()


def mathtype_rust_source_digest() -> str | None:
    """Return a digest for Rust converter sources that affect generated MTEF."""
    if MATHTYPE_RUST_PROJECT is None:
        return None
    project_dir = MATHTYPE_RUST_PROJECT.parent
    paths = [MATHTYPE_RUST_PROJECT, project_dir / "Cargo.lock"]
    src_dir = project_dir / "src"
    if src_dir.exists():
        paths.extend(src_dir.rglob("*.rs"))
    return file_group_sha256(paths)


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


def cache_font_size_key(font_size_pt: float | None, prefs_template: Path | None) -> float | None:
    """Return the MathType preference size that affects generated object bytes."""
    if font_size_pt is None or prefs_template is None:
        return None
    return round(font_size_pt * 2) / 2


def mathtype_cache_key(
    latex: str,
    font_size_key: float | None,
    prefs_digest: str | None,
    helper_digest: str | None,
    rust_source_digest: str | None,
    rust_exe_digest: str | None,
    conversion_method: MathTypeSingleConversionMethod,
) -> str:
    """Build a stable cache key from the exact MathType inputs.

    The key uses the normalized TeX payload and generated preference contents,
    because those are the values passed to the MathType OLE helper. Rust
    converter digests are included so generated parts do not outlive the
    converter implementation or backend selection that produced them.
    """
    if conversion_method == "set-data":
        rust_source_digest = None
        rust_exe_digest = None

    payload = {
        "version": MATHTYPE_CACHE_VERSION,
        "tex_payload": mathtype_tex_payload(latex),
        "font_size_pt": font_size_key,
        "prefs_sha256": prefs_digest,
        "helper_sha256": helper_digest,
        "mathtype_rust_source_sha256": rust_source_digest,
        "mathtype_rust_exe_sha256": rust_exe_digest,
        "conversion_method": conversion_method,
    }
    data = json.dumps(payload, ensure_ascii=True, sort_keys=True, separators=(",", ":")).encode("utf-8")
    return hashlib.sha256(data).hexdigest()


def cache_paths(cache_key: str) -> tuple[Path, Path, Path]:
    """Return the cache file paths for one generated MathType equation."""
    folder = MATHTYPE_CACHE_DIR / cache_key[:2] / cache_key
    return folder / "equation.ole.bin", folder / "preview.wmf", folder / "metadata.json"


def valid_cached_parts(ole_path: Path, wmf_path: Path) -> bool:
    """Return True when cached MathType OLE and WMF files look usable."""
    if not ole_path.exists() or not wmf_path.exists():
        return False
    try:
        compound = CompoundFile(ole_path.read_bytes())
        has_mathtype_payload = compound.read_stream("Equation Native").find(b"DSMT") >= 0
        has_placeable_preview = wmf_path.read_bytes()[:4] == bytes.fromhex("d7cdc69a")
        return has_mathtype_payload and has_placeable_preview
    except Exception as exc:
        log_warning(f"[mathtype] warning: ignoring invalid cached equation {ole_path}: {exc}")
        return False


def restore_cached_equation(
    cache_key: str,
    ole_path: Path,
    wmf_path: Path,
    metadata_path: Path,
) -> bool:
    """Copy cached MathType parts into the current build work directory."""
    cached_ole, cached_wmf, cached_metadata = cache_paths(cache_key)
    if not valid_cached_parts(cached_ole, cached_wmf):
        return False

    shutil.copy2(cached_ole, ole_path)
    shutil.copy2(cached_wmf, wmf_path)
    if cached_metadata.exists():
        shutil.copy2(cached_metadata, metadata_path)
    elif metadata_path.exists():
        metadata_path.unlink()
    return True


def store_cached_equation(
    cache_key: str,
    ole_path: Path,
    wmf_path: Path,
    metadata_path: Path,
) -> None:
    """Save generated MathType parts so later builds can skip MathType COM."""
    cached_ole, cached_wmf, cached_metadata = cache_paths(cache_key)
    cached_ole.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(ole_path, cached_ole)
    shutil.copy2(wmf_path, cached_wmf)
    if metadata_path.exists():
        shutil.copy2(metadata_path, cached_metadata)


def generate_cached_equation_parts_for_method(
    index: int,
    latex: str,
    font_size_key: float | None,
    prefs_digest: str | None,
    helper_digest: str | None,
    rust_source_digest: str | None,
    rust_exe_digest: str | None,
    input_path: Path,
    ole_path: Path,
    wmf_path: Path,
    metadata_path: Path,
    mtef_path: Path,
    prefs_file: Path | None,
    conversion_method: MathTypeSingleConversionMethod,
) -> bool:
    """Restore or generate one equation for one cache-isolated backend."""
    cache_key = mathtype_cache_key(
        latex,
        font_size_key,
        prefs_digest,
        helper_digest,
        rust_source_digest,
        rust_exe_digest,
        conversion_method,
    )
    if restore_cached_equation(cache_key, ole_path, wmf_path, metadata_path):
        log_debug(f"[mathtype] cache hit eq={index} method={conversion_method} key={cache_key[:12]}")
        return True

    log_debug(f"[mathtype] cache miss eq={index} method={conversion_method} key={cache_key[:12]}")
    generate_uncached_equation_parts(
        index,
        input_path,
        ole_path,
        wmf_path,
        metadata_path,
        mtef_path,
        prefs_file=prefs_file,
        conversion_method=conversion_method,
    )
    store_cached_equation(cache_key, ole_path, wmf_path, metadata_path)
    return False


def generate_cached_equation_parts_auto(
    index: int,
    latex: str,
    font_size_key: float | None,
    prefs_digest: str | None,
    helper_digest: str | None,
    rust_source_digest: str | None,
    rust_exe_digest: str | None,
    input_path: Path,
    ole_path: Path,
    wmf_path: Path,
    metadata_path: Path,
    mtef_path: Path,
    prefs_file: Path | None,
) -> tuple[int, int]:
    """Try the rust cache/generator first, then the set-data cache/generator."""
    try:
        hit = generate_cached_equation_parts_for_method(
            index,
            latex,
            font_size_key,
            prefs_digest,
            helper_digest,
            rust_source_digest,
            rust_exe_digest,
            input_path,
            ole_path,
            wmf_path,
            metadata_path,
            mtef_path,
            prefs_file,
            "rust",
        )
        return int(hit), int(not hit)
    except (RuntimeError, FileNotFoundError):
        log_warning(
            f"[mathtype] mathtype-rust auto path failed for equation {index}; "
            "trying MathType TeX input fallback"
        )
        hit = generate_cached_equation_parts_for_method(
            index,
            latex,
            font_size_key,
            prefs_digest,
            helper_digest,
            rust_source_digest,
            rust_exe_digest,
            input_path,
            ole_path,
            wmf_path,
            metadata_path,
            mtef_path,
            prefs_file,
            "set-data",
        )
        return int(hit), 1 + int(not hit)


def warn_if_conversion_outputs_differ(
    index: int,
    rust_ole_path: Path,
    rust_metadata_path: Path,
    set_data_ole_path: Path,
    set_data_metadata_path: Path,
) -> None:
    """Warn when both MathType backends produce different OLE or JSON results."""
    differing_parts = []
    try:
        if mathtype_ole_mtef_sha256(rust_ole_path) != mathtype_ole_mtef_sha256(set_data_ole_path):
            differing_parts.append("OLE MTEF")
    except (KeyError, ValueError) as exc:
        differing_parts.append(f"OLE MTEF unreadable ({exc})")

    try:
        if json_result_sha256(rust_metadata_path) != json_result_sha256(set_data_metadata_path):
            differing_parts.append("JSON")
    except (OSError, ValueError) as exc:
        differing_parts.append(f"JSON unreadable ({exc})")

    if differing_parts:
        log_warning(
            f"[mathtype] warning: rust and set-data outputs differ for equation {index}: "
            f"{', '.join(differing_parts)}; using rust output"
        )


def make_ole_from_format(
    format_name: str,
    input_path: Path,
    output_path: Path,
    binary: bool = False,
    preview_output: Path | None = None,
    metadata_output: Path | None = None,
    prefs_file: Path | None = None,
    method_name: str = "set-data",
) -> None:
    """Ask MathType OLE to create an Equation.DSMT4 compound file without Word."""
    command = [
        str(HELPER_EXE),
        "--method",
        method_name,
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


def make_ole_from_mathtype_rust(
    input_path: Path,
    output_path: Path,
    mtef_output: Path,
    prefs_file: Path | None = None,
) -> None:
    """Generate MathType OLE and bare MTEF from LaTeX via the Rust converter."""
    rust_exe = build_mathtype_rust_converter()
    command = [
        str(rust_exe),
        "--input",
        str(input_path),
        "--output",
        str(output_path),
        "--mtef-output",
        str(mtef_output),
    ]
    if prefs_file is not None:
        command.extend(["--prefs-file", str(prefs_file)])
    run(command, stderr_as_warning=False)


def make_wmf_metadata_from_mtef(
    mtef_path: Path,
    helper_ole_output: Path,
    wmf_output: Path,
    metadata_output: Path,
    prefs_file: Path | None = None,
) -> None:
    """Generate WMF preview and metadata from bare MTEF via MathType SDK."""
    make_ole_from_format(
        "MathType EF",
        mtef_path,
        helper_ole_output,
        binary=True,
        preview_output=wmf_output,
        metadata_output=metadata_output,
        prefs_file=prefs_file,
        method_name="sdk-xform-ole",
    )


def make_ole_wmf_metadata_with_mathtype_rust(
    input_path: Path,
    ole_path: Path,
    wmf_path: Path,
    metadata_path: Path,
    mtef_path: Path,
    prefs_file: Path | None = None,
) -> None:
    """Generate OLE, WMF, and metadata through Rust MTEF conversion."""
    make_ole_from_mathtype_rust(input_path, ole_path, mtef_path, prefs_file=prefs_file)
    sdk_ole_path = ole_path.with_name(f"{ole_path.stem}.sdk{ole_path.suffix}")
    make_wmf_metadata_from_mtef(
        mtef_path,
        sdk_ole_path,
        wmf_path,
        metadata_path,
        prefs_file=prefs_file,
    )
    if sdk_ole_path.exists():
        sdk_ole_path.unlink()


def make_ole_wmf_metadata_with_mathtype_set_data(
    input_path: Path,
    ole_path: Path,
    wmf_path: Path,
    metadata_path: Path,
    prefs_file: Path | None = None,
) -> None:
    """Generate OLE, WMF, and metadata by importing TeX through MathType OLE."""
    make_ole_from_format(
        "TeX Input Language",
        input_path,
        ole_path,
        preview_output=wmf_path,
        metadata_output=metadata_path,
        prefs_file=prefs_file,
    )


def generate_uncached_equation_parts(
    index: int,
    input_path: Path,
    ole_path: Path,
    wmf_path: Path,
    metadata_path: Path,
    mtef_path: Path,
    prefs_file: Path | None = None,
    conversion_method: MathTypeConversionMethod = DEFAULT_MATHTYPE_CONVERSION_METHOD,
) -> None:
    """Generate MathType parts using the configured conversion backend."""
    if conversion_method == "both":
        raise ValueError("both conversion mode is only supported through generate_equation_parts")

    if conversion_method == "rust":
        make_ole_wmf_metadata_with_mathtype_rust(
            input_path,
            ole_path,
            wmf_path,
            metadata_path,
            mtef_path,
            prefs_file=prefs_file,
        )
        log_info(f"[mathtype] mathtype-rust conversion succeeded for equation {index}")
        return

    if conversion_method == "set-data":
        make_ole_wmf_metadata_with_mathtype_set_data(
            input_path,
            ole_path,
            wmf_path,
            metadata_path,
            prefs_file=prefs_file,
        )
        log_info(f"[mathtype] MathType TeX input conversion succeeded for equation {index}")
        return

    try:
        make_ole_wmf_metadata_with_mathtype_rust(
            input_path,
            ole_path,
            wmf_path,
            metadata_path,
            mtef_path,
            prefs_file=prefs_file,
        )
        log_info(f"[mathtype] mathtype-rust auto path succeeded for equation {index}")
    except (RuntimeError, FileNotFoundError):
        log_warning(
            f"[mathtype] mathtype-rust auto path failed for equation {index}; "
            "trying MathType TeX input fallback"
        )
        try:
            make_ole_wmf_metadata_with_mathtype_set_data(
                input_path,
                ole_path,
                wmf_path,
                metadata_path,
                prefs_file=prefs_file,
            )
        except RuntimeError as fallback_exc:
            raise RuntimeError(
                f"mathtype-rust auto path and MathType TeX input fallback both failed for equation {index}"
            ) from fallback_exc
        log_info(f"[mathtype] MathType TeX input fallback succeeded for equation {index}")


def inspect_ole(path: Path) -> CompoundFile:
    """Print the key MathType OLE stream evidence."""
    compound = CompoundFile(path.read_bytes())
    names = ", ".join(entry.name for entry in compound.entries if entry.name)
    native = compound.read_stream("Equation Native")
    log_debug(f"[mathtype] {path}: streams={names}")
    log_debug(f"[mathtype] Equation Native bytes={len(native)}, DSMT offset={native.find(b'DSMT')}")
    return compound


def iter_equation_requests_with_progress(
    requests: list[EquationRequest],
) -> Iterator[tuple[int, EquationRequest]]:
    """Yield MathType requests with a progress bar for slow COM conversion."""
    return tqdm(
        enumerate(requests, start=1),
        total=len(requests),
        desc="[mathtype] converting equations",
        unit="eq",
        dynamic_ncols=True,
        disable=None if should_log("INFO") else True,
    )


def generate_equation_parts(
    requests: list[EquationRequest],
    output_dir: Path,
    conversion_method: MathTypeConversionMethod = DEFAULT_MATHTYPE_CONVERSION_METHOD,
) -> list[GeneratedEquation]:
    """Generate OLE bins and WMF previews for all marker-bound formulas."""
    conversion_method = normalize_conversion_method(conversion_method)
    output_dir.mkdir(parents=True, exist_ok=True)
    equations: list[GeneratedEquation] = []
    prefs_template = MATHTYPE_DEFAULT_PREFS_TEMPLATE if MATHTYPE_DEFAULT_PREFS_TEMPLATE.exists() else None
    prefs_cache: dict[float, Path] = {}
    needs_variable_sizes = any(request.font_size_pt is not None for request in requests)
    cache_hits = 0
    cache_misses = 0
    helper_digest = file_sha256(HELPER_EXE)
    rust_source_digest = mathtype_rust_source_digest() if conversion_method != "set-data" else None
    rust_exe_digest = mathtype_rust_exe_digest_for_method(conversion_method)
    if needs_variable_sizes and prefs_template is None:
        log_warning(
            "[mathtype] warning: MathType preference template not found; "
            "equations will fall back to MathType's current default size"
        )

    for index, request in iter_equation_requests_with_progress(requests):
        latex = request.latex
        input_path = output_dir / f"eq_{index:03d}.tex"
        ole_path = output_dir / f"eq_{index:03d}.ole.bin"
        wmf_path = output_dir / f"eq_{index:03d}.wmf"
        metadata_path = output_dir / f"eq_{index:03d}.json"
        mtef_path = output_dir / f"eq_{index:03d}.mtef.bin"
        set_data_ole_path = output_dir / f"eq_{index:03d}.set-data.ole.bin"
        set_data_wmf_path = output_dir / f"eq_{index:03d}.set-data.wmf"
        set_data_metadata_path = output_dir / f"eq_{index:03d}.set-data.json"
        set_data_mtef_path = output_dir / f"eq_{index:03d}.set-data.mtef.bin"
        prefs_path: Path | None = None
        font_size_key = cache_font_size_key(request.font_size_pt, prefs_template)

        if font_size_key is not None and prefs_template is not None:
            prefs_path = prefs_cache.get(font_size_key)
            if prefs_path is None:
                prefs_path = output_dir / "prefs" / f"full-{format_font_size_pt(font_size_key).replace('.', '_')}pt.eqp"
                write_sized_prefs_file(prefs_template, prefs_path, font_size_key)
                prefs_cache[font_size_key] = prefs_path

        write_latex_input(input_path, latex)
        prefs_digest = file_sha256(prefs_path) if prefs_path is not None else None
        if conversion_method == "both":
            rust_hit = generate_cached_equation_parts_for_method(
                index,
                latex,
                font_size_key,
                prefs_digest,
                helper_digest,
                rust_source_digest,
                rust_exe_digest,
                input_path,
                ole_path,
                wmf_path,
                metadata_path,
                mtef_path,
                prefs_path,
                "rust",
            )
            set_data_hit = generate_cached_equation_parts_for_method(
                index,
                latex,
                font_size_key,
                prefs_digest,
                helper_digest,
                rust_source_digest,
                rust_exe_digest,
                input_path,
                set_data_ole_path,
                set_data_wmf_path,
                set_data_metadata_path,
                set_data_mtef_path,
                prefs_path,
                "set-data",
            )
            cache_hits += int(rust_hit) + int(set_data_hit)
            cache_misses += int(not rust_hit) + int(not set_data_hit)
            warn_if_conversion_outputs_differ(
                index,
                ole_path,
                metadata_path,
                set_data_ole_path,
                set_data_metadata_path,
            )
        elif conversion_method == "auto":
            hits, misses = generate_cached_equation_parts_auto(
                index,
                latex,
                font_size_key,
                prefs_digest,
                helper_digest,
                rust_source_digest,
                rust_exe_digest,
                input_path,
                ole_path,
                wmf_path,
                metadata_path,
                mtef_path,
                prefs_path,
            )
            cache_hits += hits
            cache_misses += misses
        else:
            hit = generate_cached_equation_parts_for_method(
                index,
                latex,
                font_size_key,
                prefs_digest,
                helper_digest,
                rust_source_digest,
                rust_exe_digest,
                input_path,
                ole_path,
                wmf_path,
                metadata_path,
                mtef_path,
                prefs_path,
                conversion_method=conversion_method,
            )
            cache_hits += int(hit)
            cache_misses += int(not hit)
        compound = inspect_ole(ole_path)
        if compound.read_stream("Equation Native").find(b"DSMT") < 0:
            raise ValueError(f"generated OLE lacks DSMT marker: {ole_path}")
        if not wmf_path.exists() or wmf_path.read_bytes()[:4] != bytes.fromhex("d7cdc69a"):
            raise ValueError(f"generated WMF preview is missing placeable header: {wmf_path}")
        equations.append(GeneratedEquation(latex=latex, ole_path=ole_path, wmf_path=wmf_path, metadata_path=metadata_path))
    log_info(f"[mathtype] cache summary: hits={cache_hits}, misses={cache_misses}, dir={MATHTYPE_CACHE_DIR}")
    return equations
