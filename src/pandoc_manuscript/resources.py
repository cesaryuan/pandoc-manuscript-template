"""Locate pmt package resources from a source tree or an installed wheel."""

from __future__ import annotations

from importlib import resources
from pathlib import Path
from typing import Iterable


PACKAGE_NAME = "pandoc_manuscript"


def package_root() -> Path:
    """Return the installed package directory as a filesystem path."""
    return Path(str(resources.files(PACKAGE_NAME)))


def package_resource_path(relative_path: str | Path) -> Path:
    """Return a filesystem path for a resource bundled inside the pmt package."""
    return package_root() / Path(relative_path)


def source_tree_root() -> Path | None:
    """Return the pmt repository root when running from this source checkout."""
    root = Path(__file__).resolve().parents[2]
    # Detect the tool repository itself, not a generated manuscript project.
    if (root / "pyproject.toml").is_file() and (root / "src" / PACKAGE_NAME).is_dir():
        return root
    return None


def template_root() -> Path:
    """Return a filesystem root containing project template resources."""
    source_root = source_tree_root()
    if source_root is not None:
        return source_root

    packaged_root = resources.files(PACKAGE_NAME).joinpath("_template")
    if (packaged_root / "pandoc").is_dir():
        return Path(str(packaged_root))
    raise RuntimeError("Could not locate packaged Pandoc manuscript template resources.")


def iter_project_template_entries() -> Iterable[str]:
    """Yield top-level entries copied by `pmt init` into a new paper project."""
    yield from (
        "manuscript.md",
        "style.yml",
        ".gitignore",
        "images",
        "examples",
        "pandoc/csl",
    )
