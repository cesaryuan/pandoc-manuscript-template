[English](README.md) | [简体中文](README.zh-CN.md)

# Pandoc Manuscript Template

用 Markdown 写论文，用 Word 交稿。

PMT 是一个面向 AI 时代的、以 DOCX 为核心输出的学术写作工作流。现在 AI 很擅长起草、改写和整理 Markdown，但很多期刊、编辑和合作者最终仍然要 `.docx`。PMT 解决的正是这个错位问题：你继续用清晰、可版本控制、对 AI 友好的 Markdown 写作，在需要提交的时候再稳定地产出接近期刊工作流的 Word 文档。

<!--
README 首图建议，方便你后面自己截图或绘制：
- 用一个横向 3 面板流程图，不要只放 logo。
- 左侧放 Markdown 稿件编辑界面，最好能看到引用、交叉引用，以及一个简短的 AI 提示词或对话片段。
- 中间放终端，显示 `pmt build docx` 和 `pmt build-reply`。
- 右侧放排版完成的 Word 主稿页面，再加一个 reviewer reply 的 DOCX 页面。
- 图上只保留 3 个短标语最抓眼球，比如：“AI writes Markdown well”, “PMT turns it into DOCX”, “Journal-ready output”。
- 最重要的是让人一眼看懂“同一份内容从 Markdown 流到 Word”的前后对比，而不是抽象图标。
-->

## 为什么会有这个项目

对于很多研究团队来说，Markdown 正在变成一种非常自然的写作格式，尤其是在 AI 已经深度参与起草和修改的情况下。它比 LaTeX 更容易生成、更容易审阅，也更容易做版本比较。LaTeX 当然依然强大，但对很多主要任务是写作和修改的作者来说，它并不总是最友好的选择。Typst 也很有潜力，但目前还不是多数出版社默认接受的主流格式。

现实是，DOCX 仍然是很多出版社、编辑和合作者最喜欢的格式。

PMT 就是围绕这个现实构建的：

- 用 Markdown 写稿
- 保持源文件对人和 AI 都好编辑
- 在交付时生成 Word 优先的投稿文件
- 保留学术写作真正需要的能力：参考文献、公式、表格、图片、交叉引用，以及审稿回复

## PMT 的价值

PMT 不只是一个通用的 Pandoc 封装器。它是一个面向真实投稿流程的 manuscript workflow。

- **DOCX 优先**：主目标是高质量 Word 稿件，而不是把 DOCX 当成顺手导出的副产品。
- **对 AI 友好**：Markdown 更适合 LLM 生成，也更适合人在 Git 里审阅。
- **一条命令初始化项目**：`pmt init` 可以直接生成论文目录结构、稿件、样式元数据、参考文献和 agent 指南。
- **面向投稿的后处理**：Pandoc 结束后，PMT 还会做 DOCX 侧的格式整理和增强。
- **支持审稿回复**：`build-reply` 可以生成 DOCX 或 TXT，并自动解析正文中的引用和交叉引用。
- **自管理 Pandoc 工具链**：如果系统里没有 `pandoc` 或 `pandoc-crossref`，PMT 可以把它们下载到当前项目的 `.pmt/tools`。
- **保留其他输出**：虽然以 DOCX 为核心，但仍然支持 LaTeX 和 JSON 输出。

## 你能得到什么

- 用 `pmt init` 初始化论文工程
- 用 `pmt doctor` 检查环境
- 用 `pmt setup` 准备项目本地工具
- 用 `pmt build` 构建 DOCX、LaTeX、JSON
- 用 `pmt build-reply` 构建审稿回复
- 图、表、公式、章节的交叉引用
- 基于 CSL 的参考文献格式
- 通过 reference DOCX 控制 Word 样式
- 面向 DOCX 的后处理：作者信息、表格行为、样式、行号相关工作流
- Word 不友好图片场景下的 SVG 处理和回退方案
- 需要时支持 MathType 相关的 DOCX 工作流

## 快速开始

### 前置依赖

建议准备以下工具：

