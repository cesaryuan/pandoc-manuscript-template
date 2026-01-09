# Cursor IDE Configuration Examples

This directory contains example Cursor IDE configurations from the original project. These are provided as reference for users who use Cursor IDE.

## Original Configurations

- [`article.mdc`](article.mdc) - LaTeX academic writing assistant (Chinese interface)
- [`article-md.mdc`](article-md.mdc) - Markdown academic writing assistant (Chinese interface)
- [`translate-to-en.md`](translate-to-en.md) - Translation command

## Purpose

These configurations were originally designed to:

- **Provide academic writing guidance** - Enforce academic tone and technical accuracy
- **Ensure Pandoc Markdown compliance** - Follow Pandoc's Markdown syntax specifications
- **Support Pandoc-crossref** - Proper syntax for figures, tables, equations, and citations
- **Bilingual support** - Assist with English content creation (with Chinese instructions)

## Using with Claude Code

Claude Code users can achieve similar functionality without these specific configuration files. Instead, you can:

### 1. Use Project-Specific Prompts

When working on your manuscript, you can prompt Claude Code with instructions like:

```
I'm working on an academic manuscript in Pandoc Markdown format.
Please help me write/revise content with:
- Academic tone and technical accuracy
- Proper Pandoc Markdown syntax (pandoc-crossref for references)
- Clear, concise language appropriate for journal publication
- Proper citation formatting using [@key] syntax
```

### 2. Create a Custom Skill (Advanced)

For users who want persistent academic writing assistance, consider creating a Claude Code skill:

1. Navigate to your Claude Code skills directory
2. Create a new skill for academic writing assistance
3. Include instructions for:
   - Pandoc Markdown syntax requirements
   - Academic writing best practices
   - Citation and cross-reference formatting

See the [Claude Code documentation](https://docs.anthropic.com/claude/docs/claude-code) for details on creating custom skills.

### 3. Reference Key Pandoc Markdown Rules

When Claude Code is assisting with your manuscript, remind it of these key rules:

- **Metadata**: Title, authors, abstract, keywords go in YAML front matter
- **Figures**: `![Caption](path){#fig:label}` - reference with `@fig:label`
- **Tables**: Use `Table: Caption {#tbl:label}` - reference with `@tbl:label`
- **Equations**: `$$ equation $$ {#eq:label}` - reference with `@eq:label`
- **Sections**: `# Section {#sec:label}` - reference with `@sec:label`
- **Citations**: `[@key]` for parenthetical, `@key` for narrative

## Adapting for Other IDEs

### VS Code

VS Code users can use:
- [Copilot](https://github.com/features/copilot) with custom instructions in `.github/copilot-instructions.md`
- [Continue](https://continue.dev/) with custom prompts
- [Cody](https://sourcegraph.com/cody) with context-aware assistance

### Cursor IDE

To use these original configurations in Cursor:

1. Copy the relevant `.mdc` files to your `.cursor/rules/` directory
2. Modify the `globs` pattern if needed (e.g., `*.md` for all Markdown files)
3. Customize the instructions for your specific needs
4. Restart Cursor IDE to apply changes

## File Descriptions

### article-md.mdc

This rule applies to all Markdown files and enforces:

- Academic writing style for Markdown manuscripts
- Pandoc Markdown syntax compliance
- Proper metadata structure in YAML front matter
- Correct cross-reference and citation syntax
- Pandoc-crossref integration

**Language**: Chinese (instructions for AI assistant)
**Applicability**: `*.md` files

### article.mdc

This rule applies to LaTeX files (not included in this template by default, as we generate LaTeX from Markdown).

**Language**: Chinese (instructions for AI assistant)
**Applicability**: `*.tex` files

### translate-to-en.md

A command for translating content from Chinese to English. This was specific to the original bilingual workflow.

## Notes

- These configurations are **examples** and may need customization for your workflow
- The instructions are in **Chinese** - you may want to translate them to your preferred language
- Claude Code and other modern AI assistants can provide similar functionality without these specific files
- Consider these as inspiration for creating your own IDE-specific configurations

## Questions?

For more information on:
- **Pandoc Markdown syntax**: See the main [README.md](../../README.md) in the project root
- **Claude Code skills**: See [Claude Code documentation](https://docs.anthropic.com/claude/docs/claude-code)
- **Cursor IDE rules**: See [Cursor documentation](https://cursor.sh/docs)
