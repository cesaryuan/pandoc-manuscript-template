from pathlib import Path
import struct
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "src"))

import json

import pytest

from pandoc_manuscript.mathtype import ole_parts
from pandoc_manuscript.mathtype import preview_wmf
from pandoc_manuscript.mathtype import docx_ole
from pandoc_manuscript.mathtype import convert_marked_docx as convert_marked_docx_module
from pandoc_manuscript.mathtype.ole_parts import decode_process_output


def test_decode_process_output_falls_back_for_localized_helper_errors() -> None:
    """Preserve Chinese stderr from older helpers that use a Windows code page."""
    message = "[ole-helper] 找不到文件\n"

    assert decode_process_output(message.encode("gb18030")) == message


def minimal_wmf(include_window_mapping: bool = True) -> bytes:
    """Build a tiny placeable WMF for preview validation tests."""
    records = []
    if include_window_mapping:
        records.append(struct.pack("<IHhh", 5, ole_parts.WMF_META_SETWINDOWORG, 0, 0))
        records.append(struct.pack("<IHhh", 5, ole_parts.WMF_META_SETWINDOWEXT, 448, 832))
    records.append(struct.pack("<IH", 3, ole_parts.WMF_META_EOF))
    record_bytes = b"".join(records)
    standard_header = struct.pack(
        "<HHHIHIH",
        1,
        9,
        0x0300,
        (ole_parts.WMF_HEADER_SIZE + len(record_bytes)) // 2,
        0,
        5,
        0,
    )
    return ole_parts.PLACEABLE_WMF_KEY_BYTES + bytes(18) + standard_header + record_bytes


def test_wmf_preview_validation_requires_window_mapping() -> None:
    """Reject SDK-style WMFs that Word displays but exports as blank PDFs."""
    assert ole_parts.has_placeable_wmf_header(minimal_wmf())
    assert ole_parts.wmf_has_window_mapping(minimal_wmf())
    assert not ole_parts.wmf_has_window_mapping(minimal_wmf(include_window_mapping=False))


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

    # `both` cross-checks the MathType SDK against rust, so it is Windows-only;
    # off Windows it degrades to `rust`. Pin the platform to exercise both here.
    monkeypatch.setattr(ole_parts.platform, "system", lambda: "Windows")
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
        wmf_path.write_bytes(minimal_wmf())
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


def test_generate_equation_parts_degrades_sdk_modes_to_rust_off_windows(monkeypatch, tmp_path) -> None:
    """Off Windows, SDK-dependent modes fall back to the cross-platform rust backend."""
    methods = []

    monkeypatch.setattr(ole_parts.platform, "system", lambda: "Darwin")
    monkeypatch.setattr(ole_parts, "iter_equation_requests_with_progress", lambda requests: enumerate(requests, start=1))
    monkeypatch.setattr(ole_parts, "mathtype_rust_source_digest", lambda: "rust-source")
    monkeypatch.setattr(ole_parts, "mathtype_rust_exe_digest_for_method", lambda method: "rust-exe")
    monkeypatch.setattr(ole_parts, "preview_renderer_digest", lambda: "renderer")
    monkeypatch.setattr(ole_parts, "restore_cached_equation", lambda *args: False)
    monkeypatch.setattr(ole_parts, "store_cached_equation", lambda *args: None)
    monkeypatch.setattr(ole_parts, "inspect_ole", lambda path: FakeCompound())
    monkeypatch.setattr(ole_parts, "log_warning", lambda message: None)

    def fake_generate_uncached(index, input_path, ole_path, wmf_path, metadata_path, mtef_path, **kwargs):
        methods.append(kwargs["conversion_method"])
        ole_path.write_bytes(b"ole")
        wmf_path.write_bytes(minimal_wmf())
        metadata_path.write_text("{}", encoding="utf-8")

    monkeypatch.setattr(ole_parts, "generate_uncached_equation_parts", fake_generate_uncached)

    ole_parts.generate_equation_parts([ole_parts.EquationRequest("x")], tmp_path, conversion_method="both")

    assert methods == ["rust"]


def test_make_ole_wmf_metadata_with_rust_uses_cross_platform_off_windows(monkeypatch, tmp_path) -> None:
    """Off Windows, draw the preview with the cross-platform LaTeX->WMF renderer."""
    calls = {}
    input_path = tmp_path / "eq.tex"
    input_path.write_text("$x^2$", encoding="utf-8")

    monkeypatch.setattr(ole_parts.platform, "system", lambda: "Darwin")
    monkeypatch.setattr(ole_parts, "make_ole_from_mathtype_rust", lambda *a, **k: calls.setdefault("rust_ole", True))
    monkeypatch.setattr(ole_parts, "make_wmf_metadata_from_mtef", lambda *a, **k: calls.setdefault("sdk", True))

    def fake_cross(latex, wmf_output, metadata_output, font_size_pt=None):
        calls["cross"] = (latex, font_size_pt)

    monkeypatch.setattr(ole_parts, "make_wmf_metadata_cross_platform", fake_cross)

    ole_parts.make_ole_wmf_metadata_with_mathtype_rust(
        input_path,
        tmp_path / "eq.ole.bin",
        tmp_path / "eq.wmf",
        tmp_path / "eq.json",
        tmp_path / "eq.mtef.bin",
        font_size_pt=10.5,
    )

    assert calls.get("rust_ole") is True
    assert calls.get("cross") == ("$x^2$", 10.5)
    assert "sdk" not in calls


