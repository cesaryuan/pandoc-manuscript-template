from pathlib import Path
import subprocess
import sys
from types import SimpleNamespace

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "src"))

from pandoc_manuscript import reply_build
from pandoc_manuscript.reply_build import extract_citation_clusters, replace_citations


def test_extract_citation_clusters_skips_crossrefs() -> None:
    """Extract bibliography clusters without treating cross-references as citations."""
    markdown = "See [@zhang2022critical; @li2023neuralangelo] and [@fig:overview]."

    assert extract_citation_clusters(markdown) == ["[@zhang2022critical; @li2023neuralangelo]"]


def test_replace_citations_prefers_resolved_cluster_display() -> None:
    """Use citeproc-resolved cluster text so CSL delimiters and sorting survive."""
    markdown = "Prior work [@zhang2022critical; @li2023neuralangelo] is relevant."

    resolved = replace_citations(
        markdown,
        {
            "zhang2022critical": "[53]",
            "li2023neuralangelo": "[54]",
        },
        {"[@zhang2022critical; @li2023neuralangelo]": "[53, 54]"},
    )

    assert resolved == "Prior work [53, 54] is relevant."


def test_replace_citations_keeps_unresolved_clusters() -> None:
    """Keep unresolved citation clusters instead of doing unsafe partial replacements."""
    markdown = "Prior work [@zhang2022critical; @li2023neuralangelo] is relevant."

    resolved = replace_citations(
        markdown,
        {
            "zhang2022critical": "[53]",
            "li2023neuralangelo": "[54]",
        },
    )

    assert resolved == markdown


def test_replace_citations_protects_unresolved_clusters() -> None:
    """Avoid partial replacements that recreate the old double-bracket bug."""
    markdown = "Cluster [@zhang2022critical; @missing] and bare @zhang2022critical."

    resolved = replace_citations(markdown, {"zhang2022critical": "[53]"})

    assert resolved == "Cluster [@zhang2022critical; @missing] and bare [53]."


def test_prepare_line_source_pdf_uses_soffice_on_non_windows(tmp_path, monkeypatch) -> None:
    """Convert DOCX line sources with soffice when Word COM is unavailable."""
    source_docx = tmp_path / "manuscript.docx"
    source_docx.write_bytes(b"docx")
    pdf_dir = tmp_path / "reply-line-source-pdf"
    calls = []

    def fake_run(cmd, **kwargs):
        """Pretend soffice created the PDF path that its CLI derives from DOCX stem."""
        calls.append((cmd, kwargs))
        outdir = Path(cmd[cmd.index("--outdir") + 1])
        outdir.mkdir(parents=True, exist_ok=True)
        (outdir / "manuscript.pdf").write_bytes(b"%PDF")
        return subprocess.CompletedProcess(cmd, 0, stdout="", stderr="")

    monkeypatch.setattr(reply_build.sys, "platform", "linux")
    monkeypatch.setattr(reply_build, "LINE_SOURCE_PDF_DIR", pdf_dir)
    monkeypatch.setattr(reply_build.subprocess, "run", fake_run)

    result = reply_build.prepare_line_source_pdf(source_docx)

    assert result == pdf_dir / "manuscript.pdf"
    assert result.exists()
    assert calls == [
        (
            [
                "soffice",
                "--headless",
                "--convert-to",
                "pdf",
                "--outdir",
                str(pdf_dir.resolve()),
                str(source_docx.resolve()),
            ],
            {
                "capture_output": True,
                "text": True,
                "encoding": "utf-8",
                "errors": "replace",
            },
        )
    ]


def test_default_reply_line_source_is_manuscript_markdown() -> None:
    """Use manuscript.md by default so line sources rebuild from current manuscript content."""
    assert reply_build.DEFAULT_REPLY_LINE_SOURCE == "manuscript.md"


