use super::*;

impl Parser {
    /// Parse rows and columns for the supported \begin...\end environments.
    pub(super) fn parse_environment(&mut self, name: &str) -> Result<Expr, String> {
        let base_name = name.strip_suffix('*').unwrap_or(name);
        if name.ends_with('*') {
            // mathtools matrix* variants accept an optional alignment specifier.
            let _ = self.parse_optional_bracket_group()?;
        }
        let parsed_rows;
        let kind = match base_name {
            "equation" => return self.parse_wrapper_environment(name),
            "split" => EnvironmentKind::Split,
            "align" => EnvironmentKind::Align,
            "alignat" => {
                self.parse_raw_group("alignat column count")?;
                EnvironmentKind::AlignAt
            }
            "aligned" => EnvironmentKind::Aligned,
            "alignedat" => {
                self.parse_raw_group("alignedat column count")?;
                EnvironmentKind::AlignedAt
            }
            "gather" => EnvironmentKind::Gather,
            "gathered" => EnvironmentKind::Gathered,
            "cases" | "dcases" => EnvironmentKind::Cases,
            "rcases" | "drcases" => EnvironmentKind::RightCases,
            "array" => {
                // Preserve row-rule control words such as \hdashline so the writer can
                // reproduce MathType's mixed native/raw array layout instead of flattening
                // the whole environment into one raw fallback run.
                let column_spec = self.parse_raw_group("array column specifier")?;
                parsed_rows = self.parse_array_rows(name)?;
                return Ok(Expr::Environment {
                    kind: EnvironmentKind::Array,
                    rows: parsed_rows.rows,
                    trivia: EnvironmentTrivia {
                        column_spec: Some(column_spec),
                        row_leading: parsed_rows.row_leading,
                        separator_leading: parsed_rows.separator_leading,
                        end_leading: parsed_rows.end_leading,
                    },
                });
            }
            "subarray" => {
                // Preserve the column specifier so the writer can reproduce MathType's
                // mixed raw/native rendering for big-operator limits that use subarray.
                let column_spec = self.parse_raw_group("subarray column specifier")?;
                parsed_rows = self.parse_environment_rows(name)?;
                return Ok(Expr::Subarray {
                    column_spec,
                    rows: parsed_rows.rows,
                });
            }
            "matrix" => {
                parsed_rows = self.parse_environment_rows(name)?;
                return Ok(Expr::Matrix {
                    kind: MatrixKind::Plain,
                    rows: parsed_rows.rows,
                });
            }
            // MathType writes smallmatrix as a plain matrix entered under an explicit script-size record.
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
            },
        })
    }

    /// Parse transparent wrappers such as equation/equation* around real math content.
    pub(super) fn parse_wrapper_environment(&mut self, name: &str) -> Result<Expr, String> {
        let mut items = Vec::new();
        loop {
            self.skip_ws();
            if self.pos >= self.chars.len() {
                return Err(format!("unterminated \\begin{{{name}}} environment"));
            }
            if self.starts_end_environment(name) {
                self.consume_end_environment(name)?;
                break;
            }
            items.push(self.parse_complete_atom()?);
        }
        Ok(Expr::Sequence(items))
    }

    /// Preserve an unsupported environment using the probe-backed fallback shape.
    pub(super) fn parse_unsupported_environment(&mut self, name: &str) -> Result<Expr, String> {
        if unsupported_environment_uses_hybrid_begin_end(name) {
            let body = self.parse_hybrid_unsupported_environment_body(name)?;
            return Ok(Expr::Sequence(vec![
                Expr::RawTex("\\begin".to_string()),
                visible_text_sequence(name),
                body,
                Expr::RawTex("\\end".to_string()),
                visible_text_sequence(name),
            ]));
        }
        let mut raw = format!("\\begin{{{name}}}");
        loop {
            if self.pos >= self.chars.len() {
                return Err(format!("unterminated unsupported environment: {name}"));
            }
            if self.starts_end_environment(name) {
                raw.push_str(&format!("\\end{{{name}}}"));
                self.consume_end_environment(name)?;
                break;
            }
            let ch = self
                .next()
                .ok_or_else(|| format!("unterminated unsupported environment: {name}"))?;
            raw.push(ch);
        }
        Ok(Expr::RawTex(raw))
    }

    /// Parse probe-known unsupported environments whose begin/end stay raw while
    /// MathType still tokenizes the interior as visible characters.
    pub(super) fn parse_hybrid_unsupported_environment_body(
        &mut self,
        name: &str,
    ) -> Result<Expr, String> {
        let mut items = Vec::new();
        loop {
            let _ = self.consume_raw_whitespace();
            if self.pos >= self.chars.len() {
                return Err(format!("unterminated unsupported environment: {name}"));
            }
            if self.starts_end_environment(name) {
                self.consume_end_environment(name)?;
                break;
            }
            if self.consume_row_separator() {
                continue;
            }
            let atom = self.parse_complete_atom()?;
            if expr_is_empty_sequence(&atom) {
                continue;
            }
            items.push(atom);
        }
        Ok(Expr::Sequence(items))
    }

    /// Parse an environment into rows split by & and \\ separators.
    pub(super) fn parse_environment_rows(
        &mut self,
        name: &str,
    ) -> Result<ParsedEnvironmentRows, String> {
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
                    end_leading: leading_ws,
                });
            }
            row_leading.push(leading_ws);
            let mut cells = Vec::new();
            let mut row_separator_prefixes = Vec::new();
            loop {
                let cell = self.parse_sequence_until_environment_stop(name)?;
                cells.push(cell);
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
                    self.consume_end_environment(name)?;
                    rows.push(cells);
                    separator_leading.push(row_separator_prefixes);
                    return Ok(ParsedEnvironmentRows {
                        rows,
                        row_leading,
                        separator_leading,
                        end_leading: trailing_ws,
                    });
                }
                return Err(format!("expected &, \\\\, or \\end{{{name}}}"));
            }
            rows.push(cells);
            separator_leading.push(row_separator_prefixes);
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
                    self.consume_end_environment(name)?;
                    rows.push(cells);
                    separator_leading.push(row_separator_prefixes);
                    return Ok(ParsedEnvironmentRows {
                        rows,
                        row_leading,
                        separator_leading,
                        end_leading: String::new(),
                    });
                }
                return Err(format!("expected &, \\\\, or \\end{{{name}}}"));
            }
            rows.push(cells);
            separator_leading.push(row_separator_prefixes);
        }
    }

    /// Keep probe-known array rule commands so the writer can inject them between rows.
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
            let _ = self.consume_raw_whitespace();
            if self.pos >= self.chars.len()
                || self.peek() == Some('&')
                || self.starts_command("end")
                || self.starts_row_separator()
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
            let atom = self.parse_complete_atom()?;
            if expr_is_empty_sequence(&atom) {
                continue;
            }
            items.push(atom);
        }
        Ok(Expr::Sequence(items))
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
