from pathlib import Path
import shutil
import subprocess

import pytest


def test_table_metadata_filter_keeps_uncaptioned_table_attributes() -> None:
    """Keep metadata for tables that intentionally omit visible captions."""
    pandoc = shutil.which("pandoc")
    if pandoc is None:
        pytest.skip("pandoc is not installed")

    filter_path = Path(__file__).resolve().parents[1] / "pandoc" / "filters" / "table_metadata.lua"
    markdown = """\
| **Algorithm: Demo** |
|---|
| Step |
: {revision_rows="*"}
"""

    result = subprocess.run(
        [pandoc, "--lua-filter", str(filter_path), "-f", "markdown", "-t", "native"],
        input=markdown,
        text=True,
        capture_output=True,
        check=True,
    )

    assert "PMT_TABLE_METADATA:" in result.stdout
    assert '\\"revision_rows\\":\\"*\\"' in result.stdout
