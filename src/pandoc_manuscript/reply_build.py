"""Build reviewer-reply DOCX/TXT files using manuscript numbering and citations."""

from __future__ import annotations

import errno
import hashlib
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

from . import runtime_cache_version
from .logging_utils import log_debug, log_error, log_info, log_success, log_warning
from .metadata import load_merged_metadata_with_status, merge_metadata, parse_yaml_file
from .mathtype.convert_marked_docx import convert_marked_docx
from .mathtype.marked_docx import extract_marked_equation_requests
from .mathtype.ole_parts import build_helper, check_mathtype_availability
from .paths import (
    PMT_MATHTYPE_WORK_DIR,
    PMT_REPLY_LINE_SOURCE_CACHE_DIR,
    PMT_REPLY_LINE_SOURCE_DOCX_DIR,
    PMT_REPLY_LINE_SOURCE_PDF_DIR,
    PMT_REPLY_PROBE_DIR,
)
from .postprocess.final_docx_syntax_check import validate_final_docx_syntax
from .postprocess_docx import postprocess_docx
from .resources import package_resource_path, template_root
from . import svg_filters as svg_filter_helpers
from .svg_filters import (
    should_convert_docx_svg_to_png,
    should_embed_docx_svg_images,
    svg_embed_images_filter_args,
    svg_to_png_filter_args,
)
from .tools import pandoc_command, pandoc_tools_env


DEFAULT_OUTPUT_DIR = "output"
DEFAULT_STYLE_FILE = "style.yml"
DEFAULT_REPLY_MANUSCRIPT_FILE = "manuscript.md"
DEFAULT_REPLY_LINE_SOURCE = DEFAULT_REPLY_MANUSCRIPT_FILE
DEFAULT_REPLY_FROM_FORMAT = "markdown"
DEFAULT_REPLY_OUTPUT_FILE = "output/docx/<reply-name>.docx"
LINE_SOURCE_PDF_DIR = PMT_REPLY_LINE_SOURCE_PDF_DIR
LINE_SOURCE_DOCX_DIR = PMT_REPLY_LINE_SOURCE_DOCX_DIR
LINE_SOURCE_CACHE_DIR = PMT_REPLY_LINE_SOURCE_CACHE_DIR
REPLY_PROBE_DIR = PMT_REPLY_PROBE_DIR
LABEL_CHARS_NO_DOT = r"A-Za-z0-9_:\-"
LABEL_CONTINUATION = rf"(?:[{LABEL_CHARS_NO_DOT}]|\.(?=[{LABEL_CHARS_NO_DOT}]))"
REF_PATTERN = re.compile(rf"@((?:sec|fig|tbl|eq):[A-Za-z0-9]{LABEL_CONTINUATION}*)")
REF_BOUNDARY = rf"(?![{LABEL_CHARS_NO_DOT}]|\.(?=[{LABEL_CHARS_NO_DOT}]))"
DISPLAY_EQUATION_LABEL_PATTERN = re.compile(
    rf"(?<!\$)\$\$(?!\$)(?P<math>.*?)(?<!\$)\$\$(?!\$)\s*"
    rf"\{{#(?P<label>eq:[A-Za-z0-9]{LABEL_CONTINUATION}*)(?P<attrs>[^}}]*)\}}",
    re.DOTALL,
)
CITATION_PATTERN = re.compile(r"(?<![\w:])@([A-Za-z0-9_][A-Za-z0-9_:.#/$%&+?<>~/-]*)")
CITATION_CLUSTER_PATTERN = re.compile(r"\[([^\]\n]*@[^\]\n]*)\]")
PROBE_SENTINEL = "PANDOC_REPLY_REF_PROBE"
CITATION_PROBE_SENTINEL = "PANDOC_REPLY_CITE_PROBE"
CITATION_CLUSTER_PROBE_SENTINEL = "PANDOC_REPLY_CITE_CLUSTER_PROBE"
CROSSREF_PREFIXES = ("sec:", "fig:", "tbl:", "eq:")
LINE_REGEX_PATTERN = re.compile(r"\(Line `([^`]+)`\)")
IMAGE_MARKDOWN_PATTERN = re.compile(r"!\[(?P<alt>[^\]]*)\]\((?P<target>[^)]*)\)(?:\s*\{[^}]*\})?")
LABELED_CAPTION_ATTRIBUTE_PATTERN = re.compile(
    r"(?m)^(?P<caption>\s*(?:Table|Figure)?:\s+.*?)"
    r"[ \t]*\{#(?:tbl|fig):[A-Za-z0-9][^}\r\n]*\}[ \t]*$"
)
STANDALONE_LABEL_ATTRIBUTE_PATTERN = re.compile(r"(?m)^[ \t]*\{#(?:eq|fig|tbl):[A-Za-z0-9][^}]*\}[ \t]*\r?\n?")
REPLY_CUSTOM_STYLE_DIV_OPEN_PATTERN = re.compile(
    r"^\s*:::\s*\{[^}\n]*custom-style\s*=\s*['\"]Reply to Reviewers['\"][^}\n]*\}\s*$"
)
DIV_CLOSE_PATTERN = re.compile(r"^\s*:::\s*$")
BR_TAG_PATTERN = re.compile(r"(?i)<br\s*/?>")
ESCAPED_ORDERED_LIST_MARKER_PATTERN = re.compile(r"(?m)^(\s*\d+)\\\.(?=\s)")
ORDERED_LIST_MARKER_PATTERN = re.compile(r"^\s*\d+\.(?=\s)")
EXTRA_BLANK_LINES_PATTERN = re.compile(r"(?:[ \t]*\r?\n){3,}")
ANSI_RED = "\033[31m"
ANSI_RESET = "\033[0m"
PREFIX_WORDS = {
    "sec": "Section",
    "fig": "Figure",
    "tbl": "Table",
    "eq": "Equation",
}
REPLY_EQUATION_OPENXML_PREFIX = (
    '<w:pPr><w:tabs>'
    '<w:tab w:val="center" w:leader="none" w:pos="4888" />'
    '<w:tab w:val="right" w:leader="none" w:pos="9746" />'
    '</w:tabs></w:pPr><w:r><w:tab /></w:r>'
)
REPLY_EQUATION_OPENXML_NUMBER_TAB = '<w:r><w:tab /></w:r>'
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
        description="Markdown, DOCX, or PDF source used for reply line placeholders.",
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
        description="Explicit reply DOCX or TXT output path.",
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


