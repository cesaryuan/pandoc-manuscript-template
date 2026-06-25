from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "src"))

from pandoc_manuscript.mathtype import ole_parts
from pandoc_manuscript.mathtype.ole_parts import decode_process_output


def test_decode_process_output_falls_back_for_localized_helper_errors() -> None:
    """Preserve Chinese stderr from older helpers that use a Windows code page."""
    message = "[ole-helper] 找不到文件\n"

    assert decode_process_output(message.encode("gb18030")) == message


def test_make_ole_from_mathtype_rust_uses_file_input(monkeypatch, tmp_path) -> None:
    """Pass normalized TeX files through mathtype-rust for fallback conversion."""
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
    """Use the helper's MTEF SDK path to create fallback WMF and JSON files."""
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


def test_mathtype_cache_key_includes_rust_fallback_digests() -> None:
    """Invalidate fallback cache entries when the Rust converter changes."""
    base = ole_parts.mathtype_cache_key("x", None, None, "helper", "rust-src-a", "rust-exe")
    changed = ole_parts.mathtype_cache_key("x", None, None, "helper", "rust-src-b", "rust-exe")

    assert base != changed