def test_prepare_line_source_pdf_builds_markdown_before_pdf(tmp_path, monkeypatch) -> None:
    """Convert Markdown line sources through a temporary DOCX before PDF extraction."""
    source_markdown = tmp_path / "manuscript.md"
    source_markdown.write_text("# Manuscript\n", encoding="utf-8")
    docx_dir = tmp_path / "line-source-docx"
    pdf_dir = tmp_path / "line-source-pdf"
    calls = []

    def fake_build_markdown_line_source_docx(source, target):
        """Pretend the normal manuscript DOCX build created the intermediate file."""
        calls.append(("build", source, target))
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(b"docx")

    def fake_export_docx_to_pdf_with_word(source, target):
        """Pretend Word exported the intermediate DOCX to PDF."""
        calls.append(("pdf", source, target))
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(b"%PDF")

    monkeypatch.setattr(reply_build, "LINE_SOURCE_DOCX_DIR", docx_dir)
    monkeypatch.setattr(reply_build, "LINE_SOURCE_PDF_DIR", pdf_dir)
    monkeypatch.setattr(reply_build.sys, "platform", "win32")
    monkeypatch.setattr(reply_build, "build_markdown_line_source_docx", fake_build_markdown_line_source_docx)
    monkeypatch.setattr(reply_build, "export_docx_to_pdf_with_word", fake_export_docx_to_pdf_with_word)

    result = reply_build.prepare_line_source_pdf(source_markdown)

    expected_docx = docx_dir / "manuscript.docx"
    expected_pdf = pdf_dir / "manuscript.pdf"
    assert result == expected_pdf
    assert calls == [
        ("build", source_markdown, expected_docx),
        ("pdf", expected_docx, expected_pdf),
    ]


def test_build_reply_docx_uses_svg_filters(tmp_path, monkeypatch) -> None:
    """Apply reply SVG embedding and rasterization filters during Pandoc DOCX build."""
    reply = tmp_path / "reply.md"
    reply.write_text("See ![layout](figures/layout.svg).\n", encoding="utf-8")
    manuscript = tmp_path / "manuscript.md"
    manuscript.write_text("# Manuscript\n", encoding="utf-8")
    output = tmp_path / "reply.docx"
    reference_doc = tmp_path / "reference.docx"
    reference_doc.write_bytes(b"docx")
    style = tmp_path / "style.yml"
    style.write_text("docxEmbedSvgImages: true\n", encoding="utf-8")
    resolved_reply = tmp_path / "reply.resolved.md"
    calls: list[tuple[list[str], dict[str, str]]] = []

    monkeypatch.setattr(reply_build, "write_reply_style_metadata_file", lambda _: style)
    monkeypatch.setattr(
        reply_build,
        "load_reply_metadata",
        lambda *_: {"docxEmbedSvgImages": True, "docxConvertSvgToPng": False},
    )
    monkeypatch.setattr(reply_build, "resolve_mathtype_enabled", lambda requested: False)
    monkeypatch.setattr(reply_build, "resolve_reference_map", lambda *args: {})
    monkeypatch.setattr(reply_build, "resolve_citation_map", lambda *args: {})
    monkeypatch.setattr(reply_build, "resolve_citation_cluster_map", lambda *args: {})
    monkeypatch.setattr(reply_build, "resolve_line_regexes", lambda text, source: text)
    monkeypatch.setattr(reply_build, "replace_references", lambda text, refs: text)
    monkeypatch.setattr(reply_build, "replace_citations", lambda text, refs, clusters=None: text)
    monkeypatch.setattr(reply_build, "resolved_reply_path", lambda _: resolved_reply)
    monkeypatch.setattr(reply_build, "pandoc_command", lambda: "pandoc")
    monkeypatch.setattr(reply_build, "table_metadata_filter_args", lambda: ["--lua-filter", "table.lua"])
    monkeypatch.setattr(reply_build, "svg_embed_images_filter_args", lambda: ["--filter", "embed.py"])
    monkeypatch.setattr(reply_build, "svg_to_png_filter_args", lambda: ["--filter", "png.py"])
    monkeypatch.setattr(reply_build, "pandoc_tools_env", lambda env=None: env or {})
    monkeypatch.setattr(reply_build, "postprocess_docx", lambda *args, **kwargs: True)
    monkeypatch.setattr(reply_build, "validate_final_docx_syntax", lambda path: [])

    def fake_run_command(cmd, env=None):
        """Capture the Pandoc command without running external tools."""
        calls.append((cmd, env or {}))
        output.write_bytes(b"docx")
        return subprocess.CompletedProcess(cmd, 0)

    monkeypatch.setattr(reply_build, "run_command", fake_run_command)

    reply_build.build_reply_docx(
        reply=reply,
        manuscript=manuscript,
        manuscript_line_source=manuscript,
        output=output,
        reference_doc=reference_doc,
        style=style,
        from_format="markdown",
    )

    assert len(calls) == 1
    cmd, env = calls[0]
    assert "--filter" in cmd
    assert "embed.py" in cmd
    assert "png.py" in cmd
    assert env["PMT_SVG_EMBED_IMAGES"] == "true"
    assert env["PMT_SVG_TO_PNG_CONVERT_ALL"] == "false"
    assert str(tmp_path.resolve()) in env["PMT_SVG_EMBED_BASE_DIRS"]


