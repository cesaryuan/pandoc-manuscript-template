# DOCX Post-Processing Scripts

This directory contains Python scripts for post-processing generated DOCX files. These scripts are cross-platform and do not require Microsoft Word.

## Files

- `postprocess_docx.py` - Main orchestrator script
- `merge_table_cells.py` - Merge table cells based on markers
- `process_table_metadata.py` - Apply table metadata from captions
- `autofit_tables.py` - Auto-fit tables to window width

## Quick Start

**Prerequisites:**
```bash
# Install uv (if not already installed)
# Windows:
powershell -ExecutionPolicy ByPass -c "irm https://astral.sh/uv/install.ps1 | iex"

# macOS/Linux:
curl -LsSf https://astral.sh/uv/install.sh | sh
```

**Usage:**
```bash
# Process a DOCX file (UV automatically handles dependencies)
uv run scripts/postprocess_docx.py output/docx/manuscript.docx

# Or run individual scripts
uv run scripts/merge_table_cells.py output/docx/manuscript.docx
uv run scripts/process_table_metadata.py output/docx/manuscript.docx
uv run scripts/autofit_tables.py output/docx/manuscript.docx
```

## How It Works

The Python scripts use the `python-docx` library to manipulate DOCX files directly:

1. **No Microsoft Word required** - Works on Windows, macOS, and Linux
2. **Direct XML manipulation** - Modifies the DOCX internal structure
3. **Processing steps:**
   - Merges table cells based on markers (`<<` or `!<!` for left, `^^` or `!^!` for up)
   - Applies table metadata from captions (margins, alignment, row height, etc.)
   - Auto-fits tables to window width (100%)
   - Centers tables on page

**Limitations:**
- Cannot update fields/cross-references (must be done manually in Word)
- Limited table style support compared to Word COM API

**Advantages:**
- ✅ Cross-platform (Windows, macOS, Linux)
- ✅ No Microsoft Word installation needed
- ✅ Faster execution (no Word startup overhead)
- ✅ Can run on servers and CI/CD pipelines
- ✅ Open source dependencies

## Configuration

Edit the top of `Makefile` to enable/disable post-processing:

```makefile
# DOCX post-processing
ENABLE_DOCX_POSTPROCESS = true    # Set to false to disable
```

## Features and Syntax

### Installation

**Option 1: UV (Recommended)**
```bash
# Install uv package manager
# Windows:
powershell -ExecutionPolicy ByPass -c "irm https://astral.sh/uv/install.ps1 | iex"

# macOS/Linux:
curl -LsSf https://astral.sh/uv/install.sh | sh

# No additional installation needed - UV handles dependencies automatically
```

**Option 2: Traditional pip**
```bash
# Install dependencies manually
pip install python-docx>=1.1.0
```

### Features and Syntax

#### 1. Merge Table Cells

**Markers:**
- `<<` or `!<!` - Merge cell with the one to its left
- `^^` or `!^!` - Merge cell with the one above

**Example Markdown:**
```markdown
| Header 1 | Header 2 | Header 3 |
|----------|----------|----------|
| A        | <<       | C        |
| D        | E        | F        |
| ^^       | ^^       | G        |

: Table with merged cells
```

**Result:**
- Row 1: Cells "Header 1" and "Header 2" are merged
- Row 3: Cells in columns 1 and 2 are merged with row 2

**Usage:**
```bash
uv run scripts/merge_table_cells.py output/docx/manuscript.docx
```

#### 2. Process Table Metadata

**Metadata Format:** `|key=value key2=value2|` at the end of table caption

**Supported Metadata Keys:**

| Key | Values | Example | Description |
|-----|--------|---------|-------------|
| `cell_margin` | `0.1cm`, `5pt`, `0.5in` | `cell_margin=0.1cm` | Set all cell margins |
| `cell_margin_top` | Same as above | `cell_margin_top=0.2cm` | Set top margin only |
| `cell_margin_bottom` | Same as above | `cell_margin_bottom=0.2cm` | Set bottom margin only |
| `cell_margin_left` | Same as above | `cell_margin_left=0.15cm` | Set left margin only |
| `cell_margin_right` | Same as above | `cell_margin_right=0.15cm` | Set right margin only |
| `row_height` | `1cm`, `20pt`, `0.5in` | `row_height=0.8cm` | Set row height for all rows |
| `alignment` | `left`, `center`, `right` | `alignment=center` | Set table alignment on page |
| `autofit` | `fixed`, `content`, `window` | `autofit=window` | Set autofit behavior |

**Example Markdown:**
```markdown
| Method | Accuracy | F1-Score |
|--------|----------|----------|
| Baseline | 78.3% | 77.8% |
| Proposed | 92.4% | 92.1% |

: Performance comparison. Best results in **bold**. |cell_margin=0.1cm alignment=center|
```

**After processing:** The metadata `|cell_margin=0.1cm alignment=center|` is removed from the caption, and the settings are applied to the table.

**Usage:**
```bash
uv run scripts/process_table_metadata.py output/docx/manuscript.docx
```

#### 3. Auto-fit Tables

Sets all tables to auto-fit to window width (100%) and centers them on the page.

**Usage:**
```bash
# With center alignment (default)
uv run scripts/autofit_tables.py output/docx/manuscript.docx

# Without center alignment
uv run scripts/autofit_tables.py output/docx/manuscript.docx --no-center
```

#### 4. Complete Pipeline

Runs all processing steps in sequence:

