"""Shared Pandoc SVG filter helpers for manuscript and reply DOCX builds."""

from __future__ import annotations

import os
import stat
import sys
from pathlib import Path
from typing import Any

from . import runtime_cache_version
from .paths import PMT_FILTER_WORK_DIR, PMT_SVG_EMBED_CACHE_DIR, PMT_SVG_PNG_CACHE_DIR
from .resources import template_root

SVG_TO_PNG_DPI_KEYS = ("docxSvgToPngDpi", "docx-svg-to-png-dpi")
SVG_TO_PNG_SCALE_KEYS = ("docxSvgToPngScale", "docx-svg-to-png-scale")
SVG_TO_PNG_WIDTH_KEYS = ("docxSvgToPngWidth", "docx-svg-to-png-width")


def to_pandoc_path(path: Path) -> str:
    """Return a Pandoc-friendly path string."""
    return path.as_posix()


def metadata_bool(value: Any) -> bool:
    """Normalize YAML feature flags such as true, yes, or on."""
    if isinstance(value, bool):
        return value
    if isinstance(value, str):
        return value.strip().lower() in {"1", "true", "yes", "on"}
    if isinstance(value, (int, float)):
        return bool(value)
    return False


def metadata_first(metadata: dict[str, Any], keys: tuple[str, ...], default: Any = None) -> Any:
    """Return the first configured metadata value from a list of aliases."""
    for key in keys:
        if key in metadata:
            return metadata[key]
    return default


def metadata_float(metadata: dict[str, Any], keys: tuple[str, ...], default: float) -> float:
    """Read a positive float metadata option with a clear validation error."""
    value = metadata_first(metadata, keys, default)
    try:
        parsed = float(value)
    except (TypeError, ValueError) as exc:
        raise ValueError(f"{keys[0]} must be a number, got: {value!r}") from exc
    if parsed <= 0:
        raise ValueError(f"{keys[0]} must be positive, got: {value!r}")
    return parsed


def metadata_int(metadata: dict[str, Any], keys: tuple[str, ...]) -> int:
    """Read a positive integer metadata option with a clear validation error."""
    value = metadata_first(metadata, keys)
    if isinstance(value, bool):
        raise ValueError(f"{keys[0]} must be a positive integer, got: {value!r}")
    try:
        parsed = int(value)
    except (TypeError, ValueError) as exc:
        raise ValueError(f"{keys[0]} must be a positive integer, got: {value!r}") from exc
    if str(parsed) != str(value).strip():
        raise ValueError(f"{keys[0]} must be a positive integer, got: {value!r}")
    if parsed <= 0:
        raise ValueError(f"{keys[0]} must be positive, got: {value!r}")
    return parsed


def metadata_has_any(metadata: dict[str, Any], keys: tuple[str, ...]) -> bool:
    """Return True when any alias is explicitly present in metadata."""
    return any(key in metadata for key in keys)


def configured_svg_to_png_controls(metadata: dict[str, Any]) -> list[str]:
    """Return explicitly enabled SVG rasterization size controls."""
    controls: list[str] = []
    if metadata_has_any(metadata, SVG_TO_PNG_WIDTH_KEYS):
        controls.append("docxSvgToPngWidth")
    if metadata_has_any(metadata, SVG_TO_PNG_SCALE_KEYS):
        controls.append("docxSvgToPngScale")
    if metadata_has_any(metadata, SVG_TO_PNG_DPI_KEYS):
        controls.append("docxSvgToPngDpi")
    return controls


def validate_svg_to_png_controls(metadata: dict[str, Any]) -> None:
    """Reject ambiguous SVG rasterization controls before Pandoc runs."""
    controls = configured_svg_to_png_controls(metadata)
    if len(controls) > 1:
        options = ", ".join(controls)
        raise ValueError(f"Only one of docxSvgToPngWidth, docxSvgToPngScale, docxSvgToPngDpi can be set; got: {options}")


def should_embed_docx_svg_images(metadata: dict[str, Any]) -> bool:
    """Return True when DOCX builds should inline child images inside SVG files."""
    return metadata_bool(
        metadata_first(
            metadata,
            (
                "docxEmbedSvgImages",
                "docx-embed-svg-images",
                "embedSvgImages",
                "embed-svg-images",
            ),
            True,
        )
    )


def should_convert_docx_svg_to_png(metadata: dict[str, Any]) -> bool:
    """Return True when DOCX builds should rasterize all SVG images."""
    return metadata_bool(
        metadata_first(
            metadata,
            (
                "docxConvertSvgToPng",
                "docx-convert-svg-to-png",
                "convertSvgToPng",
                "convert-svg-to-png",
            ),
            False,
        )
    )


