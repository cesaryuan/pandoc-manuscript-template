"""Generate MathType OLE bins, WMF previews, and placement metadata."""

import hashlib
import json
import locale
import os
import platform
import re
import shutil
import struct
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


HELPER_EXE = resource_path("mathtype/ole_helper/bin/Release/net48/MathTypeOleHelper.exe")
MATHTYPE_RUST_SOURCE_EXE_NAME = "mathtype-rust.exe" if os.name == "nt" else "mathtype-rust"
MATHTYPE_RUST_PACKAGE_EXE_NAME = MATHTYPE_RUST_SOURCE_EXE_NAME
MATHTYPE_RUST_PACKAGE_EXE = resource_path(Path("mathtype/bin") / MATHTYPE_RUST_PACKAGE_EXE_NAME)
LATEX2WMF_SOURCE_EXE_NAME = "latex2wmf.exe" if os.name == "nt" else "latex2wmf"
LATEX2WMF_PACKAGE_EXE_NAME = LATEX2WMF_SOURCE_EXE_NAME
LATEX2WMF_PACKAGE_EXE = resource_path(Path("mathtype/bin") / LATEX2WMF_PACKAGE_EXE_NAME)
MATHTYPE_PROG_ID = "Equation.DSMT4"
MATHTYPE_MT6_RELATIVE_PATHS = (
    Path("System/64/MT6.dll"),
    Path("System/32/MT6.dll"),
    Path("MT6.dll"),
)
# Keep the sizing template in-repo so builds do not depend on a local MathType preferences path.
MATHTYPE_DEFAULT_PREFS_TEMPLATE = resource_path("mathtype/Times+Symbol 12.eqp")
MATHTYPE_CACHE_DIR = PMT_MATHTYPE_CACHE_DIR
MATHTYPE_CACHE_VERSION = 4
PLACEABLE_WMF_KEY_BYTES = bytes.fromhex("d7cdc69a")
PLACEABLE_WMF_HEADER_SIZE = 22
WMF_HEADER_SIZE = 18
WMF_META_EOF = 0x0000
WMF_META_SETWINDOWORG = 0x020B
WMF_META_SETWINDOWEXT = 0x020C
BEGIN_ALIGNED_RE = re.compile(r"\\begin\s*\{\s*aligned\s*\}")
END_ALIGNED_RE = re.compile(r"\\end\s*\{\s*aligned\s*\}")
MathTypeSingleConversionMethod = Literal["rust", "rust-sdk", "set-data"]
MathTypeConversionMethod = Literal["rust", "rust-sdk", "set-data", "auto", "both"]
MathTypeSvgBackend = Literal["ratex", "typst"]
MathTypeMathStyle = Literal["inline", "display"]
DEFAULT_MATHTYPE_CONVERSION_METHOD: MathTypeConversionMethod = "rust"
DEFAULT_MATHTYPE_SVG_BACKEND: MathTypeSvgBackend = "ratex"


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
LATEX2WMF_PROJECT = source_tree_path("scripts/latex2wmf/Cargo.toml")
LATEX2WMF_SOURCE_EXE = source_tree_path(Path("scripts/latex2wmf/target/debug") / LATEX2WMF_SOURCE_EXE_NAME)
LATEX2WMF_EXE = LATEX2WMF_SOURCE_EXE or LATEX2WMF_PACKAGE_EXE
MATHTYPE_RUST_BUILD_CHECKED = False
LATEX2WMF_BUILD_CHECKED = False


