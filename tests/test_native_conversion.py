"""Exercise the real C ABI against CLI artifacts when native builds are available."""

import json
import os
import subprocess
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

import pytest

from pandoc_manuscript.mathtype import native, ole_parts

ROOT = Path(__file__).resolve().parents[1]


@pytest.mark.parametrize("project", ["mathtype-rust"])
def test_missing_library_never_launches_cli(monkeypatch, tmp_path, project):
    """An executable-only installation must fail explicitly without starting its CLI."""
    (tmp_path / f"{project}.exe").write_bytes(b"legacy executable")
    monkeypatch.setattr(native, "source_tree_root", lambda: None)
    monkeypatch.setattr(native, "library_path", lambda name: tmp_path / native.library_name(name))

    def forbid_process(*args, **kwargs):
        """Catch any attempt to fall back to a subprocess."""
        pytest.fail("Missing native libraries must not launch a CLI")

    monkeypatch.setattr(subprocess, "run", forbid_process)
    native._load_converter.cache_clear()
    with pytest.raises(FileNotFoundError, match="native library is missing"):
        native.get_converter(project)


@pytest.fixture
def converters():
    """Load prebuilt libraries without triggering Cargo in ordinary Python tests."""
    result = {}
    for project in ("mathtype-rust",):
        path = ROOT / "scripts" / project / "target/release" / native.library_name(project)
        if not path.exists():
            pytest.skip("Build the mathtype-rust release cdylib and both reference CLIs first")
        result[project] = native.NativeConverter(project, path)
    return result


@pytest.mark.parametrize("latex", [r"\frac{\alpha_1}{2}", r"\mathbf{x}+\sqrt{y}"])
def test_equation_matches_cli(converters, tmp_path, latex):
    """Keep byte-exact OLE/MTEF compatibility, including preference-file handling."""
    prefs = ROOT / "src/pandoc_manuscript/mathtype/Times+Symbol 12.eqp"
    result = converters["mathtype-rust"].call(latex=latex, prefs_file=str(prefs))
    executable = ROOT / "scripts/mathtype-rust/target/release" / ("mathtype-rust.exe" if os.name == "nt" else "mathtype-rust")
    subprocess.run([str(executable), "--latex", latex, "--output", str(tmp_path / "eq.ole"),
                    "--mtef-output", str(tmp_path / "eq.mtef"), "--prefs-file", str(prefs)], check=True)
    assert result["ole"] == (tmp_path / "eq.ole").read_bytes()
    assert result["mtef"] == (tmp_path / "eq.mtef").read_bytes()


@pytest.mark.parametrize("backend,style", [("typst", "inline"), ("ratex", "display")])
def test_preview_matches_cli(converters, tmp_path, backend, style):
    """Preserve vector artifacts and baseline metadata for both rendering engines."""
    latex = r"\frac{x_1}{2}"
    result = converters["mathtype-rust"].call(operation="render_wmf", latex=latex, svg_backend=backend, math_style=style, font_size_pt=10.5)
    executable = ROOT / "scripts/latex2wmf/target/release" / ("latex2wmf.exe" if os.name == "nt" else "latex2wmf")
    subprocess.run([str(executable), "--latex", latex, "--output", str(tmp_path / "eq.wmf"),
                    "--metadata-output", str(tmp_path / "eq.json"), "--svg-output", str(tmp_path / "eq.svg"),
                    "--svg-backend", backend, "--math-style", style, "--font-size", "10.5"], check=True)
    assert result["wmf"] == (tmp_path / "eq.wmf").read_bytes()
    assert result["svg"] == (tmp_path / "eq.svg").read_text(encoding="utf-8")
    assert json.loads(result["metadata_json"]) == json.loads((tmp_path / "eq.json").read_text())


