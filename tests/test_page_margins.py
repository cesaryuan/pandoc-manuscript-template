from pathlib import Path
import sys

import pytest
from docx import Document

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "src"))

from pandoc_manuscript.docx.postprocess.page_margins import (
    apply_page_margin_metadata,
    normalize_page_margins,
)


def test_apply_page_margin_metadata_updates_all_docx_sections() -> None:
    """Apply docxPageMargins values to every generated DOCX section."""
    doc = Document()
    doc.add_section()

    result = apply_page_margin_metadata(
        doc,
        {
            "docxPageMargins": {
                "top": "2.54cm",
                "bottom": "2.54cm",
                "left": "3.17cm",
                "right": "3.17cm",
            }
        },
    )

    assert result == {
        "margins": {
            "top": "2.54cm",
            "bottom": "2.54cm",
            "left": "3.17cm",
            "right": "3.17cm",
        },
        "sections": 2,
    }
    for section in doc.sections:
        assert section.top_margin.cm == pytest.approx(2.54, abs=0.001)
        assert section.bottom_margin.cm == pytest.approx(2.54, abs=0.001)
        assert section.left_margin.cm == pytest.approx(3.17, abs=0.001)
        assert section.right_margin.cm == pytest.approx(3.17, abs=0.001)


def test_docx_page_margins_partial_config_leaves_other_sides_unset() -> None:
    """Do not invent margin defaults when only some sides are configured."""
    normalized = normalize_page_margins({"docxPageMargins": {"top": "72pt"}})

    assert normalized is not None
    margins, display_values = normalized
    assert set(margins) == {"top"}
    assert display_values == {"top": "72pt"}
