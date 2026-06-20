"""Build reviewer-reply DOCX files using manuscript numbering and citations."""

from __future__ import annotations

import errno
import json
import os
import re
import shutil
import subprocess
import sys
import time
import uuid
from contextlib import contextmanager
from pathlib import Path
from typing import Any, Iterator

import yaml
from pydantic import AliasChoices, Field
from pydantic_settings import BaseSettings, CliPositionalArg, CliSuppress, SettingsConfigDict

from .logging_utils import log_debug, log_error, log_info, log_success, log_warning
from .metadata import load_merged_metadata_with_status, merge_metadata, parse_yaml_file
from .mathtype.convert_marked_docx import convert_marked_docx
from .mathtype.marked_docx import extract_marked_equation_requests
from .mathtype.ole_parts import build_helper, check_mathtype_availability
from .postprocess.final_docx_syntax_check import validate_final_docx_syntax
from .postprocess_docx import postprocess_docx
from .resources import package_resource_path, template_root


DEFAULT_OUTPUT_DIR = "output"
DEFAULT_STYLE_FILE = "style.yml"
DEFAULT_REPLY_MANUSCRIPT_FILE = "manuscript.md"
DEFAULT_REPLY_LINE_SOURCE = "output/docx/manuscript.docx"
DEFAULT_REPLY_FROM_FORMAT = "markdown"
DEFAULT_REPLY_OUTPUT_FILE = "output/docx/<reply-name>.docx"
LINE_SOURCE_PDF_DIR = Path("tmp/reply-line-source-pdf")
REPLY_PROBE_DIR = Path("tmp/reply-probes")
LABEL_CHARS_NO_DOT = r"A-Za-z0-9_:\-"
LABEL_CONTINUATION = rf"(?:[{LABEL_CHARS_NO_DOT}]|\.(?=[{LABEL_CHARS_NO_DOT}]))"
REF_PATTERN = re.compile(rf"@((?:sec|fig|tbl|eq):[A-Za-z0-9]{LABEL_CONTINUATION}*)")
REF_BOUNDARY = rf"(?![{LABEL_CHARS_NO_DOT}]|\.(?=[{LABEL_CHARS_NO_DOT}]))"
CITATION_PATTERN = re.compile(r"(?<![\w:])@([A-Za-z0-9_][A-Za-z0-9_:.#/$%&+?<>~/-]*)")
CITATION_CLUSTER_PATTERN = re.compile(r"\[([^\]\n]*@[^\]\n]*)\]")
PROBE_SENTINEL = "PANDOC_REPLY_REF_PROBE"
CITATION_PROBE_SENTINEL = "PANDOC_REPLY_CITE_PROBE"
CITATION_CLUSTER_PROBE_SENTINEL = "PANDOC_REPLY_CITE_CLUSTER_PROBE"
CROSSREF_PREFIXES = ("sec:", "fig:", "tbl:", "eq:")
LINE_REGEX_PATTERN = re.compile(r"\(Line `([^`]+)`\)")
ANSI_RED = "\033[31m"
ANSI_RESET = "\033[0m"
PREFIX_WORDS = {
    "sec": "Section",
    "fig": "Figure",
    "tbl": "Table",
    "eq": "Equation",
}
BUILD_REPLY_CLI_CONFIG = SettingsConfigDict(
    cli_kebab_case=True,
    cli_implicit_flags=True,
    cli_hide_none_type=True,
    cli_parse_none_str="auto",
    cli_shortcuts={
        "output_file": ["-o", "--output-file"],
    },
)


@contextmanager
def project_directory(project_dir: Path) -> Iterator[None]:
    """Temporarily run reply builds from the selected manuscript project."""
    previous_cwd = Path.cwd()
    os.chdir(project_dir)
    try:
        yield
    finally:
        os.chdir(previous_cwd)