def normalize_conversion_method(value: object | None) -> MathTypeConversionMethod:
    """Normalize style metadata for MathType equation conversion backends."""
    if value is None:
        return DEFAULT_MATHTYPE_CONVERSION_METHOD
    text = str(value).strip().casefold().replace("_", "-")
    aliases: dict[str, MathTypeConversionMethod] = {
        "rust": "rust",
        "mathtype-rust": "rust",
        "mtef": "rust",
        "rust-sdk": "rust-sdk",
        "sdk": "rust-sdk",
        "sdk-xform-ole": "rust-sdk",
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
        allowed = "rust, rust-sdk, set-data, auto, both"
        raise ValueError(f"mathtypeConversionMethod must be one of: {allowed}; got {value!r}")
    return method


def normalize_svg_backend(value: object | None) -> MathTypeSvgBackend:
    """Normalize the cross-platform LaTeX-to-SVG renderer selection."""
    if value is None:
        return DEFAULT_MATHTYPE_SVG_BACKEND
    text = str(value).strip().casefold().replace("_", "-")
    aliases: dict[str, MathTypeSvgBackend] = {
        "ratex": "ratex",
        "typst": "typst",
        "typst-as-lib": "typst",
    }
    backend = aliases.get(text)
    if backend is None:
        raise ValueError(f"mathtypeSvgBackend must be one of: ratex, typst; got {value!r}")
    return backend


@dataclass(frozen=True)
class EquationRequest:
    """MathType generation inputs extracted from a marker-bound DOCX formula."""

    latex: str
    font_size_pt: float | None = None
    math_style: MathTypeMathStyle = "display"


@dataclass
class GeneratedEquation:
    """Generated MathType object parts for a single DOCX math node."""

    latex: str
    ole_path: Path
    wmf_path: Path
    metadata_path: Path | None = None
    math_style: MathTypeMathStyle = "display"

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


def check_native_converter_availability(
    converters: tuple[tuple[str, Path | None, Path], ...],
) -> MathTypeAvailability:
    """Check whether the requested native converters can be built or executed."""
    reasons: list[str] = []
    details: list[str] = []
    for name, project, executable in converters:
        if project is not None and project.exists():
            details.append(f"{name} source project found: {project}.")
            if shutil.which("cargo") is None and not executable.exists():
                reasons.append(
                    f"{name} requires `cargo`, or a prebuilt executable at {executable}."
                )
        elif executable.exists():
            details.append(f"Packaged {name} executable found: {executable}.")
        else:
            reasons.append(
                f"{name} executable is missing: {executable}. Install a platform wheel that bundles it, "
                "or run from a source checkout with Cargo available."
            )
    return MathTypeAvailability(tuple(reasons), tuple(details))


def check_cross_platform_mathtype_availability() -> MathTypeAvailability:
    """Check the two Rust executables used by the cross-platform backend."""
    return check_native_converter_availability(
        (
            ("mathtype-rust", MATHTYPE_RUST_PROJECT, MATHTYPE_RUST_EXE),
            ("latex2wmf", LATEX2WMF_PROJECT, LATEX2WMF_EXE),
        )
    )


def check_mathtype_availability(
    conversion_method: object | None = None,
) -> MathTypeAvailability:
    """Check the native backend and, when required, MathType OLE tooling.

    This is intentionally a lightweight preflight for build.py. It catches the
    common hard failures before Pandoc does any work, while the actual converter
    still performs the real OLE generation for each equation.
    """
    method = normalize_conversion_method(conversion_method)
    native = check_cross_platform_mathtype_availability()
    if method == "rust":
        return native
    if method == "auto" and native.usable:
        return MathTypeAvailability(
            (),
            (*native.details, "Auto mode can use the cross-platform Rust fallback without MathType."),
        )

    needs_cross_platform = method == "both"
    if method == "rust-sdk":
        native = check_native_converter_availability(
            (("mathtype-rust", MATHTYPE_RUST_PROJECT, MATHTYPE_RUST_EXE),)
        )
        needs_cross_platform = True
    reasons: list[str] = list(native.reasons) if needs_cross_platform else []
    details: list[str] = list(native.details) if needs_cross_platform else []

    system = platform.system()
    if system != "Windows":
        reasons.append(
            "The selected MathType SDK path requires Windows because it uses COM/OLE "
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

    helper_exists = HELPER_EXE.exists()
    if helper_exists:
        details.append(f"MathType OLE helper executable found: {HELPER_EXE}.")
    else:
        reasons.append(
            f"MathType OLE helper executable is missing: {HELPER_EXE}. "
            "Install a Windows wheel that bundles it or build it explicitly before running pmt."
        )

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
    stdout_as_debug: bool = False,
    stderr_as_debug: bool = False,
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
        if stdout_as_debug:
            log_debug(stdout.strip())
        else:
            log_info(stdout.strip())
    if stderr.strip():
        if result.returncode == 0 and stderr_as_debug:
            log_debug(stderr.strip())
        elif stderr_as_warning or result.returncode != 0:
            log_warning(stderr.strip())
        else:
            log_info(stderr.strip())
    if result.returncode != 0:
        raise RuntimeError(f"command failed: {' '.join(command)}")
    return subprocess.CompletedProcess(command, result.returncode, stdout=stdout, stderr=stderr)


def require_helper_executable() -> Path:
    """Return the prebuilt MathType helper without compiling during conversion."""
    if HELPER_EXE.exists():
        log_debug(f"[mathtype] helper executable found: {HELPER_EXE}")
        return HELPER_EXE
    raise FileNotFoundError(
        f"MathType OLE helper executable is missing: {HELPER_EXE}. "
        "Install a Windows wheel that bundles it or build the helper explicitly before running pmt."
    )


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


def build_latex2wmf_converter() -> Path:
    """Ensure the cross-platform SVG-to-WMF converter exists."""
    global LATEX2WMF_BUILD_CHECKED
    if LATEX2WMF_PROJECT is not None and LATEX2WMF_PROJECT.exists():
        if shutil.which("cargo") is None:
            if LATEX2WMF_EXE.exists():
                log_warning(
                    "[mathtype] warning: cargo not found; using existing latex2wmf executable "
                    f"without rebuilding: {LATEX2WMF_EXE}"
                )
                return LATEX2WMF_EXE
            raise RuntimeError(
                f"latex2wmf requires `cargo`, or a prebuilt executable at {LATEX2WMF_EXE}"
            )
        if not LATEX2WMF_BUILD_CHECKED:
            log_info(f"[mathtype] building latex2wmf converter: {LATEX2WMF_PROJECT}")
            run(
                ["cargo", "build", "--manifest-path", str(LATEX2WMF_PROJECT)],
                stderr_as_warning=False,
            )
            LATEX2WMF_BUILD_CHECKED = True
        if not LATEX2WMF_EXE.exists():
            raise FileNotFoundError(f"Cargo build did not create expected executable: {LATEX2WMF_EXE}")
        return LATEX2WMF_EXE

    if LATEX2WMF_EXE.exists():
        log_debug(f"[mathtype] latex2wmf executable found: {LATEX2WMF_EXE}")
        return LATEX2WMF_EXE
    raise FileNotFoundError(
        f"latex2wmf executable is missing: {LATEX2WMF_EXE}. Install a platform wheel that bundles "
        "latex2wmf, or run from a source checkout with scripts/latex2wmf."
    )


def native_exe_digest_for_method(conversion_method: MathTypeConversionMethod) -> str | None:
    """Build and hash the native converters used by the selected backend."""
    if conversion_method == "set-data":
        return None
    if conversion_method == "rust-sdk":
        return file_sha256(build_mathtype_rust_converter())
    if conversion_method == "auto":
        try:
            return file_group_sha256(
                [build_mathtype_rust_converter(), build_latex2wmf_converter()]
            )
        except (RuntimeError, FileNotFoundError):
            return None
    return file_group_sha256(
        [build_mathtype_rust_converter(), build_latex2wmf_converter()]
    )


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


def native_source_digest_for_method(
    conversion_method: MathTypeConversionMethod,
) -> str | None:
    """Return a digest for native sources used by the selected backend."""
    if conversion_method == "set-data":
        return None
    selected = (
        (MATHTYPE_RUST_PROJECT,)
        if conversion_method == "rust-sdk"
        else (MATHTYPE_RUST_PROJECT, LATEX2WMF_PROJECT)
    )
    projects = [project for project in selected if project is not None]
    if not projects:
        return None
    paths: list[Path] = []
    for project in projects:
        project_dir = project.parent
        paths.extend([project, project_dir / "Cargo.lock"])
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
    """Return the half-point size used by native previews and optional preferences."""
    if font_size_pt is None:
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
    svg_backend: MathTypeSvgBackend = DEFAULT_MATHTYPE_SVG_BACKEND,
    math_style: MathTypeMathStyle = "display",
) -> str:
    """Build a stable cache key from the exact MathType inputs.

    The key uses the normalized TeX payload and generated preference contents,
    because those are the values passed to the MathType OLE helper. Rust
    converter digests are included so generated parts do not outlive the
    converter implementation or backend selection that produced them.
    """
    svg_backend_key: str | None = svg_backend
    math_style_key: str | None = math_style
    if conversion_method == "set-data":
        rust_source_digest = None
        rust_exe_digest = None
        svg_backend_key = None
        math_style_key = None
    elif conversion_method == "rust-sdk":
        svg_backend_key = None
        math_style_key = None
    else:
        # The cross-platform Rust path does not execute the MathType helper.
        helper_digest = None

    payload = {
        "version": MATHTYPE_CACHE_VERSION,
        "tex_payload": mathtype_tex_payload(latex),
        "font_size_pt": font_size_key,
        "prefs_sha256": prefs_digest,
        "helper_sha256": helper_digest,
        "mathtype_rust_source_sha256": rust_source_digest,
        "mathtype_rust_exe_sha256": rust_exe_digest,
        "conversion_method": conversion_method,
        "svg_backend": svg_backend_key,
        "math_style": math_style_key,
    }
    data = json.dumps(payload, ensure_ascii=True, sort_keys=True, separators=(",", ":")).encode("utf-8")
    return hashlib.sha256(data).hexdigest()


def cache_paths(cache_key: str) -> tuple[Path, Path, Path]:
    """Return the cache file paths for one generated MathType equation."""
    folder = MATHTYPE_CACHE_DIR / cache_key[:2] / cache_key
    return folder / "equation.ole.bin", folder / "preview.wmf", folder / "metadata.json"


def has_placeable_wmf_header(wmf_bytes: bytes) -> bool:
    """Return True when bytes start with an Aldus placeable WMF header."""
    return wmf_bytes.startswith(PLACEABLE_WMF_KEY_BYTES)


def wmf_has_window_mapping(wmf_bytes: bytes) -> bool:
    """Return True when a WMF record stream defines a replay window.

    MathType SDK previews can display in Word but export blank PDFs when these
    records are missing, because Word's PDF exporter replays the WMF records
    without relying only on the placeable header bounds.
    """
    offset = PLACEABLE_WMF_HEADER_SIZE if has_placeable_wmf_header(wmf_bytes) else 0
    if len(wmf_bytes) < offset + WMF_HEADER_SIZE:
        return False

    seen_org = False
    seen_ext = False
    position = offset + WMF_HEADER_SIZE
    while position + 6 <= len(wmf_bytes):
        size_words, function = struct.unpack_from("<IH", wmf_bytes, position)
        if size_words < 3:
            return False
        next_position = position + size_words * 2
        if next_position > len(wmf_bytes):
            return False
        if function == WMF_META_SETWINDOWORG:
            seen_org = True
        elif function == WMF_META_SETWINDOWEXT:
            seen_ext = True
        elif function == WMF_META_EOF:
            break
        position = next_position
    return seen_org and seen_ext


def valid_cached_parts(ole_path: Path, wmf_path: Path) -> bool:
    """Return True when cached MathType OLE and WMF files look usable."""
    if not ole_path.exists() or not wmf_path.exists():
        return False
    try:
        compound = CompoundFile(ole_path.read_bytes())
        has_mathtype_payload = compound.read_stream("Equation Native").find(b"DSMT") >= 0
        wmf_bytes = wmf_path.read_bytes()
        has_placeable_preview = has_placeable_wmf_header(wmf_bytes)
        return has_mathtype_payload and has_placeable_preview and wmf_has_window_mapping(wmf_bytes)
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
    svg_backend: MathTypeSvgBackend = DEFAULT_MATHTYPE_SVG_BACKEND,
    math_style: MathTypeMathStyle = "display",
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
        svg_backend,
        math_style,
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
        svg_backend=svg_backend,
        font_size_pt=font_size_key,
        math_style=math_style,
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
    svg_backend: MathTypeSvgBackend = DEFAULT_MATHTYPE_SVG_BACKEND,
    math_style: MathTypeMathStyle = "display",
) -> tuple[int, int]:
    """Try the preferred set-data cache/generator first, then rust fallback."""
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
            "set-data",
            svg_backend,
            math_style,
        )
        return int(hit), int(not hit)
    except (RuntimeError, FileNotFoundError):
        log_warning(
            f"[mathtype] MathType TeX input auto path failed for equation {index}; "
            "trying mathtype-rust fallback"
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
            "rust",
            svg_backend,
            math_style,
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
            f"{', '.join(differing_parts)}; using set-data output"
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
    helper_exe = require_helper_executable()
    command = [
        str(helper_exe),
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
    run(command, stderr_as_warning=False, stdout_as_debug=True, stderr_as_debug=True)


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


def make_wmf_metadata_cross_platform(
    input_path: Path,
    wmf_output: Path,
    metadata_output: Path,
    svg_backend: MathTypeSvgBackend = DEFAULT_MATHTYPE_SVG_BACKEND,
    font_size_pt: float | None = None,
    math_style: MathTypeMathStyle = "display",
) -> None:
    """Generate formula WMF and placement JSON without MathType or Windows."""
    executable = build_latex2wmf_converter()
    command = [
        str(executable),
        "--input",
        str(input_path),
        "--output",
        str(wmf_output),
        "--metadata-output",
        str(metadata_output),
        "--svg-backend",
        svg_backend,
        "--math-style",
        math_style,
        "--font-size",
        format_font_size_pt(font_size_pt or 12.0),
    ]
    run(command, stderr_as_warning=False, stdout_as_debug=True, stderr_as_debug=True)


def make_ole_wmf_metadata_with_mathtype_rust(
    input_path: Path,
    ole_path: Path,
    wmf_path: Path,
    metadata_path: Path,
    mtef_path: Path,
    prefs_file: Path | None = None,
    svg_backend: MathTypeSvgBackend = DEFAULT_MATHTYPE_SVG_BACKEND,
    font_size_pt: float | None = None,
    math_style: MathTypeMathStyle = "display",
) -> None:
    """Generate OLE, WMF, and metadata through cross-platform Rust tools."""
    make_ole_from_mathtype_rust(input_path, ole_path, mtef_path, prefs_file=prefs_file)
    make_wmf_metadata_cross_platform(
        input_path,
        wmf_path,
        metadata_path,
        svg_backend=svg_backend,
        font_size_pt=font_size_pt,
        math_style=math_style,
    )


def make_ole_wmf_metadata_with_mathtype_rust_sdk(
    input_path: Path,
    ole_path: Path,
    wmf_path: Path,
    metadata_path: Path,
    mtef_path: Path,
    prefs_file: Path | None = None,
) -> None:
    """Generate Rust OLE/MTEF, then use the MathType SDK for WMF and JSON."""
    make_ole_from_mathtype_rust(input_path, ole_path, mtef_path, prefs_file=prefs_file)
    sdk_ole_path = ole_path.with_name(f"{ole_path.stem}.sdk{ole_path.suffix}")
    try:
        make_wmf_metadata_from_mtef(
            mtef_path,
            sdk_ole_path,
            wmf_path,
            metadata_path,
            prefs_file=prefs_file,
        )
    finally:
        # The helper's OLE wraps the same MTEF only to activate the SDK preview.
        sdk_ole_path.unlink(missing_ok=True)


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
    svg_backend: MathTypeSvgBackend = DEFAULT_MATHTYPE_SVG_BACKEND,
    font_size_pt: float | None = None,
    math_style: MathTypeMathStyle = "display",
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
            svg_backend=svg_backend,
            font_size_pt=font_size_pt,
            math_style=math_style,
        )
        log_debug(f"[mathtype] mathtype-rust conversion succeeded for equation {index}")
        return

    if conversion_method == "rust-sdk":
        make_ole_wmf_metadata_with_mathtype_rust_sdk(
            input_path,
            ole_path,
            wmf_path,
            metadata_path,
            mtef_path,
            prefs_file=prefs_file,
        )
        log_debug(f"[mathtype] mathtype-rust + SDK conversion succeeded for equation {index}")
        return

    if conversion_method == "set-data":
        make_ole_wmf_metadata_with_mathtype_set_data(
            input_path,
            ole_path,
            wmf_path,
            metadata_path,
            prefs_file=prefs_file,
        )
        log_debug(f"[mathtype] MathType TeX input conversion succeeded for equation {index}")
        return

    try:
        make_ole_wmf_metadata_with_mathtype_set_data(
            input_path,
            ole_path,
            wmf_path,
            metadata_path,
            prefs_file=prefs_file,
        )
        log_debug(f"[mathtype] MathType TeX input auto path succeeded for equation {index}")
    except (RuntimeError, FileNotFoundError):
        log_warning(
            f"[mathtype] MathType TeX input auto path failed for equation {index}; "
            "trying mathtype-rust fallback"
        )
        try:
            make_ole_wmf_metadata_with_mathtype_rust(
                input_path,
                ole_path,
                wmf_path,
                metadata_path,
                mtef_path,
                prefs_file=prefs_file,
                svg_backend=svg_backend,
                font_size_pt=font_size_pt,
                math_style=math_style,
            )
        except (RuntimeError, FileNotFoundError) as fallback_exc:
            raise RuntimeError(
                f"MathType TeX input auto path and mathtype-rust fallback both failed for equation {index}"
            ) from fallback_exc
        log_debug(f"[mathtype] mathtype-rust fallback succeeded for equation {index}")


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
    svg_backend: MathTypeSvgBackend = DEFAULT_MATHTYPE_SVG_BACKEND,
) -> list[GeneratedEquation]:
    """Generate OLE bins and WMF previews for all marker-bound formulas."""
    conversion_method = normalize_conversion_method(conversion_method)
    svg_backend = normalize_svg_backend(svg_backend)
    output_dir.mkdir(parents=True, exist_ok=True)
    equations: list[GeneratedEquation] = []
    prefs_template = MATHTYPE_DEFAULT_PREFS_TEMPLATE if MATHTYPE_DEFAULT_PREFS_TEMPLATE.exists() else None
    prefs_cache: dict[float, Path] = {}
    needs_variable_sizes = any(request.font_size_pt is not None for request in requests)
    cache_hits = 0
    cache_misses = 0
    helper_digest = (
        file_sha256(HELPER_EXE)
        if conversion_method in {"rust-sdk", "set-data", "auto", "both"}
        else None
    )
    rust_source_digest = native_source_digest_for_method(conversion_method)
    rust_exe_digest = native_exe_digest_for_method(conversion_method)
    if needs_variable_sizes and prefs_template is None:
        log_warning(
            "[mathtype] warning: MathType preference template not found; "
            "equations will fall back to MathType's current default size"
        )

    for index, request in iter_equation_requests_with_progress(requests):
        latex = request.latex
        math_style = request.math_style
        input_path = output_dir / f"eq_{index:03d}.tex"
        ole_path = output_dir / f"eq_{index:03d}.ole.bin"
        wmf_path = output_dir / f"eq_{index:03d}.wmf"
        metadata_path = output_dir / f"eq_{index:03d}.json"
        mtef_path = output_dir / f"eq_{index:03d}.mtef.bin"
        rust_ole_path = output_dir / f"eq_{index:03d}.rust.ole.bin"
        rust_wmf_path = output_dir / f"eq_{index:03d}.rust.wmf"
        rust_metadata_path = output_dir / f"eq_{index:03d}.rust.json"
        rust_mtef_path = output_dir / f"eq_{index:03d}.rust.mtef.bin"
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
                rust_ole_path,
                rust_wmf_path,
                rust_metadata_path,
                rust_mtef_path,
                prefs_path,
                "rust",
                svg_backend,
                math_style,
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
                ole_path,
                wmf_path,
                metadata_path,
                mtef_path,
                prefs_path,
                "set-data",
                svg_backend,
                math_style,
            )
            cache_hits += int(rust_hit) + int(set_data_hit)
            cache_misses += int(not rust_hit) + int(not set_data_hit)
            warn_if_conversion_outputs_differ(
                index,
                rust_ole_path,
                rust_metadata_path,
                ole_path,
                metadata_path,
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
                svg_backend,
                math_style,
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
                svg_backend=svg_backend,
                math_style=math_style,
            )
            cache_hits += int(hit)
            cache_misses += int(not hit)
        compound = inspect_ole(ole_path)
        if compound.read_stream("Equation Native").find(b"DSMT") < 0:
            raise ValueError(f"generated OLE lacks DSMT marker: {ole_path}")
        wmf_bytes = wmf_path.read_bytes() if wmf_path.exists() else b""
        if not has_placeable_wmf_header(wmf_bytes):
            raise ValueError(f"generated WMF preview is missing placeable header: {wmf_path}")
        if not wmf_has_window_mapping(wmf_bytes):
            raise ValueError(f"generated WMF preview is missing window mapping records: {wmf_path}")
        equations.append(
            GeneratedEquation(
                latex=latex,
                ole_path=ole_path,
                wmf_path=wmf_path,
                metadata_path=metadata_path,
                math_style=math_style,
            )
        )
    log_info(f"[mathtype] cache summary: hits={cache_hits}, misses={cache_misses}, dir={MATHTYPE_CACHE_DIR}")
    return equations
