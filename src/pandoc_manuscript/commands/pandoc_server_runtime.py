"""Run the project-bound PMT HTML server used by editor integrations.

The process keeps one Haskell Pandoc worker alive, loads the same defaults and
metadata that `pmt build html` uses, and exposes only project-aware requests.
Clients send a Markdown path; they do not need to know PMT filters, templates,
CSL files, resource paths, or other Pandoc options. The Python layer applies
the same HTML post-processing as the normal build before returning HTML.
"""

from __future__ import annotations

import json
import os
import subprocess
import tempfile
import threading
from http import HTTPStatus
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from typing import Any

from pydantic import Field
from pydantic_settings import BaseSettings, CliApp, SettingsConfigDict

from ..html.postprocess import postprocess_html
from ..runtime.paths import PMT_DIR, PMT_TOOLS_BIN_DIR


class ServerRuntimeSettings(BaseSettings):
    """CLI settings for the internal PMT HTTP runtime."""

    model_config = SettingsConfigDict(cli_kebab_case=True, cli_implicit_flags=True)

    config: Path = Field(description="Project-bound PMT server configuration JSON.")
    host: str = Field(default="127.0.0.1", description="HTTP bind host.")
    port: int = Field(default=3030, description="HTTP port.")


class PandocWorker:
    """Serialize requests to one long-lived Pandoc API worker process."""

    def __init__(self, config: dict[str, Any], log_path: Path) -> None:
        self._config = config
        self._lock = threading.Lock()
        self._process = self._start_process(config, log_path)

    @staticmethod
    def _resolve_command(config: dict[str, Any]) -> list[str]:
        """Resolve the worker executable configured for the current project."""
        configured = os.environ.get("PMT_PANDOC_SERVER_WORKER_COMMAND")
        if configured:
            import shlex

            return shlex.split(configured, posix=os.name != "nt")

        names = ("pmt-pandoc-worker.exe", "pmt-pandoc-worker") if os.name == "nt" else ("pmt-pandoc-worker",)
        for name in names:
            candidate = PMT_TOOLS_BIN_DIR / name
            if candidate.exists():
                return [str(candidate)]
        raise FileNotFoundError(
            "PMT Pandoc worker not found. Build scripts/pandoc-server and install "
            "pmt-pandoc-worker into .pmt/tools/bin, or set PMT_PANDOC_SERVER_WORKER_COMMAND."
        )

    @classmethod
    def _start_process(cls, config: dict[str, Any], log_path: Path) -> subprocess.Popen[str]:
        """Start the worker with the fixed project Pandoc options."""
        command = [*cls._resolve_command(config), "--config", str(Path(config["worker_config"]).resolve())]
        log_path.parent.mkdir(parents=True, exist_ok=True)
        log_handle = log_path.open("a", encoding="utf-8")
        try:
            process = subprocess.Popen(
                command,
                stdin=subprocess.PIPE,
                stdout=subprocess.PIPE,
                stderr=log_handle,
                text=True,
                encoding="utf-8",
                errors="replace",
                cwd=config["project_dir"],
                env=os.environ.copy(),
                bufsize=1,
            )
        except BaseException:
            log_handle.close()
            raise
        log_handle.close()
        if process.stdin is None or process.stdout is None:
            process.kill()
            raise RuntimeError("Could not open the PMT Pandoc worker protocol streams.")
        return process

    def convert(self, input_path: Path) -> str:
        """Convert one project Markdown file and return post-processed HTML."""
        project_dir = Path(self._config["project_dir"]).resolve()
        source = input_path.resolve()
        if source != project_dir and project_dir not in source.parents:
            raise ValueError(f"Markdown path must stay inside the PMT project: {source}")
        output_dir = PMT_DIR / "work" / "pandoc-server-output"
        output_dir.mkdir(parents=True, exist_ok=True)
        with tempfile.NamedTemporaryFile(prefix="html-", suffix=".html", dir=output_dir, delete=False) as handle:
            output_path = Path(handle.name)
        request = {"input": str(source), "output": str(output_path)}
        try:
            with self._lock:
                assert self._process.stdin is not None
                assert self._process.stdout is not None
                self._process.stdin.write(json.dumps(request, ensure_ascii=False) + "\n")
                self._process.stdin.flush()
                response = self._process.stdout.readline()
            if not response:
                raise RuntimeError("PMT Pandoc worker exited without a response.")
            result = json.loads(response)
            if not result.get("ok"):
                raise RuntimeError(str(result.get("error", "PMT Pandoc worker conversion failed.")))
            postprocess_html(output_path, pandoc_metadata=self._config.get("pandoc_metadata", {}))
            return output_path.read_text(encoding="utf-8")
        finally:
            output_path.unlink(missing_ok=True)

    def close(self) -> None:
        """Stop the worker when the HTTP runtime shuts down."""
        if self._process.poll() is None:
            self._process.terminate()
            self._process.wait(timeout=3)

    def is_alive(self) -> bool:
        """Return whether the long-lived Pandoc worker is still running."""
        return self._process.poll() is None


