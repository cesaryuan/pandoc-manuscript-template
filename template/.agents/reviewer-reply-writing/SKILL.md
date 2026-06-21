---
name: reviewer-reply-writing
description: 当需要撰写、修改或检查论文回复信、reply to reviewers、response letter、审稿意见回复，尤其是需要按本仓库的 Pandoc Markdown/DOCX 回复信格式引用原文、引用图表、标注稿件位置或使用 Reply to Reviewers 样式时，应使用此技能。
---

# 回复信撰写

撰写或修改审稿回复信时，必须保持审稿意见原文不变，并把每条回答写成可由本仓库构建为 DOCX 的 Pandoc Markdown。回复内容应清楚说明作者如何修改稿件，并用可自动解析的 ``(Line `regex`)`` 标注修改位置。

## 必须遵守的格式

1. 引用文章原文时，用 `_强调_` 包裹，并确保引用内容与文章完全一致。
2. 图片引用不要用 `_强调_` 包裹，否则 pandoc 无法解析图片。
3. 审稿意见原文不要用 `_强调_` 包裹，保持原样。
4. 引用的原文如果是图片或表格，在题注最前面加上 `Figure @itslabel` 或 `Table @itslabel`。
5. 每段回复都用 `::: {custom-style="Reply to Reviewers"}` 包裹，以便 DOCX 应用回复样式。
6. 用 `(Line `regex`)` 引用手稿修改位置，不要手写固定行号。

图片引用示例：

```markdown
![Figure @fig:label caption](path){#fig:label}
```

表格引用示例：

```markdown
Table @tbl:label caption text...
```

## 行号 regex 规则

`(Line `regex`)` 中的 `regex` 必须能在最终手稿 PDF 或由 Word DOCX 转换得到的 PDF 文本层中唯一匹配目标修改位置。构建回复信时，脚本会用该 regex 查找实际行号：唯一匹配则自动替换为实际行号；匹配 0 个或多个位置则输出红色提示并保留占位符，等待人工修正。

编写 regex 时：

1. 优先选取被修改段落、新增句子、图题或表题中最有辨识度的一小段连续文本，通常 6-15 个英文单词。
2. 匹配最终 PDF 可见文字，而不是 Markdown 内部写法；图表编号应匹配 `Figure 3` 或 `Table 2`，不要匹配 `@fig:...` 或 `@tbl:...`。
3. 词与词之间使用 `\s+`，以兼容 PDF 换行、多个空格、断行和 Word 导出 PDF 的排版差异。
4. 图题和表题使用 `Figure\s+\d+\s+题图关键词...` 或 `Table\s+\d+\s+表题关键词...`，避免编号变化导致匹配失败。
5. 避免过短或过泛的表达式，例如 `Introduction`、`the proposed method`、`Table`、`Figure`。
6. 避免依赖易变化的数值、引用编号、页码、行号或格式符号；除非该数值本身就是修改核心。
7. PDF 文本层可能有单复数、冠词或 `using` 等差异时，用可选分组增强鲁棒性。
8. 无法确信唯一匹配时，写更长、更具体的短语，或保留需要人工确认的占位说明，不要随意写可能指错位置的 regex。

鲁棒 regex 示例：

```markdown
(Line `The workflow yields\s+(?:an\s+)?explicit textured meshes?\s+without(?: using)? dense point-cloud meshing`)
```

图题定位示例：

```markdown
(Line `Figure\s+\d+\s+Visual comparison of standard and high-density COLMAP`)
```

## 回复信模板

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

1\. This paper focused ... (不需要强调语法)

::: {custom-style="Reply to Reviewers"}
_Answer:_

...[Your reply]...

The added explanation is as follows (Line `Figure\s+\d+\s+Visual comparison of standard and high-density COLMAP`):

_“引用原文.....”_
:::
```

## 完成前检查

交付回复信前，逐条检查：

1. 所有回答块都使用 `::: {custom-style="Reply to Reviewers"}`。
2. 文章原文引用已用 `_强调_` 包裹，图片除外。
3. 审稿意见原文没有被强调、改写或重新编号。
4. 图表引用的题注前缀包含 `Figure @fig:label` 或 `Table @tbl:label`。
5. 所有修改位置都使用 `(Line `regex`)`，且 regex 面向最终 PDF 可见文本。
6. regex 不依赖 Markdown 标签、固定页码、固定行号或过泛短语。
