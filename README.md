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
- Tab-layout equation paragraphs automatically use `Para Equation`, based on `Body Text`, with 0.5 line spacing after and single line spacing. To apply only this step to an existing DOCX in place, run `uv run python -m pandoc_manuscript.docx.postprocess.para_equation_style path/to/file.docx` (add `--no-save` for a dry run).
  The style's center and right tab stops use half and all of the first DOCX section's writable width (page width minus left/right margins). Direct paragraph tab stops are removed so equations inherit the style's positions; rerun the step after changing page margins.
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
uvx --from pandoc-manuscript-template pmt init my-paper
cd my-paper
pmt doctor
pmt build docx
```

To initialize the manuscript project in the current directory, omit the target directory:

```bash
pmt init
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

- [`template/.agents/manuscript-syntax.md`](template/.agents/manuscript-syntax.md): manuscript syntax, citations, cross-references, pseudocode, revision markup, and style metadata
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
pmt init [directory]
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

## Maintainer releases and native build caches

Release with `uvx bump-my-version bump patch` followed by
`git push origin main --tags`. The **Publish to PyPI** workflow publishes only
on `v*` tag pushes. Its `main` builds and manual runs build and verify the same
three platform wheels without publishing them.

Changes to either Rust submodule, the native build hook, helper inputs, or the
publishing workflow trigger cache warming on `main`. To warm or refresh caches
manually, run **Publish to PyPI** with **Run workflow**, selecting `main`.
Complete the first warm-up before creating the next release tag: GitHub allows
tags to restore default-branch caches, but not caches saved under other tags.
Pushing `main` and a tag together does not make the tag wait for cache warming;
it can use an older `main` cache and compile any changed dependencies.

The workflow pins Rust to `1.98.0` and caches Cargo downloads and release build
outputs separately for Windows, macOS 14, and manylinux 2.28. Cache keys include
the toolchain, native lockfiles, submodule revisions, build configuration, and
equation preferences, rather than the Python package version. Only successful
`main` builds save caches; tag builds restore them and still run Cargo with
`--locked`, package the wheel, and verify its native libraries.

The wheel builds and ships only the `mathtype-rust` shared library. Its versioned
C ABI handles OLE/MTEF conversion and `operation="render_wmf"` requests, linking
`latex2wmf` once as a pinned Git dependency. The renderer retains backend, style,
font size, and math-font options. The standalone `latex2wmf` crate and CLI remain
available for development; its dynamic library is not shipped.

CI sets `CARGO_TARGET_DIR` to persist native build artifacts. On Linux this directory and Cargo's download cache live on the host via
the container's `/host` mount, so they survive the manylinux container. Local
builds retain their normal per-project target directories unless this environment
variable is set. Build logs report elapsed time for each Rust library and the
Windows .NET helper. A cold cache or a toolchain change still requires compilation;
actual release speedups should be measured after a successful warm-up.

Before saving the Linux cache, the workflow transfers its container-created files
to the runner user and checks directory sizes and readability. A lookup after
saving requires the exact cache entry to exist remotely; an archive/upload failure
therefore fails the warm-up instead of silently leaving future releases uncached.

## Acknowledgments

- [Pandoc](https://pandoc.org/)
- [pandoc-crossref](https://github.com/lierdakil/pandoc-crossref)

## Support

- Review the syntax guide in [`template/.agents/manuscript-syntax.md`](template/.agents/manuscript-syntax.md)
- Open an issue with a minimal reproducible example

### Typst equation font

Configure `style.yml` to select an installed OpenType math font:

```yaml
mathtype: true
mathtypeConversionMethod: rust
mathtypeSvgBackend: typst
mathtypeTypstMathFont: Cambria Math
```

Bundled math families are `XITS Math` (the default) and `New Computer Modern Math`.
You can also set `mathtypeTypstMathFont: fonts/STIXTwoMath-Regular.otf` to load a font
file without installing it. Relative paths resolve beside the style file; absolute
paths are supported. Accepted files are `.otf`, `.ttf`, `.ttc`, and `.otc`, and must
contain an OpenType math font. Collections select their first math family.
When using a custom family name, install it in every build environment.
Changing the family or font file contents invalidates the formula preview cache.
This controls Typst SVG/WMF previews, not editable MathType OLE font preferences.
It has no effect on RaTeX or native MathType previews (`set-data` / `rust-sdk`).
`auto` applies it only when using `rust`; `both` retains the native MathType result.

If the `mathtype-rust` native library reports a formula conversion error, the build warns with the
exit code and LaTeX input and continues. The affected formula retains its original
Word equation (OMML); other formulas are converted normally. In `both` mode,
conversion continues with the `set-data` result. Failed conversions are not cached.
