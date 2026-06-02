#!/usr/bin/env python3
"""
Probe low-level DOCX replacement of OMML with a MathType OLE object.

This script copies one known-good MathType OLE object from a sample DOCX and
uses it to replace OMML nodes in a target DOCX. It is a structural probe only:
the inserted MathType object contains the sample equation, not a conversion of
the original OMML content.
"""

import argparse
import copy
import itertools
import re
import shutil
import zipfile
import xml.etree.ElementTree as ET
from dataclasses import dataclass
from pathlib import Path


NS = {
    "w": "http://schemas.openxmlformats.org/wordprocessingml/2006/main",
    "m": "http://schemas.openxmlformats.org/officeDocument/2006/math",
    "r": "http://schemas.openxmlformats.org/officeDocument/2006/relationships",
    "v": "urn:schemas-microsoft-com:vml",
    "o": "urn:schemas-microsoft-com:office:office",
    "w14": "http://schemas.microsoft.com/office/word/2010/wordml",
    "rel": "http://schemas.openxmlformats.org/package/2006/relationships",
    "ct": "http://schemas.openxmlformats.org/package/2006/content-types",
}

REL_IMAGE = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/image"
REL_OLE = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/oleObject"
CONTENT_OLE = "application/vnd.openxmlformats-officedocument.oleObject"
CONTENT_WMF = "image/x-wmf"


for prefix, uri in NS.items():
    if prefix not in {"rel", "ct"}:
        ET.register_namespace(prefix, uri)


@dataclass
class MathTypeTemplate:
    """Package parts copied from a known-good MathType DOCX object."""

    object_element: ET.Element
    ole_bytes: bytes
    image_bytes: bytes


def qn(prefix: str, local: str) -> str:
    """Build a namespaced XML name."""
    return f"{{{NS[prefix]}}}{local}"


def read_xml(archive: zipfile.ZipFile, name: str) -> ET.Element:
    """Read and parse an XML part from a DOCX zip package."""
    return ET.fromstring(archive.read(name))


def serialize_xml(root: ET.Element) -> bytes:
    """Serialize an XML element with declaration."""
    return ET.tostring(root, encoding="utf-8", xml_declaration=True)


def find_next_numeric_id(existing: list[str], prefix: str) -> itertools.count:
    """Return a counter starting after the highest numeric suffix in existing IDs."""
    max_id = 0
    pattern = re.compile(rf"^{re.escape(prefix)}(\d+)$")
    for value in existing:
        match = pattern.match(value)
        if match:
            max_id = max(max_id, int(match.group(1)))
    return itertools.count(max_id + 1)


def extract_mathtype_template(sample_docx: Path) -> MathTypeTemplate:
    """Extract the first MathType OLE object, preview image, and object XML."""
    with zipfile.ZipFile(sample_docx) as archive:
        document = read_xml(archive, "word/document.xml")
        rels = read_xml(archive, "word/_rels/document.xml.rels")
        rel_by_id = {rel.get("Id"): rel.get("Target") for rel in rels}

        ole_object = document.find(".//w:object", NS)
        if ole_object is None:
            raise ValueError(f"No w:object found in sample DOCX: {sample_docx}")

        image = ole_object.find(".//v:imagedata", NS)
        ole = ole_object.find(".//o:OLEObject", NS)
        if image is None or ole is None:
            raise ValueError("Sample object is missing v:imagedata or o:OLEObject")

        image_target = rel_by_id[image.get(qn("r", "id"))]
        ole_target = rel_by_id[ole.get(qn("r", "id"))]

        return MathTypeTemplate(
            object_element=copy.deepcopy(ole_object),
            ole_bytes=archive.read(f"word/{ole_target}"),
            image_bytes=archive.read(f"word/{image_target}"),
        )


def collect_parent_map(root: ET.Element) -> dict[ET.Element, ET.Element]:
    """Build a child-to-parent map because ElementTree does not expose parents."""
    return {child: parent for parent in root.iter() for child in parent}


def top_level_omml_nodes(root: ET.Element) -> list[ET.Element]:
    """Return OMML nodes that can be replaced without also replacing nested math."""
    parent_map = collect_parent_map(root)
    result: list[ET.Element] = []
    for node in root.iter():
        if node.tag not in {qn("m", "oMath"), qn("m", "oMathPara")}:
            continue
        parent = parent_map.get(node)
        if parent is not None and parent.tag == qn("m", "oMathPara"):
            continue
        result.append(node)
    return result