def run_command(cmd: list[str], env: dict[str, str] | None = None) -> subprocess.CompletedProcess[str]:
    """Run a command, echo it, and raise with captured output on failure."""
    log_info(f"[Run] {' '.join(cmd)}")
    result = subprocess.run(
        cmd,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        env=env,
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


def extract_labeled_equation_labels(markdown: str) -> list[str]:
    """Return equation labels attached to display-math blocks in reply text."""
    labels = sorted({match.group("label") for match in DISPLAY_EQUATION_LABEL_PATTERN.finditer(markdown)})
    log_info(f"[INFO] Found {len(labels)} labeled reply equation block(s).")
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
    work_dir = PMT_MATHTYPE_WORK_DIR / "reply"
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


def reply_svg_base_dirs(reply: Path) -> list[Path]:
    """Return lookup roots shared by reply DOCX SVG filters."""
    return svg_filter_helpers.unique_resolved_dirs([Path.cwd(), reply.parent])


def svg_embed_images_filter_env(
    reply: Path,
    metadata: dict[str, Any],
    embed_images: bool | None = None,
) -> dict[str, str]:
    """Return environment settings consumed by the reply SVG embedding filter."""
    return svg_filter_helpers.svg_embed_images_filter_env(
        reply_svg_base_dirs(reply),
        metadata,
        embed_images=embed_images,
    )


def svg_to_png_filter_env(
    reply: Path,
    metadata: dict[str, Any],
    convert_all: bool | None = None,
) -> dict[str, str]:
    """Return environment settings consumed by the reply SVG-to-PNG filter."""
    return svg_filter_helpers.svg_to_png_filter_env(
        reply_svg_base_dirs(reply),
        metadata,
        convert_all=convert_all,
    )


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
        work_dir=PMT_MATHTYPE_WORK_DIR / "reply" / target_docx.stem,
    )


