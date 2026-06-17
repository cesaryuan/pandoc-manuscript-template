"""CLI entry point for the Pandoc manuscript template package."""

from __future__ import annotations

import os
import shutil
import subprocess
import sys
from contextlib import contextmanager
from pathlib import Path
from typing import ClassVar, Iterator, Literal

from pydantic import AliasChoices, Field, PrivateAttr
from pydantic_settings import (
    BaseSettings,
    CliApp,
    CliPositionalArg,
    CliSubCommand,
    SettingsConfigDict,
    get_subcommand,
)

from . import __version__
from .build import BuildCliSettings, run_build_command
from .resources import iter_project_template_entries, package_resource_path, template_root


BuildTarget = Literal["docx", "reply", "latex", "json", "clean", "distclean", "help"]

BUILD_CLI_CONFIG = SettingsConfigDict(
    cli_kebab_case=True,
    cli_implicit_flags=True,
    cli_shortcuts={
        "manuscript_option": ["-m", "--manuscript"],
        "output_dir": ["-o", "--output-dir"],
    },
)

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


class InitSettings(BaseSettings):
    """Settings for `pmt init`."""

    model_config = SettingsConfigDict(cli_kebab_case=True, cli_implicit_flags=True)

    directory: CliPositionalArg[str]
    force: bool = False

    def run(self) -> int:
        """Create a new manuscript project from the packaged template files."""
        root = template_root()
        target = Path(self.directory).resolve()
        if target.exists() and any(target.iterdir()) and not self.force:
            raise RuntimeError(f"Target directory is not empty. Use --force to overwrite entries: {target}")

        target.mkdir(parents=True, exist_ok=True)
        for relative_name in iter_project_template_entries():
            source = root / relative_name
            if not source.exists():
                continue
            copy_template_entry(source, target / relative_name, overwrite=self.force)

        # Empty directories are not preserved in wheels/sdists, but manuscripts
        # conventionally keep figures under images/ from the beginning.
        (target / "images").mkdir(exist_ok=True)

        log(f"[OK] Created manuscript project: {target}")
        log("Next: cd into the project and run `pmt doctor`, then `pmt build docx`.")
        return 0


@contextmanager
def project_directory(project_dir: Path) -> Iterator[None]:
    """Temporarily run build commands from the selected manuscript project."""
    previous_cwd = Path.cwd()
    os.chdir(project_dir)
    try:
        yield
    finally:
        os.chdir(previous_cwd)


class BuildCommandSettings(BaseSettings):
    """Settings for `pmt build`."""

    model_config = BUILD_CLI_CONFIG

    target: CliPositionalArg[BuildTarget] = "docx"
    markdown: CliPositionalArg[str | None] = None
    manuscript_option: str | None = Field(default=None, validation_alias=AliasChoices("m", "manuscript"))
    output_dir: str | None = Field(default=None, validation_alias=AliasChoices("o", "output-dir"))
    project_dir: Path = Path(".")
    reply_manuscript: str | None = None
    manuscript_line_source: str | None = None
    from_format: str | None = None
    reference_doc: str | None = None
    output_file: str | None = None

    def run(self) -> int:
        """Run the selected build target."""
        project_dir = self.project_dir.resolve()
        if not project_dir.exists():
            raise FileNotFoundError(f"Project directory not found: {project_dir}")

        with project_directory(project_dir):
            build_settings = BuildCliSettings(
                target=self.target,
                markdown=self.markdown,
                manuscript_option=self.manuscript_option,
                output_dir=self.output_dir,
                reply_manuscript=self.reply_manuscript,
                manuscript_line_source=self.manuscript_line_source,
                from_format=self.from_format,
                reference_doc=self.reference_doc,
                output_file=self.output_file,
            )
            return int(run_build_command(build_settings))


