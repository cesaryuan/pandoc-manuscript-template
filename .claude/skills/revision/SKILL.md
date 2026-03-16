---
name: revision
description: "This skill should be used when handling journal reviewer comments and preparing a revised manuscript with a point-by-point response letter. This skill should be triggered when users mention reviewer comments, revision, rebuttal, or response letter."
---

# Revision

Help the user respond to journal reviewer comments by preparing a point-by-point response letter and applying corresponding changes to the manuscript.

## Workflow

1. Read the reviewer comments (provided by user as text or file)
2. Read the current manuscript
3. For each reviewer comment, draft a response and identify needed manuscript changes
4. Present the response letter draft to the user for review
5. After user approval, apply changes to the manuscript

## Response Letter Format

Use the following structure for the response letter (Markdown format):

```markdown
# Response to Reviewers

We thank the reviewers for their constructive comments. Below we address each
comment point by point. Reviewer comments are in **bold**, and our responses
follow each comment. All changes in the manuscript are highlighted in blue.

## Reviewer 1

**Comment 1: [Summarize the reviewer's comment]**

[Response explaining what was done and why]

[If applicable: "We have revised Section X to clarify this point (see lines XX-XX)."]

**Comment 2: ...**

...

## Reviewer 2

...
```

## Response Writing Guidelines

**Tone:**
- Always respectful and professional, even for critical or unfair comments
- Thank the reviewer for specific insights when genuine
- Never dismissive or defensive

**Response types:**

For comments requiring changes:
- Acknowledge the issue
- Explain what was changed and where
- Quote the new/revised text if brief

For comments disagreeing with:
- Acknowledge the reviewer's perspective
- Provide evidence or reasoning for the current approach
- Offer a compromise if possible (e.g., adding a discussion paragraph)

For comments requesting additional experiments:
- If feasible: perform and report results
- If not feasible: explain why and propose alternatives

**Manuscript changes:**
- Track all changes clearly — note which section/paragraph was modified
- Ensure changes are consistent across the manuscript (e.g., if terminology changes, update everywhere)
- Verify cross-references remain valid after restructuring

## Common Revision Patterns

**Adding clarification:** Insert 1-2 sentences at the relevant location, maintain paragraph flow.

**Restructuring a section:** Follow the academic writing rules in CLAUDE.md — no pseudo-headings, narrative paragraphs, proper transitions.

**Adding references:** Add to `.bib` file and cite with `[@key]` syntax. Verify the key doesn't conflict with existing entries.

**Updating figures/tables:** Update both the figure/table and any text that describes or references it.
