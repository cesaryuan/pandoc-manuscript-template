from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "src"))

from pandoc_manuscript.cli import InitSettings


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
