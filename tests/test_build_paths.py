from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "src"))

from pandoc_manuscript import build, reply_build
from pandoc_manuscript.mathtype import ole_parts
from pandoc_manuscript.paths import PMT_CACHE_DIR, PMT_DIR, PMT_WORK_DIR


def test_generated_work_and_cache_paths_are_under_pmt() -> None:
    """Keep pmt's temporary files and reusable caches in one hidden project directory."""
    paths = [
        Path(build.SETTINGS.mathtype_work_dir),
        reply_build.LINE_SOURCE_PDF_DIR,
        reply_build.REPLY_PROBE_DIR,
        ole_parts.MATHTYPE_CACHE_DIR,
    ]

    for path in paths:
        assert path.parts[0] == PMT_DIR.name


def test_svg_to_png_cache_uses_pmt_cache(monkeypatch) -> None:
    """Route SVG rasterization artifacts away from final output directories."""
    monkeypatch.setattr(build.SETTINGS, "manuscript_file", "manuscript.md")

    env = build.docx_svg_to_png_filter_env({})

    assert Path(env["PMT_SVG_TO_PNG_DIR"]) == (Path.cwd() / PMT_CACHE_DIR / "svg-png").resolve()


def test_python_filter_wrapper_uses_pmt_work_dir(monkeypatch, tmp_path) -> None:
    """Place generated Pandoc filter launchers under the pmt work directory."""
    filter_path = tmp_path / "filter.py"
    filter_path.write_text("print('ok')\n", encoding="utf-8")
    monkeypatch.chdir(tmp_path)

    wrapper = build.python_filter_wrapper(filter_path, "sample_filter")

    assert wrapper.parts[: len(PMT_WORK_DIR.parts)] == PMT_WORK_DIR.parts