class BuildReplySettings(BaseSettings):
    """Settings for `pmt build-reply`."""

    model_config = BUILD_REPLY_CLI_CONFIG

    markdown: CliPositionalArg[str] = Field(description="Reply markdown file.")

    project_dir: Path = Field(default=Path("."), description="Manuscript project directory.")
    reply_manuscript: str | None = Field(
        default=DEFAULT_REPLY_MANUSCRIPT_FILE,
        description="Manuscript source used to resolve reply references.",
    )
    manuscript_line_source: str | None = Field(
        default=DEFAULT_REPLY_LINE_SOURCE,
        description="DOCX source used for reply line placeholders.",
    )
    from_format: str | None = Field(
        default=DEFAULT_REPLY_FROM_FORMAT,
        description="Pandoc input format for reply reference probes.",
    )
    reference_doc: CliSuppress[str | None] = Field(
        default=None,
        description="Override the bundled DOCX reference document.",
    )
    output_file: str = Field(
        default=DEFAULT_REPLY_OUTPUT_FILE,
        validation_alias=AliasChoices("o", "output-file"),
        description="Explicit reply DOCX output path.",
    )

    def run(self) -> int:
        """Run the standalone reply build target."""
        project_dir = self.project_dir.resolve()
        if not project_dir.exists():
            raise FileNotFoundError(f"Project directory not found: {project_dir}")

        with project_directory(project_dir):
            return run_build_reply_command(
                markdown=self.markdown,
                reply_manuscript=self.reply_manuscript,
                manuscript_line_source=self.manuscript_line_source,
                from_format=self.from_format,
                reference_doc=self.reference_doc,
                output_file=self.output_file,
            )


def to_pandoc_path(path: Path) -> str:
    """Return a Pandoc-friendly path string."""
    return path.as_posix()


def load_reply_style_metadata(style: Path) -> dict[str, Any]:
    """Load style.yml and apply its optional reply-specific metadata section."""
    metadata = parse_yaml_file(style)
    reply_metadata = metadata.pop("reply", None)
    if reply_metadata is None:
        return metadata
    if not isinstance(reply_metadata, dict):
        raise ValueError(f"The `reply` section in {style} must be a YAML mapping")
    return merge_metadata(metadata, reply_metadata)


def write_reply_style_metadata_file(style: Path) -> Path:
    """Write flattened reply metadata for Pandoc filters that require top-level keys."""
    REPLY_PROBE_DIR.mkdir(parents=True, exist_ok=True)
    metadata = load_reply_style_metadata(style)
    flattened_style = REPLY_PROBE_DIR / "style.reply.flat.yml"
    flattened_style.write_text(
        yaml.safe_dump(metadata, allow_unicode=True, sort_keys=False),
        encoding="utf-8",
    )
    return flattened_style


def load_reply_metadata(reply: Path, flattened_style: Path) -> dict[str, Any]:
    """Load flattened reply style metadata, allowing reply markdown without YAML."""
    metadata, _ = load_merged_metadata_with_status(
        reply,
        [flattened_style],
        allow_missing_header=True,
    )
    return metadata


def log_red(message: str) -> None:
    """Print a red warning for line-regex matches that are not uniquely resolved."""
    log_warning(f"{ANSI_RED}{message}{ANSI_RESET}")


