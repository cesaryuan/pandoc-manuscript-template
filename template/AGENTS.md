This template converts Pandoc Markdown manuscripts to DOCX, with optional LaTeX source generation for advanced users.

- Main file: `manuscript.md` — edit this to write the paper
- Style file: `style.yml` — edit this for style-related metadata, including citation style, cross-reference wording/numbering, subfigure behavior, and DOCX body text settings
- Images: place in `images/` directory
- References: `.bib` file specified in YAML header

For any syntax, formatting pattern, or writing fragment not covered in this file, consult `README.md` first and follow its more detailed guidance.

## Style Metadata

If the user wants to change style-related content, update `style.yml` rather than the YAML header in `manuscript.md`. The build loads `style.yml` first and then overlays the manuscript YAML metadata, so any field explicitly present in `manuscript.md` still takes precedence for that manuscript.

## Pandoc Markdown Syntax

**Cross-references:**
- Figures: `![caption](path){#fig:label}` → `[@fig:label]`
- Tables: `: Caption {#tbl:label}` → `[@tbl:label]`
- Equations: `$$ math $$ {#eq:label}` → `[@eq:label]`
- Sections: `# Title {#sec:label}` → `[@sec:label]`
- Citations: `[@key]` (parenthetical), `[@key1; @key2]` (multiple)

**Tables:** Please prefer to use pipe_tables which is identical to PHP Markdown Extra tables.
```markdown
| **Method** | **Accuracy (%)** |
|:----------:|:----------------:|
| Baseline   | 78.3             |
| **Proposed** | **92.4**       |

: Performance comparison. {#tbl:results}
```
- Bold only for highlighting best results in comparison tables
- Alignment: `:--` left, `:--:` center, `--:` right
- For advanced DOCX table formatting (cell merging, metadata), see README.md

**Subfigures** (requires `subfigGrid: true` in `style.yml` or merged YAML metadata):
```markdown
<div id="fig:results">
![caption of a](a.png){#fig:a width=50%} # Only percent allowed in subfigure width
![caption of b](b.png){#fig:b width=50%}

![caption of c](c.png){#fig:c width=50%}
![caption of d](d.png){#fig:d width=50%}
<!-- here should be a blank line -->
Main caption ( 2x2 grid of subfigures, change line by adding a blank line between images).
</div>

**Pseudocode/Algorithms:**
```markdown
Write pseudocode as a one-column pipe table. Use bold control words such as `**for**` and `**if**`. This template does not currently support cross references.

| **Algorithm: Library borrowing workflow** |
|---|
| **Input:** Request list $R$ |
| **for** each request **do** |
| \ \ Check availability |
| **end for** |

: Library borrowing workflow. {#tbl:algorithm}
```

## If User want to Change Citation Styles

1. Visit [Zotero Style Repository](https://www.zotero.org/styles) and find a CSL file for user required target journal or preferred citation style.
2. Download CSL file and save to `pandoc/` directory
3. **Update `style.yml`**:
   ```yaml
   csl: pandoc/csl-style-downloaded.csl
   ```
## Academic Writing Rules

**Critical Don'ts:**
1. No subsections under Introduction — must be coherent narrative
2. No bold as pseudo-headings — no `**Label**: content...` patterns
3. No short bullet lists — use narrative paragraphs (3-7 sentences)
4. No `@fig:label shows...` paragraph openers — lead with narrative, reference at end
5. No separate "Related Work" section — merge into unified Introduction
6. Minimize `###` headings — prefer narrative flow within `##` sections
7. No single-sentence paragraphs (except transitions)
8. Bold only for table best-results and contribution statements

**Style and language preferences:**
- Use formal academic prose with simple, precise, and common research vocabulary
- Avoid contractions such as `it's` or `doesn't`; use full forms instead
- Prefer natural academic flow; remove mechanical transitions and obvious AI-sounding wording
- Do not rewrite for the sake of rewriting; keep passages that are already clear and publication-ready
- Avoid noun possessives for methods, models, or systems when possible; prefer `the performance of X` or similar structures
- Preserve established technical abbreviations such as `LLM` unless expansion is explicitly needed
- Do not add bold or italics for emphasis in body text

**Editing heuristics:**
- Split long or awkward sentences when clarity improves, but preserve technical meaning
- Prefer coherent narrative paragraphs over short bullets
- Avoid opening a paragraph with a bare cross-reference such as `[@fig:case] shows`; state the point first, then attach the reference
- Minimize em dashes; prefer commas, parentheses, or subordinate clauses when suitable
- Replace inflated words such as `leverage`, `delve into`, `pivotal`, `underscore`, and `unveil` with plainer alternatives when possible

## 按照修改意见修改论文

按照审稿意见修改论文时有以下几个指导原则务必要遵守：
1. 不要修改任何与审稿意见无关部分的内容，即使该部分存在问题，但是因为审稿意见或者我们的建议修改中并没有提到，所以不要擅自修改。
2. 在任何被修改的段落上面用 markdown 注释表明此处修改对应哪一条审稿意见，当已经存在其他审稿意见的对应注释时，在下面新增注释而不是修改其他审稿意见的原有注释。
3. 不要做无关紧要的只修改一两个单词的修改。倾向于穿插加入完整的句子或者段落，仅在必要时修改文中某些单词和描述
4. 新增和修改的句子中不要有审稿回复痕迹，不要用那种明显是为了回复审稿人的语气的句子，应该用写论文的语气
5. 对于新增或者修改表格：请你直接写表格并用合理的数值帮我填上，这样方便我总览你添加的表格结构，以及和我的实验数据做对比。
6. 对于新增或者修改图片：对于新加的图片，先用占位符表示，然后再下面用 markdown 注释写上该图片预期要展示的内容，用中文；记住不需要你绘制 svg 或者其他图片，你需要的只是用注释说明该图片的预期内容。对于要对现有图片做修改的，在现有图片下面用注释说明大概需要修改哪些内容，你可以读取图片来查看现在图片的布局和内容等等，从而帮助你更好地了解要做什么修改。

## Reminders

- Image paths relative to `manuscript.md` location
- Citation keys must match `.bib` entries exactly
- Always preserve technical content when improving structure and flow
- When in doubt, follow conventions of the user's target journal
