"""
Pandoc filter that converts local SVG image references to PNG for DOCX output.

This is useful for journal submission systems that reject SVG files even when
Word can display them. The filter rewrites only local ``.svg``/``.svgz`` image
URLs in the Pandoc AST and leaves the source Markdown unchanged.
"""

from __future__ import annotations

import hashlib
import json
import os
import shutil
import subprocess
import sys
from pathlib import Path
from urllib.parse import unquote, urlparse

import panflute as pf


SVG_SUFFIXES = {".svg", ".svgz"}
CACHE_METADATA_VERSION = 1
LOG_LEVELS = {"DEBUG": 10, "INFO": 20, "WARNING": 30, "WARN": 30, "ERROR": 40}
CONVERTED: set[Path] = set()
REUSED: set[Path] = set()
SKIPPED: set[str] = set()


def should_log(level: str) -> bool:
    """Return True when a filter message should be emitted."""
    configured = os.getenv("PANDOC_TEMPLATE_LOG_LEVEL", "INFO").strip().upper()
    return LOG_LEVELS.get(level, 30) >= LOG_LEVELS.get(configured, 20)


def log_info(message: str) -> None:
    """Emit filter progress at INFO level."""
    if should_log("INFO"):
        print(message, file=sys.stderr)


def log_debug(message: str) -> None:
    """Emit per-image details only when DEBUG logging is enabled."""
    if should_log("DEBUG"):
        print(message, file=sys.stderr)


def log_warning(message: str) -> None:
    """Emit a filter warning at the default log level."""
    if should_log("WARNING"):
        print(message, file=sys.stderr)


def parse_float_env(name: str, default: float) -> float:
    """Parse a positive float environment option with a safe fallback."""
    value = os.getenv(name)
    if value is None:
        return default
    try:
        parsed = float(value)
    except ValueError:
        log_warning(f"[WARN] Invalid {name}={value!r}; using {default}")
        return default
    if parsed <= 0:
        log_warning(f"[WARN] {name} must be positive; using {default}")
        return default
    return parsed


def path_from_url(url: str) -> str | None:
    """Return a local filesystem path string for a Pandoc image URL."""
    if not url:
        return None

    parsed = urlparse(url)
    scheme = parsed.scheme.lower()
    is_windows_drive = len(scheme) == 1 and len(url) > 2 and url[1:3] in {":\\", ":/"}
    if scheme and scheme != "file" and not is_windows_drive:
        return None

    path_text = parsed.path if scheme == "file" else url
    path_text = path_text.split("?", 1)[0].split("#", 1)[0]
    if not path_text:
        return None
    return unquote(path_text)


def is_svg_path(path_text: str) -> bool:
    """Return True when a local image path points to an SVG file."""
    return Path(path_text).suffix.lower() in SVG_SUFFIXES


def configured_base_dirs() -> list[Path]:
    """Return project-relative lookup roots used for resolving image paths."""
    raw_dirs = os.getenv("PMT_SVG_TO_PNG_BASE_DIRS", ".")
    dirs: list[Path] = []
    for raw_dir in raw_dirs.split(os.pathsep):
        if not raw_dir:
            continue
        path = Path(raw_dir).resolve()
        if path not in dirs:
            dirs.append(path)
    return dirs or [Path.cwd().resolve()]


def resolve_source_path(path_text: str, base_dirs: list[Path]) -> Path | None:
    """Resolve a local SVG reference against the project and manuscript dirs."""
    source = Path(path_text)
    if source.is_absolute():
        return source.resolve() if source.exists() else None

    for base_dir in base_dirs:
        candidate = (base_dir / source).resolve()
        if candidate.exists():
            return candidate
    return None


def output_path_for(source: Path, output_root: Path, base_dirs: list[Path]) -> Path:
    """Return a stable PNG cache path for one SVG source."""
    source = source.resolve()
    for base_dir in base_dirs:
        try:
            relative = source.relative_to(base_dir)
        except ValueError:
            continue
        return (output_root / relative).with_suffix(".png")

    # Outside-project absolute paths need a hash to avoid filename collisions.
    digest = hashlib.sha256(str(source).encode("utf-8")).hexdigest()[:12]
    return output_root / f"{source.stem}-{digest}.png"


def convert_with_resvg_py(source: Path, target: Path, dpi: float, scale: float) -> str:
    """Convert SVG to PNG with the pure package-managed resvg binding."""
    import resvg_py

    target.write_bytes(
        resvg_py.svg_to_bytes(
            svg_path=str(source),
            dpi=dpi,
            zoom=scale,
        )
    )
    return "resvg-py"


def convert_with_cairosvg(source: Path, target: Path, dpi: float, scale: float) -> str:
    """Convert SVG to PNG with CairoSVG's Python API."""
    import cairosvg

    cairosvg.svg2png(
        url=str(source),
        write_to=str(target),
        dpi=dpi,
        scale=scale,
    )
    return "CairoSVG"


def convert_with_rsvg(source: Path, target: Path, dpi: float) -> str:
    """Convert SVG to PNG with rsvg-convert when CairoSVG is unavailable."""
    executable = shutil.which("rsvg-convert")
    if executable is None:
        raise RuntimeError("rsvg-convert is not available on PATH")

    subprocess.run(
        [
            executable,
            "--format",
            "png",
            "--dpi-x",
            str(dpi),
            "--dpi-y",
            str(dpi),
            "--output",
            str(target),
            str(source),
        ],
        check=True,
        capture_output=True,
        text=True,
    )
    return "rsvg-convert"


