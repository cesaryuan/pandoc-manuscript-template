"""Read hidden LaTeX markers and replace their bound OMML nodes in DOCX."""

import copy
import zipfile
import xml.etree.ElementTree as ET
from dataclasses import dataclass
from pathlib import Path

from .docx_ole import (
    MathTypeTemplate,
    append_relationship,
    collect_parent_map,
    ensure_default_content_type,
    extract_mathtype_template,
    find_next_numeric_id,
    make_object_run,
    qn,
    read_xml,
    serialize_xml,
    top_level_omml_nodes,
    CONTENT_OLE,
    CONTENT_WMF,
    REL_IMAGE,
    REL_OLE,
    NS,
)

from .ole_parts import GeneratedEquation


MATH_TYPE_MARKER_PREFIX = "MTLATEX:"


@dataclass
class MarkedFormulaBinding:
    """A hidden LaTeX marker bound to the OMML node that immediately follows it."""

    latex: str
    kind: str
    marker_run: ET.Element
    omml_node: ET.Element


def marker_from_run(run: ET.Element) -> tuple[str, str] | None:
    """Return (kind, latex) from a hidden MathType marker run, if present."""
    text = "".join(node.text or "" for node in run.findall(".//w:t", NS))
    if not text.startswith(MATH_TYPE_MARKER_PREFIX):
        return None
    remainder = text[len(MATH_TYPE_MARKER_PREFIX) :]
    if ":" not in remainder:
        raise ValueError(f"Malformed MathType marker: {text!r}")
    kind, latex = remainder.split(":", 1)
    if kind not in {"inline", "display"}:
        raise ValueError(f"Unknown MathType marker kind: {kind!r}")
    return kind, latex.strip()


def iter_marker_and_omml_signals(root: ET.Element):
    """Yield hidden marker runs and top-level OMML nodes in document order."""
    top_level_nodes = set(top_level_omml_nodes(root))

    def walk(node: ET.Element):
        if node in top_level_nodes:
            yield "omml", node
            return
        if node.tag == qn("w", "r") and marker_from_run(node) is not None:
            yield "marker", node
            return
        for child in list(node):
            yield from walk(child)

    yield from walk(root)


def find_marked_formula_bindings(root: ET.Element) -> list[MarkedFormulaBinding]:
    """Bind each hidden LaTeX marker to the next top-level OMML node."""
    bindings: list[MarkedFormulaBinding] = []
    pending_marker: tuple[str, str, ET.Element] | None = None

    for signal_type, node in iter_marker_and_omml_signals(root):
        if signal_type == "marker":
            marker = marker_from_run(node)
            if marker is None:
                continue
            if pending_marker is not None:
                _kind, _latex, marker_run = pending_marker
                raise ValueError(f"MathType marker was not followed by OMML: {marker_from_run(marker_run)!r}")
            kind, latex = marker
            pending_marker = (kind, latex, node)
            continue

        if pending_marker is None:
            continue
        kind, latex, marker_run = pending_marker
        bindings.append(MarkedFormulaBinding(latex=latex, kind=kind, marker_run=marker_run, omml_node=node))
        pending_marker = None

    if pending_marker is not None:
        _kind, _latex, marker_run = pending_marker
        raise ValueError(f"Trailing MathType marker was not followed by OMML: {marker_from_run(marker_run)!r}")
    return bindings


def extract_marked_latex_values(source: Path) -> list[str]:
    """Read LaTeX values from hidden marker runs in a DOCX document part."""
    with zipfile.ZipFile(source) as archive:
        document = read_xml(archive, "word/document.xml")
    return [binding.latex for binding in find_marked_formula_bindings(document)]


def remove_marker_run(parent_map: dict[ET.Element, ET.Element], marker_run: ET.Element, index: int) -> None:
    """Remove a marker run and its now-empty paragraph, if Pandoc emitted one."""
    marker_parent = parent_map.get(marker_run)
    if marker_parent is None:
        raise ValueError(f"marker has no parent for equation {index}")
    marker_parent.remove(marker_run)

    # Display math markers can live in their own hidden paragraph. After the run
    # is removed, delete that empty paragraph so the final DOCX has no blank line.
    if marker_parent.tag == qn("w", "p"):
        non_pr_children = [child for child in list(marker_parent) if child.tag != qn("w", "pPr")]
        if not non_pr_children:
            paragraph_parent = parent_map.get(marker_parent)
            if paragraph_parent is not None:
                paragraph_parent.remove(marker_parent)


