#!/usr/bin/env python3
"""
Probe a no-Word MathType OLE path from LaTeX text to DOCX injection.

This intentionally stays outside the normal build path. It drives MathType's
OLE server directly, saves a DOCX-embeddable OLE compound file, then reuses the
existing low-level DOCX object injection code.
"""

import argparse
import copy
import json
import struct
import subprocess
import zipfile
from dataclasses import dataclass
from pathlib import Path

from probe_mathtype_docx_ole_replace import (
    MathTypeTemplate,
    append_relationship,
    collect_parent_map,
    ensure_default_content_type,
    extract_mathtype_template,
    find_next_numeric_id,
    make_object_run,
    qn,
    read_xml,
    replace_omml_with_template,
    serialize_xml,
    top_level_omml_nodes,
    CONTENT_OLE,
    CONTENT_WMF,
    REL_IMAGE,
    REL_OLE,
    NS,
)


HELPER_PROJECT = Path("scripts/mathtype_ole_helper/MathTypeOleHelper.csproj")
HELPER_EXE = Path("scripts/mathtype_ole_helper/bin/Debug/net9.0-windows/MathTypeOleHelper.exe")
END_OF_CHAIN = 0xFFFFFFFE
FREE_SECTOR = 0xFFFFFFFF
FAT_SECTOR = 0xFFFFFFFD
NO_STREAM = 0xFFFFFFFF


@dataclass
class DirectoryEntry:
    """A single CFB directory entry needed for stream extraction."""

    name: str
    object_type: int
    start_sector: int
    size: int


@dataclass
class GeneratedEquation:
    """Generated MathType object parts for a single Markdown math node."""

    latex: str
    ole_path: Path
    wmf_path: Path
    metadata_path: Path | None = None

    @property
    def baseline_from_bottom_pt(self) -> float | None:
        """Return MathType's baseline distance from the preview bottom.

        The helper records this in a sidecar JSON file because Word XML needs
        the value later when placing inline OLE equations.
        """
        if self.metadata_path is None or not self.metadata_path.exists():
            return None
        data = json.loads(self.metadata_path.read_text(encoding="utf-8-sig"))
        mathtype = data.get("mathtype")
        if not isinstance(mathtype, dict):
            return None
        value = mathtype.get("baseline_from_bottom_pt")
        if isinstance(value, (int, float)) and value > 0:
            return float(value)
        return None