def convert_svg_to_png(source: Path, target: Path, dpi: float, scale: float) -> str:
    """Convert one SVG file to PNG, preferring the package-managed binding."""
    target.parent.mkdir(parents=True, exist_ok=True)
    converter_errors: list[str] = []
    try:
        return convert_with_resvg_py(source, target, dpi, scale)
    except ImportError as exc:
        converter_errors.append(f"resvg-py import error: {exc}")
    except Exception as exc:
        converter_errors.append(f"resvg-py error: {exc}")

    try:
        return convert_with_cairosvg(source, target, dpi, scale)
    except ImportError as exc:
        converter_errors.append(f"CairoSVG import error: {exc}")
    except Exception as exc:
        # Fall back to rsvg-convert for environments where CairoSVG imports but
        # the platform Cairo runtime is missing or cannot render a specific SVG.
        converter_errors.append(f"CairoSVG error: {exc}")

    try:
        if scale != 1:
            log_warning("[WARN] docxSvgToPngScale is ignored when falling back to rsvg-convert")
        return convert_with_rsvg(source, target, dpi)
    except Exception as rsvg_error:
        converter_errors.append(f"rsvg-convert error: {rsvg_error}")
        raise RuntimeError(
            "Could not convert SVG to PNG. Install the Python dependency "
            "`resvg-py`, or make `rsvg-convert` available on PATH. "
            + "; ".join(converter_errors)
        ) from rsvg_error


def cache_metadata_path(target: Path) -> Path:
    """Return the sidecar metadata path used to validate a generated PNG."""
    return target.with_suffix(target.suffix + ".meta.json")


def expected_cache_metadata(source: Path, dpi: float, scale: float) -> dict[str, object]:
    """Build cache metadata that changes when source content or options change."""
    stat_result = source.stat()
    return {
        "version": CACHE_METADATA_VERSION,
        "source": str(source.resolve()),
        "source_mtime_ns": stat_result.st_mtime_ns,
        "source_size": stat_result.st_size,
        "dpi": dpi,
        "scale": scale,
    }


def cache_metadata_matches(target: Path, expected: dict[str, object]) -> bool:
    """Return True only when the PNG and its sidecar metadata are current."""
    metadata_path = cache_metadata_path(target)
    if not target.exists() or not metadata_path.exists():
        return False
    try:
        actual = json.loads(metadata_path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        log_debug(f"[svg-to-png] Ignoring stale cache metadata {metadata_path}: {exc}")
        return False
    return all(actual.get(key) == value for key, value in expected.items())


def write_cache_metadata(target: Path, metadata: dict[str, object], converter: str) -> None:
    """Write sidecar metadata so option changes invalidate old PNG caches."""
    payload = {**metadata, "converter": converter}
    cache_metadata_path(target).write_text(
        json.dumps(payload, ensure_ascii=False, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )


def ensure_png(source: Path, target: Path, dpi: float, scale: float) -> Path:
    """Create or reuse the PNG cache file for one SVG source."""
    expected_metadata = expected_cache_metadata(source, dpi, scale)
    if cache_metadata_matches(target, expected_metadata):
        REUSED.add(target)
        log_debug(f"[svg-to-png] Reusing {target}")
        return target

    converter = convert_svg_to_png(source, target, dpi, scale)
    write_cache_metadata(target, expected_metadata, converter)
    CONVERTED.add(target)
    log_debug(f"[svg-to-png] Converted {source} -> {target} with {converter}")
    return target


def rewrite_image(elem: pf.Image, base_dirs: list[Path], output_root: Path, dpi: float, scale: float) -> pf.Image | None:
    """Rewrite one local SVG image URL to its generated PNG path."""
    path_text = path_from_url(elem.url)
    if path_text is None or not is_svg_path(path_text):
        return None

    source = resolve_source_path(path_text, base_dirs)
    if source is None:
        SKIPPED.add(elem.url)
        log_warning(f"[WARN] SVG image not found, leaving unchanged: {elem.url}")
        return None

    target = ensure_png(source, output_path_for(source, output_root, base_dirs), dpi, scale)
    elem.url = target.as_posix()
    return elem


def action(elem: pf.Element, doc: pf.Doc) -> pf.Element | None:
    """Panflute action that converts and rewrites SVG image elements."""
    if not isinstance(elem, pf.Image):
        return None
    return rewrite_image(
        elem,
        doc.pmt_svg_base_dirs,
        doc.pmt_svg_output_root,
        doc.pmt_svg_dpi,
        doc.pmt_svg_scale,
    )


def prepare(doc: pf.Doc) -> None:
    """Load conversion settings from the build process environment."""
    doc.pmt_svg_base_dirs = configured_base_dirs()
    doc.pmt_svg_output_root = Path(os.getenv("PMT_SVG_TO_PNG_DIR", "tmp/svg-png")).resolve()
    doc.pmt_svg_dpi = parse_float_env("PMT_SVG_TO_PNG_DPI", 300)
    doc.pmt_svg_scale = parse_float_env("PMT_SVG_TO_PNG_SCALE", 1)


def finalize(doc: pf.Doc) -> None:
    """Report an INFO-level summary after all images have been inspected."""
    total = len(CONVERTED) + len(REUSED)
    if total:
        log_info(f"[svg-to-png] SVG image PNG cache ready: {total} file(s)")
    if SKIPPED:
        log_warning(f"[WARN] SVG image conversion skipped for {len(SKIPPED)} missing file(s)")


def main(doc: pf.Doc | None = None) -> pf.Doc:
    """Run the SVG-to-PNG Pandoc filter."""
    return pf.run_filter(action, prepare=prepare, finalize=finalize, doc=doc)


if __name__ == "__main__":
    main()
