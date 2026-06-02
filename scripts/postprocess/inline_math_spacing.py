#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = [
#   "python-docx>=1.1.0",
# ]
# ///
"""
Keep standalone inline math as inline math in Word-rendered DOCX files.

Pandoc correctly emits a single inline equation as ``m:oMath``. When that
equation is the only visible content in a paragraph, Word can render or
normalize it like a display equation. Adding a trailing text-space run keeps
the paragraph in Word's inline-equation path while remaining visually harmless.
"""

import argparse
import sys
from pathlib import Path
from typing import Iterable

if __package__ in (None, ""):
    # Allow direct execution via `uv run scripts/postprocess/<script>.py`.
    sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from postprocess.common import open_docx, print_error, print_success, save_docx

try:
    from docx.document import Document as DocumentObject
    from docx.oxml import OxmlElement
    from docx.oxml.ns import qn
except ImportError:
    print("Error: python-docx is not installed. Install it with: pip install python-docx")
    sys.exit(1)


XML_SPACE = "{http://www.w3.org/XML/1998/namespace}space"
IGNORABLE_PARAGRAPH_CHILDREN = {
    qn("w:pPr"),
    qn("w:bookmarkStart"),
    qn("w:bookmarkEnd"),
    qn("w:proofErr"),
    qn("w:permStart"),
    qn("w:permEnd"),
}


def iter_paragraph_elements(doc: DocumentObject) -> Iterable:
    """Yield all paragraph XML elements in the main document body."""
    return doc.element.body.iter(qn("w:p"))


def paragraph_has_display_math(paragraph_element) -> bool:
    """Return whether the paragraph contains Word display math."""
    return bool(paragraph_element.findall(f".//{qn('m:oMathPara')}"))


def paragraph_visible_text(paragraph_element) -> str:
    """Return visible text found in Word text runs for a paragraph."""
    return "".join(node.text or "" for node in paragraph_element.findall(f".//{qn('w:t')}"))


def has_trailing_space_after_math(paragraph_element, math_element) -> bool:
    """Return whether a whitespace-only text run already follows the math element."""
    children = list(paragraph_element)
    try:
        math_index = children.index(math_element)
    except ValueError:
        return False

    for child in children[math_index + 1 :]:
        if child.tag in IGNORABLE_PARAGRAPH_CHILDREN:
            continue
        text_nodes = child.findall(f".//{qn('w:t')}")
        if text_nodes and all((node.text or "").isspace() for node in text_nodes):
            return True
        return False
    return False


def create_space_run():
    """Create a preserved single-space text run for Word inline math compatibility."""
    run = OxmlElement("w:r")
    text = OxmlElement("w:t")
    text.set(XML_SPACE, "preserve")
    text.text = " "
    run.append(text)
    return run


def should_add_trailing_space(paragraph_element, math_elements: list) -> bool:
    """Return whether a paragraph needs the standalone inline-math workaround."""
    if len(math_elements) != 1:
        return False
    if paragraph_has_display_math(paragraph_element):
        return False
    if paragraph_visible_text(paragraph_element).strip():
        return False
    return not has_trailing_space_after_math(paragraph_element, math_elements[0])


def add_space_after_standalone_inline_math(doc: DocumentObject) -> int:
    """Add a trailing space after paragraphs that contain only one inline math element."""
    updated = 0
    for paragraph_element in iter_paragraph_elements(doc):
        math_elements = paragraph_element.findall(qn("m:oMath"))
        if not should_add_trailing_space(paragraph_element, math_elements):
            continue

        math_element = math_elements[0]
        paragraph_element.insert(paragraph_element.index(math_element) + 1, create_space_run())
        updated += 1
    return updated


def process_file(docx_path: str, save: bool = True) -> int | None:
    """Process a DOCX file and optionally save the inline-math spacing fix."""
    doc, docx_path_abs = open_docx(docx_path)
    if doc is None or docx_path_abs is None:
        return None
    updated = add_space_after_standalone_inline_math(doc)
    if save and updated:
        save_docx(doc, docx_path_abs)

    print_success(f"Added trailing spaces after {updated} standalone inline math paragraph(s)")
    return updated


def main() -> None:
    """Main entry point for command-line usage."""
    parser = argparse.ArgumentParser(
        description="Add trailing spaces after standalone inline math in DOCX files"
    )
    parser.add_argument("docx_path", help="Path to the DOCX file to process")
    parser.add_argument("--no-save", action="store_true", help="Do not save changes")
    args = parser.parse_args()

    result = process_file(args.docx_path, save=not args.no_save)
    sys.exit(0 if result is not None else 1)


if __name__ == "__main__":
    main()
