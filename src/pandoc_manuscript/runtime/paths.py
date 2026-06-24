"""Shared paths for pmt-generated working files and caches."""

from pathlib import Path


PMT_DIR = Path(".pmt")
PMT_WORK_DIR = PMT_DIR / "work"
PMT_CACHE_DIR = PMT_DIR / "cache"
PMT_TOOLS_DIR = PMT_DIR / "tools"
PMT_TOOLS_BIN_DIR = PMT_TOOLS_DIR / "bin"
PMT_TOOLS_DOWNLOAD_DIR = PMT_CACHE_DIR / "tools" / "downloads"
PMT_TOOLS_EXTRACT_DIR = PMT_WORK_DIR / "tool-extract"
PMT_FILTER_WORK_DIR = PMT_WORK_DIR / "filters"
PMT_MATHTYPE_WORK_DIR = PMT_WORK_DIR / "mathtype-build"
PMT_REPLY_WORK_DIR = PMT_WORK_DIR / "reply"
PMT_REPLY_LINE_SOURCE_PDF_DIR = PMT_REPLY_WORK_DIR / "line-source-pdf"
PMT_REPLY_LINE_SOURCE_DOCX_DIR = PMT_REPLY_WORK_DIR / "line-source-docx"
PMT_REPLY_PROBE_DIR = PMT_REPLY_WORK_DIR / "probes"
PMT_REPLY_LINE_SOURCE_CACHE_DIR = PMT_CACHE_DIR / "reply" / "line-source"
PMT_SVG_EMBED_CACHE_DIR = PMT_CACHE_DIR / "svg-embedded"
PMT_SVG_PNG_CACHE_DIR = PMT_CACHE_DIR / "svg-png"
PMT_MATHTYPE_CACHE_DIR = PMT_CACHE_DIR / "mathtype"


def pmt_path(path: str | Path) -> str:
    """Return a stable POSIX-style string for pmt default settings."""
    return Path(path).as_posix()
