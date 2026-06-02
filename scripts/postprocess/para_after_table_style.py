#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = [
#   "python-docx>=1.1.0",
# ]
# ///
"""
Apply the 'Para After Table' style to body text paragraphs after tables.

This script targets top-level paragraphs that immediately follow a table and
currently use the document Body Text/正文文本 style. It creates 'Para After Table'
as a Body Text-based style with 3 pt spacing before, then applies it only to
those post-table body paragraphs.
"""

import argparse
import sys
from pathlib import Path
from typing import Iterable, Optional, cast

try:
    from docx import Document
    from docx.document import Document as DocumentObject
    from docx.enum.style import WD_STYLE_TYPE
    from docx.oxml.ns import qn
    from docx.shared import Pt
    from docx.styles.style import _ParagraphStyle
    from docx.table import Table
    from docx.text.paragraph import Paragraph
except ImportError:
    print("Error: python-docx is not installed. Install it with: pip install python-docx")
    sys.exit(1)


BODY_TEXT_STYLE_NAMES = ("Body Text", "正文文本")
PARA_AFTER_TABLE_STYLE_NAME = "Para After Table"
PARA_AFTER_TABLE_SPACE_BEFORE_PT = 6.0


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
    """Print an informational message in cyan."""
    print(f"{Colors.CYAN}{message}{Colors.RESET}")


def print_warning(message: str) -> None:
    """Print a warning message in yellow."""
    print(f"{Colors.YELLOW}{message}{Colors.RESET}")


def print_error(message: str) -> None:
    """Print an error message in red."""
    print(f"{Colors.RED}{message}{Colors.RESET}")


def get_body_text_style(doc: DocumentObject) -> tuple[_ParagraphStyle | None, str | None]:
    """Return the Word Body Text/正文文本 paragraph style used for post-table paragraphs."""
    for style_name in BODY_TEXT_STYLE_NAMES:
        try:
            return cast(_ParagraphStyle, doc.styles[style_name]), style_name
        except KeyError:
            continue
    return None, None


def configure_para_after_table_style(style: _ParagraphStyle, body_text_style: _ParagraphStyle) -> None:
    """Configure the post-table paragraph style based on Body Text with 3 pt spacing before."""
    style.base_style = body_text_style
    style.paragraph_format.space_before = Pt(PARA_AFTER_TABLE_SPACE_BEFORE_PT)

    # Keep this edge-case style visible so users can inspect or tweak table-following paragraphs in Word.
    style.hidden = False
    style.quick_style = True
    style.priority = 1


def ensure_para_after_table_style_exists(doc: DocumentObject) -> bool:
    """Ensure 'Para After Table' exists and is based on the document Body Text/正文文本 style."""
    body_text_style, body_text_style_name = get_body_text_style(doc)
    if body_text_style is None:
        print_warning("Neither 'Body Text' nor '正文文本' style was found, skipping post-table paragraph style")
        return False

    try:
        para_after_table_style = cast(_ParagraphStyle, doc.styles[PARA_AFTER_TABLE_STYLE_NAME])
        configure_para_after_table_style(para_after_table_style, body_text_style)
        print_info(f"'{PARA_AFTER_TABLE_STYLE_NAME}' style already exists")
        return True
    except KeyError:
        print_info(f"'{PARA_AFTER_TABLE_STYLE_NAME}' style not found, creating it...")

    try:
        style = cast(
            _ParagraphStyle,
            doc.styles.add_style(PARA_AFTER_TABLE_STYLE_NAME, WD_STYLE_TYPE.PARAGRAPH),
        )
        configure_para_after_table_style(style, body_text_style)
        print_success(
            f"'{PARA_AFTER_TABLE_STYLE_NAME}' style created based on '{body_text_style_name}' "
            f"with {PARA_AFTER_TABLE_SPACE_BEFORE_PT:g} pt spacing before"
        )
        return True
    except Exception as e:
        print_error(f"Failed to create '{PARA_AFTER_TABLE_STYLE_NAME}' style: {e}")
        return False


def iter_body_blocks(doc: DocumentObject) -> Iterable[Paragraph | Table]:
    """Yield top-level body paragraphs and tables in document order."""
    for child in doc.element.body.iterchildren():
        if child.tag == qn("w:p"):
            yield Paragraph(child, doc)
        elif child.tag == qn("w:tbl"):
            yield Table(child, doc)


def is_body_text_paragraph(paragraph: Paragraph) -> bool:
    """Return whether a paragraph currently uses Body Text/正文文本 style."""
    style = paragraph.style
    return bool(style is not None and style.name in BODY_TEXT_STYLE_NAMES)


def apply_para_after_table_style(doc: DocumentObject) -> int:
    """Apply 'Para After Table' to Body Text paragraphs immediately following tables."""
    updated = 0
    previous_was_table = False

    for block in iter_body_blocks(doc):
        if isinstance(block, Table):
            previous_was_table = True
            continue

        if previous_was_table and is_body_text_paragraph(block):
            # Only restyle regular body paragraphs directly after tables; captions and headings keep their own styles.
            block.style = PARA_AFTER_TABLE_STYLE_NAME
            updated += 1

        previous_was_table = False

    return updated


def process_para_after_table_style(doc: DocumentObject) -> int | None:
    """Ensure the style exists, then apply it to matching post-table paragraphs."""
    if not ensure_para_after_table_style_exists(doc):
        return None
    return apply_para_after_table_style(doc)


def process_file(docx_path: str, save: bool = True) -> Optional[DocumentObject]:
    """Process a DOCX file and optionally save Para After Table style changes."""
    docx_file = Path(docx_path)
    if not docx_file.exists():
        print_error(f"DOCX file not found: {docx_path}")
        return None

    docx_path_abs = docx_file.resolve()
    print_info(f"Processing: {docx_path_abs}")

    try:
        doc = Document(str(docx_path_abs))
        updated = process_para_after_table_style(doc)
        if updated is None:
            return None

        if save:
            doc.save(str(docx_path_abs))
            print_success("Document saved")

        print_success(f"Applied '{PARA_AFTER_TABLE_STYLE_NAME}' style to {updated} paragraph(s)")
        return doc
    except Exception as e:
        print_error(f"\nPara After Table style processing failed: {e}")
        import traceback

        traceback.print_exc()
        return None


def main() -> None:
    """Main entry point for command-line usage."""
    parser = argparse.ArgumentParser(
        description="Apply 'Para After Table' to Body Text paragraphs after tables"
    )
    parser.add_argument("docx_path", help="Path to the DOCX file to process")
    parser.add_argument("--no-save", action="store_true", help="Do not save changes")
    args = parser.parse_args()

    result = process_file(args.docx_path, save=not args.no_save)
    sys.exit(0 if result else 1)


if __name__ == "__main__":
    main()