class FakePdfPage:
    """Minimal PyMuPDF page double for line-number extraction tests."""

    def __init__(self, text: str = "", blocks: list[dict] | None = None) -> None:
        self.text = text
        self.blocks = blocks or []

    def get_text(self, kind: str):
        """Return fake text or dict output matching the PyMuPDF API shape we use."""
        if kind == "text":
            return self.text
        if kind == "dict":
            return {"blocks": self.blocks}
        raise ValueError(kind)


class FakePdfDocument(list):
    """Context-manager list of pages with PyMuPDF-like metadata."""

    def __init__(self, pages: list[FakePdfPage], metadata: dict[str, str]) -> None:
        super().__init__(pages)
        self.metadata = metadata

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc, traceback) -> None:
        return None


def fake_text_line(text: str, bbox: tuple[float, float, float, float]) -> dict:
    """Build a PyMuPDF dict text-line entry for layout matching tests."""
    return {"bbox": bbox, "spans": [{"text": text}]}


def fake_text_block(lines: list[dict]) -> dict:
    """Build a PyMuPDF dict text block containing fake line entries."""
    return {"type": 0, "lines": lines}


def test_extract_pdf_numbered_lines_uses_layout_for_libreoffice(tmp_path, monkeypatch) -> None:
    """LibreOffice PDFs list line numbers after body text, so pair by y coordinate first."""
    pdf = tmp_path / "line-source.pdf"
    pdf.write_bytes(b"%PDF")
    document = FakePdfDocument(
        [
            FakePdfPage(
                text="Body text A\nBody text B\n10\n11\n",
                blocks=[
                    fake_text_block(
                        [
                            fake_text_line("Body text A", (54.0, 100.0, 250.0, 112.0)),
                            fake_text_line("Body text B", (54.0, 120.0, 250.0, 132.0)),
                            fake_text_line("Footnote text", (54.0, 700.0, 250.0, 712.0)),
                            fake_text_line("10", (24.0, 100.0, 40.0, 112.0)),
                            fake_text_line("11", (24.0, 120.0, 40.0, 132.0)),
                            fake_text_line("1", (24.0, 700.0, 40.0, 712.0)),
                        ]
                    )
                ],
            )
        ],
        {"producer": "LibreOffice 25.2"},
    )
    monkeypatch.setitem(sys.modules, "fitz", SimpleNamespace(open=lambda _: document))

    assert reply_build.extract_pdf_numbered_lines(pdf) == [
        (10, 1, "Body text A"),
        (11, 1, "Body text B"),
    ]


def test_extract_pdf_numbered_lines_prefers_layout_for_word(tmp_path, monkeypatch) -> None:
    """Microsoft Word PDFs try layout matching before the text-order fallback."""
    pdf = tmp_path / "line-source.pdf"
    pdf.write_bytes(b"%PDF")
    document = FakePdfDocument(
        [
            FakePdfPage(
                text="Text-order body\n99\n",
                blocks=[
                    fake_text_block(
                        [
                            fake_text_line("Layout body", (54.0, 100.0, 250.0, 112.0)),
                            fake_text_line("10", (24.0, 100.0, 40.0, 112.0)),
                        ]
                    )
                ],
            )
        ],
        {"producer": "Microsoft® Word for Microsoft 365"},
    )
    monkeypatch.setitem(sys.modules, "fitz", SimpleNamespace(open=lambda _: document))

    assert reply_build.extract_pdf_numbered_lines(pdf) == [(10, 1, "Layout body")]


