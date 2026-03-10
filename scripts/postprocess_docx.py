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
    uv run postprocess_docx.py path/to/file.docx path/to/manuscript.md
"""

import argparse
import sys
from pathlib import Path

try:
    from docx import Document
except ImportError:
    print("Error: python-docx is not installed. Install it with: pip install python-docx")
    sys.exit(1)

# Import processing modules
try:
    from postprocess.merge_table_cells import merge_table_cells
    from postprocess.process_table_metadata import process_table_metadata
    from postprocess.autofit_tables import autofit_tables
    from postprocess.table_text_style import process_all_tables as convert_table_text_style, ensure_table_text_style_exists
    from postprocess.insert_author_info import insert_author_info_to_doc
    from postprocess.clear_subfigure_table_format import clear_subfigure_table_format
except ImportError as e:
    print(f"Error: Failed to import processing modules: {e}")
    print("Make sure all scripts are in the same directory:")
    print("  - merge_table_cells.py")
    print("  - process_table_metadata.py")
    print("  - autofit_tables.py")
    print("  - table_text_style.py")
    print("  - insert_author_info.py")
    print("  - clear_subfigure_table_format.py")
    sys.exit(1)


class Colors:
    """ANSI color codes for terminal output"""
    GREEN = '\033[92m'
    CYAN = '\033[96m'
    YELLOW = '\033[93m'
    RED = '\033[91m'
    RESET = '\033[0m'


def print_success(message: str):
    """Print success message in green"""
    print(f"{Colors.GREEN}{message}{Colors.RESET}")


def print_info(message: str):
    """Print info message in cyan"""
    print(f"{Colors.CYAN}{message}{Colors.RESET}")


def print_warning(message: str):
    """Print warning message in yellow"""
    print(f"{Colors.YELLOW}{message}{Colors.RESET}")


def print_error(message: str):
    """Print error message in red"""
    print(f"{Colors.RED}{message}{Colors.RESET}")


def postprocess_docx(docx_path: str, md_path: str = '') -> bool:
    """
    Post-process a DOCX file with all processing steps.

    Args:
        docx_path: Path to the DOCX file to process
        md_path: Path to the markdown file with YAML metadata (optional)

    Returns:
        True if successful, False otherwise
    """
    # Validate inputs
    print_info("Validating inputs...")
    docx_file = Path(docx_path)

    if not docx_file.exists():
        print_error(f"DOCX file not found: {docx_path}")
        return False

    if md_path and not Path(md_path).exists():
        print_error(f"Markdown file not found: {md_path}")
        return False

    docx_path_abs = docx_file.resolve()
    print_info("=== Starting DOCX Post-Processing Pipeline ===")
    print_info(f"Target file: {docx_path_abs}")
    if md_path:
        print_info(f"Markdown file: {Path(md_path).resolve()}")
    print_info("")

    try:
        # Open document (shared across all steps)
        print_info("Initializing document...")
        doc = Document(str(docx_path_abs))
        print_success("Document opened successfully")
        print_info("")

        # ===================================================================
        # Step 1: Insert author information (if md_path provided)
        # ===================================================================
        if md_path:
            print_info("Step 1: Inserting author information...")
            try:
                authors, affiliations, has_footnote = insert_author_info_to_doc(doc, md_path)
                if authors > 0:
                    print_success(f"Authors: {authors}, Affiliations: {affiliations}, Footnote: {'Yes' if has_footnote else 'No'}")
                else:
                    print_warning("No authors found in YAML metadata, skipping")
                print_success("Step 1 completed")
                print_info("")
            except Exception as e:
                print_error(f"Step 1 failed: {e}")
                raise
        else:
            print_info("Step 1: Skipping author information (no markdown file provided)")
            print_info("")

        # ===================================================================
        # Step 2: Merge table cells based on markers
        # ===================================================================
        print_info("Step 2: Merging table cells...")
        try:
            left_merges, up_merges = merge_table_cells(doc)
            print_success(f"Left merges: {left_merges}, Up merges: {up_merges}")
            print_success("Step 2 completed")
            print_info("")
        except Exception as e:
            print_error(f"Step 2 failed: {e}")
            raise

        # ===================================================================
        # Step 3: Process table metadata from captions
        # ===================================================================
        print_info("Step 3: Processing table metadata...")
        try:
            processed, settings = process_table_metadata(doc)
            print_success(f"Processed {processed} table(s), Applied {settings} setting(s)")
            print_success("Step 3 completed")
            print_info("")
        except Exception as e:
            print_error(f"Step 3 failed: {e}")
            raise

        # ===================================================================
        # Step 4: Clear formatting for subfigure layout tables
        # ===================================================================
        print_info("Step 4: Clearing subfigure table formatting...")
        try:
            processed_count = clear_subfigure_table_format(doc)
            print_success(f"Cleared formatting for {processed_count} subfigure table(s)")
            print_success("Step 4 completed")
            print_info("")
        except Exception as e:
            print_error(f"Step 4 failed: {e}")
            raise
        
        
        # ===================================================================
        # Step 5: Convert table text style from Compact to Table Text
        # ===================================================================
        print_info("Step 5: Converting table text style...")
        try:
            # Ensure Table Text style exists
            if not ensure_table_text_style_exists(doc):
                print_warning("Could not ensure Table Text style exists, skipping style conversion")
            else:
                stats = convert_table_text_style(doc)
                print_success(f"Converted {stats['converted']} paragraph(s) from 'Compact' to 'Table Text'")
            print_success("Step 5 completed")
            print_info("")
        except Exception as e:
            print_error(f"Step 5 failed: {e}")
            raise

        # ===================================================================
        # Step 6: Auto-fit tables to window
        # ===================================================================
        print_info("Step 6: Auto-fitting tables to window...")
        try:
            fitted_count = autofit_tables(doc, center_align=True)
            print_success(f"Auto-fitted {fitted_count} table(s)")
            print_success("Step 6 completed")
            print_info("")
        except Exception as e:
            print_error(f"Step 6 failed: {e}")
            raise

        # ===================================================================
        # Save document (all changes from all scripts)
        # ===================================================================
        print_info("Saving all changes to document...")
        doc.save(str(docx_path_abs))
        print_success("Document saved successfully")

        print_success("=== Post-Processing Pipeline Completed Successfully ===")
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
  uv run postprocess_docx.py manuscript.docx manuscript.md
  uv run postprocess_docx.py output/docx/manuscript.docx manuscript.md

Processing steps:
  1. Insert author information from YAML metadata (if md_path provided)
  2. Merge table cells based on markers (!<! and !^!)
  3. Process table metadata from captions (|key=value|)
  4. Convert table text style from 'Compact' to 'Table Text'
  5. Clear formatting for tables above 'Image Caption' paragraphs
  6. Auto-fit tables to window width and center align

This script applies all post-processing steps in sequence.
        """
    )
    parser.add_argument("docx_path", help="Path to the DOCX file to process")
    parser.add_argument("md_path", nargs="?", default='manuscript.md', help="Path to the markdown file with YAML metadata (optional)")

    args = parser.parse_args()

    success = postprocess_docx(args.docx_path, args.md_path)
    sys.exit(0 if success else 1)


if __name__ == "__main__":
    main()
