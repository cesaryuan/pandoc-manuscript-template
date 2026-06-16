"""CLI entry point for the Pandoc manuscript template package."""

from __future__ import annotations

import argparse
import importlib.util
import os
import shutil
import subprocess
import sys
from contextlib import contextmanager
from pathlib import Path
from types import ModuleType
from typing import Iterator

from . import __version__
from .resources import iter_project_template_entries, template_root


IGNORE_NAMES = {
    ".git",
    ".pandoc-cache",
    ".venv",
    "__pycache__",
    "output",
    "tmp",
    "target",
}


def log(message: str) -> None:
    """Print a pmt CLI status line."""
    print(message)


def copy_template_entry(source: Path, destination: Path, *, overwrite: bool) -> None:
    """Copy one template file or directory while avoiding generated artifacts."""
    if destination.exists():
        if not overwrite:
            raise FileExistsError(f"Target already exists: {destination}")
        if destination.is_dir():
            shutil.rmtree(destination)
        else:
            destination.unlink()

    if source.is_dir():
        shutil.copytree(source, destination, ignore=ignore_generated_artifacts)
    else:
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(source, destination)


def ignore_generated_artifacts(directory: str, names: list[str]) -> set[str]:
    """Return generated/cache names that should not be copied into new projects."""
    return {name for name in names if name in IGNORE_NAMES or name.endswith(".pyc")}


def init_project(args: argparse.Namespace) -> int:
    """Create a new manuscript project from the packaged template files."""
    root = template_root()
    target = Path(args.directory).resolve()
    if target.exists() and any(target.iterdir()) and not args.force:
        raise RuntimeError(f"Target directory is not empty. Use --force to overwrite entries: {target}")

    target.mkdir(parents=True, exist_ok=True)
    for relative_name in iter_project_template_entries():
        source = root / relative_name
        if not source.exists():
            continue
        copy_template_entry(source, target / relative_name, overwrite=args.force)

    # Empty directories are not preserved in wheels/sdists, but manuscripts
    # conventionally keep figures under images/ from the beginning.
    (target / "images").mkdir(exist_ok=True)

    log(f"[OK] Created manuscript project: {target}")
    log("Next: cd into the project and run `pmt doctor`, then `pmt build docx`.")
    return 0


@contextmanager
def build_environment(project_dir: Path, resource_root: Path) -> Iterator[None]:
    """Temporarily run the existing build script as if it were package-owned."""
    previous_cwd = Path.cwd()
    previous_resource_root = os.environ.get("PMT_RESOURCE_ROOT")
    scripts_dir = str(resource_root / "scripts")
    inserted_path = False

    if scripts_dir not in sys.path:
        sys.path.insert(0, scripts_dir)
        inserted_path = True
    os.environ["PMT_RESOURCE_ROOT"] = str(resource_root)
    os.chdir(project_dir)
    try:
        yield
    finally:
        os.chdir(previous_cwd)
        if previous_resource_root is None:
            os.environ.pop("PMT_RESOURCE_ROOT", None)
        else:
            os.environ["PMT_RESOURCE_ROOT"] = previous_resource_root
        if inserted_path:
            try:
                sys.path.remove(scripts_dir)
            except ValueError:
                pass


def load_build_module(resource_root: Path) -> ModuleType:
    """Load the repository build script from the active pmt resource root."""
    build_script = resource_root / "scripts" / "build.py"
    if not build_script.exists():
        raise FileNotFoundError(f"Build script not found: {build_script}")

    module_name = "_pmt_build"
    spec = importlib.util.spec_from_file_location(module_name, build_script)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"Could not load build script: {build_script}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def run_build(args: argparse.Namespace) -> int:
    """Run the existing manuscript build pipeline through the pmt CLI."""
    resource_root = template_root()
    forwarded_args = list(getattr(args, "build_args", []))
    if getattr(args, "build_help", False):
        forwarded_args.append("--help")
    shortcut_target = getattr(args, "target", None)
    if shortcut_target:
        forwarded_args = [shortcut_target, *forwarded_args]

    project_dir, forwarded_args = extract_project_dir(
        forwarded_args,
        default_project_dir=getattr(args, "project_dir", None) or ".",
    )
    if not project_dir.exists():
        raise FileNotFoundError(f"Project directory not found: {project_dir}")

    with build_environment(project_dir, resource_root):
        module = load_build_module(resource_root)
        previous_argv = sys.argv[:]
        sys.argv = ["pmt build", *forwarded_args]
        try:
            module.main()
        except SystemExit as exc:
            return int(exc.code or 0)
        finally:
            sys.argv = previous_argv
    return 0


