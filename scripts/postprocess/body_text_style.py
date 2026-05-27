#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = [
#   "python-docx>=1.1.0",
#   "pyyaml>=6.0",
# ]
# ///
"""
Apply DOCX body text style settings from manuscript YAML metadata.

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

try:
    import yaml
    from docx import Document
    from docx.document import Document as DocumentObject
    from docx.oxml import OxmlElement
    from docx.oxml.ns import qn
    from docx.shared import Pt
except ImportError as e:
    print(f"Error: Missing dependency: {e}")
    print("Install with: pip install python-docx pyyaml")
    sys.exit(1)


DEFAULT_FIRST_LINE_INDENT_CHARS = 2.0
DEFAULT_SPACE_BEFORE_PT = 0.0
DEFAULT_SPACE_AFTER_PT = 0.0
BODY_TEXT_METADATA_KEYS = ("bodyText", "body-text", "body_text", "docxBodyText", "docx-body-text")
BODY_TEXT_STYLE_NAMES = ("Body Text", "正文文本")


class Colors:
    """ANSI color codes for terminal output."""
    GREEN = '\033[92m'
    CYAN = '\033[96m'
    YELLOW = '\033[93m'
    RED = '\033[91m'
    RESET = '\033[0m'


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


def parse_yaml_header(md_path: str | Path) -> dict[str, Any]:
    """Parse YAML front matter from a markdown file."""
    content = Path(md_path).read_text(encoding='utf-8')
    match = re.match(r'^---\s*\n(.*?)\n---', content, re.DOTALL)
    if not match:
        raise ValueError("No YAML front matter found in markdown file")

    metadata = yaml.safe_load(match.group(1)) or {}
    if not isinstance(metadata, dict):
        raise ValueError("YAML front matter must be a mapping")
    return metadata


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


def get_body_text_style(doc: DocumentObject):
    """Return the available body text paragraph style."""
    for style_name in BODY_TEXT_STYLE_NAMES:
        try:
            return doc.styles[style_name], style_name
        except KeyError:
            continue
    return None, None


def set_style_first_line_indent_chars(style, chars: float) -> None:
    """Set a style first-line indent using Word's character-based OOXML value."""
    if chars < 0:
        raise ValueError("bodyText.firstLineIndentChars must be greater than or equal to 0")

    p_pr = style.element.get_or_add_pPr()
    ind = p_pr.find(qn('w:ind'))
    if ind is None:
        ind = OxmlElement('w:ind')
        p_pr.append(ind)

    # Word stores character indents in hundredths of a character.
    ind.set(qn('w:firstLineChars'), str(int(round(chars * 100))))
    for attr_name in ('w:firstLine', 'w:hanging', 'w:hangingChars'):
        attr = qn(attr_name)
        if attr in ind.attrib:
            del ind.attrib[attr]


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


def apply_body_text_style_metadata(doc: DocumentObject, md_path: str | Path) -> dict[str, Any] | None:
    """Read manuscript YAML metadata and apply body text style settings."""
    metadata = parse_yaml_header(md_path)
    settings = normalize_body_text_settings(metadata)
    if settings is None:
        return None
    return apply_body_text_settings(doc, settings)


def process_file(docx_path: str, md_path: str, save: bool = True) -> dict[str, Any] | None:
    """Process a DOCX file using body text metadata from a markdown file."""
    docx_file = Path(docx_path)
    md_file = Path(md_path)
    if not docx_file.exists():
        print_error(f"DOCX file not found: {docx_path}")
        return None
    if not md_file.exists():
        print_error(f"Markdown file not found: {md_path}")
        return None

    doc = Document(str(docx_file))
    result = apply_body_text_style_metadata(doc, md_file)
    if result is None:
        print_warning("No bodyText metadata found, skipping")
        return None

    if save:
        doc.save(str(docx_file))
    print_success(
        "Applied Body Text style: "
        f"first-line indent {result['first_line_indent_chars']} chars, "
        f"before {result['space_before_pt']} pt, after {result['space_after_pt']} pt"
    )
    return result


def main() -> None:
    """Main entry point for command-line usage."""
    parser = argparse.ArgumentParser(
        description="Apply DOCX Body Text style settings from manuscript YAML metadata"
    )
    parser.add_argument("docx_path", help="Path to the DOCX file to process")
    parser.add_argument("md_path", nargs="?", default="manuscript.md", help="Path to the markdown manuscript")
    parser.add_argument("--no-save", action="store_true", help="Do not save changes")
    args = parser.parse_args()

    result = process_file(args.docx_path, args.md_path, save=not args.no_save)
    sys.exit(0 if result is not None else 1)


if __name__ == "__main__":
    main()
