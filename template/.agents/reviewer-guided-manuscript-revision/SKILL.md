---
name: reviewer-guided-manuscript-revision
description: 当需要根据审稿意见、编辑意见、作者拟定的回复策略或具体修改建议来修改 Pandoc Markdown 学术论文稿件时，应使用此技能。适用于 reviewer revision、按审稿意见改论文、逐条落实审稿建议、添加审稿意见对应注释、补充图表占位和实验表格等任务；不适用于泛泛润色或与审稿意见无关的全文重写。
---

# 按审稿意见修改论文

根据审稿意见修改论文时，必须把修改范围严格限制在审稿意见或作者明确建议覆盖的内容内。目标是让稿件本身自然变好，而不是写出带有回复信语气的文字。

## 修改原则

1. 不要修改任何与审稿意见无关的内容。即使发现其他段落存在问题，只要审稿意见或作者建议没有提到，也不要擅自修改。
2. 在任何被修改的段落上方添加 Markdown 注释，说明该修改对应哪一条审稿意见。
3. 如果段落上方已经有其他审稿意见的对应注释，在下方新增注释，不要改写或删除原有注释。
4. 不要做无关紧要的单词级微调。优先穿插加入完整句子或段落，只在必要时修改个别词语和描述。
5. 新增或修改的句子必须保持论文正文语气，不要出现明显的审稿回复痕迹。
6. 修改后检查 diff，移除任何越界修改、无关润色或没有审稿意见依据的改动。

审稿意见注释示例：

```markdown
<!-- Revision for R1-3: clarify the data preprocessing workflow. -->
The preprocessing workflow consists of three stages...
```

如果已有注释：

```markdown
<!-- Revision for R1-2: define the evaluation metric. -->
<!-- Revision for R2-1: add the cross-dataset comparison. -->
The evaluation metric is computed as...
```

## 写作要求

修改稿件正文时：

1. 用论文自身的叙述逻辑组织新增内容，不要写“according to the reviewer”“we have revised”“to address this comment”等回复信表达。
2. 让新增句子自然连接上下文，必要时新增一个完整段落，而不是只替换一两个词。
3. 如果必须改变已有表述，保留原段落的技术含义和引用关系，不要顺手扩展到其他主题。
4. 如果审稿意见要求解释方法、实验、局限性或应用边界，优先补充可直接留在论文中的技术说明。

## 表格修改

新增或修改表格时：

1. 直接写出完整 Markdown 表格。
2. 用合理的占位数值填充表格，方便作者总览结构并与真实实验数据对比。
3. 表格标题、列名、单位和指标方向要清楚。
4. 如果数值是占位值，在表格附近用 Markdown 注释说明需要作者替换为真实实验结果。

示例：

```markdown
<!-- Revision for R2-4: add an ablation comparison table. -->
<!-- TODO: Replace the placeholder values with the final experimental results. -->
| Configuration | Accuracy (%) | Runtime (s) | Memory (GB) |
|---|---:|---:|---:|
| Baseline | 82.4 | 18.6 | 4.2 |
| Proposed module | 87.9 | 20.1 | 4.8 |

: Ablation comparison of the proposed module. {#tbl:ablation}
```

## 图片修改

新增或修改图片时：

1. 新增图片先用 Markdown 图片占位符表示，不需要绘制 SVG 或其他图片。
2. 在新增图片下方用中文 Markdown 注释说明该图片预期展示的内容。
3. 修改现有图片时，先读取或查看图片，理解当前布局和内容；再在现有图片下方添加中文 Markdown 注释，说明需要修改哪些内容。
4. 图片说明应具体到图中元素、对比关系、布局或标注方式，方便后续制作真实图片。

新增图片示例：

```markdown
<!-- Revision for R3-2: add a workflow overview figure. -->
![Workflow overview placeholder](images/workflow-overview-placeholder.png){#fig:workflow-overview width=90%}
<!-- 该图预期展示完整方法流程：左侧为输入数据，中间为三个处理阶段，右侧为输出结果；需要用箭头标明数据流，并突出新增模块在流程中的位置。 -->
```

现有图片修改示例：

```markdown
![Existing comparison result](images/comparison-result.png){#fig:comparison-result width=90%}
<!-- Revision for R1-5: 需要在该图中增加本文方法与基线方法的局部放大对比，并用统一颜色标注误差较大的区域。 -->
```

## 完成前检查

交付修改前，逐条检查：

1. 每处修改都能追溯到审稿意见或作者明确建议。
2. 每个被修改段落上方都有对应的 Markdown 注释。
3. 旧的审稿意见注释没有被覆盖、合并或删除。
4. 正文没有出现回复信语气。
5. 表格有完整结构和合理占位数值，需要替换真实数据的位置有注释。
6. 新增图片使用占位符，图片预期内容用中文注释说明。
7. 现有图片修改建议写在图片下方，并基于当前图片内容。
8. `git diff` 中没有无关段落、无关格式化或只改一两个词的低价值修改。
