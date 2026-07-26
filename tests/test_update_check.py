from __future__ import annotations

from pathlib import Path
import sys
import urllib.error

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "src"))

from pandoc_manuscript import cli
from pandoc_manuscript.runtime import update_check


class FakeResponse:
    """Minimal context-managed HTTP response for update-check tests."""

    def __init__(self, body: bytes) -> None:
        self._body = body

    def __enter__(self) -> "FakeResponse":
        return self

    def __exit__(self, exc_type, exc_value, traceback) -> None:
        """Leave the test response without suppressing an exception."""
        return None

    def read(self) -> bytes:
        """Return the configured JSON response body."""
        return self._body


def test_available_update_returns_newer_pypi_release(monkeypatch) -> None:
    """Notify users only when the PyPI release is newer than their install."""
    body = b'{"info": {"version": "0.5.4"}}'
    monkeypatch.setattr(update_check.urllib.request, "urlopen", lambda *args, **kwargs: FakeResponse(body))

    assert update_check.available_update("0.5.3") == "0.5.4"
    assert update_check.available_update("0.5.4") is None


def test_available_update_ignores_network_failures(monkeypatch) -> None:
    """Keep completed pmt commands successful when PyPI cannot be reached."""
    def fail(*args, **kwargs):
        """Simulate a failed PyPI request."""
        raise urllib.error.URLError("offline")

    monkeypatch.setattr(update_check.urllib.request, "urlopen", fail)

    assert update_check.available_update("0.5.3") is None


def test_available_update_ignores_unexpected_pypi_json(monkeypatch) -> None:
    """Treat a structurally invalid JSON response like an unavailable update check."""
    body = b'{"info": "unexpected"}'
    monkeypatch.setattr(update_check.urllib.request, "urlopen", lambda *args, **kwargs: FakeResponse(body))

    assert update_check.available_update("0.5.3") is None


def test_cli_checks_for_updates_after_a_command(monkeypatch, capsys) -> None:
    """Run the update notification after CLI command output is complete."""
    calls: list[str] = []
    monkeypatch.setattr(cli, "notify_if_update_available", calls.append)

    assert cli.main(["--version"]) == 0

    captured = capsys.readouterr()
    assert captured.out.startswith("pmt ")
    assert calls == [cli.__version__]
