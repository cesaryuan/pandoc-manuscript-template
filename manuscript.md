---
title: "Your Manuscript Title: A Comprehensive Study"
authors:
  - name: First Author
    email: first.author@university.edu
    affiliation: 
      - Department of Example, University Name, City, Country
      - Institute of Research, University Name, City, Country

  - name: Second Author
    email: second.author@university.edu
    affiliation: 
      - Department of Example, University Name, City, Country
      - Institute of Another Research, University Name, City, Country
    corresponding: true

abstract: |
  This is a template for academic manuscripts using Pandoc. Replace this abstract with your own summary of the research. The abstract should concisely describe the research problem, methodology, key results, and conclusions. Aim for 150-250 words depending on journal requirements. This template demonstrates proper formatting for figures, tables, citations, cross-references, and equations using Pandoc Markdown syntax.

keywords: [Keyword 1, Keyword 2, Keyword 3, Keyword 4, Keyword 5]

# Bibliography configuration
bibliography: examples/references/sample-references.bib
reference-section-title: References
link-citations: true
# Citation Style Language (CSL) file
# Default: Vancouver style (numeric citations)
# Common alternatives:
#   - pandoc/csl/elsevier-vancouver.csl (Vancouver)
#   - pandoc/csl/engineering-applications-of-artificial-intelligence.csl (EA-AI journal)
#   - Or download from https://www.zotero.org/styles
csl: pandoc/csl/sage-vancouver.csl

# Pandoc-crossref settings for cross-references
## Item caption prefix format
figureTitle: "Figure "
tableTitle: "Table "
titleDelim: ""
## Item cross-reference format
figPrefix: "Figure"
tblPrefix: "Table"
secPrefix: "Section"
eqnPrefix: "Equation"
# Support for centered equations and right-aligned numbering
tableEqns: true
eqnBlockTemplate: |
   `<w:pPr><w:tabs><w:tab w:val="center" w:leader="none" w:pos="4153" /><w:tab w:val="right" w:leader="none" w:pos="9360" /></w:tabs></w:pPr><w:r><w:tab /></w:r>`{=openxml} $$t$$ `<w:r><w:tab /></w:r>`{=openxml} $$nmi$$
eqnBlockInlineMath: true    # Support for centered equations and right-aligned numbering
## Options for section numbering and auto-labeling
linkReferences: true        # Convert references to hyperlinks
autoSectionLabels: true     # Auto-add "sec:" prefix to section labels
autoEqnLabels: true         # Number all display equations
numberSections: true        # Number sections. Add 1.1., 1.2., etc.
sectionsDepth: 3            # Depth of numbered sections
subfigGrid: true            # Subfigure grid layout support
subfigureChildTemplate: ($$i$$) $$t$$
subfigureTemplate: $$figureTitle$$ $$i$$$$titleDelim$$ $$t$$

# =============================================================================
# JOURNAL-SPECIFIC CONFIGURATION (Uncomment and customize as needed)
# =============================================================================
# --- Example 1: Wiley Journal ---
# documentclass: WileyNJDv5
# classoption: [HARVARD, Times2COL]
# header-includes:
# - |
#   ```{=latex}
#   \journal{Journal Abbrev.}
#   \volume{00}
#   \copyyear{2025}
#   \startpage{1}
#   \articletype{RESEARCH ARTICLE}
#   \author[1]{First Author}
#   \author[2]{Second Author}
#   \authormark{AUTHOR \textsc{et al.}}
#   \titlemark{Short Title for Header}
#   \address[1]{\orgdiv{Department}, \orgname{University}, \orgaddress{\city{City}, \country{Country}}}
#   \corres{Corresponding Author, Department, University, City, Country. \email{email@university.edu}}
#   ```
# =============================================================================

---

# Introduction {#sec:introduction}

This template demonstrates the structure and features of an academic manuscript written in Pandoc Markdown. Replace this content with your own research introduction.

The introduction should provide a coherent narrative that guides readers from the general context to your specific research. Start by establishing the broader research area and its importance. You can cite previous work to provide context [@smith2023machine], cite multiple works together [@johnson2022data; @chen2024neural], or refer to specific pages [@garcia2022open, p. 237].

Previous research has shown interesting results [@williams2023dataset], which motivates the current investigation. For a comprehensive review of the field, see @miller2023survey. While significant progress has been made, several challenges remain unaddressed.

This study addresses three key research questions: (1) What is the primary research question? (2) What secondary questions support the main investigation? (3) What practical implications can be derived from the findings?