def python_filter_wrapper(filter_path: Path, name: str) -> Path:
    """Create a Pandoc filter wrapper that runs with pmt's Python interpreter.

    Pandoc executes JSON filters as external programs. Installed package data
    filters may otherwise run under a system Python that cannot import pmt's
    dependencies, which caused the SVG filter to miss panflute in uv tool installs.
    """
    wrapper_dir = PMT_FILTER_WORK_DIR
    wrapper_dir.mkdir(parents=True, exist_ok=True)
    filter_path = filter_path.resolve()

    if os.name == "nt":
        wrapper_path = wrapper_dir / f"{name}.cmd"
        wrapper_path.write_text(
            f'@echo off\r\n"{sys.executable}" "{filter_path}" %*\r\n',
            encoding="utf-8",
            newline="",
        )
        return wrapper_path

    wrapper_path = wrapper_dir / name
    wrapper_path.write_text(
        f"#!{sys.executable}\n"
        "import runpy\n"
        f"runpy.run_path({str(filter_path)!r}, run_name=\"__main__\")\n",
        encoding="utf-8",
        newline="\n",
    )
    # Shared workspaces can reuse a wrapper owned by another user when it is
    # already executable, so only touch the mode when the execute bit is missing.
    if not os.access(wrapper_path, os.X_OK):
        wrapper_path.chmod(wrapper_path.stat().st_mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)
    return wrapper_path


def svg_filter_path(name: str) -> Path:
    """Return the repository or package path for one bundled SVG filter."""
    filter_path = template_root() / "pandoc" / "filters" / name
    if not filter_path.exists():
        raise FileNotFoundError(f"SVG Pandoc filter not found: {filter_path}")
    return filter_path


def svg_embed_images_filter_args() -> list[str]:
    """Return Pandoc args for the self-contained SVG image filter."""
    return [
        "--filter",
        to_pandoc_path(python_filter_wrapper(svg_filter_path("svg_embed_images.py"), "svg_embed_images_filter")),
    ]


def svg_to_png_filter_args() -> list[str]:
    """Return Pandoc args for the SVG-to-PNG image filter."""
    return [
        "--filter",
        to_pandoc_path(python_filter_wrapper(svg_filter_path("svg_to_png.py"), "svg_to_png_filter")),
    ]


def unique_resolved_dirs(candidates: list[Path]) -> list[Path]:
    """Return unique resolved lookup roots while preserving order."""
    unique: list[Path] = []
    for candidate in candidates:
        resolved = candidate.resolve()
        if resolved not in unique:
            unique.append(resolved)
    return unique


def svg_embed_images_filter_env(
    base_dirs: list[Path],
    metadata: dict[str, Any],
    embed_images: bool | None = None,
) -> dict[str, str]:
    """Return environment settings consumed by the SVG child-image embedding filter."""
    if embed_images is None:
        embed_images = should_embed_docx_svg_images(metadata)
    return {
        "PMT_SVG_EMBED_DIR": str(PMT_SVG_EMBED_CACHE_DIR.resolve()),
        "PMT_SVG_EMBED_BASE_DIRS": os.pathsep.join(str(path) for path in unique_resolved_dirs(base_dirs)),
        "PMT_SVG_EMBED_PMT_VERSION": runtime_cache_version(),
        "PMT_SVG_EMBED_IMAGES": "true" if embed_images else "false",
    }


def svg_to_png_filter_env(
    base_dirs: list[Path],
    metadata: dict[str, Any],
    convert_all: bool | None = None,
) -> dict[str, str]:
    """Return environment settings consumed by the SVG-to-PNG filter."""
    if convert_all is None:
        convert_all = should_convert_docx_svg_to_png(metadata)
    validate_svg_to_png_controls(metadata)
    env = {
        "PMT_SVG_TO_PNG_DIR": str(PMT_SVG_PNG_CACHE_DIR.resolve()),
        "PMT_SVG_TO_PNG_BASE_DIRS": os.pathsep.join(str(path) for path in unique_resolved_dirs(base_dirs)),
        "PMT_SVG_TO_PNG_DPI": str(
            metadata_float(metadata, SVG_TO_PNG_DPI_KEYS, 300)
        ),
        "PMT_SVG_TO_PNG_SCALE": str(
            metadata_float(metadata, SVG_TO_PNG_SCALE_KEYS, 1)
        ),
        "PMT_SVG_TO_PNG_PMT_VERSION": runtime_cache_version(),
        "PMT_SVG_TO_PNG_CONVERT_ALL": "true" if convert_all else "false",
    }
    if metadata_has_any(metadata, SVG_TO_PNG_WIDTH_KEYS):
        env["PMT_SVG_TO_PNG_WIDTH"] = str(metadata_int(metadata, SVG_TO_PNG_WIDTH_KEYS))
    return env
