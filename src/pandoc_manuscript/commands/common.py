"""Shared helpers used by multiple pmt command modules."""

from __future__ import annotations

import os
from contextlib import contextmanager
from pathlib import Path
from typing import Iterator


def log(message: str) -> None:
    """Print a pmt CLI status line."""
    print(message)


@contextmanager
def project_directory(project_dir: Path) -> Iterator[None]:
    """Temporarily run command logic from the selected manuscript project."""
    previous_cwd = Path.cwd()
    os.chdir(project_dir)
    try:
        yield
    finally:
        os.chdir(previous_cwd)
