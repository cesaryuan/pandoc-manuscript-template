"""Cross-platform LaTeX -> WMF equation previews without MathType.

The Windows MathType path renders equation previews through the MathType SDK
(``MT6.dll`` via COM). That engine only exists on Windows, so on macOS/Linux we
render the preview ourselves: LaTeX -> SVG (MathJax today; a real TeX + dvisvgm
front-end plugs in when available) -> WMF (LibreOffice ``soffice``). LibreOffice
already emits an Aldus *placeable* WMF with window-mapping records, so the output
passes the same preview checks the SDK output does (see ``ole_parts``).

The editable equation body (OLE ``Equation Native``/MTEF stream) is still produced
byte-for-byte by ``mathtype-rust``; this module only supplies the preview picture
and the size/baseline metadata that Word uses to place it.
"""

from __future__ import annotations

import hashlib
import json
import re
import shutil
import struct
import subprocess
from dataclasses import dataclass
from pathlib import Path

from ..runtime.logging import log_debug
from ..runtime.paths import PMT_MATHTYPE_CACHE_DIR
from ..runtime.resources import package_resource_path

# MathType's in-repo sizing template is "Times+Symbol 12"; use 12pt as the em
# size when a formula has no resolved Word font size.
DEFAULT_EM_PT = 12.0
JS_DIR = package_resource_path("mathtype/js")
TEX2SVG_SCRIPT = JS_DIR / "tex2svg.mjs"
MATHJAX_NODE_DIR = PMT_MATHTYPE_CACHE_DIR / "mathjax-node"
SOFFICE_MACOS_PATH = Path("/Applications/LibreOffice.app/Contents/MacOS/soffice")

# WMF preview geometry: 1 point = 20 twips, and a placeable WMF at 1440
# units/inch stores its bounds in signed 16-bit words.
TWIPS_PER_PT = 20
WMF_UNITS_PER_INCH = 1440
# Raster fallback: render the bitmap at ~3x the on-screen size (~288 dpi) so the
# embedded picture stays crisp when the metafile path is unavailable.
RASTER_FALLBACK_SCALE = 3.0


class MetafileExportError(RuntimeError):
    """LibreOffice could not export an SVG to a WMF/EMF metafile.

    Happens on rare, very complex equations that exceed LibreOffice's metafile
    exporter; the caller falls back to a rasterized bitmap wrapped in a WMF.
    """


@dataclass(frozen=True)
class PreviewMetrics:
    """Point-size metrics derived from the rendered SVG."""

    width_pt: float
    height_pt: float
    baseline_from_bottom_pt: float


def find_node() -> str | None:
    """Return the node executable path, or None when node is unavailable."""
    return shutil.which("node")


def find_npm() -> str | None:
    """Return the npm executable path, or None when npm is unavailable."""
    return shutil.which("npm")


def find_soffice() -> str | None:
    """Return the LibreOffice ``soffice`` path across platforms, or None."""
    found = shutil.which("soffice") or shutil.which("libreoffice")
    if found:
        return found
    if SOFFICE_MACOS_PATH.exists():
        return str(SOFFICE_MACOS_PATH)
    return None


def find_dvisvgm() -> str | None:
    """Return the dvisvgm path for the optional real-LaTeX front-end, or None."""
    return shutil.which("dvisvgm")


def find_tex() -> str | None:
    """Return a DVI-producing LaTeX engine for the optional front-end, or None."""
    return shutil.which("latex") or shutil.which("pdflatex")


