from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "src"))

from pandoc_manuscript.mathtype import ole_parts
from pandoc_manuscript.mathtype import convert_marked_docx as convert_marked_docx_module
from pandoc_manuscript.mathtype.ole_parts import decode_process_output


def test_decode_process_output_falls_back_for_localized_helper_errors() -> None:
    """Preserve Chinese stderr from older helpers that use a Windows code page."""
    message = "[ole-helper] 找不到文件\n"

    assert decode_process_output(message.encode("gb18030")) == message


def test_make_ole_from_mathtype_rust_uses_file_input(monkeypatch, tmp_path) -> None:
    """Pass normalized TeX files through mathtype-rust for Rust conversion."""
    calls = []
    rust_exe = tmp_path / "mathtype-rust.exe"
    input_path = tmp_path / "eq.tex"
    ole_path = tmp_path / "eq.ole.bin"
    mtef_path = tmp_path / "eq.mtef.bin"

    monkeypatch.setattr(ole_parts, "build_mathtype_rust_converter", lambda: rust_exe)
    monkeypatch.setattr(ole_parts, "run", lambda command, **kwargs: calls.append((command, kwargs)))

    ole_parts.make_ole_from_mathtype_rust(input_path, ole_path, mtef_path)

    assert calls == [
        (
            [
                str(rust_exe),
                "--input",
                str(input_path),
                "--output",
                str(ole_path),
                "--mtef-output",
                str(mtef_path),
            ],
            {"stderr_as_warning": False},
        )
    ]


def test_make_wmf_metadata_from_mtef_uses_sdk_xform_ole(monkeypatch, tmp_path) -> None:
    """Use the helper's MTEF SDK path to create Rust-path WMF and JSON files."""
    calls = []
    mtef_path = tmp_path / "eq.mtef.bin"
    helper_ole_path = tmp_path / "eq.sdk.ole.bin"
    wmf_path = tmp_path / "eq.wmf"
    metadata_path = tmp_path / "eq.json"
    prefs_path = tmp_path / "size.eqp"

    monkeypatch.setattr(ole_parts, "run", lambda command, **kwargs: calls.append(command))

    ole_parts.make_wmf_metadata_from_mtef(
        mtef_path,
        helper_ole_path,
        wmf_path,
        metadata_path,
        prefs_file=prefs_path,
    )

    command = calls[0]
    assert command[command.index("--method") + 1] == "sdk-xform-ole"
    assert command[command.index("--format") + 1] == "MathType EF"
    assert command[command.index("--input") + 1] == str(mtef_path)
    assert command[command.index("--output") + 1] == str(helper_ole_path)
    assert command[command.index("--preview-output") + 1] == str(wmf_path)
    assert command[command.index("--metadata-output") + 1] == str(metadata_path)
    assert command[command.index("--prefs-file") + 1] == str(prefs_path)
    assert "--binary" in command


def test_generate_uncached_equation_parts_uses_rust_method(monkeypatch, tmp_path) -> None:
    """Use only the Rust path when style metadata selects rust."""
    calls = []

    monkeypatch.setattr(
        ole_parts,
        "make_ole_wmf_metadata_with_mathtype_rust",
        lambda *args, **kwargs: calls.append("rust"),
    )
    monkeypatch.setattr(
        ole_parts,
        "make_ole_wmf_metadata_with_mathtype_set_data",
        lambda *args, **kwargs: calls.append("set-data"),
    )

    ole_parts.generate_uncached_equation_parts(
        1,
        tmp_path / "eq.tex",
        tmp_path / "eq.ole.bin",
        tmp_path / "eq.wmf",
        tmp_path / "eq.json",
        tmp_path / "eq.mtef.bin",
        conversion_method="rust",
    )

    assert calls == ["rust"]


