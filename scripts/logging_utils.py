"""Small shared logging gate for repository scripts."""

from __future__ import annotations

import os
import sys
from typing import TextIO


LOG_LEVELS = {
    "DEBUG": 10,
    "INFO": 20,
    "SUCCESS": 20,
    "WARNING": 30,
    "WARN": 30,
    "ERROR": 40,
}
DEFAULT_LOG_LEVEL = "WARNING"
LOG_LEVEL_ENV = "PANDOC_TEMPLATE_LOG_LEVEL"


def normalize_log_level(level: str | None) -> int:
    """Return a numeric log level, falling back to WARNING for invalid values."""
    if level is None:
        return LOG_LEVELS[DEFAULT_LOG_LEVEL]
    return LOG_LEVELS.get(level.strip().upper(), LOG_LEVELS[DEFAULT_LOG_LEVEL])


def current_log_level() -> int:
    """Return the active minimum log level from the environment."""
    return normalize_log_level(os.environ.get(LOG_LEVEL_ENV))


def should_log(level: str) -> bool:
    """Return True when a message should be emitted at the active log level."""
    return normalize_log_level(level) >= current_log_level()


def log_message(level: str, message: str, *, stream: TextIO | None = None) -> None:
    """Print a message only when it meets the configured log threshold."""
    if not should_log(level):
        return
    print(message, file=stream or sys.stdout)


def log_debug(message: str) -> None:
    """Print a debug message when DEBUG logging is enabled."""
    log_message("DEBUG", message)


def log_info(message: str) -> None:
    """Print an informational message when INFO logging is enabled."""
    log_message("INFO", message)


def log_success(message: str) -> None:
    """Print a success/progress message when INFO logging is enabled."""
    log_message("SUCCESS", message)


def log_warning(message: str) -> None:
    """Print a warning message; warnings are visible at the default level."""
    log_message("WARNING", message, stream=sys.stderr)


def log_error(message: str) -> None:
    """Print an error message; errors are visible at the default level."""
    log_message("ERROR", message, stream=sys.stderr)
