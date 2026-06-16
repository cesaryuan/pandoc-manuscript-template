#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = [
#   "python-docx>=1.1.0",
#   "pyyaml>=6.0",
# ]
# ///
"""
Apply DOCX body text style settings from merged YAML metadata.

Supported metadata:
    bodyText:
      firstLineIndentChars: 2
      paragraphSpacing:
        before: 0pt
        after: 0pt
"""

import argparse
import re
import sys
from pathlib import Path
from typing import Any

if __package__ in (None, ""):
    # Allow direct execution via `uv run scripts/postprocess/<script>.py`.
    sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

try:
    from docx.document import Document as DocumentObject
    from docx.shared import Pt
except ImportError as e:
    print(f"Error: Missing dependency: {e}")
    print("Install with: pip install python-docx pyyaml")
    sys.exit(1)

from postprocess.common import (
    get_body_text_style,
    open_docx,
    print_error,
    print_debug_success,
    print_warning,
    save_docx,
    set_style_first_line_indent_chars as set_common_style_first_line_indent_chars,
    validate_existing_file,
)

from metadata import load_merged_metadata


DEFAULT_FIRST_LINE_INDENT_CHARS = 2.0
DEFAULT_SPACE_BEFORE_PT = 0.0
DEFAULT_SPACE_AFTER_PT = 0.0
BODY_TEXT_METADATA_KEYS = ("bodyText", "body-text", "body_text", "docxBodyText", "docx-body-text")


def first_present(mapping: dict[str, Any], keys: tuple[str, ...]) -> Any:
    """Return the first present value from a mapping for a group of alias keys."""
    for key in keys:
        if key in mapping:
            return mapping[key]
    return None


def parse_number(value: Any, field_name: str, default: float) -> float:
    """Parse a numeric metadata value with a fallback default."""
    if value is None:
        return default
    if isinstance(value, (int, float)):
        return float(value)
    if isinstance(value, str):
        cleaned = value.strip()
        if not cleaned:
            return default
        match = re.match(r'^(-?\d+(?:\.\d+)?)\s*(chars?|characters?|ch|字符)?$', cleaned, re.IGNORECASE)
        if not match:
            raise ValueError(f"{field_name} must be a number, got: {value}")
        return float(match.group(1))
    raise ValueError(f"{field_name} must be a number, got: {value!r}")


def parse_points(value: Any, field_name: str, default: float) -> float:
    """Parse a point value from YAML metadata."""
    if value is None:
        return default
    if isinstance(value, (int, float)):
        return float(value)
    if not isinstance(value, str):
        raise ValueError(f"{field_name} must be a point value, got: {value!r}")

    cleaned = value.strip().lower()
    if not cleaned:
        return default

    match = re.match(r'^(-?\d+(?:\.\d+)?)\s*(pt|磅)?$', cleaned)
    if not match:
        raise ValueError(f"{field_name} must use points, for example 0pt or 6pt")
    return float(match.group(1))


def normalize_body_text_settings(metadata: dict[str, Any]) -> dict[str, float] | None:
    """Normalize body text metadata and fill defaults for omitted subfields."""
    raw_settings = first_present(metadata, BODY_TEXT_METADATA_KEYS)
    if raw_settings is None:
        return None
    if not isinstance(raw_settings, dict):
        raise ValueError("bodyText metadata must be a mapping")

    spacing = first_present(raw_settings, ("paragraphSpacing", "paragraph-spacing", "paragraph_spacing")) or {}
    if not isinstance(spacing, dict):
        raise ValueError("bodyText.paragraphSpacing metadata must be a mapping")

    indent_value = first_present(
        raw_settings,
        ("firstLineIndentChars", "first-line-indent-chars", "first_line_indent_chars"),
    )
    before_value = first_present(spacing, ("before", "spaceBefore", "space-before", "space_before"))
    after_value = first_present(spacing, ("after", "spaceAfter", "space-after", "space_after"))

    return {
        "first_line_indent_chars": parse_number(
            indent_value,
            "bodyText.firstLineIndentChars",
            DEFAULT_FIRST_LINE_INDENT_CHARS,
        ),
        "space_before_pt": parse_points(
            before_value,
            "bodyText.paragraphSpacing.before",
            DEFAULT_SPACE_BEFORE_PT,
        ),
        "space_after_pt": parse_points(
            after_value,
            "bodyText.paragraphSpacing.after",
            DEFAULT_SPACE_AFTER_PT,
        ),
    }


def set_style_first_line_indent_chars(style, chars: float) -> None:
    """Set Body Text first-line indent with the metadata field name in errors."""
    set_common_style_first_line_indent_chars(
        style,
        chars,
        "bodyText.firstLineIndentChars",
    )


def apply_body_text_settings(doc: DocumentObject, settings: dict[str, float]) -> dict[str, Any]:
    """Apply normalized body text settings to the DOCX Body Text style."""
    style, style_name = get_body_text_style(doc)
    if style is None:
        raise KeyError("Neither 'Body Text' nor '正文文本' style was found in the DOCX")

    set_style_first_line_indent_chars(style, settings["first_line_indent_chars"])
    paragraph_format = style.paragraph_format
    paragraph_format.space_before = Pt(settings["space_before_pt"])
    paragraph_format.space_after = Pt(settings["space_after_pt"])

    return {
        "style_name": style_name,
        **settings,
    }


def apply_body_text_style_metadata(doc: DocumentObject, metadata: dict[str, Any]) -> dict[str, Any] | None:
    """Apply body text style settings from already-loaded metadata."""
    settings = normalize_body_text_settings(metadata)
    if settings is None:
        return None
    return apply_body_text_settings(doc, settings)


def process_file(
    docx_path: str,
    md_path: str,
    save: bool = True,
    metadata_files: list[str | Path] | None = None,
) -> dict[str, Any] | None:
    """Process a DOCX file using merged body text metadata."""
    doc, docx_file = open_docx(docx_path)
    if doc is None or docx_file is None:
        return None
    md_file = validate_existing_file(md_path, "Markdown file")
    if md_file is None:
        return None

    metadata = load_merged_metadata(md_file, metadata_files)
    result = apply_body_text_style_metadata(doc, metadata)
    if result is None:
        print_warning("No bodyText metadata found, skipping")
        return None

    if save:
        save_docx(doc, docx_file)
    print_debug_success(
        "Applied Body Text style: "
        f"first-line indent {result['first_line_indent_chars']} chars, "
        f"before {result['space_before_pt']} pt, after {result['space_after_pt']} pt"
    )
    return result


def main() -> None:
    """Main entry point for command-line usage."""
    parser = argparse.ArgumentParser(
        description="Apply DOCX Body Text style settings from merged YAML metadata"
    )
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
