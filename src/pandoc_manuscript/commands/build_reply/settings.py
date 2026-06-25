"""Settings and CLI defaults for `pmt build-reply`."""

from __future__ import annotations

from pathlib import Path

from pydantic import AliasChoices, Field
from pydantic_settings import BaseSettings, CliPositionalArg, CliSuppress, SettingsConfigDict

from ..common import project_directory


DEFAULT_OUTPUT_DIR = "output"
DEFAULT_STYLE_FILE = "style.yml"
DEFAULT_REPLY_MANUSCRIPT_FILE = "manuscript.md"
DEFAULT_REPLY_LINE_SOURCE = DEFAULT_REPLY_MANUSCRIPT_FILE
DEFAULT_REPLY_FROM_FORMAT = "markdown"
DEFAULT_REPLY_OUTPUT_FILE = "output/docx/<reply-name>.docx"
BUILD_REPLY_CLI_CONFIG = SettingsConfigDict(
    cli_kebab_case=True,
    cli_implicit_flags=True,
    cli_hide_none_type=True,
    cli_parse_none_str="auto",
    cli_shortcuts={
        "output_file": ["-o", "--output-file"],
    },
)


class BuildReplySettings(BaseSettings):
    """Settings for `pmt build-reply`."""

    model_config = BUILD_REPLY_CLI_CONFIG

    markdown: CliPositionalArg[str] = Field(description="Reply markdown file.")
    project_dir: Path = Field(default=Path("."), description="Manuscript project directory.")
    reply_manuscript: str | None = Field(
        default=DEFAULT_REPLY_MANUSCRIPT_FILE,
        description="Manuscript source used to resolve reply references.",
    )
    manuscript_line_source: str | None = Field(
        default=DEFAULT_REPLY_LINE_SOURCE,
        description="Markdown, DOCX, or PDF source used for reply line placeholders.",
    )
    from_format: str | None = Field(
        default=DEFAULT_REPLY_FROM_FORMAT,
        description="Pandoc input format for reply reference probes.",
    )
    reference_doc: CliSuppress[str | None] = Field(
        default=None,
        description="Override the bundled DOCX reference document.",
    )
    output_file: str = Field(
        default=DEFAULT_REPLY_OUTPUT_FILE,
        validation_alias=AliasChoices("o", "output-file"),
        description="Explicit reply DOCX or TXT output path.",
    )

    def run(self) -> int:
        """Run the standalone reply build target."""
        from .command import run_build_reply_command

        project_dir = self.project_dir.resolve()
        if not project_dir.exists():
            raise FileNotFoundError(f"Project directory not found: {project_dir}")

        with project_directory(project_dir):
            return run_build_reply_command(
                markdown=self.markdown,
                reply_manuscript=self.reply_manuscript,
                manuscript_line_source=self.manuscript_line_source,
                from_format=self.from_format,
                reference_doc=self.reference_doc,
                output_file=self.output_file,
            )