def file_sha256(path: Path) -> str:
    """Return a SHA-256 digest for a dependency file."""
    return hashlib.sha256(path.read_bytes()).hexdigest()


def hash_json_payload(payload: dict[str, Any]) -> str:
    """Return a stable digest for a JSON-serializable cache payload."""
    text = json.dumps(payload, sort_keys=True, separators=(",", ":"))
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def dependency_record(path: Path) -> dict[str, str]:
    """Return the stable cache record for one input file."""
    resolved = path.resolve()
    return {
        "path": str(resolved),
        "sha256": file_sha256(resolved),
    }


def line_source_pdf_backend() -> str:
    """Return the DOCX-to-PDF backend used on this platform."""
    return "word-com" if sys.platform == "win32" else "soffice"


def cached_line_source_path(kind: str, key: str, suffix: str) -> Path:
    """Return a persistent line-source cache path for a computed key."""
    return LINE_SOURCE_CACHE_DIR / kind / f"{key}{suffix}"


def work_line_source_path(directory: Path, stem: str, key: str, suffix: str) -> Path:
    """Return a hash-suffixed transient line-source work path."""
    return directory / f"{stem}.{key[:12]}{suffix}"


def copy_from_cache(cached: Path, target: Path, label: str) -> bool:
    """Copy a cached line-source artifact into the work directory if present."""
    if not cached.exists():
        return False
    target.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(cached, target)
    log_info(f"[LINE] Reusing cached {label}: {target}")
    return True


def store_in_cache(source: Path, cached: Path, label: str) -> None:
    """Store a generated line-source artifact in the persistent cache."""
    cached.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(source, cached)
    log_debug(f"[LINE] Cached {label}: {cached}")


def docx_line_source_pdf_cache_key(source_docx: Path) -> str:
    """Return a fingerprint for converting a DOCX line source to PDF."""
    return hash_json_payload(
        {
            "kind": "docx-line-source-pdf",
            "schema": 1,
            "pmt_version": runtime_cache_version(),
            "backend": line_source_pdf_backend(),
            "source": dependency_record(source_docx),
        }
    )


def prepare_cached_docx_line_source_pdf(source_docx: Path) -> Path:
    """Convert or reuse the cached PDF for a DOCX line source."""
    key = docx_line_source_pdf_cache_key(source_docx)
    cached_pdf = cached_line_source_path("pdf", key, ".pdf")
    target_pdf = work_line_source_path(LINE_SOURCE_PDF_DIR, source_docx.stem, key, ".pdf")
    if copy_from_cache(cached_pdf, target_pdf, "line-source PDF"):
        return target_pdf

    if sys.platform == "win32":
        export_docx_to_pdf_with_word(source_docx, target_pdf)
    else:
        export_docx_to_pdf_with_soffice(source_docx, target_pdf)
    store_in_cache(target_pdf, cached_pdf, "line-source PDF")
    return target_pdf


def inline_to_text(inline: dict[str, Any]) -> str:
    """Convert a Pandoc JSON inline node to reply-safe Markdown text."""
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
    if tag == "Superscript":
        # Superscript CSL styles drop brackets and emit the cite as a Cite inline
        # directly after the probe label; keep Markdown markup for the final DOCX.
        return f"^{inlines_to_text(content or [])}^"
    if tag in {"Emph", "Strong", "Span", "SmallCaps", "Strikeout", "Subscript"}:
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
    """Flatten Pandoc JSON inlines into normalized reply Markdown text."""
    text = "".join(inline_to_text(inline) for inline in inlines)
    return re.sub(r"\s+", " ", text.replace("\u00a0", " ")).strip()


def split_probe_inlines(inlines: list[dict[str, Any]], sentinel: str) -> tuple[str, list[dict[str, Any]]] | None:
    """Return a probe label and display tail from a matching Pandoc paragraph."""
    if not inlines or inlines[0].get("t") != "Str" or inlines[0].get("c") != sentinel:
        return None

    index = 1
    while index < len(inlines) and inlines[index].get("t") in {"Space", "SoftBreak", "LineBreak"}:
        index += 1
    if index >= len(inlines) or inlines[index].get("t") != "Str":
        return None

    label = str(inlines[index].get("c"))
    index += 1
    while index < len(inlines) and inlines[index].get("t") in {"Space", "SoftBreak", "LineBreak"}:
        index += 1
    return label, inlines[index:]


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
        probe = split_probe_inlines(inlines, sentinel)
        if probe is None:
            continue
        label, display_inlines = probe
        display = inlines_to_text(display_inlines)
        if display and label in requested and not any(marker in display for marker in unresolved_markers):
            resolved[label] = display

    return resolved


