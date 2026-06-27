use super::*;

impl Parser {
    /// Parse one braced command whose MathType fallback keeps the command name raw.
    pub(super) fn parse_raw_prefix_group_command(
        &mut self,
        label: &str,
        command: &str,
    ) -> Result<Expr, String> {
        Ok(raw_prefix_expr(command, self.parse_required_group(label)?))
    }

    /// Preserve MathType's overlap wrappers as one raw prefix followed by visible groups.
    pub(super) fn parse_overlap_wrapper(&mut self, command: &str) -> Result<Expr, String> {
        let first = self.parse_required_group("overlap content")?;
        let mut args = Vec::new();
        // Flatten the visible group even when there is only one argument so the
        // writer emits the same single-line slot structure MathType uses for
        // mathclap/mathllap/mathrlap script operands.
        push_visible_items(&mut args, first);
        // MathType keeps a second adjacent braced group visible in the same raw-prefix run,
        // as in mathrlap{,/}{=}.
        self.skip_ws();
        if self.peek() == Some('{') {
            push_visible_items(
                &mut args,
                self.parse_required_group("overlap trailing content")?,
            );
        }
        Ok(raw_prefix_sequence(command, args))
    }

    /// Preserve unsupported TeX primitive assignments as one raw text run.
    pub(super) fn consume_raw_primitive(&mut self, command: String) -> String {
        let mut raw = format!("\\{command}");
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() {
                break;
            }
            raw.push(ch);
            self.pos += 1;
        }
        raw
    }

    /// Parse \middle followed by a delimiter inside \left...\right content.
    pub(super) fn parse_middle_delimiter(&mut self) -> Result<Expr, String> {
        self.skip_ws();
        if self.pos >= self.chars.len()
            || self.starts_command("right")
            || self.starts_command("end")
            || self.starts_row_separator()
            || matches!(self.peek(), Some('}' | '&'))
        {
            return self.parse_unsupported_command("middle".to_string());
        }
        let delimiter = self.parse_delimiter_char("middle delimiter")?;
        Ok(if delimiter == '.' {
            Expr::Sequence(Vec::new())
        } else {
            Expr::Char(delimiter)
        })
    }

    /// Preserve MathType's hybrid \char" form: raw command prefix plus visible hex digits.
    pub(super) fn parse_char_code(&mut self) -> Result<Expr, String> {
        self.skip_ws();
        let start = self.pos;
        if self.peek() != Some('"') {
            return self.parse_unsupported_command("char".to_string());
        }
        self.pos += 1;
        let digits_start = self.pos;
        while self.peek().is_some_and(|ch| ch.is_ascii_hexdigit()) {
            self.pos += 1;
        }
        if self.pos == digits_start {
            self.pos = start;
            return self.parse_unsupported_command("char".to_string());
        }
        let digits = self.chars[digits_start..self.pos]
            .iter()
            .collect::<String>();
        u32::from_str_radix(&digits, 16)
            .map_err(|err| format!("invalid \\char hex code {digits}: {err}"))?;
        Ok(raw_prefix_sequence_with_raw(
            "\\char\"".to_string(),
            vec![self.parse_visible_wrapper_text(&digits)?],
        ))
    }

    /// Preserve MathType's explicit display/text binomial wrappers instead of collapsing them.
    pub(super) fn parse_styled_binom(&mut self, kind: StyleKind) -> Result<Expr, String> {
        let upper = self.parse_required_group("binomial upper")?;
        let lower = self.parse_required_group("binomial lower")?;
        Ok(Expr::Style {
            kind,
            content: Box::new(Expr::Pile {
                kind: PileKind::Binom,
                upper: Box::new(upper),
                lower: Box::new(lower),
            }),
        })
    }

    /// Parse TeX's delimiter-based \verb literal as visible typewriter text.
    pub(super) fn parse_verb_literal(&mut self) -> Result<Expr, String> {
        let Some(delimiter) = self.next() else {
            return Ok(Expr::RawTex("\\verb".to_string()));
        };
        if delimiter.is_whitespace() {
            return Ok(Expr::RawTex("\\verb".to_string()));
        }
        let start = self.pos;
        while self.peek().is_some_and(|ch| ch != delimiter) {
            self.pos += 1;
        }
        if self.peek() != Some(delimiter) {
            self.pos = start;
            return Ok(Expr::RawTex("\\verb".to_string()));
        }
        let content = self.chars[start..self.pos].iter().collect::<String>();
        self.pos += 1;
        let visible =
            self.parse_visible_wrapper_text(&format!("{delimiter}{content}{delimiter}"))?;
        Ok(raw_prefix_expr("verb", visible))
    }

    /// Parse `\not` as a relation overlay instead of collapsing it to Unicode.
    ///
    /// MathType TeX Input keeps the base relation glyph and adds the same `embNOT`
    /// decoration in MTEF, so we preserve the inner relation expression here.
    pub(super) fn parse_not_relation(&mut self) -> Result<Expr, String> {
        let relation = self.parse_required_group_or_atom("not relation")?;
        if relation_char(&relation).is_none() {
            return Ok(raw_prefix_expr("not", relation));
        }
        Ok(Expr::NotRelation(Box::new(relation)))
    }

    /// Preserve MathType TeX Input fallback for unsupported control words.
    pub(super) fn parse_unsupported_command(&mut self, command: String) -> Result<Expr, String> {
        let mut raw = format!("\\{command}");
        let consumed_ws = self.consume_ws();
        if self.peek() == Some('{') {
            self.pos += 1;
            let argument = self.parse_sequence(Some('}'))?;
            Ok(Expr::Sequence(vec![Expr::RawTex(raw), argument]))
        } else {
            if consumed_ws && raw_command_preserves_trailing_space(&command) {
                raw.push(' ');
            }
            Ok(Expr::RawTex(raw))
        }
    }

    /// Parse one atom, its scripts, and any operand required by big operators.
    pub(super) fn parse_complete_atom(&mut self) -> Result<Expr, String> {
        let mut atom = self.parse_atom_with_scripts()?;
        if matches!(atom, Expr::BigOp { body: None, .. }) && !self.big_op_operand_stops() {
            let body = self.parse_big_op_operand()?;
            if let Expr::BigOp {
                body: body_slot, ..
            } = &mut atom
            {
                *body_slot = Some(Box::new(body));
            }
        } else if matches!(
            atom,
            Expr::Integral { .. } | Expr::IntegralOp { body: None, .. }
        ) && !self.big_op_operand_stops()
        {
            let body = self.parse_big_op_operand()?;
            match &mut atom {
                Expr::Integral { kind } => {
                    atom = Expr::IntegralOp {
                        kind: *kind,
                        lower: None,
                        upper: None,
                        body: Some(Box::new(body)),
                    };
                }
                Expr::IntegralOp {
                    body: body_slot, ..
                } => {
                    *body_slot = Some(Box::new(body));
                }
                _ => {}
            }
        }
        Ok(atom)
    }

    /// Parse an atom followed by optional subscript/superscript records.
    pub(super) fn parse_atom_with_scripts(&mut self) -> Result<Expr, String> {
        // MathType accepts leading `_` / `^` as scripts on an empty base instead
        // of treating the underscore/caret as a visible atom.
        let mut atom = if matches!(self.peek(), Some('_' | '^')) {
            Expr::Sequence(Vec::new())
        } else {
            self.parse_atom()?
        };
        let mut limit_modifier = None;
        loop {
            let consumed_ws = self.consume_ws();
            if let Some(modifier) = self.consume_limits_modifier() {
                limit_modifier = Some(modifier);
                continue;
            }
            match self.peek() {
                Some('_') => {
                    self.pos += 1;
                    let sub = self.parse_script_arg()?;
                    atom = merge_script(atom, Some(sub), None, limit_modifier.take());
                }
                Some('^') => {
                    self.pos += 1;
                    if let Some(raw) = self.parse_raw_prime_script_group()? {
                        atom = raw_superscript_suffix_expr(atom, &raw);
                        continue;
                    }
                    let sup = self.parse_script_arg()?;
                    atom = merge_script(atom, None, Some(sup), limit_modifier.take());
                }
                _ => {
                    if consumed_ws {
                        // MathType keeps this source-space inside the next raw-text fallback run.
                        self.pending_raw_ws = true;
                    }
                    break;
                }
            }
        }
        Ok(atom)
    }

    /// Parse the term MathType places in the first slot of a big-op template.
    pub(super) fn parse_big_op_operand(&mut self) -> Result<Expr, String> {
        self.skip_ws();
        if self.big_op_operand_stops() {
            return Err("expected big-operator operand".to_string());
        }
        self.parse_complete_atom()
    }

    /// Stop a big-op operand at top-level separators; nested groups parse themselves.
    pub(super) fn big_op_operand_stops(&self) -> bool {
        self.pos >= self.chars.len()
            || self.starts_command("right")
            || self.starts_command("end")
            || self.starts_row_separator()
            || matches!(self.peek(), Some('}' | ',' | '+' | '-' | '=' | '&'))
    }

    /// Parse a dynamic delimiter body until the paired \right command.
    pub(super) fn parse_sequence_until_right(&mut self) -> Result<Expr, String> {
        let mut items = Vec::new();
        loop {
            self.skip_ws();
            if self.pos >= self.chars.len() {
                return Err("unterminated \\left...\\right group".to_string());
            }
            if self.starts_command("right") {
                break;
            }
            if let Some(infix) = self.consume_infix_command() {
                let left = Expr::Sequence(items);
                let right = self.parse_sequence_until_right()?;
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

    /// Consume \right and return either a native delimiter char or one raw fallback token.
    pub(super) fn parse_right_delimiter_spec(&mut self) -> Result<LeftRightDelimiter, String> {
        self.expect('\\')?;
        let start = self.pos;
        while self.peek().is_some_and(|ch| ch.is_ascii_alphabetic()) {
            self.pos += 1;
        }
        let command: String = self.chars[start..self.pos].iter().collect();
        if command != "right" {
            return Err(format!("expected \\right, found \\{command}"));
        }
        self.parse_left_right_delimiter("right delimiter")
    }

    /// Parse one delimiter after \left or \right, preserving probe-known raw control words.
    pub(super) fn parse_left_right_delimiter(
        &mut self,
        label: &str,
    ) -> Result<LeftRightDelimiter, String> {
        self.skip_ws();
        if self.peek() == Some('\\') {
            self.pos += 1;
            let start = self.pos;
            while self.peek().is_some_and(|ch| ch.is_ascii_alphabetic()) {
                self.pos += 1;
            }
            if start != self.pos {
                let command: String = self.chars[start..self.pos].iter().collect();
                if raw_left_right_delimiter_command(&command) {
                    return Ok(LeftRightDelimiter::RawCommand(command));
                }
                return delimiter_command_char(&command)
                    .map(LeftRightDelimiter::Char)
                    .ok_or_else(|| format!("unsupported {label}: \\{command}"));
            }
            let ch = self.next().ok_or_else(|| format!("expected {label}"))?;
            return Ok(LeftRightDelimiter::Char(if ch == '|' {
                '\u{2016}'
            } else {
                ch
            }));
        }
        self.next()
            .map(LeftRightDelimiter::Char)
            .ok_or_else(|| format!("expected {label}"))
    }

    /// Parse one delimiter after \left or \right, including escaped braces.
    pub(super) fn parse_delimiter_char(&mut self, label: &str) -> Result<char, String> {
        self.skip_ws();
        if self.peek() == Some('\\') {
            self.pos += 1;
            let start = self.pos;
            while self.peek().is_some_and(|ch| ch.is_ascii_alphabetic()) {
                self.pos += 1;
            }
            if start != self.pos {
                let command: String = self.chars[start..self.pos].iter().collect();
                return delimiter_command_char(&command)
                    .ok_or_else(|| format!("unsupported {label}: \\{command}"));
            }
            let ch = self.next().ok_or_else(|| format!("expected {label}"))?;
            return Ok(if ch == '|' { '\u{2016}' } else { ch });
        }
        self.next().ok_or_else(|| format!("expected {label}"))
    }

    /// Consume \limits or \nolimits before scripts so limit-style functions keep the hint.
    pub(super) fn consume_limits_modifier(&mut self) -> Option<LimitModifier> {
        for (command, modifier) in [
            ("limits", LimitModifier::Limits),
            ("nolimits", LimitModifier::NoLimits),
        ] {
            if self.starts_command(command) {
                self.pos += 1 + command.len();
                return Some(modifier);
            }
        }
        None
    }

    /// Consume an optional star used by commands such as \operatorname*.
    pub(super) fn consume_optional_star(&mut self) -> bool {
        self.skip_ws();
        if self.peek() == Some('*') {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    /// Parse either a braced switch argument or the remaining local scope.
    pub(super) fn parse_switch_content(&mut self, label: &str) -> Result<Expr, String> {
        self.skip_ws();
        if self.peek() == Some('{') {
            self.pos += 1;
            return self.parse_sequence(Some('}'));
        }
        let mut items = Vec::new();
        while !self.switch_content_stops() {
            items.push(self.parse_complete_atom()?);
            self.skip_ws();
        }
        if items.is_empty() {
            Err(format!("expected {label}"))
        } else {
            Ok(Expr::Sequence(items))
        }
    }

    /// Stop a switch command at the same local separators as ordinary sequences.
    pub(super) fn switch_content_stops(&self) -> bool {
        self.pos >= self.chars.len()
            || self.starts_command("right")
            || matches!(self.peek(), Some('}' | '&'))
            || self.starts_row_separator()
    }

    /// Parse an optional bracketed group such as the index in \sqrt[n]{...}.
    pub(super) fn parse_optional_bracket_group(&mut self) -> Result<Option<Expr>, String> {
        self.skip_ws();
        if self.peek() != Some('[') {
            return Ok(None);
        }
        self.pos += 1;
        Ok(Some(self.parse_sequence(Some(']'))?))
    }

    /// Parse \hspace as a MathType-ignored layout hint when a length is present.
    pub(super) fn parse_hspace_content(&mut self) -> Result<Expr, String> {
        self.consume_optional_star();
        let _ignored_width = self.parse_raw_group("hspace width")?;
        Ok(Expr::Sequence(Vec::new()))
    }

    /// Parse \cline as a MathType-ignored table-rule hint when a span is present.
    pub(super) fn parse_cline_content(&mut self) -> Result<Expr, String> {
        let _ignored_span = self.parse_raw_group("cline span")?;
        Ok(Expr::Sequence(Vec::new()))
    }

    /// Parse a required fraction argument, accepting TeX's single-atom shorthand.
    pub(super) fn parse_required_group_or_atom(&mut self, label: &str) -> Result<Expr, String> {
        self.skip_ws();
        if self.peek() == Some('{') {
            self.pos += 1;
            self.parse_sequence(Some('}'))
        } else if self.pos < self.chars.len() {
            self.parse_atom_with_scripts()
        } else {
            Err(format!("expected {label}"))
        }
    }

    /// Parse a required braced group and report which slot was missing.
    pub(super) fn parse_required_group(&mut self, label: &str) -> Result<Expr, String> {
        self.skip_ws();
        if self.peek() != Some('{') {
            return Err(format!("expected braced {label}"));
        }
        self.pos += 1;
        self.parse_sequence(Some('}'))
    }

    /// Preserve prime-only superscript groups as raw source because MathType's
    /// TeX Input keeps forms like `^{'}` and `^{' '}` as fallback text.
    pub(super) fn parse_raw_prime_script_group(&mut self) -> Result<Option<String>, String> {
        self.skip_ws();
        if self.peek() != Some('{') {
            return Ok(None);
        }
        let checkpoint = self.pos;
        let raw = self.parse_raw_group("prime superscript")?;
        if is_raw_prime_script_group(&raw) {
            Ok(Some(raw))
        } else {
            self.pos = checkpoint;
            Ok(None)
        }
    }

    /// Return raw text inside a simple braced group such as \operatorname{mean}.
    pub(super) fn parse_raw_group(&mut self, label: &str) -> Result<String, String> {
        self.skip_ws();
        if self.peek() != Some('{') {
            return Err(format!("expected braced {label}"));
        }
        self.pos += 1;
        let start = self.pos;
        let mut depth = 1usize;
        while let Some(ch) = self.next() {
            match ch {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        let end = self.pos - 1;
                        return Ok(self.chars[start..end].iter().collect());
                    }
                }
                _ => {}
            }
        }
        Err(format!("unterminated braced {label}"))
    }
}
