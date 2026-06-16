#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = [
#   "python-docx>=1.1.0",
#   "pyyaml>=6.0",
# ]
# ///
"""
Post-process DOCX file - orchestrator script.
This script calls individual processing scripts in sequence.

Usage:
    import postprocess_docx
    postprocess_docx.postprocess_docx("path/to/file.docx", metadata)
"""

import argparse
import json
import sys
from pathlib import Path
from typing import Any, Callable

from logging_utils import log_debug, log_info, log_success

try:
    from docx import Document
except ImportError:
    print("Error: python-docx is not installed. Install it with: pip install python-docx")
    sys.exit(1)

# Import processing modules
try:
    from postprocess.common import print_error, print_debug_success, print_warning
    from postprocess.merge_table_cells import merge_table_cells
    from postprocess.process_table_metadata import process_table_metadata
    from postprocess.autofit_tables import autofit_tables
    from postprocess.table_text_style import process_all_tables as convert_table_text_style, ensure_table_text_style_exists
    from postprocess.para_after_table_style import process_para_after_table_style
    from postprocess.insert_author_info import insert_author_info_to_doc
    from postprocess.clear_subfigure_table_format import clear_subfigure_table_format
    from postprocess.format_equation_layout_tables import format_equation_layout_tables
    from postprocess.body_text_style import apply_body_text_style_metadata
    from postprocess.inline_math_spacing import add_space_after_standalone_inline_math
    from postprocess.line_numbers import apply_line_number_metadata
    from postprocess.where_paragraph_style import process_where_paragraph_styles
    from postprocess.reply_blue_italic_style import apply_reply_blue_italic_style
except ImportError as e:
    print(f"Error: Failed to import processing modules: {e}")
    print("Make sure all scripts are in the same directory:")
    print("  - metadata.py")
    print("  - merge_table_cells.py")
    print("  - process_table_metadata.py")
    print("  - autofit_tables.py")
    print("  - table_text_style.py")
    print("  - para_after_table_style.py")
    print("  - insert_author_info.py")
    print("  - clear_subfigure_table_format.py")
    print("  - format_equation_layout_tables.py")
    print("  - body_text_style.py")
    print("  - inline_math_spacing.py")
    print("  - line_numbers.py")
    print("  - where_paragraph_style.py")
    print("  - reply_blue_italic_style.py")
    sys.exit(1)


def run_pipeline_step(label: str, action: Callable[[], None]) -> None:
    """Run one post-processing step with consistent pipeline logging."""
    log_info(f"[postprocess] Step: {label}...")
    try:
        action()
        log_debug("[postprocess] Step completed")
    except Exception as e:
        print_error(f"Step failed: {e}")
        raise


def log_skip(label: str, reason: str) -> None:
    """Log a skipped optional pipeline step using the same step format."""
    log_info(f"[postprocess] Step: Skipping {label} ({reason})")


