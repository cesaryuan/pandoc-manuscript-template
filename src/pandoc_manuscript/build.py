#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = [
#   "python-docx>=1.1.0",
#   "pyyaml>=6.0",
#   "lxml>=4.9.0",
#   "panflute>=2.3.0",
#   "pymupdf>=1.26.0",
#   "pydantic-settings>=2.0.0",
#   "dotenv>=0.9.0",
# ]
# ///
"""
Pandoc Manuscript Template - Build Script

A simple and readable build system for academic manuscripts.
Replaces the complex Makefile with clean Python code.

Usage:
    pmt build docx       # Generate DOCX (default)
    pmt build docx paper.md  # Generate DOCX from a specific markdown file
    pmt build docx paper.md --output-dir build  # Generate DOCX in build/docx
    pmt build reply reply.md  # Generate reviewer reply DOCX
    pmt build latex      # Generate LaTeX
    pmt build latex paper.md # Generate LaTeX from a specific markdown file
    pmt build latex paper.md --output-dir build # Generate LaTeX in build/latex
    pmt build json       # Generate Pandoc JSON AST for debugging
    pmt build json paper.md --output-dir build  # Generate JSON in build/json
    pmt build clean      # Remove generated files
    pmt build distclean  # Deep clean (including cache)
    pmt build --help     # Show CLI help

Configuration:
    Build settings are typed by BuildSettings below and can be overridden by CLI args.
"""

import errno
import shutil
import subprocess
from pathlib import Path
from typing import Any, Literal, Tuple

from pydantic_settings import BaseSettings, CliApp, CliPositionalArg, SettingsConfigDict

from .logging_utils import log_error, log_info, log_success, log_warning
from .metadata import (
    MissingYamlFrontMatterError,
    load_merged_metadata_with_status,
    parse_yaml_file,
    parse_yaml_header,
)
from .mathtype.convert_marked_docx import convert_marked_docx
from .mathtype.ole_parts import build_helper, check_mathtype_availability
from .postprocess.final_docx_syntax_check import validate_final_docx_syntax
from .postprocess_docx import postprocess_docx as run_docx_postprocess
from .reply_build import build_reply_docx
from .resources import package_resource_path, template_root

# ============================================================================
# SETTINGS
# ============================================================================


class BuildSettings(BaseSettings):
    """Typed runtime settings shared by CLI parsing and build functions."""

    model_config = SettingsConfigDict(env_prefix="PMT_", extra="ignore")

    project_name: str = "manuscript"
    manuscript_file: str = "manuscript.md"
    style_file: str = "style.yml"
    output_dir: str = "output"
    docx_dir: str = "output/docx"
    latex_dir: str = "output/latex"
    json_dir: str = "output/json"
    enable_docx_postprocess: bool = True
    mathtype_marker_filter: str = "mathtype/mathtype_markers.lua"
    mathtype_work_dir: str = "tmp/mathtype-build"
    reference_doc: str | None = None
    reply_manuscript_file: str = "manuscript.md"
    reply_line_source: str = "output/docx/manuscript.docx"
    reply_from_format: str = "markdown"
    reply_output_file: str | None = None


SETTINGS = BuildSettings()

BuildTarget = Literal["docx", "reply", "latex", "json", "clean", "distclean"]


class BuildCliSettings(BaseSettings):
    """CLI settings for manuscript builds parsed by pydantic-settings."""

    model_config = SettingsConfigDict(
        cli_kebab_case=True,
        cli_shortcuts={
            "manuscript_option": ["-m", "--manuscript"],
            "output_dir": ["-o", "--output-dir"],
        },
    )

    target: CliPositionalArg[BuildTarget] = "docx"
    markdown: CliPositionalArg[str | None] = None
    manuscript_option: str | None = None
    output_dir: str | None = None
    reply_manuscript: str | None = None
    manuscript_line_source: str | None = None
    from_format: str | None = None
    reference_doc: str | None = None
    output_file: str | None = None

    def cli_cmd(self) -> None:
        """Execute the parsed build command when run through CliApp."""
        raise SystemExit(run_build_command(self))

