"""Command-line helpers for the Pandoc manuscript template."""

from __future__ import annotations

import importlib.metadata as importlib_metadata
import json

__version__ = "0.1.0"

_DIST_NAME = "pandoc-manuscript-template"


def _direct_url_commit_id() -> str | None:
    """Return the installed Git commit id recorded by PEP 610 metadata."""
    try:
        direct_url_text = importlib_metadata.distribution(_DIST_NAME).read_text("direct_url.json")
    except importlib_metadata.PackageNotFoundError:
        return None
    if not direct_url_text:
        return None
    try:
        direct_url = json.loads(direct_url_text)
    except json.JSONDecodeError:
        return None
    commit_id = direct_url.get("vcs_info", {}).get("commit_id")
    return commit_id if isinstance(commit_id, str) and commit_id else None


def runtime_cache_version() -> str:
    """Return a cache key version that changes for Git-installed revisions."""
    commit_id = _direct_url_commit_id()
    if commit_id:
        return f"{__version__}+git.{commit_id}"
    return __version__
