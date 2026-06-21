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
3. 避免依赖公式中的文本；行内或行间公式在最终 PDF 文本层中的表示不可预测，可能丢失、拆分、重排或被转换成与源 Markdown 完全不同的字符。
4. 如果目标句子包含公式，例如 `bda dawh dawfa fgsg $100 \times 100$`，不要把 `100`、`\times`、`100 \times 100` 或公式附近的符号作为 regex 锚点；优先匹配公式前后稳定的自然语言短语。
5. 词与词之间使用 `\s+`，以兼容 PDF 换行、多个空格、断行和 Word 导出 PDF 的排版差异。
6. 图题和表题使用 `Figure\s+\d+\s+题图关键词...` 或 `Table\s+\d+\s+表题关键词...`，避免编号变化导致匹配失败。
7. 避免过短或过泛的表达式，例如 `Introduction`、`the proposed method`、`Table`、`Figure`。
8. 避免依赖易变化的数值、引用编号、页码、行号或格式符号；除非该数值本身就是修改核心。
9. PDF 文本层可能有单复数、冠词或 `using` 等差异时，用可选分组增强鲁棒性。
10. 无法确信唯一匹配时，写更长、更具体的短语，或保留需要人工确认的占位说明，不要随意写可能指错位置的 regex。

鲁棒 regex 示例：

```markdown
(Line `The workflow yields\s+(?:an\s+)?explicit textured meshes?\s+without(?: using)? dense point-cloud meshing`)
```

图题定位示例：

```markdown
(Line `Figure\s+\d+\s+Visual comparison of standard and high-density COLMAP`)
```

## 每条意见的回复正文

每条回答优先采用“修改动作 + 修改位置 + 具体内容 + 必要证据”的结构，而不是泛泛表示感谢或同意。回复的目标是让编辑和审稿人一眼看出：这条意见是否被正面处理、具体处理了什么、证据在哪里、哪些边界仍然存在。

### 推荐结构

对大多数实质性意见，按下面的顺序组织回答：

1. 第一段先直接回答问题，并概括本次修改动作。
2. 在同一段或下一段给出修改位置，优先写 `@sec:`、`@fig:`、`@tbl:` 配合 `(Line `regex`)`。
3. 具体说明改了什么，例如新增了哪类讨论、补了哪些实验、加入了哪些基线、改写了哪些定义、增加了哪些表格或图片。
4. 如果修改幅度较大，贴出新增或改写后的原文，并用 `_强调_` 包裹；如果新增的是图片或表格，则直接贴图表，不要用强调语法包裹。
5. 如果该意见无法被当前实验或材料完全解决，明确收窄 claim、补充 limitation，并说明已经采取的替代回应。

### 写作原则

1. 不要把大多数回答压缩成单段。只改了一两个词或一两句的小修可以简短处理；只要是较大的实质性修改，就至少写出“修改摘要 + 具体证据”两层内容。
2. 不要反复使用 `Thank you`、`We agree`、`We fully agree` 作为回答主体。对于实质性批评，开头更适合直接写 `We have added...`、`We have revised...`、`In the revised manuscript, we...`。
3. 对正面评价或礼貌性过渡，可以保留一句简短感谢，但感谢句不能取代技术回应，也不要连续多条意见都用相同的感谢开头。
4. 每个回答都要具体说明修改内容，而不是只说“已修改”或“已澄清”。优先点明修改对象，例如段落、公式定义、图题、表格、实验设置、评价协议、限制性讨论、未来工作。
5. 如果已经在论文中修改，必须说明修改位置。优先使用 `@sec:` / `@fig:` / `@tbl:` / `@eq:` 加 `(Line `regex`)` 的组合，而不是只写模糊的 section name。
6. 对于新增参考文献，不要只说“补充了相关文献”。应说明新增了哪些类型的近期相关工作；如果回复依赖这些文献支撑论证，优先直接列出关键条目或 cite key。
7. 如果审稿意见涉及创新性、动机、实验不足、对比不足、适用范围、局限性、计算代价、可复现性等问题，必须明确写出论文中新增了什么证据来回应这些问题。
8. 如果某条意见无法完全完成，不能回避。直接说明当前边界、原因和剩余风险，并说明已通过补充讨论、收窄结论、增加局限性说明、补充实验或未来工作来尽量回应。
9. 不要把回复写成辩解。需要保留不同意见时，用“重新界定 claim + 给出证据 + 说明边界”的方式回应，而不是情绪化反驳。

### 从优秀回复中提炼出的常用模式

1. 先给结论，再给证据：
   `We have added ... in @sec:... (Line `regex`) to clarify ...`
2. 如果一个意见触发了多处联动修改，集中概括所有改动，再逐一落点：
   `In the revised manuscript, we added ..., revised ..., and expanded ...`
3. 对于图表或实验增强，除了说明“新增了什么”，还要写清楚图表的作用：
   不要只写“added Table 5”；要写“the table reports vertices, faces, file size, loading time, and ROI-SSIM, showing that ...”
4. 对于正文改写，优先用：
   `The added text is as follows ...`
   `The revised explanation is as follows ...`
   `The newly added figure/table is as follows ...`
5. 对于容易被误解的点，明确写出“原文哪里会造成误解，本次如何纠正”：
   例如 `The original description may lead readers to think ... We revised ... to state clearly that ...`
6. 对于证据仍不充分的问题，不假装完全解决，而是同步收窄论断：
   例如 `In the revised manuscript, we have added new evidence and, at the same time, narrowed the claims to the scope currently supported by the experiments.`

### 常见意见的回应重点

1. **创新性或动机不清：**
   明确补充 motivation、problem gap、contribution wording，并在回复中指出具体新增句子或段落。
2. **实验不足、对比不足：**
   明确写出新增了哪些 baseline、case study、ablation、held-out evaluation、runtime 或 complexity analysis，并指出新增图表分别证明什么。
3. **适用范围或泛化性不足：**
   一方面补充额外案例或讨论；另一方面清楚界定当前验证边界，不要继续维持超出证据范围的泛化 claim。
4. **局限性未讨论：**
   不要只说“已在结论中讨论”。应概括 limitation 的核心内容，并说明对应 future work。
5. **符号、定义、表达不一致：**
   说明统一了哪些符号、术语、图内标注和公式定义，并在必要时贴出修订后的定义段。
6. **数据、协议、可复现性不清：**
   优先补充样本数量、训练/测试划分、过滤条件、硬件环境、运行时间、参数设置和评价协议；回复中要把这些补充点明确列出来。

### 不建议机械重复的句式

以下句式可以作为局部表达参考，但不要在相邻多条意见中重复套用：

- `Thank you for this valuable comment.`
- `We have carefully revised the manuscript according to this comment.`
- `The novelty of the proposed method has been further clarified in the revised manuscript.`
- `For clarity, the newly added text is as follows.`
- `The comparison results have been added in Table X.`
- `More details are provided in Section X.`

更重要的是复用它们背后的功能：简短致谢、概括修改动作、指出落点、给出新增原文或图表证据。

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
6. regex 不依赖 Markdown 标签、固定页码、固定行号、过泛短语或公式文本层。
7. 对每条实质性意见，回答里都能看到“修改动作 + 修改位置 + 具体改动 + 必要证据”，而不是只有感谢或空泛表态。
8. 重大修改已贴出新增原文、图或表；小修改才允许只做简短说明。
9. 对仍有边界的问题，回复中已经明确收窄 claim、补充 limitation 或 future work，而不是假装完全解决。
