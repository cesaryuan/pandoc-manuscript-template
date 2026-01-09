# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when helping users write academic papers using this Pandoc manuscript template.

## Overview

This template helps academic writers create manuscripts in Pandoc Markdown format and convert them to DOCX (for journal submission) or LaTeX/PDF (for archival).

**Starting a new manuscript:**
- Users should copy `manuscript.example.md` to create their own manuscript file (e.g., `manuscript.md`)
- The `manuscript.example.md` file provides a complete template with proper structure, syntax examples, and writing style
- When helping users start a new paper, reference the structure and conventions in `manuscript.example.md`

## Pandoc Markdown Syntax Guide

### Figures

Insert figures with captions and labels for cross-referencing:

Reference figures in text: `@fig:label` or `see @fig:label`

Example:
```markdown
![Comparison of model performance across different datasets.](images/results.png){#fig:performance}

As shown in @fig:performance, the proposed method outperforms baseline approaches.
```

### Tables

Create tables using pipe syntax with a caption below. Reference tables in text: `@tbl:label`

**Important table conventions:**
- Use **bold** only for highlighting best results in comparison tables
- Avoid excessive formatting (colors, merged cells) - keep tables simple
- Use `**bold**` for column headers if needed: `| **Header 1** | **Header 2** |`

Example:
```markdown
| **Method**  | **Accuracy (%)** | **F1-Score (%)** |
|-------------|------------------|------------------|
| Baseline    | 78.3             | 77.8             |
| **Proposed**| **92.4**         | **92.4**         |

: Performance comparison. Best results in **bold**. {#tbl:results}

As shown in @tbl:results, the proposed method achieves superior performance.
```

### Equations

Inline equations use single dollar signs: `$E = mc^2$`. Display equations use double dollar signs with labels. Reference equations in text: `@eq:label`.

Example:
```markdown
The optimization objective is defined as:

$$
\mathcal{L}() = \frac{1}{N}
$$ {#eq:loss}

where $\ell(\cdot)$ is the loss function. We minimize @eq:loss using gradient descent.
```

### Citations and References

**Bibliography file**: Specify in the YAML header of `manuscript.md`:
```yaml
bibliography: path/to/references.bib
```

**Citation syntax**:
```markdown
[@key]                    # Parenthetical: (Author, 2023)
@key                      # Narrative: Author (2023)
[@key1; @key2]            # Multiple: (Author1, 2023; Author2, 2024)
[@key, p. 42]             # With page: (Author, 2023, p. 42)
[@key1; @key2; @key3]     # Three or more
```

Examples:
```markdown
Previous research has shown promising results [@smith2023].
According to @johnson2024, the method achieves high accuracy.
Multiple studies [@chen2022; @garcia2023; @williams2024] have explored this topic.
For detailed analysis, see @miller2023 [p. 237].
```

**BibTeX format**: Create a `.bib` file with entries like:
```bibtex
@article{smith2023,
  author = {Smith, John and Doe, Jane},
  title = {A Novel Approach to Machine Learning},
  journal = {Journal of AI Research},
  year = {2023},
  volume = {15},
  pages = {123--145}
}
```

### Section Cross-References

Label sections and reference them:

```markdown
# Introduction {#sec:introduction}
# Methods {#sec:methods}
# Results {#sec:results}

Reference: @sec:methods describes the methodology.
```

## Academic Writing Style Guidelines

### 1. Introduction Structure

**DO NOT** divide Introduction into subsections. The introduction should be a coherent narrative without section headings.

**Good example**:
```markdown
# Introduction {#sec:introduction}

[Opening paragraph establishing context...]

Previous research has shown... [@citation1; @citation2]

[Research gap paragraph...]

This study addresses three key questions...

The main contributions are threefold. First, ... Second, ... Third, ...
```

**Bad example** (avoid this):
```markdown
# Introduction {#sec:introduction}

## Background
[content]

## Research Questions
[content]

## Contributions
[content]
```

### 2. Minimal Use of Bold Text

Use **bold** sparingly in academic writing:

✅ **Appropriate uses:**
- Highlighting best results in tables: `| **Proposed** | **92.4** |`
- Author names in contribution statements: `**First Author**: Conceptualization`

