from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "src"))

from pandoc_manuscript import cli
from pandoc_manuscript.cli import InitSettings
from pandoc_manuscript.tooling.pandoc_tools import ResolvedTool


def test_init_copies_packaged_agents_directory(tmp_path) -> None:
    """Copy reusable agent guidance into new manuscript projects."""
    target = tmp_path / "paper"

    InitSettings(directory=str(target)).run()

    assert (target / "manuscript-syntax.md").is_file()
    assert (target / ".agents" / "manuscript-review" / "SKILL.md").is_file()
    assert (target / ".agents" / "word-manuscript-fix" / "scripts" / "unescape_latex.py").is_file()


def test_init_merge_agents_directory_keeps_existing_files(tmp_path) -> None:
    """Merge .agents by filling missing files without overwriting local edits."""
    target = tmp_path / "paper"
    existing_skill = target / ".agents" / "word-manuscript-fix" / "SKILL.md"
    existing_skill.parent.mkdir(parents=True)
    existing_skill.write_text("local skill notes\n", encoding="utf-8")

    InitSettings(directory=str(target), merge=True).run()

    assert existing_skill.read_text(encoding="utf-8") == "local skill notes\n"
    assert (target / ".agents" / "manuscript-review" / "SKILL.md").is_file()
    assert (target / ".agents" / "word-manuscript-fix" / "scripts" / "unescape_latex.py").is_file()


def test_init_setup_runs_after_project_creation(tmp_path, monkeypatch) -> None:
    """Run optional tool setup inside the newly initialized project."""
    target = tmp_path / "paper"
    calls = []

    def fake_setup_pandoc_tools(*, force=False):
        """Record init --setup without touching the network."""
        calls.append((Path.cwd(), force))
        return (
            ResolvedTool("pandoc", Path(".pmt/tools/bin/pandoc"), ".pmt/tools"),
            ResolvedTool("pandoc-crossref", Path(".pmt/tools/bin/pandoc-crossref"), ".pmt/tools"),
        )

    monkeypatch.setattr(cli, "setup_pandoc_tools", fake_setup_pandoc_tools)

    InitSettings(directory=str(target), setup=True).run()

    assert (target / "manuscript.md").is_file()
    assert calls == [(target.resolve(), False)]
