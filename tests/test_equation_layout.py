from pathlib import Path
import sys

import yaml

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "src"))

from pandoc_manuscript.commands import build
from pandoc_manuscript.commands.build_reply import resolve as reply_resolve
from pandoc_manuscript.docx.equation_layout import (
    equation_tab_stops_from_metadata,
    sync_eqn_block_template_with_page_margins,
)


EQN_TEMPLATE = (
    '<w:pPr><w:tabs>'
    '<w:tab w:val="center" w:leader="none" w:pos="4888" />'
    '<w:tab w:val="right" w:leader="none" w:pos="9746" />'
    "</w:tabs></w:pPr>"
)


def test_equation_tab_stops_follow_docx_page_margins() -> None:
    """Derive equation tab stops from the DOCX writable text width."""
    metadata = {
        "docxPageMargins": {
            "left": "3.17cm",
            "right": "3.17cm",
        }
    }

    assert equation_tab_stops_from_metadata(metadata) == (4156, 8312)


def test_eqn_block_template_sync_updates_openxml_positions() -> None:
    """Rewrite center/right w:pos values while preserving the surrounding template."""
    synced, tab_stops = sync_eqn_block_template_with_page_margins(
        {
            "docxPageMargins": {
                "left": "3.17cm",
                "right": "3.17cm",
            },
            "eqnBlockTemplate": EQN_TEMPLATE,
        }
    )

    assert tab_stops == (4156, 8312)
    assert 'w:val="center" w:leader="none" w:pos="4156"' in synced["eqnBlockTemplate"]
    assert 'w:val="right" w:leader="none" w:pos="8312"' in synced["eqnBlockTemplate"]


def test_build_writes_adjusted_docx_metadata_file(tmp_path, monkeypatch) -> None:
    """Pass Pandoc a generated metadata file with margin-synced equation tabs."""
    monkeypatch.chdir(tmp_path)
    Path("style.yml").write_text("placeholder: true\n", encoding="utf-8")

    args = build.style_metadata_args(
        {
            "docxPageMargins": {"left": "3.17cm", "right": "3.17cm"},
            "eqnBlockTemplate": EQN_TEMPLATE,
        }
    )

    metadata = yaml.safe_load(Path(args[1]).read_text(encoding="utf-8"))
    assert args[0] == "--metadata-file"
    assert Path(args[1]).parts[:3] == (".pmt", "work", "metadata")
    assert 'w:pos="4156"' in metadata["eqnBlockTemplate"]
    assert 'w:pos="8312"' in metadata["eqnBlockTemplate"]


def test_reply_labeled_equation_tabs_follow_metadata_margins() -> None:
    """Use margin-synced tab stops for resolved reply-side labeled equations."""
    markdown = "$$ a+b $$ {#eq:sum}"

    resolved = reply_resolve.replace_labeled_equation_blocks(
        markdown,
        {"eq:sum": "Equation 7"},
        {"docxPageMargins": {"left": "3.17cm", "right": "3.17cm"}},
    )

    assert 'w:pos="4156"' in resolved
    assert 'w:pos="8312"' in resolved
    assert "(7)" in resolved
