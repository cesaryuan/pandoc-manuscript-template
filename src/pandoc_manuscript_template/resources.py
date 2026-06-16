"""Locate template resources from either the source tree or an installed wheel."""

from __future__ import annotations

from importlib import resources
from pathlib import Path
from typing import Iterable


PACKAGE_NAME = "pandoc_manuscript_template"


def source_tree_root() -> Path | None:
    """Return the repository root when pmt is running from this source checkout."""
    root = Path(__file__).resolve().parents[2]
    if (root / "scripts" / "build.py").exists() and (root / "pandoc").is_dir():
        return root
    return None


def template_root() -> Path:
    """Return a filesystem root containing scripts, pandoc resources, and examples."""
    source_root = source_tree_root()
    if source_root is not None:
        return source_root

    packaged_root = resources.files(PACKAGE_NAME).joinpath("_template")
    if (packaged_root / "scripts" / "build.py").is_file():
        return Path(str(packaged_root))
    raise RuntimeError("Could not locate packaged Pandoc manuscript template resources.")


def iter_project_template_entries() -> Iterable[str]:
    """Yield top-level entries copied by `pmt init` into a new paper project."""
    yield from (
        "manuscript.md",
        "style.yml",
        "README.md",
        "Makefile",
        ".gitignore",
        "images",
        "examples",
        "pandoc",
        "scripts",
    )
