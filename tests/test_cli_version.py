from pathlib import Path
import sys
import importlib.metadata as importlib_metadata

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "src"))

from pandoc_manuscript import cli


def test_cli_version_uses_distribution_metadata(capsys) -> None:
    """Keep `pmt --version` synchronized with the installed package metadata."""
    expected = importlib_metadata.version("pandoc-manuscript-template")

    result = cli.main(["--version"])

    assert result == 0
    assert capsys.readouterr().out.strip() == f"pmt {expected}"
