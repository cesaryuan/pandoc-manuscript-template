#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = [
#   "python-docx>=1.1.0",
#   "pyyaml>=6.0",
#   "lxml>=4.9.0",
#   "panflute>=2.3.0",
#   "dotenv>=0.9.0",
# ]
# ///
"""
Pandoc Manuscript Template - Build Script

A simple and readable build system for academic manuscripts.
Replaces the complex Makefile with clean Python code.

Usage:
    python build.py docx       # Generate DOCX (default)
    python build.py docx paper.md  # Generate DOCX from a specific markdown file
    python build.py docx paper.md --output-dir build  # Generate DOCX in build/docx
    python build.py latex      # Generate LaTeX
    python build.py latex paper.md # Generate LaTeX from a specific markdown file
    python build.py latex paper.md --output-dir build # Generate LaTeX in build/latex
    python build.py json       # Generate Pandoc JSON AST for debugging
    python build.py json paper.md --output-dir build  # Generate JSON in build/json
    python build.py clean      # Remove generated files
    python build.py distclean  # Deep clean (including cache)
    python build.py help       # Show this help

Configuration:
    Edit the CONFIG section below to customize your build.
"""

import argparse
import errno
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Any, Callable, Tuple

import yaml
from metadata import load_merged_metadata
from mathtype.ole_parts import check_mathtype_availability

# ============================================================================
# CONFIGURATION - Customize these variables for your project
# ============================================================================

CONFIG = {
    'project_name': 'manuscript',
    'manuscript_file': 'manuscript.md',
    'style_file': 'style.yml',
    'output_dir': 'output',
    'docx_dir': 'output/docx',
    'latex_dir': 'output/latex',
    'json_dir': 'output/json',
    'enable_docx_postprocess': True,
    'mathtype_marker_filter': 'pandoc/filters/mathtype_markers.lua',
    'mathtype_work_dir': 'tmp/mathtype-build',
}

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
    print(f"[Run] {' '.join(str(c) for c in cmd)}")

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
        print(f"Warning: Could not detect Pandoc version: {e}")
        return (999, 0, 0, 0)  # Assume latest version


def should_use_mathbfit_filter() -> bool:
    """Check if we should use the mathbfit filter (Pandoc <= 3.8.3.0)."""
    version = get_pandoc_version()
    threshold = (3, 8, 3, 0)
    return version <= threshold


def to_pandoc_path(path: Path) -> str:
    """Return a Pandoc-friendly path string for generated defaults files."""
    return path.as_posix()


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

    CONFIG['manuscript_file'] = to_pandoc_path(path)
    if derive_project_name:
        CONFIG['project_name'] = path.stem


def configure_output_dir(output_dir: str | Path) -> None:
    """Configure the base output directory and derived DOCX/LaTeX directories."""
    path = Path(output_dir)
    if path.exists() and not path.is_dir():
        raise ValueError(f"Output path exists but is not a directory: {path}")

    CONFIG['output_dir'] = to_pandoc_path(path)
    CONFIG['docx_dir'] = to_pandoc_path(path / 'docx')
    CONFIG['latex_dir'] = to_pandoc_path(path / 'latex')
    CONFIG['json_dir'] = to_pandoc_path(path / 'json')


def pandoc_defaults_with_runtime_paths(defaults_file: Path, output_file: Path) -> dict:
    """Load a Pandoc defaults file and replace input/output paths at runtime."""
    with defaults_file.open('r', encoding='utf-8') as f:
        defaults = yaml.safe_load(f) or {}

    defaults['input-files'] = [CONFIG['manuscript_file']]
    defaults['output-file'] = to_pandoc_path(output_file)
    add_style_metadata_file(defaults)
    return defaults


def add_style_metadata_file(defaults: dict) -> None:
    """Add style.yml as build-time defaults that manuscript metadata can override."""
    style_file = Path(CONFIG['style_file'])
    if not should_use_style_metadata_file():
        return

    metadata_files = defaults.get('metadata-files') or []
    if isinstance(metadata_files, (str, Path)):
        metadata_files = [to_pandoc_path(Path(metadata_files))]

    style_path = to_pandoc_path(style_file)
    if style_path not in metadata_files:
        # Put style.yml first so any existing metadata-files and the manuscript
        # YAML header can override these output-style defaults.
        defaults['metadata-files'] = [style_path, *metadata_files]


def should_use_style_metadata_file() -> bool:
    """Return True when the configured style metadata file exists for this build."""
    style_file = Path(CONFIG['style_file'])
    if not style_file.exists():
        print(f"[INFO] Style metadata file not found, skipping: {style_file}")
        return False
    return True


def style_metadata_cli_args() -> list[str]:
    """Return post-processing CLI args for the optional style metadata file."""
    if not should_use_style_metadata_file():
        return []
    return ['--metadata-file', CONFIG['style_file']]


