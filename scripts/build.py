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
    python build.py latex      # Generate LaTeX
    python build.py latex paper.md # Generate LaTeX from a specific markdown file
    python build.py clean      # Remove generated files
    python build.py distclean  # Deep clean (including cache)
    python build.py help       # Show this help

Configuration:
    Edit the CONFIG section below to customize your build.
"""

import argparse
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Tuple

import yaml

# ============================================================================
# CONFIGURATION - Customize these variables for your project
# ============================================================================

CONFIG = {
    'project_name': 'manuscript',
    'manuscript_file': 'manuscript.md',
    'output_dir': 'output',
    'docx_dir': 'output/docx',
    'latex_dir': 'output/latex',
    'enable_docx_postprocess': True,
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


def pandoc_defaults_with_runtime_paths(defaults_file: Path, output_file: Path) -> dict:
    """Load a Pandoc defaults file and replace input/output paths at runtime."""
    with defaults_file.open('r', encoding='utf-8') as f:
        defaults = yaml.safe_load(f) or {}

    defaults['input-files'] = [CONFIG['manuscript_file']]
    defaults['output-file'] = to_pandoc_path(output_file)
    return defaults


def run_pandoc(defaults_file: Path, output_file: Path, extra_args: list[str] | None = None) -> None:
    """Run Pandoc with a temporary defaults file for the selected markdown input."""
    extra_args = extra_args or []
    defaults = pandoc_defaults_with_runtime_paths(defaults_file, output_file)

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
    extra_args = []

    # Add filter for older Pandoc versions
    if should_use_mathbfit_filter():
        print("[INFO] Using mathbfit filter (Pandoc <= 3.8.3.0)")
        extra_args.extend(['--filter', 'pandoc/filters/to_mathbfit.py'])

    # Run pandoc
    run_pandoc(Path('pandoc/pandoc-docx.yml'), docx_file, extra_args=extra_args)

    # Post-process DOCX if enabled
    if CONFIG['enable_docx_postprocess']:
        print("\n[DOCX] Running Python post-processing...\n")
        run_command(
            ['uv', 'run', 'scripts/postprocess_docx.py', str(docx_file), CONFIG['manuscript_file']],
            stream_output=True,
        )

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


def clean():
    """Remove all generated files."""
    print("\n[Clean] Cleaning generated files...\n")

    output_dir = Path(CONFIG['output_dir'])
    if output_dir.exists():
        shutil.rmtree(output_dir)
        print(f"Removed: {output_dir}")

    print("\n[OK] Clean complete.")


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
        choices=['docx', 'latex', 'clean', 'distclean', 'help'],
        help='Build target (default: docx)'
    )
    parser.add_argument(
        'manuscript',
        nargs='?',
        help='Markdown file to build for docx/latex targets'
    )
    parser.add_argument(
        '-m',
        '--manuscript',
        dest='manuscript_option',
        help='Markdown file to build for docx/latex targets'
    )

    args = parser.parse_args()
    if args.manuscript and args.manuscript_option:
        parser.error("Specify the markdown file either positionally or with --manuscript, not both.")

    manuscript_arg = args.manuscript_option or args.manuscript
    if manuscript_arg and args.target not in {'docx', 'latex'}:
        parser.error("A markdown file can only be specified for docx or latex targets.")

    if args.target in {'docx', 'latex'}:
        configure_manuscript(
            manuscript_arg or CONFIG['manuscript_file'],
            derive_project_name=bool(manuscript_arg),
        )

    # Dispatch to target function
    targets = {
        'docx': build_docx,
        'latex': build_latex,
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
