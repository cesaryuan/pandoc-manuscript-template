"""HTML manuscript build entry point and HTML-specific post-processing."""

from __future__ import annotations

import os
from pathlib import Path

from .postprocess import postprocess_html


def build_html() -> None:
    """Generate and post-process one standalone HTML manuscript file."""
    # Import the shared build primitives lazily to keep HTML implementation
    # details in this package without creating a module import cycle.
    from ..commands.build import (
        SETTINGS,
        ensure_output_parent,
        is_chinese_language,
        load_build_metadata,
        manuscript_output_file,
        resource_path,
        run_pandoc,
    )
    from ..runtime.logging import log_info, log_success

    log_info("\n[HTML] Building HTML...\n")
    html_file = manuscript_output_file(SETTINGS.html_dir, "html")
    ensure_output_parent(html_file)
    effective = load_build_metadata()
    source_dir = Path(SETTINGS.manuscript_file).resolve().parent
    resource_path_option = os.pathsep.join((str(source_dir), str(Path.cwd())))
    run_pandoc(
        resource_path("pandoc/pandoc-html.yml"),
        html_file,
        effective,
        extra_args=["--resource-path", resource_path_option],
    )
    postprocess_html(
        html_file,
        pandoc_metadata=effective.pandoc_metadata,
        chinese_mode=is_chinese_language(effective.pandoc_metadata.get("lang")),
    )
    log_success(f"\n[OK] HTML created: {html_file}")