1. `uv`
2. `pandoc` 3.0+ 和 `pandoc-crossref`
3. 可选：Microsoft Word 或 `soffice`，用于某些行号来源工作流
4. 可选：MathType，用于需要 MathType 公式的 DOCX 输出

如果 `pandoc` 或 `pandoc-crossref` 不在 `PATH` 中，PMT 可以把受管工具下载到项目内的 `.pmt/tools`。

### Python 版本粗检

如果你只是想快速做一次面向语法的 Python 版本检查，可以直接用 Ruff：

```bash
uvx ruff check .
```

这只是粗略检查，能发现不符合当前 Python 目标版本的语法，但不能证明运行时一定兼容。

### 创建第一个项目

```bash
uvx --from pandoc-manuscript-template pmt init my-paper
cd my-paper
pmt doctor
pmt build docx
```

生成结果：

```text
output/docx/manuscript.docx
```

如果你更喜欢先全局安装一次工具：

```bash
uv tool install --upgrade pandoc-manuscript-template
pmt init my-paper
```

每次 `pmt` 命令结束后，PMT 会读取本地缓存的 PyPI 更新状态；发现新版本时会提示升级命令。后台 worker 最多每小时刷新一次缓存，因此命令不会等待网络请求。使用以下命令升级已安装的 PMT：

```bash
uv tool upgrade pandoc-manuscript-template
```

## 典型工作流

```bash
# 初始化一个新的论文项目
pmt init my-paper --setup

# 检查依赖和项目文件
pmt doctor

# 构建主稿
pmt build docx

# 显式构建另一个 Markdown 文件
pmt build docx paper.md -o build/paper.docx

# 构建审稿回复
pmt build-reply reply.md --reply-manuscript manuscript.md -o output/docx/reply.docx
```

## 这个项目最吸引人的地方

### 1. Markdown 真的适合写和改

PMT 不会逼你放弃纯文本写作。你的 manuscript 仍然易于 diff、重构、喂给 AI、以及协作审阅。

### 2. DOCX 不是附带功能，而是主要目标

很多学术写作工具链把 DOCX 当成“顺便导出一下”。PMT 从一开始就是 Word 导向的，默认设置和后处理都围绕最终交付文档来设计。

### 3. 面向真实投稿细节

PMT 关注的不只是“把 Markdown 转成 Word”，还包括那些常常在投稿前最后几天最容易出问题的环节：

- 审稿回复
- 图表引用
- 公式编号
- 参考文献格式
- Word reference document
- SVG 等 DOCX 图片边界情况

### 4. 适合自动化，但文件仍然透明

整个流程可以脚本化、可复现、可版本控制，但生成出来的项目结构依然是研究者一眼能看懂的论文目录，而不是黑盒。

## 文档导航

- [`template/manuscript-syntax.md`](template/manuscript-syntax.md)：稿件语法、引用、交叉引用、伪代码、修订标记、样式元数据
- [`template/manuscript.md`](template/manuscript.md)：示例稿件
- [`AGENTS.md`](AGENTS.md)：仓库级 agent 指南

## 什么时候特别适合用 PMT

如果你符合下面这些情况，PMT 会很合适：

- 你会大量借助 AI 起草和修改论文
- 你希望稿件源文件更适合 Git 管理，而不是直接修改二进制 Word
- 目标期刊仍然要求 DOCX
- 你需要一套可重复的主稿和审稿回复工作流
- 你想要 Pandoc 的能力，但不想强迫所有合作者都进入 LaTeX-first 工作方式

## 命令速览

```bash
pmt init my-paper
pmt setup
pmt doctor
pmt build docx
pmt build latex
pmt build json
pmt build-reply reply.md -o output/docx/reply.docx
pmt clean
pmt distclean
```

完整 CLI 请查看 `pmt --help`。

## 致谢

- [Pandoc](https://pandoc.org/)
- [pandoc-crossref](https://github.com/lierdakil/pandoc-crossref)

## 支持

- 先查看 [`template/manuscript-syntax.md`](template/manuscript-syntax.md)
- 提 issue 时尽量附上最小可复现示例
