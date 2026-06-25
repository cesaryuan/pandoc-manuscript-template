#!/usr/bin/env python3
"""Apply DOCX page margins from merged YAML metadata."""

import argparse
import re
import sys
from pathlib import Path
from typing import Any

try:
    from docx.document import Document as DocumentObject
    from docx.shared import Cm, Inches, Mm, Pt
except ImportError as e:
    print(f"Error: Missing dependency: {e}")
    print("Install with: pip install python-docx pyyaml")
    sys.exit(1)

from ...runtime.metadata import load_merged_metadata
from .common import open_docx, print_debug, print_debug_success, save_docx, validate_existing_file


PAGE_MARGIN_METADATA_KEYS = ("docxPageMargins", "docx-page-margins", "docx_page_margins")
MARGIN_SIDE_ALIASES = {
    "top": ("top",),
    "bottom": ("bottom",),
    "left": ("left", "inside"),
    "right": ("right", "outside"),
}


def first_present(mapping: dict[str, Any], keys: tuple[str, ...]) -> Any:
    """Return the first configured metadata value for a group of alias keys."""
    for key in keys:
        if key in mapping:
            return mapping[key]
    return None


def parse_margin_length(value: Any, field_name: str):
    """Parse a DOCX page margin length with common Word-friendly units."""
    if isinstance(value, (int, float)):
        return Pt(float(value))
    if not isinstance(value, str):
        raise ValueError(f"{field_name} must be a length value, got: {value!r}")

    cleaned = value.strip().lower()
    match = re.match(r"^(-?\d+(?:\.\d+)?)\s*(pt|磅|cm|厘米|mm|毫米|in|inch|inches|英寸)?$", cleaned)
    if not match:
        raise ValueError(f"{field_name} must be a length such as 72pt, 2.54cm, or 1in")

    amount = float(match.group(1))
    if amount < 0:
        raise ValueError(f"{field_name} must be greater than or equal to 0")

    unit = match.group(2) or "pt"
    if unit in ("pt", "磅"):
        return Pt(amount)
    if unit in ("cm", "厘米"):
        return Cm(amount)
    if unit in ("mm", "毫米"):
        return Mm(amount)
    return Inches(amount)


def get_page_margin_metadata(metadata: dict[str, Any]) -> tuple[dict[str, Any], str] | None:
    """Return configured docxPageMargins metadata and the key that supplied it."""
    for key in PAGE_MARGIN_METADATA_KEYS:
        if key not in metadata:
            continue
        raw_margins = metadata[key]
        if raw_margins is None:
            return None
        if not isinstance(raw_margins, dict):
            raise ValueError(f"{key} metadata must be a mapping")
        return raw_margins, key
    return None


def normalize_page_margins(metadata: dict[str, Any]) -> tuple[dict[str, Any], dict[str, str]] | None:
    """Normalize docxPageMargins into section attributes without defaulting missing sides."""
    metadata_item = get_page_margin_metadata(metadata)
    if metadata_item is None:
        return None

    raw_margins, metadata_key = metadata_item
    margins: dict[str, Any] = {}
    display_values: dict[str, str] = {}
    for side, aliases in MARGIN_SIDE_ALIASES.items():
        value = first_present(raw_margins, aliases)
        if value is None:
            continue
        margins[side] = parse_margin_length(value, f"{metadata_key}.{side}")
        display_values[side] = str(value)

    return margins, display_values


def apply_page_margins(doc: DocumentObject, margins: dict[str, Any]) -> int:
    """Apply normalized page margins to every section in the DOCX document."""
    updated = 0
    for section in doc.sections:
        if "top" in margins:
            section.top_margin = margins["top"]
        if "bottom" in margins:
            section.bottom_margin = margins["bottom"]
        if "left" in margins:
            section.left_margin = margins["left"]
        if "right" in margins:
            section.right_margin = margins["right"]
        updated += 1
    return updated


def apply_page_margin_metadata(doc: DocumentObject, metadata: dict[str, Any]) -> dict[str, Any] | None:
    """Apply docxPageMargins metadata to all DOCX sections."""
    normalized = normalize_page_margins(metadata)
    if normalized is None:
        return None

    margins, display_values = normalized
    if not margins:
        return None

    return {
        "margins": display_values,
        "sections": apply_page_margins(doc, margins),
    }


def process_file(
    docx_path: str,
    md_path: str,
    save: bool = True,
    metadata_files: list[str | Path] | None = None,
) -> dict[str, Any] | None:
    """Process a DOCX file using merged docxPageMargins metadata."""
    doc, docx_file = open_docx(docx_path)
    if doc is None or docx_file is None:
        return None
    md_file = validate_existing_file(md_path, "Markdown file")
    if md_file is None:
        return None

    metadata = load_merged_metadata(md_file, metadata_files)
    result = apply_page_margin_metadata(doc, metadata)
    if result is None:
        print_debug("No docxPageMargins metadata found, skipping")
        return None

    if save:
        save_docx(doc, docx_file)
    print_debug_success(f"Applied page margins: sections={result['sections']}, margins={result['margins']}")
    return result


def main() -> None:
    """Main entry point for command-line usage."""
    parser = argparse.ArgumentParser(description="Apply DOCX page margins from merged YAML metadata")
    parser.add_argument("docx_path", help="Path to the DOCX file to process")
    parser.add_argument("md_path", nargs="?", default="manuscript.md", help="Path to the markdown manuscript")
    parser.add_argument(
        "--metadata-file",
        action="append",
        default=[],
        help="YAML metadata file to merge before manuscript metadata",
    )
    parser.add_argument("--no-save", action="store_true", help="Do not save changes")
    args = parser.parse_args()

    result = process_file(
        args.docx_path,
        args.md_path,
        save=not args.no_save,
        metadata_files=args.metadata_file,
    )
    sys.exit(0 if result is not None else 1)


if __name__ == "__main__":
    main()
