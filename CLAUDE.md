---
version: 1.1.0
last_updated: 2026-01-14
---

# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when helping users write academic papers using this Pandoc manuscript template.

## Table of Contents

- [Quick Start](#quick-start) - Fast reference for common tasks
- [Assistance Priority Guide](#assistance-priority-guide) - What to fix first
- [Critical Don'ts](#critical-donts---what-not-to-do) - Non-negotiable rules
- [Pandoc Markdown Syntax Guide](#pandoc-markdown-syntax-guide) - Syntax reference
- [Academic Writing Style Guidelines](#academic-writing-style-guidelines) - Style rules
- [Common Assistance Scenarios](#common-assistance-scenarios) - Task-based help

## Quick Start

### Overview

This template helps academic writers create manuscripts in Pandoc Markdown format and convert them to DOCX (for journal submission) or LaTeX/PDF (for archival).

**Starting a new manuscript:**
- Users should copy `manuscript.example.md` to create their own manuscript file (e.g., `manuscript.md`)
- The `manuscript.example.md` file provides a complete template with proper structure, syntax examples, and writing style
- When helping users start a new paper, reference the structure and conventions in `manuscript.example.md`

### Quick Reference Card

**Common Syntax:**
- **Figures**: `![caption](path){#fig:label}` → reference: `@fig:label`
- **Tables**: Caption with `: Description {#tbl:label}` → reference: `@tbl:label`
- **Citations**: `[@key]` (parenthetical), `@key` (narrative), `[@key1; @key2]` (multiple)
- **Equations**: `$$ math $$ {#eq:label}` → reference: `@eq:label`; inline: `$ math $`
- **Sections**: `# Title {#sec:label}` → reference: `@sec:label`

**Writing Rules:**
- ❌ Subsections in Introduction
- ❌ Bold as pseudo-headings
- ❌ Short bullet lists
- ❌ Separate "Related Work" section
- ✅ Narrative paragraphs (3-7 sentences)
- ✅ Unified Introduction with Related Work integrated
- ✅ Lead with narrative, end with figure reference

## Assistance Priority Guide

When helping users, follow this priority order:

### Priority 1: Critical Structure Issues
Address these first as they affect the entire document structure:
1. **Separate Introduction and Related Work sections** → Merge into unified Introduction
2. **Subsections within Introduction** → Remove and reorganize into narrative
3. **Excessive ### (level 3) headings** → Convert to narrative paragraphs

### Priority 2: Major Flow Issues
Fix these after structure is correct:
4. **Pseudo-headings** (bold text used as headers like "**Data Collection**: ...") → Convert to narrative
5. **List-heavy writing** (short bullet lists instead of paragraphs) → Transform to narrative
6. **Repetitive figure patterns** (every paragraph starts with "@fig:label shows...") → Integrate naturally

### Priority 3: Minor Formatting
Polish these after structure and flow are fixed:
7. **Bold text overuse** → Remove unnecessary bold formatting
8. **Cross-reference syntax errors** → Fix `@fig:`, `@tbl:`, `@eq:`, `@sec:` usage
9. **Citation formatting** → Ensure proper `[@key]` or `@key` syntax

## Critical Don'ts - What NOT to Do

🚫 **NEVER do these - they are non-negotiable violations of academic writing standards:**

1. **Add subsections (##) under Introduction** - Introduction must be a coherent narrative without section headings
2. **Use bold text as paragraph headers** - No "**Data Collection**: ..." patterns
3. **Replace narrative with short bullet lists** - Use flowing paragraphs instead
4. **Start paragraphs with "@fig:label shows..."** - Lead with narrative, reference at end
5. **Create separate "Related Work" section** - Merge into unified Introduction in engineering papers
6. **Use excessive ### (level 3) headings** - Convert to narrative within ## sections
7. **Add single-sentence paragraphs** - Combine into coherent units (except for transitions)
8. **Use bold for emphasis in body text** - Reserve bold only for table results and contribution statements

## Pandoc Markdown Syntax Guide

### Tables

Create tables using pipe syntax with a caption below.

**Important table conventions:**
- Use **bold** only for highlighting best results in comparison tables
- Avoid excessive formatting (colors, merged cells) - keep tables simple
- Use `**bold**` for column headers if needed: `| **Header 1** | **Header 2** |`
- Use `:--` for left alignment, `:--:` for center alignment (default), and `--:` for right alignment
- For advanced typesetting requirements, refer to `### Advanced Table Formatting (DOCX Post-Processing)` in README.md

**Syntax:**
```markdown
| **Method**  | **Accuracy (%)** | **F1-Score (%)** |
|:-----------:|:----------------:|:----------------:|
| Baseline    | 78.3             | 77.8             |
| **Proposed**| **92.4**         | **92.4**         |

: Performance comparison. Best results in **bold**. {#tbl:results}

As shown in @tbl:results, ...
```

### Bibliography

**Bibliography file** - Specify in the YAML header of `manuscript.md`:
```yaml
bibliography: path/to/references.bib
```

## Academic Writing Style Guidelines

### 1. Introduction Structure

**DO NOT** divide Introduction into subsections. The introduction should be a coherent narrative without section headings.

**Good example:**
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

### 2. Bold Text Usage - Complete Guide

Use **bold** sparingly in academic writing. This is the definitive guide for all bold text usage.

✅ **Appropriate uses:**
- Highlighting best results in tables: `| **Proposed** | **92.4** |`
- Author names in contribution statements: `**First Author**: Conceptualization`

❌ **Avoid:**
- Bolding list item labels: ~~`**Data preprocessing**: Clean data`~~
- Emphasizing concepts in text: ~~`The **main advantage** is...`~~
- Section-like headers within paragraphs (pseudo-headings): ~~`**Data Collection**: We collected...`~~
- Any use of bold as a substitute for proper section headings

**Note:** Other guidelines reference this section for bold text rules.

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

**Exception:** Bullet lists are acceptable when listing specific technical details or enumerated items where narrative would be awkward.

### 4. Numbered Lists Should Have Substantial Content

When using numbered lists, each item should be a complete sentence or paragraph, not a short phrase.

**Good example:**
```markdown
This study has three main limitations. (1) The results are based on a specific
dataset, and generalization to other domains requires further validation.
(2) The proposed method requires more computational resources than simpler baselines,
which may limit applicability in resource-constrained environments. (3) Performance
may vary with different hyperparameter configurations.
```

**Or as a narrative paragraph:**
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

**Important:** Always check cross-reference syntax: `@fig:label`, `@tbl:label`, `@eq:label`, `@sec:label`

### 7. Avoid Pseudo-Headings and Excessive Subsections

**CRITICAL RULE:** Never use bold text as pseudo-headings within paragraphs. Sections should flow as coherent narrative paragraphs, not as fragmented lists with bold labels. See [Guideline #2](#2-bold-text-usage---complete-guide) for complete bold text rules.

❌ **Bad example** (pseudo-headings):
```markdown
## Methods

### Data Processing

**Data Collection**: We collected data from multiple sources...

**Data Cleaning**: The data was preprocessed by removing outliers...

**Feature Extraction**: Features were extracted using the following approach...
```

✅ **Good example** (narrative flow):
```markdown
## Methods

Data was collected from multiple sources including sensor networks and historical records. The raw data underwent preprocessing to remove outliers and handle missing values through interpolation. Subsequently, feature extraction was performed using principal component analysis, which identified the most discriminative characteristics for classification.
```

**Guidelines for subsection hierarchy:**

- **Level 1 (`#`)**: Main sections (Introduction, Methods, Results, etc.)
  - **No subsections** for: Introduction, Conclusion
  - **Subsections allowed** for: Methods, Results, Discussion

- **Level 2 (`##`)**: Major subsections within Methods/Results/Discussion
  - Should represent logically distinct components
  - **Avoid excessive fragmentation** - aim for 3-5 substantial subsections maximum
  - Each subsection should be substantial (multiple paragraphs)

- **Level 3 (`###`)**: Generally **AVOID** unless absolutely necessary
  - If you find yourself needing level 3 headings, **reconsider the structure**
  - Instead, use narrative transitions between topics within level 2 sections
  - If level 3 is truly needed, immediately convert content to flowing paragraphs

**Refactoring strategy when encountering excessive subsections:**

1. **Identify the main concepts** under each level 2 heading
2. **Merge related sub-topics** into coherent narrative paragraphs
3. **Use transitional phrases** to connect concepts:
   - "First, ... Second, ... Third, ..."
   - "To address this, ..."
   - "Subsequently, ..."
   - "Building upon this, ..."
4. **Eliminate bold pseudo-headings** entirely
5. **Preserve technical content** while improving flow

**Example transformation:**

Before (over-structured):
```markdown
## Methodology

### Data Collection
We collected data from sensors.

### Data Preprocessing
**Outlier Removal**: Outliers were removed using IQR method.
**Normalization**: Data was normalized to [0,1] range.

### Feature Engineering
**Feature Selection**: We selected top 10 features.
**Feature Transformation**: Features were transformed using PCA.
```

After (narrative):
```markdown
## Methodology

Data collection was performed using calibrated sensors deployed at strategic locations. The collected data underwent preprocessing to ensure quality and consistency. Outliers were identified and removed using the interquartile range method, and the remaining values were normalized to the [0,1] range for uniform scaling.

Feature engineering involved two complementary steps. First, feature selection identified the ten most discriminative variables using mutual information criterion. Second, principal component analysis transformed the selected features into an orthogonal representation, reducing dimensionality while preserving variance.
```

### 8. Paragraph Length and Coherence

Each paragraph should:
- Focus on **one main idea** or closely related concepts
- Contain **3-7 sentences** typically
- Start with a **topic sentence** establishing the main point
- Use **transitions** to connect with preceding/following paragraphs
- **Avoid single-sentence paragraphs** except for transitions or emphasis

Combine short, fragmented paragraphs into coherent units. Break overly long paragraphs (>10 sentences) into logical subdivisions.

### 9. Integrating Introduction and Related Work

In engineering and technical papers, Introduction and Related Work are typically **merged into a single unified Introduction section** rather than separated.

**Structure of unified Introduction:**
1. Problem statement and motivation (1-2 paragraphs)
2. Traditional approaches and their limitations (1 paragraph)
3. Deep learning/modern approaches with literature review (2-4 paragraphs)
   - Organize by approach type (CNNs, RNNs, GANs, Diffusion models, etc.)
   - Cite relevant work naturally within narrative flow
   - Highlight advances and remaining limitations
4. Research gaps and motivation (1 paragraph)
5. Contributions and paper organization (1 paragraph)

**Good example structure:**
```markdown
# Introduction {#sec:introduction}

[Problem and motivation...]

Traditional methods include... but fail to...

Deep learning has emerged as... [@cite1; @cite2]. Specifically, CNNs have been applied to... [@cite3], while RNNs demonstrate... [@cite4]. More recently, diffusion models... [@cite5; @cite6].

Despite these advances, existing methods lack...

This study addresses... The key contributions are threefold...

The remainder of this paper is organized as follows: @sec:methods...
```

**Avoid:**
- Separate "# Related Work" section (use unified Introduction instead)
- Subsections within Introduction (## Background, ## Prior Work, etc.)
- Exhaustive literature review without connecting to motivation

### 10. Natural Figure Integration

Avoid monotonous figure presentation patterns. Integrate figures naturally into narrative flow rather than starting paragraphs with figure references.

❌ **Bad pattern** (monotonous):
```markdown
![Caption](image.png){#fig:label}

@fig:label presents the results showing...

![Another caption](image2.png){#fig:label2}

@fig:label2 shows the performance across...

![Yet another](image3.png){#fig:label3}

@fig:label3 illustrates the architecture of...
```

✅ **Good pattern** (natural integration):
```markdown
The experimental results demonstrate superior performance across all
scenarios, with the proposed method achieving 95% accuracy. Detailed
comparisons across different datasets are shown in @fig:results.

![Performance comparison across datasets](results.png){#fig:results}

The network architecture consists of three main components: an encoder
for feature extraction, a transformer for sequence modeling, and a
decoder for reconstruction, as illustrated in @fig:architecture.

![Network architecture diagram](arch.png){#fig:architecture}
```

**Guidelines:**
- **Lead with narrative**, end with figure reference: "...descriptive text, as shown in @fig:label."
- **Place figure after** the paragraph that describes/references it
- **Vary sentence structures** - don't always use "as shown in" or "illustrated in"
  - "...is depicted in @fig:label"
  - "...are presented in @fig:results"
  - "(@fig:comparison)"
  - "...demonstrated in @fig:analysis"
- **Integrate figure content** into the narrative before referencing
- **Avoid** starting paragraphs with "@fig:label presents/shows/illustrates..."

## Common Assistance Scenarios

Scenarios are organized by priority (see [Assistance Priority Guide](#assistance-priority-guide)).

### High Priority Scenarios

#### Scenario 1: User wants to start a new manuscript from scratch

1. Copying `manuscript.example.md` as a starting point: `cp manuscript.example.md manuscript.md`
2. Reference the structure in `manuscript.example.md` for proper academic writing style
3. Guide them to replace placeholder content with their own research
4. Ensure they update the YAML header with their title, authors, and bibliography path


## Important Reminders

- Always check cross-reference syntax: `@fig:label`, `@tbl:label`, `@eq:label`, `@sec:label`
- Image paths should be relative to `manuscript.md` location
- Citation keys must match entries in the `.bib` file exactly
- Academic writing prioritizes clarity and formal tone over stylistic flourishes
- When in doubt, follow conventions of the user's target journal or field
- Always preserve technical content while improving structure and flow