#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = [
#   "python-docx>=1.1.0",
# ]
# ///
"""
Apply the 'Where Paragraph' style to equation explanation paragraphs.

The post-processor targets paragraphs that immediately follow a display-style
math paragraph and begin with the word "where". This keeps formula definitions
separate from regular body paragraphs in exported DOCX files.
"""

import argparse
import re
import sys
from pathlib import Path
from typing import Iterable, cast

try:
    from docx import Document
    from docx.document import Document as DocumentObject
    from docx.enum.style import WD_STYLE_TYPE
    from docx.oxml import OxmlElement
    from docx.oxml.ns import qn
    from docx.styles.style import _ParagraphStyle
    from docx.text.paragraph import Paragraph
except ImportError:
    print("Error: python-docx is not installed. Install it with: pip install python-docx")
    sys.exit(1)


WHERE_STYLE_NAME = "Where Paragraph"
WHERE_FIRST_LINE_INDENT_CHARS = 0.0
BODY_TEXT_STYLE_CANDIDATES = ("Body Text", "正文文本")
BASE_STYLE_CANDIDATES = (*BODY_TEXT_STYLE_CANDIDATES, "First Paragraph", "Normal", "正文")
WHERE_START_PATTERN = re.compile(r"^\s*where\b", re.IGNORECASE)
EQUATION_NUMBER_PATTERN = re.compile(r"^[\s\t\r\n()（）\[\]【】0-9ivxlcdmIVXLCDM.\-–—]*$")


class Colors:
    """ANSI color codes for terminal output."""

    GREEN = "\033[92m"
    CYAN = "\033[96m"
    YELLOW = "\033[93m"
    RED = "\033[91m"
    RESET = "\033[0m"


def print_success(message: str) -> None:
    """Print a success message in green."""
    print(f"{Colors.GREEN}{message}{Colors.RESET}")


def print_info(message: str) -> None:
    """Print an info message in cyan."""
    print(f"{Colors.CYAN}{message}{Colors.RESET}")


def print_warning(message: str) -> None:
    """Print a warning message in yellow."""
    print(f"{Colors.YELLOW}{message}{Colors.RESET}")


def print_error(message: str) -> None:
    """Print an error message in red."""
    print(f"{Colors.RED}{message}{Colors.RESET}")


def get_first_existing_paragraph_style(
    doc: DocumentObject, style_names: tuple[str, ...]
) -> tuple[_ParagraphStyle | None, str | None]:
    """Return the first paragraph style found from a list of candidate names."""
    for style_name in style_names:
        try:
            return cast(_ParagraphStyle, doc.styles[style_name]), style_name
        except KeyError:
            continue
    return None, None


def set_where_next_paragraph_style(doc: DocumentObject, style: _ParagraphStyle) -> bool:
    """Set the paragraph style Word uses after pressing Enter in a where paragraph."""
    body_text_style, body_text_style_name = get_first_existing_paragraph_style(
        doc, BODY_TEXT_STYLE_CANDIDATES
    )
    if body_text_style is None:
        print_error("Neither 'Body Text' nor '正文文本' style was found, cannot set next paragraph style")
        return False

    style.next_paragraph_style = body_text_style
    print_info(f"'Where Paragraph' next paragraph style set to '{body_text_style_name}'")
    return True


def set_style_first_line_indent_chars(style: _ParagraphStyle, chars: float) -> None:
    """Set a paragraph style first-line indent using Word's character-based OOXML value."""
    if chars < 0:
        raise ValueError("Where Paragraph first-line indent must be greater than or equal to 0")

    p_pr = style.element.get_or_add_pPr()
    ind = p_pr.find(qn("w:ind"))
    if ind is None:
        ind = OxmlElement("w:ind")
        p_pr.append(ind)

    # This style is for equation definitions, so it explicitly disables inherited first-line indent.
    ind.set(qn("w:firstLineChars"), str(int(round(chars * 100))))
    for attr_name in ("w:firstLine", "w:hanging", "w:hangingChars"):
        attr = qn(attr_name)
        if attr in ind.attrib:
            del ind.attrib[attr]


def set_where_paragraph_format(style: _ParagraphStyle) -> None:
    """Apply fixed paragraph formatting for the Where Paragraph style."""
    set_style_first_line_indent_chars(style, WHERE_FIRST_LINE_INDENT_CHARS)
    print_info("'Where Paragraph' first-line indent set to 0 chars")