class PmtHtmlRequestHandler(BaseHTTPRequestHandler):
    """Serve project-aware HTML conversion and health endpoints."""

    server_version = "PMT-Pandoc-Server/0.1"

    def _json_response(self, payload: object, status: int = HTTPStatus.OK) -> None:
        """Write one UTF-8 JSON response."""
        body = json.dumps(payload, ensure_ascii=False).encode("utf-8")
        self.send_response(status)
        self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def do_GET(self) -> None:  # noqa: N802
        """Handle the health and version endpoint."""
        if self.path == "/version":
            worker = getattr(self.server, "worker", None)
            if worker is None or not worker.is_alive():
                self._json_response({"error": "Pandoc worker is not running."}, HTTPStatus.SERVICE_UNAVAILABLE)
                return
            self._json_response({"server": self.server_version, "protocol": "pmt-html-v1"})
            return
        self._json_response({"error": "Not found"}, HTTPStatus.NOT_FOUND)

    def do_POST(self) -> None:  # noqa: N802
        """Convert one path or a batch of paths using the project configuration."""
        length = int(self.headers.get("Content-Length", "0"))
        try:
            payload = json.loads(self.rfile.read(length).decode("utf-8"))
            if self.path in {"/", "/convert"}:
                paths = [payload.get("path", "manuscript.md")]
                batch = False
            elif self.path == "/batch":
                paths = [item["path"] for item in payload]
                batch = True
            else:
                self._json_response({"error": "Not found"}, HTTPStatus.NOT_FOUND)
                return
            results = [{"path": path, "output": self.server.worker.convert(Path(path))} for path in paths]
            self._json_response(results if batch else results[0])
        except (KeyError, TypeError, ValueError, json.JSONDecodeError, OSError, RuntimeError) as exc:
            self._json_response({"error": str(exc)}, HTTPStatus.BAD_REQUEST)

    def log_message(self, format: str, *args: object) -> None:
        """Keep request logging in the PMT server log instead of stderr."""
        return


def main() -> int:
    """Start the project-bound PMT HTML HTTP service."""
    settings = CliApp.run(ServerRuntimeSettings)
    config = json.loads(settings.config.read_text(encoding="utf-8"))
    worker_config_path = PMT_DIR / "pandoc-server-worker.json"
    worker_config_path.write_text(
        json.dumps(
            {
                # Match the Haskell worker's record field names used by Aeson.
                "projectDir": config["project_dir"],
                "pandocArgs": config["pandoc_args"],
            },
            indent=2,
        ),
        encoding="utf-8",
    )
    config["worker_config"] = str(worker_config_path)
    worker = PandocWorker(config, PMT_DIR / "pandoc-server-worker.log")
    server = ThreadingHTTPServer((settings.host, settings.port), PmtHtmlRequestHandler)
    server.worker = worker  # type: ignore[attr-defined]
    try:
        server.serve_forever()
    finally:
        worker.close()
        server.server_close()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