```bash
uv run scripts/postprocess_docx.py output/docx/manuscript.docx
```

**Processing order:**
1. Merge table cells (markers)
2. Apply table metadata (captions)
3. Auto-fit tables to window

### Advanced Usage

#### Use as Python Modules

```python
from docx import Document
from merge_table_cells import merge_table_cells
from process_table_metadata import process_table_metadata
from autofit_tables import autofit_tables

# Open document
doc = Document("manuscript.docx")

# Process
left, up = merge_table_cells(doc)
processed, settings = process_table_metadata(doc)
fitted = autofit_tables(doc)

# Save
doc.save("manuscript.docx")
```

#### Customize Processing

You can modify the Python scripts to add custom logic:

**Example: Skip certain tables**
```python
# In autofit_tables.py, modify the autofit_tables function:
for i, table in enumerate(doc.tables, start=1):
    # Skip first table (often a title or abstract table)
    if i == 1:
        continue

    set_table_autofit_window(table)
    set_table_center_alignment(table)
```

**Example: Apply different margins based on table size**
```python
# In process_table_metadata.py, add custom logic:
def apply_custom_margins(table):
    num_cols = len(table.columns)

    if num_cols > 5:
        # Wide tables: smaller margins
        set_cell_margins(table, top=Pt(2), bottom=Pt(2),
                        left=Pt(3), right=Pt(3))
    else:
        # Narrow tables: larger margins
        set_cell_margins(table, top=Pt(4), bottom=Pt(4),
                        left=Pt(6), right=Pt(6))
```

### Troubleshooting

#### Import Error: "No module named 'docx'"

**Solution 1 (UV):** Use UV to run the script (it handles dependencies automatically):
```bash
uv run scripts/postprocess_docx.py output/docx/manuscript.docx
```

**Solution 2 (pip):** Install python-docx manually:
```bash
pip install python-docx
```

Note: The package name is `python-docx`, but you import it as `docx`.

#### Script fails with "Failed to import processing modules"

**Cause:** The scripts need to be in the same directory or Python path.

**Solution:** Run from project root:
```bash
# Run from project root
cd /path/to/pandoc-manuscript-template
uv run scripts/postprocess_docx.py output/docx/manuscript.docx
```

#### Merged cells not working correctly

**Cause:** Complex table structures or pre-existing merges may interfere.

**Solution:**
1. Ensure markers are the ONLY content in cells
2. Process merges from simple to complex (left first, then up)
3. Check the original table structure in Markdown

#### Metadata not being applied

**Cause:** Caption style not recognized or metadata format incorrect.

**Solutions:**
1. Ensure caption has "Caption" or "题注" style in Word
2. Check metadata format: `|key=value key2=value2|` at the END of caption
3. Use correct units: `cm`, `mm`, `in`, `pt`
4. No spaces in key names: `cell_margin` not `cell margin`

#### Tables not centered after auto-fit

**Cause:** Document template may override table alignment.

**Solution:** Use metadata to explicitly set alignment:
```markdown
: Table caption |alignment=center|
```

### Comparison with Word COM API

| Feature | Python Scripts | Word COM API |
|---------|----------------|--------------|
| **Cross-platform** | ✅ Yes | ❌ Windows only |
| **Requires Word** | ❌ No | ✅ Yes |
| **Merge cells** | ✅ Full support | ✅ Full support |
| **Table metadata** | ✅ Full support | ✅ Full support |
| **Auto-fit tables** | ✅ Via XML | ✅ Native API |
| **Update fields** | ❌ Not supported | ✅ Supported |
| **Table styles** | ⚠️ Limited | ✅ Full support |
| **Performance** | ✅ Fast | ⚠️ Slower (Word startup) |
| **CI/CD friendly** | ✅ Yes | ⚠️ Windows only |
| **Server deployment** | ✅ Easy | ❌ Difficult |

**Recommendation:**
- Use **Python** for most workflows (cross-platform, no Word needed)
- If you need field updates or advanced Word features, update them manually in Word

## Requirements

**Python version:** 3.11 or later

**Dependencies (automatically managed by UV):**
- `python-docx >= 1.1.0` - DOCX manipulation library

**Package Manager:**
- `uv` - Fast Python package installer (recommended)
- Alternative: `pip` (manual dependency management)

### About UV Inline Scripts

All Python scripts in this directory use **PEP 723 inline script metadata** format. This means:

1. **Dependencies are declared in each script** - No separate `requirements.txt` needed
2. **UV automatically manages dependencies** - Just run `uv run script.py`
3. **Reproducible environments** - Each script specifies its exact dependencies
4. **No virtual environment needed** - UV handles isolation automatically

**Inline script metadata example:**
```python
#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = [
#   "python-docx>=1.1.0",
# ]
# ///
```

This approach provides:
- ✅ **Simpler workflow** - No separate dependency installation step
- ✅ **Better reproducibility** - Dependencies are versioned with the script
- ✅ **Faster execution** - UV is significantly faster than pip
- ✅ **Automatic caching** - Dependencies are cached and reused

## Security Note

The Python scripts only use the `python-docx` library, which is a widely-used open source package. Always review dependencies before installation.

## References

- [python-docx Documentation](https://python-docx.readthedocs.io/)
- [OOXML Specification](http://www.ecma-international.org/publications/standards/Ecma-376.htm)
- [Office Open XML Wikipedia](https://en.wikipedia.org/wiki/Office_Open_XML)
