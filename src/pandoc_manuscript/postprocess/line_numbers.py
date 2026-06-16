#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = [
#   "python-docx>=1.1.0",
#   "pyyaml>=6.0",
# ]
# ///
"""Apply DOCX line-number settings from merged YAML metadata."""

import argparse
import sys
from pathlib import Path
from typing import Any

try:
    from docx.document import Document as DocumentObject
    from docx.oxml import OxmlElement
    from docx.oxml.ns import qn
except ImportError as e:
    print(f"Error: Missing dependency: {e}")
    print("Install with: pip install python-docx pyyaml")
    sys.exit(1)

from ..metadata import load_merged_metadata
from .common import open_docx, print_debug, print_debug_success, print_warning, save_docx, validate_existing_file


LINE_NUMBER_METADATA_KEYS = ("show-line-numbers", "showLineNumbers", "show_line_numbers")
DEFAULT_RESTART = "continuous"
RESTART_ALIASES = {
    "continuous": "continuous",
    "continue": "continuous",
    "continuously": "continuous",
    "true": "continuous",
    "on": "continuous",
    "yes": "continuous",
    "1": "continuous",
    "连续": "continuous",
    "restart-page": "newPage",
    "restart-each-page": "newPage",
    "new-page": "newPage",
    "newpage": "newPage",
    "page": "newPage",
    "每页": "newPage",
    "每页重编": "newPage",
    "按页重启": "newPage",
    "restart-section": "newSection",
    "restart-each-section": "newSection",
    "new-section": "newSection",
    "newsection": "newSection",
    "section": "newSection",
    "每节": "newSection",
    "每节重编": "newSection",
    "按节重启": "newSection",
}
DISABLED_VALUES = {"false", "off", "no", "0", "none", "disable", "disabled", "不显示", "关闭", "无"}


def first_present(mapping: dict[str, Any], keys: tuple[str, ...]) -> Any:
    """Return the first configured metadata value for a group of alias keys."""
    for key in keys:
        if key in mapping:
            return mapping[key]
    return None


def normalize_line_number_setting(value: Any) -> str | None:
    """Normalize show-line-numbers metadata to a Word restart mode.

    This handles the common shorthand `show-line-numbers: true` as continuous
    numbering, while string values select Word's line-number restart behavior.
    """
    if value is None:
        return None
    if isinstance(value, bool):
        return DEFAULT_RESTART if value else None
    if isinstance(value, (int, float)):
        return DEFAULT_RESTART if bool(value) else None
    if not isinstance(value, str):
        raise ValueError("show-line-numbers must be a boolean or string")

    cleaned = value.strip()
    if not cleaned:
        return None

    lookup_key = cleaned.lower().replace("_", "-").replace(" ", "-")
    if lookup_key in DISABLED_VALUES:
        return None
    if cleaned in RESTART_ALIASES:
        return RESTART_ALIASES[cleaned]
    if lookup_key in RESTART_ALIASES:
        return RESTART_ALIASES[lookup_key]

    valid = "continuous, restart-page/newPage, restart-section/newSection, 连续, 每页重编, 每节重编"
    raise ValueError(f"Unsupported show-line-numbers value: {value!r}. Expected one of: {valid}")


def line_number_setting_from_metadata(metadata: dict[str, Any]) -> str | None:
    """Return the normalized line-number setting from merged metadata."""
    return normalize_line_number_setting(first_present(metadata, LINE_NUMBER_METADATA_KEYS))


def get_or_add_line_number_type(sect_pr):
    """Return a section's w:lnNumType element, creating it in Word-compatible order."""
    existing = sect_pr.find(qn("w:lnNumType"))
    if existing is not None:
        return existing

    ln_num_type = OxmlElement("w:lnNumType")
    for tag in ("w:pgNumType", "w:cols", "w:formProt", "w:vAlign", "w:noEndnote"):
        anchor = sect_pr.find(qn(tag))
        if anchor is not None:
            anchor.addprevious(ln_num_type)
            return ln_num_type
    sect_pr.append(ln_num_type)
    return ln_num_type


def apply_line_numbers(doc: DocumentObject, restart: str) -> int:
    """Enable line numbers on every document section using the given restart mode."""
    updated = 0
    for section in doc.sections:
        ln_num_type = get_or_add_line_number_type(section._sectPr)
        ln_num_type.set(qn("w:countBy"), "1")
        ln_num_type.set(qn("w:restart"), restart)
        updated += 1
    return updated


def apply_line_number_metadata(doc: DocumentObject, metadata: dict[str, Any]) -> dict[str, Any] | None:
    """Apply show-line-numbers metadata to a DOCX document."""
    restart = line_number_setting_from_metadata(metadata)
    if restart is None:
        return None
    return {
        "restart": restart,
        "sections": apply_line_numbers(doc, restart),
    }


def process_file(
    docx_path: str,
    md_path: str,
    save: bool = True,
    metadata_files: list[str | Path] | None = None,
) -> dict[str, Any] | None:
    """Process a DOCX file using merged show-line-numbers metadata."""
    doc, docx_file = open_docx(docx_path)
    if doc is None or docx_file is None:
        return None
    md_file = validate_existing_file(md_path, "Markdown file")
    if md_file is None:
        return None

    metadata = load_merged_metadata(md_file, metadata_files)
    result = apply_line_number_metadata(doc, metadata)
    if result is None:
        print_debug("No enabled show-line-numbers metadata found, skipping")
        return None

    if save:
        save_docx(doc, docx_file)
    print_debug_success(f"Applied line numbers: restart={result['restart']}, sections={result['sections']}")
    return result


def main() -> None:
    """Main entry point for command-line usage."""
    parser = argparse.ArgumentParser(description="Apply DOCX line-number settings from merged YAML metadata")
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
