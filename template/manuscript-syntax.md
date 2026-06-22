# Manuscript Syntax

This document separates manuscript content syntax from style metadata. Use the
`Manuscript Syntax` section for content written in `manuscript.md`, and the
`Style Metadata` section for style-related defaults in `style.yml`.

## Author Metadata

The DOCX post-processing step reads author information from the YAML header in `manuscript.md` and inserts formatted author names, affiliations, and the corresponding-author footnote after the title. Use `authors` as the preferred field name. The singular alias `author` is also accepted by the post-processor for compatibility.

Each author must be written as a YAML mapping with at least a `name` field. Optional fields are:

- `affiliation`: A single affiliation key or a single inline affiliation string.
- `affiliations`: One affiliation key/string or a list of affiliation keys/strings.
- `email`: Used in the generated corresponding-author footnote.
- `title`: Added in parentheses in the generated corresponding-author footnote.
- `corresponding`: Use `true` to generate the default correspondence footnote, or provide a string to use as the complete footnote text.

Affiliations can be written inline under each author, or defined once in a top-level `affiliations` map and then referenced by key. The singular top-level alias `affiliation` is also accepted when using keyed affiliations.

### Format 1: Inline Single Affiliation

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

### Format 2: Inline Multiple Affiliations

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

### Format 3: Keyed Affiliations

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

### Format 4: Custom Corresponding-Author Footnote

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

## Optional LaTeX Source Configuration

The primary workflow is DOCX generation. If you also generate LaTeX source, you can edit the YAML header in `manuscript.md` for document-class-specific output:

### Example 1: Elsevier Journal

Uncomment and customize this section in `manuscript.md`:

````yaml
documentclass: elsarticle
classoption: [preprint, 3p, authoryear]
header-includes:
- |
  ```{=latex}
  \journal{Journal of Example Research}
  ```
````

### Example 2: Springer Journal

```yaml
documentclass: svjour3
classoption: [smallextended]
```

### Example 3: Wiley Journal (e.g., Computer-Aided Civil Engineering)

````yaml
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
````

### Example 4: IEEE Journal

```yaml
documentclass: IEEEtran
classoption: [journal]
```

## Cross-References

