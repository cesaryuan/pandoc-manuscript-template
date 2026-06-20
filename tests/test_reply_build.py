from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "src"))

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
