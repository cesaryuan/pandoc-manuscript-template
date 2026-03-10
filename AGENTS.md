---
version: 2.0.0
last_updated: 2026-03-03
---

# CLAUDE.md

Guidance for helping users write academic papers using this Pandoc manuscript template.

## Project Overview

This template converts Pandoc Markdown manuscripts to DOCX (journal submission) or LaTeX/PDF (archival).

- Main file: `manuscript.md` — edit this to write the paper
- Build: `make docx` / `make pdf` / `make help`
- Images: place in `images/` directory
- References: `.bib` file specified in YAML header

## Pandoc Markdown Syntax

**Cross-references:**
- Figures: `![caption](path){#fig:label}` → `[@fig:label]`
- Tables: `: Caption {#tbl:label}` → `[@tbl:label]`
- Equations: `$$ math $$ {#eq:label}` → `[@eq:label]`
- Sections: `# Title {#sec:label}` → `[@sec:label]`
- Citations: `[@key]` (parenthetical), `[@key1; @key2]` (multiple)

**Tables:**
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

**Subfigures** (requires `subfigGrid: true` in YAML):
```markdown
<div id="fig:results">
![caption of a](a.png){#fig:a width=50%} # Only percent allowed in subfigure width
![caption of b](b.png){#fig:b width=50%}

![caption of c](c.png){#fig:c width=50%}
![caption of d](d.png){#fig:d width=50%}

Main caption ( 2x2 grid of subfigures, change line by adding a blank line between images).
</div>
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

**Standard section structure:**
- **Introduction** (no subsections, integrates related work)
- **Methods** (subsections allowed)
- **Results** (subsections allowed)
- **Discussion** (subsections allowed)
- **Conclusion** (usually no subsections)

## Skills

- **`/manuscript-review`** — Detailed writing style guidelines with examples for reviewing and improving manuscript quality
- **`/revision`** — Guidance for handling reviewer comments and writing response letters

## Reminders

- Image paths relative to `manuscript.md` location
- Citation keys must match `.bib` entries exactly
- Always preserve technical content when improving structure and flow
- When in doubt, follow conventions of the user's target journal
