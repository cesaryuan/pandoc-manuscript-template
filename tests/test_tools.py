from pathlib import Path
import io
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


def test_progress_line_shows_percentage_for_known_size() -> None:
    """Render a determinate progress line when Content-Length is known."""
    line = tools.progress_line(512, 1024)

    assert "50.0%" in line
    assert "512 B/1.0 KiB" in line


class FakeDownloadResponse:
    """Small response double for streamed download progress tests."""

    def __init__(self, chunks: list[bytes], content_length: int | None = None) -> None:
        self.chunks = chunks
        self.headers = {}
        if content_length is not None:
            self.headers["Content-Length"] = str(content_length)

    def read(self, size: int) -> bytes:
        """Return the next fake network chunk."""
        if not self.chunks:
            return b""
        return self.chunks.pop(0)


def test_copy_response_with_progress_streams_body_and_finishes(monkeypatch) -> None:
    """Copy downloaded bytes in chunks and emit a final progress update."""
    response = FakeDownloadResponse([b"abc", b"def"], content_length=6)
    output = io.BytesIO()
    updates = []

    def fake_write_progress(downloaded, total, *, final=False):
        """Record progress updates without writing to the test terminal."""
        updates.append((downloaded, total, final))

    monkeypatch.setattr(tools, "write_progress", fake_write_progress)

    tools.copy_response_with_progress(response, output)

    assert output.getvalue() == b"abcdef"
    assert updates == [(3, 6, False), (6, 6, False), (6, 6, True)]


def test_setup_pandoc_tools_installs_managed_tools_even_when_path_exists(monkeypatch, tmp_path) -> None:
    """Prepare .pmt/tools explicitly instead of reusing system PATH tools."""
    tools.TOOL_CACHE.clear()
    monkeypatch.chdir(tmp_path)
    monkeypatch.setattr(shutil, "which", lambda command: f"C:/system/{command}.exe")
    monkeypatch.setattr(tools, "crossref_pandoc_version", lambda path: "3.9.0.2")
    monkeypatch.setattr(tools, "pandoc_release_for_crossref", lambda version: {"tag_name": version, "assets": []})
    calls = []

    def fake_install_release_tool(tool, release=None, *, force_download=False):
        """Create a fake managed executable without network access."""
        calls.append((tool, release, force_download))
        executable = tools.managed_executable(tool)
        executable.parent.mkdir(parents=True, exist_ok=True)
        executable.write_text("fake", encoding="utf-8")
        return tools.ResolvedTool(tool, executable, ".pmt/tools (fake)")

    monkeypatch.setattr(tools, "install_release_tool", fake_install_release_tool)

    pandoc, crossref = tools.setup_pandoc_tools()

    assert pandoc.executable == tools.managed_executable("pandoc")
    assert crossref.executable == tools.managed_executable("pandoc-crossref")
    assert calls == [
        ("pandoc-crossref", None, False),
        ("pandoc", {"tag_name": "3.9.0.2", "assets": []}, False),
    ]
    tools.TOOL_CACHE.clear()
