#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = [
#   "pywin32>=306; platform_system == 'Windows'",
# ]
# ///
"""Compare two DOCX files with Microsoft Word COM and save tracked changes.

This Windows-only script delegates the comparison to Word's own Compare engine.
It requires Microsoft Word and pywin32, but it produces the closest result to
using Word's Review > Compare UI by hand.
"""

import argparse
import sys
from pathlib import Path
from typing import Any


WD_ALERTS_NONE = 0
WD_COMPARE_DESTINATION_NEW = 2
WD_DO_NOT_SAVE_CHANGES = 0
WD_FORMAT_XML_DOCUMENT = 12
WD_GRANULARITY_CHAR_LEVEL = 0
WD_GRANULARITY_WORD_LEVEL = 1


def print_info(message: str) -> None:
    """Print an informational progress message."""
    print(f"[INFO] {message}")


def print_error(message: str) -> None:
    """Print an error message to stderr."""
    print(f"[ERROR] {message}", file=sys.stderr)


def parse_args() -> argparse.Namespace:
    """Parse command-line arguments for the Word COM compare helper."""
    parser = argparse.ArgumentParser(
        description=(
            "Use Microsoft Word COM to compare two DOCX files and save a DOCX "
            "containing Word tracked changes."
        )
    )
    parser.add_argument("old_docx", help="Older/original DOCX file")
    parser.add_argument("new_docx", help="Newer/revised DOCX file")
    parser.add_argument("output_docx", help="Output DOCX file with Word comparison revisions")
    parser.add_argument(
        "--author",
        default="Pandoc Manuscript Diff",
        help="Revision author shown by Word for comparison changes",
    )
    parser.add_argument(
        "--allow-overwrite",
        action="store_true",
        help="Allow replacing an existing output file",
    )
    parser.add_argument(
        "--visible",
        action="store_true",
        help="Show Word while comparing; useful when debugging Word prompts",
    )
    parser.add_argument(
        "--char-level",
        action="store_true",
        help="Compare at character granularity instead of Word's word-level default",
    )
    parser.add_argument(
        "--ignore-formatting",
        action="store_true",
        help="Ignore formatting-only differences during comparison",
    )
    parser.add_argument(
        "--ignore-moves",
        action="store_true",
        help="Do not ask Word to detect moved text",
    )
    parser.add_argument(
        "--keep-word-open",
        action="store_true",
        help="Leave Word open after saving the result for manual inspection",
    )
    return parser.parse_args()


def validate_docx_path(path: str | Path, label: str) -> Path:
    """Validate that a DOCX path exists and looks like a file."""
    candidate = Path(path)
    if not candidate.exists():
        raise FileNotFoundError(f"{label} not found: {candidate}")
    if not candidate.is_file():
        raise ValueError(f"{label} is not a file: {candidate}")
    if candidate.suffix.lower() != ".docx":
        raise ValueError(f"{label} must use a .docx extension: {candidate}")
    return candidate.resolve()


def prepare_output_path(path: str | Path, allow_overwrite: bool) -> Path:
    """Validate and prepare an output DOCX path before Word writes it."""
    output = Path(path)
    if output.exists() and not allow_overwrite:
        raise FileExistsError(f"Output already exists; pass --allow-overwrite to replace it: {output}")
    if output.exists() and output.is_dir():
        raise ValueError(f"Output path is a directory: {output}")
    output.parent.mkdir(parents=True, exist_ok=True)
    return output.resolve()


def import_word_com() -> Any:
    """Import pywin32's COM client, raising a clear dependency error if missing."""
    try:
        import win32com.client  # type: ignore[import-not-found]
    except ImportError as exc:
        raise RuntimeError(
            "pywin32 is required for Word COM automation. Run with `uv run` "
            "or install it with `pip install pywin32` on Windows."
        ) from exc
    return win32com.client


def open_word_document(word: Any, path: Path) -> Any:
    """Open a DOCX read-only in Word for comparison.

    OpenAndRepair helps with generated DOCX files that Word can repair
    interactively but would otherwise refuse in a silent COM run.
    """
    print_info(f"Opening: {path}")
    return word.Documents.Open(
        FileName=str(path),
        ConfirmConversions=False,
        ReadOnly=True,
        AddToRecentFiles=False,
        Visible=False,
        OpenAndRepair=True,
    )


