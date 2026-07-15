"""CLI entry point for the Pandoc manuscript template package."""

from __future__ import annotations

import sys

from pydantic import Field, PrivateAttr
from pydantic_settings import BaseSettings, CliApp, CliSubCommand, SettingsConfigDict, get_subcommand

from . import __version__
from .commands.build import BuildCommandSettings
from .commands.clean import CleanSettings, DistcleanSettings
from .commands.common import log
from .commands.doctor import DoctorSettings
from .commands.init import InitSettings
from .commands.build_reply import BuildReplySettings
from .commands.setup import SetupSettings
from .runtime.logging import verbose_logging


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
    setup: CliSubCommand[SetupSettings | None]
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
        with verbose_logging(command.verbose):
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


__all__ = [
    "BuildCommandSettings",
    "BuildReplySettings",
    "CleanSettings",
    "DistcleanSettings",
    "DoctorSettings",
    "InitSettings",
    "PmtCli",
    "SetupSettings",
    "main",
]


if __name__ == "__main__":
    raise SystemExit(main())
