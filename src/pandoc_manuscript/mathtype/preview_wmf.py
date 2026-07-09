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
import shutil
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


def svg_to_wmf(svg_path: Path, wmf_path: Path) -> None:
    """Convert an SVG to a placeable WMF using LibreOffice headless."""
    soffice = find_soffice()
    if soffice is None:
        raise RuntimeError("LibreOffice (soffice) is required for SVG->WMF conversion but was not found")

    svg_path = svg_path.resolve()
    wmf_path = wmf_path.resolve()
    wmf_path.parent.mkdir(parents=True, exist_ok=True)
    # Use an isolated profile so conversion still works when the user has a
    # LibreOffice window open (a shared profile lock otherwise no-ops the run).
    profile_dir = (PMT_MATHTYPE_CACHE_DIR / "soffice-profile").resolve()
    profile_dir.mkdir(parents=True, exist_ok=True)

    _run([
        soffice,
        "--headless",
        f"-env:UserInstallation=file://{profile_dir}",
        "--convert-to",
        "wmf",
        "--outdir",
        str(wmf_path.parent),
        str(svg_path),
    ])
    produced = wmf_path.parent / f"{svg_path.stem}.wmf"
    if not produced.exists():
        raise RuntimeError(f"soffice did not create a WMF for {svg_path}")
    if produced != wmf_path:
        shutil.move(str(produced), str(wmf_path))


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
    svg_to_wmf(svg_path, wmf_output)

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
