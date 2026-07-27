[English](README.md) | [简体中文](README.zh-CN.md)

# Pandoc Manuscript Template

Write in Markdown. Submit in Word.

PMT is a DOCX-first academic writing workflow built for the AI era. AI tools are already great at drafting, revising, and restructuring Markdown. The problem is that many journals, editors, and collaborators still expect `.docx`. PMT bridges that gap: you keep the clarity and version-control friendliness of Markdown, while generating submission-ready Word documents when it is time to deliver.

<!--
Hero image idea for the README:
- Use a wide 3-panel workflow graphic instead of a logo-only banner.
- Left panel: a clean Markdown manuscript in an editor, with citations, cross-references, and a short AI chat prompt visible.
- Middle panel: a terminal running `pmt build docx` and `pmt build-reply`.
- Right panel: a polished Word manuscript page plus a reviewer-reply DOCX page.
- Add 3 short callouts on top of the image: "AI writes Markdown well", "PMT turns it into DOCX", "Journal-ready output".
- The most eye-catching version will show the same content flowing from raw Markdown to polished Word, not abstract icons.
-->

## Why This Exists

Markdown has become a very natural writing format for research teams, especially when AI is part of the drafting loop. It is easier to generate, review, diff, and refine than LaTeX for many authors. LaTeX is still powerful, but it is not always the most approachable tool for collaborators who mainly need to write and revise. Typst is promising, but it is not yet the default format most journals ask for.

DOCX, however, is still the format a lot of publishers, editors, and co-authors want.

PMT is built around that reality:

- Write the manuscript in Markdown.
- Keep sources easy for humans and AI to edit.
- Generate Word-first output for submission.
- Preserve the pieces academic writing actually needs: citations, equations, tables, figures, cross-references, and reviewer replies.

## Why PMT

PMT is not just a generic Pandoc wrapper. It is a manuscript workflow with opinionated support for the annoying parts of real submission work.

- **DOCX-first workflow**: the primary target is a polished Word manuscript, not DOCX as an afterthought.
- **AI-friendly authoring**: Markdown is easier for LLMs to generate and easier for humans to review in Git.
- **One-command project bootstrap**: `pmt init` creates a reusable paper workspace with manuscript files, style metadata, references, and agent guidance.
- **Submission-oriented post-processing**: PMT applies DOCX-specific cleanup and formatting after Pandoc runs.
- **Reviewer reply support**: build response letters as DOCX or TXT, while resolving manuscript references and citations.
- **Managed Pandoc tools**: if `pandoc` or `pandoc-crossref` are missing, PMT can install project-local copies under `.pmt/tools`.
- **Optional LaTeX and JSON output**: keep a Markdown-centered workflow without giving up other export targets.

## What You Get

- Manuscript scaffolding with `pmt init`
- Environment checks with `pmt doctor`
- Project-local tool setup with `pmt setup`
- DOCX, LaTeX, and JSON builds with `pmt build`
- Reviewer reply builds with `pmt build-reply`
- Cross-references for figures, tables, equations, and sections
- CSL-based citations
- Reference DOCX support for Word styling
- DOCX post-processing for author blocks, table behavior, styles, and line-number-related workflows
- SVG handling and DOCX fallbacks for figures that Word does not handle well
- Cross-platform MathType-compatible OLE/WMF equations, with an optional native MathType comparison path on Windows

## Quick Start

### Prerequisites

Install these tools first:

1. `uv` for running the CLI and Python environment
2. `pandoc` 3.0+ and `pandoc-crossref`
3. Optional: Microsoft Word or `soffice` for line-number source workflows
4. Optional: MathType on Windows only if you select `rust-sdk`, `set-data`, `auto`, or `both`; the default `rust` path is self-contained

If `pandoc` or `pandoc-crossref` are not on `PATH`, PMT can download managed project-local copies into `.pmt/tools`.

### Rough Python Compatibility Check

If you just want a quick syntax-level check against the project's minimum Python target, use Ruff:

```bash
uvx ruff check .
```

This is only a rough version-compatibility check. It can catch syntax that does not fit the configured Python target, but it does not prove runtime compatibility.

### Create Your First Project

```bash
uvx --from git+https://github.com/cesaryuan/pandoc-manuscript-template pmt init my-paper
cd my-paper
pmt doctor
pmt build docx
```

That produces:

```text
output/docx/manuscript.docx
```

If you prefer installing the tool once:

```bash
uv tool install --upgrade pandoc-manuscript-template
pmt init my-paper
```

After each `pmt` invocation, PMT reads its cached PyPI update status and prints an upgrade hint when one is available. A silent background worker refreshes that cache at most once every hour, so commands do not wait for network I/O. Upgrade an installed PMT tool with:

```bash
uv tool upgrade pandoc-manuscript-template
```

## Typical Workflow

```bash
# Create a new manuscript project
pmt init my-paper --setup

# Check dependencies and project files
pmt doctor

# Build the main manuscript
pmt build docx

# Build another Markdown file explicitly
pmt build docx paper.md -o build/paper.docx

# Build a reviewer reply
pmt build-reply reply.md --reply-manuscript manuscript.md -o output/docx/reply.docx
```

## Standout Features

### 1. Markdown that stays pleasant to edit

PMT leans into plain-text authoring instead of fighting it. Your manuscript remains easy to diff, refactor, prompt into AI tools, and review collaboratively.

### 2. DOCX output that is actually the point

Many academic writing pipelines treat DOCX as a secondary export. PMT treats it as the main delivery format, with Word-oriented defaults and post-processing built into the workflow.

### 3. Better fit for real submission tasks

PMT goes beyond "convert Markdown to Word" by helping with the parts that tend to break late in the process:

- reviewer replies
- figure and table references
- equation numbering
- citation formatting
- Word reference documents
- DOCX figure edge cases such as SVG conversion or embedding

### 4. Friendly to automation without hiding the files

The output is scripted, reproducible, and version-controlled, but the source project still looks like a normal manuscript folder that a researcher can understand quickly.

## Documentation Map

- [`template/manuscript-syntax.md`](template/manuscript-syntax.md): manuscript syntax, citations, cross-references, pseudocode, revision markup, and style metadata
- [`template/manuscript.md`](template/manuscript.md): example manuscript content
- [`AGENTS.md`](AGENTS.md): repository-specific guidance for coding agents

In generated projects, `style.yml` keeps PMT-owned build settings at the top
level and places metadata sent to Pandoc under `pandocMetadata`. Manuscript YAML
overrides only the Pandoc metadata domain.

## When PMT Is a Good Fit

PMT is especially useful if:

- you draft heavily with AI and want a format AI handles naturally
- you want Git-friendly manuscript sources instead of editing Word binaries directly
- your target journal still expects DOCX
- you need a repeatable manuscript and reviewer-reply workflow
- you want Pandoc power without forcing every collaborator into a LaTeX-first workflow

## Commands at a Glance

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

Use `pmt --help` to see the full CLI.

Add `--verbose` to any command, for example `pmt build docx --verbose`, to show
detailed debug logs such as complete external command lines. Normal output keeps
the main build stages, warnings, and results concise.

## Acknowledgments

- [Pandoc](https://pandoc.org/)
- [pandoc-crossref](https://github.com/lierdakil/pandoc-crossref)

## Support

- Review the syntax guide in [`template/manuscript-syntax.md`](template/manuscript-syntax.md)
- Open an issue with a minimal reproducible example