def style_metadata_files() -> list[str]:
    """Return existing style metadata files in the same order Pandoc receives them."""
    if not should_use_style_metadata_file():
        return []
    return [CONFIG['style_file']]


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
    return load_merged_metadata(CONFIG['manuscript_file'], style_metadata_files())


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
    work_dir = Path(CONFIG['mathtype_work_dir'])
    work_dir.mkdir(parents=True, exist_ok=True)
    # Keep the marker DOCX out of the final output directory but preserve it for
    # debugging failed conversions.
    return work_dir / f"{CONFIG['project_name']}.marked.docx"


def mathtype_filter_args() -> list[str]:
    """Return Pandoc args that insert hidden LaTeX markers before DOCX writing."""
    marker_filter = Path(CONFIG['mathtype_marker_filter'])
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

    print("[INFO] MathType DOCX equations enabled by metadata: mathtype: true")
    availability = check_mathtype_availability()
    if availability.usable:
        return True

    print(
        availability.format_failure(
            "[WARN] MathType was requested by metadata, but MathType conversion will be skipped."
        )
    )
    print("[WARN] Building DOCX with Pandoc/Word equations instead.\n")
    return False


def run_mathtype_conversion(marked_docx: Path, target_docx: Path) -> None:
    """Convert a marked DOCX's OMML equations into MathType OLE equations."""
    print("\n[DOCX] Converting equations to MathType OLE objects...\n")
    run_command(
        [
            sys.executable,
            'scripts/mathtype/convert_marked_docx.py',
            '--mode',
            'all',
            '--source',
            str(marked_docx),
            '--target',
            str(target_docx),
            '--work-dir',
            str(Path(CONFIG['mathtype_work_dir']) / CONFIG['project_name']),
        ],
        stream_output=True,
    )


def json_debug_defaults(defaults: dict) -> dict:
    """Return DOCX-like defaults adjusted for Pandoc JSON debugging output.

    The JSON build is meant to expose the filtered Pandoc AST, so it keeps the
    DOCX filter chain but removes writer-only options that do not apply to JSON.
    """
    defaults = dict(defaults)
    defaults['to'] = 'json'
    defaults.pop('reference-doc', None)
    defaults.pop('template', None)
    return defaults


def run_pandoc(
    defaults_file: Path,
    output_file: Path,
    extra_args: list[str] | None = None,
    defaults_mutator: Callable[[dict], dict] | None = None,
) -> None:
    """Run Pandoc with a temporary defaults file for the selected markdown input."""
    extra_args = extra_args or []
    defaults = pandoc_defaults_with_runtime_paths(defaults_file, output_file)
    if defaults_mutator:
        defaults = defaults_mutator(defaults)

    with tempfile.TemporaryDirectory(prefix='pandoc-build-') as temp_dir:
        temp_defaults = Path(temp_dir) / defaults_file.name
        with temp_defaults.open('w', encoding='utf-8') as f:
            yaml.safe_dump(defaults, f, sort_keys=False, allow_unicode=True)

        # The repo defaults keep manuscript.md fixed; this temporary copy lets
        # command-line builds target another markdown file without editing YAML.
        cmd = ['pandoc', '--defaults', str(temp_defaults), *extra_args]
        run_command(cmd, stream_output=True)


# ============================================================================
# BUILD TARGETS
# ============================================================================

def build_docx():
    """Generate DOCX file with optional post-processing."""
    print("\n[DOCX] Building DOCX...\n")

    # Create output directory
    docx_dir = Path(CONFIG['docx_dir'])
    docx_dir.mkdir(parents=True, exist_ok=True)

    docx_file = docx_dir / f"{CONFIG['project_name']}.docx"
    ensure_docx_target_writable(docx_file)
    extra_args = []
    metadata = load_build_metadata()
    use_mathtype = resolve_mathtype_build_enabled(should_use_mathtype(metadata))
    pandoc_output = docx_file

    # Add filter for older Pandoc versions
    if should_use_mathbfit_filter():
        print("[INFO] Using mathbfit filter (Pandoc <= 3.8.3.0)")
        extra_args.extend(['--filter', 'pandoc/filters/to_mathbfit.py'])

    if use_mathtype:
        pandoc_output = mathtype_marked_docx_path()
        extra_args.extend(mathtype_filter_args())

    # Run pandoc
    run_pandoc(Path('pandoc/pandoc-docx.yml'), pandoc_output, extra_args=extra_args)

    # Post-process DOCX if enabled
    if CONFIG['enable_docx_postprocess']:
        print("\n[DOCX] Running Python post-processing...\n")
        postprocess_docx = pandoc_output if use_mathtype else docx_file
        # When MathType is enabled, post-process the marker DOCX before
        # replacing OMML. Several DOCX fixes detect equation layout tables from
        # OMML, which is gone after OLE conversion.
        run_command(
            [
                'uv',
                'run',
                'scripts/postprocess_docx.py',
                str(postprocess_docx),
                CONFIG['manuscript_file'],
                *style_metadata_cli_args(),
            ],
            stream_output=True,
        )

    if use_mathtype:
        run_mathtype_conversion(pandoc_output, docx_file)

    print(f"\n[OK] DOCX created: {CONFIG['docx_dir']}/{CONFIG['project_name']}.docx")


