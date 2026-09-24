"""Normalize Chinese nested numbering in generated HTML."""

from __future__ import annotations

import re

from lxml import etree


HEADING_NUMBER_PATTERN = re.compile(r"(?P<number>\d+(?:-\d+)+)(?=(?:\s|[.。]|$))")
SECTION_REFERENCE_PATTERN = re.compile(r"节(?P<space>\s+)(?P<number>\d+(?:-\d+)+)")


def _replace_text_node(node: etree._Element, pattern: re.Pattern[str], replacement) -> bool:
    """Replace matches in one text node and report whether it changed."""
    if node.text is None:
        return False
    updated = pattern.sub(replacement, node.text)
    if updated == node.text:
        return False
    node.text = updated
    return True


def normalize_chinese_numbering(document: etree._ElementTree) -> dict[str, int]:
    """Change nested heading numbers and section references from hyphens to dots."""
    root = document.getroot()
    heading_count = 0
    section_reference_count = 0

    for heading in root.xpath("//h2|//h3|//h4|//h5|//h6"):
        changed = False
        for node in heading.iter():
            changed = _replace_text_node(
                node,
                HEADING_NUMBER_PATTERN,
                lambda match: match.group("number").replace("-", "."),
            ) or changed
        if changed:
            heading_count += 1

    for node in root.iter():
        if _replace_text_node(
            node,
            SECTION_REFERENCE_PATTERN,
            lambda match: f"节{match.group('space')}{match.group('number').replace('-', '.')}",
        ):
            section_reference_count += 1

    return {
        "headings": heading_count,
        "section_references": section_reference_count,
    }