❌ **Avoid:**
- Bolding list item labels: ~~`**Data preprocessing**: Clean data`~~
- Emphasizing concepts in text: ~~`The **main advantage** is...`~~
- Section-like headers within paragraphs

### 3. Avoid Bullet Lists, Prefer Narrative

Academic papers use bullet lists sparingly. Prefer narrative paragraphs.

**Good example** (narrative):
```markdown
The procedure consists of four main stages. First, we perform data preprocessing
to clean and normalize the input data. Second, feature extraction is conducted
using [method], which captures [characteristics] from the data. Third, model
training optimizes @eq:loss using [algorithm]. Finally, we evaluate performance
using the metrics described in @sec:results.
```

**Bad example** (avoid short bullet lists):
```markdown
The procedure includes:
- Data preprocessing
- Feature extraction
- Model training
- Evaluation
```

**Exception**: Bullet lists are acceptable when listing specific technical details or enumerated items where narrative would be awkward.

### 4. Numbered Lists Should Have Substantial Content

When using numbered lists, each item should be a complete sentence or paragraph, not a short phrase.

**Good example**:
```markdown
This study has three main limitations. (1) The results are based on a specific
dataset, and generalization to other domains requires further validation.
(2) The proposed method requires more computational resources than simpler baselines,
which may limit applicability in resource-constrained environments. (3) Performance
may vary with different hyperparameter configurations.
```

Or as a narrative paragraph:
```markdown
This study has several limitations that should be acknowledged. The results are
based on a specific dataset, and generalization to other domains requires further
validation. Additionally, the proposed method requires more computational resources...
```

### 5. Standard Section Structure

Typical academic paper sections:
- **Introduction** (no subsections)
- **Methods** (with subsections like "Experimental Setup", "Procedure")
- **Results** (with subsections like "Quantitative Results", "Statistical Analysis")
- **Discussion** (with subsections like "Interpretation", "Limitations", "Future Work")
- **Conclusion** (usually no subsections)

### 6. Cross-Reference Usage

Use cross-references liberally to connect sections:
```markdown
As described in @sec:methods, we employ...
The results in @tbl:results demonstrate...
By optimizing @eq:loss, we achieve...
See @sec:discussion for detailed analysis.
```

## How to Generate Output

Users can generate DOCX and PDF outputs using these commands:

```bash
make docx          # Creates output/docx/manuscript.docx
make pdf           # Creates output/manuscript.pdf
```

## Common Assistance Scenarios

### Scenario 1: User wants to start a new manuscript from scratch

Help them:
1. Suggest copying `manuscript.example.md` as a starting point: `cp manuscript.example.md manuscript.md`
2. Reference the structure in `manuscript.example.md` for proper academic writing style
3. Guide them to replace placeholder content with their own research
4. Ensure they update the YAML header with their title, authors, and bibliography path

### Scenario 2: User wants to add a figure

Help them:
1. Choose an appropriate label (e.g., `{#fig:results}`)
2. Write a descriptive caption
3. Add cross-references in the text where the figure is discussed

### Scenario 3: User wants to add citations

Help them:
1. Ensure the `.bib` file is specified in YAML header
2. Use appropriate citation syntax (parenthetical vs. narrative)
3. Group related citations together: `[@key1; @key2; @key3]`

### Scenario 4: User's writing is too listy

Transform bullet lists into narrative paragraphs following academic writing conventions:
- Use transition words: "First,", "Second,", "Additionally,", "Finally,"
- Make complete sentences with proper context
- Connect ideas logically

### Scenario 5: User's Introduction has subsections

Advise removing subsections and reorganizing into coherent narrative paragraphs.

### Scenario 6: User overuses bold text

Identify unnecessary bold formatting and suggest removing it, keeping only table emphasis and required formatting.

## Important Reminders

- Always check cross-reference syntax: `@fig:label`, `@tbl:label`, `@eq:label`, `@sec:label`
- Image paths should be relative to `manuscript.md` location
- Citation keys must match entries in the `.bib` file exactly
- Academic writing prioritizes clarity and formal tone over stylistic flourishes
- When in doubt, follow conventions of the user's target journal or field
