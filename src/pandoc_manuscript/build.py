"""
Build normal manuscript targets for the Pandoc manuscript template.
"""

import errno
import os
import shutil
import subprocess
from pathlib import Path
from typing import Any, Tuple

from pydantic_settings import BaseSettings, SettingsConfigDict

from .logging_utils import log_error, log_info, log_success, log_warning
from .metadata import (
    MissingYamlFrontMatterError,
    load_merged_metadata_with_status,
    parse_yaml_file,
    parse_yaml_header,
)
from .mathtype.convert_marked_docx import convert_marked_docx
from .mathtype.ole_parts import build_helper, check_mathtype_availability
from .paths import (
    PMT_DIR,
    PMT_MATHTYPE_WORK_DIR,
    PMT_WORK_DIR,
    pmt_path,
)
from .postprocess.final_docx_syntax_check import validate_final_docx_syntax
from .postprocess_docx import postprocess_docx as run_docx_postprocess
from .resources import package_resource_path, template_root
from .svg_filters import (
    python_filter_wrapper,
    should_convert_docx_svg_to_png,
    should_embed_docx_svg_images,
)
from . import svg_filters as svg_filter_helpers
from .tools import ensure_pandoc_tools, pandoc_command, pandoc_tools_env

# ============================================================================
# SETTINGS
# ============================================================================


DEFAULT_OUTPUT_DIR = "output"


class BuildSettings(BaseSettings):
    """Typed runtime settings shared by CLI parsing and build functions."""

    model_config = SettingsConfigDict(env_prefix="PMT_", extra="ignore")

    project_name: str = "manuscript"
    manuscript_file: str = "manuscript.md"
    style_file: str = "style.yml"
    output_dir: str = DEFAULT_OUTPUT_DIR
    docx_dir: str = "output/docx"
    latex_dir: str = "output/latex"
    json_dir: str = "output/json"
    output_file: str | None = None
    enable_docx_postprocess: bool = True
    mathtype_marker_filter: str = "mathtype/mathtype_markers.lua"
    mathtype_work_dir: str = pmt_path(PMT_MATHTYPE_WORK_DIR)
    reference_doc: str | None = None


SETTINGS = BuildSettings()

# ============================================================================
# UTILITY FUNCTIONS
# ============================================================================

def run_command(
    cmd: list,
    cwd: Path | None = None,
    check: bool = True,
    stream_output: bool = False,
    env: dict[str, str] | None = None,
) -> subprocess.CompletedProcess:
    """Run a shell command and return the result.

    Args:
        cmd: Command and arguments as a list
        cwd: Working directory for the command
        check: Raise exception on non-zero exit code
        stream_output: If True, stream stdout/stderr to console in real-time
        env: Extra environment variables for this command
    """
    log_info(f"[Run] {' '.join(str(c) for c in cmd)}")
    command_env = None if env is None else {**os.environ, **env}

    if stream_output:
        # Stream output to console in real-time
        return subprocess.run(cmd, cwd=cwd, check=check, env=command_env)
    else:
        # Capture output (for commands where we need to parse it)
        return subprocess.run(cmd, cwd=cwd, check=check, capture_output=True, text=True, env=command_env)


def get_pandoc_version() -> Tuple[int, ...]:
    """Get Pandoc version as a tuple of integers."""
    try:
        pandoc = ensure_pandoc_tools()[0].executable
        result = subprocess.run([str(pandoc), '--version'], capture_output=True, text=True, check=True)
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


def configure_output_file(output_file: str | Path | None) -> None:
    """Configure an exact manuscript output file and use its parent as output root."""
    if output_file is None:
        SETTINGS.output_file = None
        return

    path = Path(output_file)
    SETTINGS.output_file = to_pandoc_path(path)
    configure_output_dir(path.parent)


def configure_reference_doc(reference_doc: str | None) -> None:
    """Configure a user-supplied reference DOCX for DOCX-producing targets."""
    if reference_doc:
        SETTINGS.reference_doc = reference_doc


def manuscript_output_file(default_dir: str, suffix: str) -> Path:
    """Return the explicit output file, or the target's default derived file."""
    if SETTINGS.output_file:
        return Path(SETTINGS.output_file)
    return Path(default_dir) / f"{SETTINGS.project_name}.{suffix}"