def test_make_ole_wmf_metadata_with_rust_uses_sdk_on_windows(monkeypatch, tmp_path) -> None:
    """On Windows, keep drawing the preview through the MathType SDK path."""
    calls = {}
    input_path = tmp_path / "eq.tex"
    input_path.write_text("$x$", encoding="utf-8")

    monkeypatch.setattr(ole_parts.platform, "system", lambda: "Windows")
    monkeypatch.setattr(ole_parts, "make_ole_from_mathtype_rust", lambda *a, **k: None)
    monkeypatch.setattr(ole_parts, "make_wmf_metadata_from_mtef", lambda *a, **k: calls.setdefault("sdk", True))
    monkeypatch.setattr(ole_parts, "make_wmf_metadata_cross_platform", lambda *a, **k: calls.setdefault("cross", True))

    ole_parts.make_ole_wmf_metadata_with_mathtype_rust(
        input_path,
        tmp_path / "eq.ole.bin",
        tmp_path / "eq.wmf",
        tmp_path / "eq.json",
        tmp_path / "eq.mtef.bin",
    )

    assert calls.get("sdk") is True
    assert "cross" not in calls


def test_check_mathtype_availability_non_windows_ready(monkeypatch) -> None:
    """Non-Windows is usable when mathtype-rust and the preview renderer are present."""
    monkeypatch.setattr(ole_parts.platform, "system", lambda: "Darwin")
    monkeypatch.setattr(ole_parts.shutil, "which", lambda name: "/usr/bin/cargo" if name == "cargo" else None)
    monkeypatch.setattr(ole_parts, "cross_platform_renderer_available", lambda: (True, []))

    availability = ole_parts.check_mathtype_availability()

    assert availability.usable
    assert availability.reasons == ()


def test_check_mathtype_availability_non_windows_reports_missing_tools(monkeypatch, tmp_path) -> None:
    """Non-Windows surfaces the missing rust toolchain and renderer tools."""
    monkeypatch.setattr(ole_parts.platform, "system", lambda: "Darwin")
    monkeypatch.setattr(ole_parts.shutil, "which", lambda name: None)
    monkeypatch.setattr(ole_parts, "MATHTYPE_RUST_EXE", tmp_path / "missing-mathtype-rust")
    monkeypatch.setattr(
        ole_parts,
        "cross_platform_renderer_available",
        lambda: (False, ["LibreOffice (soffice) was not found; needed to convert SVG to WMF."]),
    )

    availability = ole_parts.check_mathtype_availability()

    assert not availability.usable
    assert any("mathtype-rust" in reason for reason in availability.reasons)
    assert any("soffice" in reason for reason in availability.reasons)


def test_mathtype_cache_key_preview_digest_only_changes_cross_platform_key() -> None:
    """The preview renderer digest scopes to the cross-platform path only."""
    windows_style = ole_parts.mathtype_cache_key("x", None, None, "helper", "src", "exe", "rust")
    windows_explicit_none = ole_parts.mathtype_cache_key(
        "x", None, None, "helper", "src", "exe", "rust", preview_digest=None
    )
    with_preview = ole_parts.mathtype_cache_key(
        "x", None, None, "helper", "src", "exe", "rust", preview_digest="renderer-v1"
    )
    with_preview_v2 = ole_parts.mathtype_cache_key(
        "x", None, None, "helper", "src", "exe", "rust", preview_digest="renderer-v2"
    )

    # A None digest must not disturb the historical (Windows/SDK) key bytes.
    assert windows_style == windows_explicit_none
    assert with_preview != windows_style
    assert with_preview != with_preview_v2


def test_make_wmf_metadata_cross_platform_writes_sdk_compatible_metadata(monkeypatch, tmp_path) -> None:
    """Emit width/height and mathtype.baseline_from_bottom_pt so placement matches the SDK schema."""
    wmf_output = tmp_path / "eq.wmf"
    metadata_output = tmp_path / "eq.json"

    def fake_render(latex, svg_path, em_pt):
        svg_path.write_text("<svg/>", encoding="utf-8")
        assert em_pt == 10.5
        return preview_wmf.PreviewMetrics(width_pt=48.9, height_pt=21.3, baseline_from_bottom_pt=7.2)

    monkeypatch.setattr(preview_wmf, "render_latex_to_svg", fake_render)
    monkeypatch.setattr(preview_wmf, "svg_to_wmf", lambda svg, wmf: wmf.write_bytes(b"WMF"))

    preview_wmf.make_wmf_metadata_cross_platform("$x$", wmf_output, metadata_output, font_size_pt=10.5)

    data = json.loads(metadata_output.read_text(encoding="utf-8"))
    assert data["width_pt"] == 48.9
    assert data["mathtype"]["baseline_from_bottom_pt"] == 7.2
    assert wmf_output.read_bytes() == b"WMF"