The template uses [pandoc-crossref](https://github.com/lierdakil/pandoc-crossref) for automatic numbering:

- **Figures**: `![Caption](image.png){#fig:label}` -> Reference with `@fig:label`
- **Tables**: `Table: Caption {#tbl:label}` -> Reference with `@tbl:label`
- **Equations**: `$$ equation $$ {#eq:label}` -> Reference with `@eq:label`
- **Sections**: `# Section {#sec:label}` -> Reference with `@sec:label`

Example:
```markdown
See @fig:results for details. As shown in @tbl:comparison and @eq:model...
```

## Marking Revisions in Red

Use Pandoc custom styles to mark substantive manuscript revisions in generated DOCX files. The default reference DOCX includes a character style named `Revision Char`, so revised inline text can be written as a bracketed span:

```markdown
The proposed workflow improves [the adaptive sampling stage]{custom-style="Revision Char"} while keeping the original preprocessing steps unchanged.
```

Recommended revision-marking rules:

- Use `Revision Char` only for modified or newly added words, phrases, sentences, or paragraphs that need to appear in red.
- Do not mark very small edits within one sentence, such as changes under three words.
- When old text is replaced, omit the deleted wording and mark only the new or modified surviving text.
- For heavily revised existing paragraphs, mark only the changed parts. Mark a whole paragraph only when the entire paragraph is newly added.
- Keep cross-reference tokens such as `@fig:result` or `@tbl:comparison` outside the styled span unless the reference text itself is substantively changed.

For a modified or newly added figure, mark the caption rather than the image path. Size-only figure changes do not need revision markup.

```markdown
![[Updated model comparison under the same evaluation protocol.]{custom-style="Revision Char"}](figures/model-comparison.png){#fig:model-comparison}
```

For tables, use revision attributes on the table caption. Use `revision-rows="*"` for a newly added table. For a modified table, list changed or added 1-based row or column numbers with `revision-rows="..."` and `revision-columns="..."`; row numbers include the table header row.

```markdown
| **Method** | **Accuracy (%)** | **Runtime (s)** |
|:----------:|:----------------:|:---------------:|
| Baseline   | 78.3             | 12.4            |
| Proposed   | 92.4             | 10.1            |

: Performance comparison. {#tbl:performance revision-columns="2,3" revision-rows="3"}
```

The underscore forms `revision_rows` and `revision_columns` are equivalent and are documented with the other DOCX table attributes below.

## Writing Pseudocode

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

## Citations

Use standard Pandoc citation syntax:

- Single citation: `[@smith2023]`
- Multiple citations: `[@smith2023; @jones2024]`
- Narrative citation: `@smith2023 showed that...`
- With page numbers: `[@smith2023, p. 42]`

## Advanced Table Formatting (DOCX Post-Processing)

When generating DOCX output, three post-processing scripts automatically enhance table formatting:

### 1. Table Attributes

Add standard Pandoc attributes to table captions to control DOCX table properties. Pandoc does not preserve arbitrary table attributes in the generated DOCX, so `pmt build docx` runs a Lua filter that embeds a hidden WordprocessingML marker before conversion. The DOCX post-processor reads the marker, applies the settings, and removes it before saving the final document.

**Syntax**: Add attributes at the end of the Pandoc table caption.
For tables that should not have a visible caption, use an attribute-only caption line such as `: {revision_rows="*"}`.

**Available attribute keys**:
- `cell_margin="0.10cm"` - Set all cell margins (supports cm, mm, in, pt)
- `cell_margin_top="0.10cm"`, `cell_margin_bottom="0.10cm"`, `cell_margin_left="0.10cm"`, `cell_margin_right="0.10cm"` - Individual margins
- `cell_spacing="0pt"` - Spacing between cells
- `row_height="0.5cm"` - Set row height for all rows
- `revision_rows="1,2,3"` - Mark changed or added 1-based rows in red text
- `revision_columns="6,7"` - Mark changed or added 1-based columns in red text
- `revision_rows="*"` or `revision_columns="*"` - Mark the entire table and its caption in red text
- `alignment="center"` - Table alignment (left, center, right)
- `autofit="window"` - Autofit behavior (fixed, content, window)

Revision row and column numbers are 1-based and include the table header row.

Hyphenated aliases such as `cell-margin="0.10cm"` and `revision-columns="6,7"` are also accepted.

**Example**:
```markdown
| **Method** | **Accuracy (%)** |
|:----------:|:----------------:|
| Baseline   | 78.3             |
| Proposed   | 92.4             |

: Performance comparison. {#tbl:results cell_margin="0.10cm" autofit="window" alignment="center" revision_rows="*"}
```

The attributes are applied to the DOCX table without appearing in the final caption.

### 2. Cell Merging

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

### 3. Auto-fit Tables

All tables are automatically fitted to window width and centered. This can be overridden using the `autofit` or `alignment` table attributes.

**Post-processing modules location**: `src/pandoc_manuscript/postprocess/`
- `process_table_metadata.py` - Applies metadata collected from Pandoc table attributes
- `merge_table_cells.py` - Merges cells based on markers
- `autofit_tables.py` - Auto-fits tables to window

These modules run automatically during `pmt build docx` and `pmt build-reply` when DOCX post-processing is enabled.

# Style Metadata

This section describes style-related defaults in `style.yml`. Keep paper
content and manuscript-specific metadata in `manuscript.md`; keep reusable
formatting, citation, cross-reference, and DOCX style defaults here.

## Output Style Metadata

Style-oriented metadata lives in `style.yml` so the manuscript YAML header can stay focused on the paper itself. During `pmt build docx`, `pmt build latex`, the build loads `style.yml` before `manuscript.md`; any field already defined in the manuscript YAML header overrides the style default.

If you want to change style-related content in a generated manuscript project, edit `style.yml`. The top-level keys cover the normal manuscript build, while the optional `reply:` section stores reply-specific overrides used by `pmt build-reply`. This includes the CSL citation style, reference title, citation-link behavior, cross-reference labels and prefixes, section/equation numbering behavior, subfigure layout options, and DOCX body text formatting.

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

For journal submission systems that reject SVG image files, enable DOCX-only
SVG rasterization in `style.yml`:

```yaml
docxConvertSvgToPng: true
# Optional rasterization controls:
docxSvgToPngDpi: 300
docxSvgToPngScale: 1
```

During `pmt build docx`, local Markdown image references ending in `.svg` or
`.svgz` are converted into PNG files under the pmt cache directory
(`.pmt/cache/svg-png/` by default), and the temporary Pandoc document uses those PNG paths. The
original Markdown file is not rewritten. The converter uses the Python `resvg-py` dependency.

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

## Changing Citation Styles

1. **Browse styles**: Visit [Zotero Style Repository](https://www.zotero.org/styles)
2. **Download CSL file**: Save to `pandoc/` directory
3. **Update `style.yml`**:
   ```yaml
   csl: pandoc/your-style.csl
   ```

Common styles included:
- `elsevier-vancouver.csl`: Numeric citations (Vancouver style)
- `engineering-applications-of-artificial-intelligence.csl`: EA-AI journal style