class ReplySettings(BaseSettings):
    """Settings for the `pmt reply` shortcut."""

    model_config = BUILD_CLI_CONFIG
    target: ClassVar[Literal["reply"]] = "reply"

    markdown: CliPositionalArg[str | None] = None
    manuscript_option: str | None = Field(default=None, validation_alias=AliasChoices("m", "manuscript"))
    output_dir: str | None = Field(default=None, validation_alias=AliasChoices("o", "output-dir"))
    project_dir: Path = Path(".")
    reply_manuscript: str | None = None
    manuscript_line_source: str | None = None
    from_format: str | None = None
    reference_doc: str | None = None
    output_file: str | None = None

    def run(self) -> int:
        """Run the reply shortcut target."""
        project_dir = self.project_dir.resolve()
        if not project_dir.exists():
            raise FileNotFoundError(f"Project directory not found: {project_dir}")

        with project_directory(project_dir):
            build_settings = BuildCliSettings(
                target=self.target,
                markdown=self.markdown,
                manuscript_option=self.manuscript_option,
                output_dir=self.output_dir,
                reply_manuscript=self.reply_manuscript,
                manuscript_line_source=self.manuscript_line_source,
                from_format=self.from_format,
                reference_doc=self.reference_doc,
                output_file=self.output_file,
            )
            return int(run_build_command(build_settings))


class CleanSettings(BaseSettings):
    """Settings for `pmt clean` and `pmt distclean`."""

    model_config = SettingsConfigDict(
        cli_kebab_case=True,
        cli_implicit_flags=True,
        cli_shortcuts={"output_dir": ["-o", "--output-dir"]},
    )
    target: ClassVar[Literal["clean", "distclean"]] = "clean"

    output_dir: str | None = Field(default=None, validation_alias=AliasChoices("o", "output-dir"))
    project_dir: Path = Path(".")

    def run(self) -> int:
        """Run the clean target."""
        project_dir = self.project_dir.resolve()
        if not project_dir.exists():
            raise FileNotFoundError(f"Project directory not found: {project_dir}")

        with project_directory(project_dir):
            build_settings = BuildCliSettings(
                target=self.target,
                output_dir=self.output_dir,
            )
            return int(run_build_command(build_settings))


class DistcleanSettings(CleanSettings):
    """Settings for `pmt distclean`."""

    target: ClassVar[Literal["clean", "distclean"]] = "distclean"


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


class DoctorSettings(BaseSettings):
    """Settings for `pmt doctor`."""

    model_config = SettingsConfigDict(cli_kebab_case=True, cli_implicit_flags=True)

    project_dir: Path = Path(".")

    def run(self) -> int:
        """Check the local environment and current manuscript project."""
        project_dir = self.project_dir.resolve()
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
                ("pmt package build module", True, "pandoc_manuscript.build"),
                ("pmt pandoc defaults", (root / "pandoc" / "pandoc-docx.yml").exists(), str(root / "pandoc")),
                (
                    "pmt MathType marker filter",
                    package_resource_path("mathtype/mathtype_markers.lua").exists(),
                    str(package_resource_path("mathtype/mathtype_markers.lua")),
                ),
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


class PmtCli(BaseSettings):
    """Pandoc Manuscript Template CLI."""

    model_config = SettingsConfigDict(
        cli_prog_name="pmt",
        cli_kebab_case=True,
        cli_implicit_flags=True,
        extra="ignore",
    )

    version_flag: bool = Field(default=False, alias="version")
    init: CliSubCommand[InitSettings | None]
    build: CliSubCommand[BuildCommandSettings | None]
    reply: CliSubCommand[ReplySettings | None]
    clean: CliSubCommand[CleanSettings | None]
    distclean: CliSubCommand[DistcleanSettings | None]
    doctor: CliSubCommand[DoctorSettings | None]

    _exit_code: int = PrivateAttr(default=0)

    def cli_cmd(self) -> None:
        """Dispatch the selected pmt subcommand."""
        if self.version_flag:
            log(f"pmt {__version__}")
            self._exit_code = 0
            return
        command = get_subcommand(self, is_required=False)
        if command is None:
            log("Use `pmt --help` to see available commands.")
            self._exit_code = 1
            return
        self._exit_code = int(command.run())


def main(argv: list[str] | None = None) -> int:
    """Run the pmt command-line interface."""
    try:
        app = CliApp.run(PmtCli, cli_args=argv, cli_parse_args=True)
        return app._exit_code
    except KeyboardInterrupt:
        print("\n[WARN] Interrupted by user.", file=sys.stderr)
        return 1
    except Exception as exc:
        print(f"[ERROR] {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
