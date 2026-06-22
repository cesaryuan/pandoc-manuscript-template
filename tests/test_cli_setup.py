from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "src"))

from pandoc_manuscript import cli
from pandoc_manuscript.tools import ResolvedTool


def test_setup_command_runs_in_project_directory(tmp_path, monkeypatch) -> None:
    """Run pmt setup from the selected project directory."""
    calls = []

    def fake_setup_pandoc_tools(*, force=False):
        """Record the setup working directory without touching the network."""
        calls.append((Path.cwd(), force))
        return (
            ResolvedTool("pandoc", Path(".pmt/tools/bin/pandoc"), ".pmt/tools"),
            ResolvedTool("pandoc-crossref", Path(".pmt/tools/bin/pandoc-crossref"), ".pmt/tools"),
        )

    monkeypatch.setattr(cli, "setup_pandoc_tools", fake_setup_pandoc_tools)

    result = cli.SetupSettings(project_dir=tmp_path, force=True).run()

    assert result == 0
    assert calls == [(tmp_path.resolve(), True)]
