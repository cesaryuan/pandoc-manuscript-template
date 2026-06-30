"""Custom Hatchling build steps for native helper artifacts."""

from __future__ import annotations

import os
import shutil
import subprocess
from pathlib import Path
from typing import Any

from hatchling.builders.hooks.plugin.interface import BuildHookInterface


class CustomBuildHook(BuildHookInterface):
    """Build and include the Rust MathType fallback executable in wheels."""

    PLUGIN_NAME = "pmt-native-helpers"

    def initialize(self, version: str, build_data: dict[str, Any]) -> None:
        """Compile mathtype-rust before wheel file selection.

        The source tree intentionally does not store a copied executable under
        ``src/``; this hook prevents stale wheel artifacts after Rust edits.
        """
        if self.target_name != "wheel" or version == "editable":
            return

        if os.name != "nt":
            # MathType wheel bundling is a Windows-only release concern. Other
            # platforms still need to build successfully without the helper.
            return

        root = Path(self.root)
        executable = self.build_mathtype_rust_executable(root)
        build_data.setdefault("force_include", {})[str(executable)] = (
            f"pandoc_manuscript/mathtype/bin/{executable.name}"
        )

    def build_mathtype_rust_executable(self, root: Path) -> Path:
        """Build the Windows MathType helper and return the executable path."""
        manifest = root / "scripts" / "mathtype-rust" / "Cargo.toml"
        if not manifest.exists():
            raise FileNotFoundError(f"mathtype-rust manifest is missing: {manifest}")
        if shutil.which("cargo") is None:
            raise RuntimeError("Building a wheel with MathType fallback requires `cargo` on PATH.")

        print("[pmt build] building Windows MathType helper with cargo", flush=True)
        subprocess.run(
            [
                "cargo",
                "build",
                "--manifest-path",
                str(manifest),
                "--bin",
                "mathtype-rust",
            ],
            cwd=root,
            check=True,
        )

        exe_name = "mathtype-rust.exe"
        executable = root / "scripts" / "mathtype-rust" / "target" / "debug" / exe_name
        if not executable.exists():
            raise FileNotFoundError(f"cargo did not create expected executable: {executable}")
        return executable