def ensure_output_parent(output_file: Path) -> None:
    """Create the parent directory for an explicit or derived build output."""
    output_file.parent.mkdir(parents=True, exist_ok=True)


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


def docx_svg_to_png_filter_args() -> list[str]:
    """Return Pandoc args for the DOCX SVG-to-PNG image filter."""
    return svg_filter_helpers.svg_to_png_filter_args()


def docx_svg_embed_images_filter_args() -> list[str]:
    """Return Pandoc args for the DOCX self-contained SVG image filter."""
    return svg_filter_helpers.svg_embed_images_filter_args()


def docx_svg_base_dirs() -> list[Path]:
    """Return lookup roots shared by DOCX SVG filters."""
    manuscript_dir = Path(SETTINGS.manuscript_file).parent
    return svg_filter_helpers.unique_resolved_dirs([Path.cwd(), manuscript_dir])


def docx_svg_embed_images_filter_env(
    metadata: dict[str, Any],
    embed_images: bool | None = None,
) -> dict[str, str]:
    """Return environment settings consumed by the SVG child-image embedding filter."""
    return svg_filter_helpers.svg_embed_images_filter_env(
        docx_svg_base_dirs(),
        metadata,
        embed_images=embed_images,
    )


def docx_svg_to_png_filter_env(metadata: dict[str, Any], convert_all: bool | None = None) -> dict[str, str]:
    """Return environment settings consumed by the SVG-to-PNG Pandoc filter."""
    return svg_filter_helpers.svg_to_png_filter_env(
        docx_svg_base_dirs(),
        metadata,
        convert_all=convert_all,
    )


def table_metadata_filter_args() -> list[str]:
    """Return Pandoc args for embedding hidden table-attribute markers into DOCX."""
    filter_path = resource_path("pandoc/filters/table_metadata.lua")
    if not filter_path.exists():
        raise FileNotFoundError(f"Table metadata Pandoc filter not found: {filter_path}")
    return ["--lua-filter", to_pandoc_path(filter_path)]


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
    extra_env: dict[str, str] | None = None,
) -> None:
    """Run Pandoc with original defaults so ${.} resolves beside that file."""
    extra_args = extra_args or []
    cmd = [
        pandoc_command(),
        '--defaults',
        str(defaults_file),
        *style_metadata_args(),
        *project_csl_args(),
        '--output',
        to_pandoc_path(output_file),
        *extra_args,
        SETTINGS.manuscript_file,
    ]
    run_command(cmd, stream_output=True, env=pandoc_tools_env(extra_env))


def reference_doc_args() -> list[str]:
    """Return Pandoc args that override the bundled DOCX reference document."""
    if not SETTINGS.reference_doc:
        return []
    return ['--reference-doc', to_pandoc_path(Path(SETTINGS.reference_doc))]


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


def build_docx():
    """Generate DOCX file with optional post-processing."""
    log_info("\n[DOCX] Building DOCX...\n")

    docx_file = manuscript_output_file(SETTINGS.docx_dir, "docx")
    ensure_output_parent(docx_file)
    ensure_docx_target_writable(docx_file)
    extra_args = []
    metadata = load_build_metadata()
    use_mathtype = resolve_mathtype_build_enabled(should_use_mathtype(metadata))
    pandoc_output = docx_file
    pandoc_env = {}
    extra_args.extend(reference_doc_args())
    extra_args.extend(table_metadata_filter_args())

    embed_svg_images = should_embed_docx_svg_images(metadata)
    if embed_svg_images:
        log_info("[INFO] Embedding linked child images inside SVG files for DOCX")
    extra_args.extend(docx_svg_embed_images_filter_args())
    pandoc_env.update(docx_svg_embed_images_filter_env(metadata, embed_images=embed_svg_images))

    convert_all_svg = should_convert_docx_svg_to_png(metadata)
    if convert_all_svg:
        log_info("[INFO] Converting referenced SVG images to PNG for DOCX")
    extra_args.extend(docx_svg_to_png_filter_args())
    pandoc_env.update(docx_svg_to_png_filter_env(metadata, convert_all=convert_all_svg))

    # Add filter for older Pandoc versions
    if should_use_mathbfit_filter():
        log_info("[INFO] Using mathbfit filter (Pandoc <= 3.8.3.0)")
        extra_args.extend(['--filter', to_pandoc_path(resource_path('pandoc/filters/to_mathbfit.py'))])

    if use_mathtype:
        pandoc_output = mathtype_marked_docx_path()
        extra_args.extend(mathtype_filter_args())

    # Run pandoc
    run_pandoc(
        resource_path('pandoc/pandoc-docx.yml'),
        pandoc_output,
        extra_args=extra_args,
        extra_env=pandoc_env,
    )

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

    log_success(f"\n[OK] DOCX created: {docx_file}")


