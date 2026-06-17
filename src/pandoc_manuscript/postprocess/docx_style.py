#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = [
#   "python-docx>=1.1.0",
#   "pyyaml>=6.0",
# ]
# ///
"""
Apply DOCX paragraph style settings from merged YAML metadata.

Supported metadata:
    docxStyle:
      '正文文本':
        firstLineIndentChars: 2
        paragraphSpacing:
          before: 0pt
          after: 0pt
      'Para After Table':
        paragraphSpacing:
          before: 0pt
          after: 6pt
"""

import argparse
import re
import sys
from pathlib import Path
from typing import Any

try:
    from docx.document import Document as DocumentObject
    from docx.shared import Pt
    from docx.styles.style import _ParagraphStyle
except ImportError as e:
    print(f"Error: Missing dependency: {e}")
    print("Install with: pip install python-docx pyyaml")
    sys.exit(1)

from .common import (
    BODY_TEXT_STYLE_NAMES,
    open_docx,
    print_debug_success,
    print_warning,
    save_docx,
    set_style_first_line_indent_chars as set_common_style_first_line_indent_chars,
    validate_existing_file,
)

from ..metadata import load_merged_metadata


DOCX_STYLE_METADATA_KEYS = ("docxStyle", "docx-style", "docx_style")
# Keep legacy aliases so existing manuscripts with bodyText metadata continue to build.
BODY_TEXT_METADATA_KEYS = ("bodyText", "body-text", "body_text", "docxBodyText", "docx-body-text")
LEGACY_BODY_TEXT_STYLE_NAME = "正文文本"


def first_present(mapping: dict[str, Any], keys: tuple[str, ...]) -> Any:
    """Return the first present value from a mapping for a group of alias keys."""
    for key in keys:
        if key in mapping:
            return mapping[key]
    return None


def first_present_item(mapping: dict[str, Any], keys: tuple[str, ...]) -> tuple[str, Any] | None:
    """Return the first present key/value pair from a mapping for alias-aware lookup."""
    for key in keys:
        if key in mapping:
            return key, mapping[key]
    return None


def parse_number(value: Any, field_name: str) -> float:
    """Parse a numeric metadata value."""
    if isinstance(value, (int, float)):
        return float(value)
    if isinstance(value, str):
        cleaned = value.strip()
        if not cleaned:
            raise ValueError(f"{field_name} must be a number")
        match = re.match(r'^(-?\d+(?:\.\d+)?)\s*(chars?|characters?|ch|字符)?$', cleaned, re.IGNORECASE)
        if not match:
            raise ValueError(f"{field_name} must be a number, got: {value}")
        return float(match.group(1))
    raise ValueError(f"{field_name} must be a number, got: {value!r}")


def parse_points(value: Any, field_name: str) -> float:
    """Parse a point value from YAML metadata."""
    if isinstance(value, (int, float)):
        return float(value)
    if not isinstance(value, str):
        raise ValueError(f"{field_name} must be a point value, got: {value!r}")

    cleaned = value.strip().lower()
    if not cleaned:
        raise ValueError(f"{field_name} must use points, for example 0pt or 6pt")

    match = re.match(r'^(-?\d+(?:\.\d+)?)\s*(pt|磅)?$', cleaned)
    if not match:
        raise ValueError(f"{field_name} must use points, for example 0pt or 6pt")
    return float(match.group(1))


def get_docx_style_metadata(metadata: dict[str, Any]) -> tuple[dict[str, Any], str, bool] | None:
    """Return preferred docxStyle metadata or legacy bodyText metadata mapped to 正文文本."""
    docx_style_item = first_present_item(metadata, DOCX_STYLE_METADATA_KEYS)
    if docx_style_item is not None:
        docx_style_key, docx_style = docx_style_item
        if not isinstance(docx_style, dict):
            raise ValueError(f"{docx_style_key} metadata must be a mapping")
        return docx_style, docx_style_key, False

    legacy_item = first_present_item(metadata, BODY_TEXT_METADATA_KEYS)
    if legacy_item is None:
        return None

    legacy_key, raw_settings = legacy_item
    if not isinstance(raw_settings, dict):
        raise ValueError(f"{legacy_key} metadata must be a mapping")
    # Legacy bodyText always targeted the body paragraph style; expose it through the new style map.
    return {LEGACY_BODY_TEXT_STYLE_NAME: raw_settings}, legacy_key, True


def normalize_paragraph_style_settings(
    raw_settings: dict[str, Any],
    field_prefix: str,
) -> dict[str, float | str]:
    """Normalize one paragraph style metadata block without inventing unspecified formatting."""
    spacing = first_present(raw_settings, ("paragraphSpacing", "paragraph-spacing", "paragraph_spacing")) or {}
    if not isinstance(spacing, dict):
        raise ValueError(f"{field_prefix}.paragraphSpacing metadata must be a mapping")

    indent_value = first_present(
        raw_settings,
        ("firstLineIndentChars", "first-line-indent-chars", "first_line_indent_chars"),
    )
    before_value = first_present(spacing, ("before", "spaceBefore", "space-before", "space_before"))
    after_value = first_present(spacing, ("after", "spaceAfter", "space-after", "space_after"))

    normalized: dict[str, float | str] = {}
    if indent_value is not None:
        normalized["first_line_indent_chars"] = parse_number(
            indent_value,
            f"{field_prefix}.firstLineIndentChars",
        )
    if before_value is not None:
        normalized["space_before_pt"] = parse_points(
            before_value,
            f"{field_prefix}.paragraphSpacing.before",
        )
    if after_value is not None:
        normalized["space_after_pt"] = parse_points(
            after_value,
            f"{field_prefix}.paragraphSpacing.after",
        )
    return normalized