def test_wmf_wrapping_bitmap_is_a_valid_sized_placeable_wmf() -> None:
    """The raster fallback wraps a DIB in a placeable, window-mapped WMF of the right size."""
    # Minimal 2x2 24bpp bottom-up DIB: 40-byte header + two 4-byte-aligned rows.
    info_header = struct.pack("<IiiHHIIiiII", 40, 2, 2, 1, 24, 0, 16, 0, 0, 0, 0)
    dib = info_header + b"\x00" * 16

    wmf = preview_wmf._wmf_wrapping_bitmap(dib, 2, 2, 20.0, 10.0)

    assert ole_parts.has_placeable_wmf_header(wmf)
    assert ole_parts.wmf_has_window_mapping(wmf)
    width_pt, height_pt = docx_ole.wmf_size_points(wmf)
    assert round(width_pt) == 20 and round(height_pt) == 10


def test_make_wmf_metadata_falls_back_to_bitmap_on_metafile_error(monkeypatch, tmp_path) -> None:
    """When LibreOffice cannot vectorize, fall back to the rasterized WMF path."""
    calls = []
    wmf_output = tmp_path / "eq.wmf"
    metadata_output = tmp_path / "eq.json"

    def fake_render(latex, svg_path, em_pt):
        svg_path.write_text("<svg/>", encoding="utf-8")
        return preview_wmf.PreviewMetrics(width_pt=480.0, height_pt=14.0, baseline_from_bottom_pt=3.0)

    def fake_wmf(svg_path, out):
        raise preview_wmf.MetafileExportError("too complex")

    def fake_bitmap(svg_path, out, width_pt, height_pt):
        calls.append((width_pt, height_pt))
        out.write_bytes(b"WMF-RASTER")

    monkeypatch.setattr(preview_wmf, "render_latex_to_svg", fake_render)
    monkeypatch.setattr(preview_wmf, "svg_to_wmf", fake_wmf)
    monkeypatch.setattr(preview_wmf, "svg_to_wmf_via_bitmap", fake_bitmap)

    preview_wmf.make_wmf_metadata_cross_platform("$x$", wmf_output, metadata_output, font_size_pt=None)

    assert calls == [(480.0, 14.0)]
    assert wmf_output.read_bytes() == b"WMF-RASTER"
    assert json.loads(metadata_output.read_text())["mathtype"]["baseline_from_bottom_pt"] == 3.0


def test_cross_platform_renderer_available_reports_missing_node(monkeypatch) -> None:
    """Report a clear reason when node is unavailable for rendering."""
    monkeypatch.setattr(preview_wmf, "find_node", lambda: None)
    monkeypatch.setattr(preview_wmf, "find_npm", lambda: "/usr/bin/npm")
    monkeypatch.setattr(preview_wmf, "find_soffice", lambda: "/usr/bin/soffice")

    ok, reasons = preview_wmf.cross_platform_renderer_available()

    assert not ok
    assert any("node" in reason for reason in reasons)


REQUIRES_LOCAL_RENDERER = not preview_wmf.cross_platform_renderer_available()[0]


@pytest.mark.skipif(REQUIRES_LOCAL_RENDERER, reason="node + MathJax + LibreOffice not available")
def test_cross_platform_preview_end_to_end(tmp_path) -> None:
    """End-to-end: render a real formula and produce a valid placeable WMF preview."""
    wmf_output = tmp_path / "eq.wmf"
    metadata_output = tmp_path / "eq.json"

    preview_wmf.make_wmf_metadata_cross_platform(
        r"$$\frac{1}{2}\alpha_i^2 + \sqrt{x}$$", wmf_output, metadata_output, font_size_pt=10.5
    )

    wmf_bytes = wmf_output.read_bytes()
    assert ole_parts.has_placeable_wmf_header(wmf_bytes)
    assert ole_parts.wmf_has_window_mapping(wmf_bytes)
    width_pt, height_pt = docx_ole.wmf_size_points(wmf_bytes)
    assert width_pt > 0 and height_pt > 0
    data = json.loads(metadata_output.read_text(encoding="utf-8"))
    assert data["mathtype"]["baseline_from_bottom_pt"] > 0


class FakeCompound:
    """Minimal OLE inspector double with a DSMT-bearing native stream."""

    def read_stream(self, name: str) -> bytes:
        """Return a valid MathType marker for generated-output validation."""
        return b"DSMT"