def run_command(cmd: list[str]) -> subprocess.CompletedProcess[str]:
    """Run a command, echo it, and raise with captured output on failure."""
    log_info(f"[Run] {' '.join(cmd)}")
    result = subprocess.run(
        cmd,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if result.returncode != 0:
        if result.stdout.strip():
            log_error(result.stdout.strip())
        if result.stderr.strip():
            log_error(result.stderr.strip())
        result.check_returncode()
    return result


def extract_reference_labels(markdown: str) -> list[str]:
    """Return unique manuscript-style cross-reference labels used in reply text."""
    labels = sorted(set(REF_PATTERN.findall(markdown)))
    log_info(f"[INFO] Found {len(labels)} manuscript-style references in reply.")
    return labels


def extract_citation_keys(markdown: str) -> list[str]:
    """Return unique bibliography citation keys used in reply text."""
    keys = set(citation_keys_in_text(markdown))
    citations = sorted(keys)
    log_info(f"[INFO] Found {len(citations)} bibliography citations in reply.")
    return citations


def citation_keys_in_text(text: str) -> list[str]:
    """Return bibliography citation keys from a text fragment in encounter order."""
    keys: list[str] = []
    seen: set[str] = set()
    for key in CITATION_PATTERN.findall(text):
        if key.startswith(CROSSREF_PREFIXES) or key in seen:
            continue
        keys.append(key)
        seen.add(key)
    return keys


def extract_citation_clusters(markdown: str) -> list[str]:
    """Return bracketed citation clusters such as `[@a; @b]` from reply text."""
    clusters: list[str] = []
    seen: set[str] = set()
    for match in CITATION_CLUSTER_PATTERN.finditer(markdown):
        cluster = match.group(0)
        if cluster in seen or not citation_keys_in_text(match.group(1)):
            continue
        clusters.append(cluster)
        seen.add(cluster)
    log_info(f"[INFO] Found {len(clusters)} bracketed citation clusters in reply.")
    return clusters


def metadata_bool(value: Any) -> bool:
    """Normalize YAML feature flags such as mathtype: true or mathtype: yes."""
    if isinstance(value, bool):
        return value
    if isinstance(value, str):
        return value.strip().lower() in {"1", "true", "yes", "on"}
    if isinstance(value, (int, float)):
        return bool(value)
    return False


def resolve_mathtype_enabled(requested: bool) -> bool:
    """Return whether MathType conversion should run for this reply build."""
    if not requested:
        return False

    log_info("[INFO] MathType DOCX equations enabled by reply metadata: mathtype: true")
    availability = check_mathtype_availability()
    if availability.usable:
        return True

    log_warning(
        availability.format_failure(
            "[WARN] MathType was requested by reply metadata, but MathType conversion will be skipped."
        )
    )
    log_warning("[WARN] Building reply DOCX with Pandoc/Word equations instead.\n")
    return False


def mathtype_marked_docx_path(output: Path) -> Path:
    """Return the intermediate reply DOCX path carrying hidden LaTeX markers."""
    work_dir = Path("tmp/mathtype-build/reply")
    work_dir.mkdir(parents=True, exist_ok=True)
    return work_dir / f"{output.stem}.marked.docx"


def mathtype_filter_args() -> list[str]:
    """Return Pandoc args that insert hidden LaTeX markers before DOCX writing."""
    marker_filter = package_resource_path("mathtype/mathtype_markers.lua")
    if not marker_filter.exists():
        raise FileNotFoundError(f"MathType marker filter not found: {marker_filter}")
    return ["--lua-filter", to_pandoc_path(marker_filter)]


def table_metadata_filter_args() -> list[str]:
    """Return Pandoc args for embedding hidden reply table-attribute markers."""
    filter_path = template_root() / "pandoc" / "filters" / "table_metadata.lua"
    if not filter_path.exists():
        raise FileNotFoundError(f"Table metadata Pandoc filter not found: {filter_path}")
    return ["--lua-filter", to_pandoc_path(filter_path)]


def run_mathtype_conversion(marked_docx: Path, target_docx: Path) -> None:
    """Convert a marked reply DOCX's OMML equations into MathType OLE equations."""
    if not extract_marked_equation_requests(marked_docx):
        # Replies often contain no formulas even when the shared style enables MathType.
        # In that case the marked DOCX is already the final DOCX.
        log_info("[INFO] No MathType equation markers found; keeping Pandoc DOCX equations unchanged.")
        target_docx.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(marked_docx, target_docx)
        return

    log_info("\n[DOCX] Converting reply equations to MathType OLE objects...\n")
    build_helper()
    convert_marked_docx(
        source=marked_docx,
        target=target_docx,
        work_dir=Path("tmp/mathtype-build/reply") / target_docx.stem,
    )


def inline_to_text(inline: dict[str, Any]) -> str:
    """Convert a Pandoc JSON inline node to plain text for probe parsing."""
    tag = inline.get("t")
    content = inline.get("c")

    if tag == "Str":
        return str(content)
    if tag == "Space":
        return " "
    if tag in {"SoftBreak", "LineBreak"}:
        return " "
    if tag in {"Code", "Math"} and isinstance(content, list):
        return str(content[-1])
    if tag in {"Emph", "Strong", "Span", "SmallCaps", "Strikeout", "Superscript", "Subscript"}:
        if tag == "Span" and isinstance(content, list) and len(content) >= 2:
            return inlines_to_text(content[1])
        return inlines_to_text(content or [])
    if tag == "Link" and isinstance(content, list) and len(content) >= 2:
        return inlines_to_text(content[1])
    if tag == "Cite" and isinstance(content, list) and len(content) >= 2:
        return inlines_to_text(content[1])
    if tag == "Quoted" and isinstance(content, list) and len(content) >= 2:
        return inlines_to_text(content[1])
    if tag == "RawInline" and isinstance(content, list) and len(content) >= 2:
        return str(content[1])
    return ""


def inlines_to_text(inlines: list[dict[str, Any]]) -> str:
    """Flatten Pandoc JSON inlines into normalized display text."""
    text = "".join(inline_to_text(inline) for inline in inlines)
    return re.sub(r"\s+", " ", text.replace("\u00a0", " ")).strip()


def extract_probe_map(
    document: dict[str, Any],
    sentinel: str,
    labels: list[str],
    unresolved_markers: tuple[str, ...],
) -> dict[str, str]:
    """Extract label-to-display text mappings from Pandoc JSON probe blocks."""
    requested = set(labels)
    resolved: dict[str, str] = {}

    for block in document.get("blocks", []):
        if block.get("t") != "Para":
            continue
        inlines = block.get("c", [])
        if len(inlines) < 4:
            continue
        if inlines[0].get("t") != "Str" or inlines[0].get("c") != sentinel:
            continue
        if inlines[2].get("t") != "Str":
            continue
        label = inlines[2].get("c")
        display = inlines_to_text(inlines[4:])
        if display and label in requested and not any(marker in display for marker in unresolved_markers):
            resolved[label] = display

    return resolved


def write_probe_file(name: str, lines: list[str]) -> Path:
    """Write a stable probe file under tmp without relying on tempfile ACLs."""
    REPLY_PROBE_DIR.mkdir(parents=True, exist_ok=True)
    probe_path = REPLY_PROBE_DIR / name
    probe_path.write_text("\n\n".join(lines) + "\n", encoding="utf-8")
    return probe_path


def resolve_reference_map(
    manuscript: Path,
    style: Path,
    labels: list[str],
    from_format: str,
) -> dict[str, str]:
    """Resolve reply labels using the manuscript's pandoc-crossref numbering."""
    if not labels:
        return {}

    probe_path = write_probe_file(
        "reference-probe.md",
        [f"{PROBE_SENTINEL} {label} @{label}" for label in labels],
    )
    cmd = [
        "pandoc",
        "--metadata-file",
        str(style),
        "-f",
        from_format,
        "-t",
        "json",
        "--filter",
        "pandoc-crossref",
        str(manuscript),
        str(probe_path),
    ]
    result = run_command(cmd)
    if result.stderr.strip():
        log_warning(result.stderr.strip())

    document = json.loads(result.stdout)
    resolved = extract_probe_map(document, PROBE_SENTINEL, labels, ("¿",))
    missing = [label for label in labels if label not in resolved]
    if missing:
        log_warning("[WARN] These labels were not resolved from manuscript.md:")
        for label in missing:
            log_warning(f"  - {label}")
    log_info(f"[INFO] Resolved {len(resolved)} references from manuscript numbering.")
    return resolved


def resolve_citation_map(
    manuscript: Path,
    style: Path,
    citations: list[str],
    from_format: str,
) -> dict[str, str]:
    """Resolve bibliography citations using the manuscript's citeproc numbering."""
    if not citations:
        return {}

    probe_path = write_probe_file(
        "citation-probe.md",
        [f"{CITATION_PROBE_SENTINEL} {key} [@{key}]" for key in citations],
    )
    cmd = [
        "pandoc",
        "--metadata-file",
        str(style),
        "-f",
        from_format,
        "-t",
        "json",
        "--filter",
        "pandoc-crossref",
        "--citeproc",
        str(manuscript),
        str(probe_path),
    ]
    result = run_command(cmd)
    if result.stderr.strip():
        log_warning(result.stderr.strip())

    document = json.loads(result.stdout)
    resolved = extract_probe_map(document, CITATION_PROBE_SENTINEL, citations, ("???",))
    missing = [key for key in citations if key not in resolved]
    if missing:
        log_warning("[WARN] These citation keys were not resolved from manuscript bibliography:")
        for key in missing:
            log_warning(f"  - {key}")
    log_info(f"[INFO] Resolved {len(resolved)} citations from manuscript citeproc output.")
    return resolved


def resolve_citation_cluster_map(
    manuscript: Path,
    style: Path,
    citation_clusters: list[str],
    from_format: str,
) -> dict[str, str]:
    """Resolve bracketed citation clusters so citeproc keeps sorting and delimiters."""
    if not citation_clusters:
        return {}

    cluster_ids = [str(index) for index, _ in enumerate(citation_clusters)]
    probe_path = write_probe_file(
        "citation-cluster-probe.md",
        [
            f"{CITATION_CLUSTER_PROBE_SENTINEL} {cluster_id} {cluster}"
            for cluster_id, cluster in zip(cluster_ids, citation_clusters)
        ],
    )
    cmd = [
        "pandoc",
        "--metadata-file",
        str(style),
        "-f",
        from_format,
        "-t",
        "json",
        "--filter",
        "pandoc-crossref",
        "--citeproc",
        str(manuscript),
        str(probe_path),
    ]
    result = run_command(cmd)
    if result.stderr.strip():
        log_warning(result.stderr.strip())

    document = json.loads(result.stdout)
    resolved_by_id = extract_probe_map(document, CITATION_CLUSTER_PROBE_SENTINEL, cluster_ids, ("???",))
    resolved = {
        cluster: resolved_by_id[cluster_id]
        for cluster_id, cluster in zip(cluster_ids, citation_clusters)
        if cluster_id in resolved_by_id
    }
    missing = [cluster for cluster_id, cluster in zip(cluster_ids, citation_clusters) if cluster_id not in resolved_by_id]
    if missing:
        log_warning("[WARN] These citation clusters were not resolved from manuscript bibliography:")
        for cluster in missing:
            log_warning(f"  - {cluster}")
    log_info(f"[INFO] Resolved {len(resolved)} citation clusters from manuscript citeproc output.")
    return resolved


def number_only(label: str, display: str) -> str:
    """Strip a cross-reference prefix when reply prose already supplies it."""
    prefix = PREFIX_WORDS.get(label.split(":", 1)[0])
    if not prefix:
        return display
    pattern = re.compile(rf"^{re.escape(prefix)}\s+", re.IGNORECASE)
    return pattern.sub("", display).strip()


def replace_references(markdown: str, reference_map: dict[str, str]) -> str:
    """Replace reply reference tokens with manuscript-derived display numbers."""
    resolved = markdown
    for label in sorted(reference_map, key=len, reverse=True):
        display = reference_map[label]
        short = number_only(label, display)
        escaped = re.escape(label)
        prefix = PREFIX_WORDS.get(label.split(":", 1)[0])

        if prefix:
            # Avoid "Section Section 5" when reply prose already names the reference type.
            resolved = re.sub(
                rf"\b{prefix}\s+\[@{escaped}\]",
                f"{prefix} {short}",
                resolved,
                flags=re.IGNORECASE,
            )
            resolved = re.sub(
                rf"\b{prefix}\s+@{escaped}{REF_BOUNDARY}",
                f"{prefix} {short}",
                resolved,
                flags=re.IGNORECASE,
            )

        resolved = re.sub(rf"\[@{escaped}\]", display, resolved)
        resolved = re.sub(rf"@{escaped}{REF_BOUNDARY}", display, resolved)

    return resolved



def replace_citations(
    markdown: str,
    citation_map: dict[str, str],
    citation_cluster_map: dict[str, str] | None = None,
) -> str:
    """Replace bibliography citation tokens with manuscript-derived numbers."""
    cluster_map = citation_cluster_map or {}
    protected_clusters: list[str] = []

    def replace_cluster(match: re.Match[str]) -> str:
        """Replace a complete citation cluster before touching bare keys."""
        cluster = match.group(0)
        display = cluster_map.get(cluster)
        if display:
            return display

        placeholder = f"@@PMT_CITE_CLUSTER_{len(protected_clusters)}@@"
        protected_clusters.append(cluster)
        return placeholder

    resolved = CITATION_CLUSTER_PATTERN.sub(replace_cluster, markdown)
    for key in sorted(citation_map, key=len, reverse=True):
        display = citation_map[key]
        escaped = re.escape(key)
        resolved = re.sub(rf"\[@{escaped}\]", display, resolved)
        resolved = re.sub(rf"(?<![\w:])@{escaped}\b", display, resolved)

    for index, cluster in enumerate(protected_clusters):
        resolved = resolved.replace(f"@@PMT_CITE_CLUSTER_{index}@@", cluster)
    return resolved


def normalized_pdf_line_text(text: str) -> str:
    """Normalize one PDF text line so regexes can survive extraction artifacts."""
    return re.sub(r"\s+", " ", text.replace("\u00a0", " ")).strip()


def export_docx_to_pdf_with_word(source_docx: Path, target_pdf: Path) -> None:
    """Export a DOCX line source to PDF through Microsoft Word COM automation."""
    if sys.platform != "win32":
        raise RuntimeError("DOCX line-source conversion requires Microsoft Word COM on Windows.")

    source_docx = source_docx.resolve()
    target_pdf = target_pdf.resolve()
    target_pdf.parent.mkdir(parents=True, exist_ok=True)
    target_pdf.unlink(missing_ok=True)

    log_info(f"[LINE] Converting DOCX line source to PDF with Word COM: {source_docx}")
    powershell = r"""
param([string]$source, [string]$target)
$word = $null
$document = $null
try {
    $word = New-Object -ComObject Word.Application
    $word.Visible = $false
    $word.DisplayAlerts = 0
    $document = $word.Documents.Open($source, $false, $true, $false)
    $document.SaveAs2($target, 17)
} finally {
    if ($null -ne $document) {
        $document.Close($false) | Out-Null
        [System.Runtime.InteropServices.Marshal]::ReleaseComObject($document) | Out-Null
    }
    if ($null -ne $word) {
        $word.Quit() | Out-Null
        [System.Runtime.InteropServices.Marshal]::ReleaseComObject($word) | Out-Null
    }
    [GC]::Collect()
    [GC]::WaitForPendingFinalizers()
}
"""
    script_path = LINE_SOURCE_PDF_DIR / "word_docx_to_pdf.ps1"
    script_path.parent.mkdir(parents=True, exist_ok=True)
    script_path.write_text(powershell, encoding="utf-8")
    result = subprocess.run(
        [
            "powershell",
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-File",
            str(script_path),
            str(source_docx),
            str(target_pdf),
        ],
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if result.returncode != 0:
        if result.stdout.strip():
            log_error(result.stdout.strip())
        if result.stderr.strip():
            log_error(result.stderr.strip())
        raise RuntimeError(f"Word COM DOCX-to-PDF conversion failed: {source_docx}")
    if not target_pdf.exists():
        raise RuntimeError(f"Word COM conversion did not create PDF: {target_pdf}")
    log_info(f"[LINE] Word COM PDF created: {target_pdf}")


def prepare_line_source_pdf(line_source: Path) -> Path:
    """Return a PDF path for line-regex matching, converting DOCX sources if needed."""
    if not line_source.exists():
        raise FileNotFoundError(f"Line source file not found: {line_source}")

    suffix = line_source.suffix.lower()
    if suffix == ".pdf":
        return line_source
    if suffix in {".docx", ".docm"}:
        target_pdf = LINE_SOURCE_PDF_DIR / f"{line_source.stem}.pdf"
        export_docx_to_pdf_with_word(line_source, target_pdf)
        return target_pdf
    raise RuntimeError(f"Line source must be a PDF or Word document: {line_source}")


def extract_pdf_numbered_lines(pdf: Path) -> list[tuple[int, int, str]]:
    """Extract manuscript line numbers and their corresponding text from a PDF."""
    try:
        import fitz
    except ImportError as exc:
        raise RuntimeError("PyMuPDF is required to resolve reply line regexes.") from exc

    if not pdf.exists():
        raise FileNotFoundError(f"Manuscript PDF not found for line resolution: {pdf}")

    numbered_lines: list[tuple[int, int, str]] = []
    with fitz.open(pdf) as document:
        for page_index, page in enumerate(document, start=1):
            lines = page.get_text("text").splitlines()
            i = 0
            while i < len(lines) - 1:
                text = normalized_pdf_line_text(lines[i])
                maybe_number = lines[i + 1].strip()
                if text and re.fullmatch(r"\d+", maybe_number):
                    numbered_lines.append((int(maybe_number), page_index, text))
                    i += 2
                    continue
                i += 1
    log_info(f"[INFO] Extracted {len(numbered_lines)} numbered PDF text lines from {pdf}.")
    return numbered_lines


def build_pdf_search_text(
    numbered_lines: list[tuple[int, int, str]],
) -> tuple[str, list[tuple[int, int, int]]]:
    """Join numbered PDF lines and keep offsets for mapping regex hits to line numbers."""
    parts: list[str] = []
    offsets: list[tuple[int, int, int]] = []
    cursor = 0
    for line_number, page_number, text in numbered_lines:
        offsets.append((cursor, line_number, page_number))
        parts.append(text)
        cursor += len(text) + 1
    return " ".join(parts), offsets


def line_number_for_offset(offsets: list[tuple[int, int, int]], position: int) -> tuple[int, int]:
    """Return the PDF line number and page containing a joined-text character offset."""
    current_line = 0
    current_page = 0
    for offset, line_number, page_number in offsets:
        if offset > position:
            break
        current_line = line_number
        current_page = page_number
    return current_line, current_page


def resolve_line_regexes(markdown: str, line_source: Path) -> str:
    """Replace (Line `regex`) placeholders with unique line numbers from PDF/DOCX source."""
    patterns = sorted(set(LINE_REGEX_PATTERN.findall(markdown)))
    if not patterns:
        return markdown

    line_source_pdf = prepare_line_source_pdf(line_source)
    search_text, offsets = build_pdf_search_text(extract_pdf_numbered_lines(line_source_pdf))
    replacements: dict[str, str] = {}

    for pattern in patterns:
        try:
            regex = re.compile(pattern, flags=re.IGNORECASE)
        except re.error as exc:
            log_red(f"[LINE] Invalid regex in reply line placeholder: `{pattern}` ({exc})")
            continue

        matches = list(regex.finditer(search_text))
        if len(matches) != 1:
            log_red(f"[LINE] Regex `{pattern}` matched {len(matches)} PDF locations; leaving placeholder unchanged.")
            continue

        line_number, page_number = line_number_for_offset(offsets, matches[0].start())
        replacements[pattern] = f"(Line {line_number})"
        log_debug(f"[LINE] `{pattern}` -> Line {line_number} (PDF page {page_number})")

    def replace_match(match: re.Match[str]) -> str:
        """Return resolved line text, preserving unresolved regex placeholders."""
        return replacements.get(match.group(1), match.group(0))

    resolved = LINE_REGEX_PATTERN.sub(replace_match, markdown)
    log_info(f"[INFO] Resolved {len(replacements)} of {len(patterns)} unique line regexes.")
    return resolved


def ensure_output_writable(output: Path) -> None:
    """Fail early when an existing DOCX output is locked by Word or another app."""
    if not output.exists():
        return
    if output.is_dir():
        raise RuntimeError(f"Output path is a directory and cannot be overwritten: {output}")

    try:
        with output.open("r+b"):
            pass
    except OSError as exc:
        lock_like_errors = {errno.EACCES, errno.EPERM}
        lock_like_winerrors = {5, 32, 33}
        if exc.errno in lock_like_errors or getattr(exc, "winerror", None) in lock_like_winerrors:
            raise RuntimeError(
                "Output DOCX appears to be open or locked. Close it in Word and retry: "
                f"{output}"
            ) from exc
        raise


def resolved_reply_path(reply: Path) -> Path:
    """Return a temporary reply path outside the source tree's visible files."""
    REPLY_PROBE_DIR.mkdir(parents=True, exist_ok=True)
    return REPLY_PROBE_DIR / f"{reply.stem}.{uuid.uuid4().hex}.resolved.md"


def reply_resource_path(reply: Path) -> str:
    """Return Pandoc resource search paths that preserve reply-relative assets."""
    candidates = [Path("."), reply.parent]
    unique: list[Path] = []
    for candidate in candidates:
        if candidate not in unique:
            unique.append(candidate)
    return os.pathsep.join(to_pandoc_path(path) for path in unique)


def cleanup_resolved_reply_path(path: Path) -> None:
    """Remove the temporary reply file, tolerating short Windows file locks."""
    lock_like_winerrors = {5, 32, 33}
    for attempt in range(5):
        try:
            path.unlink(missing_ok=True)
            return
        except OSError as exc:
            is_locked = exc.errno in {errno.EACCES, errno.EPERM} or getattr(exc, "winerror", None) in lock_like_winerrors
            if not is_locked:
                raise
            if attempt < 4:
                # Pandoc can briefly keep the input file locked on Windows after it exits.
                time.sleep(0.1)
                continue
            # The resolved file is only a cache, so a warning is better than failing.
            log_warning(f"[WARN] Could not remove temporary reply file because it is still locked: {path}")


def build_reply_docx(
    reply: Path,
    manuscript: Path,
    manuscript_line_source: Path,
    output: Path,
    reference_doc: Path,
    style: Path,
    from_format: str,
) -> None:
    """Build a reviewer-reply DOCX with manuscript references resolved first."""
    ensure_output_writable(output)
    reply_text = reply.read_text(encoding="utf-8")
    flattened_style = write_reply_style_metadata_file(style)
    metadata = load_reply_metadata(reply, flattened_style)
    use_mathtype = resolve_mathtype_enabled(metadata_bool(metadata.get("mathtype")))
    pandoc_output = mathtype_marked_docx_path(output) if use_mathtype else output
    if use_mathtype:
        ensure_output_writable(pandoc_output)

    labels = extract_reference_labels(reply_text)
    citations = extract_citation_keys(reply_text)
    citation_clusters = extract_citation_clusters(reply_text)
    reference_map = resolve_reference_map(manuscript, flattened_style, labels, from_format)
    citation_map = resolve_citation_map(manuscript, flattened_style, citations, from_format)
    citation_cluster_map = resolve_citation_cluster_map(manuscript, flattened_style, citation_clusters, from_format)
    resolved_text = resolve_line_regexes(reply_text, manuscript_line_source)
    resolved_text = replace_references(resolved_text, reference_map)
    resolved_text = replace_citations(resolved_text, citation_map, citation_cluster_map)

    output.parent.mkdir(parents=True, exist_ok=True)
    temp_reply_path = resolved_reply_path(reply)
    temp_reply_path.write_text(resolved_text, encoding="utf-8")

    try:
        mathtype_args = mathtype_filter_args() if use_mathtype else []
        cmd = [
            "pandoc",
            str(temp_reply_path),
            "-f",
            from_format,
            "-o",
            str(pandoc_output),
            "--reference-doc",
            str(reference_doc),
            "--resource-path",
            reply_resource_path(reply),
            *table_metadata_filter_args(),
            *mathtype_args,
        ]
        run_command(cmd)

        log_info("[INFO] Running reply DOCX post-processing...")
        if not postprocess_docx(
            str(pandoc_output),
            metadata,
            skip_author_info=True,
            reply_style_formatting=True,
        ):
            raise RuntimeError(f"Reply DOCX post-processing failed: {pandoc_output}")

        if use_mathtype:
            # Post-process before conversion because equation layout fixes inspect OMML.
            run_mathtype_conversion(pandoc_output, output)

        syntax_findings = validate_final_docx_syntax(output)
        if syntax_findings:
            raise RuntimeError("Reply DOCX still contains unrendered Pandoc syntax")
    finally:
        cleanup_resolved_reply_path(temp_reply_path)

    log_success(f"[OK] Reply DOCX created: {output}")


def checked_markdown_path(markdown_path: str | Path) -> Path:
    """Return an existing reply markdown path, raising clear input errors."""
    path = Path(markdown_path)
    if not path.exists():
        raise FileNotFoundError(f"Reply markdown file not found: {path}")
    if not path.is_file():
        raise ValueError(f"Reply markdown path is not a file: {path}")
    return path


def reply_output_path(reply: Path, output_file: str | None) -> Path:
    """Return the exact DOCX output file for a reply build."""
    if output_file and output_file != DEFAULT_REPLY_OUTPUT_FILE:
        return Path(output_file)
    return Path(DEFAULT_OUTPUT_DIR) / "docx" / f"{reply.stem}.docx"


def reply_reference_doc_path(reference_doc: str | None) -> Path:
    """Return the active reference DOCX for reply builds."""
    if reference_doc:
        return Path(reference_doc)
    return template_root() / "pandoc" / "manuscript-template" / "reference-doc.docx"


def run_build_reply_command(
    *,
    markdown: str,
    reply_manuscript: str | None = None,
    manuscript_line_source: str | None = None,
    from_format: str | None = None,
    reference_doc: str | None = None,
    output_file: str | None = None,
) -> int:
    """Apply parsed `pmt build-reply` settings and run the reply build."""
    reply = checked_markdown_path(markdown)

    try:
        log_info("\n[DOCX] Building reviewer reply DOCX...\n")
        build_reply_docx(
            reply=reply,
            manuscript=Path(reply_manuscript or DEFAULT_REPLY_MANUSCRIPT_FILE),
            manuscript_line_source=Path(manuscript_line_source or DEFAULT_REPLY_LINE_SOURCE),
            output=reply_output_path(reply, output_file),
            reference_doc=reply_reference_doc_path(reference_doc),
            style=Path(DEFAULT_STYLE_FILE),
            from_format=from_format or DEFAULT_REPLY_FROM_FORMAT,
        )
        return 0
    except KeyboardInterrupt:
        log_warning("\n\n[WARN] Build interrupted by user.")
        return 1
    except Exception as exc:
        log_error(f"\n[ERROR] {exc}")
        return 1
