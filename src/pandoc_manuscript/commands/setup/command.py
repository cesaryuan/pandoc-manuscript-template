"""`pmt setup` command implementation."""

from __future__ import annotations

from pathlib import Path

from pydantic import Field
from pydantic_settings import SettingsConfigDict

from ..common import VerboseCommandSettings, log, project_directory
from .pandoc_tools import setup_pandoc_tools


class SetupSettings(VerboseCommandSettings):
    """Settings for `pmt setup`."""

    model_config = SettingsConfigDict(cli_kebab_case=True, cli_implicit_flags=True)

    project_dir: Path = Field(default=Path("."), description="Project directory that receives .pmt/tools.")
    force: bool = Field(default=False, description="Redownload and reinstall managed Pandoc tools.")

    def run(self) -> int:
        """Download project-local Pandoc tools into .pmt/tools."""
        project_dir = self.project_dir.resolve()
        if not project_dir.exists():
            raise FileNotFoundError(f"Project directory not found: {project_dir}")

        with project_directory(project_dir):
            pandoc, crossref = setup_pandoc_tools(force=self.force)

        log(f"[OK] pandoc: {pandoc.executable} [{pandoc.source}]")
        log(f"[OK] pandoc-crossref: {crossref.executable} [{crossref.source}]")
        return 0
