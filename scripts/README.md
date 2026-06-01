# DOCX Post-Processing Scripts

This directory contains Python scripts for post-processing generated DOCX files. These scripts are cross-platform and do not require Microsoft Word.

## Files

- `postprocess_docx.py` - Main orchestrator script
- `merge_table_cells.py` - Merge table cells based on markers
- `process_table_metadata.py` - Apply table metadata from captions
- `autofit_tables.py` - Auto-fit regular tables to window width
- `format_equation_layout_tables.py` - Format borders, margins, and column widths for equation layout tables
- `body_text_style.py` - Apply Body Text style settings from merged YAML metadata
- `inline_math_spacing.py` - Keep standalone inline math from rendering as display math in Word
- `where_paragraph_style.py` - Apply `Where Paragraph` style to where clauses after equation paragraphs or layout tables

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
uv run scripts/postprocess/format_equation_layout_tables.py output/docx/manuscript.docx
```

## How It Works

The Python scripts use the `python-docx` library to manipulate DOCX files directly:

1. **No Microsoft Word required** - Works on Windows, macOS, and Linux
2. **Direct XML manipulation** - Modifies the DOCX internal structure
3. **Processing steps:**
   - Merges table cells based on markers (`!<!` for left, `!^!` for up)
   - Applies Body Text paragraph style settings from merged YAML metadata
   - Applies table metadata from captions (margins, alignment, row height, etc.)
   - Auto-fits regular tables to window width (100%)
   - Centers tables on page
   - Hides borders on equation layout tables, removes the right margin from equation-number cells, and sets number columns to 0.6 cm for numbers below 10, otherwise 0.8 cm
   - Applies `Where Paragraph` style to `where` paragraphs immediately after equation paragraphs or layout tables
   - Adds a trailing space after standalone inline math paragraphs

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
- `!<!` - Merge cell with the one to its left
- `!^!` - Merge cell with the one above

**Example Markdown:**
```markdown
| Header 1 | Header 2 | Header 3 |
|----------|----------|----------|
| A        | !<!      | C        |
| D        | E        | F        |
| !^!      | !^!      | G        |

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

Sets regular tables to auto-fit to window width (100%) and centers them on the page. Equation layout tables are skipped so their alignment is preserved.

**Usage:**
```bash
# With center alignment (default)
uv run scripts/autofit_tables.py output/docx/manuscript.docx

# Without center alignment
uv run scripts/autofit_tables.py output/docx/manuscript.docx --no-center
```

#### 4. Format Equation Layout Tables

Hides all borders on equation layout tables, sets the right margin of the equation-number cell to zero, and sets the first and last columns to 0.6 cm for equation numbers below 10, otherwise 0.8 cm.

**Usage:**
```bash
uv run scripts/postprocess/format_equation_layout_tables.py output/docx/manuscript.docx
```

#### 5. Body Text Style Metadata

Set body paragraph indentation and spacing in `style.yml`, or override it in the manuscript YAML header:

```yaml
bodyText:
  firstLineIndentChars: 2
  paragraphSpacing:
    before: 0pt
    after: 0pt
```

The post-processor applies these settings to the DOCX `Body Text` style. Paragraph spacing values use points, and the first-line indent uses Word's character-based indent. When both `style.yml` and `manuscript.md` define a value, the manuscript value wins.

**Usage:**
```bash
uv run scripts/postprocess/body_text_style.py output/docx/manuscript.docx manuscript.md --metadata-file style.yml
```

#### 6. Complete Pipeline

Runs all processing steps in sequence:

```bash
uv run scripts/postprocess_docx.py output/docx/manuscript.docx
```

**Processing order:**
1. Insert author information from YAML metadata
2. Apply Body Text style settings from merged YAML metadata
3. Merge table cells (markers)
4. Apply table metadata (captions)
5. Clear subfigure table formatting
6. Convert table text style from `Compact` to `Table Text`
7. Auto-fit tables to window
8. Format equation layout tables
9. Apply `Where Paragraph` style after equation paragraphs or layout tables
10. Add trailing spaces after standalone inline math

### Advanced Usage

#### Use as Python Modules

```python
from docx import Document
from merge_table_cells import merge_table_cells
from process_table_metadata import process_table_metadata
from autofit_tables import autofit_tables
from format_equation_layout_tables import format_equation_layout_tables
from metadata import load_merged_metadata
from body_text_style import apply_body_text_style_metadata

# Open document
doc = Document("manuscript.docx")

# Process
metadata = load_merged_metadata("manuscript.md", ["style.yml"])
apply_body_text_style_metadata(doc, metadata)
left, up = merge_table_cells(doc)
processed, settings = process_table_metadata(doc)
fitted = autofit_tables(doc)
equation_tables = format_equation_layout_tables(doc)

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
