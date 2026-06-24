from pandoc_manuscript.docx.postprocess.final_docx_syntax_check import (
    detect_unrendered_pandoc_syntax,
)


def pattern_names(text: str) -> list[str]:
    """Return only pattern labels so syntax-check tests stay focused."""
    return [name for name, _matched_text in detect_unrendered_pandoc_syntax(text)]


def test_detects_pandoc_fenced_divs_opening_marker() -> None:
    """Catch residual fenced_divs openers that Pandoc should have consumed."""
    assert "Pandoc fenced_divs marker" in pattern_names("::: {.note #demo}")


def test_detects_pandoc_fenced_divs_closing_marker() -> None:
    """Catch residual fenced_divs closing markers left as visible DOCX text."""
    assert "Pandoc fenced_divs marker" in pattern_names(":::")
