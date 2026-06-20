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
    CliSuppress,
    SettingsConfigDict,
    get_subcommand,
)

from . import __version__
from .build import DEFAULT_OUTPUT_DIR, run_build_command
from .paths import PMT_DIR
from .reply_build import BuildReplySettings
from .resources import iter_project_template_entries, package_resource_path, project_template_root, template_root


BuildTarget = Literal["docx", "latex", "json", "clean", "distclean"]

BUILD_CLI_CONFIG = SettingsConfigDict(
    cli_kebab_case=True,
    cli_implicit_flags=True,
    cli_hide_none_type=True,
    cli_parse_none_str="auto",
    cli_shortcuts={
        "manuscript_option": ["-m", "--manuscript"],
        "output_file": ["-o", "--output-file"],
    },
)

IGNORE_NAMES = {
    ".git",
    PMT_DIR.name,
    ".pandoc-cache",
    ".venv",
    "__pycache__",
    "output",
    "tmp",
    "target",
}

AGENTS_TEMPLATE_DESTINATION = "AGENTS.md"
AGENTS_TEMPLATE_START = "<!-- pmt template guidance: begin -->"
AGENTS_TEMPLATE_END = "<!-- pmt template guidance: end -->"


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


def merge_agents_template(source: Path, destination: Path) -> None:
    """Append the packaged AGENTS guidance to an existing AGENTS.md file.

    This keeps the user's local agent instructions intact while making the
    template instructions available in the generated manuscript project.
    """
    source_text = source.read_text(encoding="utf-8").strip()
    existing_text = destination.read_text(encoding="utf-8")
    if AGENTS_TEMPLATE_START in existing_text:
        log(f"[OK] AGENTS.md already contains the packaged guidance: {destination}")
        return

    merged_text = (
        existing_text.rstrip()
        + "\n\n"
        + AGENTS_TEMPLATE_START
        + "\n\n"
        + source_text
        + "\n\n"
        + AGENTS_TEMPLATE_END
        + "\n"
    )
    destination.write_text(merged_text, encoding="utf-8", newline="\n")
    log(f"[OK] Merged AGENTS.md: {destination}")


class InitSettings(BaseSettings):
    """Settings for `pmt init`."""

    model_config = SettingsConfigDict(cli_kebab_case=True, cli_implicit_flags=True)

    directory: CliPositionalArg[str]
    force: bool = Field(default=False, description="Overwrite existing template entries in the target project.")
    merge: bool = Field(
        default=False,
        description="Merge the packaged AGENTS.md guidance into an existing AGENTS.md file.",
    )

    def run(self) -> int:
        """Create a new manuscript project from the packaged template files."""
        root = project_template_root()
        target = Path(self.directory).resolve()

        if self.force and self.merge:
            raise RuntimeError("Use only one of --force or --merge for `pmt init`.")

        target.mkdir(parents=True, exist_ok=True)
        template_entries = list(iter_project_template_entries())
        existing_entries = {
            destination_name
            for _, destination_name in template_entries
            if (target / destination_name).exists()
        }
        blocking_entries = sorted(name for name in existing_entries if name != "AGENTS.md")
        if blocking_entries and not self.force and not self.merge:
            joined = ", ".join(blocking_entries)
            raise RuntimeError(
                f"Target already contains template files: {joined}. "
                f"Use --force to overwrite them: {target}"
            )
        if "AGENTS.md" in existing_entries and not self.force and not self.merge:
            log("[WARN] AGENTS.md already exists; use --merge to combine the packaged template notes automatically.")

        for source_name, destination_name in template_entries:
            source = root / source_name
            if not source.exists():
                continue
            destination = target / destination_name
            if destination_name == AGENTS_TEMPLATE_DESTINATION and destination.exists():
                if self.merge:
                    merge_agents_template(source, destination)
                elif not self.force:
                    continue
                else:
                    copy_template_entry(source, destination, overwrite=True)
                continue
            if destination.exists() and not self.force:
                continue
            copy_template_entry(source, destination, overwrite=self.force)

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

    target: CliPositionalArg[BuildTarget] = Field(default="docx", description="Build target.")
    markdown: CliPositionalArg[str | None] = Field(
        default=None,
        description="Input markdown file. Auto: manuscript.md.",
    )
    manuscript_option: str | None = Field(
        default=None,
        validation_alias=AliasChoices("m", "manuscript"),
        description="Input markdown file, equivalent to the positional MARKDOWN argument.",
    )

    output_file: str | None = Field(
        default=None,
        validation_alias=AliasChoices("o", "output-file"),
        description="Exact output file path for DOCX, LaTeX, and JSON builds.",
    )
    project_dir: Path = Field(default=Path("."), description="Manuscript project directory.")
    reference_doc: CliSuppress[str | None] = Field(
        default=None,
        description="Override the bundled DOCX reference document.",
    )

    def run(self) -> int:
        """Run the selected build target."""
        project_dir = self.project_dir.resolve()
        if not project_dir.exists():
            raise FileNotFoundError(f"Project directory not found: {project_dir}")

        with project_directory(project_dir):
            return int(
                run_build_command(
                    target=self.target,
                    markdown=self.markdown,
                    manuscript_option=self.manuscript_option,
                    output_file=self.output_file,
                    reference_doc=self.reference_doc,
                )
            )


class CleanSettings(BaseSettings):
    """Settings for `pmt clean` and `pmt distclean`."""

    model_config = SettingsConfigDict(
        cli_kebab_case=True,
        cli_implicit_flags=True,
        cli_hide_none_type=True,
        cli_parse_none_str="auto",
        cli_shortcuts={"output_dir": ["-o", "--output-dir"]},
    )
    target: ClassVar[Literal["clean", "distclean"]] = "clean"

    output_dir: str = Field(
        default=DEFAULT_OUTPUT_DIR,
        validation_alias=AliasChoices("o", "output-dir"),
        description="Base output directory.",
    )
    project_dir: Path = Field(default=Path("."), description="Manuscript project directory.")

    def run(self) -> int:
        """Run the clean target."""
        project_dir = self.project_dir.resolve()
        if not project_dir.exists():
            raise FileNotFoundError(f"Project directory not found: {project_dir}")

        with project_directory(project_dir):
            return int(
                run_build_command(
                    target=self.target,
                    output_dir=self.output_dir,
                )
            )


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

        for module_name in ("docx", "yaml", "lxml", "panflute", "resvg_py", "fitz"):
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
        cli_hide_none_type=True,
        cli_parse_none_str="auto",
        extra="ignore",
    )

    version_flag: bool = Field(default=False, alias="version")
    init: CliSubCommand[InitSettings | None]
    build: CliSubCommand[BuildCommandSettings | None]
    build_reply: CliSubCommand[BuildReplySettings | None]
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