# ============================================================================
# UTILITY FUNCTIONS
# ============================================================================

def run_command(cmd: list, cwd: Path | None = None, check: bool = True, stream_output: bool = False) -> subprocess.CompletedProcess:
    """Run a shell command and return the result.

    Args:
        cmd: Command and arguments as a list
        cwd: Working directory for the command
        check: Raise exception on non-zero exit code
        stream_output: If True, stream stdout/stderr to console in real-time
    """
    log_info(f"[Run] {' '.join(str(c) for c in cmd)}")

    if stream_output:
        # Stream output to console in real-time
        return subprocess.run(cmd, cwd=cwd, check=check)
    else:
        # Capture output (for commands where we need to parse it)
        return subprocess.run(cmd, cwd=cwd, check=check, capture_output=True, text=True)


def get_pandoc_version() -> Tuple[int, ...]:
    """Get Pandoc version as a tuple of integers."""
    try:
        result = subprocess.run(['pandoc', '--version'], capture_output=True, text=True, check=True)
        version_line = result.stdout.split('\n')[0]
        version_str = version_line.split()[1]
        # Handle versions like "3.8.3" or "3.8.3.0"
        parts = version_str.split('.')
        # Pad with zeros if needed (e.g., "3.8.3" -> [3, 8, 3, 0])
        while len(parts) < 4:
            parts.append('0')
        return tuple(int(p) for p in parts[:4])
    except Exception as e:
        log_warning(f"[WARN] Could not detect Pandoc version: {e}")
        return (999, 0, 0, 0)  # Assume latest version


def should_use_mathbfit_filter() -> bool:
    """Check if we should use the mathbfit filter (Pandoc <= 3.8.3.0)."""
    version = get_pandoc_version()
    threshold = (3, 8, 3, 0)
    return version <= threshold


def to_pandoc_path(path: Path) -> str:
    """Return a Pandoc-friendly path string for generated defaults files."""
    return path.as_posix()


def resource_path(path: str | Path) -> Path:
    """Resolve a bundled pmt template or package runtime resource path."""
    path = Path(path)
    if path.is_absolute():
        return path
    if path.parts and path.parts[0] in {"mathtype", "mathtype_ole_helper"}:
        return package_resource_path(path)
    return template_root() / path


def is_relative_to(path: Path, parent: Path) -> bool:
    """Return True when path is inside parent on Python versions without Path.is_relative_to needs."""
    try:
        path.relative_to(parent)
        return True
    except ValueError:
        return False


def configure_manuscript(markdown_path: str | Path, derive_project_name: bool = False) -> None:
    """Configure the runtime markdown input and validate that it exists.

    derive_project_name is used for command-line markdown overrides so a custom
    input such as paper.md writes paper.docx/paper.tex instead of manuscript.*.
    """
    path = Path(markdown_path)
    if not path.exists():
        raise FileNotFoundError(f"Markdown file not found: {path}")
    if not path.is_file():
        raise ValueError(f"Markdown path is not a file: {path}")

    SETTINGS.manuscript_file = to_pandoc_path(path)
    if derive_project_name:
        SETTINGS.project_name = path.stem


def configure_output_dir(output_dir: str | Path) -> None:
    """Configure the base output directory and derived DOCX/LaTeX directories."""
    path = Path(output_dir)
    if path.exists() and not path.is_dir():
        raise ValueError(f"Output path exists but is not a directory: {path}")

    SETTINGS.output_dir = to_pandoc_path(path)
    SETTINGS.docx_dir = to_pandoc_path(path / 'docx')
    SETTINGS.latex_dir = to_pandoc_path(path / 'latex')
    SETTINGS.json_dir = to_pandoc_path(path / 'json')


