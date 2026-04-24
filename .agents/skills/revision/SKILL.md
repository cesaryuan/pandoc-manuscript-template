---
name: revision
description: "当需要处理期刊审稿意见并准备修订稿及逐点回复信时，应使用此技能。当用户提到审稿意见、修改、rebuttal 或回复信时，应触发此技能。"
---

# 论文修订

帮助用户回应期刊审稿意见，起草逐点回复信，并将相应修改落实到稿件中。

## 工作流程

1. 阅读审稿意见（用户以文本或文件形式提供）
2. 阅读当前稿件
3. 针对每条审稿意见起草回复，并识别稿件中需要修改的内容
4. 将回复信草稿呈现给用户审阅
5. 在用户确认后，把修改应用到稿件中

## 回复信格式

回复信建议采用以下结构（Markdown 格式）：

```markdown
# Response to Reviewers

We thank the reviewers for their constructive comments. Below we address each
comment point by point. Reviewer comments are in **bold**, and our responses
follow each comment. All changes in the manuscript are highlighted in blue.

## Reviewer 1

**Comment 1: [Summarize the reviewer's comment]**

[Response explaining what was done and why]

[If applicable: "We have revised Section X to clarify this point (see lines XX-XX)."]

**Comment 2: ...**

...

## Reviewer 2

...
```

## 回复写作指南

**语气：**
- 始终保持尊重和专业，即使面对尖锐或不公正的评论也是如此
- 如果审稿人提出了确有价值的见解，应真诚表示感谢
- 不要使用轻视、对抗或防御性的语气

**回复类型：**

对于需要修改的意见：
- 先确认问题
- 说明修改了什么，以及修改位置在哪里
- 如果新文本或修订文本较短，可直接引用出来

对于需要说明不同意见的情况：
- 先承认审稿人的视角有其合理性
- 为当前做法提供证据或论证
- 如果可能，给出折中方案（例如补充一段讨论）

对于要求补充实验的意见：
- 如果可行：补做实验并报告结果
- 如果不可行：解释原因，并提出替代方案

**稿件修改：**
- 清晰追踪所有改动，注明修改了哪个章节或段落
- 确保全文修改前后一致，例如术语一旦变化，全文都要同步更新
- 在重组结构后，检查交叉引用是否仍然有效

## 常见修订模式

**补充说明：** 在相关位置插入 1-2 句解释，并保持段落行文自然。

**重组章节：** 遵循 `CLAUDE.md` 中的学术写作规范，不使用伪标题，保持叙述性段落，并做好过渡。

**补充参考文献：** 将条目加入 `.bib` 文件，并使用 `[@key]` 语法引用。注意检查 key 是否与现有条目冲突。

**更新图表：** 同时更新图表本身，以及所有描述或引用该图表的正文内容。
