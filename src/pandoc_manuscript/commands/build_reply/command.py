"""Thin command entry points for `pmt build-reply`."""

from __future__ import annotations

from pathlib import Path

from ...runtime.logging import log_error, log_info, log_warning
from .output import (
    build_reply_docx,
    build_reply_txt,
    reply_output_format,
    reply_output_path,
    reply_reference_doc_path,
)
from .settings import (
    DEFAULT_REPLY_FROM_FORMAT,
    DEFAULT_REPLY_LINE_SOURCE,
    DEFAULT_REPLY_MANUSCRIPT_FILE,
    DEFAULT_STYLE_FILE,
)


def checked_markdown_path(markdown_path: str | Path) -> Path:
    """Return an existing reply markdown path, raising clear input errors."""
    path = Path(markdown_path)
    if not path.exists():
        raise FileNotFoundError(f"Reply markdown file not found: {path}")
    if not path.is_file():
        raise ValueError(f"Reply markdown path is not a file: {path}")
    return path


def run_build_reply_command(
    *,
    markdown: str,
    reply_manuscript: str | None = None,
    manuscript_line_source: str | None = None,
    from_format: str | None = None,
    reference_doc: str | None = None,
    output_file: str | None = None,
) -> int:
    """Apply parsed `pmt build-reply` settings and run the reply build."""
    reply = checked_markdown_path(markdown)

    try:
        output = reply_output_path(reply, output_file)
        output_format = reply_output_format(output)
        manuscript = Path(reply_manuscript or DEFAULT_REPLY_MANUSCRIPT_FILE)
        line_source = Path(manuscript_line_source or DEFAULT_REPLY_LINE_SOURCE)
        active_from_format = from_format or DEFAULT_REPLY_FROM_FORMAT
        if output_format == "txt":
            log_info("\n[TXT] Building reviewer reply TXT...\n")
            build_reply_txt(
                reply=reply,
                manuscript=manuscript,
                manuscript_line_source=line_source,
                output=output,
                style=Path(DEFAULT_STYLE_FILE),
                from_format=active_from_format,
            )
        else:
            log_info("\n[DOCX] Building reviewer reply DOCX...\n")
            build_reply_docx(
                reply=reply,
                manuscript=manuscript,
                manuscript_line_source=line_source,
                output=output,
                reference_doc=reply_reference_doc_path(reference_doc),
                style=Path(DEFAULT_STYLE_FILE),
                from_format=active_from_format,
            )
        return 0
    except KeyboardInterrupt:
        log_warning("\n\n[WARN] Build interrupted by user.")
        return 1
    except Exception as exc:
        log_error(f"\n[ERROR] {exc}")
        return 1
