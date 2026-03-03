#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = [
#   "python-docx>=1.1.0",
#   "pyyaml>=6.0",
#   "lxml>=4.9.0",
#   "panflute>=2.3.0",
# ]
# ///
"""
Pandoc Manuscript Template - Build Script

A simple and readable build system for academic manuscripts.
Replaces the complex Makefile with clean Python code.

Usage:
    python build.py docx       # Generate DOCX (default)
    python build.py latex      # Generate LaTeX
    python build.py pdf        # Generate PDF
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
from pathlib import Path
from typing import Tuple

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
    print(f"→ Running: {' '.join(str(c) for c in cmd)}")

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


# ============================================================================
# BUILD TARGETS
# ============================================================================

def build_docx():
    """Generate DOCX file with optional post-processing."""
    print("\n📄 Building DOCX...\n")

    # Create output directory
    docx_dir = Path(CONFIG['docx_dir'])
    docx_dir.mkdir(parents=True, exist_ok=True)

    # Build pandoc command
    cmd = ['pandoc', '--defaults', 'pandoc/pandoc-docx.yml']

    # Add filter for older Pandoc versions
    if should_use_mathbfit_filter():
        print("ℹ️  Using mathbfit filter (Pandoc ≤ 3.8.3.0)")
        cmd.extend(['--filter', 'pandoc/filters/to_mathbfit.py'])

    # Run pandoc
    run_command(cmd, stream_output=True)

    # Post-process DOCX if enabled
    if CONFIG['enable_docx_postprocess']:
        print("\n🔧 Running Python post-processing...\n")
        docx_file = docx_dir / f"{CONFIG['project_name']}.docx"
        run_command(['uv', 'run', 'scripts/postprocess_docx.py', str(docx_file)], stream_output=True)

    print(f"\n✅ DOCX created: {CONFIG['docx_dir']}/{CONFIG['project_name']}.docx")


def build_latex():
    """Generate LaTeX file."""
    print("\n📝 Building LaTeX...\n")

    # Create output directory
    latex_dir = Path(CONFIG['latex_dir'])
    latex_dir.mkdir(parents=True, exist_ok=True)

    # Run pandoc
    run_command(['pandoc', '--defaults', 'pandoc/pandoc-latex.yml'], stream_output=True)

    print(f"\n✅ LaTeX created: {CONFIG['latex_dir']}/{CONFIG['project_name']}.tex")


def build_pdf():
    """Generate PDF from LaTeX (requires LaTeX installation)."""
    print("\n📕 Building PDF...\n")

    # First generate LaTeX
    build_latex()

    # Compile to PDF
    print("\n🔨 Compiling LaTeX to PDF...\n")
    latex_dir = Path(CONFIG['latex_dir'])
    build_dir = latex_dir / 'build'

    try:
        # Run latexmk
        run_command([
            'latexmk',
            '-interaction=nonstopmode',
            '-file-line-error',
            '-xelatex',
            f'-outdir={build_dir}',
            f'{CONFIG["project_name"]}.tex'
        ], cwd=latex_dir, stream_output=True)

        # Move PDF to output directory
        pdf_file = build_dir / f'{CONFIG["project_name"]}.pdf'
        output_pdf = Path(CONFIG['output_dir']) / f'{CONFIG["project_name"]}.pdf'
        shutil.move(str(pdf_file), str(output_pdf))

        # Clean up build directory
        shutil.rmtree(build_dir)

        print(f"\n✅ PDF created: {output_pdf}")

    except subprocess.CalledProcessError as e:
        print(f"\n❌ LaTeX compilation failed. Check the logs above.")
        sys.exit(1)


def clean():
    """Remove all generated files."""
    print("\n🧹 Cleaning generated files...\n")

    output_dir = Path(CONFIG['output_dir'])
    if output_dir.exists():
        shutil.rmtree(output_dir)
        print(f"Removed: {output_dir}")

    print("\n✅ Clean complete.")


def distclean():
    """Deep clean (including Pandoc cache)."""
    print("\n🧹 Deep cleaning...\n")

    # Regular clean
    clean()

    # Remove Pandoc cache
    cache_dir = Path('.pandoc-cache')
    if cache_dir.exists():
        shutil.rmtree(cache_dir)
        print(f"Removed: {cache_dir}")

    print("\n✅ Deep clean complete.")


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
        choices=['docx', 'latex', 'pdf', 'clean', 'distclean', 'help'],
        help='Build target (default: docx)'
    )

    args = parser.parse_args()

    # Dispatch to target function
    targets = {
        'docx': build_docx,
        'latex': build_latex,
        'pdf': build_pdf,
        'clean': clean,
        'distclean': distclean,
        'help': show_help,
    }

    try:
        targets[args.target]()
    except KeyboardInterrupt:
        print("\n\n⚠️  Build interrupted by user.")
        sys.exit(1)
    except Exception as e:
        print(f"\n❌ Error: {e}")
        sys.exit(1)


if __name__ == '__main__':
    main()