def ensure_where_paragraph_style_exists(doc: DocumentObject) -> bool:
    """Ensure the DOCX contains a paragraph style named 'Where Paragraph'."""
    try:
        style = cast(_ParagraphStyle, doc.styles[WHERE_STYLE_NAME])
        if not set_where_next_paragraph_style(doc, style):
            return False
        set_where_paragraph_format(style)
        print_info("'Where Paragraph' style already exists")
        return True
    except KeyError:
        print_info("'Where Paragraph' style not found, creating it...")

    try:
        style = cast(_ParagraphStyle, doc.styles.add_style(WHERE_STYLE_NAME, WD_STYLE_TYPE.PARAGRAPH))
        base_style, base_style_name = get_first_existing_paragraph_style(doc, BASE_STYLE_CANDIDATES)
        if base_style is not None:
            style.base_style = base_style
            print_info(f"'Where Paragraph' style based on '{base_style_name}' style")
        if not set_where_next_paragraph_style(doc, style):
            return False
        set_where_paragraph_format(style)

        # Where clauses are equation definitions, so keep them available as a named style.
        style.hidden = False
        style.quick_style = True
        style.priority = 1
        print_success("'Where Paragraph' style created successfully")
        return True
    except Exception as e:
        print_error(f"Failed to create 'Where Paragraph' style: {e}")
        return False


def paragraph_contains_math(paragraph: Paragraph) -> bool:
    """Return whether a paragraph contains Word math elements."""
    element = paragraph._p
    return bool(element.findall(f".//{qn('m:oMath')}") or element.findall(f".//{qn('m:oMathPara')}"))


def paragraph_has_equation_layout(paragraph: Paragraph) -> bool:
    """Return whether a math paragraph looks like a standalone/display equation."""
    element = paragraph._p
    return bool(element.findall(f".//{qn('w:tab')}") or element.findall(f".//{qn('w:tabs')}"))


def paragraph_visible_text(paragraph: Paragraph) -> str:
    """Return visible non-math text from a paragraph."""
    return paragraph.text or ""


def is_equation_paragraph(paragraph: Paragraph) -> bool:
    """Return whether a paragraph should be treated as the formula before a where clause."""
    if not paragraph_contains_math(paragraph):
        return False
    if paragraph_has_equation_layout(paragraph):
        return True

    # Unnumbered display equations may contain only math and punctuation visible to python-docx.
    return bool(EQUATION_NUMBER_PATTERN.match(paragraph_visible_text(paragraph)))


def starts_with_where(paragraph: Paragraph) -> bool:
    """Return whether a paragraph begins with the word 'where'."""
    return bool(WHERE_START_PATTERN.match(paragraph_visible_text(paragraph)))


def iter_adjacent_paragraph_pairs(paragraphs: Iterable[Paragraph]) -> Iterable[tuple[Paragraph, Paragraph]]:
    """Yield adjacent paragraph pairs in document order."""
    previous: Paragraph | None = None
    for paragraph in paragraphs:
        if previous is not None:
            yield previous, paragraph
        previous = paragraph


def apply_where_paragraph_style(doc: DocumentObject) -> int:
    """Style where paragraphs that immediately follow equation paragraphs."""
    updated = 0
    for previous, current in iter_adjacent_paragraph_pairs(doc.paragraphs):
        if not is_equation_paragraph(previous) or not starts_with_where(current):
            continue
        current.style = WHERE_STYLE_NAME
        updated += 1
    return updated


def process_file(docx_path: str, save: bool = True) -> int | None:
    """Process a DOCX file and optionally save where paragraph style changes."""
    docx_file = Path(docx_path)
    if not docx_file.exists():
        print_error(f"DOCX file not found: {docx_path}")
        return None

    doc = Document(str(docx_file))
    if not ensure_where_paragraph_style_exists(doc):
        return None

    updated = apply_where_paragraph_style(doc)
    if save:
        doc.save(str(docx_file))

    print_success(f"Applied 'Where Paragraph' style to {updated} paragraph(s)")
    return updated


def main() -> None:
    """Main entry point for command-line usage."""
    parser = argparse.ArgumentParser(
        description="Apply 'Where Paragraph' style after DOCX equation paragraphs"
    )
    parser.add_argument("docx_path", help="Path to the DOCX file to process")
    parser.add_argument("--no-save", action="store_true", help="Do not save changes")
    args = parser.parse_args()

    result = process_file(args.docx_path, save=not args.no_save)
    sys.exit(0 if result is not None else 1)


if __name__ == "__main__":
    main()
