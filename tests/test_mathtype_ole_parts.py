from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "src"))

from pandoc_manuscript.mathtype.ole_parts import decode_process_output


def test_decode_process_output_falls_back_for_localized_helper_errors() -> None:
    """Preserve Chinese stderr from older helpers that use a Windows code page."""
    message = "[ole-helper] 找不到文件\n"

    assert decode_process_output(message.encode("gb18030")) == message