def build_latex():
    """Generate LaTeX file."""
    log_info("\n[LaTeX] Building LaTeX...\n")

    latex_file = manuscript_output_file(SETTINGS.latex_dir, "tex")
    ensure_output_parent(latex_file)

    # Run pandoc
    run_pandoc(resource_path('pandoc/pandoc-latex.yml'), latex_file)

    log_success(f"\n[OK] LaTeX created: {latex_file}")


def build_json():
    """Generate Pandoc JSON AST for debugging filters and metadata."""
    log_info("\n[JSON] Building Pandoc JSON AST...\n")

    json_file = manuscript_output_file(SETTINGS.json_dir, "json")
    ensure_output_parent(json_file)

    # Reuse the DOCX defaults because they carry the normal crossref/citeproc
    # pipeline users most often need to inspect when debugging manuscript builds.
    run_pandoc(
        resource_path('pandoc/pandoc-docx.yml'),
        json_file,
        extra_args=['--to', 'json'],
    )

    log_success(f"\n[OK] JSON created: {json_file}")


def clean():
    """Remove generated outputs and transient work files, keeping reusable caches."""
    log_info("\n[Clean] Cleaning generated files...\n")

    output_dir = Path(SETTINGS.output_dir)
    if output_dir.exists():
        ensure_safe_clean_dir(output_dir)
        shutil.rmtree(output_dir)
        log_info(f"Removed: {output_dir}")

    if PMT_WORK_DIR.exists():
        shutil.rmtree(PMT_WORK_DIR)
        log_info(f"Removed: {PMT_WORK_DIR}")

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
    """Deep clean generated outputs, transient work files, and reusable caches."""
    log_info("\n[Clean] Deep cleaning...\n")

    # Regular clean
    clean()

    for cache_dir in (PMT_DIR, Path(".pandoc-cache")):
        if cache_dir.exists():
            shutil.rmtree(cache_dir)
            log_info(f"Removed: {cache_dir}")

    log_success("\n[OK] Deep clean complete.")


def run_build_command(
    *,
    target: str = "docx",
    markdown: str | None = None,
    manuscript_option: str | None = None,
    output_dir: str | None = None,
    output_file: str | None = None,
    reference_doc: str | None = None,
) -> int:
    """Run the selected manuscript build target with direct settings values."""
    configure_output_file(None)
    if output_dir and target in {"docx", "latex", "json"}:
        raise ValueError(f"--output-dir is not supported by the {target} target; use --output-file instead.")
    if output_file and target not in {"docx", "latex", "json"}:
        raise ValueError("--output-file is only supported by the docx, latex, and json targets.")
    if output_dir:
        configure_output_dir(output_dir)
    if output_file:
        configure_output_file(output_file)

    if markdown and manuscript_option:
        raise ValueError("Specify the markdown file either positionally or with --manuscript, not both.")

    manuscript_arg = manuscript_option or markdown
    if manuscript_arg and target not in {"docx", "latex", "json"}:
        raise ValueError("A markdown file can only be specified for docx, latex, or json targets.")
    if reference_doc and target != "docx":
        raise ValueError("--reference-doc is only supported by the docx target.")
    configure_reference_doc(reference_doc)

    if target in {"docx", "latex", "json"}:
        configure_manuscript(manuscript_arg or SETTINGS.manuscript_file, derive_project_name=bool(manuscript_arg))

    targets = {
        "docx": build_docx,
        "latex": build_latex,
        "json": build_json,
        "clean": clean,
        "distclean": distclean,
    }

    try:
        targets[target]()
        return 0
    except KeyboardInterrupt:
        log_warning("\n\n[WARN] Build interrupted by user.")
        return 1
    except Exception as e:
        log_error(f"\n[ERROR] {e}")
        return 1
