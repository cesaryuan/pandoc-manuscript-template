from pathlib import Path
import sys

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "src"))

from pandoc_manuscript import build, reply_build
from pandoc_manuscript.mathtype import ole_parts
from pandoc_manuscript.paths import (
    PMT_CACHE_DIR,
    PMT_DIR,
    PMT_REPLY_LINE_SOURCE_CACHE_DIR,
    PMT_SVG_EMBED_CACHE_DIR,
    PMT_SVG_PNG_CACHE_DIR,
    PMT_WORK_DIR,
)


def test_generated_work_and_cache_paths_are_under_pmt() -> None:
    """Keep pmt's temporary files and reusable caches in one hidden project directory."""
    paths = [
        Path(build.SETTINGS.mathtype_work_dir),
        reply_build.LINE_SOURCE_PDF_DIR,
        reply_build.LINE_SOURCE_CACHE_DIR,
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
    assert env["PMT_SVG_TO_PNG_CONVERT_ALL"] == "false"


def test_svg_embed_cache_uses_pmt_cache(monkeypatch) -> None:
    """Route self-contained SVG cache files away from final output directories."""
    monkeypatch.setattr(build.SETTINGS, "manuscript_file", "manuscript.md")

    env = build.docx_svg_embed_images_filter_env({})

    assert Path(env["PMT_SVG_EMBED_DIR"]) == (Path.cwd() / PMT_SVG_EMBED_CACHE_DIR).resolve()
    assert env["PMT_SVG_EMBED_IMAGES"] == "true"


def test_svg_embed_env_keeps_global_embedding_switch(monkeypatch) -> None:
    """Pass the global SVG child-image embedding switch to the DOCX filter."""
    monkeypatch.setattr(build.SETTINGS, "manuscript_file", "manuscript.md")

    env = build.docx_svg_embed_images_filter_env({"docxEmbedSvgImages": True})

    assert env["PMT_SVG_EMBED_IMAGES"] == "true"


def test_reply_svg_embed_env_uses_shared_cache(tmp_path) -> None:
    """Route reply self-contained SVG cache files through the shared pmt cache."""
    reply = tmp_path / "reply.md"

    env = reply_build.svg_embed_images_filter_env(reply, {"docxEmbedSvgImages": True})

    assert Path(env["PMT_SVG_EMBED_DIR"]) == (Path.cwd() / PMT_SVG_EMBED_CACHE_DIR).resolve()
    assert env["PMT_SVG_EMBED_IMAGES"] == "true"
    assert str(tmp_path.resolve()) in env["PMT_SVG_EMBED_BASE_DIRS"]


def test_reply_svg_to_png_env_uses_shared_cache(tmp_path) -> None:
    """Route reply SVG rasterization cache files through the shared pmt cache."""
    reply = tmp_path / "reply.md"

    env = reply_build.svg_to_png_filter_env(reply, {"docxConvertSvgToPng": True})

    assert Path(env["PMT_SVG_TO_PNG_DIR"]) == (Path.cwd() / PMT_SVG_PNG_CACHE_DIR).resolve()
    assert env["PMT_SVG_TO_PNG_CONVERT_ALL"] == "true"
    assert str(tmp_path.resolve()) in env["PMT_SVG_TO_PNG_BASE_DIRS"]


def test_svg_to_png_env_keeps_global_conversion_switch(monkeypatch) -> None:
    """Pass the global SVG rasterization switch to the shared DOCX filter."""
    monkeypatch.setattr(build.SETTINGS, "manuscript_file", "manuscript.md")

    env = build.docx_svg_to_png_filter_env({"docxConvertSvgToPng": True})

    assert env["PMT_SVG_TO_PNG_CONVERT_ALL"] == "true"


def test_svg_to_png_env_passes_width_control(monkeypatch) -> None:
    """Pass the optional SVG-to-PNG output width to the shared DOCX filter."""
    monkeypatch.setattr(build.SETTINGS, "manuscript_file", "manuscript.md")

    env = build.docx_svg_to_png_filter_env({"docxSvgToPngWidth": 1600})

    assert env["PMT_SVG_TO_PNG_WIDTH"] == "1600"


@pytest.mark.parametrize(
    "metadata",
    [
        {"docxSvgToPngWidth": 1600, "docxSvgToPngScale": 2},
        {"docxSvgToPngWidth": 1600, "docxSvgToPngDpi": 300},
        {"docxSvgToPngScale": 2, "docxSvgToPngDpi": 300},
    ],
)
def test_svg_to_png_size_controls_are_mutually_exclusive(monkeypatch, metadata: dict[str, object]) -> None:
    """Reject ambiguous global SVG-to-PNG size controls."""
    monkeypatch.setattr(build.SETTINGS, "manuscript_file", "manuscript.md")

    with pytest.raises(ValueError, match="Only one of docxSvgToPngWidth"):
        build.docx_svg_to_png_filter_env(metadata)


def test_python_filter_wrapper_uses_pmt_work_dir(monkeypatch, tmp_path) -> None:
    """Place generated Pandoc filter launchers under the pmt work directory."""
    filter_path = tmp_path / "filter.py"
    filter_path.write_text("print('ok')\n", encoding="utf-8")
    monkeypatch.chdir(tmp_path)

    wrapper = build.python_filter_wrapper(filter_path, "sample_filter")

    assert wrapper.parts[: len(PMT_WORK_DIR.parts)] == PMT_WORK_DIR.parts


def test_reply_line_source_cache_uses_pmt_cache() -> None:
    """Keep reusable reply line-source artifacts under the shared pmt cache."""
    assert reply_build.LINE_SOURCE_CACHE_DIR == PMT_REPLY_LINE_SOURCE_CACHE_DIR
    assert reply_build.LINE_SOURCE_CACHE_DIR.parts[: len(PMT_CACHE_DIR.parts)] == PMT_CACHE_DIR.parts
