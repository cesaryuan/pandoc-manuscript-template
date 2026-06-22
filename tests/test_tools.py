from pathlib import Path
import shutil
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "src"))

from pandoc_manuscript import tools
from pandoc_manuscript.paths import PMT_TOOLS_BIN_DIR


def test_pandoc_asset_selection_uses_platform_preferences(monkeypatch) -> None:
    """Select the platform-specific Pandoc archive from GitHub release metadata."""
    monkeypatch.setattr(tools, "current_platform_key", lambda: ("windows", "x64"))
    release = {
        "assets": [
            {"name": "pandoc-3.10-linux-amd64.tar.gz", "browser_download_url": "linux"},
            {"name": "pandoc-3.10-windows-x86_64.zip", "browser_download_url": "windows"},
        ]
    }

    assert tools.select_release_asset("pandoc", release) == {
        "name": "pandoc-3.10-windows-x86_64.zip",
        "url": "windows",
    }


def test_crossref_asset_selection_uses_platform_preferences(monkeypatch) -> None:
    """Select the platform-specific pandoc-crossref archive."""
    monkeypatch.setattr(tools, "current_platform_key", lambda: ("linux", "arm64"))
    release = {
        "assets": [
            {"name": "pandoc-crossref-Linux-X64.tar.xz", "browser_download_url": "x64"},
            {"name": "pandoc-crossref-Linux-ARM64.tar.xz", "browser_download_url": "arm64"},
        ]
    }

    assert tools.select_release_asset("pandoc-crossref", release) == {
        "name": "pandoc-crossref-Linux-ARM64.tar.xz",
        "url": "arm64",
    }


def test_pandoc_tools_env_prepends_managed_bin(monkeypatch, tmp_path) -> None:
    """Expose .pmt/tools/bin to Pandoc so managed filters are discoverable."""
    monkeypatch.chdir(tmp_path)

    env = tools.pandoc_tools_env({"PATH": "base"})

    assert env["PATH"].split(tools.os.pathsep)[0] == str((tmp_path / PMT_TOOLS_BIN_DIR).resolve())
    assert env["PATH"].endswith("base")


def test_resolve_tool_prefers_system_path(monkeypatch, tmp_path) -> None:
    """Use an existing system command instead of downloading a managed copy."""
    tools.TOOL_CACHE.clear()
    fake_executable = tmp_path / ("pandoc.exe" if tools.os.name == "nt" else "pandoc")
    fake_executable.write_text("fake", encoding="utf-8")

    monkeypatch.setattr(shutil, "which", lambda command: str(fake_executable) if command == "pandoc" else None)
    monkeypatch.setattr(
        tools,
        "install_release_tool",
        lambda command: (_ for _ in ()).throw(AssertionError("download should not run")),
    )

    resolved = tools.resolve_tool("pandoc")

    assert resolved.executable == fake_executable
    assert resolved.source == "PATH"
    tools.TOOL_CACHE.clear()


def test_crossref_pandoc_version_parses_reported_build_version(monkeypatch, tmp_path) -> None:
    """Read the Pandoc ABI version reported by pandoc-crossref --version."""
    crossref = tmp_path / "pandoc-crossref.exe"
    crossref.write_text("fake", encoding="utf-8")
    monkeypatch.setattr(
        tools,
        "subprocess_run_version",
        lambda executable: (
            "pandoc-crossref v0.3.24 built with Pandoc v3.9.0.2, "
            "pandoc-types v1.23.1.1 and GHC 9.8.4"
        ),
    )

    assert tools.crossref_pandoc_version(crossref) == "3.9.0.2"


def test_download_proxy_prefers_https_proxy_env(monkeypatch) -> None:
    """Use HTTPS_PROXY before consulting the operating system proxy."""
    monkeypatch.setenv("HTTPS_PROXY", "http://127.0.0.1:7890")
    monkeypatch.setattr(
        tools,
        "system_https_proxy",
        lambda: (_ for _ in ()).throw(AssertionError("system proxy should not be read")),
    )

    proxy = tools.download_proxy_config()

    assert proxy.proxy == "http://127.0.0.1:7890"
    assert proxy.source == "HTTPS_PROXY"


def test_download_proxy_uses_system_proxy_when_env_missing(monkeypatch) -> None:
    """Fall back to the OS proxy when HTTPS_PROXY is not set."""
    monkeypatch.delenv("HTTPS_PROXY", raising=False)
    monkeypatch.delenv("https_proxy", raising=False)
    monkeypatch.setattr(tools, "system_https_proxy", lambda: "http://127.0.0.1:7891")

    proxy = tools.download_proxy_config()

    assert proxy.proxy == "http://127.0.0.1:7891"
    assert proxy.source == "system proxy"


def test_download_proxy_direct_when_no_proxy(monkeypatch) -> None:
    """Use direct downloads when neither environment nor system proxy exists."""
    monkeypatch.delenv("HTTPS_PROXY", raising=False)
    monkeypatch.delenv("https_proxy", raising=False)
    monkeypatch.setattr(tools, "system_https_proxy", lambda: None)

    proxy = tools.download_proxy_config()

    assert proxy.proxy is None
    assert proxy.source == "direct"