def write_probe_file(name: str, lines: list[str]) -> Path:
    """Write a stable probe file under .pmt without relying on tempfile ACLs."""
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
        pandoc_command(),
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
    result = run_command(cmd, env=pandoc_tools_env())
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
        pandoc_command(),
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
    result = run_command(cmd, env=pandoc_tools_env())
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
        pandoc_command(),
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
    result = run_command(cmd, env=pandoc_tools_env())
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


def compact_display_math_for_inline(math: str) -> str:
    """Collapse display-math line breaks so Pandoc keeps the tab-layout formula inline."""
    return re.sub(r"[ \t]*\r?\n[ \t]*", " ", math.strip())


def equation_label_number(label: str, reference_map: dict[str, str]) -> str | None:
    """Return a parenthesized equation number resolved from manuscript crossrefs."""
    display = reference_map.get(label)
    if not display:
        return None

    short = number_only(label, display)
    if re.fullmatch(r"\(.+\)", short):
        return short
    return f"({short})"


def raw_openxml_inline(xml: str) -> str:
    """Wrap a small OpenXML fragment as a Pandoc raw inline."""
    return f"`{xml}`{{=openxml}}"


def replace_labeled_equation_blocks(markdown: str, reference_map: dict[str, str]) -> str:
    """Render labeled reply equations with manuscript numbers and Word tab stops."""
    replacements = 0

    def replace_match(match: re.Match[str]) -> str:
        """Return a tab-layout equation paragraph or keep unresolved syntax unchanged."""
        nonlocal replacements
        label = match.group("label")
        number = equation_label_number(label, reference_map)
        if number is None:
            return match.group(0)

        replacements += 1
        math = compact_display_math_for_inline(match.group("math"))
        # Reply builds intentionally skip pandoc-crossref on the reply itself, so
        # use the manuscript-resolved number and the same tab layout as style.yml.
        return (
            f"{raw_openxml_inline(REPLY_EQUATION_OPENXML_PREFIX)}"
            f"${math}$"
            f"{raw_openxml_inline(REPLY_EQUATION_OPENXML_NUMBER_TAB)}"
            f"{number}"
        )

    resolved = DISPLAY_EQUATION_LABEL_PATTERN.sub(replace_match, markdown)
    if replacements:
        log_info(f"[INFO] Formatted {replacements} labeled reply equation block(s) with manuscript numbering.")
    return resolved


def strip_labeled_equation_attributes(markdown: str) -> str:
    """Remove reply-side equation labels while preserving display-math Markdown."""

    def replace_match(match: re.Match[str]) -> str:
        """Return the original display equation without the Pandoc label attribute."""
        return f"$${match.group('math')}$$"

    return DISPLAY_EQUATION_LABEL_PATTERN.sub(replace_match, markdown)


def image_placeholder(match: re.Match[str]) -> str:
    """Return a readable placeholder for an image removed from TXT output."""
    alt = match.group("alt").strip()
    target = match.group("target").strip().split(None, 1)[0]
    label = alt or target or "image"
    return f"[Image: {label}]"


def strip_reply_custom_style_divs(markdown: str) -> str:
    """Remove reply-only custom-style div wrappers from TXT output."""
    stripped_lines: list[str] = []
    reply_div_depth = 0
    for line in markdown.splitlines(keepends=True):
        line_text = line.strip()
        if REPLY_CUSTOM_STYLE_DIV_OPEN_PATTERN.fullmatch(line_text):
            reply_div_depth += 1
            continue
        if reply_div_depth and DIV_CLOSE_PATTERN.fullmatch(line_text):
            reply_div_depth -= 1
            continue
        stripped_lines.append(line)
    return "".join(stripped_lines)


