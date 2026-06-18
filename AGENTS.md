This repository is the `pmt` tool, not a manuscript project.

- Core implementation lives in `src/pandoc_manuscript/`.
- Template content for generated paper projects lives in `template/`.
- Keep changes scoped to the file being edited. Do not rewrite generated output or cache directories unless the user explicitly asks.
- When changing CLI behavior, template copying, or packaging, update the corresponding docs and run a focused syntax check or smoke test.
- Keep comments and docstrings brief, and only add them where they explain a special case or a bug fix.