def configure_reference_doc(reference_doc: str | None) -> None:
    """Configure a user-supplied reference DOCX for DOCX-producing targets."""
    if reference_doc:
        SETTINGS.reference_doc = reference_doc


def configure_reply_options(
    *,
    manuscript: str | None = None,
    line_source: str | None = None,
    from_format: str | None = None,
    output_file: str | None = None,
) -> None:
    """Configure reviewer-reply build inputs while keeping manuscript defaults stable."""
    if manuscript:
        SETTINGS.reply_manuscript_file = manuscript
    if line_source:
        SETTINGS.reply_line_source = line_source
    if from_format:
        SETTINGS.reply_from_format = from_format
    if output_file:
        SETTINGS.reply_output_file = output_file


def should_use_style_metadata_file() -> bool:
    """Return True when the configured style metadata file exists for this build."""
    style_file = Path(SETTINGS.style_file)
    if not style_file.exists():
        log_info(f"[INFO] Style metadata file not found, skipping: {style_file}")
        return False
    return True


def style_metadata_files() -> list[str]:
    """Return existing style metadata files in the same order Pandoc receives them."""
    if not should_use_style_metadata_file():
        return []
    return [SETTINGS.style_file]


def ensure_docx_target_writable(target: Path) -> None:
    """Fail early when an existing DOCX target is open or cannot be overwritten.

    Word commonly locks an opened DOCX on Windows. Pandoc would otherwise fail
    later with a less direct error, so probe write access before starting work.
    """
    if not target.exists():
        return
    if target.is_dir():
        raise RuntimeError(f"Target DOCX path is a directory and cannot be overwritten: {target}")

    try:
        with target.open('r+b'):
            pass
    except OSError as exc:
        lock_like_errors = {errno.EACCES, errno.EPERM}
        lock_like_winerrors = {5, 32, 33}
        if exc.errno in lock_like_errors or getattr(exc, 'winerror', None) in lock_like_winerrors:
            raise RuntimeError(
                "目标 DOCX 可能已经在 Word 中打开，或正被其他程序占用，当前无法写入。\n"
                f"请关闭后重试: {target}"
            ) from exc
        raise


def load_build_metadata() -> dict[str, Any]:
    """Load style defaults plus manuscript metadata for build-time feature flags."""
    metadata_files = style_metadata_files()
    metadata, has_yaml_header = load_merged_metadata_with_status(
        SETTINGS.manuscript_file,
        metadata_files,
        allow_missing_header=True,
    )
    if not has_yaml_header:
        # Reply-style documents may omit manuscript YAML; keep style.yml defaults.
        log_warning(f"[WARN] No YAML front matter found in {SETTINGS.manuscript_file}; using metadata files only")
    return metadata


def metadata_bool(value: Any) -> bool:
    """Normalize YAML feature flags such as mathtype: true or mathtype: yes."""
    if isinstance(value, bool):
        return value
    if isinstance(value, str):
        return value.strip().lower() in {'1', 'true', 'yes', 'on'}
    if isinstance(value, (int, float)):
        return bool(value)
    return False


def should_use_mathtype(metadata: dict[str, Any]) -> bool:
    """Return True when merged metadata requests MathType DOCX equations."""
    return metadata_bool(metadata.get('mathtype'))


def mathtype_marked_docx_path() -> Path:
    """Return the intermediate DOCX path that carries hidden LaTeX markers."""
    work_dir = Path(SETTINGS.mathtype_work_dir)
    work_dir.mkdir(parents=True, exist_ok=True)
    # Keep the marker DOCX out of the final output directory but preserve it for
    # debugging failed conversions.
    return work_dir / f"{SETTINGS.project_name}.marked.docx"


def mathtype_filter_args() -> list[str]:
    """Return Pandoc args that insert hidden LaTeX markers before DOCX writing."""
    marker_filter = resource_path(SETTINGS.mathtype_marker_filter)
    if not marker_filter.exists():
        raise FileNotFoundError(f"MathType marker filter not found: {marker_filter}")
    return ['--lua-filter', to_pandoc_path(marker_filter)]