def normalize_txt_markdown_spacing(markdown: str) -> str:
    """Collapse cleanup leftovers to at most one blank line."""
    return EXTRA_BLANK_LINES_PATTERN.sub("\n\n", markdown).strip("\r\n")


def unescape_ordered_list_markers(markdown: str) -> str:
    """Restore escaped ordered-list markers such as `1\\.` in TXT output."""
    return ESCAPED_ORDERED_LIST_MARKER_PATTERN.sub(r"\1.", markdown)


def ensure_blank_line_before_ordered_lists(markdown: str) -> str:
    """Reinsert one blank line before an ordered-list item after prose or captions."""
    lines = markdown.splitlines()
    normalized_lines: list[str] = []
    for line in lines:
        if (
            ORDERED_LIST_MARKER_PATTERN.match(line)
            and normalized_lines
            and normalized_lines[-1].strip()
            and not ORDERED_LIST_MARKER_PATTERN.match(normalized_lines[-1])
        ):
            normalized_lines.append("")
        normalized_lines.append(line)
    return "\n".join(normalized_lines)


def render_reply_txt_markdown(markdown: str) -> str:
    """Prepare resolved reply Markdown for journal TXT submission."""
    text = strip_reply_custom_style_divs(markdown)
    text = BR_TAG_PATTERN.sub("", text)
    text = unescape_ordered_list_markers(text)
    text = ensure_blank_line_before_ordered_lists(text)
    text = IMAGE_MARKDOWN_PATTERN.sub(image_placeholder, text)
    text = strip_labeled_equation_attributes(text)
    text = LABELED_CAPTION_ATTRIBUTE_PATTERN.sub(lambda match: match.group("caption"), text)
    text = STANDALONE_LABEL_ATTRIBUTE_PATTERN.sub("", text)
    text = normalize_txt_markdown_spacing(text)
    if text and not text.endswith("\n"):
        text += "\n"
    return text


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


def resolve_reply_markdown(
    reply_text: str,
    manuscript: Path,
    manuscript_line_source: Path,
    flattened_style: Path,
    from_format: str,
    *,
    format_labeled_equations: bool,
) -> str:
    """Resolve manuscript-derived reply placeholders before writing an output format."""
    labels = extract_reference_labels(reply_text)
    if format_labeled_equations:
        labels += extract_labeled_equation_labels(reply_text)
    labels = sorted(set(labels))
    citations = extract_citation_keys(reply_text)
    citation_clusters = extract_citation_clusters(reply_text)
    reference_map = resolve_reference_map(manuscript, flattened_style, labels, from_format)
    citation_map = resolve_citation_map(manuscript, flattened_style, citations, from_format)
    citation_cluster_map = resolve_citation_cluster_map(manuscript, flattened_style, citation_clusters, from_format)

    resolved_text = resolve_line_regexes(reply_text, manuscript_line_source)
    if format_labeled_equations:
        resolved_text = replace_labeled_equation_blocks(resolved_text, reference_map)
    resolved_text = replace_references(resolved_text, reference_map)
    return replace_citations(resolved_text, citation_map, citation_cluster_map)


def normalized_pdf_line_text(text: str) -> str:
    """Normalize one PDF text line so regexes can survive extraction artifacts."""
    return re.sub(r"\s+", " ", text.replace("\u00a0", " ")).strip()


def pdf_metadata_source(metadata: dict[str, str] | None) -> str:
    """Join PDF metadata fields for producer-specific extraction decisions."""
    if not metadata:
        return ""
    return " ".join(str(value) for value in metadata.values()).lower()


def is_libreoffice_pdf(metadata: dict[str, str] | None) -> bool:
    """Return whether PDF metadata identifies LibreOffice as the producer."""
    return "libreoffice" in pdf_metadata_source(metadata)


def is_microsoft_word_pdf(metadata: dict[str, str] | None) -> bool:
    """Return whether PDF metadata identifies Microsoft Word as the producer."""
    source = pdf_metadata_source(metadata)
    return "microsoft" in source and "word" in source