def remove_existing_output(output_docx: Path, allow_overwrite: bool) -> None:
    """Remove an allowed existing output file before Word SaveAs2 writes it."""
    if not output_docx.exists():
        return
    if not allow_overwrite:
        raise FileExistsError(f"Output already exists; pass --allow-overwrite to replace it: {output_docx}")
    # Word SaveAs2 can still prompt or fail on an existing file; deleting only
    # the explicit output path keeps overwrite behavior deterministic.
    output_docx.unlink()


def compare_documents_with_word(
    old_docx: Path,
    new_docx: Path,
    output_docx: Path,
    author: str,
    visible: bool,
    char_level: bool,
    compare_formatting: bool,
    compare_moves: bool,
    allow_overwrite: bool,
    keep_word_open: bool,
) -> None:
    """Use Word COM to compare two DOCX files and save the comparison document."""
    win32com_client = import_word_com()
    word = None
    original_doc = None
    revised_doc = None
    compared_doc = None
    should_quit_word = True

    try:
        print_info("Starting Microsoft Word COM instance...")
        # DispatchEx starts an isolated Word instance so this script does not
        # accidentally close or reuse the user's already-open Word session.
        word = win32com_client.DispatchEx("Word.Application")
        word.Visible = bool(visible)
        word.DisplayAlerts = WD_ALERTS_NONE

        original_doc = open_word_document(word, old_docx)
        revised_doc = open_word_document(word, new_docx)
        granularity = WD_GRANULARITY_CHAR_LEVEL if char_level else WD_GRANULARITY_WORD_LEVEL

        print_info("Running Word Compare...")
        compared_doc = word.CompareDocuments(
            OriginalDocument=original_doc,
            RevisedDocument=revised_doc,
            Destination=WD_COMPARE_DESTINATION_NEW,
            Granularity=granularity,
            CompareFormatting=compare_formatting,
            CompareCaseChanges=True,
            CompareWhitespace=True,
            CompareTables=True,
            CompareHeaders=True,
            CompareFootnotes=True,
            CompareTextboxes=True,
            CompareFields=True,
            CompareComments=True,
            CompareMoves=compare_moves,
            RevisedAuthor=author,
            IgnoreAllComparisonWarnings=False,
        )

        remove_existing_output(output_docx, allow_overwrite)
        print_info(f"Saving comparison result: {output_docx}")
        compared_doc.SaveAs2(FileName=str(output_docx), FileFormat=WD_FORMAT_XML_DOCUMENT)
        print_info("Comparison document saved")

        if keep_word_open:
            print_info("Leaving Word open as requested")
            should_quit_word = False
            word.Visible = True
    finally:
        if not keep_word_open:
            for doc in (compared_doc, revised_doc, original_doc):
                if doc is not None:
                    try:
                        doc.Close(SaveChanges=WD_DO_NOT_SAVE_CHANGES)
                    except Exception:
                        pass
        if word is not None and should_quit_word:
            word.Quit(SaveChanges=WD_DO_NOT_SAVE_CHANGES)


def main() -> int:
    """Run the Word COM DOCX comparison command-line workflow."""
    args = parse_args()
    try:
        old_docx = validate_docx_path(args.old_docx, "Old DOCX")
        new_docx = validate_docx_path(args.new_docx, "New DOCX")
        output_docx = prepare_output_path(args.output_docx, args.allow_overwrite)
        if old_docx == new_docx:
            raise ValueError("Old and new DOCX paths must be different files")
        if output_docx in {old_docx, new_docx}:
            raise ValueError("Output DOCX must be different from both inputs")

        compare_documents_with_word(
            old_docx=old_docx,
            new_docx=new_docx,
            output_docx=output_docx,
            author=args.author,
            visible=args.visible,
            char_level=args.char_level,
            compare_formatting=not args.ignore_formatting,
            compare_moves=not args.ignore_moves,
            allow_overwrite=args.allow_overwrite,
            keep_word_open=args.keep_word_open,
        )
        print_info(f"Output written: {output_docx}")
        return 0
    except Exception as exc:
        print_error(str(exc))
        return 1


if __name__ == "__main__":
    sys.exit(main())
