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
    """Build and include binary-only MathType runtime tools in wheels."""

    PLUGIN_NAME = "pmt-native-helpers"

    def initialize(self, version: str, build_data: dict[str, Any]) -> None:
        """Compile platform runtime tools before wheel file selection.

        Rust executables are never copied into ``src/``, and the .NET helper is
        rebuilt into ``.pmt`` so wheel contents cannot become stale.
        """
        if self.target_name != "wheel" or version == "editable":
            return

        root = Path(self.root)
        executables = [
            self.build_native_executable(root, "mathtype-rust"),
            self.build_native_executable(root, "latex2wmf"),
        ]
        if os.name == "nt":
            executables.append(self.build_mathtype_ole_helper(root))
        force_include = build_data.setdefault("force_include", {})
        for executable in executables:
            if executable.name == "MathTypeOleHelper.exe":
                destination = "pandoc_manuscript/mathtype/ole_helper/bin/Release/net48/MathTypeOleHelper.exe"
            else:
                destination = f"pandoc_manuscript/mathtype/bin/{executable.name}"
            force_include[str(executable)] = destination

        # XITS Math is compiled into latex2wmf; ship its OFL and upstream
        # notices so installed wheels retain the required attribution.
        font_assets = root / "scripts" / "latex2wmf" / "assets" / "fonts"
        font_notices = (
            "XITS-NOTICE.txt",
            "XITS-OFL.txt",
            "XITS-README.txt",
        )
        for notice in font_notices:
            source = font_assets / notice
            if not source.exists():
                raise FileNotFoundError(f"XITS Math notice is missing: {source}")
            force_include[str(source)] = f"pandoc_manuscript/mathtype/bin/{notice}"

        # MiTeX's Typst sources are compiled into latex2wmf rather than shipped
        # as source files; retain the upstream Apache-2.0 notice beside it.
        mitex_license = root / "scripts" / "latex2wmf" / "assets" / "mitex" / "MITEX-APACHE-2.0.txt"
        if not mitex_license.exists():
            raise FileNotFoundError(f"MiTeX license is missing: {mitex_license}")
        force_include[str(mitex_license)] = "pandoc_manuscript/mathtype/bin/MITEX-APACHE-2.0.txt"

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

    def build_mathtype_ole_helper(self, root: Path) -> Path:
        """Build the Windows SDK helper once while producing the wheel."""
        project = root / "src" / "pandoc_manuscript" / "mathtype" / "ole_helper" / "MathTypeOleHelper.csproj"
        if not project.exists():
            raise FileNotFoundError(f"MathType OLE helper project is missing: {project}")
        if shutil.which("dotnet") is None:
            raise RuntimeError("Building the Windows wheel requires `dotnet` on PATH.")

        # Keep wheel builds from rewriting the tracked source-checkout fallback binary.
        output_dir = root / ".pmt" / "native-wheel" / "MathTypeOleHelper"
        print("[pmt build] building MathTypeOleHelper release executable with dotnet", flush=True)
        subprocess.run(
            [
                "dotnet",
                "build",
                str(project),
                "-c",
                "Release",
                "-v:quiet",
                "--output",
                str(output_dir),
            ],
            cwd=root,
            check=True,
        )
        executable = output_dir / "MathTypeOleHelper.exe"
        if not executable.exists():
            raise FileNotFoundError(f"dotnet did not create expected executable: {executable}")
        return executable