def build_latex():
    """Generate LaTeX file."""
    print("\n[LaTeX] Building LaTeX...\n")

    # Create output directory
    latex_dir = Path(CONFIG['latex_dir'])
    latex_dir.mkdir(parents=True, exist_ok=True)

    latex_file = latex_dir / f"{CONFIG['project_name']}.tex"

    # Run pandoc
    run_pandoc(Path('pandoc/pandoc-latex.yml'), latex_file)

    print(f"\n[OK] LaTeX created: {CONFIG['latex_dir']}/{CONFIG['project_name']}.tex")


def build_json():
    """Generate Pandoc JSON AST for debugging filters and metadata."""
    print("\n[JSON] Building Pandoc JSON AST...\n")

    json_dir = Path(CONFIG['json_dir'])
    json_dir.mkdir(parents=True, exist_ok=True)

    json_file = json_dir / f"{CONFIG['project_name']}.json"

    # Reuse the DOCX defaults because they carry the normal crossref/citeproc
    # pipeline users most often need to inspect when debugging manuscript builds.
    run_pandoc(
        Path('pandoc/pandoc-docx.yml'),
        json_file,
        defaults_mutator=json_debug_defaults,
    )

    print(f"\n[OK] JSON created: {CONFIG['json_dir']}/{CONFIG['project_name']}.json")


def clean():
    """Remove all generated files."""
    print("\n[Clean] Cleaning generated files...\n")

    output_dir = Path(CONFIG['output_dir'])
    if output_dir.exists():
        ensure_safe_clean_dir(output_dir)
        shutil.rmtree(output_dir)
        print(f"Removed: {output_dir}")

    print("\n[OK] Clean complete.")


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
    print("\n[Clean] Deep cleaning...\n")

    # Regular clean
    clean()

    # Remove Pandoc cache
    cache_dir = Path('.pandoc-cache')
    if cache_dir.exists():
        shutil.rmtree(cache_dir)
        print(f"Removed: {cache_dir}")

    print("\n[OK] Deep clean complete.")


def show_help():
    """Show help information."""
    print(__doc__)
    print("\nCurrent Configuration:")
    for key, value in CONFIG.items():
        print(f"  {key}: {value}")
    print("\nTo disable DOCX post-processing:")
    print("  Set CONFIG['enable_docx_postprocess'] = False in build.py")


# ============================================================================
# MAIN
# ============================================================================

def main():
    parser = argparse.ArgumentParser(
        description='Build system for Pandoc manuscript template',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog=__doc__
    )
    parser.add_argument(
        'target',
        nargs='?',
        default='docx',
        choices=['docx', 'latex', 'json', 'clean', 'distclean', 'help'],
        help='Build target (default: docx)'
    )
    parser.add_argument(
        'manuscript',
        nargs='?',
        help='Markdown file to build for docx/latex/json targets'
    )
    parser.add_argument(
        '-m',
        '--manuscript',
        dest='manuscript_option',
        help='Markdown file to build for docx/latex/json targets'
    )
    parser.add_argument(
        '-o',
        '--output-dir',
        help='Base output directory (default: output)'
    )

    args = parser.parse_args()
    if args.output_dir:
        configure_output_dir(args.output_dir)

    if args.manuscript and args.manuscript_option:
        parser.error("Specify the markdown file either positionally or with --manuscript, not both.")

    manuscript_arg = args.manuscript_option or args.manuscript
    if manuscript_arg and args.target not in {'docx', 'latex', 'json'}:
        parser.error("A markdown file can only be specified for docx, latex, or json targets.")

    if args.target in {'docx', 'latex', 'json'}:
        configure_manuscript(
            manuscript_arg or CONFIG['manuscript_file'],
            derive_project_name=bool(manuscript_arg),
        )

    # Dispatch to target function
    targets = {
        'docx': build_docx,
        'latex': build_latex,
        'json': build_json,
        'clean': clean,
        'distclean': distclean,
        'help': show_help,
    }

    try:
        targets[args.target]()
    except KeyboardInterrupt:
        print("\n\n[WARN] Build interrupted by user.")
        sys.exit(1)
    except Exception as e:
        print(f"\n[ERROR] {e}")
        sys.exit(1)


if __name__ == '__main__':
    main()