def make_object_run(template: MathTypeTemplate, image_rid: str, ole_rid: str, index: int) -> ET.Element:
    """Create a Word run containing a cloned MathType OLE object."""
    run = ET.Element(qn("w", "r"))
    obj = copy.deepcopy(template.object_element)

    shape = obj.find(".//v:shape", NS)
    image = obj.find(".//v:imagedata", NS)
    ole = obj.find(".//o:OLEObject", NS)
    if shape is None or image is None or ole is None:
        raise ValueError("Template object is missing shape/image/OLE child")

    shape_id = f"_x0000_i{3000 + index}"
    shape.set("id", shape_id)
    image.set(qn("r", "id"), image_rid)
    ole.set(qn("r", "id"), ole_rid)
    ole.set("ShapeID", shape_id)
    # Word only requires ObjectID uniqueness inside the document.
    ole.set("ObjectID", f"_{1841932809 + index}")

    run.append(obj)
    return run


def append_relationship(rels_root: ET.Element, rid: str, rel_type: str, target: str) -> None:
    """Append a document relationship for a newly added part."""
    rel = ET.Element(f"{{{NS['rel']}}}Relationship")
    rel.set("Id", rid)
    rel.set("Type", rel_type)
    rel.set("Target", target)
    rels_root.append(rel)


def ensure_default_content_type(types_root: ET.Element, extension: str, content_type: str) -> None:
    """Ensure the DOCX content type table can resolve a newly added part extension."""
    for default in types_root.findall("ct:Default", NS):
        if default.get("Extension") == extension:
            default.set("ContentType", content_type)
            return
    default = ET.Element(f"{{{NS['ct']}}}Default")
    default.set("Extension", extension)
    default.set("ContentType", content_type)
    types_root.insert(0, default)


def replace_omml_with_template(source: Path, sample: Path, target: Path, limit: int) -> int:
    """Create a DOCX where the first OMML nodes are replaced by sample MathType OLE objects."""
    template = extract_mathtype_template(sample)
    target.parent.mkdir(parents=True, exist_ok=True)

    with zipfile.ZipFile(source) as in_zip:
        document = read_xml(in_zip, "word/document.xml")
        rels = read_xml(in_zip, "word/_rels/document.xml.rels")
        content_types = read_xml(in_zip, "[Content_Types].xml")
        existing_names = set(in_zip.namelist())

        existing_rids = [rel.get("Id", "") for rel in rels.findall("rel:Relationship", NS)]
        rid_counter = find_next_numeric_id(existing_rids, "rId")
        parent_map = collect_parent_map(document)

        replaced = 0
        added_parts: dict[str, bytes] = {}
        for node in top_level_omml_nodes(document):
            if replaced >= limit:
                break
            parent = parent_map[node]
            child_index = list(parent).index(node)

            replacement_index = replaced + 1
            image_rid = f"rId{next(rid_counter)}"
            ole_rid = f"rId{next(rid_counter)}"
            image_name = f"word/media/mathtype_probe_{replacement_index}.wmf"
            ole_name = f"word/embeddings/mathtype_probe_{replacement_index}.bin"

            parent.remove(node)
            parent.insert(child_index, make_object_run(template, image_rid, ole_rid, replacement_index))
            append_relationship(rels, image_rid, REL_IMAGE, image_name.removeprefix("word/"))
            append_relationship(rels, ole_rid, REL_OLE, ole_name.removeprefix("word/"))
            added_parts[image_name] = template.image_bytes
            added_parts[ole_name] = template.ole_bytes
            replaced += 1

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

    return replaced


def copy_for_probe(source: Path, target: Path) -> None:
    """Copy a source file to the probe output path."""
    target.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(source, target)


def main() -> int:
    """Run the low-level DOCX MathType OLE replacement probe."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", default="output/docx/manuscript.docx", help="DOCX containing OMML")
    parser.add_argument("--sample", default="mathtype.docx", help="DOCX containing one MathType OLE object")
    parser.add_argument("--target", default="tmp/mathtype-lowlevel-probe.docx", help="Output probe DOCX")
    parser.add_argument("--limit", type=int, default=1, help="Number of top-level OMML nodes to replace")
    parser.add_argument("--copy-only", action="store_true", help="Only copy the source DOCX")
    args = parser.parse_args()

    source = Path(args.source)
    sample = Path(args.sample)
    target = Path(args.target)

    if args.copy_only:
        copy_for_probe(source, target)
        print(f"[probe] copied {source} -> {target}")
        return 0

    replaced = replace_omml_with_template(source, sample, target, args.limit)
    print(f"[probe] wrote {target}")
    print(f"[probe] replaced top-level OMML nodes: {replaced}")
    return 0 if replaced else 1


if __name__ == "__main__":
    raise SystemExit(main())
