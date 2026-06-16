"""Shared helpers for reading and merging Pandoc YAML metadata."""

from pathlib import Path
import re
from typing import Any

import yaml


class MissingYamlFrontMatterError(ValueError):
    """Raised when a markdown file does not start with a YAML front matter block."""


def parse_yaml_header(md_path: str | Path) -> dict[str, Any]:
    """Parse YAML front matter from a markdown manuscript file."""
    path = Path(md_path)
    content = path.read_text(encoding='utf-8')
    match = re.match(r'^---\s*\n(.*?)\n---', content, re.DOTALL)
    if not match:
        raise MissingYamlFrontMatterError(f"No YAML front matter found in markdown file: {path}")

    metadata = yaml.safe_load(match.group(1)) or {}
    if not isinstance(metadata, dict):
        raise ValueError("YAML front matter must be a mapping")
    return metadata


def parse_yaml_file(yaml_path: str | Path) -> dict[str, Any]:
    """Parse a standalone YAML metadata file."""
    metadata = yaml.safe_load(Path(yaml_path).read_text(encoding='utf-8')) or {}
    if not isinstance(metadata, dict):
        raise ValueError(f"YAML metadata file must be a mapping: {yaml_path}")
    return metadata


def load_metadata_files(metadata_files: list[str | Path] | None = None) -> dict[str, Any]:
    """Load standalone metadata files and merge them in the given order."""
    metadata: dict[str, Any] = {}
    for metadata_file in metadata_files or []:
        path = Path(metadata_file)
        if not path.exists():
            raise FileNotFoundError(f"Metadata file not found: {path}")
        metadata = merge_metadata(metadata, parse_yaml_file(path))
    return metadata


def merge_metadata(base: dict[str, Any], override: dict[str, Any]) -> dict[str, Any]:
    """Recursively merge metadata so later sources override earlier defaults."""
    merged = dict(base)
    for key, value in override.items():
        existing = merged.get(key)
        if isinstance(existing, dict) and isinstance(value, dict):
            merged[key] = merge_metadata(existing, value)
        else:
            merged[key] = value
    return merged


def load_merged_metadata(
    md_path: str | Path,
    metadata_files: list[str | Path] | None = None,
    *,
    allow_missing_header: bool = False,
) -> dict[str, Any]:
    """Load metadata files first, then overlay manuscript YAML metadata."""
    metadata, _ = load_merged_metadata_with_status(
        md_path,
        metadata_files,
        allow_missing_header=allow_missing_header,
    )
    return metadata


def load_merged_metadata_with_status(
    md_path: str | Path,
    metadata_files: list[str | Path] | None = None,
    *,
    allow_missing_header: bool = False,
) -> tuple[dict[str, Any], bool]:
    """Load merged metadata and report whether markdown YAML was present.

    Reviewer replies and a few post-processing paths can validly omit YAML
    front matter, so callers can opt into metadata-file-only fallback without
    repeating string-based exception checks.
    """
    metadata = load_metadata_files(metadata_files)
    try:
        return merge_metadata(metadata, parse_yaml_header(md_path)), True
    except MissingYamlFrontMatterError:
        if allow_missing_header:
            return metadata, False
        raise
