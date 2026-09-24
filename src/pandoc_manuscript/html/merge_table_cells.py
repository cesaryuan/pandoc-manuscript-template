"""Merge HTML table cells marked with Papper's DOCX-compatible markers."""

from __future__ import annotations

from lxml import etree

from .common import direct_table_cells, element_text


def _span(cell: etree._Element, name: str) -> int:
    """Read a positive HTML span attribute, defaulting to one."""
    try:
        return max(1, int(cell.get(name, "1")))
    except ValueError:
        return 1


def _add_span(cell: etree._Element, name: str, amount: int) -> None:
    """Increase one cell's colspan or rowspan by a marker cell's span."""
    value = _span(cell, name) + amount
    if value > 1:
        cell.set(name, str(value))


def _clear_marker(cell: etree._Element) -> None:
    """Remove marker contents before deleting or reusing the cell."""
    cell.clear()


def merge_table_cells(document: etree._ElementTree) -> tuple[int, int]:
    """Apply left and upward merges to top-level HTML tables."""
    left_count = 0
    up_count = 0
    root = document.getroot()
    tables = root.xpath("//table[not(ancestor::table)]")

    for table in tables:
        rows = table.xpath("./tr|./thead/tr|./tbody/tr|./tfoot/tr")
        for row in reversed(rows):
            cells = direct_table_cells(row)
            for index in range(len(cells) - 1, 0, -1):
                cell = cells[index]
                if element_text(cell) != "!<!":
                    continue
                _add_span(cells[index - 1], "colspan", _span(cell, "colspan"))
                cell.getparent().remove(cell)
                left_count += 1

        rows = table.xpath("./tr|./thead/tr|./tbody/tr|./tfoot/tr")
        for row_index in range(len(rows) - 1, 0, -1):
            cells = direct_table_cells(rows[row_index])
            above_cells = direct_table_cells(rows[row_index - 1])
            for index in range(len(cells) - 1, -1, -1):
                cell = cells[index]
                if element_text(cell) != "!^!" or index >= len(above_cells):
                    continue
                _add_span(above_cells[index], "rowspan", _span(cell, "rowspan"))
                cell.getparent().remove(cell)
                up_count += 1

    return left_count, up_count
