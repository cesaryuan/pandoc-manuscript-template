from __future__ import annotations

import importlib.util
from pathlib import Path

import panflute as pf


def load_svg_filter():
    """Load the repository Pandoc filter as a normal Python module."""
    path = Path(__file__).resolve().parents[1] / "pandoc" / "filters" / "svg_to_png.py"
    spec = importlib.util.spec_from_file_location("svg_to_png_filter_for_tests", path)
    assert spec is not None
    assert spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class DummyDoc:
    """Provide the filter attributes normally attached by panflute prepare()."""

    pmt_svg_base_dirs = [Path.cwd()]
    pmt_svg_output_root = Path(".pmt/cache/svg-png")
    pmt_svg_dpi = 300
    pmt_svg_scale = 1
    pmt_svg_pmt_version = "test"
    pmt_svg_convert_all = False


def image(url: str, **attributes: str) -> pf.Image:
    """Create a small panflute image node for filter unit tests."""
    elem = pf.Image(pf.Str("caption"), url=url)
    elem.attributes.update(attributes)
    return elem


def test_to_png_attribute_converts_one_svg(monkeypatch) -> None:
    """Allow one SVG image to opt into PNG conversion without a global switch."""
    svg_filter = load_svg_filter()
    calls: list[str] = []

    def fake_rewrite(elem: pf.Image, *args: object) -> pf.Image:
        calls.append(elem.url)
        elem.url = ".pmt/cache/svg-png/figure.png"
        return elem

    monkeypatch.setattr(svg_filter, "rewrite_image", fake_rewrite)

    elem = image("figure.svg", **{"to-png": "true"})
    result = svg_filter.action(elem, DummyDoc())

    assert result is elem
    assert calls == ["figure.svg"]
    assert elem.url == ".pmt/cache/svg-png/figure.png"
    assert "to-png" not in elem.attributes


def test_svg_without_to_png_is_left_unchanged(monkeypatch) -> None:
    """Do not rasterize ordinary SVG images when the global switch is disabled."""
    svg_filter = load_svg_filter()

    def fail_rewrite(elem: pf.Image, *args: object) -> pf.Image:
        raise AssertionError("rewrite_image should not be called")

    monkeypatch.setattr(svg_filter, "rewrite_image", fail_rewrite)

    elem = image("figure.svg")
    result = svg_filter.action(elem, DummyDoc())

    assert result is None
    assert elem.url == "figure.svg"


def test_global_switch_still_converts_svg(monkeypatch) -> None:
    """Keep the existing metadata switch behavior for all SVG images."""
    svg_filter = load_svg_filter()
    calls: list[str] = []

    class ConvertAllDoc(DummyDoc):
        pmt_svg_convert_all = True

    def fake_rewrite(elem: pf.Image, *args: object) -> pf.Image:
        calls.append(elem.url)
        elem.url = ".pmt/cache/svg-png/figure.png"
        return elem

    monkeypatch.setattr(svg_filter, "rewrite_image", fake_rewrite)

    elem = image("figure.svg")
    result = svg_filter.action(elem, ConvertAllDoc())

    assert result is elem
    assert calls == ["figure.svg"]
