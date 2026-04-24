---
name: word-manuscript-fix
description: 当需要将 Word 稿件转换为或修复为本仓库使用的 Pandoc Markdown 格式时，应使用此技能，尤其适用于需要先转换 `.docx`，或者生成后的 Markdown 仍存在格式问题的情况。
---

# Word 稿件修复

当用户从 `.docx` 稿件开始，或从已经由 `pandoc` 生成的 Markdown 开始，但稿件中仍然存在转义后的 LaTeX 数学公式、纯文本编号参考文献、Word 风格的 `_Ref...` 锚点、基于表格的图片和公式排版、图片尺寸精度过高、不符合项目规范的表格语法，或者缺少 YAML front matter 时，应使用此技能。

如果用户要求直接转换 `.docx` 文件，先运行：

`pandoc .\input.docx -o manuscript-generated.md --extract-media=<项目文件夹>/images --wrap=none`

然后再基于生成的 Markdown 文件继续执行本技能的其余步骤。

## 本技能必须修复的问题

- 将纯文本编号引用和参考文献列表转换为 `[@citekey]` 引文，并生成对应的 `.bib` 文件。
- 生成有意义的 citekey，而不是数字键；应使用可读的 `authorYearKeyword` 风格，例如 `he2016resnet` 或 `shorten2019augmentation`。
- 将 Word 风格的交叉引用，如 `[]{#_Ref221975190 .anchor}表 1...` 和 `如[表 1](#_Ref221975190)所示`，替换为本仓库使用的 `pandoc-crossref` 语法。
- 将使用 Word 表格排版的图片或图注改写为普通的 Pandoc Markdown 图片块。
- 将表示子图的一组图片（通常也是通过表格布局）转换为仓库推荐的 `pandoc-crossref` 子图结构。
- 将图片尺寸的精度降低到四位有效数字。
- 将 Word 导出的表格重写为项目推荐的 Markdown 表格语法。
- 参照 `manuscript.md` 的结构，补充本仓库要求的 YAML front matter。

## 工作流程

1. 如果输入是 `.docx`，先使用 `pandoc .\input.docx -o manuscript-generated.md --extract-media=<项目文件夹>/images --wrap=none` 进行转换。
2. 检查生成后的 Markdown，识别其中的转义公式、参考文献格式、交叉引用格式、图片布局、子图布局、图片尺寸、表格语法以及缺失的 front matter。
3. 仅为恢复被转义的数学公式运行 `scripts/unescape_latex.py`。
4. 其余问题都应直接编辑目标 Markdown 文件完成修复；不要为了引文转换、交叉引用转换、子图重建、图片尺寸清理或表格清理再额外编写脚本。
5. 将纯文本参考文献列表重建为 `.bib` 文件，并把文中的数字引用替换为有意义的 `@citekey` 引用。
6. 将 `_Ref...` 这类 Word 锚点替换为 `pandoc-crossref` 标签，如 `#fig:*` 和 `#tbl:*`，并把 `[表 1](#_Ref...)` 这类文本链接改写为 `[@tbl:*]` 或 `[@fig:*]` 引用。
7. 将仅用于摆放图片和图注的布局表格改写为语义化的 Pandoc 图片块或子图组。
8. 将数据表格改写为仓库推荐的 Markdown 表格语法，而不是保留 Word 导出的边框表格。
9. 将图片尺寸四舍五入到四位有效数字；如果可以，优先只保留 `width`。
10. 依据 `manuscript.md` 补充或修复 YAML front matter，并将 `bibliography` 指向生成的 `.bib` 文件。

## 输出要求

- 数学公式应以 Pandoc 数学格式正确渲染，而不是显示为可见的 LaTeX 原文。
- 引文必须使用 `[@citekey]` 语法，参考文献应存放在 `.bib` 文件中，而不是手写的编号列表。
- Citekey 必须是有意义的字符串，而不是简单数字。
- 图片使用 `![caption](path){#fig:label}` 语法，表格使用项目风格的 Markdown 表格语法，并带有如 `: Caption {#tbl:label}` 的标题行。
- 最终稿件中不再保留 Word 风格的 `_Ref...` 锚点，而应替换为 `pandoc-crossref` 引用。
- 通过表格实现的图片与图注布局，应改写为语义化的图片块。
- 子图组应使用仓库推荐的 `<div id="fig:...">` 模式，并包含子图片块。
- 图片尺寸应被四舍五入为四位有效数字。
- 最终稿件必须包含 `manuscript.md` 所要求的 YAML front matter。

## 备注

- 优先采用最小化、尽量不改变原意的修改。
- 保持 Introduction 为不带子小节的连贯叙述。
- 不要添加加粗伪标题。
- 如果某个 Word 表格只是用于布局，应将其替换为语义化 Markdown，而不是继续保留该表格。
- 除了恢复被转义的 LaTeX 数学公式外，其余修复都应通过直接编辑稿件和参考文献文件完成，而不是继续编写额外自动化脚本。
