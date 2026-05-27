# Pandoc Manuscript Template

A professional, reusable template for academic manuscripts focused on DOCX output, with optional LaTeX source generation using [Pandoc](https://pandoc.org/).

## Features

- **DOCX-focused output**: Generate Word-ready manuscripts for journal submission
- **Automatic formatting**: Consistent styling using reference documents and templates
- **Cross-references**: Automatic numbering and linking for figures, tables, equations, and sections
- **Flexible citations**: Support for 9000+ citation styles via CSL
- **Journal-ready DOCX workflow**: Reference-document styling and post-processing for submission files
- **Reproducible**: Version-controlled workflow with Make-based builds

## Quick Start

### Prerequisites

Install the following tools:

1. **Pandoc** (>= 3.0): [Download](https://pandoc.org/installing.html)
2. **UV**: For Pandoc filters
3. **Make** (optional but recommended):
   - Windows: Install via [Chocolatey](https://chocolatey.org/) (`choco install make`) or use WSL
   - macOS/Linux: Pre-installed

### Generate Your First Document

1. **Clone this repository**:
   ```bash
   git clone <repository-url>
   cd pandoc-manuscript-template
   ```

2. **Initialize submodules**:
   ```bash
   git submodule update --init --recursive
   ```

3. **Generate DOCX**:
   ```bash
   make docx
   # or 
   uv run scripts/build.py docx
   # Output: output/docx/manuscript.docx
   ```

4. **View available commands**:
   ```bash
   make help
   ```

## Usage Guide

### Writing Your Manuscript

**Edit `manuscript.md`**: Replace template content with your research.

### Author Metadata

The DOCX post-processing step reads author information from the YAML header in `manuscript.md` and inserts formatted author names, affiliations, and the corresponding-author footnote after the title. Use `authors` as the preferred field name. The singular alias `author` is also accepted by the post-processor for compatibility.

Each author must be written as a YAML mapping with at least a `name` field. Optional fields are:

- `affiliation`: A single affiliation key or a single inline affiliation string.
- `affiliations`: One affiliation key/string or a list of affiliation keys/strings.
- `email`: Used in the generated corresponding-author footnote.
- `title`: Added in parentheses in the generated corresponding-author footnote.
- `corresponding`: Use `true` to generate the default correspondence footnote, or provide a string to use as the complete footnote text.

Affiliations can be written inline under each author, or defined once in a top-level `affiliations` map and then referenced by key. The singular top-level alias `affiliation` is also accepted when using keyed affiliations.

#### Format 1: Inline Single Affiliation

This is the simplest format and matches the default `manuscript.md` template:

```yaml
authors:
  - name: First Author
    email: first.author@university.edu
    affiliation: Department of Example, University Name, City, Country

  - name: Second Author
    email: second.author@university.edu
    affiliation: Department of Example, University Name, City, Country
    corresponding: true
```

#### Format 2: Inline Multiple Affiliations

Use `affiliations` as a list when an author has more than one affiliation. Repeated affiliation text is automatically assigned the same superscript label.

```yaml
authors:
  - name: First Author
    affiliations:
      - Department of Example, University Name, City, Country
      - Research Center, Institute Name, City, Country

  - name: Second Author
    affiliations:
      - Department of Example, University Name, City, Country
      - Research Center, Institute Name, City, Country
    corresponding: true
    email: second.author@university.edu
```

#### Format 3: Keyed Affiliations

Use a top-level `affiliations` map when several authors share the same institutions. Author entries can reference one key with `affiliation`, or several keys with `affiliations`.

```yaml
authors:
  - name: First Author
    affiliations: [a, b]

  - name: Second Author
    affiliation: a
    corresponding: true
    email: second.author@university.edu
    title: Professor

affiliations:
  a: Department of Example, University Name, City, Country
  b: Research Center, Institute Name, City, Country
```

#### Format 4: Custom Corresponding-Author Footnote

Set `corresponding` to a string when the journal requires custom wording. The string is used as the complete footnote text.

```yaml
authors:
  - name: First Author
    affiliation: a

  - name: Second Author
    affiliation: a
    corresponding: "Correspondence concerning this article should be addressed to Second Author, Department of Example, University Name. Email: second.author@university.edu."

affiliations:
  a: Department of Example, University Name, City, Country
```

Avoid Pandoc's compact string-only author syntax, such as `author: [First Author, Second Author]`, when you need this template's DOCX author formatting. The post-processing script expects each author to be a mapping so it can read affiliations and correspondence metadata.

### Optional LaTeX Source Configuration

The primary workflow is DOCX generation. If you also generate LaTeX source with `make latex`, you can edit the YAML header in `manuscript.md` for document-class-specific output:

#### Example 1: Elsevier Journal

Uncomment and customize this section in `manuscript.md`:

```yaml
documentclass: elsarticle
classoption: [preprint, 3p, authoryear]
header-includes:
- |
  ```{=latex}
  \journal{Journal of Example Research}
  ```
```

#### Example 2: Springer Journal

```yaml
documentclass: svjour3
classoption: [smallextended]
```

#### Example 3: Wiley Journal (e.g., Computer-Aided Civil Engineering)

```yaml
documentclass: WileyNJDv5
classoption: [HARVARD, Times2COL]
header-includes:
- |
  ```{=latex}
  \journal{Comput Aided Civ Inf.}
  \volume{00}
  \copyyear{2025}
  \startpage{1}
  \articletype{RESEARCH ARTICLE}
  ```
```

#### Example 4: IEEE Journal

```yaml
documentclass: IEEEtran
classoption: [journal]
```

### Changing Citation Styles

1. **Browse styles**: Visit [Zotero Style Repository](https://www.zotero.org/styles)
2. **Download CSL file**: Save to `pandoc/` directory
3. **Update `pandoc/pandoc-docx.yml`**:
   ```yaml
   csl: pandoc/your-style.csl
   ```

Common styles included:
- `elsevier-vancouver.csl`: Numeric citations (Vancouver style)
- `engineering-applications-of-artificial-intelligence.csl`: EA-AI journal style

### Cross-References

The template uses [pandoc-crossref](https://github.com/lierdakil/pandoc-crossref) for automatic numbering:

- **Figures**: `![Caption](image.png){#fig:label}` → Reference with `@fig:label`
- **Tables**: `Table: Caption {#tbl:label}` → Reference with `@tbl:label`
- **Equations**: `$$ equation $$ {#eq:label}` → Reference with `@eq:label`
- **Sections**: `# Section {#sec:label}` → Reference with `@sec:label`

Example:
```markdown
See @fig:results for details. As shown in @tbl:comparison and @eq:model...
```

### Citations

Use standard Pandoc citation syntax:

- Single citation: `[@smith2023]`
- Multiple citations: `[@smith2023; @jones2024]`
- Narrative citation: `@smith2023 showed that...`
- With page numbers: `[@smith2023, p. 42]`

### Advanced Table Formatting (DOCX Post-Processing)

When generating DOCX output with `make docx`, three post-processing scripts automatically enhance table formatting:

#### 1. Table Metadata

Add metadata to table captions to control table properties. The metadata is automatically removed from the final caption.

**Syntax**: Add `|key=value key2=value2|` at the end of the table caption.

**Available metadata keys**:
- `cell_margin=0.10cm` - Set all cell margins (supports cm, mm, in, pt)
- `cell_margin_top=0.10cm`, `cell_margin_bottom=0.10cm`, `cell_margin_left=0.10cm`, `cell_margin_right=0.10cm` - Individual margins
- `cell_spacing=0pt` - Spacing between cells
- `row_height=0.5cm` - Set row height for all rows
- `alignment=center` - Table alignment (left, center, right)
- `autofit=window` - Autofit behavior (fixed, content, window)

**Example**:
```markdown
| **Method** | **Accuracy (%)** |
|:----------:|:----------------:|
| Baseline   | 78.3             |
| Proposed   | 92.4             |

: Performance comparison. |cell_margin=0.10cm autofit=window alignment=center| {#tbl:results}
```

The metadata `|cell_margin=0.10cm autofit=window alignment=center|` will be applied to the table and then removed from the caption in the final DOCX.

#### 2. Cell Merging

Use special markers to merge table cells in the generated DOCX:

- `!<!` - Merge with the cell to the left
- `!^!` - Merge with the cell above

**Example**:
```markdown
| **Category** | **Subcategory** | **Value** |
|:------------:|:---------------:|:---------:|
| Group A      | Item 1          | 10        |
| !^!          | Item 2          | 20        |
| Group B      | Item 3          | 30        |

: Table with merged cells. {#tbl:merged}
```

In this example, "Group A" will span two rows (merging with the cell below containing `!^!`).

**Important notes**:
- Markers are processed and removed during DOCX generation
- Left merges (`!<!`) are processed first, then up merges (`!^!`)
- The marker cell must be empty except for the marker itself

#### 3. Auto-fit Tables

All tables are automatically fitted to window width and centered. This can be overridden using the `autofit` metadata key.

**Post-processing scripts location**: `scripts/`
- `process-table-metadata.ps1` - Applies metadata from captions
- `merge-table-cells.ps1` - Merges cells based on markers
- `autofit-tables.ps1` - Auto-fits tables to window

These scripts run automatically when `ENABLE_DOCX_POSTPROCESS = true` in the Makefile (Windows only, requires Microsoft Word).

## Build System

### Using Make (Recommended)

```bash
make docx          # Generate DOCX
make latex         # Generate LaTeX source only
make clean         # Remove generated files
make help          # Show available commands
```

### Direct Pandoc Commands

If Make is not available:

```bash
# Generate DOCX
pandoc --defaults pandoc/pandoc-docx.yml

# Generate LaTeX
pandoc --defaults pandoc/pandoc-latex.yml
```

## Advanced Customization

### Custom LaTeX Preamble

Edit `pandoc/templates/common.latex` to add custom LaTeX packages or commands for the optional LaTeX source output.

### Custom DOCX Styling

1. Generate a reference document:
   ```bash
   pandoc -o custom-reference.docx --print-default-data-file reference.docx
   ```

2. Open `custom-reference.docx` in Word and modify styles

3. Update `pandoc/pandoc.yml`:
   ```yaml
   reference-doc: custom-reference.docx
   ```

See the [manuscript-template submodule](pandoc/manuscript-template/) for advanced DOCX customization.

### Adding New Filters

1. Create Python filter in `pandoc/filters/`
2. Add to filter list in `pandoc/pandoc-docx.yml` or `pandoc/pandoc-latex.yml`:
   ```yaml
   filters:
     - your_filter.py
   ```

## Journal Submission Checklist

- [ ] Update title, authors, and affiliations in YAML header
- [ ] Replace abstract and keywords
- [ ] Write manuscript content in `manuscript.md`
- [ ] Add figures to appropriate directory and reference in text
- [ ] Create/update bibliography file with all references
- [ ] Select appropriate citation style (CSL file)
- [ ] Generate DOCX: `make docx`
- [ ] Review output in Word/LibreOffice
- [ ] Verify all figures, tables, and references appear correctly
- [ ] Run journal-specific formatting checks (line numbers, anonymization, etc.)

## Examples

See `manuscript.md` for a complete example demonstrating:
- Multi-author affiliations with corresponding author
- Abstract and keywords
- Section organization (Introduction, Methods, Results, Discussion, Conclusion)
- Figure and table cross-references
- Mathematical equations with numbering
- Citations in various formats
- Acknowledgments and supplementary sections

For a real-world example of a complete research paper, see `examples/references/paper-specific-example.bib`.


## Acknowledgments

- [Pandoc](https://pandoc.org/) - Universal document converter
- [pandoc-crossref](https://github.com/lierdakil/pandoc-crossref) - Cross-reference filter

## Support

For issues and questions:
- Review [Pandoc documentation](https://pandoc.org/MANUAL.html)
- Create an issue with minimal reproducible example

---

**Version**: 1.0.0
**Last Updated**: 2025-01-09