def extract_project_dir(build_args: list[str], default_project_dir: str = ".") -> tuple[Path, list[str]]:
    """Remove pmt's wrapper-only --project-dir option from build arguments."""
    project_dir = default_project_dir
    forwarded_args: list[str] = []
    index = 0
    while index < len(build_args):
        arg = build_args[index]
        if arg == "--project-dir":
            if index + 1 >= len(build_args):
                raise ValueError("--project-dir requires a directory")
            project_dir = build_args[index + 1]
            index += 2
            continue
        if arg.startswith("--project-dir="):
            project_dir = arg.split("=", 1)[1]
            index += 1
            continue
        forwarded_args.append(arg)
        index += 1
    return Path(project_dir).resolve(), forwarded_args


def command_status(command: list[str]) -> tuple[bool, str]:
    """Run a short diagnostic command and return whether it succeeded."""
    executable = shutil.which(command[0])
    if executable is None:
        return False, "not found on PATH"
    try:
        result = subprocess.run(
            [executable, *command[1:]],
            capture_output=True,
            text=True,
            check=False,
        )
    except OSError as exc:
        return False, str(exc)
    first_line = (result.stdout or result.stderr).splitlines()
    detail = first_line[0] if first_line else f"exit code {result.returncode}"
    return result.returncode == 0, detail


def import_status(module_name: str) -> tuple[bool, str]:
    """Return whether a Python dependency can be imported."""
    try:
        __import__(module_name)
    except Exception as exc:
        return False, str(exc)
    return True, "available"


def doctor(args: argparse.Namespace) -> int:
    """Check the local environment and current manuscript project."""
    project_dir = Path(args.project_dir).resolve()
    root = template_root()
    checks: list[tuple[str, bool, str]] = []

    for command in (["pandoc", "--version"], ["pandoc-crossref", "--version"]):
        ok, detail = command_status(command)
        checks.append((" ".join(command), ok, detail))

    for module_name in ("docx", "yaml", "lxml", "panflute", "fitz"):
        ok, detail = import_status(module_name)
        checks.append((f"python import {module_name}", ok, detail))

    checks.extend(
        [
            ("pmt resource scripts/build.py", (root / "scripts" / "build.py").exists(), str(root)),
            ("project directory", project_dir.exists(), str(project_dir)),
            ("project manuscript.md", (project_dir / "manuscript.md").exists(), str(project_dir / "manuscript.md")),
            ("project style.yml", (project_dir / "style.yml").exists(), str(project_dir / "style.yml")),
        ]
    )

    has_error = False
    for label, ok, detail in checks:
        prefix = "[OK]" if ok else "[ERROR]"
        log(f"{prefix} {label}: {detail}")
        has_error = has_error or not ok
    return 1 if has_error else 0


def build_parser() -> argparse.ArgumentParser:
    """Build the top-level pmt argument parser."""
    parser = argparse.ArgumentParser(
        prog="pmt",
        description="Pandoc Manuscript Template CLI",
    )
    parser.add_argument("--version", action="version", version=f"pmt {__version__}")
    subparsers = parser.add_subparsers(dest="command")

    init_parser = subparsers.add_parser("init", help="Create a new manuscript project")
    init_parser.add_argument("directory", help="Directory to create or populate")
    init_parser.add_argument("--force", action="store_true", help="Overwrite existing template entries")
    init_parser.set_defaults(func=init_project)

    build_parser_ = subparsers.add_parser(
        "build",
        add_help=False,
        help="Build DOCX, reviewer reply, LaTeX, or JSON output",
    )
    build_parser_.add_argument("-h", "--help", dest="build_help", action="store_true", help="Show build help")
    build_parser_.add_argument("--project-dir", help="Project directory to build")
    build_parser_.add_argument("build_args", nargs=argparse.REMAINDER, help="Arguments forwarded to the build script")
    build_parser_.set_defaults(func=run_build)

    for target in ("docx", "reply", "latex", "json", "clean", "distclean"):
        target_parser = subparsers.add_parser(
            target,
            add_help=False,
            help=f"Shortcut for `pmt build {target}`",
        )
        target_parser.add_argument("-h", "--help", dest="build_help", action="store_true", help="Show build help")
        target_parser.add_argument("--project-dir", help="Project directory to build")
        target_parser.add_argument("build_args", nargs=argparse.REMAINDER, help="Arguments forwarded to the build script")
        target_parser.set_defaults(func=run_build, target=target)

    doctor_parser = subparsers.add_parser("doctor", help="Check environment and project readiness")
    doctor_parser.add_argument("--project-dir", default=".", help="Project directory to check")
    doctor_parser.set_defaults(func=doctor)
    return parser


def main(argv: list[str] | None = None) -> int:
    """Run the pmt command-line interface."""
    parser = build_parser()
    args = parser.parse_args(argv)
    if not hasattr(args, "func"):
        parser.print_help()
        return 0

    try:
        return int(args.func(args))
    except KeyboardInterrupt:
        print("\n[WARN] Interrupted by user.", file=sys.stderr)
        return 1
    except Exception as exc:
        print(f"[ERROR] {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
