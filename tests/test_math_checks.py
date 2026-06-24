from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "src"))

from pandoc_manuscript.commands import build, reply_build
from pandoc_manuscript.checks.math import find_hat_macro_order_issues


def test_find_hat_macro_order_issues_reports_locations_and_rewrites() -> None:
    """Report MathType-sensitive hat ordering with stable rewrite hints."""
    text = (
        "Safe: $\\mathbf{\\hat{C}}$.\n"
        "Warn: $\\hat{\\mathbf{C}}$ and $\\hat{\\mathcal{\\Gamma}}$.\n"
    )

    issues = find_hat_macro_order_issues(text)

    assert [(issue.line, issue.column) for issue in issues] == [(2, 8), (2, 31)]
    assert [issue.suggested_text for issue in issues] == [
        "\\mathbf{\\hat{C}}",
        "\\mathcal{\\hat{\\Gamma}}",
    ]


def test_find_hat_macro_order_issues_ignores_safe_and_non_style_forms() -> None:
    """Avoid warning on already-safe formulas or unrelated hat usage."""
    text = (
        "$\\hat{C}$\n"
        "$\\mathbf{\\hat{C}}$\n"
        "$\\hat{\\frac{a}{b}}$\n"
    )

    assert find_hat_macro_order_issues(text) == []


def test_run_build_command_forwards_hat_preflight_flag(tmp_path, monkeypatch) -> None:
    """Pass the hat-order preflight flag through to the DOCX build implementation."""
    manuscript = tmp_path / "paper.md"
    manuscript.write_text("$\\hat{\\mathbf{C}}$\n", encoding="utf-8")
    calls: list[Path] = []
    kwargs_seen: list[dict[str, bool]] = []

    monkeypatch.setattr(build, "warn_mathtype_hat_style_order", lambda path: calls.append(path))
    monkeypatch.setattr(build, "build_docx", lambda **kwargs: kwargs_seen.append(kwargs))

    result = build.run_build_command(target="docx", markdown=str(manuscript))

    assert result == 0
    assert kwargs_seen == [{"warn_hat_order": True}]
    assert calls == []


def test_run_build_reply_command_txt_skips_hat_preflight(tmp_path, monkeypatch) -> None:
    """Skip the MathType hat-order preflight for TXT-only reply builds."""
    reply = tmp_path / "reply.md"
    manuscript = tmp_path / "manuscript.md"
    output = tmp_path / "reply.txt"
    reply.write_text("$\\hat{\\mathbf{C}}$\n", encoding="utf-8")
    manuscript.write_text("$\\hat{\\mathbf{D}}$\n", encoding="utf-8")
    calls: list[Path] = []

    monkeypatch.setattr(reply_build, "warn_mathtype_hat_style_order", lambda path: calls.append(path))
    monkeypatch.setattr(reply_build, "build_reply_txt", lambda **kwargs: None)

    result = reply_build.run_build_reply_command(
        markdown=str(reply),
        reply_manuscript=str(manuscript),
        output_file=str(output),
    )

    assert result == 0
    assert calls == []