def postprocess_docx(
    docx_path: str,
    metadata: dict[str, Any] | None = None,
    skip_author_info: bool = False,
    reply_style_formatting: bool = False,
) -> bool:
    """
    Post-process a DOCX file with all processing steps.

    Args:
        docx_path: Path to the DOCX file to process
        metadata: Merged style and manuscript metadata from the build layer
        skip_author_info: Skip author insertion for non-manuscript outputs
        reply_style_formatting: Apply reply-only blue italic caption/table styling

    Returns:
        True if successful, False otherwise
    """
    # Validate inputs
    log_info("[postprocess] Validating inputs...")
    docx_file = Path(docx_path)
    metadata = metadata or {}

    if not docx_file.exists():
        print_error(f"DOCX file not found: {docx_path}")
        return False

    docx_path_abs = docx_file.resolve()
    log_info("[postprocess] Starting DOCX post-processing pipeline")
    log_info(f"[postprocess] Target file: {docx_path_abs}")
    log_debug(f"[postprocess] Metadata keys: {len(metadata)}")

    try:
        # Open document (shared across all steps)
        log_info("[postprocess] Initializing document...")
        doc = Document(str(docx_path_abs))
        log_debug("[postprocess] Document opened successfully")

        if skip_author_info:
            log_skip("author information", "disabled for this build")
        else:
            def insert_author_info_step() -> None:
                """Insert author metadata and log the number of inserted records."""
                authors, affiliations, has_footnote = insert_author_info_to_doc(doc, metadata)
                if authors > 0:
                    print_debug_success(
                        f"Authors: {authors}, Affiliations: {affiliations}, "
                        f"Footnote: {'Yes' if has_footnote else 'No'}"
                    )
                else:
                    print_warning("No authors found in YAML metadata, skipping")

            run_pipeline_step("Inserting author information", insert_author_info_step)

        def apply_body_text_style_step() -> None:
            """Apply merged YAML bodyText metadata to the DOCX body style."""
            result = apply_body_text_style_metadata(doc, metadata)
            if result is None:
                print_warning("No bodyText metadata found, skipping")
                return
            print_debug_success(
                f"Style '{result['style_name']}': "
                f"first-line indent {result['first_line_indent_chars']} chars, "
                f"before {result['space_before_pt']} pt, "
                f"after {result['space_after_pt']} pt"
            )

        def apply_line_number_step() -> None:
            """Apply merged YAML line-number metadata to all DOCX sections."""
            result = apply_line_number_metadata(doc, metadata)
            if result is None:
                print_warning("No enabled show-line-numbers metadata found, skipping")
                return
            print_debug_success(
                f"Line numbers: restart={result['restart']}, sections={result['sections']}"
            )

        def merge_table_cells_step() -> None:
            """Merge table cells marked with left/up merge placeholders."""
            left_merges, up_merges = merge_table_cells(doc)
            print_debug_success(f"Left merges: {left_merges}, Up merges: {up_merges}")

        def process_table_metadata_step() -> None:
            """Apply table caption metadata and report the applied setting count."""
            processed, settings = process_table_metadata(doc)
            print_debug_success(f"Processed {processed} table(s), Applied {settings} setting(s)")

        def clear_subfigure_table_format_step() -> None:
            """Clear formatting from tables used only for subfigure layout."""
            processed_count = clear_subfigure_table_format(doc)
            print_debug_success(f"Cleared formatting for {processed_count} subfigure table(s)")

        def convert_table_text_style_step() -> None:
            """Convert table paragraphs from Compact to the shared Table Text style."""
            if not ensure_table_text_style_exists(doc):
                print_warning("Could not ensure Table Text style exists, skipping style conversion")
                return
            stats = convert_table_text_style(doc)
            print_debug_success(f"Converted {stats['converted']} paragraph(s) from 'Compact' to 'Table Text'")

        def apply_para_after_table_style_step() -> None:
            """Style regular body paragraphs that directly follow tables."""
            para_after_table_count = process_para_after_table_style(doc)
            if para_after_table_count is None:
                print_warning("Could not ensure Para After Table style exists, skipping style conversion")
                return
            print_debug_success(f"Styled {para_after_table_count} paragraph(s) as 'Para After Table'")

        def autofit_tables_step() -> None:
            """Auto-fit regular tables while leaving equation layout tables alone."""
            fitted_count = autofit_tables(doc, center_align=True)
            print_debug_success(f"Auto-fitted {fitted_count} table(s)")

        def format_equation_layout_tables_step() -> None:
            """Hide borders and tune widths for equation layout tables."""
            equation_table_count = format_equation_layout_tables(doc)
            print_debug_success(f"Formatted {equation_table_count} equation layout table(s)")

        def apply_where_paragraph_style_step() -> None:
            """Style where clauses that immediately follow equations."""
            where_count = process_where_paragraph_styles(doc)
            if where_count is None:
                print_warning("Could not ensure Where Paragraph style exists, skipping style conversion")
                return
            print_debug_success(f"Styled {where_count} paragraph(s) as 'Where Paragraph'")

        def add_inline_math_spacing_step() -> None:
            """Add a trailing space to standalone inline math paragraphs for Word rendering."""
            fixed_count = add_space_after_standalone_inline_math(doc)
            print_debug_success(f"Fixed {fixed_count} standalone inline math paragraph(s)")

        def apply_reply_blue_italic_style_step() -> None:
            """Apply reply-only blue italic formatting to captions and regular tables."""
            stats = apply_reply_blue_italic_style(doc)
            print_debug_success(
                f"Formatted {stats['caption_styles']} caption style(s), "
                f"{stats['caption_paragraphs']} caption paragraph(s), "
                f"{stats['table_runs']} table run(s)"
            )

        # Keep this ordered list explicit because DOCX post-processing steps are order-sensitive.
        pipeline_steps: list[tuple[str, Callable[[], None]]] = [
            # Metadata-driven document-wide settings must run before table-specific cleanup.
            ("Applying body text style metadata", apply_body_text_style_step),
            ("Applying line-number metadata", apply_line_number_step),
            ("Merging table cells", merge_table_cells_step),
            ("Processing table metadata", process_table_metadata_step),
            ("Clearing subfigure table formatting", clear_subfigure_table_format_step),
            ("Converting table text style", convert_table_text_style_step),
            ("Applying post-table paragraph style", apply_para_after_table_style_step),
            ("Auto-fitting tables to window", autofit_tables_step),
            ("Formatting equation layout tables", format_equation_layout_tables_step),
            ("Applying where paragraph style", apply_where_paragraph_style_step),
            ("Adding spaces after standalone inline math", add_inline_math_spacing_step),
        ]
        if reply_style_formatting:
            pipeline_steps.append(("Applying reply blue italic caption/table style", apply_reply_blue_italic_style_step))

        for label, action in pipeline_steps:
            run_pipeline_step(label, action)

        # ===================================================================
        # Save document (all changes from all scripts)
        # ===================================================================
        log_info("[postprocess] Saving all changes to document...")
        doc.save(str(docx_path_abs))
        log_success("[postprocess] Document saved successfully")

        log_success("[postprocess] DOCX post-processing completed successfully")
        return True

    except Exception as e:
        print_error("=== Post-Processing Pipeline Failed ===")
        print_error(f"Error: {e}")
        import traceback
        traceback.print_exc()
        return False