def test_generate_uncached_equation_parts_uses_set_data_method(monkeypatch, tmp_path) -> None:
    """Use only MathType TeX import when style metadata selects set-data."""
    calls = []

    monkeypatch.setattr(
        ole_parts,
        "make_ole_wmf_metadata_with_mathtype_rust",
        lambda *args, **kwargs: calls.append("rust"),
    )
    monkeypatch.setattr(
        ole_parts,
        "make_ole_wmf_metadata_with_mathtype_set_data",
        lambda *args, **kwargs: calls.append("set-data"),
    )

    ole_parts.generate_uncached_equation_parts(
        1,
        tmp_path / "eq.tex",
        tmp_path / "eq.ole.bin",
        tmp_path / "eq.wmf",
        tmp_path / "eq.json",
        tmp_path / "eq.mtef.bin",
        conversion_method="set-data",
    )

    assert calls == ["set-data"]


def test_generate_uncached_equation_parts_auto_falls_back_to_set_data(monkeypatch, tmp_path) -> None:
    """Use MathType TeX import only after the Rust path fails in auto mode."""
    calls = []

    def fail_rust(*args, **kwargs):
        calls.append("rust")
        raise RuntimeError("rust failed")

    monkeypatch.setattr(ole_parts, "make_ole_wmf_metadata_with_mathtype_rust", fail_rust)
    monkeypatch.setattr(
        ole_parts,
        "make_ole_wmf_metadata_with_mathtype_set_data",
        lambda *args, **kwargs: calls.append("set-data"),
    )

    ole_parts.generate_uncached_equation_parts(
        1,
        tmp_path / "eq.tex",
        tmp_path / "eq.ole.bin",
        tmp_path / "eq.wmf",
        tmp_path / "eq.json",
        tmp_path / "eq.mtef.bin",
        conversion_method="auto",
    )

    assert calls == ["rust", "set-data"]


def test_normalize_conversion_method_accepts_style_aliases() -> None:
    """Normalize user-facing style metadata values to conversion backends."""
    assert ole_parts.normalize_conversion_method(None) == "rust"
    assert ole_parts.normalize_conversion_method("mathtype-rust") == "rust"
    assert ole_parts.normalize_conversion_method("tex") == "set-data"
    assert ole_parts.normalize_conversion_method("fallback") == "auto"


def test_convert_marked_docx_passes_style_conversion_method(monkeypatch, tmp_path) -> None:
    """Pass style metadata backend selection into MathType part generation."""
    seen = {}

    monkeypatch.setattr(
        convert_marked_docx_module,
        "extract_marked_equation_requests",
        lambda source: [ole_parts.EquationRequest("x")],
    )
    monkeypatch.setattr(
        convert_marked_docx_module,
        "replace_marked_omml_with_generated",
        lambda source, target, equations: 1,
    )
    monkeypatch.setattr(convert_marked_docx_module, "inspect_docx", lambda target: None)

    def fake_generate_equation_parts(requests, output_dir, conversion_method="rust"):
        seen["conversion_method"] = conversion_method
        return [
            ole_parts.GeneratedEquation(
                latex=requests[0].latex,
                ole_path=tmp_path / "eq.ole.bin",
                wmf_path=tmp_path / "eq.wmf",
            )
        ]

    monkeypatch.setattr(convert_marked_docx_module, "generate_equation_parts", fake_generate_equation_parts)

    replaced = convert_marked_docx_module.convert_marked_docx(
        tmp_path / "source.docx",
        tmp_path / "target.docx",
        tmp_path / "work",
        metadata={"mathtypeConversionMethod": "set-data"},
    )

    assert replaced == 1
    assert seen["conversion_method"] == "set-data"


def test_mathtype_cache_key_includes_rust_converter_and_method_digests() -> None:
    """Invalidate cache entries when the Rust converter or selected method changes."""
    base = ole_parts.mathtype_cache_key("x", None, None, "helper", "rust-src-a", "rust-exe", "rust")
    changed_source = ole_parts.mathtype_cache_key("x", None, None, "helper", "rust-src-b", "rust-exe", "rust")
    changed_method = ole_parts.mathtype_cache_key("x", None, None, "helper", "rust-src-a", "rust-exe", "set-data")

    assert base != changed_source
    assert base != changed_method
