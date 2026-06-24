"""`pmt clean` and `pmt distclean` command implementations."""

from __future__ import annotations

from pathlib import Path
from typing import ClassVar, Literal

from pydantic import AliasChoices, Field
from pydantic_settings import BaseSettings, SettingsConfigDict

from .build import DEFAULT_OUTPUT_DIR, run_build_command
from .common import project_directory


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