The main contributions of this work are threefold. First, we propose [describe first contribution]. Second, we provide [describe second contribution]. Third, we demonstrate [describe third contribution]. The remainder of this paper is organized as follows: @sec:methods describes the methodology, @sec:results presents experimental results, [@sec:discussion] discusses the findings and limitations, and @sec:conclusion concludes the paper.

# Methods {#sec:methods}

This section describes the methodology employed in the research.

## Experimental Setup

@tbl:setup summarizes the experimental configuration used in this study.

| **Parameter**      | **Value**          | **Description**                    |
|--------------------|--------------------|------------------------------------|
| Dataset size       | 10,000 samples     | Total number of observations       |
| Training split     | 70%                | Portion used for model training    |
| Validation split   | 15%                | Portion used for hyperparameter tuning |
| Test split         | 15%                | Portion used for final evaluation  |
| Cross-validation   | 5-fold             | Number of folds for CV             |

: Experimental setup configuration {#tbl:setup}

The table above demonstrates basic table formatting. For DOCX output, you can use advanced formatting features by adding metadata to table captions or using cell merge markers (see examples in @tbl:advanced-formatting and @tbl:merged-cells).

## Advanced Table Formatting (DOCX Only)

When generating DOCX output with `make docx`, you can use special features for enhanced table formatting.

### Table Metadata

Add metadata to control table properties. The metadata is automatically removed from the final caption:

| **Property**     | **Value**          | **Description**                    |
|------------------|--------------------|------------------------------------|
| Cell margins     | 0.10 cm            | Padding inside cells               |
| Cell spacing     | 0 pt               | Space between cells                |
| Autofit          | Window             | Table width adjustment             |
| Alignment        | Center             | Table position on page             |

: Table formatting properties. |cell_margin=0.10cm cell_spacing=0pt autofit=window alignment=center| {#tbl:advanced-formatting}

The caption above includes metadata `|cell_margin=0.10cm cell_spacing=0pt autofit=window alignment=center|` which will be applied to the table and then removed in the final DOCX.

### Cell Merging

Use special markers to merge cells: `!<!` merges left, `!^!` merges up:

| **Category** | **Subcategory** | **Value** | **Notes**        |
|:------------:|:---------------:|:---------:|:----------------:|
| Group A      | Item 1          | 10        | First item       |
| !^!          | Item 2          | 20        | Second item      |
| Group B      | Item 3          | 30        | Third item       |
| !^!          | Item 4          | 40        | !<!              |

: Example of merged cells using markers. {#tbl:merged-cells}

In @tbl:merged-cells, "Group A" spans two rows, "Group B" spans two rows, and the last row has a merged cell spanning two columns.

## Mathematical Formulation

The proposed approach can be expressed mathematically. Consider a function $f(x)$ defined as:

$$
f(\boldsymbol{x}) = \sum_{i=1}^{n} w_i \boldsymbol{x_i + b}
$$ {#eq:linear}

where $w_i$ represents the weight parameters, $x_i$ are input features, and $b$ is the bias term. The optimization objective minimizes the loss function $\mathcal{L}$:

$$
\mathcal{L}(\theta) = \frac{1}{N} \sum_{j=1}^{N} \ell(y_j, \hat{y}_j) + \lambda R(\theta)
$$ {#eq:loss}

where $\ell(\cdot)$ is the per-sample loss, $R(\theta)$ is a regularization term, and $\lambda$ controls the regularization strength. See @sec:results for empirical validation of @eq:loss.

## Procedure

The experimental procedure consists of four main stages. First, we perform data preprocessing to clean and normalize the input data, removing outliers and handling missing values according to [specific criteria]. Second, feature extraction is conducted using [method name], which captures [specific characteristics] from the raw data. Third, model training is performed by optimizing @eq:linear and @eq:loss using [optimization algorithm] with [specific hyperparameters]. Finally, we evaluate the trained model using the metrics and protocols described in @sec:results.

# Results {#sec:results}

This section presents the experimental results and analysis.

## Quantitative Results

@tbl:results presents the quantitative comparison of different approaches.

To illustrate how standard figures are inserted and referenced in this template, we include a synthetic trend chart in @fig:single-example. The example uses an image stored under `examples/images/`, which is convenient for demonstrating relative paths in a reusable template repository.

![A single-figure example showing a synthetic performance trend across evaluation steps.](examples/images/single-figure-example.png){#fig:single-example width=85%}

Multi-panel layouts can be prepared with the built-in subfigure grid support enabled in the YAML header. A simple two-panel example is provided in @fig:subfigure-example to show how child figures can share one main caption while still keeping individual labels. In this layout, the left panel (@fig:subfigure-a) can be used to present one condition or ablation case, while the right panel (@fig:subfigure-b) can show the corresponding comparison setting.

<div id="fig:subfigure-example">
![Left panel showing one synthetic subfigure example.](examples/images/subfigure-a-example.png){#fig:subfigure-a width=49%}
![Right panel showing another synthetic subfigure example.](examples/images/subfigure-b-example.png){#fig:subfigure-b width=49%}

An example of a multi-subfigure layout using two synthetic panels.
</div>

| **Method**      | **Accuracy (%)** | **Precision (%)** | **Recall (%)** | **F1-Score (%)** |
|-----------------|------------------|-------------------|----------------|------------------|
| Baseline        | 78.3             | 76.5              | 79.2           | 77.8             |
| Method A        | 85.7             | 84.2              | 86.5           | 85.3             |
| Method B        | 89.1             | 88.3              | 89.8           | 89.0             |
| **Proposed**    | **92.4**         | **91.7**          | **93.1**       | **92.4**         |

: Performance comparison across different methods. Best results in **bold**. {#tbl:results}

As shown in @tbl:results, the proposed method achieves superior performance across all metrics, with accuracy improvements of 14.1 percentage points over the baseline.

## Statistical Significance

We performed statistical significance testing using paired t-tests ($\alpha = 0.05$). The improvements shown in @tbl:results are statistically significant ($p < 0.001$) compared to all baseline methods.

# Discussion {#sec:discussion}

## Interpretation of Results

The results presented in @sec:results demonstrate the effectiveness of the proposed approach. The performance gains can be attributed to three main factors. First, the improved feature representation described in @sec:methods enables the model to capture more discriminative information from the input data. Second, the optimized training procedure employing the loss function in @eq:loss effectively balances prediction accuracy and generalization capability. Third, the robust evaluation methodology outlined in @tbl:setup ensures that the performance estimates are reliable and reproducible across different experimental conditions.

## Comparison with Related Work

Previous work by @anderson2021deep achieved 87.2% accuracy on similar tasks, which is lower than our proposed method's 92.4% (@tbl:results). The approach by @chen2024neural reported comparable precision but lower recall.

Recent theoretical work [@lee2023chapter] provides a framework that helps explain our empirical results.

## Limitations

This study has several limitations that should be acknowledged. The results are based on a specific dataset, and generalization to other domains or application contexts requires further empirical validation. Additionally, the proposed method requires more computational resources than simpler baseline approaches, which may limit its applicability in resource-constrained environments. Performance may also vary with different hyperparameter configurations, requiring careful tuning for optimal results in new problem settings.

## Future Directions

Several promising directions exist for future research. Extension to larger-scale datasets would help validate the scalability and robustness of the proposed approach. Integration with recent advances in [related field] could potentially enhance performance further. Investigating deployment considerations for real-world applications, including computational efficiency and system integration challenges, would facilitate practical adoption. Finally, systematic investigation of failure cases and edge conditions would provide deeper insights into the method's limitations and guide future improvements.

# Conclusion {#sec:conclusion}

This paper presents [brief summary of main contribution]. The proposed approach achieves [key result] as demonstrated in @tbl:results, with improvements of [specific metrics] over existing baselines. The methodology described in @sec:methods provides a systematic framework for [application domain], while the experimental validation in @sec:results confirms its effectiveness across multiple evaluation criteria.

This Pandoc Markdown template supports automatic DOCX formatting, cross-references to tables, sections, and equations, flexible citation styles using CSL, mathematical notation, and optional LaTeX source generation. Users should modify the YAML header to adjust formatting and citation styles according to their target journal requirements.

## Data and Code Availability {.unnumbered}

[Optionally include information about data and code availability, following your target journal's requirements]

# Acknowledgments {.unnumbered}

This work was supported by [funding source]. We thank [individuals or organizations] for [specific contributions].

# Author Contributions {.unnumbered}

**First Author**: Conceptualization, Methodology, Writing - Original Draft
**Second Author**: Investigation, Formal Analysis, Writing - Review & Editing
**Third Author**: Resources, Supervision, Funding Acquisition

# Conflict of Interest {.unnumbered}

The authors declare no conflict of interest.
