from pathlib import Path
import sys
import importlib.metadata as importlib_metadata

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "src"))

from pandoc_manuscript import cli


def test_cli_version_uses_distribution_metadata(monkeypatch, capsys) -> None:
    """Keep `pmt --version` synchronized with the installed package metadata."""
    expected = importlib_metadata.version("pandoc-manuscript-template")
    monkeypatch.setattr(cli, "notify_if_update_available", lambda version: None)

    result = cli.main(["--version"])

    assert result == 0
    assert capsys.readouterr().out.strip() == f"pmt {expected}"