def normalize_docx_style_settings(metadata: dict[str, Any]) -> list[dict[str, Any]] | None:
    """Normalize all configured docxStyle entries into style application records."""
    metadata_item = get_docx_style_metadata(metadata)
    if metadata_item is None:
        return None

    style_map, metadata_key, uses_legacy_body_text = metadata_item
    normalized_styles: list[dict[str, Any]] = []
    for style_name, raw_settings in style_map.items():
        field_prefix = f"{metadata_key}.{style_name}"
        if not isinstance(style_name, str) or not style_name.strip():
            raise ValueError(f"{metadata_key} style names must be non-empty strings")
        if not isinstance(raw_settings, dict):
            raise ValueError(f"{field_prefix} metadata must be a mapping")
        normalized_styles.append(
            {
                "style_name": style_name,
                "candidate_style_names": BODY_TEXT_STYLE_NAMES if uses_legacy_body_text else (style_name,),
                **normalize_paragraph_style_settings(raw_settings, field_prefix),
            }
        )

    return normalized_styles


def set_style_first_line_indent_chars(style: _ParagraphStyle, chars: float, field_name: str) -> None:
    """Set a paragraph style first-line indent using the metadata field name in errors."""
    set_common_style_first_line_indent_chars(
        style,
        chars,
        field_name,
    )


def get_paragraph_style(
    doc: DocumentObject,
    style_name: str,
    candidate_style_names: tuple[str, ...],
) -> tuple[_ParagraphStyle, str] | None:
    """Return a paragraph style by exact DOCX style name, or None when it is missing or unsuitable."""
    for candidate_style_name in candidate_style_names:
        try:
            style = doc.styles[candidate_style_name]
        except KeyError:
            continue
        if not isinstance(style, _ParagraphStyle):
            print_warning(f"DOCX style '{candidate_style_name}' is not a paragraph style, skipping")
            return None
        return style, candidate_style_name

    print_warning(f"DOCX style '{style_name}' was not found, skipping")
    return None


def apply_paragraph_style_settings(doc: DocumentObject, settings: dict[str, Any]) -> dict[str, Any] | None:
    """Apply normalized paragraph style settings to one DOCX style if it exists."""
    style_name = settings["style_name"]
    style_result = get_paragraph_style(doc, style_name, settings["candidate_style_names"])
    if style_result is None:
        return None
    style, applied_style_name = style_result

    if "first_line_indent_chars" in settings:
        set_style_first_line_indent_chars(
            style,
            settings["first_line_indent_chars"],
            f"docxStyle.{style_name}.firstLineIndentChars",
        )
    paragraph_format = style.paragraph_format
    if "space_before_pt" in settings:
        paragraph_format.space_before = Pt(settings["space_before_pt"])
    if "space_after_pt" in settings:
        paragraph_format.space_after = Pt(settings["space_after_pt"])

    return {
        **settings,
        "applied_style_name": applied_style_name,
    }


def apply_docx_style_metadata(doc: DocumentObject, metadata: dict[str, Any]) -> dict[str, Any] | None:
    """Apply all paragraph style settings from already-loaded docxStyle metadata."""
    normalized_styles = normalize_docx_style_settings(metadata)
    if normalized_styles is None:
        return None

    applied_styles: list[dict[str, Any]] = []
    skipped_styles: list[str] = []
    for settings in normalized_styles:
        result = apply_paragraph_style_settings(doc, settings)
        if result is None:
            skipped_styles.append(settings["style_name"])
            continue
        applied_styles.append(result)

    return {
        "applied": applied_styles,
        "skipped": skipped_styles,
    }


def process_file(
    docx_path: str,
    md_path: str,
    save: bool = True,
    metadata_files: list[str | Path] | None = None,
) -> dict[str, Any] | None:
    """Process a DOCX file using merged docxStyle metadata."""
    doc, docx_file = open_docx(docx_path)
    if doc is None or docx_file is None:
        return None
    md_file = validate_existing_file(md_path, "Markdown file")
    if md_file is None:
        return None

    metadata = load_merged_metadata(md_file, metadata_files)
    result = apply_docx_style_metadata(doc, metadata)
    if result is None:
        print_warning("No docxStyle metadata found, skipping")
        return None

    if save:
        save_docx(doc, docx_file)
    for applied in result["applied"]:
        indent = applied.get("first_line_indent_chars", "unchanged")
        before = applied.get("space_before_pt", "unchanged")
        after = applied.get("space_after_pt", "unchanged")
        print_debug_success(
            f"Applied style '{applied['applied_style_name']}': "
            f"first-line indent {indent} chars, "
            f"before {before} pt, after {after} pt"
        )
    return result


def main() -> None:
    """Main entry point for command-line usage."""
    parser = argparse.ArgumentParser(
        description="Apply DOCX paragraph style settings from merged YAML metadata"
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