def test_pipeline_is_in_process_and_recovers_from_errors(converters, monkeypatch, tmp_path):
    """A failed formula removes stale files without breaking subsequent native calls."""
    monkeypatch.setattr(native, "get_converter", converters.get)

    def forbid_process(*args, **kwargs):
        """Reject process launches after the shared libraries have loaded."""
        pytest.fail("Native formula conversion must not start a subprocess")

    monkeypatch.setattr(subprocess, "run", forbid_process)
    source = tmp_path / "eq.tex"
    source.write_text(r"\frac{x}{2}", encoding="utf-8-sig")
    ole, mtef, wmf, metadata = [tmp_path / name for name in ("eq.ole", "eq.mtef", "eq.wmf", "eq.json")]
    ole_parts.make_ole_from_mathtype_rust(source, ole, mtef)
    ole_parts.make_wmf_metadata_cross_platform(source, wmf, metadata)
    assert ole.read_bytes().startswith(bytes.fromhex("d0cf11e0a1b11ae1"))
    assert wmf.read_bytes().startswith(bytes.fromhex("d7cdc69a"))
    with pytest.raises(ole_parts.FormulaPreviewError):
        ole_parts.make_wmf_metadata_cross_platform(source, wmf, metadata, svg_backend="invalid")
    assert not wmf.exists() and not metadata.exists()
    with pytest.raises(ole_parts.FormulaConversionError):
        ole_parts.make_ole_from_mathtype_rust(source, ole, mtef, prefs_file=tmp_path / "missing.eqp")
    assert not ole.exists() and not mtef.exists()
    ole_parts.make_ole_from_mathtype_rust(source, ole, mtef)
    ole_parts.make_wmf_metadata_cross_platform(source, wmf, metadata)
    assert ole.exists() and wmf.exists()


def test_cached_typst_keeps_each_requests_layout(converters, tmp_path):
    """Compare concurrent mixed layouts with independent CLI renders to catch shared-input leaks."""
    converter = converters["mathtype-rust"]
    executable = ROOT / "scripts/latex2wmf/target/release" / ("latex2wmf.exe" if os.name == "nt" else "latex2wmf")
    contexts = [
        dict(latex=r"\frac{x_1}{2}", math_style="inline", font_size_pt=10.5),
        dict(latex=r"\frac{x_2}{3}", math_style="display", font_size_pt=16.0),
        dict(latex=r"\text{中文}+x", math_style="inline", font_size_pt=12.0),
        dict(latex=r"\sqrt{y}", math_style="display", font_size_pt=9.0),
    ]
    expected = []
    for index, context in enumerate(contexts):
        output = tmp_path / f"{index}.wmf"
        metadata = tmp_path / f"{index}.json"
        subprocess.run([
            str(executable), "--latex", context["latex"], "--svg-backend", "typst",
            "--math-style", context["math_style"], "--font-size", str(context["font_size_pt"]),
            "--output", str(output), "--metadata-output", str(metadata),
        ], check=True, capture_output=True)
        expected.append((output.read_bytes(), json.loads(metadata.read_text(encoding="utf-8"))))
    with ThreadPoolExecutor(max_workers=4) as executor:
        futures = [executor.submit(converter.call, operation="render_wmf", **context) for context in contexts * 2]
        for index, future in enumerate(futures):
            result = future.result()
            assert (result["wmf"], json.loads(result["metadata_json"])) == expected[index % len(contexts)]


def test_cached_typst_observes_font_file_replacement(converters, tmp_path):
    """A cached font must not conceal replacement, deletion, or a different requested family."""
    converter = converters["mathtype-rust"]
    original = (ROOT / "scripts/latex2wmf/assets/fonts/XITSMath-Regular.otf").read_bytes()
    font = tmp_path / "数学字体.otf"
    font.write_bytes(original)
    request = dict(operation="render_wmf", latex=r"\frac{x}{2}", math_font=str(font))
    expected = converter.call(**request)["wmf"]
    font.write_bytes(b"not an OpenType math font")
    with pytest.raises(RuntimeError, match="no OpenType math font"):
        converter.call(**request)
    font.unlink()
    with pytest.raises(RuntimeError, match="failed to read math font"):
        converter.call(**request)
    with pytest.raises(RuntimeError, match="not installed"):
        converter.call(operation="render_wmf", latex="x", math_font="PMT nonexistent math font 20260908")
    font.write_bytes(original)
    assert converter.call(**request)["wmf"] == expected
