# Pandoc Manuscript Template

A professional, reusable template for academic manuscripts supporting both DOCX and LaTeX/PDF output formats using [Pandoc](https://pandoc.org/).

## Features

- **Dual-format output**: Generate both DOCX (for submission) and LaTeX/PDF (for archival)
- **Automatic formatting**: Consistent styling using reference documents and templates
- **Cross-references**: Automatic numbering and linking for figures, tables, equations, and sections
- **Flexible citations**: Support for 9000+ citation styles via CSL
- **Journal-ready**: Examples for common journal document classes (Elsevier, Springer, Wiley, IEEE)
- **Reproducible**: Version-controlled workflow with Make-based builds

## Quick Start

### Prerequisites

Install the following tools:

1. **Pandoc** (>= 3.0): [Download](https://pandoc.org/installing.html)
2. **Python** (>= 3.8): For Pandoc filters
3. **LaTeX distribution** (optional, for PDF output):
   - Windows: [MiKTeX](https://miktex.org/) or [TeX Live](https://www.tug.org/texlive/)
   - macOS: [MacTeX](https://www.tug.org/mactex/)
   - Linux: `sudo apt-get install texlive-full`
4. **Make** (optional but recommended):
   - Windows: Install via [Chocolatey](https://chocolatey.org/) (`choco install make`) or use WSL
   - macOS/Linux: Pre-installed

Install Python dependencies:

```bash
pip install panflute
```

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
   # Output: output/docx/manuscript.docx
   ```

4. **Generate PDF** (requires LaTeX):
   ```bash
   make pdf
   # Output: output/manuscript.pdf
   ```

5. **View available commands**:
   ```bash
   make help
   ```

## Usage Guide

### Writing Your Manuscript

1. **Edit `manuscript.md`**: Replace template content with your research
2. **Add images**: Place figures in `examples/images/` or create your own `images/` directory
3. **Manage references**: Edit `examples/references/sample-references.bib` or use your own `.bib` file

### Customizing for Different Journals

The template supports multiple journal formats. Edit the YAML header in `manuscript.md`:

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

When generating DOCX output with `make docx`, three PowerShell scripts automatically enhance table formatting:

#### 1. Table Metadata (process-table-metadata.ps1)

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

#### 2. Cell Merging (merge-table-cells.ps1)

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

#### 3. Auto-fit Tables (autofit-tables.ps1)

All tables are automatically fitted to window width and centered. This can be overridden using the `autofit` metadata key.

**Post-processing scripts location**: `scripts/`
- `process-table-metadata.ps1` - Applies metadata from captions
- `merge-table-cells.ps1` - Merges cells based on markers
- `autofit-tables.ps1` - Auto-fits tables to window

These scripts run automatically when `ENABLE_DOCX_POSTPROCESS = true` in the Makefile (Windows only, requires Microsoft Word).

## File Organization

```
pandoc-manuscript-template/
├── manuscript.md              # Your manuscript content (EDIT THIS)
├── Makefile                   # Build automation
├── README.md                  # This file
│
├── output/                    # Generated files (gitignored)
│   ├── docx/
│   └── latex/
│
├── examples/                  # Example content and references
│   ├── images/                # Sample figures
│   └── references/
│       ├── sample-references.bib      # Generic sample bibliography
│       └── paper-specific-example.bib # Advanced example
│
├── pandoc/                    # Pandoc infrastructure
│   ├── pandoc-docx.yml        # DOCX output configuration
│   ├── pandoc-latex.yml       # LaTeX/PDF output configuration
│   ├── filters/               # Python filters for document processing
│   │   ├── emf_to_pdf.py     # Convert EMF images to PDF
│   │   ├── resource_move.py  # Copy resources to output directory
│   │   ├── path_filter.py    # Adjust file paths
│   │   └── table_convert.py  # Table format conversion
│   ├── templates/             # LaTeX templates
│   │   ├── common.latex
│   │   ├── default.latex
│   │   └── fonts.latex
│   ├── manuscript-template/   # DOCX reference document (submodule)
│   ├── elsevier-vancouver.csl # Citation style (numeric)
│   └── engineering-applications-of-artificial-intelligence.csl
│
└── .vscode/                   # VS Code settings (optional)
```

## Build System

### Using Make (Recommended)

```bash
make docx          # Generate DOCX
make latex         # Generate LaTeX source only
make pdf           # Generate PDF via LaTeX
make dist          # Create distribution archive (PDF + LaTeX source)
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

# Generate PDF (from LaTeX)
cd output/latex
xelatex manuscript.tex
bibtex manuscript
xelatex manuscript.tex
xelatex manuscript.tex
```

## Advanced Customization

### Custom LaTeX Preamble

Edit `pandoc/templates/common.latex` to add custom LaTeX packages or commands.

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

## Troubleshooting

### Common Issues

**Error: "pandoc-crossref not found"**
- Install: Download from [releases](https://github.com/lierdakil/pandoc-crossref/releases)
- Or install via: `pip install pandoc-crossref` (if available for your platform)

**Error: "LaTeX Error: File 'xxx.sty' not found"**
- Install missing LaTeX package via your distribution's package manager
- MiKTeX: Packages auto-install on first use
- TeX Live: `tlmgr install <package-name>`

**Images not appearing in PDF**
- Ensure image paths are correct relative to `manuscript.md`
- Check that EMF files have corresponding PDF versions (or use PNG/JPG)

**Citations not rendering**
- Verify bibliography file path in YAML header
- Ensure `citeproc` filter is enabled
- Check citation keys match entries in `.bib` file

**DOCX formatting incorrect**
- Update reference document (`pandoc/manuscript-template/reference-doc.docx`)
- Ensure `reference-doc` path in `pandoc/pandoc-docx.yml` is correct

### Platform-Specific Notes

**Windows**:
- Use PowerShell or WSL for best compatibility
- Ensure Pandoc and Python are in system PATH
- Consider using [Chocolatey](https://chocolatey.org/) for package management

**macOS**:
- Use Homebrew to install dependencies: `brew install pandoc`
- MacTeX provides complete LaTeX distribution

**Linux**:
- Install via package manager: `sudo apt-get install pandoc texlive-full`
- May need to install `make` separately: `sudo apt-get install build-essential`

## Journal Submission Checklist

- [ ] Update title, authors, and affiliations in YAML header
- [ ] Replace abstract and keywords
- [ ] Write manuscript content in `manuscript.md`
- [ ] Add figures to appropriate directory and reference in text
- [ ] Create/update bibliography file with all references
- [ ] Configure journal-specific document class in YAML header
- [ ] Select appropriate citation style (CSL file)
- [ ] Generate DOCX: `make docx`
- [ ] Review output in Word/LibreOffice
- [ ] Generate PDF: `make pdf`
- [ ] Verify all figures, tables, and references appear correctly
- [ ] Run journal-specific formatting checks (line numbers, anonymization, etc.)

## IDE Integration

### Cursor IDE

Example Cursor IDE configurations are available in `examples/cursor-configs/` for users who want academic writing assistance. These provide templates for:
- Pandoc Markdown syntax guidance
- Academic writing style enforcement
- Bilingual content support

### VS Code

The template includes `.vscode/settings.json` with recommended settings:
- Word wrap enabled for Markdown files
- GitLens annotations configured for document history

### Other IDEs

The template uses standard Pandoc Markdown syntax and is compatible with any text editor.

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

## Contributing

Contributions are welcome! Please:
1. Fork the repository
2. Create a feature branch
3. Submit a pull request with clear description of changes

## License

[Specify your license here - MIT, Apache 2.0, CC-BY, etc.]

## Acknowledgments

- [Pandoc](https://pandoc.org/) - Universal document converter
- [pandoc-crossref](https://github.com/lierdakil/pandoc-crossref) - Cross-reference filter
- [manuscript-template](https://github.com/rnwest/pandoc-manuscript-template) - DOCX reference document tools

## Support

For issues and questions:
- Check the [Troubleshooting](#troubleshooting) section
- Review [Pandoc documentation](https://pandoc.org/MANUAL.html)
- Create an issue with minimal reproducible example

---

**Version**: 1.0.0
**Last Updated**: 2025-01-09
