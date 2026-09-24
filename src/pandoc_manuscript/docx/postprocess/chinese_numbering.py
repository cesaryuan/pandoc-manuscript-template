"""Normalize Chinese DOCX heading and section-reference numbering."""

from __future__ import annotations

import re
from collections.abc import Callable, Iterable

from docx.document import Document as DocumentObject
from docx.oxml.ns import qn
from docx.table import Table
from docx.text.paragraph import Paragraph


HEADING_NUMBER_PATTERN = re.compile(r"(?P<number>\d+(?:-\d+)+)(?=(?:\t|\s|[.。]|$))")
SECTION_REFERENCE_PATTERN = re.compile(r"节(?P<space>\s+)(?P<number>\d+(?:-\d+)+)")


def iter_docx_paragraphs(doc: DocumentObject) -> Iterable[Paragraph]:
    """Yield body and table paragraphs, including paragraphs in nested tables."""
    yield from doc.paragraphs
    for table in doc.tables:
        yield from iter_table_paragraphs(table)


def iter_table_paragraphs(table: Table) -> Iterable[Paragraph]:
    """Yield paragraphs from a table and any nested tables."""
    for row in table.rows:
        for cell in row.cells:
            yield from cell.paragraphs
            for nested_table in cell.tables:
                yield from iter_table_paragraphs(nested_table)


def paragraph_heading_level(paragraph: Paragraph) -> int | None:
    """Return a built-in heading level, or None for non-heading paragraphs."""
    style_name = paragraph.style.name or ""
    match = re.search(r"(?:Heading|标题)\s*(\d+)", style_name, re.IGNORECASE)
    if match is None:
        style_id = paragraph.style.style_id or ""
        match = re.fullmatch(r"Heading(\d+)", style_id, re.IGNORECASE)
    return int(match.group(1)) if match else None


def replace_text_spans(
    paragraph: Paragraph,
    pattern: re.Pattern[str],
    replacement: Callable[[re.Match[str]], str],
) -> bool:
    """Replace regex matches across Word text nodes while preserving run formatting."""
    text_nodes = paragraph._p.findall(f".//{qn('w:t')}")
    node_texts = [node.text or "" for node in text_nodes]
    full_text = "".join(node_texts)
    matches = list(pattern.finditer(full_text))
    if not matches:
        return False

    for match in reversed(matches):
        start, end = match.span()
        replacement_text = replacement(match)
        offsets: list[tuple[int, int]] = []
        cursor = 0
        for node_text in node_texts:
            offsets.append((cursor, cursor + len(node_text)))
            cursor += len(node_text)
        first = next(index for index, (_, node_end) in enumerate(offsets) if start < node_end)
        last = next(index for index, (node_start, _) in enumerate(offsets) if end <= node_start + len(node_texts[index]))
        first_start = start - offsets[first][0]
        last_end = end - offsets[last][0]
        if first == last:
            node_texts[first] = node_texts[first][:first_start] + replacement_text + node_texts[first][last_end:]
            continue
        node_texts[first] = node_texts[first][:first_start] + replacement_text
        for index in range(first + 1, last):
            node_texts[index] = ""
        node_texts[last] = node_texts[last][last_end:]

    for node, node_text in zip(text_nodes, node_texts, strict=True):
        node.text = node_text
    return True


def normalize_chinese_numbering(doc: DocumentObject) -> dict[str, int]:
    """Change nested heading numbers and section references from hyphens to dots."""
    heading_count = 0
    section_reference_count = 0
    for paragraph in iter_docx_paragraphs(doc):
        heading_level = paragraph_heading_level(paragraph)
        if heading_level is not None and heading_level >= 2:
            heading_changed = False
            for text_node in paragraph._p.findall(f".//{qn('w:t')}"):
                node_text = text_node.text or ""
                match = HEADING_NUMBER_PATTERN.match(node_text)
                if match is None:
                    continue
                text_node.text = (
                    node_text[: match.start("number")]
                    + match.group("number").replace("-", ".")
                    + node_text[match.end("number") :]
                )
                heading_changed = True
                break
            if heading_changed:
                heading_count += 1
        if replace_text_spans(
            paragraph,
            SECTION_REFERENCE_PATTERN,
            lambda match: f"节{match.group('space')}{match.group('number').replace('-', '.')}",
        ):
            section_reference_count += 1
    return {
        "headings": heading_count,
        "section_references": section_reference_count,
    }