def extract_pdf_numbered_lines_by_text_order(document: Any) -> list[tuple[int, int, str]]:
    """Extract line-number pairs from PDFs whose text stream interleaves text then number."""
    numbered_lines: list[tuple[int, int, str]] = []
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
    return numbered_lines


def y_center(bbox: tuple[float, float, float, float]) -> float:
    """Return a text-line bounding box's vertical center for layout matching."""
    return (bbox[1] + bbox[3]) / 2


def group_body_lines_by_y(
    text_lines: list[tuple[str, tuple[float, float, float, float]]],
) -> list[tuple[str, tuple[float, float, float, float]]]:
    """Keep one body text line per y position, avoiding isolated superscript fragments."""
    groups: list[list[tuple[str, tuple[float, float, float, float]]]] = []
    for text, bbox in text_lines:
        center = y_center(bbox)
        for group in groups:
            if abs(y_center(group[0][1]) - center) <= 1.0:
                group.append((text, bbox))
                break
        else:
            groups.append([(text, bbox)])

    body_lines: list[tuple[str, tuple[float, float, float, float]]] = []
    for group in groups:
        body_lines.append(max(group, key=lambda item: len(item[0])))
    return body_lines


def keep_increasing_line_numbers(
    numbered_lines: list[tuple[int, int, str]],
) -> list[tuple[int, int, str]]:
    """Drop LibreOffice footnote line-number resets that appear after body line numbers."""
    kept: list[tuple[int, int, str]] = []
    last_line_number = 0
    for item in numbered_lines:
        line_number, _, _ = item
        if line_number <= last_line_number:
            log_debug(f"[LINE] Skipping non-increasing LibreOffice line number: {line_number}")
            continue
        kept.append(item)
        last_line_number = line_number
    return kept


def extract_pdf_numbered_lines_by_layout(document: Any) -> list[tuple[int, int, str]]:
    """Pair left-margin line numbers with body text by y coordinate for LibreOffice PDFs."""
    numbered_lines: list[tuple[int, int, str]] = []
    for page_index, page in enumerate(document, start=1):
        number_lines: list[tuple[int, tuple[float, float, float, float]]] = []
        text_lines: list[tuple[str, tuple[float, float, float, float]]] = []

        for block in page.get_text("dict").get("blocks", []):
            if block.get("type") != 0:
                continue
            for line in block.get("lines", []):
                bbox = line.get("bbox")
                if not bbox:
                    continue
                text = normalized_pdf_line_text(
                    "".join(str(span.get("text", "")) for span in line.get("spans", []))
                )
                if not text:
                    continue
                line_bbox = tuple(float(value) for value in bbox)
                if re.fullmatch(r"\d+", text):
                    number_lines.append((int(text), line_bbox))
                else:
                    text_lines.append((text, line_bbox))

        if not number_lines or not text_lines:
            continue

        body_left = min(bbox[0] for _, bbox in text_lines)
        margin_numbers = [(number, bbox) for number, bbox in number_lines if bbox[2] < body_left - 2]
        body_lines = group_body_lines_by_y(text_lines)

        for number, number_bbox in margin_numbers:
            number_y = y_center(number_bbox)
            matches = [
                (text, bbox)
                for text, bbox in body_lines
                if abs(y_center(bbox) - number_y) <= max(3.0, (number_bbox[3] - number_bbox[1]) * 0.75)
            ]
            if not matches:
                continue
            text, _ = max(matches, key=lambda item: len(item[0]))
            numbered_lines.append((number, page_index, text))
    return keep_increasing_line_numbers(numbered_lines)


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


