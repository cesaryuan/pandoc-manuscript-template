---
name: manuscript-review
description: "This skill should be used when reviewing or improving the quality of an academic manuscript written in Pandoc Markdown. It checks structure, writing style, and formatting against academic writing guidelines, and provides specific actionable suggestions. This skill should be triggered when users ask to review, check, polish, or improve their manuscript."
---

# Manuscript Review

Review the current manuscript file for academic writing quality issues. Read the manuscript, check against the guidelines below in priority order, then report specific issues with locations and suggested fixes.

## Review Procedure

1. Read the full manuscript file (`manuscript.md` or as specified by the user)
2. Check each guideline below in priority order
3. Report findings grouped by priority, with file locations and concrete fix suggestions
4. If the user agrees, apply the fixes

## Priority 1: Critical Structure Issues

Address these first as they affect the entire document.

**1.1 Separate Introduction and Related Work sections**
- Introduction and Related Work must be merged into a single unified Introduction
- Structure: problem/motivation → traditional approaches → modern approaches with citations → research gaps → contributions → paper organization

**1.2 Subsections within Introduction**
- Introduction must be a coherent narrative without any `##` headings
- Remove all subsections and reorganize into flowing paragraphs

**1.3 Excessive `###` (level 3) headings**
- Convert to narrative paragraphs with transitional phrases
- Use "First,...", "Subsequently,...", "Building upon this,..." to connect topics

## Priority 2: Major Flow Issues

Fix after structure is correct.

**2.1 Pseudo-headings** — Bold text used as headers

Detect pattern: `**Label**: content...` at paragraph start. Convert to narrative flow.

Before:
```markdown
**Data Collection**: We collected data from multiple sources...

**Data Cleaning**: The data was preprocessed by removing outliers...
```

After:
```markdown
Data was collected from multiple sources including sensor networks and historical
records. The raw data underwent preprocessing to remove outliers and handle missing
values through interpolation.
```

**2.2 List-heavy writing** — Short bullet lists instead of paragraphs

Convert to narrative using enumeration phrases.

Before:
```markdown
The procedure includes:
- Data preprocessing
- Feature extraction
- Model training
```

After:
```markdown
The procedure consists of three main stages. First, data preprocessing cleans
and normalizes the input. Second, feature extraction captures discriminative
characteristics. Third, model training optimizes the objective function.
```

**2.3 Repetitive figure patterns**

Detect: paragraphs starting with `@fig:label shows/presents/illustrates...`

Fix: lead with narrative content, place figure reference at end of paragraph.
```markdown
The experimental results demonstrate superior performance, with the proposed
method achieving 95% accuracy, as shown in @fig:results.
```

Vary phrasing: "as shown in", "depicted in", "presented in", parenthetical `(@fig:label)`.

## Priority 3: Minor Formatting

Polish after structure and flow are fixed.

**3.1 Bold text overuse**
- Allowed: best results in tables, author names in contribution statements
- Forbidden: emphasis in body text, list item labels, pseudo-headings

**3.2 Cross-reference syntax errors**
- Verify all `@fig:`, `@tbl:`, `@eq:`, `@sec:` references point to valid labels

**3.3 Citation formatting**
- Parenthetical: `[@key]` or `[@key1; @key2]`
- Narrative: `@key showed that...`

## Additional Checks

**Paragraph quality:**
- Each paragraph: 3-7 sentences, one main idea, topic sentence first
- Flag single-sentence paragraphs (except transitions)
- Flag overly long paragraphs (>10 sentences)

**Section structure:**
- Introduction: no subsections, integrates related work
- Methods/Results/Discussion: subsections allowed, 3-5 max per section
- Conclusion: usually no subsections

**Numbered lists:**
- Each item should be a complete sentence or paragraph, not a short phrase
