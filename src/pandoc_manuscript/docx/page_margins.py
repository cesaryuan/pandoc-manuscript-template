"""Prepare DOCX reference documents with metadata-driven page margins."""

from __future__ import annotations

import re
import shutil
from pathlib import Path
from typing import Any

from docx import Document
from docx.document import Document as DocumentObject
from docx.shared import Cm, Inches, Mm, Pt

from ..runtime.metadata import PmtSettings

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


def normalize_page_margins(settings: PmtSettings) -> tuple[dict[str, Any], dict[str, str]] | None:
    """Normalize docxPageMargins into section attributes without defaulting missing sides."""
    raw_margins = settings.docx_page_margins
    if raw_margins is None:
        return None

    margins: dict[str, Any] = {}
    display_values: dict[str, str] = {}
    for side, aliases in MARGIN_SIDE_ALIASES.items():
        value = first_present(raw_margins, aliases)
        if value is None:
            continue
        margins[side] = parse_margin_length(value, f"docxPageMargins.{side}")
        display_values[side] = str(value)

    return margins, display_values


def apply_page_margins(doc: DocumentObject, margins: dict[str, Any]) -> int:
    """Apply normalized page margins to every section in a DOCX document."""
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


def apply_page_margin_settings(doc: DocumentObject, settings: PmtSettings) -> dict[str, Any] | None:
    """Apply typed DOCX page-margin settings to a document object."""
    normalized = normalize_page_margins(settings)
    if normalized is None:
        return None

    margins, display_values = normalized
    if not margins:
        return None

    return {
        "margins": display_values,
        "sections": apply_page_margins(doc, margins),
    }


def write_reference_doc_with_page_margins(
    source_docx: Path,
    target_docx: Path,
    settings: PmtSettings,
) -> dict[str, Any] | None:
    """Copy a reference DOCX and apply docxPageMargins before Pandoc reads it.

    Pandoc sizes DOCX images from the reference document's writable page width,
    so page margins must be present in the reference DOCX instead of applied
    after conversion.
    """
    normalized = normalize_page_margins(settings)
    if normalized is None:
        return None

    margins, display_values = normalized
    if not margins:
        return None
    if not source_docx.exists():
        raise FileNotFoundError(f"Reference DOCX not found: {source_docx}")

    target_docx.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(source_docx, target_docx)
    doc = Document(str(target_docx))
    sections = apply_page_margins(doc, margins)
    doc.save(str(target_docx))
    return {
        "source": source_docx,
        "target": target_docx,
        "margins": display_values,
        "sections": sections,
    }