def resolve_mathtype_build_enabled(requested: bool) -> bool:
    """Return whether this build should actually run MathType conversion.

    Users may keep `mathtype: true` in shared style metadata on machines that
    do not have MathType installed. In that case, continue with normal
    Pandoc/Word equations instead of failing the whole DOCX build.
    """
    if not requested:
        return False

    log_info("[INFO] MathType DOCX equations enabled by metadata: mathtype: true")
    availability = check_mathtype_availability()
    if availability.usable:
        return True

    log_warning(
        availability.format_failure(
            "[WARN] MathType was requested by metadata, but MathType conversion will be skipped."
        )
    )
    log_warning("[WARN] Building DOCX with Pandoc/Word equations instead.\n")
    return False


def run_mathtype_conversion(marked_docx: Path, target_docx: Path) -> None:
    """Convert a marked DOCX's OMML equations into MathType OLE equations."""
    log_info("\n[DOCX] Converting equations to MathType OLE objects...\n")
    build_helper()
    convert_marked_docx(
        source=marked_docx,
        target=target_docx,
        work_dir=Path(SETTINGS.mathtype_work_dir) / SETTINGS.project_name,
    )


def run_pandoc(
    defaults_file: Path,
    output_file: Path,
    extra_args: list[str] | None = None,
) -> None:
    """Run Pandoc with original defaults so ${.} resolves beside that file."""
    extra_args = extra_args or []
    cmd = [
        'pandoc',
        '--defaults',
        str(defaults_file),
        *style_metadata_args(),
        *project_csl_args(),
        '--output',
        to_pandoc_path(output_file),
        *extra_args,
        SETTINGS.manuscript_file,
    ]
    run_command(cmd, stream_output=True)


def reference_doc_args() -> list[str]:
    """Return Pandoc args that override the bundled DOCX reference document."""
    if not SETTINGS.reference_doc:
        return []
    return ['--reference-doc', to_pandoc_path(Path(SETTINGS.reference_doc))]


def reply_reference_doc_path() -> Path:
    """Return the active reference DOCX path for reviewer reply builds."""
    if SETTINGS.reference_doc:
        return Path(SETTINGS.reference_doc)
    return resource_path('pandoc/manuscript-template/reference-doc.docx')


def style_metadata_args() -> list[str]:
    """Return Pandoc CLI args for project style metadata when style.yml exists."""
    return [
        arg
        for metadata_file in style_metadata_files()
        for arg in ('--metadata-file', metadata_file)
    ]


def project_csl_args() -> list[str]:
    """Return a project-relative --csl override when metadata explicitly sets CSL."""
    csl = project_metadata_csl()
    if not csl:
        return []
    return ['--csl', str(csl)]


def project_metadata_csl() -> Any:
    """Return CSL configured by style.yml or manuscript YAML, if present."""
    csl: Any = None
    for metadata_file in style_metadata_files():
        csl = parse_yaml_file(metadata_file).get('csl', csl)

    try:
        csl = parse_yaml_header(SETTINGS.manuscript_file).get('csl', csl)
    except MissingYamlFrontMatterError:
        pass
    return csl


# ============================================================================
# BUILD TARGETS
# ============================================================================