def _run(command: list[str], *, cwd: Path | None = None, env: dict[str, str] | None = None) -> subprocess.CompletedProcess[str]:
    """Run a command capturing text output; raise with stderr on failure."""
    result = subprocess.run(
        command,
        cwd=str(cwd) if cwd is not None else None,
        env=env,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if result.returncode != 0:
        detail = (result.stderr or result.stdout or "").strip()
        raise RuntimeError(f"command failed ({result.returncode}): {' '.join(command)}\n{detail}")
    return result


def ensure_renderer_installed() -> Path:
    """Install the bundled renderer + MathJax once into a writable cache dir.

    The package directory can be read-only (installed wheel), and ESM ``import``
    resolves ``node_modules`` relative to the script's own location (it ignores
    ``NODE_PATH``). So the script and its dependencies must live together: copy
    the tiny script into the cache dir and ``npm install`` beside it.
    """
    MATHJAX_NODE_DIR.mkdir(parents=True, exist_ok=True)
    script = MATHJAX_NODE_DIR / TEX2SVG_SCRIPT.name
    shutil.copy2(TEX2SVG_SCRIPT, script)
    shutil.copy2(JS_DIR / "package.json", MATHJAX_NODE_DIR / "package.json")

    node_modules = MATHJAX_NODE_DIR / "node_modules"
    if not (node_modules / "mathjax-full").is_dir():
        npm = find_npm()
        if npm is None:
            raise RuntimeError("npm is required to install the MathJax renderer but was not found on PATH")
        log_debug(f"[mathtype] installing MathJax renderer into {MATHJAX_NODE_DIR}")
        _run([npm, "install", "--no-audit", "--no-fund", "--loglevel=error"], cwd=MATHJAX_NODE_DIR)
        if not (node_modules / "mathjax-full").is_dir():
            raise RuntimeError(f"npm install did not create the MathJax dependency in {node_modules}")
    return script


def render_latex_to_svg(latex: str, svg_path: Path, em_pt: float) -> PreviewMetrics:
    """Render one LaTeX payload to an SVG sized in points; return its metrics."""
    node = find_node()
    if node is None:
        raise RuntimeError("node is required to render equation previews but was not found on PATH")
    script = ensure_renderer_installed()

    svg_path.parent.mkdir(parents=True, exist_ok=True)
    input_path = svg_path.with_suffix(".tex.txt")
    input_path.write_text(latex, encoding="utf-8")

    result = _run(
        [node, str(script), "--input", str(input_path), "--em-pt", f"{em_pt:g}"],
    )
    svg_path.write_text(result.stdout, encoding="utf-8")

    try:
        metrics = json.loads(result.stderr.strip().splitlines()[-1])
    except (ValueError, IndexError) as exc:
        raise RuntimeError(f"could not parse renderer metrics for {latex!r}: {result.stderr!r}") from exc
    if metrics.get("error") or metrics.get("width_pt") is None:
        raise RuntimeError(f"renderer reported no size for {latex!r}: {metrics}")
    return PreviewMetrics(
        width_pt=float(metrics["width_pt"]),
        height_pt=float(metrics["height_pt"]),
        baseline_from_bottom_pt=float(metrics["baseline_from_bottom_pt"]),
    )


def _soffice_convert(src: Path, out_path: Path, fmt: str) -> None:
    """Convert one file with LibreOffice headless into out_path (by extension)."""
    soffice = find_soffice()
    if soffice is None:
        raise RuntimeError("LibreOffice (soffice) is required for conversion but was not found")

    src = src.resolve()
    out_path = out_path.resolve()
    out_path.parent.mkdir(parents=True, exist_ok=True)
    # Isolated profile so conversion works even when a LibreOffice window is open
    # (a shared profile lock otherwise silently no-ops the run).
    profile_dir = (PMT_MATHTYPE_CACHE_DIR / "soffice-profile").resolve()
    profile_dir.mkdir(parents=True, exist_ok=True)

    _run([
        soffice,
        "--headless",
        f"-env:UserInstallation=file://{profile_dir}",
        "--convert-to",
        fmt,
        "--outdir",
        str(out_path.parent),
        str(src),
    ])
    produced = out_path.parent / f"{src.stem}.{fmt}"
    if produced != out_path and produced.exists():
        shutil.move(str(produced), str(out_path))


def svg_to_wmf(svg_path: Path, wmf_path: Path) -> None:
    """Convert an SVG to a placeable WMF using LibreOffice headless.

    Raises MetafileExportError when LibreOffice returns success but writes no
    metafile (its exporter fails silently on very complex equations).
    """
    _soffice_convert(svg_path, wmf_path, "wmf")
    if not wmf_path.exists():
        raise MetafileExportError(f"LibreOffice produced no WMF for {svg_path}")


def _wmf_wrapping_bitmap(dib: bytes, src_w: int, src_h: int, width_pt: float, height_pt: float) -> bytes:
    """Build a placeable WMF that draws a DIB, for the raster fallback path.

    The metafile holds one StretchDIBits record mapping the bitmap into a window
    sized in twips, so the picture displays at the equation's true point size
    (with the bitmap's own resolution) and passes the same placeable-header and
    window-mapping checks the LibreOffice WMFs do.
    """
    dest_w = max(1, round(width_pt * TWIPS_PER_PT))
    dest_h = max(1, round(height_pt * TWIPS_PER_PT))
    if len(dib) % 2:  # WMF records are word-aligned
        dib += b"\x00"

    def record(function: int, params: bytes) -> bytes:
        size_words = 3 + len(params) // 2  # 2 words size + 1 word function + params
        return struct.pack("<IH", size_words, function) + params

    set_window_org = record(0x020B, struct.pack("<hh", 0, 0))
    set_window_ext = record(0x020C, struct.pack("<hh", dest_h, dest_w))  # (Y, X)
    stretch = record(
        0x0F43,  # META_STRETCHDIB
        struct.pack("<IH", 0x00CC0020, 0)  # SRCCOPY, DIB_RGB_COLORS
        + struct.pack("<hhhh", src_h, src_w, 0, 0)  # SrcHeight, SrcWidth, YSrc, XSrc
        + struct.pack("<hhhh", dest_h, dest_w, 0, 0)  # DestHeight, DestWidth, YDest, XDest
        + dib,
    )
    eof = record(0x0000, b"")
    records = set_window_org + set_window_ext + stretch + eof

    max_record_words = max(len(r) for r in (set_window_org, set_window_ext, stretch, eof)) // 2
    header = struct.pack(
        "<HHHIHIH",
        1,            # mtType: in-memory
        9,            # mtHeaderSize (words)
        0x0300,       # mtVersion
        (18 + len(records)) // 2,  # mtSize (total words)
        0,            # mtNoObjects
        max_record_words,
        0,            # mtNoParameters
    )

    # Aldus placeable header: key, hwmf, bbox, units/inch, reserved, checksum.
    placeable = struct.pack(
        "<IHhhhhHI",
        0x9AC6CDD7, 0,
        0, 0, dest_w, dest_h,
        WMF_UNITS_PER_INCH, 0,
    )
    checksum = 0
    for (word,) in struct.iter_unpack("<H", placeable[:20]):
        checksum ^= word
    placeable += struct.pack("<H", checksum)

    return placeable + header + records


def svg_to_wmf_via_bitmap(svg_path: Path, wmf_path: Path, width_pt: float, height_pt: float) -> None:
    """Rasterize an SVG (via LibreOffice BMP) and wrap it in a placeable WMF.

    Fallback for equations LibreOffice cannot export as a metafile. The SVG is
    rasterized larger than its display size for a crisp bitmap, then drawn back
    at the true point size inside the WMF window.
    """
    scaled_svg = svg_path.with_name(f"{svg_path.stem}.raster.svg")
    text = svg_path.read_text(encoding="utf-8")
    text = re.sub(r'width="([0-9.]+)pt"', lambda m: f'width="{float(m.group(1)) * RASTER_FALLBACK_SCALE:.3f}pt"', text, count=1)
    text = re.sub(r'height="([0-9.]+)pt"', lambda m: f'height="{float(m.group(1)) * RASTER_FALLBACK_SCALE:.3f}pt"', text, count=1)
    scaled_svg.write_text(text, encoding="utf-8")

    bmp_path = svg_path.with_suffix(".bmp")
    _soffice_convert(scaled_svg, bmp_path, "bmp")
    if not bmp_path.exists():
        raise RuntimeError(f"LibreOffice produced no BMP fallback for {svg_path}")

    bmp = bmp_path.read_bytes()
    # BMP = 14-byte BITMAPFILEHEADER + DIB (BITMAPINFOHEADER + pixels).
    width_px, height_px = struct.unpack_from("<ii", bmp, 18)
    dib = bmp[14:]
    wmf_path.write_bytes(_wmf_wrapping_bitmap(dib, width_px, abs(height_px), width_pt, height_pt))
    bmp_path.unlink(missing_ok=True)
    scaled_svg.unlink(missing_ok=True)


def make_wmf_metadata_cross_platform(
    latex: str,
    wmf_output: Path,
    metadata_output: Path,
    font_size_pt: float | None = None,
) -> None:
    """Render a LaTeX payload to a WMF preview plus SDK-compatible metadata JSON."""
    em_pt = font_size_pt if font_size_pt and font_size_pt > 0 else DEFAULT_EM_PT
    svg_path = wmf_output.with_suffix(".svg")
    metrics = render_latex_to_svg(latex, svg_path, em_pt)
    try:
        svg_to_wmf(svg_path, wmf_output)
    except MetafileExportError:
        # Rare very-complex equation LibreOffice cannot vectorize: keep the build
        # going with a crisp rasterized preview wrapped in a placeable WMF.
        log_debug(f"[mathtype] metafile export failed for {latex!r}; using rasterized WMF fallback")
        svg_to_wmf_via_bitmap(svg_path, wmf_output, metrics.width_pt, metrics.height_pt)

    metadata = {
        "width_pt": metrics.width_pt,
        "height_pt": metrics.height_pt,
        "mathtype": {
            "width_pt": metrics.width_pt,
            "height_pt": metrics.height_pt,
            "baseline_from_bottom_pt": metrics.baseline_from_bottom_pt,
        },
        "renderer": "mathjax+soffice",
    }
    metadata_output.parent.mkdir(parents=True, exist_ok=True)
    metadata_output.write_text(json.dumps(metadata, ensure_ascii=True, indent=2), encoding="utf-8")
    log_debug(
        f"[mathtype] cross-platform preview: {metrics.width_pt:.1f}x{metrics.height_pt:.1f}pt "
        f"baseline={metrics.baseline_from_bottom_pt:.1f}pt"
    )


def preview_renderer_digest() -> str:
    """Return a digest of the renderer so cached WMFs die with renderer changes.

    Covers the tex2svg script, its pinned dependency versions, and the SVG->WMF
    backend identity, but not exact tool patch levels (kept coarse on purpose).
    """
    parts = [
        TEX2SVG_SCRIPT.read_text(encoding="utf-8") if TEX2SVG_SCRIPT.exists() else "",
        (JS_DIR / "package.json").read_text(encoding="utf-8") if (JS_DIR / "package.json").exists() else "",
        "soffice" if find_soffice() else "no-soffice",
        f"em={DEFAULT_EM_PT}",
    ]
    return hashlib.sha256("|".join(parts).encode("utf-8")).hexdigest()


def cross_platform_renderer_available() -> tuple[bool, list[str]]:
    """Return whether the cross-platform preview renderer can run, with reasons."""
    reasons: list[str] = []
    if find_node() is None:
        reasons.append("node (Node.js) was not found on PATH; needed to render LaTeX to SVG.")
    if find_npm() is None and not (MATHJAX_NODE_DIR / "node_modules" / "mathjax-full").is_dir():
        reasons.append("npm was not found and MathJax is not yet installed; needed once to fetch the renderer.")
    if find_soffice() is None:
        reasons.append("LibreOffice (soffice) was not found; needed to convert SVG to WMF.")
    return (not reasons), reasons
