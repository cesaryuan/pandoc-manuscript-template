"""HTML manuscript build entry point and HTML-specific post-processing."""

from __future__ import annotations

import os
from pathlib import Path

from .postprocess import postprocess_html


# HTML keeps display equations as native MathML blocks instead of DOCX-style
# equation layout tables and inline equation-number workarounds.
HTML_EQUATION_METADATA = {
    "equationNumberTeX": "\\\\tag",
    "eqnIndexTemplate": "$$i$$",
    "eqnBlockInlineMath": False,
    "tableEqns": False,
}


def build_html(
    *,
    start_server: bool = False,
    server_host: str = "127.0.0.1",
    server_port: int = 3030,
    server_command: str | None = None,
) -> None:
    """Generate HTML and optionally start a reusable Pandoc server for clients."""
    # Import the shared build primitives lazily to keep HTML implementation
    # details in this package without creating a module import cycle.
    from ..commands.build import (
        SETTINGS,
        ensure_output_parent,
        load_build_metadata,
        manuscript_output_file,
        prepare_pandoc_language,
        resource_path,
        csl_args,
        pandoc_filter_env,
        style_metadata_args,
        run_pandoc,
    )
    from ..runtime.logging import log_info, log_success
    from ..commands.pandoc_server import ensure_pandoc_server, write_pmt_server_config
    from ..commands.setup import pandoc_tools_env

    log_info("\n[HTML] Building HTML...\n")
    html_file = manuscript_output_file(SETTINGS.html_dir, "html")
    ensure_output_parent(html_file)
    effective, chinese_mode = prepare_pandoc_language(load_build_metadata())
    effective.pandoc_metadata.update(HTML_EQUATION_METADATA)
    if chinese_mode:
        log_info("[HTML] Chinese language metadata enabled")
    source_dir = Path(SETTINGS.manuscript_file).resolve().parent
    resource_path_option = os.pathsep.join((str(source_dir), str(Path.cwd())))
    if start_server:
        server_config = write_pmt_server_config(
            project_dir=Path.cwd(),
            pandoc_args=[
                "--defaults",
                str(resource_path("pandoc/pandoc-html.yml")),
                *style_metadata_args(effective),
                *csl_args(effective.pandoc_metadata),
                "--resource-path",
                resource_path_option,
            ],
            pandoc_metadata=effective.pandoc_metadata,
        )
        ensure_pandoc_server(
            host=server_host,
            port=server_port,
            command=server_command,
            config_path=server_config,
            environment=pandoc_tools_env(pandoc_filter_env(effective.pmt_settings)),
        )
    run_pandoc(
        resource_path("pandoc/pandoc-html.yml"),
        html_file,
        effective,
        extra_args=["--resource-path", resource_path_option],
    )
    postprocess_html(
        html_file,
        pandoc_metadata=effective.pandoc_metadata,
    )
    log_success(f"\n[OK] HTML created: {html_file}")