class CompoundFile:
    """Minimal CFB reader for verifying MathType OLE streams."""

    def __init__(self, data: bytes):
        if data[:8] != bytes.fromhex("d0cf11e0a1b11ae1"):
            raise ValueError("not an OLE compound file")
        self.data = data
        self.sector_size = 1 << struct.unpack_from("<H", data, 30)[0]
        self.mini_sector_size = 1 << struct.unpack_from("<H", data, 32)[0]
        self.first_dir_sector = struct.unpack_from("<I", data, 48)[0]
        self.mini_cutoff = struct.unpack_from("<I", data, 56)[0]
        self.first_minifat_sector = struct.unpack_from("<I", data, 60)[0]
        self.num_minifat_sectors = struct.unpack_from("<I", data, 64)[0]
        self.fat = self._read_fat()
        self.entries = self._read_directory()
        self.root = next((entry for entry in self.entries if entry.object_type == 5), None)
        self.mini_fat = self._read_minifat()
        self.root_mini_stream = self._read_regular_stream(self.root) if self.root else b""

    def _sector(self, sector_id: int) -> bytes:
        offset = (sector_id + 1) * self.sector_size
        return self.data[offset : offset + self.sector_size]

    def _chain(self, start_sector: int, fat: list[int] | None = None) -> list[int]:
        table = self.fat if fat is None else fat
        chain: list[int] = []
        sector = start_sector
        while sector not in {END_OF_CHAIN, FREE_SECTOR, NO_STREAM}:
            if sector >= len(table):
                raise ValueError(f"sector chain points outside FAT: {sector}")
            chain.append(sector)
            sector = table[sector]
        return chain

    def _read_fat(self) -> list[int]:
        difat = [
            value
            for value in struct.unpack_from("<109I", self.data, 76)
            if value not in {FREE_SECTOR, END_OF_CHAIN}
        ]
        sectors: list[int] = []
        for fat_sector in difat:
            if fat_sector == FAT_SECTOR:
                continue
            sectors.extend(struct.unpack("<" + "I" * (self.sector_size // 4), self._sector(fat_sector)))
        return sectors

    def _read_directory(self) -> list[DirectoryEntry]:
        raw = b"".join(self._sector(sector) for sector in self._chain(self.first_dir_sector))
        entries: list[DirectoryEntry] = []
        for offset in range(0, len(raw), 128):
            chunk = raw[offset : offset + 128]
            name_len = struct.unpack_from("<H", chunk, 64)[0]
            if name_len < 2:
                continue
            name = chunk[: name_len - 2].decode("utf-16le", errors="replace")
            entries.append(
                DirectoryEntry(
                    name=name,
                    object_type=chunk[66],
                    start_sector=struct.unpack_from("<I", chunk, 116)[0],
                    size=struct.unpack_from("<Q", chunk, 120)[0],
                )
            )
        return entries

    def _read_minifat(self) -> list[int]:
        if self.first_minifat_sector in {END_OF_CHAIN, FREE_SECTOR, NO_STREAM}:
            return []
        raw = b"".join(self._sector(sector) for sector in self._chain(self.first_minifat_sector))
        return list(struct.unpack("<" + "I" * (len(raw) // 4), raw))

    def _read_regular_stream(self, entry: DirectoryEntry | None) -> bytes:
        if entry is None or entry.start_sector in {END_OF_CHAIN, FREE_SECTOR, NO_STREAM}:
            return b""
        raw = b"".join(self._sector(sector) for sector in self._chain(entry.start_sector))
        return raw[: entry.size]

    def read_stream(self, name: str) -> bytes:
        """Read a named stream from regular or mini-stream storage."""
        entry = next((item for item in self.entries if item.name == name), None)
        if entry is None:
            raise KeyError(name)
        if entry.size >= self.mini_cutoff or not self.mini_fat:
            return self._read_regular_stream(entry)

        parts = []
        sector = entry.start_sector
        while sector not in {END_OF_CHAIN, FREE_SECTOR, NO_STREAM}:
            offset = sector * self.mini_sector_size
            parts.append(self.root_mini_stream[offset : offset + self.mini_sector_size])
            if sector >= len(self.mini_fat):
                raise ValueError(f"mini sector chain points outside MiniFAT: {sector}")
            sector = self.mini_fat[sector]
        return b"".join(parts)[: entry.size]


def run(command: list[str], echo_stdout: bool = True) -> subprocess.CompletedProcess:
    """Run a command and echo its useful output for probe logs."""
    result = subprocess.run(
        command,
        check=False,
        text=True,
        capture_output=True,
        encoding="utf-8",
        errors="replace",
    )
    if echo_stdout and result.stdout.strip():
        print(result.stdout.strip())
    if result.stderr.strip():
        print(result.stderr.strip())
    if result.returncode != 0:
        raise RuntimeError(f"command failed: {' '.join(command)}")
    return result


def build_helper() -> None:
    """Build the small .NET OLE helper used by this probe."""
    run(["dotnet", "build", str(HELPER_PROJECT), "-v:quiet"])


def mathtype_tex_payload(latex: str) -> str:
    """Return TeX text in the math-delimited form accepted by MathType OLE.

    Pandoc's JSON AST gives math content without the surrounding delimiters.
    MathType's OLE SetData path rejects bare fragments such as ``f(x)`` or
    ``w_i`` with DV_E_FORMATETC, but accepts the same TeX when wrapped as
    ``$...$``.
    """
    text = latex.strip()
    if text.startswith("$$") and text.endswith("$$"):
        return text
    if text.startswith("$") and text.endswith("$"):
        return text
    if text.startswith(r"\(") and text.endswith(r"\)"):
        return "$" + text[2:-2].strip() + "$"
    if text.startswith(r"\[") and text.endswith(r"\]"):
        return "$$" + text[2:-2].strip() + "$$"
    return "$" + text + "$"


def write_latex_input(path: Path, latex: str) -> None:
    """Write MathType-ready TeX as UTF-8; the helper converts it to UTF-16LE."""
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(mathtype_tex_payload(latex), encoding="utf-8")


def make_ole_from_format(
    format_name: str,
    input_path: Path,
    output_path: Path,
    binary: bool = False,
    preview_output: Path | None = None,
    metadata_output: Path | None = None,
) -> None:
    """Ask MathType OLE to create an Equation.DSMT4 compound file without Word."""
    command = [
        str(HELPER_EXE),
        "--method",
        "set-data",
        "--pre-verb",
        "2",
        "--format",
        format_name,
        "--input",
        str(input_path),
        "--output",
        str(output_path),
        "--encoding",
        "utf16le",
        "--no-verb",
    ]
    if binary:
        command.append("--binary")
    if preview_output is not None:
        command.extend(["--preview-output", str(preview_output)])
    if metadata_output is not None:
        command.extend(["--metadata-output", str(metadata_output)])
    run(command)


def latex_to_mathml(latex: str, output_path: Path) -> Path:
    """Convert one LaTeX math fragment to MathML using Pandoc."""
    output_path.parent.mkdir(parents=True, exist_ok=True)
    markdown_path = output_path.with_suffix(".md")
    markdown_path.write_text("$$\n" + latex.strip() + "\n$$\n", encoding="utf-8")
    result = run(
        ["pandoc", str(markdown_path), "-f", "markdown", "-t", "html", "--mathml", "--wrap=none"],
        echo_stdout=False,
    )
    html = result.stdout
    start = html.find("<math")
    end = html.find("</math>", start)
    if start < 0 or end < 0:
        raise ValueError(f"Pandoc did not produce MathML for {markdown_path}")
    mathml = html[start : end + len("</math>")]
    output_path.write_text(mathml, encoding="utf-8")
    return output_path


def inspect_ole(path: Path) -> CompoundFile:
    """Print the key MathType OLE stream evidence."""
    compound = CompoundFile(path.read_bytes())
    names = ", ".join(entry.name for entry in compound.entries if entry.name)
    native = compound.read_stream("Equation Native")
    print(f"[probe] {path}: streams={names}")
    print(f"[probe] Equation Native bytes={len(native)}, DSMT offset={native.find(b'DSMT')}")
    return compound


def inject_generated_ole(source: Path, sample: Path, generated_ole: Path, generated_wmf: Path, target: Path) -> int:
    """Inject one generated OLE object and WMF preview."""
    template = extract_mathtype_template(sample)
    replacement_template = MathTypeTemplate(
        object_element=template.object_element,
        ole_bytes=generated_ole.read_bytes(),
        image_bytes=generated_wmf.read_bytes(),
    )

    # Monkey-patching the extracted template keeps this probe isolated from the
    # existing structural replacement implementation.
    original_extract = extract_mathtype_template
    try:
        import probe_mathtype_docx_ole_replace as replace_module

        replace_module.extract_mathtype_template = lambda _: replacement_template
        return replace_omml_with_template(source, sample, target, 1)
    finally:
        import probe_mathtype_docx_ole_replace as replace_module

        replace_module.extract_mathtype_template = original_extract


def extract_markdown_math(markdown: Path, metadata_file: Path | None = None) -> list[str]:
    """Extract math strings from Markdown using Pandoc's parser order."""
    command = ["pandoc", str(markdown), "-t", "json"]
    if metadata_file is not None and metadata_file.exists():
        command.extend(["--metadata-file", str(metadata_file)])
    result = run(command, echo_stdout=False)
    ast = json.loads(result.stdout)
    formulas: list[str] = []

    def walk(value: object) -> None:
        if isinstance(value, dict):
            if value.get("t") == "Math":
                content = value.get("c", [])
                if isinstance(content, list) and len(content) == 2:
                    formulas.append(str(content[1]))
                return
            for child in value.values():
                walk(child)
        elif isinstance(value, list):
            for child in value:
                walk(child)

    # Pandoc includes metadata in the JSON AST. Restrict extraction to document
    # blocks so crossref/style metadata math snippets are not paired with DOCX
    # OMML nodes that only exist in the manuscript body.
    walk(ast.get("blocks", ast))
    return formulas


def generate_equation_parts(latex_values: list[str], output_dir: Path) -> list[GeneratedEquation]:
    """Generate OLE bins and WMF previews for all extracted LaTeX formulas."""
    output_dir.mkdir(parents=True, exist_ok=True)
    equations: list[GeneratedEquation] = []
    for index, latex in enumerate(latex_values, start=1):
        input_path = output_dir / f"eq_{index:03d}.tex"
        mathml_path = output_dir / f"eq_{index:03d}.mathml"
        ole_path = output_dir / f"eq_{index:03d}.ole.bin"
        wmf_path = output_dir / f"eq_{index:03d}.wmf"
        metadata_path = output_dir / f"eq_{index:03d}.json"
        write_latex_input(input_path, latex)
        try:
            make_ole_from_format(
                "TeX Input Language",
                input_path,
                ole_path,
                preview_output=wmf_path,
                metadata_output=metadata_path,
            )
        except RuntimeError as exc:
            print(f"[probe] TeX input failed for equation {index}; falling back to MathML: {exc}")
            latex_to_mathml(latex, mathml_path)
            make_ole_from_format(
                "application/mathml+xml",
                mathml_path,
                ole_path,
                preview_output=wmf_path,
                metadata_output=metadata_path,
            )
        compound = inspect_ole(ole_path)
        if compound.read_stream("Equation Native").find(b"DSMT") < 0:
            raise ValueError(f"generated OLE lacks DSMT marker: {ole_path}")
        if not wmf_path.exists() or wmf_path.read_bytes()[:4] != bytes.fromhex("d7cdc69a"):
            raise ValueError(f"generated WMF preview is missing placeable header: {wmf_path}")
        equations.append(GeneratedEquation(latex=latex, ole_path=ole_path, wmf_path=wmf_path, metadata_path=metadata_path))
    return equations


def replace_all_omml_with_generated(source: Path, sample: Path, target: Path, equations: list[GeneratedEquation]) -> int:
    """Replace top-level OMML nodes with generated MathType OLE objects."""
    template = extract_mathtype_template(sample)
    target.parent.mkdir(parents=True, exist_ok=True)

    with zipfile.ZipFile(source) as in_zip:
        document = read_xml(in_zip, "word/document.xml")
        rels = read_xml(in_zip, "word/_rels/document.xml.rels")
        content_types = read_xml(in_zip, "[Content_Types].xml")
        existing_names = set(in_zip.namelist())

        nodes = top_level_omml_nodes(document)
        if len(nodes) != len(equations):
            raise ValueError(f"math count mismatch: markdown={len(equations)}, docx_omml={len(nodes)}")

        existing_rids = [rel.get("Id", "") for rel in rels.findall("rel:Relationship", NS)]
        rid_counter = find_next_numeric_id(existing_rids, "rId")
        parent_map = collect_parent_map(document)
        added_parts: dict[str, bytes] = {}

        for index, (node, equation) in enumerate(zip(nodes, equations), start=1):
            parent = parent_map[node]
            child_index = list(parent).index(node)
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
            parent.remove(node)
            parent.insert(child_index, make_object_run(item_template, image_rid, ole_rid, index, node.tag == qn("m", "oMath")))
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
        print(f"[probe] docx={path}")
        print(f"[probe] embeddings={len(embeddings)}, Equation.DSMT4={document_xml.count(b'Equation.DSMT4')}")
        print(f"[probe] generated WMF previews={len(previews)}")
        print(f"[probe] oMath tokens={document_xml.count(b'<m:oMath') + document_xml.count(b'<m:oMathPara')}")
        print(f"[probe] ole relationships={rels_xml.count(b'/oleObject')}")


def main() -> int:
    """Run the no-Word LaTeX-to-OLE feasibility probe."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--latex", default=r"x^2 + y^2 = z^2", help="LaTeX equation text")
    parser.add_argument("--source", default="output/docx/manuscript.docx", help="DOCX containing OMML")
    parser.add_argument("--markdown", default="manuscript.md", help="Markdown source for full replacement")
    parser.add_argument("--metadata-file", default=None, help="Optional Pandoc metadata file for parsing")
    parser.add_argument("--sample", default="mathtype.docx", help="DOCX containing one MathType OLE object")
    parser.add_argument("--target", default="tmp/mathtype-latex-native-ole-probe.docx", help="Output DOCX")
    parser.add_argument("--mode", choices=["single", "all"], default="single", help="Probe one formula or all Markdown math")
    args = parser.parse_args()

    build_helper()

    tmp = Path("tmp")
    latex_path = tmp / "mathtype-latex-input.txt"
    tex_ole = tmp / "mathtype-latex-texinput.ole.bin"
    native_path = tmp / "mathtype-latex-equation-native.mtef"
    native_ole = tmp / "mathtype-latex-native.ole.bin"
    target = Path(args.target)

    if args.mode == "all":
        metadata_file = Path(args.metadata_file) if args.metadata_file else None
        formulas = extract_markdown_math(Path(args.markdown), metadata_file)
        print(f"[probe] Markdown math nodes: {len(formulas)}")
        equations = generate_equation_parts(formulas, tmp / "mathtype-all")
        replaced = replace_all_omml_with_generated(Path(args.source), Path(args.sample), target, equations)
        print(f"[probe] replaced top-level OMML nodes: {replaced}")
        inspect_docx(target)
        return 0

    write_latex_input(latex_path, args.latex)
    make_ole_from_format(
        "TeX Input Language",
        latex_path,
        tex_ole,
        preview_output=tmp / "mathtype-latex-preview.wmf",
    )
    tex_compound = inspect_ole(tex_ole)

    native = tex_compound.read_stream("Equation Native")
    native_path.write_bytes(native)
    print(f"[probe] wrote native MTEF stream: {native_path}")

    final_ole = tex_ole
    try:
        make_ole_from_format("Native", native_path, native_ole, binary=True)
        inspect_ole(native_ole)
        final_ole = native_ole
    except RuntimeError as exc:
        # MathType's OLE server accepts TeX/MathML as custom input formats, but
        # the registry's "Native" label does not map to a simple SetData name.
        print(f"[probe] Native SetData round-trip failed; using TeX-generated OLE bin: {exc}")

    replaced = inject_generated_ole(Path(args.source), Path(args.sample), final_ole, tmp / "mathtype-latex-preview.wmf", target)
    print(f"[probe] replaced top-level OMML nodes: {replaced}")
    inspect_docx(target)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