def replace_marked_omml_with_generated(source: Path, sample: Path, target: Path, equations: list[GeneratedEquation]) -> int:
    """Replace marker-bound OMML nodes with generated MathType OLE objects."""
    template = extract_mathtype_template(sample)
    target.parent.mkdir(parents=True, exist_ok=True)

    with zipfile.ZipFile(source) as in_zip:
        document = read_xml(in_zip, "word/document.xml")
        rels = read_xml(in_zip, "word/_rels/document.xml.rels")
        content_types = read_xml(in_zip, "[Content_Types].xml")
        existing_names = set(in_zip.namelist())

        bindings = find_marked_formula_bindings(document)
        if len(bindings) != len(equations):
            raise ValueError(f"math count mismatch: markers={len(bindings)}, generated={len(equations)}")

        existing_rids = [rel.get("Id", "") for rel in rels.findall("rel:Relationship", NS)]
        rid_counter = find_next_numeric_id(existing_rids, "rId")
        parent_map = collect_parent_map(document)
        added_parts: dict[str, bytes] = {}

        for index, (binding, equation) in enumerate(zip(bindings, equations), start=1):
            if binding.latex != equation.latex:
                raise ValueError(f"marker/equation mismatch at {index}: {binding.latex!r} != {equation.latex!r}")

            image_rid = f"rId{next(rid_counter)}"
            ole_rid = f"rId{next(rid_counter)}"
            image_name = f"word/media/mathtype_formula_{index}.wmf"
            ole_name = f"word/embeddings/mathtype_formula_{index}.bin"

            item_template = MathTypeTemplate(
                object_element=copy.deepcopy(template.object_element),
                ole_bytes=equation.ole_path.read_bytes(),
                image_bytes=equation.wmf_path.read_bytes(),
                baseline_from_bottom_pt=equation.baseline_from_bottom_pt,
            )

            remove_marker_run(parent_map, binding.marker_run, index)

            parent = parent_map[binding.omml_node]
            child_index = list(parent).index(binding.omml_node)
            parent.remove(binding.omml_node)
            parent.insert(
                child_index,
                make_object_run(item_template, image_rid, ole_rid, index),
            )
            append_relationship(rels, image_rid, REL_IMAGE, image_name.removeprefix("word/"))
            append_relationship(rels, ole_rid, REL_OLE, ole_name.removeprefix("word/"))
            added_parts[image_name] = item_template.image_bytes
            added_parts[ole_name] = item_template.ole_bytes

        ensure_default_content_type(content_types, "bin", CONTENT_OLE)
        ensure_default_content_type(content_types, "wmf", CONTENT_WMF)
        replacements = {
            "word/document.xml": serialize_xml(document),
            "word/_rels/document.xml.rels": serialize_xml(rels),
            "[Content_Types].xml": serialize_xml(content_types),
        }

        with zipfile.ZipFile(target, "w", zipfile.ZIP_DEFLATED) as out_zip:
            for item in in_zip.infolist():
                if item.filename in replacements or item.filename in added_parts:
                    continue
                out_zip.writestr(item, in_zip.read(item.filename))
            for name, data in replacements.items():
                out_zip.writestr(name, data)
            for name, data in added_parts.items():
                if name in existing_names:
                    raise ValueError(f"Generated part already exists: {name}")
                out_zip.writestr(name, data)

    return len(equations)


def inspect_docx(path: Path) -> None:
    """Report DOCX package markers after injection."""
    with zipfile.ZipFile(path) as archive:
        names = archive.namelist()
        document_xml = archive.read("word/document.xml")
        rels_xml = archive.read("word/_rels/document.xml.rels")
        embeddings = [name for name in names if name.startswith("word/embeddings/")]
        previews = [name for name in names if name.startswith("word/media/mathtype_formula_")]
        print(f"[mathtype] docx={path}")
        print(f"[mathtype] embeddings={len(embeddings)}, Equation.DSMT4={document_xml.count(b'Equation.DSMT4')}")
        print(f"[mathtype] generated WMF previews={len(previews)}")
        print(f"[mathtype] oMath tokens={document_xml.count(b'<m:oMath') + document_xml.count(b'<m:oMathPara')}")
        print(f"[mathtype] MathType markers={document_xml.count(MATH_TYPE_MARKER_PREFIX.encode('utf-8'))}")
        print(f"[mathtype] ole relationships={rels_xml.count(b'/oleObject')}")
