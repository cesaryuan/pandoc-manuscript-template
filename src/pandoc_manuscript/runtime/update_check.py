"""Check PyPI for a newer PMT release after a CLI command finishes."""

from __future__ import annotations

import json
import urllib.error
import urllib.request
from typing import Any

from packaging.version import InvalidVersion, Version

from .logging import log_warning


DIST_NAME = "pandoc-manuscript-template"
PYPI_JSON_URL = f"https://pypi.org/pypi/{DIST_NAME}/json"
REQUEST_TIMEOUT_SECONDS = 2
USER_AGENT = f"{DIST_NAME} update check"


def available_update(installed_version: str) -> str | None:
    """Return a newer PyPI version, or None when checking is not possible."""
    try:
        installed = Version(installed_version)
    except InvalidVersion:
        return None

    request = urllib.request.Request(PYPI_JSON_URL, headers={"User-Agent": USER_AGENT})
    try:
        with urllib.request.urlopen(request, timeout=REQUEST_TIMEOUT_SECONDS) as response:
            payload: Any = json.loads(response.read().decode("utf-8"))
    except (OSError, UnicodeDecodeError, json.JSONDecodeError, urllib.error.URLError):
        # The update check must never make an otherwise completed command fail.
        return None

    info = payload.get("info") if isinstance(payload, dict) else None
    latest_version = info.get("version") if isinstance(info, dict) else None
    if not isinstance(latest_version, str):
        return None

    try:
        latest = Version(latest_version)
    except InvalidVersion:
        return None
    return latest_version if latest > installed else None


def notify_if_update_available(installed_version: str) -> None:
    """Print a non-blocking upgrade hint when PyPI has a newer PMT release."""
    latest_version = available_update(installed_version)
    if latest_version is None:
        return
    log_warning(
        f"[UPDATE] pmt {latest_version} is available, upgrade with "
        f"`uv tool upgrade {DIST_NAME}`"
    )
