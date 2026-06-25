from pathlib import Path
import sys

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "src"))

from pandoc_manuscript.commands import build, clean as clean_command


def test_clean_settings_calls_clean_directly(tmp_path, monkeypatch) -> None:
    """Dispatch pmt clean without routing through the build command."""
    calls = []

    monkeypatch.setattr(build, "run_build_command", lambda **_: pytest.fail("build dispatch should not run"))
    monkeypatch.setattr(clean_command, "clean", lambda output_dir: calls.append(("clean", output_dir)))

    result = clean_command.CleanSettings(project_dir=tmp_path, output_dir="artifacts").run()

    assert result == 0
    assert calls == [("clean", "artifacts")]


def test_distclean_settings_calls_distclean_directly(tmp_path, monkeypatch) -> None:
    """Dispatch pmt distclean without routing through the build command."""
    calls = []

    monkeypatch.setattr(build, "run_build_command", lambda **_: pytest.fail("build dispatch should not run"))
    monkeypatch.setattr(clean_command, "distclean", lambda output_dir: calls.append(("distclean", output_dir)))

    result = clean_command.DistcleanSettings(project_dir=tmp_path, output_dir="artifacts").run()

    assert result == 0
    assert calls == [("distclean", "artifacts")]
