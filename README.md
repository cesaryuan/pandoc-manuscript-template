# Pandoc Manuscript Template

A professional, reusable template for academic manuscripts focused on DOCX output, with optional LaTeX source generation using [Pandoc](https://pandoc.org/).

## Features

- **DOCX-focused output**: Generate Word-ready manuscripts for journal submission
- **Automatic formatting**: Consistent styling using reference documents and templates
- **Cross-references**: Automatic numbering and linking for figures, tables, equations, and sections
- **Flexible citations**: Support for 9000+ citation styles via CSL
- **Journal-ready DOCX workflow**: Reference-document styling and post-processing for submission files
- **Installable CLI**: Use `pmt` directly after package installation or through `uvx`
- **Reproducible**: Version-controlled workflow with CLI, Python

## Quick Start

### Prerequisites

Install the following tools:

1. **Pandoc** (>= 3.0): [Download](https://pandoc.org/installing.html)
2. **pandoc-crossref**: Required for figure, table, equation, and section references
3. **UV**: Recommended for running the `pmt` CLI and Python filters

### Generate Your First Document

1. **Create a manuscript project with `pmt`**:
   ```bash
   uvx --from git+https://github.com/yourname/pandoc-manuscript-template pmt init my-paper
   cd my-paper
   ```

   `pmt init` also writes an `AGENTS.md` file into the new project. If the
   target already has one, `pmt` leaves it in place and warns so you can merge
   the template notes manually. Use `pmt init my-paper --merge` to append the
   packaged `AGENTS.md` guidance and copy missing files from the packaged
   `.agents/` directory automatically without replacing existing files.

   When the package is installed as a tool, use:
   ```bash
   uv tool install pandoc-manuscript-template
   pmt init my-paper
   ```

2. **Check your environment**:
   ```bash
   pmt doctor
   ```

3. **Generate DOCX**:
   ```bash
   pmt build docx
   # Output: output/docx/manuscript.docx
   ```

   To build a different markdown file without editing the Pandoc defaults:
   ```bash
   pmt build docx paper.md
   # Output: output/docx/paper.docx
   ```

4. **View available commands**:
   ```bash
   pmt --help
   ```

## Usage Guide

### Writing Your Manuscript

Edit `manuscript.md` to replace the template content with your research. Use
`style.yml` for style-related defaults such as citation style, cross-reference
wording, subfigure behavior, and DOCX paragraph formatting.

For supported manuscript syntax, metadata fields, citations, cross-references,
pseudocode, and DOCX table controls, see
[`template/manuscript-syntax.md`](template/manuscript-syntax.md). The same
document has a peer `Style Metadata` section for style-related defaults and
`style.yml` fields.

## Build System

### Using pmt (Recommended)

The package CLI is the preferred entry point for new projects. It can initialize
a manuscript directory, check external tools, and run the existing Pandoc build
pipeline.

```bash
pmt init my-paper     # Create a manuscript project
pmt doctor            # Check Pandoc, pandoc-crossref, Python dependencies, and project files
pmt build docx        # Generate output/docx/manuscript.docx
pmt build latex       # Generate output/latex/manuscript.tex
pmt build json        # Generate output/json/manuscript.json
pmt clean             # Remove generated files
```

Use `pmt build` for non-default inputs:

```bash
pmt build docx paper.md -o build/paper.docx
pmt build latex paper.md -o build/paper.tex
```

For DOCX output, pass `--reference-doc custom-reference.docx` to override the
bundled Word reference document. The option is supported by `pmt build docx`
and `pmt build-reply`.

Reviewer replies can be built with the same DOCX pipeline. The `build-reply` command resolves manuscript cross-references and citations against the manuscript before converting the reply letter:

```bash
pmt build-reply reply.md \
  --reply-manuscript manuscript.md \
  -o output/docx/reply.docx
```

The reply build reads its reply-specific defaults from the `reply:` section in
`style.yml`, while `--manuscript-line-source` defaults to `manuscript.md`. The
line source is only read when the reply uses ``(Line `regex`)`` placeholders.
Markdown line sources are first built to a temporary DOCX; DOCX sources are then
exported to PDF through Word COM on Windows or `soffice --headless --convert-to
pdf` on Linux and other non-Windows systems. Line placeholders are resolved
against the generated PDF text layer. The reply markdown path itself is required.

### Command Options

The `pmt build` command can build a markdown file specified on the command
line. When a markdown file is supplied, the output file name is derived from
that file's stem.

```bash
pmt build docx              # Generate output/docx/manuscript.docx
pmt build latex             # Generate output/latex/manuscript.tex
pmt build json              # Generate output/json/manuscript.json
pmt build docx paper.md     # Generate output/docx/paper.docx
pmt build latex paper.md    # Generate output/latex/paper.tex
pmt build json paper.md     # Generate output/json/paper.json
```

You can also pass the markdown path with `--manuscript` or `-m`:

```bash
pmt build docx --manuscript paper.md
pmt build latex -m paper.md
```

Use `--output-file` or `-o` to choose the exact DOCX, LaTeX, or JSON output path. The build uses the file parent as its output workspace:

```bash
pmt build docx paper.md -o build/paper-final.docx            # Generate build/paper-final.docx
pmt build json paper.md --output-file build/paper.ast.json    # Generate build/paper.ast.json
pmt build latex paper.md -o build/paper.tex                 # Generate build/paper.tex
pmt clean --output-dir build                                  # Remove build/
```

The DOCX post-processing step reads YAML metadata from the same markdown file.
For DOCX, LaTeX, JSON, and reviewer-reply builds, `--output-file` controls the complete output path. The parent directory is also used as the build output workspace. `pmt clean` still accepts `--output-dir` because it removes a generated directory rather than producing one file.

### Direct Pandoc Commands

```bash
# Generate DOCX
pandoc --metadata-file style.yml --defaults pandoc/pandoc-docx.yml

# Generate LaTeX
pandoc --metadata-file style.yml --defaults pandoc/pandoc-latex.yml
```

## Examples

See `template/manuscript.md` for a complete example demonstrating:
- Multi-author affiliations with corresponding author
- Abstract and keywords
- Section organization (Introduction, Methods, Results, Discussion, Conclusion)
- Figure and table cross-references
- Mathematical equations with numbering
- Citations in various formats
- Acknowledgments and supplementary sections

For a real-world example of a complete research paper, see `template/examples/references/paper-specific-example.bib`.

## Acknowledgments

- [Pandoc](https://pandoc.org/) - Universal document converter
- [pandoc-crossref](https://github.com/lierdakil/pandoc-crossref) - Cross-reference filter

## Support

For issues and questions:
- Review [Pandoc documentation](https://pandoc.org/MANUAL.html)
- Create an issue with minimal reproducible example
