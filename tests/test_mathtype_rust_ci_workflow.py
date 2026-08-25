"""Regression checks for the standalone MathType Rust CI workflow."""

from pathlib import Path

import yaml


WORKFLOW_PATH = (
    Path(__file__).resolve().parents[1]
    / ".github"
    / "workflows"
    / "mathtype-rust-ci.yml"
)


def _load_workflow() -> dict:
    """Load the MathType Rust workflow as a YAML mapping."""

    return yaml.safe_load(WORKFLOW_PATH.read_text(encoding="utf-8"))


def test_cargo_jobs_authenticate_private_latex2wmf_dependency() -> None:
    """Require private Git authentication before every job invokes Cargo."""

    workflow = _load_workflow()
    cargo_jobs = 0

    for job_name, job in workflow["jobs"].items():
        steps = job["steps"]
        cargo_step_indexes = [
            index
            for index, step in enumerate(steps)
            if isinstance(step.get("run"), str)
            and step["run"].lstrip().startswith("cargo ")
        ]
        if not cargo_step_indexes:
            continue

        cargo_jobs += 1
        auth_step_indexes = [
            index
            for index, step in enumerate(steps)
            if step.get("name") == "Authenticate private Cargo dependencies"
        ]
        assert auth_step_indexes, f"{job_name} does not authenticate Cargo Git fetches"

        auth_step_index = auth_step_indexes[0]
        auth_step = steps[auth_step_index]
        assert auth_step_index < min(cargo_step_indexes)
        assert auth_step["shell"] == "bash"
        assert auth_step["env"]["PRIVATE_SUBMODULES_TOKEN"] == (
            "${{ secrets.PRIVATE_SUBMODULES_TOKEN }}"
        )

        script = auth_step["run"]
        assert "git config --global" in script
        assert (
            "https://x-access-token:${PRIVATE_SUBMODULES_TOKEN}"
            "@github.com/cesaryuan/latex2wmf" in script
        )
        assert "https://github.com/cesaryuan/latex2wmf" in script

    assert cargo_jobs > 0
