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

## 回复信规范

撰写回复信时有以下几个指导原则务必要遵守：

1. 引用原文时应该用 _强调_ 标签包裹起来，以示区分。引用的原文应该和文章完全一致。
2. 引用的原文如果是图片或者表格，应该在其题注的最前面加上 `Figure @itslabel` 或者 `Table @itslabel` 来表明这是一个图片或者表格的引用。比如对于图片 `![caption](path){#fig:label}`，在回复信中应该写成 `![Figure @fig:label caption](path){#fig:label}`，这样才能保证编译出来的回复信 DOCX 中的图片编号和文章一致。
3. 应该用 ``(Line `regex`)`` 来引用手稿中的位置，其中 `regex` 指的是一个能够在最终手稿 PDF 或由 Word DOCX 转换得到的 PDF 文本层中唯一匹配到目标修改位置的正则表达式。构建回复信时，脚本会用这个正则表达式在手稿 PDF 中查找对应文本：如果唯一匹配，则自动替换为实际行号；如果匹配 0 个或多个位置，则在日志中输出红色提示并保留原占位符，等待人工修正。
4. 你的回复需要用 ::: {custom-style="Reply to Reviewers"} 包裹起来，以便在 DOCX 中应用特定样式。

`regex` 的写法必须遵守以下原则：

- 优先选取被修改段落、被新增句子、图题或表题中最有辨识度的一小段连续文本，通常 6-15 个英文单词即可。
- 正则应该对应最终手稿 PDF 中可见的文字，而不是 Markdown 源文件中的内部写法。比如图表编号在 PDF 中会显示为 `Figure 3` 或 `Table 2`，不要用 `@fig:...` 或 `@tbl:...` 去匹配 PDF 文本层。
- 词与词之间建议使用 `\s+`，而不是普通空格，以兼容 PDF 换行、多个空格、断行和 Word 导出 PDF 时的排版差异。
- 对于图题和表题，推荐写成 `Figure\s+\d+\s+题图关键词...` 或 `Table\s+\d+\s+表题关键词...`，避免因为图表编号变化导致匹配失败。
- 避免使用过短或过泛的表达式，例如 `Introduction`、`the proposed method`、`Table`、`Figure`，这类正则很容易匹配到多个位置。
- 避免依赖很容易变化的数值、引用编号、页码、行号或格式符号。除非该数值本身就是修改的核心内容，否则应优先用稳定的文字短语定位。
- 遇到 PDF 文本层可能出现单复数、冠词、是否包含 `using` 等差异时，可以用可选分组增强鲁棒性，例如 `The workflow yields\s+(?:an\s+)?explicit textured meshes?\s+without(?: using)? dense point-cloud meshing`。
- 如果无法确信正则只会唯一匹配，宁可写一个更长、更具体的短语，或者保留需要人工确认的占位说明，不要随意写一个可能指向错误位置的正则。

回复信模板：

```markdown
# Reply to comments of reviewers

::: {custom-style="Reply Header"}
**Title:** [Manuscript Title]

**Journal:** [Journal Name]

**Authors:** Author 1, Author 2, Author 3

**Manuscript Number:** XXXX-D-00-0000

<br>

The authors would like to thank the editors and reviewers for their interest in the paper, and the constructive comments and suggestions. The manuscript has been revised according to the reviewers’ comments. For easy reference, the reviewers’ suggestions are copied, followed by the answers.
:::


# Editor and Reviewer(s)' Comments to Author:

## Editor:

I have completed my evaluation of your manuscript. The reviewers recommend ...

::: {custom-style="Reply to Reviewers"}
_Answer:_

...[Your reply]...
:::

<br>

## Reviewer 1:

This paper introduces ... there are several improvements needed before potential publication.

::: {custom-style="Reply to Reviewers"}
_Answer:_

...[Your reply]...
:::

<br>

1\. This paper focused ...

::: {custom-style="Reply to Reviewers"}
_Answer:_

...[Your reply]...

The added explanation is as follows (Line `Figure\s+\d+\s+Visual comparison of standard and high-density COLMAP`):

_“引用原文.....”_
:::
```

## Reminders

- Image paths relative to `manuscript.md` location
- Citation keys must match `.bib` entries exactly
- Always preserve technical content when improving structure and flow
- When in doubt, follow conventions of the user's target journal

