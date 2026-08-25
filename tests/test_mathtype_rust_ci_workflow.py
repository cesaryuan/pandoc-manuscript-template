"""Regression checks for the standalone MathType Rust CI workflow."""

import tomllib
from pathlib import Path

import yaml


REPOSITORY_ROOT = Path(__file__).resolve().parents[1]
WORKFLOW_PATH = REPOSITORY_ROOT / ".github" / "workflows" / "mathtype-rust-ci.yml"
MATHTYPE_MANIFEST_PATH = REPOSITORY_ROOT / "scripts" / "mathtype-rust" / "Cargo.toml"
LATEX2WMF_MANIFEST_PATH = REPOSITORY_ROOT / "scripts" / "latex2wmf" / "Cargo.toml"
WMF_MINIMUM_RUST_VERSION = "1.92"


def _load_workflow() -> dict:
    """Load the MathType Rust workflow as a YAML mapping."""

    return yaml.safe_load(WORKFLOW_PATH.read_text(encoding="utf-8"))


def _load_toml(path: Path) -> dict:
    """Load a Cargo manifest as a TOML mapping."""

    return tomllib.loads(path.read_text(encoding="utf-8"))


def test_wmf_crates_and_ci_share_minimum_rust_version() -> None:
    """Keep the WMF dependency chain and its CI toolchain on Rust 1.92."""

    mathtype_manifest = _load_toml(MATHTYPE_MANIFEST_PATH)
    latex2wmf_manifest = _load_toml(LATEX2WMF_MANIFEST_PATH)
    assert mathtype_manifest["package"]["rust-version"] == WMF_MINIMUM_RUST_VERSION
    assert latex2wmf_manifest["package"]["rust-version"] == WMF_MINIMUM_RUST_VERSION

    quality_steps = {
        step["name"]: step
        for step in _load_workflow()["jobs"]["quality"]["steps"]
        if "name" in step
    }
    assert quality_steps["Install minimum supported Rust"]["run"] == (
        f"rustup toolchain install {WMF_MINIMUM_RUST_VERSION}.0 --profile minimal"
    )
    assert quality_steps["Check minimum supported Rust"]["run"] == (
        f"cargo +{WMF_MINIMUM_RUST_VERSION}.0 check --all-targets"
    )


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
