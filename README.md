# Pandoc Manuscript Template

A professional, reusable template for academic manuscripts focused on DOCX output, with optional LaTeX source generation using [Pandoc](https://pandoc.org/).

## Features

- **DOCX-focused output**: Generate Word-ready manuscripts for journal submission
- **Automatic formatting**: Consistent styling using reference documents and templates
- **Cross-references**: Automatic numbering and linking for figures, tables, equations, and sections
- **Flexible citations**: Support for 9000+ citation styles via CSL
- **Journal-ready DOCX workflow**: Reference-document styling and post-processing for submission files
- **Installable CLI**: Use `pmt` directly after package installation or through `uvx`
- **Reproducible**: Version-controlled workflow with CLI, Python

## Quick Start

### Prerequisites

Install the following tools:

1. **Pandoc** (>= 3.0): [Download](https://pandoc.org/installing.html)
2. **pandoc-crossref**: Required for figure, table, equation, and section references
3. **UV**: Recommended for running the `pmt` CLI and Python filters

### Generate Your First Document

1. **Create a manuscript project with `pmt`**:
   ```bash
   uvx --from git+https://github.com/yourname/pandoc-manuscript-template pmt init my-paper
   cd my-paper
   ```

   When the package is installed as a tool, use:
   ```bash
   uv tool install pandoc-manuscript-template
   pmt init my-paper
   ```

2. **Check your environment**:
   ```bash
   pmt doctor
   ```

3. **Generate DOCX**:
   ```bash
   pmt build docx
   # Output: output/docx/manuscript.docx
   ```

   To build a different markdown file without editing the Pandoc defaults:
   ```bash
   pmt build docx paper.md
   # Output: output/docx/paper.docx
   ```

4. **View available commands**:
   ```bash
   pmt --help
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

### Output Style Metadata

Style-oriented metadata lives in `style.yml` so the manuscript YAML header can stay focused on the paper itself. During `pmt build docx`, `pmt build latex`, the build loads `style.yml` before `manuscript.md`; any field already defined in the manuscript YAML header overrides the style default.

If you want to change style-related content in a generated manuscript project, edit `style.yml`. The top-level keys cover the normal manuscript build, while the optional `reply:` section stores reply-specific overrides used by `pmt build reply`. This includes the CSL citation style, reference title, citation-link behavior, cross-reference labels and prefixes, section/equation numbering behavior, subfigure layout options, and DOCX body text formatting.

Collapsed numeric citation ranges can use a journal-specific delimiter after Pandoc citeproc renders them. Set `citation-number-range-delimiter` in `style.yml`, or override it in the manuscript YAML header:

```yaml
citation-number-range-delimiter: "-"  # [1-3]
```

To add a space after commas between non-consecutive numeric citations, edit the active CSL file's citation layout delimiter. For example, in `pandoc/csl/elsevier-vancouver.csl`, change:

```xml
<layout prefix="[" suffix="]" delimiter=",">
```

to:

```xml
<layout prefix="[" suffix="]" delimiter=", ">
```

This changes citations such as `[1,3]` to `[1, 3]`. It does not control collapsed ranges such as `[1-3]`, which are handled by `citation-number-range-delimiter`.

The DOCX post-processing step can update paragraph styles from the merged YAML metadata. Add style names under `docxStyle`; each key is matched against an existing DOCX style name, and missing styles are reported as warnings without stopping the build. The default template uses a two-character first-line indent and no spacing before or after body paragraphs:

Common Chinese built-in names such as `标题 1`, `正文文本`, and `正文` are automatically mapped to the corresponding Word built-in style names like `Heading 1`, `Body Text`, and `Normal`. Custom styles still need to use their exact DOCX style names.

```yaml
docxStyle:
  正文文本:
    firstLineIndentChars: 2
    paragraphSpacing:
      before: 0pt
      after: 0pt
```

Use point values for paragraph spacing, such as `6pt`. The first-line indent is written as a Word character-based indent, so `2` means two characters rather than a fixed centimeter or inch value. Fields that are omitted from a style block are left unchanged in the DOCX style.

Common style fields under `docxStyle` include:

- `fontSize`: font size such as `10.5pt` or Chinese Word sizes like `小五` and `四号`
- `fontColor`: font color such as `#000000` or `rgb(0, 0, 0)`
- `lineSpacing`: paragraph line spacing such as `1.5` or `18pt`
- `alignment`: `left`, `center`, `right`, or `justify`
- `firstLineIndentChars`: Word character-based first-line indent
- `indentation`: length-based `left`, `right`, `firstLine`, or `hanging` indent values such as `0.5cm`
- `paragraphSpacing`: `before` and `after` spacing values such as `6pt`

### Optional LaTeX Source Configuration

The primary workflow is DOCX generation. If you also generate LaTeX source, you can edit the YAML header in `manuscript.md` for document-class-specific output:

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
3. **Update `style.yml`**:
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

### Writing Pseudocode

For method or workflow descriptions, the recommended pattern is to write pseudocode as a one-column pipe table. This format is easy to edit in Markdown and stays visually stable after DOCX conversion.

**Recommended conventions**:

- Use a bold first row for the algorithm title, for example `| **Algorithm: ...** |`.
- Use bold label rows such as `**Input:**`, `**Output:**`, and `**Step 1 ...:**` to separate major blocks.
- Put one operation in each table row so the procedure stays readable in both Markdown and DOCX.
- Write control keywords in bold, such as `**for**`, `**if**`, `**else**`, `**end for**`, and `**end if**`.
- Use inline math with `$...$` for symbols and variables, and use `@eq:label` when the pseudocode refers to numbered equations in the manuscript.
- Inside table cells, use escaped spaces such as `\ \ ` to show nesting. This is useful because ordinary leading spaces in Markdown tables may collapse during rendering.
- If line numbers are needed, prefix each operation row with `1.\ \`, `2.\ \`, and so on. For nested operations, add more escaped spaces after the line number, for example `4.\ \ \ \ Train ...`.

Example:

```markdown
| **Algorithm: Library book borrowing workflow** |
|---|
| **Input:** |
| Borrow request list $R=\{r_i \mid i=1,\cdots,N\}$ and catalog records $C$ |
| **Output:** |
| Updated borrowing log $L$ |
| **Step 1 Validation:** |
| Read the next request $r_i$ and extract the member ID and book ID |
| Check whether the member account is active |
| **Step 2 Availability check:** |
| **for** each request $r_i$ in $R$ **do** |
| \ \ Search the catalog record for the requested book |
| \ \ **if** a copy is available **then** |
| \ \ \ \ Mark the copy as borrowed |
| \ \ \ \ Set the due date according to the lending policy |
| \ \ **else** |
| \ \ \ \ Add the request to the waiting list |
| \ \ **end if** |
| **end for** |
| **Step 3 Logging:** |
| Write the transaction result to the borrowing log $L$ |
| **if** an overdue fine is triggered **then** |
| \ \ Notify the member and update the account balance |
| **end if** |
```

Numbered pseudocode rows use the same one-column table format:

```markdown
| **Algorithm: Dataset preparation and model evaluation workflow** |
|---|
| **Input:** Raw dataset $D$, model family $M$, evaluation metric $s$ |
| **Output:** Trained model $\hat{m}$ and evaluation score $\hat{s}$ |
| 1.\ \ Clean and normalize all records in $D$ |
| 2.\ \ Split $D$ into training, validation, and test subsets |
| 3.\ \ **for** each candidate model $m \in M$ **do** |
| 4.\ \ \ \ Train $m$ on the training subset |
| 5.\ \ \ \ Tune hyperparameters using the validation subset |
| 6.\ \ **end for** |
| 7.\ \ Select the best model $\hat{m}$ according to validation performance |
| 8.\ \ Compute $\hat{s}$ for $\hat{m}$ on the test subset |
| 9.\ \ **return** $\hat{m}$ and $\hat{s}$ |
```

This pseudocode style is currently implemented as a normal table, not as a dedicated `algorithm` float. As a result, the template does not currently support cross-references.

### Citations

Use standard Pandoc citation syntax:

- Single citation: `[@smith2023]`
- Multiple citations: `[@smith2023; @jones2024]`
- Narrative citation: `@smith2023 showed that...`
- With page numbers: `[@smith2023, p. 42]`

### Advanced Table Formatting (DOCX Post-Processing)

When generating DOCX output, three post-processing scripts automatically enhance table formatting:

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

**Post-processing modules location**: `src/pandoc_manuscript/postprocess/`
- `process_table_metadata.py` - Applies metadata from captions
- `merge_table_cells.py` - Merges cells based on markers
- `autofit_tables.py` - Auto-fits tables to window

These modules run automatically during `pmt build docx` and `pmt build reply` when DOCX post-processing is enabled.

## Build System

### Using pmt (Recommended)

The package CLI is the preferred entry point for new projects. It can initialize
a manuscript directory, check external tools, and run the existing Pandoc build
pipeline.

```bash
pmt init my-paper     # Create a manuscript project
pmt doctor            # Check Pandoc, pandoc-crossref, Python dependencies, and project files
pmt build docx        # Generate output/docx/manuscript.docx
pmt build latex       # Generate output/latex/manuscript.tex
pmt build json        # Generate output/json/manuscript.json
pmt clean             # Remove generated files
```

Use `pmt build` for non-default inputs:

```bash
pmt build docx paper.md -o build
pmt build latex paper.md
```

For DOCX output, pass `--reference-doc custom-reference.docx` to override the
bundled Word reference document. The option is supported by both `docx` and
`reply` targets.

Reviewer replies can be built with the same DOCX pipeline. The `reply` target
resolves manuscript cross-references and citations against the manuscript before
converting the reply letter:

```bash
pmt build reply reply.md \
  --reply-manuscript manuscript.md \
  --output-file output/docx/reply.docx
```

If no reply markdown path is supplied, the reply target first looks for
`submissions/dbe/reply_to_reviewers_first.md`, then falls back to `reply.md`.
The reply build reads its reply-specific defaults from the `reply:` section in
`style.yml`, while `--manuscript-line-source` defaults to
`output/docx/manuscript.docx`. The line source is only read when the reply uses
``(Line `regex`)`` placeholders.

```

### Command Options

The `pmt build` command can build a markdown file specified on the command
line. When a markdown file is supplied, the output file name is derived from
that file's stem.

```bash
pmt build docx              # Generate output/docx/manuscript.docx
pmt build latex             # Generate output/latex/manuscript.tex
pmt build json              # Generate output/json/manuscript.json
pmt build docx paper.md     # Generate output/docx/paper.docx
pmt build latex paper.md    # Generate output/latex/paper.tex
pmt build json paper.md     # Generate output/json/paper.json
```

You can also pass the markdown path with `--manuscript` or `-m`:

```bash
pmt build docx --manuscript paper.md
pmt build latex -m paper.md
```

Use `--output-dir` or `-o` to change the base output directory:

```bash
pmt build docx paper.md --output-dir build  # Generate build/docx/paper.docx
pmt build latex paper.md -o build           # Generate build/latex/paper.tex
pmt build json paper.md -o build            # Generate build/json/paper.json
pmt clean --output-dir build                # Remove build/
```

The DOCX post-processing step reads YAML metadata from the same markdown file.
When an output directory is supplied, `docx`, `latex`, and `json`
subdirectories are created under it.

### Direct Pandoc Commands

```bash
# Generate DOCX
pandoc --metadata-file style.yml --defaults pandoc/pandoc-docx.yml

# Generate LaTeX
pandoc --metadata-file style.yml --defaults pandoc/pandoc-latex.yml
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
