"""Center regular tables in generated standalone HTML documents."""

from __future__ import annotations

from lxml import etree


def _set_css_property(element: etree._Element, property_name: str, value: str) -> None:
    """Set one inline CSS property while preserving unrelated declarations."""
    declarations: list[str] = []
    replaced = False
    for declaration in element.get("style", "").split(";"):
        declaration = declaration.strip()
        if not declaration or ":" not in declaration:
            continue
        name, current_value = declaration.split(":", 1)
        if name.strip().casefold() == property_name.casefold():
            if not replaced:
                declarations.append(f"{property_name}: {value}")
                replaced = True
            continue
        declarations.append(f"{name.strip()}: {current_value.strip()}")
    if not replaced:
        declarations.append(f"{property_name}: {value}")
    element.set("style", "; ".join(declarations) + ";")


def center_html_tables(document: etree._ElementTree) -> int:
    """Center every generated table with CSS auto margins and return its count."""
    tables = document.getroot().xpath(".//table")
    for table in tables:
        _set_css_property(table, "margin-left", "auto")
        _set_css_property(table, "margin-right", "auto")
    return len(tables)
