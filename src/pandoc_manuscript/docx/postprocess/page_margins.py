#!/usr/bin/env python3
"""Standalone utility for applying DOCX page margins to an existing DOCX."""

import argparse
import sys
from pathlib import Path
from typing import Any

from ...runtime.metadata import load_merged_metadata
from ..page_margins import (
    apply_page_margin_metadata,
    apply_page_margins,
    first_present,
    get_page_margin_metadata,
    normalize_page_margins,
    parse_margin_length,
)
from .common import open_docx, print_debug, print_debug_success, save_docx, validate_existing_file


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