def test_extract_pdf_numbered_lines_falls_back_for_word_when_layout_empty(tmp_path, monkeypatch) -> None:
    """Microsoft Word PDFs use the old text-order logic when layout matching finds no lines."""
    pdf = tmp_path / "line-source.pdf"
    pdf.write_bytes(b"%PDF")
    document = FakePdfDocument(
        [
            FakePdfPage(
                text="Text-order body\n99\n",
                blocks=[fake_text_block([fake_text_line("Text-order body", (54.0, 100.0, 250.0, 112.0))])],
            )
        ],
        {"producer": "Microsoft® Word for Microsoft 365"},
    )
    monkeypatch.setitem(sys.modules, "fitz", SimpleNamespace(open=lambda _: document))

    assert reply_build.extract_pdf_numbered_lines(pdf) == [(99, 1, "Text-order body")]


def test_extract_pdf_numbered_lines_never_falls_back_for_libreoffice(tmp_path, monkeypatch) -> None:
    """LibreOffice PDFs rely only on layout matching because text order separates numbers."""
    pdf = tmp_path / "line-source.pdf"
    pdf.write_bytes(b"%PDF")
    document = FakePdfDocument(
        [
            FakePdfPage(
                text="Text-order body\n99\n",
                blocks=[fake_text_block([fake_text_line("Text-order body", (54.0, 100.0, 250.0, 112.0))])],
            )
        ],
        {"producer": "LibreOffice 25.2"},
    )
    monkeypatch.setitem(sys.modules, "fitz", SimpleNamespace(open=lambda _: document))

    assert reply_build.extract_pdf_numbered_lines(pdf) == []


def template_manuscript_pdf_path() -> Path:
    """Return the generated template/manuscript.md PDF used for extraction regression tests."""
    return Path("tests/fixtures/template-manuscript.pdf")


def expected_template_line_anchors() -> list[tuple[int, str]]:
    """Return stable anchors sampled from the generated template/manuscript.md PDF."""
    return [
        (16, "coherent narrative that guides readers from the general context"),
        (45, "caption attributes above are applied to the DOCX table"),
        (68, "procedure is useful. In this template, pseudocode"),
        (75, "synthetic trend chart in Figure 1"),
        (140, "authors declare no conflict of interest"),
    ]


def assert_extracted_line_anchors(numbered_lines: list[tuple[int, int, str]]) -> None:
    """Assert expected template PDF anchors are present at their sampled line numbers."""
    for expected_line, anchor in expected_template_line_anchors():
        matches = [line for line, _, text in numbered_lines if anchor in text]
        assert matches == [expected_line]


def test_template_manuscript_pdf_line_extractors_agree_on_line_regex_anchors() -> None:
    """Use the generated template PDF to verify both line-number extraction methods."""
    fitz = pytest.importorskip("fitz")
    pdf = template_manuscript_pdf_path()
    if not pdf.exists():
        pytest.skip(f"Build template/manuscript.md PDF first: {pdf}")

    with fitz.open(pdf) as document:
        layout_lines = reply_build.extract_pdf_numbered_lines_by_layout(document)
        text_order_lines = reply_build.extract_pdf_numbered_lines_by_text_order(document)

    assert_extracted_line_anchors(layout_lines)
    assert_extracted_line_anchors(text_order_lines)


def test_resolve_line_regexes_with_generated_template_manuscript_pdf(monkeypatch) -> None:
    """Resolve representative Line regexes against the generated template/manuscript.md PDF."""
    pytest.importorskip("fitz")
    line_source = template_manuscript_pdf_path()
    if not line_source.exists():
        pytest.skip(f"Build template/manuscript.md PDF first: {line_source}")
    monkeypatch.setattr(reply_build, "prepare_line_source_pdf", lambda path: path)

    markdown = "\n".join(
        [
            "Intro starts at (Line `coherent narrative that guides readers from the general context\\s+to your specific research`).",
            "Caption note at (Line `caption attributes above are applied to the DOCX table`).",
            "Pseudocode note at (Line `procedure is useful\\. In this template, pseudocode.*normal table`).",
            "Figure note at (Line `synthetic trend chart in Figure 1`).",
            "Disclosure note at (Line `authors declare no conflict of interest`).",
        ]
    )

    resolved = reply_build.resolve_line_regexes(markdown, line_source)

    assert "Intro starts at (Line 16)." in resolved
    assert "Caption note at (Line 45)." in resolved
    assert "Pseudocode note at (Line 68)." in resolved
    assert "Figure note at (Line 75)." in resolved
    assert "Disclosure note at (Line 140)." in resolved