def build_docx():
    """Generate DOCX file with optional post-processing."""
    log_info("\n[DOCX] Building DOCX...\n")

    # Create output directory
    docx_dir = Path(SETTINGS.docx_dir)
    docx_dir.mkdir(parents=True, exist_ok=True)

    docx_file = docx_dir / f"{SETTINGS.project_name}.docx"
    ensure_docx_target_writable(docx_file)
    extra_args = []
    metadata = load_build_metadata()
    use_mathtype = resolve_mathtype_build_enabled(should_use_mathtype(metadata))
    pandoc_output = docx_file
    extra_args.extend(reference_doc_args())

    # Add filter for older Pandoc versions
    if should_use_mathbfit_filter():
        log_info("[INFO] Using mathbfit filter (Pandoc <= 3.8.3.0)")
        extra_args.extend(['--filter', to_pandoc_path(resource_path('pandoc/filters/to_mathbfit.py'))])

    if use_mathtype:
        pandoc_output = mathtype_marked_docx_path()
        extra_args.extend(mathtype_filter_args())

    # Run pandoc
    run_pandoc(resource_path('pandoc/pandoc-docx.yml'), pandoc_output, extra_args=extra_args)

    # Post-process DOCX if enabled
    if SETTINGS.enable_docx_postprocess:
        log_info("\n[DOCX] Running Python post-processing...\n")
        postprocess_target = pandoc_output if use_mathtype else docx_file
        # When MathType is enabled, post-process the marker DOCX before
        # replacing OMML. Several DOCX fixes detect equation layout tables from
        # OMML, which is gone after OLE conversion.
        if not run_docx_postprocess(str(postprocess_target), metadata):
            raise RuntimeError("DOCX post-processing failed")

    if use_mathtype:
        run_mathtype_conversion(pandoc_output, docx_file)

    # Final output validation should inspect the real shipped DOCX rather than
    # an intermediate pre-MathType file, so syntax residue cannot slip through.
    syntax_findings = validate_final_docx_syntax(docx_file)
    if syntax_findings:
        raise RuntimeError("Final DOCX still contains unrendered Pandoc syntax")

    log_success(f"\n[OK] DOCX created: {SETTINGS.docx_dir}/{SETTINGS.project_name}.docx")


def build_reply_docx_target() -> None:
    """Generate a reviewer-reply DOCX using manuscript numbering and reply styling."""
    log_info("\n[DOCX] Building reviewer reply DOCX...\n")
    output = reply_output_path()
    build_reply_docx(
        reply=Path(SETTINGS.manuscript_file),
        manuscript=Path(SETTINGS.reply_manuscript_file),
        manuscript_line_source=Path(SETTINGS.reply_line_source),
        output=output,
        reference_doc=reply_reference_doc_path(),
        style=Path(SETTINGS.style_file),
        from_format=SETTINGS.reply_from_format,
    )


def reply_output_path() -> Path:
    """Return the output DOCX path for reply builds."""
    if SETTINGS.reply_output_file:
        return Path(SETTINGS.reply_output_file)
    docx_dir = Path(SETTINGS.docx_dir)
    docx_dir.mkdir(parents=True, exist_ok=True)
    return docx_dir / f"{SETTINGS.project_name}.docx"


def build_latex():
    """Generate LaTeX file."""
    log_info("\n[LaTeX] Building LaTeX...\n")

    # Create output directory
    latex_dir = Path(SETTINGS.latex_dir)
    latex_dir.mkdir(parents=True, exist_ok=True)

    latex_file = latex_dir / f"{SETTINGS.project_name}.tex"

    # Run pandoc
    run_pandoc(resource_path('pandoc/pandoc-latex.yml'), latex_file)

    log_success(f"\n[OK] LaTeX created: {SETTINGS.latex_dir}/{SETTINGS.project_name}.tex")


def build_json():
    """Generate Pandoc JSON AST for debugging filters and metadata."""
    log_info("\n[JSON] Building Pandoc JSON AST...\n")

    json_dir = Path(SETTINGS.json_dir)
    json_dir.mkdir(parents=True, exist_ok=True)

    json_file = json_dir / f"{SETTINGS.project_name}.json"

    # Reuse the DOCX defaults because they carry the normal crossref/citeproc
    # pipeline users most often need to inspect when debugging manuscript builds.
    run_pandoc(
        resource_path('pandoc/pandoc-docx.yml'),
        json_file,
        extra_args=['--to', 'json'],
    )

    log_success(f"\n[OK] JSON created: {SETTINGS.json_dir}/{SETTINGS.project_name}.json")


