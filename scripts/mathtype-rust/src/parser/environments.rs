use super::*;

struct ParsedUnsupportedEnvironmentBody {
    body: Expr,
    termination: UnsupportedEnvironmentTermination,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum UnsupportedEnvironmentTermination {
    MatchedOwnEnd,
    PendingAncestorClose,
    UnterminatedEof,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum UnsupportedEnvironmentShellStyle {
    SplitNameVisible,
    RawDelimitedName,
    RawInlineControlName,
}

impl Parser {
    /// Return true only for starred environments that MathType still treats as supported syntax.
    fn supports_starred_environment_base(name: &str) -> bool {
        matches!(
            name,
            "equation"
                | "matrix"
                | "smallmatrix"
                | "pmatrix"
                | "bmatrix"
                | "Bmatrix"
                | "vmatrix"
                | "Vmatrix"
        )
    }

    /// Parse rows and columns for the supported \begin...\end environments.
    pub(super) fn parse_environment(&mut self, name: &str) -> Result<Expr, String> {
        let checkpoint = self.pos;
        let parsed = (|| -> Result<Expr, String> {
            let base_name = name.strip_suffix('*').unwrap_or(name);
            if name.ends_with('*') && !Self::supports_starred_environment_base(base_name) {
                return self.parse_unsupported_environment(name);
            }
            if name.ends_with('*') && Self::supports_starred_environment_base(base_name) {
                // mathtools matrix* variants accept an optional alignment specifier.
                let _ = self.parse_optional_bracket_group()?;
            }
            let mut parsed_rows;
            let kind = match base_name {
                "equation" => {
                    if name.ends_with('*') {
                        return self.parse_hybrid_wrapper_environment(name);
                    }
                    return self.parse_transparent_wrapper_environment(name);
                }
                "split" => EnvironmentKind::Split,
                "align" => EnvironmentKind::Align,
                "alignat" => {
                    self.parse_raw_group("alignat column count")?;
                    EnvironmentKind::AlignAt
                }
                "aligned" => EnvironmentKind::Aligned,
                "alignedat" => return self.parse_hybrid_wrapper_environment(name),
                "gather" => EnvironmentKind::Gather,
                "gathered" => EnvironmentKind::Gathered,
                "cases" | "dcases" => EnvironmentKind::Cases,
                "rcases" | "drcases" => EnvironmentKind::RightCases,
                "array" => {
                    let column_spec_start = self.pos;
                    let column_spec = self.parse_optional_array_column_spec()?;
                    let bare_column_spec_raw = if column_spec.is_some()
                        && self.chars.get(column_spec_start) != Some(&'{')
                    {
                        Some(
                            self.chars[column_spec_start..self.pos]
                                .iter()
                                .collect::<String>(),
                        )
                    } else {
                        None
                    };
                    if let Some(raw) = &bare_column_spec_raw {
                        if self.pos >= self.chars.len() {
                            // Bug-fix: `\begin{array}a` keeps the recovered bare
                            // preamble token inside the raw fallback shell when the
                            // environment ends immediately at EOF.
                            return Ok(Expr::Sequence(vec![Expr::RawTex(format!(
                                "\\begin{{{name}}}{raw}"
                            ))]));
                        }
                    }
                    parsed_rows = self.parse_array_rows(name)?;
                    if parsed_rows.rows.is_empty() {
                        // Bug-fix: MathType still materializes a 1x1 empty MATRIX for
                        // `array` environments that close immediately after the preamble.
                        parsed_rows.rows.push(vec![Expr::Sequence(Vec::new())]);
                        parsed_rows.row_leading.push(String::new());
                        parsed_rows.separator_leading.push(Vec::new());
                    }
                    return Ok(Expr::Environment {
                        kind: EnvironmentKind::Array,
                        rows: parsed_rows.rows,
                        trivia: EnvironmentTrivia {
                            column_spec,
                            row_leading: parsed_rows.row_leading,
                            separator_leading: parsed_rows.separator_leading,
                            end_leading: parsed_rows.end_leading,
                            row_annotations: parsed_rows.row_annotations,
                        },
                    });
                }
                "subarray" => return self.parse_subarray_environment(name),
                "matrix" => {
                    parsed_rows = self.parse_environment_rows(name)?;
                    return Ok(Expr::Matrix {
                        kind: MatrixKind::Plain,
                        rows: parsed_rows.rows,
                    });
                }
                "smallmatrix" => {
                    parsed_rows = self.parse_environment_rows(name)?;
                    return Ok(Expr::Style {
                        kind: StyleKind::Script,
                        content: Box::new(Expr::Matrix {
                            kind: MatrixKind::Small,
                            rows: parsed_rows.rows,
                        }),
                    });
                }
                "pmatrix" => {
                    parsed_rows = self.parse_environment_rows(name)?;
                    return Ok(Expr::Matrix {
                        kind: MatrixKind::Parenthesized,
                        rows: parsed_rows.rows,
                    });
                }
                "bmatrix" => {
                    parsed_rows = self.parse_environment_rows(name)?;
                    return Ok(Expr::Matrix {
                        kind: MatrixKind::Bracketed,
                        rows: parsed_rows.rows,
                    });
                }
                "Bmatrix" => {
                    parsed_rows = self.parse_environment_rows(name)?;
                    return Ok(Expr::Matrix {
                        kind: MatrixKind::Braced,
                        rows: parsed_rows.rows,
                    });
                }
                "vmatrix" => {
                    parsed_rows = self.parse_environment_rows(name)?;
                    return Ok(Expr::Matrix {
                        kind: MatrixKind::Barred,
                        rows: parsed_rows.rows,
                    });
                }
                "Vmatrix" => {
                    parsed_rows = self.parse_environment_rows(name)?;
                    return Ok(Expr::Matrix {
                        kind: MatrixKind::DoubleBarred,
                        rows: parsed_rows.rows,
                    });
                }
                _ => return self.parse_unsupported_environment(name),
            };
            parsed_rows = self.parse_environment_rows(name)?;
            Ok(Expr::Environment {
                kind,
                rows: parsed_rows.rows,
                trivia: EnvironmentTrivia {
                    column_spec: None,
                    row_leading: parsed_rows.row_leading,
                    separator_leading: parsed_rows.separator_leading,
                    end_leading: parsed_rows.end_leading,
                    row_annotations: parsed_rows.row_annotations,
                },
            })
        })();
        match parsed {
            Ok(expr) => Ok(expr),
            Err(_err) => {
                // Bug-fix: malformed supported environments should degrade to the same
                // raw/hybrid fallback path MathType uses instead of aborting parsing.
                self.pos = checkpoint;
                let shell_style = if base_name_for_environment_shell(name) == "array" {
                    UnsupportedEnvironmentShellStyle::RawDelimitedName
                } else {
                    unsupported_environment_shell_style(name)
                };
                self.parse_unsupported_environment_with_shell(name, shell_style)
            }
        }
    }

    /// Parse transparent wrappers such as non-starred `equation` around real math content.
    pub(super) fn parse_transparent_wrapper_environment(
        &mut self,
        name: &str,
    ) -> Result<Expr, String> {
        let mut items = Vec::new();
        loop {
            self.skip_ws();
            if self.pos >= self.chars.len() {
                // Bug-fix: unterminated transparent wrappers such as `\begin{equation}a`
                // keep the successfully parsed body instead of degrading the whole
                // environment into raw fallback text.
                break;
            }
            if self.starts_end_environment(name) {
                self.consume_end_environment(name)?;
                break;
            }
            if self.starts_command("end") {
                // Bug-fix: a mismatched `\end{...}` inside transparent wrappers stays
                // visible as raw fallback text while the parsed body remains native.
                items.push(self.parse_transparent_wrapper_mismatched_end_environment_fallback()?);
                continue;
            }
            items.push(self.parse_complete_atom()?);
        }
        Ok(Expr::Sequence(items))
    }

    /// Route wrapper-like environments through the hybrid fallback parser when
    /// MathType keeps their `\begin...\end` shell instead of emitting a native layout object.
    pub(super) fn parse_hybrid_wrapper_environment(&mut self, name: &str) -> Result<Expr, String> {
        self.parse_unsupported_environment(name)
    }

    /// Parse one `subarray` environment so bodyless big operators can use the
    /// dedicated MathType mixed raw/native lower-limit fallback.
    fn parse_subarray_environment(&mut self, name: &str) -> Result<Expr, String> {
        let column_spec = self.parse_raw_group("subarray column specifier")?;
        let parsed_rows = self.parse_environment_rows(name)?;
        Ok(Expr::Subarray {
            column_spec,
            rows: parsed_rows.rows,
        })
    }

    /// Parse an optional array column specifier, allowing MathType's default one-column recovery.
    fn parse_optional_array_column_spec(&mut self) -> Result<Option<String>, String> {
        self.skip_ws();
        if self.peek() != Some('{') {
            // Bug-fix: when `array` omits the braced preamble, MathType still consumes
            // one bare preamble token before looking for rows or the closing `\end`.
            return self.parse_bare_array_column_spec();
        }
        self.parse_raw_group("array column specifier").map(Some)
    }

    /// Parse MathType's unbraced one-token `array` preamble recovery.
    fn parse_bare_array_column_spec(&mut self) -> Result<Option<String>, String> {
        if self.pos >= self.chars.len()
            || self.peek() == Some('&')
            || self.starts_row_separator()
            || self.starts_command("end")
        {
            return Ok(None);
        }
        let start = self.pos;
        let mut brace_depth = 0usize;
        while let Some(ch) = self.peek() {
            if brace_depth == 0
                && (ch.is_whitespace()
                    || ch == '&'
                    || self.starts_row_separator()
                    || self.starts_command("end"))
            {
                break;
            }
            match ch {
                '{' => brace_depth += 1,
                '}' if brace_depth > 0 => brace_depth -= 1,
                _ => {}
            }
            self.pos += 1;
        }
        if self.pos == start {
            return Ok(None);
        }
        Ok(Some(self.chars[start..self.pos].iter().collect()))
    }

    /// Preserve an unsupported environment using the probe-backed fallback shape.
    pub(super) fn parse_unsupported_environment(&mut self, name: &str) -> Result<Expr, String> {
        self.parse_unsupported_environment_with_shell(name, unsupported_environment_shell_style(name))
    }

    /// Preserve one unsupported environment using the requested begin/end shell style.
    fn parse_unsupported_environment_with_shell(
        &mut self,
        name: &str,
        shell_style: UnsupportedEnvironmentShellStyle,
    ) -> Result<Expr, String> {
        self.active_unsupported_envs.push(name.to_string());
        let parsed = self.parse_hybrid_unsupported_environment_body(name);
        self.active_unsupported_envs.pop();
        let parsed = parsed?;
        let mut items = vec![unsupported_environment_begin_expr(name, shell_style), parsed.body];
        if parsed.termination == UnsupportedEnvironmentTermination::MatchedOwnEnd {
            items.push(unsupported_environment_end_expr(name, shell_style));
        }
        Ok(Expr::Sequence(items))
    }

    /// Parse probe-known unsupported environments whose begin/end stay raw while
    /// MathType still tokenizes the interior as visible characters.
    fn parse_hybrid_unsupported_environment_body(
        &mut self,
        name: &str,
    ) -> Result<ParsedUnsupportedEnvironmentBody, String> {
        let mut items = Vec::new();
        loop {
            let mut leading_ws = self.consume_raw_whitespace();
            let pending_raw_ws = self.take_pending_raw_ws();
            if !pending_raw_ws.is_empty() {
                leading_ws.insert_str(0, &pending_raw_ws);
            }
            if self.consume_pending_unsupported_environment_close(name) {
                if !leading_ws.is_empty() {
                    items.push(Expr::RawTex(leading_ws));
                }
                return Ok(ParsedUnsupportedEnvironmentBody {
                    body: Expr::Sequence(items),
                    termination: UnsupportedEnvironmentTermination::PendingAncestorClose,
                });
            }
            if self.pos >= self.chars.len() {
                // Bug-fix: MathType still emits a partial fallback body for
                // unterminated/mismatched environments instead of failing hard.
                return Ok(ParsedUnsupportedEnvironmentBody {
                    body: Expr::Sequence(items),
                    termination: UnsupportedEnvironmentTermination::UnterminatedEof,
                });
            }
            if self.starts_end_environment(name) {
                if leading_ws.is_empty() {
                    // Bug-fix: unsupported wrappers such as `equation*` keep one
                    // same-line source space in the raw run immediately before `\end`,
                    // even when the previous visible atom already advanced past it.
                    leading_ws = self.recover_environment_separator_prefix();
                }
                if !leading_ws.is_empty() {
                    items.push(Expr::RawTex(leading_ws));
                }
                self.consume_end_environment(name)?;
                return Ok(ParsedUnsupportedEnvironmentBody {
                    body: Expr::Sequence(items),
                    termination: UnsupportedEnvironmentTermination::MatchedOwnEnd,
                });
            }
            if self.starts_command("end") {
                if leading_ws.is_empty() {
                    // Bug-fix: mismatched `\end{...}` inside unsupported fallback
                    // keeps the same recovered pre-end whitespace as a matched close.
                    leading_ws = self.recover_environment_separator_prefix();
                }
                if !leading_ws.is_empty() {
                    items.push(Expr::RawTex(leading_ws));
                }
                items.push(self.parse_mismatched_end_environment_fallback()?);
                continue;
            }
            if self.peek() == Some('&') {
                if leading_ws.is_empty() {
                    // Bug-fix: unsupported alignment-like environments preserve
                    // the same-line source space immediately before a raw `&`.
                    leading_ws = self.recover_environment_separator_prefix();
                }
                // Bug-fix: unsupported alignment-like environments keep top-level `&`
                // on the raw fallback path instead of rendering it as a visible glyph.
                items.push(Expr::RawTex(format!("{leading_ws}&")));
                self.pos += 1;
                continue;
            }
            if self.consume_row_separator() {
                if unsupported_environment_keeps_row_separator(name) {
                    if leading_ws.is_empty() {
                        // Bug-fix: fallback environments that keep a raw top-level
                        // row separator also keep one same-line source space that
                        // appears immediately before that `\\`.
                        leading_ws = self.recover_environment_separator_prefix();
                    }
                    items.push(Expr::RawTex(format!("{leading_ws}\\\\")));
                }
                continue;
            }
            let atom_start = self.pos;
            let atom = self.parse_complete_atom()?;
            let leading_ws = if leading_ws.is_empty() {
                self.recover_inline_whitespace_before(atom_start)
            } else {
                leading_ws
            };
            let atom = prefix_raw_leading_whitespace(&leading_ws, atom);
            if expr_is_empty_sequence(&atom) {
                continue;
            }
            if expr_is_mathtype_translation_failed(&atom) {
                return Ok(ParsedUnsupportedEnvironmentBody {
                    body: Expr::Text(MATHTYPE_TEXT_TRANSLATION_FAILED.to_string()),
                    termination: UnsupportedEnvironmentTermination::UnterminatedEof,
                });
            }
            items.push(atom);
            if unsupported_environment_keeps_row_separator(name) && self.starts_row_separator() {
                let separator_prefix = self.take_pending_raw_ws();
                // Bug-fix: a visible atom followed immediately by a raw-preserved
                // top-level row separator should donate its deferred source space
                // to that `\\` instead of dropping it.
                self.pos += 2;
                items.push(Expr::RawTex(format!("{separator_prefix}\\\\")));
            }
        }
    }

    /// Preserve one mismatched `\end{...}` inside unsupported-environment fallback.
    fn parse_mismatched_end_environment_fallback(&mut self) -> Result<Expr, String> {
        self.expect('\\')?;
        let start = self.pos;
        while self.peek().is_some_and(|ch| ch.is_ascii_alphabetic()) {
            self.pos += 1;
        }
        let command: String = self.chars[start..self.pos].iter().collect();
        if command != "end" {
            return Err(format!("expected \\end, found \\{command}"));
        }
        let name = self.parse_raw_group("mismatched environment end name")?;
        if self
            .active_unsupported_envs
            .iter()
            .rev()
            .skip(1)
            .any(|ancestor| ancestor == &name)
        {
            // Bug-fix: a crossed `\end{ancestor}` inside nested unsupported
            // environments should still close that ancestor later, so the outer
            // parser must not synthesize a second raw `\end{ancestor}` at EOF.
            self.pending_closed_unsupported_envs.push(name.clone());
        }
        Ok(unsupported_environment_end_expr(
            &name,
            unsupported_environment_shell_style(&name),
        ))
    }

    /// Preserve one mismatched `\end{...}` inside transparent wrappers as one raw token.
    fn parse_transparent_wrapper_mismatched_end_environment_fallback(&mut self) -> Result<Expr, String> {
        self.expect('\\')?;
        let start = self.pos;
        while self.peek().is_some_and(|ch| ch.is_ascii_alphabetic()) {
            self.pos += 1;
        }
        let command: String = self.chars[start..self.pos].iter().collect();
        if command != "end" {
            return Err(format!("expected \\end, found \\{command}"));
        }
        let name = self.parse_raw_group("mismatched environment end name")?;
        Ok(Expr::RawTex(format!("\\end{{{name}}}")))
    }

    /// Consume one pending unsupported-environment close recorded by a nested mismatch.
    fn consume_pending_unsupported_environment_close(&mut self, name: &str) -> bool {
        let Some(index) = self
            .pending_closed_unsupported_envs
            .iter()
            .position(|pending| pending == name)
        else {
            return false;
        };
        self.pending_closed_unsupported_envs.remove(index);
        true
    }

    /// Parse an environment into rows split by & and \\ separators.
    pub(super) fn parse_environment_rows(
        &mut self,
        name: &str,
    ) -> Result<ParsedEnvironmentRows, String> {
        let mut rows = Vec::new();
        let mut row_leading = Vec::new();
        let mut separator_leading = Vec::new();
        let mut row_annotations = Vec::new();
        loop {
            let leading_ws =
                normalize_environment_fallback_whitespace(&self.consume_raw_whitespace());
            if self.starts_command("end") {
                self.consume_end_environment(name)?;
                finalize_multirow_environment_annotations(name, &mut rows, &mut row_annotations);
                return Ok(ParsedEnvironmentRows {
                    rows,
                    row_leading,
                    separator_leading,
                    end_leading: leading_ws,
                    row_annotations,
                });
            }
            row_leading.push(leading_ws);
            let mut cells = Vec::new();
            let mut row_separator_prefixes = Vec::new();
            let mut row_annotation = EnvironmentRowAnnotation::default();
            loop {
                let cell = self.parse_sequence_until_environment_stop(name)?;
                cells.push(cell);
                while self.consume_environment_row_annotation(name, &mut row_annotation)? {}
                let trailing_ws =
                    normalize_environment_fallback_whitespace(&self.consume_raw_whitespace());
                if self.peek() == Some('&') {
                    // When MathType fallback preserves a literal space before `&`,
                    // recover it from the source if the parser's stop logic left
                    // the separator-adjacent trivia behind the current cursor.
                    let separator_prefix = if trailing_ws.is_empty() {
                        self.recover_environment_separator_prefix()
                    } else {
                        trailing_ws
                    };
                    row_separator_prefixes.push(separator_prefix);
                    self.pos += 1;
                    continue;
                }
                if self.consume_row_separator() {
                    break;
                }
                if self.starts_command("end") {
                    let end_leading = if trailing_ws.is_empty() {
                        // Bug-fix: supported environments can also lose one same-line
                        // source space before `\end{...}` when the last visible cell
                        // already advanced the parser cursor past it.
                        self.recover_environment_separator_prefix()
                    } else {
                        trailing_ws
                    };
                    self.consume_end_environment(name)?;
                    rows.push(cells);
                    separator_leading.push(row_separator_prefixes);
                    row_annotations.push(row_annotation);
                    finalize_single_row_environment_annotations(name, &mut rows, &mut row_annotations);
                    finalize_multirow_environment_annotations(name, &mut rows, &mut row_annotations);
                    return Ok(ParsedEnvironmentRows {
                        rows,
                        row_leading,
                        separator_leading,
                        end_leading,
                        row_annotations,
                    });
                }
                return Err(format!("expected &, \\\\, or \\end{{{name}}}"));
            }
            rows.push(cells);
            separator_leading.push(row_separator_prefixes);
            row_annotations.push(row_annotation);
        }
    }

    /// Parse array rows while preserving MathType's probe-backed inter-row raw controls.
    pub(super) fn parse_array_rows(&mut self, name: &str) -> Result<ParsedEnvironmentRows, String> {
        let mut rows = Vec::new();
        let mut row_leading = Vec::new();
        let mut separator_leading = Vec::new();
        loop {
            let leading_ws =
                normalize_environment_fallback_whitespace(&self.consume_raw_whitespace());
            if self.starts_command("end") {
                self.consume_end_environment(name)?;
                return Ok(ParsedEnvironmentRows {
                    rows,
                    row_leading,
                    separator_leading,
                    end_leading: String::new(),
                    row_annotations: Vec::new(),
                });
            }
            row_leading.push(self.consume_array_row_prefix(leading_ws));
            let mut cells = Vec::new();
            let mut row_separator_prefixes = Vec::new();
            loop {
                let cell = self.parse_sequence_until_environment_stop(name)?;
                cells.push(cell);
                let trailing_ws =
                    normalize_environment_fallback_whitespace(&self.consume_raw_whitespace());
                if self.peek() == Some('&') {
                    row_separator_prefixes.push(trailing_ws);
                    self.pos += 1;
                    continue;
                }
                if self.consume_row_separator() {
                    break;
                }
                if self.starts_command("end") {
                    let end_leading = if trailing_ws.is_empty() {
                        // Bug-fix: array-like environments keep one same-line source
                        // space before `\end{...}` on MathType's fallback path.
                        self.recover_environment_separator_prefix()
                    } else {
                        trailing_ws
                    };
                    self.consume_end_environment(name)?;
                    rows.push(cells);
                    separator_leading.push(row_separator_prefixes);
                    return Ok(ParsedEnvironmentRows {
                        rows,
                        row_leading,
                        separator_leading,
                        end_leading,
                        row_annotations: Vec::new(),
                    });
                }
                return Err(format!("expected &, \\\\, or \\end{{{name}}}"));
            }
            rows.push(cells);
            separator_leading.push(row_separator_prefixes);
        }
    }

    /// Keep array row-prefix raw trivia so the writer can inject it before the first cell.
    ///
    /// Bug-fix: MathType folds a same-line row-leading space into the raw prefix
    /// when the next cell starts with a control word such as `\hfill`.
    pub(super) fn consume_array_row_prefix(&mut self, mut leading_ws: String) -> String {
        let mut raw = String::new();
        loop {
            if self.starts_command("hline") {
                self.pos += 1 + "hline".len();
                leading_ws =
                    normalize_environment_fallback_whitespace(&self.consume_raw_whitespace());
                continue;
            }
            if self.starts_command("hdashline") {
                self.pos += 1 + "hdashline".len();
                raw.push_str(&leading_ws);
                raw.push_str("\\hdashline");
                let _ = self.consume_raw_whitespace();
                continue;
            }
            if raw.is_empty()
                && !leading_ws.is_empty()
                && starts_array_raw_row_prefix_command(self)
            {
                raw.push_str(&leading_ws);
            }
            return raw;
        }
    }

    /// Parse a cell until an environment separator appears at the current nesting level.
    pub(super) fn parse_sequence_until_environment_stop(
        &mut self,
        name: &str,
    ) -> Result<Expr, String> {
        let mut items = Vec::new();
        loop {
            let ws_start = self.pos;
            let leading_ws = self.consume_raw_whitespace();
            if self.pos >= self.chars.len()
                || self.peek() == Some('&')
                || self.starts_command("end")
                || self.starts_row_separator()
                || self.starts_environment_row_annotation(name)
            {
                self.pos = ws_start;
                break;
            }
            if let Some(infix) = self.consume_infix_command() {
                let left = Expr::Sequence(items);
                let right = self.parse_sequence_until_environment_stop(name)?;
                return Ok(infix_expr(infix, left, right));
            }
            let _ = name;
            let atom = with_leading_raw_space(self.parse_complete_atom()?, &leading_ws);
            if expr_is_empty_sequence(&atom) {
                continue;
            }
            if expr_is_mathtype_translation_failed(&atom) {
                return Ok(Expr::Text(MATHTYPE_TEXT_TRANSLATION_FAILED.to_string()));
            }
            items.push(atom);
        }
        Ok(Expr::Sequence(items))
    }

    /// Return true when the current environment row should stop before one row-level annotation command.
    fn starts_environment_row_annotation(&self, name: &str) -> bool {
        supports_environment_row_annotations(name)
            && (self.starts_command("label")
                || self.starts_command("notag")
                || self.starts_command("nonumber"))
    }

    /// Consume one row-level annotation command for numbering-aware environments.
    fn consume_environment_row_annotation(
        &mut self,
        name: &str,
        annotation: &mut EnvironmentRowAnnotation,
    ) -> Result<bool, String> {
        if !supports_environment_row_annotations(name) || self.peek() != Some('\\') {
            return Ok(false);
        }
        if self.starts_command("label") {
            self.pos += 1 + "label".len();
            let _ = self.parse_required_group("environment row label")?;
            annotation.has_label = true;
            return Ok(true);
        }
        if self.starts_command("notag") {
            self.pos += 1 + "notag".len();
            annotation.suppress_number = true;
            annotation.suppress_command = Some("\\notag".to_string());
            return Ok(true);
        }
        if self.starts_command("nonumber") {
            self.pos += 1 + "nonumber".len();
            annotation.suppress_number = true;
            annotation.suppress_command = Some("\\nonumber".to_string());
            return Ok(true);
        }
        Ok(false)
    }

    /// Parse a braced one-column row stack such as \substack{a\\b}.
    pub(super) fn parse_row_stack_group(&mut self, label: &str) -> Result<Vec<Vec<Expr>>, String> {
        self.expect('{')?;
        let mut rows = Vec::new();
        loop {
            self.skip_ws();
            if self.peek() == Some('}') {
                self.pos += 1;
                break;
            }
            let cell = self.parse_sequence_until_row_stack_stop()?;
            rows.push(vec![cell]);
            self.skip_ws();
            if self.consume_row_separator() {
                continue;
            }
            if self.peek() == Some('}') {
                self.pos += 1;
                break;
            }
            return Err(format!("expected \\\\ or closing brace for {label}"));
        }
        Ok(rows)
    }

    /// Parse one row-stack cell until a row separator or the closing brace.
    pub(super) fn parse_sequence_until_row_stack_stop(&mut self) -> Result<Expr, String> {
        let mut items = Vec::new();
        loop {
            self.skip_ws();
            if self.pos >= self.chars.len()
                || self.peek() == Some('}')
                || self.starts_row_separator()
            {
                break;
            }
            if let Some(infix) = self.consume_infix_command() {
                let left = Expr::Sequence(items);
                let right = self.parse_sequence_until_row_stack_stop()?;
                return Ok(infix_expr(infix, left, right));
            }
            let atom = self.parse_complete_atom()?;
            if expr_is_empty_sequence(&atom) {
                continue;
            }
            if expr_is_mathtype_translation_failed(&atom) {
                return Ok(Expr::Text(MATHTYPE_TEXT_TRANSLATION_FAILED.to_string()));
            }
            items.push(atom);
        }
        Ok(Expr::Sequence(items))
    }

    /// Return true when the input is at a row separator command.
    pub(super) fn starts_row_separator(&self) -> bool {
        self.peek() == Some('\\') && self.chars.get(self.pos + 1) == Some(&'\\')
    }

    /// Consume a row separator command if present.
    pub(super) fn consume_row_separator(&mut self) -> bool {
        if self.starts_row_separator() {
            self.pos += 2;
            true
        } else {
            false
        }
    }

    /// Consume \end{name} for the currently parsed environment.
    pub(super) fn consume_end_environment(&mut self, expected: &str) -> Result<(), String> {
        self.expect('\\')?;
        let start = self.pos;
        while self.peek().is_some_and(|ch| ch.is_ascii_alphabetic()) {
            self.pos += 1;
        }
        let command: String = self.chars[start..self.pos].iter().collect();
        if command != "end" {
            return Err(format!("expected \\end{{{expected}}}, found \\{command}"));
        }
        let actual = self.parse_raw_group("environment end name")?;
        if actual != expected {
            return Err(format!(
                "expected \\end{{{expected}}}, found \\end{{{actual}}}"
            ));
        }
        Ok(())
    }
}

/// Return true when an array row starts with a raw control word that keeps its leading space.
fn starts_array_raw_row_prefix_command(parser: &Parser) -> bool {
    parser.starts_command("hfil")
        || parser.starts_command("hfill")
        || parser.starts_command("hfilll")
}

/// Return the begin/end shell style MathType uses for one unsupported environment name.
fn unsupported_environment_shell_style(name: &str) -> UnsupportedEnvironmentShellStyle {
    if name.starts_with('\\') {
        UnsupportedEnvironmentShellStyle::RawInlineControlName
    } else {
        UnsupportedEnvironmentShellStyle::SplitNameVisible
    }
}

/// Return the base environment name without one optional star suffix.
fn base_name_for_environment_shell(name: &str) -> &str {
    name.strip_suffix('*').unwrap_or(name)
}

/// Build the fallback begin token for one unsupported environment.
fn unsupported_environment_begin_expr(name: &str, shell_style: UnsupportedEnvironmentShellStyle) -> Expr {
    match shell_style {
        UnsupportedEnvironmentShellStyle::SplitNameVisible => Expr::Sequence(vec![
            Expr::RawTex("\\begin".to_string()),
            visible_text_sequence(name),
        ]),
        UnsupportedEnvironmentShellStyle::RawDelimitedName => {
            Expr::RawTex(format!("\\begin{{{name}}}"))
        }
        UnsupportedEnvironmentShellStyle::RawInlineControlName => {
            Expr::RawTex(format!("\\begin{name}"))
        }
    }
}

/// Build the fallback end token for one unsupported environment.
fn unsupported_environment_end_expr(name: &str, shell_style: UnsupportedEnvironmentShellStyle) -> Expr {
    match shell_style {
        UnsupportedEnvironmentShellStyle::SplitNameVisible => Expr::Sequence(vec![
            Expr::RawTex("\\end".to_string()),
            visible_text_sequence(name),
        ]),
        UnsupportedEnvironmentShellStyle::RawDelimitedName => Expr::RawTex(format!("\\end{{{name}}}")),
        UnsupportedEnvironmentShellStyle::RawInlineControlName => Expr::RawTex(format!("\\end{name}")),
    }
}

/// Preserve raw-leading trivia in unsupported environments only when the next atom
/// itself stays on MathType's raw fallback path.
fn prefix_raw_leading_whitespace(prefix: &str, expr: Expr) -> Expr {
    if prefix.is_empty() {
        return expr;
    }
    match expr {
        Expr::RawTex(raw) => Expr::RawTex(format!("{prefix}{raw}")),
        Expr::Sequence(mut items) => {
            if let Some(first) = items.first_mut() {
                *first = prefix_raw_leading_whitespace(prefix, first.clone());
                Expr::Sequence(items)
            } else {
                Expr::Sequence(items)
            }
        }
        other => other,
    }
}

/// Return true when MathType keeps top-level row separators as raw fallback text.
pub(super) fn unsupported_environment_keeps_row_separator(name: &str) -> bool {
    matches!(name, "gather" | "gather*")
}

/// Return true for environments whose row endings carry MathType numbering metadata.
fn supports_environment_row_annotations(name: &str) -> bool {
    matches!(name, "align" | "alignat")
}

/// Extract row-level tags only after confirming the environment has multiple rows.
fn finalize_multirow_environment_annotations(
    name: &str,
    rows: &mut [Vec<Expr>],
    annotations: &mut [EnvironmentRowAnnotation],
) {
    if !supports_environment_row_annotations(name) || rows.len() <= 1 {
        return;
    }
    for (cells, annotation) in rows.iter_mut().zip(annotations.iter_mut()) {
        if annotation.tag.is_some() {
            continue;
        }
        let Some(last_cell) = cells.last_mut() else {
            continue;
        };
        annotation.tag = extract_trailing_environment_row_tag(last_cell);
    }
}

/// Keep single-row numbering suppressors visible because MathType emits them as raw suffix text.
fn finalize_single_row_environment_annotations(
    name: &str,
    rows: &mut [Vec<Expr>],
    annotations: &mut [EnvironmentRowAnnotation],
) {
    if !supports_environment_row_annotations(name) || rows.len() != 1 {
        return;
    }
    let Some(annotation) = annotations.first_mut() else {
        return;
    };
    let Some(raw) = annotation.suppress_command.take() else {
        return;
    };
    let Some(cells) = rows.first_mut() else {
        return;
    };
    let Some(cell) = cells.last_mut() else {
        return;
    };
    append_raw_suffix(cell, raw);
}

/// Extract the last `\tag{...}` payload from one environment cell when it stays at row scope.
fn extract_trailing_environment_row_tag(expr: &mut Expr) -> Option<Expr> {
    let Expr::Sequence(items) = expr else {
        return None;
    };
    let Some(Expr::Sequence(tag_items)) = items.last() else {
        return None;
    };
    let [Expr::RawTex(raw), rest @ ..] = tag_items.as_slice() else {
        return None;
    };
    if raw != "\\tag" || rest.is_empty() {
        return None;
    }
    let tag = collapse_single_sequence(Expr::Sequence(rest.to_vec()));
    items.pop();
    Some(tag)
}

/// Append one raw suffix to a cell while preserving any existing sequence structure.
fn append_raw_suffix(expr: &mut Expr, raw: String) {
    match expr {
        Expr::Sequence(items) => items.push(Expr::RawTex(raw)),
        other => {
            *other = Expr::Sequence(vec![other.clone(), Expr::RawTex(raw)]);
        }
    }
}

