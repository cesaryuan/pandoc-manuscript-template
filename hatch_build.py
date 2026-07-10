"""Custom Hatchling build steps for native helper artifacts."""

from __future__ import annotations

import os
import shutil
import subprocess
from pathlib import Path
from typing import Any

from hatchling.builders.hooks.plugin.interface import BuildHookInterface
from packaging.tags import sys_tags


class CustomBuildHook(BuildHookInterface):
    """Build and include the platform's native MathType executables in wheels."""

    PLUGIN_NAME = "pmt-native-helpers"

    def initialize(self, version: str, build_data: dict[str, Any]) -> None:
        """Compile mathtype-rust before wheel file selection.

        The source tree intentionally does not store a copied executable under
        ``src/``; this hook prevents stale wheel artifacts after Rust edits.
        """
        if self.target_name != "wheel" or version == "editable":
            return

        root = Path(self.root)
        executables = (
            self.build_native_executable(root, "mathtype-rust"),
            self.build_native_executable(root, "latex2wmf"),
        )
        force_include = build_data.setdefault("force_include", {})
        for executable in executables:
            force_include[str(executable)] = f"pandoc_manuscript/mathtype/bin/{executable.name}"

        # Native helpers make this a platform wheel even though the Python
        # package itself has no extension module or CPython ABI dependency.
        platform_tag = next(iter(sys_tags())).platform
        build_data["pure_python"] = False
        build_data["tag"] = f"py3-none-{platform_tag}"

    def build_native_executable(self, root: Path, project_name: str) -> Path:
        """Build one release-mode Rust helper and return its executable path."""
        manifest = root / "scripts" / project_name / "Cargo.toml"
        if not manifest.exists():
            raise FileNotFoundError(f"{project_name} manifest is missing: {manifest}")
        if shutil.which("cargo") is None:
            raise RuntimeError("Building a wheel with MathType native helpers requires `cargo` on PATH.")

        print(f"[pmt build] building {project_name} release helper with cargo", flush=True)
        subprocess.run(
            [
                "cargo",
                "build",
                "--manifest-path",
                str(manifest),
                "--bin",
                project_name,
                "--release",
            ],
            cwd=root,
            check=True,
        )

        suffix = ".exe" if os.name == "nt" else ""
        exe_name = f"{project_name}{suffix}"
        executable = root / "scripts" / project_name / "target" / "release" / exe_name
        if not executable.exists():
            raise FileNotFoundError(f"cargo did not create expected executable: {executable}")
        return executable