def clean():
    """Remove all generated files."""
    log_info("\n[Clean] Cleaning generated files...\n")

    output_dir = Path(SETTINGS.output_dir)
    if output_dir.exists():
        ensure_safe_clean_dir(output_dir)
        shutil.rmtree(output_dir)
        log_info(f"Removed: {output_dir}")

    log_success("\n[OK] Clean complete.")


def ensure_safe_clean_dir(output_dir: Path) -> None:
    """Reject unsafe recursive clean targets caused by a custom output directory."""
    project_dir = Path.cwd().resolve()
    resolved_output = output_dir.resolve()

    # Custom output directories make clean more flexible, but deleting the
    # project root or a directory outside the project would be too easy to do by
    # accident with options such as --output-dir . or --output-dir ..
    if resolved_output == project_dir or not is_relative_to(resolved_output, project_dir):
        raise ValueError(f"Refusing to clean unsafe output directory: {output_dir}")


def distclean():
    """Deep clean (including Pandoc cache)."""
    log_info("\n[Clean] Deep cleaning...\n")

    # Regular clean
    clean()

    # Remove Pandoc cache
    cache_dir = Path('.pandoc-cache')
    if cache_dir.exists():
        shutil.rmtree(cache_dir)
        log_info(f"Removed: {cache_dir}")

    log_success("\n[OK] Deep clean complete.")


# ============================================================================
# MAIN
# ============================================================================

def run_build_command(args: BuildCliSettings) -> int:
    """Apply parsed CLI settings and run the selected build target."""
    if args.output_dir:
        configure_output_dir(args.output_dir)
    configure_reference_doc(args.reference_doc)
    configure_reply_options(
        manuscript=args.reply_manuscript,
        line_source=args.manuscript_line_source,
        from_format=args.from_format,
        output_file=args.output_file,
    )

    if args.markdown and args.manuscript_option:
        raise ValueError("Specify the markdown file either positionally or with --manuscript, not both.")

    manuscript_arg = args.manuscript_option or args.markdown
    if manuscript_arg and args.target not in {'docx', 'reply', 'latex', 'json'}:
        raise ValueError("A markdown file can only be specified for docx, reply, latex, or json targets.")
    if args.output_file and args.target != 'reply':
        raise ValueError("--output-file is only supported by the reply target.")
    if args.reference_doc and args.target not in {'docx', 'reply'}:
        raise ValueError("--reference-doc is only supported by docx and reply targets.")
    if any([args.reply_manuscript, args.manuscript_line_source, args.from_format]) and args.target != 'reply':
        raise ValueError("Reply-specific options require the reply target.")

    if args.target in {'docx', 'reply', 'latex', 'json'}:
        configure_manuscript(
            manuscript_arg or default_input_for_target(args.target),
            derive_project_name=bool(manuscript_arg) or args.target == 'reply',
        )

    targets = {
        'docx': build_docx,
        'reply': build_reply_docx_target,
        'latex': build_latex,
        'json': build_json,
        'clean': clean,
        'distclean': distclean,
    }

    try:
        targets[args.target]()
        return 0
    except KeyboardInterrupt:
        log_warning("\n\n[WARN] Build interrupted by user.")
        return 1
    except Exception as e:
        log_error(f"\n[ERROR] {e}")
        return 1


def main(argv: list[str] | None = None) -> None:
    """Run the build CLI using pydantic-settings."""
    CliApp.run(BuildCliSettings, cli_args=argv, cli_parse_args=True)


def default_input_for_target(target: str) -> str:
    """Return the default markdown input for the selected build target."""
    if target != 'reply':
        return SETTINGS.manuscript_file
    legacy_reply = Path('submissions/dbe/reply_to_reviewers_first.md')
    if legacy_reply.exists():
        return to_pandoc_path(legacy_reply)
    return 'reply.md'


if __name__ == '__main__':
    main()
