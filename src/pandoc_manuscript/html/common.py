"""Shared HTML document helpers used by Papper's HTML post-processors."""

from __future__ import annotations

from pathlib import Path

from lxml import etree, html


def load_html_document(path: str | Path) -> etree._ElementTree:
    """Parse one generated HTML file while preserving its document structure."""
    return html.parse(str(path))


def save_html_document(document: etree._ElementTree, path: str | Path) -> None:
    """Write a standalone UTF-8 HTML document with a stable HTML5 doctype."""
    root = document.getroot()
    rendered = etree.tostring(
        root,
        method="html",
        encoding="unicode",
        doctype="<!DOCTYPE html>",
        pretty_print=True,
    )
    Path(path).write_text(rendered, encoding="utf-8")


def direct_table_cells(row: etree._Element) -> list[etree._Element]:
    """Return direct header/data cells from one HTML table row."""
    return row.xpath("./th|./td")


def element_text(element: etree._Element) -> str:
    """Return visible text from an HTML element, including nested inline markup."""
    return "".join(element.itertext()).strip()
