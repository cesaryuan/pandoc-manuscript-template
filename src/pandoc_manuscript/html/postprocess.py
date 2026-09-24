"""Ordered HTML post-processing pipeline for Papper manuscript builds."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from ..runtime.logging import log_debug, log_success
from .chinese_numbering import normalize_chinese_numbering
from .common import load_html_document, save_html_document
from .insert_author_info import insert_author_info
from .merge_table_cells import merge_table_cells


def postprocess_html(
    html_path: str | Path,
    *,
    pandoc_metadata: dict[str, Any] | None = None,
    chinese_mode: bool = False,
    skip_author_info: bool = False,
) -> bool:
    """Apply HTML counterparts of the manuscript's DOCX post-processing steps."""
    path = Path(html_path)
    if not path.exists():
        log_debug(f"[HTML] Output not found, skipping post-processing: {path}")
        return False

    document = load_html_document(path)
    metadata = pandoc_metadata or {}
    if skip_author_info:
        log_debug("[HTML] Skipping author information")
    else:
        authors, affiliations, footnote = insert_author_info(document, metadata)
        log_debug(
            f"[HTML] Authors: {authors}, Affiliations: {affiliations}, "
            f"Footnote: {'yes' if footnote else 'no'}"
        )

    if chinese_mode:
        numbering = normalize_chinese_numbering(document)
        log_debug(
            f"[HTML] Chinese numbering: headings={numbering['headings']}, "
            f"section_references={numbering['section_references']}"
        )

    left_merges, up_merges = merge_table_cells(document)
    log_debug(f"[HTML] Table merges: left={left_merges}, up={up_merges}")
    save_html_document(document, path)
    log_success(f"[OK] HTML post-processing completed: {path}")
    return True
