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
    prefs_path = tmp_path / "size.eqp"

    monkeypatch.setattr(ole_parts, "build_mathtype_rust_converter", lambda: rust_exe)
    monkeypatch.setattr(ole_parts, "run", lambda command, **kwargs: calls.append((command, kwargs)))

    ole_parts.make_ole_from_mathtype_rust(input_path, ole_path, mtef_path, prefs_file=prefs_path)

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
                "--prefs-file",
                str(prefs_path),
            ],
            {"stderr_as_warning": False, "stdout_as_debug": True, "stderr_as_debug": True},
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


def test_generate_uncached_equation_parts_auto_falls_back_to_rust(monkeypatch, tmp_path) -> None:
    """Use mathtype-rust only after the set-data path fails in auto mode."""
    calls = []

    def fail_set_data(*args, **kwargs):
        calls.append("set-data")
        raise RuntimeError("set-data failed")

    monkeypatch.setattr(ole_parts, "make_ole_wmf_metadata_with_mathtype_set_data", fail_set_data)
    monkeypatch.setattr(
        ole_parts,
        "make_ole_wmf_metadata_with_mathtype_rust",
        lambda *args, **kwargs: calls.append("rust"),
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

    assert calls == ["set-data", "rust"]


def test_normalize_conversion_method_accepts_style_aliases() -> None:
    """Normalize user-facing style metadata values to conversion backends."""
    assert ole_parts.normalize_conversion_method(None) == "rust"
    assert ole_parts.normalize_conversion_method("mathtype-rust") == "rust"
    assert ole_parts.normalize_conversion_method("tex") == "set-data"
    assert ole_parts.normalize_conversion_method("fallback") == "auto"
    assert ole_parts.normalize_conversion_method("both") == "both"


def test_generate_equation_parts_both_uses_independent_backend_caches(monkeypatch, tmp_path) -> None:
    """Generate both backends with separate cache methods and keep set-data output."""
    calls = []
    warnings = []

    monkeypatch.setattr(ole_parts, "iter_equation_requests_with_progress", lambda requests: enumerate(requests, start=1))
    monkeypatch.setattr(ole_parts, "mathtype_ole_mtef_sha256", lambda path: f"mtef:{Path(path).name}")
    monkeypatch.setattr(ole_parts, "json_result_sha256", lambda path: f"json:{Path(path).name}")
    monkeypatch.setattr(ole_parts, "mathtype_rust_source_digest", lambda: "rust-source")
    monkeypatch.setattr(ole_parts, "restore_cached_equation", lambda *args: False)
    monkeypatch.setattr(ole_parts, "store_cached_equation", lambda *args: None)
    monkeypatch.setattr(ole_parts, "inspect_ole", lambda path: FakeCompound())
    monkeypatch.setattr(ole_parts, "log_warning", lambda message: warnings.append(message))

    def fake_generate_uncached(index, input_path, ole_path, wmf_path, metadata_path, mtef_path, **kwargs):
        method = kwargs["conversion_method"]
        calls.append(method)
        ole_path.write_bytes(b"ole-" + method.encode())
        wmf_path.write_bytes(bytes.fromhex("d7cdc69a") + b"-" + method.encode())
        metadata_path.write_text('{"method":"' + method + '"}', encoding="utf-8")

    monkeypatch.setattr(ole_parts, "generate_uncached_equation_parts", fake_generate_uncached)

    equations = ole_parts.generate_equation_parts(
        [ole_parts.EquationRequest("x")],
        tmp_path,
        conversion_method="both",
    )

    assert calls == ["rust", "set-data"]
    assert equations[0].ole_path == tmp_path / "eq_001.ole.bin"
    assert (tmp_path / "eq_001.ole.bin").read_bytes() == b"ole-set-data"
    assert (tmp_path / "eq_001.rust.ole.bin").read_bytes() == b"ole-rust"
    assert any("rust and set-data outputs differ" in message for message in warnings)


def test_warn_if_conversion_outputs_differ_ignores_wmf(monkeypatch, tmp_path) -> None:
    """Compare only OLE MTEF and JSON results in both-mode diagnostics."""
    warnings = []
    rust_ole = tmp_path / "rust.ole.bin"
    set_data_ole = tmp_path / "set-data.ole.bin"
    rust_json = tmp_path / "rust.json"
    set_data_json = tmp_path / "set-data.json"
    for path in (rust_ole, set_data_ole):
        path.write_bytes(b"not-used")
    rust_json.write_text('{"height_pt": 1, "width_pt": 2}', encoding="utf-8")
    set_data_json.write_text('{"width_pt": 2, "height_pt": 1}', encoding="utf-8")

    monkeypatch.setattr(ole_parts, "mathtype_ole_mtef_sha256", lambda path: "same-mtef")
    monkeypatch.setattr(ole_parts, "log_warning", lambda message: warnings.append(message))

    ole_parts.warn_if_conversion_outputs_differ(
        1,
        rust_ole,
        rust_json,
        set_data_ole,
        set_data_json,
    )

    assert warnings == []


def test_json_result_sha256_ignores_raw_wmf_metadata(tmp_path) -> None:
    """Ignore backend-specific raw WMF metadata when hashing JSON results."""
    left = tmp_path / "left.json"
    right = tmp_path / "right.json"
    left.write_text(
        (
            '{"map_mode":8,"x_ext":800,"y_ext":448,"units_per_inch":2304,'
            '"width_pt":24.9732,"height_pt":13.9748,'
            '"mathtype":{"width_raw":800,"height_raw":448,'
            '"baseline_from_bottom_raw":96,"width_pt":25.0,"height_pt":14.0,'
            '"baseline_from_bottom_pt":3.0,"horiz_pos_type":0,"horiz_pos":0}}'
        ),
        encoding="utf-8",
    )
    right.write_text(
        (
            '{"map_mode":8,"x_ext":881,"y_ext":493,"units_per_inch":2540,'
            '"height_pt":14.0,"width_pt":25.0,'
            '"mathtype":{"height_raw":493,"width_raw":881,'
            '"baseline_from_bottom_raw":106,"height_pt":14.0,"width_pt":25.0,'
            '"baseline_from_bottom_pt":3.0,"horiz_pos_type":5,"horiz_pos":718}}'
        ),
        encoding="utf-8",
    )

    assert ole_parts.json_result_sha256(left) == ole_parts.json_result_sha256(right)


def test_json_result_sha256_reports_rounded_point_metric_difference(tmp_path) -> None:
    """Keep rounded point-size differences visible in JSON comparison."""
    left = tmp_path / "left.json"
    right = tmp_path / "right.json"
    left.write_text('{"width_pt": 90.0, "mathtype": {"height_pt": 14.0}}', encoding="utf-8")
    right.write_text('{"width_pt": 91.0, "mathtype": {"height_pt": 14.0}}', encoding="utf-8")

    assert ole_parts.json_result_sha256(left) != ole_parts.json_result_sha256(right)


def test_warn_if_conversion_outputs_differ_reports_ole_mtef_and_json(monkeypatch, tmp_path) -> None:
    """Report OLE MTEF and canonical JSON differences."""
    warnings = []
    rust_ole = tmp_path / "rust.ole.bin"
    set_data_ole = tmp_path / "set-data.ole.bin"
    rust_json = tmp_path / "rust.json"
    set_data_json = tmp_path / "set-data.json"
    for path in (rust_ole, set_data_ole):
        path.write_bytes(b"not-used")
    rust_json.write_text('{"height_pt": 1}', encoding="utf-8")
    set_data_json.write_text('{"height_pt": 2}', encoding="utf-8")

    monkeypatch.setattr(ole_parts, "mathtype_ole_mtef_sha256", lambda path: Path(path).stem)
    monkeypatch.setattr(ole_parts, "log_warning", lambda message: warnings.append(message))

    ole_parts.warn_if_conversion_outputs_differ(
        2,
        rust_ole,
        rust_json,
        set_data_ole,
        set_data_json,
    )

    assert len(warnings) == 1
    assert "OLE MTEF" in warnings[0]
    assert "JSON" in warnings[0]
    assert "WMF" not in warnings[0]


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
    changed_helper = ole_parts.mathtype_cache_key("x", None, None, "changed-helper", "rust-src-a", "rust-exe", "rust")
    changed_source = ole_parts.mathtype_cache_key("x", None, None, "helper", "rust-src-b", "rust-exe", "rust")
    changed_exe = ole_parts.mathtype_cache_key("x", None, None, "helper", "rust-src-a", "changed-rust-exe", "rust")
    changed_method = ole_parts.mathtype_cache_key("x", None, None, "helper", "rust-src-a", "rust-exe", "set-data")
    set_data_base = ole_parts.mathtype_cache_key("x", None, None, "helper", "rust-src-a", "rust-exe", "set-data")
    set_data_changed_rust = ole_parts.mathtype_cache_key(
        "x",
        None,
        None,
        "helper",
        "rust-src-b",
        "changed-rust-exe",
        "set-data",
    )

    assert base != changed_helper
    assert base != changed_source
    assert base != changed_exe
    assert base != changed_method
    assert set_data_base == set_data_changed_rust


class FakeCompound:
    """Minimal OLE inspector double with a DSMT-bearing native stream."""

    def read_stream(self, name: str) -> bytes:
        """Return a valid MathType marker for generated-output validation."""
        return b"DSMT"