def main():
    """Main entry point for command-line usage"""
    parser = argparse.ArgumentParser(
        description="Post-process DOCX files with table formatting and metadata",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  Prefer calling postprocess_docx.postprocess_docx(docx_path, metadata) from build.py.
  For standalone debugging, pass a pre-merged metadata JSON file with --metadata-json.

Processing steps:
  - Insert author information from merged metadata (if metadata provided)
  - Apply Body Text style settings from merged metadata (if metadata provided)
  - Apply line numbers from show-line-numbers metadata (if metadata provided)
  - Merge table cells based on markers (!<! and !^!)
  - Process table metadata from captions (|key=value|)
  - Clear formatting for tables above 'Image Caption' paragraphs
  - Convert table text style from 'Compact' to 'Table Text'
  - Apply 'Para After Table' style to body paragraphs after tables
  - Auto-fit tables to window width and center align
  - Format equation layout tables
  - Apply 'Where Paragraph' style after equation paragraphs
  - Add trailing spaces after standalone inline math
  - Optionally apply reply blue italic caption/table styling

This script applies all post-processing steps in sequence.
        """
    )
    parser.add_argument("docx_path", help="Path to the DOCX file to process")
    parser.add_argument(
        "--metadata-json",
        help="Path to a pre-merged metadata JSON file produced by the build layer",
    )
    parser.add_argument(
        "--skip-author-info",
        action="store_true",
        help="Skip author insertion while keeping the remaining DOCX post-processing steps",
    )
    parser.add_argument(
        "--reply-style-formatting",
        action="store_true",
        help="Apply reply-only blue italic formatting to captions and regular table text",
    )

    args = parser.parse_args()
    metadata = None
    if args.metadata_json:
        with Path(args.metadata_json).open("r", encoding="utf-8") as f:
            metadata = json.load(f)
        if not isinstance(metadata, dict):
            parser.error("--metadata-json must contain a JSON object")

    success = postprocess_docx(
        args.docx_path,
        metadata,
        skip_author_info=args.skip_author_info,
        reply_style_formatting=args.reply_style_formatting,
    )
    sys.exit(0 if success else 1)


if __name__ == "__main__":
    main()