def export_docx_to_pdf_with_soffice(source_docx: Path, target_pdf: Path) -> None:
    """Export a DOCX line source to PDF through LibreOffice's soffice CLI."""
    source_docx = source_docx.resolve()
    target_pdf = target_pdf.resolve()
    target_pdf.parent.mkdir(parents=True, exist_ok=True)

    expected_pdf = target_pdf.parent / f"{source_docx.stem}.pdf"
    target_pdf.unlink(missing_ok=True)
    if expected_pdf != target_pdf:
        expected_pdf.unlink(missing_ok=True)

    log_info(f"[LINE] Converting DOCX line source to PDF with soffice: {source_docx}")
    result = subprocess.run(
        [
            "soffice",
            "--headless",
            "--convert-to",
            "pdf",
            "--outdir",
            str(target_pdf.parent),
            str(source_docx),
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
        raise RuntimeError(f"soffice DOCX-to-PDF conversion failed: {source_docx}")
    if not expected_pdf.exists():
        raise RuntimeError(f"soffice conversion did not create PDF: {expected_pdf}")
    if expected_pdf != target_pdf:
        shutil.move(str(expected_pdf), str(target_pdf))
    log_info(f"[LINE] soffice PDF created: {target_pdf}")


def build_markdown_line_source_docx(source_markdown: Path, target_docx: Path) -> None:
    """Build a Markdown line source to DOCX before converting it to PDF."""
    from . import build as manuscript_build

    source_markdown = source_markdown.resolve()
    target_docx = target_docx.resolve()
    target_docx.parent.mkdir(parents=True, exist_ok=True)

    previous_settings = manuscript_build.SETTINGS.model_copy(deep=True)
    result = 1
    try:
        log_info(f"[LINE] Building Markdown line source DOCX: {source_markdown}")
        result = manuscript_build.run_build_command(
            target="docx",
            markdown=str(source_markdown),
            output_file=str(target_docx),
        )
    finally:
        for key, value in previous_settings.model_dump().items():
            setattr(manuscript_build.SETTINGS, key, value)

    if result != 0:
        raise RuntimeError(f"Markdown line-source DOCX build failed: {source_markdown}")
    if not target_docx.exists():
        raise RuntimeError(f"Markdown line-source DOCX build did not create: {target_docx}")
    log_info(f"[LINE] Markdown line source DOCX created: {target_docx}")


def prepare_line_source_pdf(line_source: Path) -> Path:
    """Return a PDF path for line-regex matching, converting DOCX sources if needed."""
    if not line_source.exists():
        raise FileNotFoundError(f"Line source file not found: {line_source}")

    suffix = line_source.suffix.lower()
    if suffix == ".pdf":
        return line_source
    if suffix in {".md", ".markdown"}:
        target_docx = LINE_SOURCE_DOCX_DIR / f"{line_source.stem}.docx"
        build_markdown_line_source_docx(line_source, target_docx)
        line_source = target_docx
        suffix = line_source.suffix.lower()
    if suffix in {".docx", ".docm"}:
        return prepare_cached_docx_line_source_pdf(line_source)
    raise RuntimeError(f"Line source must be a Markdown, PDF, or Word document: {line_source}")


def extract_pdf_numbered_lines(pdf: Path) -> list[tuple[int, int, str]]:
    """Extract manuscript line numbers and their corresponding text from a PDF."""
    try:
        import fitz
    except ImportError as exc:
        raise RuntimeError("PyMuPDF is required to resolve reply line regexes.") from exc

    if not pdf.exists():
        raise FileNotFoundError(f"Manuscript PDF not found for line resolution: {pdf}")

    with fitz.open(pdf) as document:
        if is_libreoffice_pdf(document.metadata):
            numbered_lines = extract_pdf_numbered_lines_by_layout(document)
            log_info(
                f"[INFO] Extracted {len(numbered_lines)} numbered PDF text lines from {pdf} using LibreOffice layout matching."
            )
            return numbered_lines

        if is_microsoft_word_pdf(document.metadata):
            numbered_lines = extract_pdf_numbered_lines_by_layout(document)
            if numbered_lines:
                log_info(
                    f"[INFO] Extracted {len(numbered_lines)} numbered PDF text lines from {pdf} using Word layout matching."
                )
                return numbered_lines
            log_warning("[WARN] Microsoft Word PDF layout matching found no numbered lines; falling back to text order.")

        numbered_lines = extract_pdf_numbered_lines_by_text_order(document)

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
    """Fail early when an existing output is locked by another app."""
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
            raise RuntimeError(f"Output file appears to be open or locked. Close it and retry: {output}") from exc
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

    resolved_text = resolve_reply_markdown(
        reply_text,
        manuscript,
        manuscript_line_source,
        flattened_style,
        from_format,
        format_labeled_equations=True,
    )

    output.parent.mkdir(parents=True, exist_ok=True)
    temp_reply_path = resolved_reply_path(reply)
    temp_reply_path.write_text(resolved_text, encoding="utf-8")

    try:
        mathtype_args = mathtype_filter_args() if use_mathtype else []
        embed_svg_images = should_embed_docx_svg_images(metadata)
        convert_all_svg = should_convert_docx_svg_to_png(metadata)
        if embed_svg_images:
            log_info("[INFO] Embedding linked child images inside reply SVG files for DOCX")
        if convert_all_svg:
            log_info("[INFO] Converting referenced reply SVG images to PNG for DOCX")
        svg_filter_args = [
            *svg_embed_images_filter_args(),
            *svg_to_png_filter_args(),
        ]
        svg_filter_env = {
            **svg_embed_images_filter_env(reply, metadata, embed_images=embed_svg_images),
            **svg_to_png_filter_env(reply, metadata, convert_all=convert_all_svg),
        }
        cmd = [
            pandoc_command(),
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
            *svg_filter_args,
            *mathtype_args,
        ]
        run_command(cmd, env=pandoc_tools_env(svg_filter_env))

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


def build_reply_txt(
    reply: Path,
    manuscript: Path,
    manuscript_line_source: Path,
    output: Path,
    style: Path,
    from_format: str,
) -> None:
    """Build a reviewer-reply TXT file with resolved manuscript placeholders."""
    ensure_output_writable(output)
    reply_text = reply.read_text(encoding="utf-8")
    flattened_style = write_reply_style_metadata_file(style)
    resolved_text = resolve_reply_markdown(
        reply_text,
        manuscript,
        manuscript_line_source,
        flattened_style,
        from_format,
        format_labeled_equations=False,
    )
    txt_text = render_reply_txt_markdown(resolved_text)

    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(txt_text, encoding="utf-8")
    log_success(f"[OK] Reply TXT created: {output}")


def checked_markdown_path(markdown_path: str | Path) -> Path:
    """Return an existing reply markdown path, raising clear input errors."""
    path = Path(markdown_path)
    if not path.exists():
        raise FileNotFoundError(f"Reply markdown file not found: {path}")
    if not path.is_file():
        raise ValueError(f"Reply markdown path is not a file: {path}")
    return path


def reply_output_path(reply: Path, output_file: str | None) -> Path:
    """Return the exact output file for a reply build."""
    if output_file and output_file != DEFAULT_REPLY_OUTPUT_FILE:
        return Path(output_file)
    return Path(DEFAULT_OUTPUT_DIR) / "docx" / f"{reply.stem}.docx"


def reply_output_format(output: Path) -> str:
    """Return the reply output format selected by the output file suffix."""
    suffix = output.suffix.lower()
    if suffix in {".docx", ".txt"}:
        return suffix.removeprefix(".")
    raise ValueError(f"Unsupported reply output suffix `{output.suffix}`; use .docx or .txt")


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
        output = reply_output_path(reply, output_file)
        output_format = reply_output_format(output)
        manuscript = Path(reply_manuscript or DEFAULT_REPLY_MANUSCRIPT_FILE)
        line_source = Path(manuscript_line_source or DEFAULT_REPLY_LINE_SOURCE)
        active_from_format = from_format or DEFAULT_REPLY_FROM_FORMAT
        if output_format == "txt":
            log_info("\n[TXT] Building reviewer reply TXT...\n")
            build_reply_txt(
                reply=reply,
                manuscript=manuscript,
                manuscript_line_source=line_source,
                output=output,
                style=Path(DEFAULT_STYLE_FILE),
                from_format=active_from_format,
            )
        else:
            log_info("\n[DOCX] Building reviewer reply DOCX...\n")
            build_reply_docx(
                reply=reply,
                manuscript=manuscript,
                manuscript_line_source=line_source,
                output=output,
                reference_doc=reply_reference_doc_path(reference_doc),
                style=Path(DEFAULT_STYLE_FILE),
                from_format=active_from_format,
            )
        return 0
    except KeyboardInterrupt:
        log_warning("\n\n[WARN] Build interrupted by user.")
        return 1
    except Exception as exc:
        log_error(f"\n[ERROR] {exc}")
        return 1
