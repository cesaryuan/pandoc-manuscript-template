use super::*;
use super::environments::unsupported_environment_keeps_row_separator;

impl Parser {
    /// Return true when MathType uses a scalable fence template for this delimiter pair.
    pub(super) fn supports_dynamic_delimiter_pair(&self, left: char, right: char) -> bool {
        matches!(
            (left, right),
            ('(', ')')
                | ('[', ']')
                | ('{', '}')
                | ('|', '|')
                | ('\u{2016}', '\u{2016}')
                | ('\u{230a}', '\u{230b}')
                | ('\u{2308}', '\u{2309}')
                | ('\u{3008}', '\u{3009}')
                | ('<', '>')
                | ('|', '\u{3009}')
        )
    }

    /// Return true when MathType keeps a one-sided `\left.\right)` fence on the native template path.
    pub(super) fn supports_one_sided_delimiter(&self, delimiter: char) -> bool {
        matches!(
            delimiter,
            '('
                | ')'
                | '['
                | ']'
                | '{'
                | '}'
                | '|'
                | '\u{2016}'
                | '\u{230a}'
                | '\u{230b}'
                | '\u{2308}'
                | '\u{2309}'
                | '\u{3008}'
                | '\u{3009}'
                | '<'
                | '>'
        )
    }

    /// Parse one braced argument when present, otherwise keep the command token raw.
    ///
    /// This recovery matches MathType's bug-compatible fallback for wrappers such
    /// as `\bra A`, where the command name stays raw but the following visible
    /// tokens should continue parsing normally.
    pub(super) fn parse_grouped_command_or_raw<F>(
        &mut self,
        command: &str,
        label: &str,
        build: F,
    ) -> Result<Expr, String>
    where
        F: FnOnce(Expr) -> Expr,
    {
        let checkpoint = self.pos;
        match self.parse_required_group(label) {
            Ok(content) => Ok(build(content)),
            Err(_) => {
                self.pos = checkpoint;
                Ok(Expr::RawTex(format!("\\{command}")))
            }
        }
    }

    /// Parse one grouped-or-atom accent operand when present, otherwise keep the command raw.
    ///
    /// Bug-fix: MathType accepts single-atom shorthand for accent commands such
    /// as `\dddot A` and only falls back to raw command text when no operand is present.
    pub(super) fn parse_grouped_or_atom_command_or_raw<F>(
        &mut self,
        command: &str,
        label: &str,
        build: F,
    ) -> Result<Expr, String>
    where
        F: FnOnce(Expr) -> Expr,
    {
        let checkpoint = self.pos;
        match self.parse_required_group_or_atom(label) {
            Ok(content) => Ok(build(content)),
            Err(_) => {
                self.pos = checkpoint;
                Ok(Expr::RawTex(format!("\\{command}")))
            }
        }
    }

    /// Parse cancel-like commands while matching MathType's bare-atom shorthand
    /// and its translation-failed fallback for unsupported option lists.
    pub(super) fn parse_strike_command(
        &mut self,
        command: &str,
        label: &str,
        kind: StrikeKind,
    ) -> Result<Expr, String> {
        let checkpoint = self.pos;
        self.skip_ws();
        if self.peek() == Some('[') {
            // Bug-fix: MathType's TeX Input rejects optional-argument variants such as
            // `\cancel[color=red]{x}` and collapses the whole formula to the shared
            // translation-failed placeholder instead of partially preserving raw text.
            let _ = self.parse_optional_bracket_group();
            let _ = self.parse_required_group_or_atom(label);
            return Ok(Expr::Text(MATHTYPE_TEXT_TRANSLATION_FAILED.to_string()));
        }
        self.pos = checkpoint;
        match self.parse_required_group_or_atom(label) {
            Ok(content) => Ok(Expr::Strike {
                kind,
                content: Box::new(content),
            }),
            Err(_) => {
                self.pos = checkpoint;
                Ok(Expr::RawTex(format!("\\{command}")))
            }
        }
    }

    /// Parse a fraction-like command, preserving successful operands on partial fallback.
    ///
    /// MathType keeps the raw command text and any already-read visible operand
    /// when only the trailing operand is missing, as in `\frac{a}`.
    pub(super) fn parse_fraction_like_command<F>(
        &mut self,
        command: &str,
        numerator_label: &str,
        denominator_label: &str,
        build: F,
    ) -> Result<Expr, String>
    where
        F: FnOnce(Expr, Expr) -> Expr,
    {
        let checkpoint = self.pos;
        let numerator = match self.parse_required_group_or_atom(numerator_label) {
            Ok(numerator) => numerator,
            Err(_) => {
                self.pos = checkpoint;
                return Ok(Expr::RawTex(format!("\\{command}")));
            }
        };
        match self.parse_required_group_or_atom(denominator_label) {
            Ok(denominator) => Ok(build(numerator, denominator)),
            Err(_) => {
                // Bug-fix: when only the trailing operand is missing, MathType keeps
                // the raw command plus the already-consumed visible leading operand
                // instead of rewinding and letting the outer parser consume it again.
                Ok(raw_prefix_expr(command, numerator))
            }
        }
    }

    /// Parse `\left...\right` while preserving MathType's partial raw fallback.
    ///
    /// This bug-fix keeps `\left` and any consumed delimiter token raw when the
    /// group is malformed or incomplete instead of aborting the whole formula.
    pub(super) fn parse_left_group(&mut self) -> Result<Expr, String> {
        let left_checkpoint = self.pos;
        let (left, left_raw) = match self.parse_left_right_delimiter_with_raw("left delimiter") {
            Ok(parsed) => parsed,
            Err(_) => {
                self.pos = left_checkpoint;
                return Ok(Expr::RawTex("\\left".to_string()));
            }
        };
        let (content, terminated) = self.parse_sequence_until_right_or_eof()?;
        if !terminated {
            return Ok(self.partial_left_right_expr(left_raw, content, None));
        }
        let right_checkpoint = self.pos;
        match self.parse_right_delimiter_spec_with_raw() {
            Ok((right, _right_raw)) => Ok(match (left, right) {
                (left, right)
                    if left_right_visible_char(&left)
                        .zip(left_right_visible_char(&right))
                        .is_some_and(|(left, right)| {
                            self.supports_dynamic_delimiter_pair(left, right)
                        }) =>
                {
                    Expr::Delimited {
                        left: left_right_visible_char(&left).expect("visible left delimiter"),
                        right: left_right_visible_char(&right).expect("visible right delimiter"),
                        content: Box::new(content),
                    }
                }
                (LeftRightDelimiter::Char('.'), right)
                    if left_right_visible_char(&right)
                        .is_some_and(|right| self.supports_one_sided_delimiter(right)) =>
                {
                    Expr::OneSidedDelimited {
                        side: OneSidedDelimiterSide::RightVisible,
                        delimiter: left_right_visible_char(&right)
                            .expect("visible one-sided right delimiter"),
                        content: Box::new(content),
                    }
                }
                (left, LeftRightDelimiter::Char('.'))
                    if left_right_visible_char(&left)
                        .is_some_and(|left| self.supports_one_sided_delimiter(left)) =>
                {
                    Expr::OneSidedDelimited {
                        side: OneSidedDelimiterSide::LeftVisible,
                        delimiter: left_right_visible_char(&left)
                            .expect("visible one-sided left delimiter"),
                        content: Box::new(content),
                    }
                }
                (LeftRightDelimiter::Char(left), LeftRightDelimiter::Char(right))
                    if self.supports_dynamic_delimiter_pair(left, right) =>
                {
                    Expr::Delimited {
                        left,
                        right,
                        content: Box::new(content),
                    }
                }
                (LeftRightDelimiter::Char('.'), LeftRightDelimiter::Char(right))
                    if self.supports_one_sided_delimiter(right) =>
                {
                    Expr::OneSidedDelimited {
                        side: OneSidedDelimiterSide::RightVisible,
                        delimiter: right,
                        content: Box::new(content),
                    }
                }
                (LeftRightDelimiter::Char(left), LeftRightDelimiter::Char('.'))
                    if self.supports_one_sided_delimiter(left) =>
                {
                    Expr::OneSidedDelimited {
                        side: OneSidedDelimiterSide::LeftVisible,
                        delimiter: left,
                        content: Box::new(content),
                    }
                }
                (LeftRightDelimiter::Char('.'), LeftRightDelimiter::Char('.')) => self
                    .rewrite_missing_outer_fence_with_middle(content)
                    .unwrap_or_else(|| Expr::Sequence(Vec::new())),
                (LeftRightDelimiter::Char(left), LeftRightDelimiter::Char(right)) => {
                    let mut items = Vec::new();
                    items.push(Expr::Char(left));
                    push_visible_items(&mut items, content);
                    items.push(Expr::Char(right));
                    Expr::Sequence(items)
                }
                (
                    LeftRightDelimiter::Command {
                        command: _,
                        ch: left,
                    },
                    LeftRightDelimiter::Command {
                        command: _,
                        ch: right,
                    },
                ) if self.supports_dynamic_delimiter_pair(left, right) => Expr::Delimited {
                    left,
                    right,
                    content: Box::new(content),
                },
                (
                    LeftRightDelimiter::Command {
                        command: left_command,
                        ch: left,
                    },
                    LeftRightDelimiter::Command {
                        command: right_command,
                        ch: right,
                    },
                ) => Expr::Sequence(vec![
                    Expr::CommandSymbol {
                        command: left_command,
                        ch: left,
                    },
                    content,
                    Expr::CommandSymbol {
                        command: right_command,
                        ch: right,
                    },
                ]),
                (left, right) => {
                    let mut items = Vec::new();
                    push_left_right_delimiter(&mut items, "left", left);
                    push_visible_items(&mut items, content);
                    push_left_right_delimiter(&mut items, "right", right);
                    Expr::Sequence(items)
                }
            }),
            Err(_) => {
                self.pos = right_checkpoint;
                if self.starts_command("right") {
                    // Bug-fix: when `\right` is present but its delimiter is malformed or
                    // missing, MathType preserves exactly one raw `\right` token in the
                    // partial fallback instead of re-parsing the same control word again.
                    self.pos += 1 + "right".len();
                }
                Ok(self.partial_left_right_expr(left_raw, content, Some("\\right")))
            }
        }
    }

    /// Preserve one partial `\left...\right` run as raw delimiter text plus visible content.
    pub(super) fn partial_left_right_expr(
        &self,
        left_raw: String,
        content: Expr,
        right_raw: Option<&str>,
    ) -> Expr {
        let mut items = Vec::new();
        // Bug-fix: partial `\left...\right` fallback keeps the full `\left<delim>`
        // token raw, not only the delimiter byte, when MathType cannot build a
        // complete native fence pair.
        items.push(Expr::RawTex(format!("\\left{left_raw}")));
        push_visible_items(&mut items, content);
        if let Some(right_raw) = right_raw {
            items.push(Expr::RawTex(right_raw.to_string()));
        }
        Expr::Sequence(items)
    }

    /// Collapse `\left.\middle(...\right.` into raw `\middle` plus visible delimiter content.
    ///
    /// Bug-fix: when both outer fences are missing, MathType does not keep a
    /// native one-sided fence template. It falls back to raw `\middle` and then
    /// renders the following delimiter as an ordinary visible glyph.
    fn rewrite_missing_outer_fence_with_middle(&self, content: Expr) -> Option<Expr> {
        let Expr::Sequence(items) = content else {
            return None;
        };
        let (delimiter_expr, rest) = match items.as_slice() {
            [Expr::Sequence(middle_items), rest @ ..] => {
                let [Expr::RawTex(raw_middle), delimiter_expr] = middle_items.as_slice() else {
                    return None;
                };
                if raw_middle != "\\middle" || rest.is_empty() {
                    return None;
                }
                (delimiter_expr.clone(), rest)
            }
            [Expr::RawTex(raw_middle), delimiter_expr, rest @ ..] => {
                if raw_middle != "\\middle" || rest.is_empty() {
                    return None;
                }
                (delimiter_expr.clone(), rest)
            }
            _ => return None,
        };
        let visible_delimiter = match delimiter_expr {
            Expr::Char(ch) => {
                if !self.supports_one_sided_delimiter(ch) {
                    return None;
                }
                Expr::Char(ch)
            }
            Expr::CommandSymbol { command, ch } => {
                if !self.supports_one_sided_delimiter(ch) {
                    return None;
                }
                Expr::CommandSymbol { command, ch }
            }
            _ => return None,
        };
        let mut rewritten = vec![Expr::RawTex("\\middle".to_string()), visible_delimiter];
        rewritten.extend_from_slice(rest);
        Some(collapse_single_sequence(Expr::Sequence(rewritten)))
    }

    /// Parse one braced command whose MathType fallback keeps the command name raw.
    pub(super) fn parse_raw_prefix_group_command(
        &mut self,
        label: &str,
        command: &str,
    ) -> Result<Expr, String> {
        self.parse_grouped_command_or_raw(command, label, |content| {
            raw_prefix_expr(command, content)
        })
    }

    /// Consume one metadata group that MathType ignores completely when parsing succeeds.
    pub(super) fn parse_ignored_group_command(
        &mut self,
        label: &str,
        command: &str,
    ) -> Result<Expr, String> {
        let checkpoint = self.pos;
        match self.parse_required_group(label) {
            Ok(_) => Ok(Expr::Sequence(Vec::new())),
            Err(_) => {
                self.pos = checkpoint;
                // Bug-fix: metadata commands such as `\label` are dropped even when the
                // required group is missing; MathType also consumes one bare atom like
                // `\label A` as the ignored metadata payload instead of leaving it visible.
                let _ = self.parse_required_group_or_atom(label);
                let _ = command;
                Ok(Expr::Sequence(Vec::new()))
            }
        }
    }

    /// Consume one reference-like group and collapse the whole formula to MathType's failure text.
    pub(super) fn parse_translation_failed_group_command(
        &mut self,
        label: &str,
        command: &str,
    ) -> Result<Expr, String> {
        let checkpoint = self.pos;
        match self.parse_required_group(label) {
            Ok(_) => {
                // Bug-fix: once MathType hits a reference-like command such as
                // `\ref` or `\eqref`, it collapses the whole formula to the
                // failure text instead of leaving following tokens to trigger a
                // secondary trailing-character parse error.
                self.pos = self.chars.len();
                Ok(Expr::Text(MATHTYPE_TEXT_TRANSLATION_FAILED.to_string()))
            }
            Err(_) => {
                self.pos = checkpoint;
                Ok(Expr::RawTex(format!("\\{command}")))
            }
        }
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

    /// Parse `\let` the way MathType exposes it: raw primitive plus two ordinary operands.
    ///
    /// Bug-fix: MathType does not execute TeX aliasing here. It keeps `\let` raw,
    /// renders the alias-name token with its current meaning, then renders the
    /// target token with its current meaning, and later uses of the alias still
    /// parse independently from the definition site.
    pub(super) fn parse_let_command(&mut self) -> Result<Expr, String> {
        let checkpoint = self.pos;
        let name_command = self.peek_next_control_word();
        let Some(name) = self.parse_let_name_operand()? else {
            self.pos = checkpoint;
            return Ok(Expr::RawTex("\\let".to_string()));
        };
        let name = normalize_let_target_expr(name);
        let target_command = self.peek_next_control_word();
        let Some(target) = self.parse_let_operand(true)? else {
            let mut items = vec![Expr::RawTex("\\let".to_string())];
            push_visible_items(&mut items, name);
            return Ok(collapse_single_sequence(Expr::Sequence(items)));
        };
        if let Some(name_command) = name_command {
            if let Some(target_command) = target_command
                .as_deref()
                .and_then(|command| self.replayable_let_target_command(command))
            {
                self.let_aliases.insert(name_command, target_command);
            } else {
                self.let_aliases.remove(&name_command);
            }
        }
        let mut items = vec![Expr::RawTex("\\let".to_string())];
        push_visible_items(&mut items, name);
        push_visible_items(&mut items, target);
        if self.next_starts_raw_fallback_command() {
            let trailing_ws = self.take_pending_raw_ws();
            if !trailing_ws.is_empty() {
                items.push(Expr::RawTex(trailing_ws));
            }
        }
        Ok(collapse_single_sequence(Expr::Sequence(items)))
    }

    /// Parse the alias-name token inside `\let` without letting following scripts attach to it.
    fn parse_let_name_operand(&mut self) -> Result<Option<Expr>, String> {
        self.skip_ws();
        if self.pos >= self.chars.len() {
            return Ok(None);
        }
        if self.peek() != Some('\\') {
            let ch = self.next().ok_or_else(|| "expected let name token".to_string())?;
            return parse_literal_char(ch, "").map(Some);
        }
        self.expect('\\')?;
        let start = self.pos;
        while self.peek().is_some_and(|ch| ch.is_ascii_alphabetic()) {
            self.pos += 1;
        }
        let command: String = self.chars[start..self.pos].iter().collect();
        if command.is_empty() {
            let ch = self
                .next()
                .ok_or_else(|| "dangling backslash in let name".to_string())?;
            if let Some(text_char) = escaped_single_char_math_char(ch) {
                return Ok(Some(Expr::Char(text_char)));
            }
            return Ok(Some(Expr::Char(ch)));
        }
        if matches!(command.as_str(), "lvert" | "rvert" | "lVert" | "rVert") {
            return Ok(Some(Expr::RawTex(format!("\\{command}"))));
        }
        if should_force_raw_simple_command(&command) {
            return Ok(Some(Expr::RawTex(format!("\\{command}"))));
        }
        if self.macros.contains_key(&command) {
            return Ok(Some(Expr::RawTex(format!("\\{command}"))));
        }
        if let Some(ch) = command_specific_to_char(&command) {
            return Ok(Some(Expr::CommandSymbol { command, ch }));
        }
        if let Some(ch) = big_symbol_command_to_char(&command) {
            return Ok(Some(Expr::BigSymbol(ch)));
        }
        if let Some(ch) = sum_operator_command_to_char(&command) {
            return Ok(Some(Expr::SumOperatorSymbol(ch)));
        }
        if let Some(ch) = delimiter_command_char(&command) {
            return Ok(Some(Expr::Char(ch)));
        }
        if let Some(chars) = command_to_sequence(&command) {
            return Ok(Some(Expr::Sequence(
                chars.iter().copied().map(Expr::Char).collect(),
            )));
        }
        if let Some(text) = command_to_text(&command) {
            return Ok(Some(Expr::Text(text.to_string())));
        }
        if let Some(ch) = command_to_char(&command) {
            return Ok(Some(Expr::Char(ch)));
        }
        if matches!(
            command.as_str(),
            "arg"
                | "argmax"
                | "argmin"
                | "arccos"
                | "arccot"
                | "arcctg"
                | "arccsc"
                | "arcsin"
                | "arcsec"
                | "arctan"
                | "arctg"
                | "ch"
                | "cos"
                | "cosec"
                | "cosh"
                | "cot"
                | "cotg"
                | "coth"
                | "csc"
                | "ctg"
                | "cth"
                | "deg"
                | "det"
                | "dim"
                | "exp"
                | "gcd"
                | "hom"
                | "injlim"
                | "ker"
                | "lg"
                | "lim"
                | "inf"
                | "liminf"
                | "limsup"
                | "ln"
                | "log"
                | "max"
                | "min"
                | "plim"
                | "projlim"
                | "sec"
                | "sh"
                | "sin"
                | "sinh"
                | "sup"
                | "tan"
                | "tanh"
                | "tg"
                | "th"
                | "Pr"
        ) {
            return Ok(Some(Expr::FunctionName(command)));
        }
        Ok(Some(Expr::RawTex(format!("\\{command}"))))
    }

    /// Parse one MathType-visible operand inside `\let`.
    ///
    /// Bug-fix: `\let` target parsing differs slightly from the normal path.
    /// Standalone vertical fence commands such as `\lvert` stay raw, while
    /// wrapper commands like `\textbf` disappear when no argument follows.
    fn parse_let_operand(&mut self, target_side: bool) -> Result<Option<Expr>, String> {
        self.skip_ws();
        if self.pos >= self.chars.len() {
            return Ok(None);
        }
        if target_side {
            if let Some(command) = self.consume_raw_let_target_command() {
                return Ok(Some(Expr::RawTex(command)));
            }
            if let Some(command) = self.peek_next_control_word() {
                if self.macros.contains_key(&command) {
                    self.pos += 1 + command.len();
                    if let Some(expr) = self.parse_shadowed_builtin_let_target_expr(&command)? {
                        return Ok(Some(expr));
                    }
                    return Ok(Some(Expr::Sequence(Vec::new())));
                }
            }
            if self.starts_command("sqrt") {
                return Ok(Some(normalize_let_target_expr(self.parse_let_target_sqrt()?)));
            }
            if let Some(raw) = self.consume_standalone_raw_let_command() {
                return Ok(Some(normalize_let_target_expr(Expr::RawTex(raw))));
            }
        }
        let expr = self.parse_atom_with_scripts()?;
        if target_side {
            return Ok(Some(self.normalize_let_target_operand(expr)?));
        }
        Ok(Some(expr))
    }

    /// Preserve native builtin parsing when a user macro shadows it inside `\let`.
    ///
    /// Bug-fix: after `\def\bar{...}`, MathType still parses `\let\foo\bar`
    /// like a native bar target and lets that target consume one visible operand
    /// when present, instead of freezing it as a pre-built empty placeholder.
    fn parse_shadowed_builtin_let_target_expr(
        &mut self,
        command: &str,
    ) -> Result<Option<Expr>, String> {
        let expr = match command {
            "bar" | "overline" => self.parse_shadowed_bar_let_target(BarTemplateKind::Over)?,
            "underline" => self.parse_shadowed_bar_let_target(BarTemplateKind::Under)?,
            _ => return Ok(None),
        };
        Ok(Some(expr))
    }

    /// Parse one shadowed bar-style `\let` target with the same operand-sniffing
    /// behavior that MathType keeps at the alias target site.
    fn parse_shadowed_bar_let_target(&mut self, kind: BarTemplateKind) -> Result<Expr, String> {
        let checkpoint = self.pos;
        let content = match self.parse_required_group_or_atom("let bar target content") {
            Ok(content) => content,
            Err(_) => {
                self.pos = checkpoint;
                return Ok(Expr::BarTemplate {
                    kind,
                    content: Box::new(Expr::Sequence(Vec::new())),
                });
            }
        };
        Ok(Expr::BarTemplate {
            kind,
            content: Box::new(content),
        })
    }


    /// Replay one `\let` alias to `\sqrt` without enabling `\sqrt`'s optional-index syntax.
    pub(super) fn parse_sqrt_let_alias(&mut self, leading_ws: &str) -> Result<Expr, String> {
        let checkpoint = self.pos;
        match self.parse_required_group_or_atom("square-root radicand") {
            Ok(radicand) => Ok(Expr::Sqrt(Box::new(radicand))),
            Err(_) => {
                self.pos = checkpoint;
                Ok(with_leading_raw_space(Expr::RawTex("\\sqrt".to_string()), leading_ws))
            }
        }
    }

    /// Normalize one `\let` target and, for empty wrapper-swallowed roots, keep consuming
    /// the next visible atom the way MathType still completes the target token.
    fn normalize_let_target_operand(&mut self, expr: Expr) -> Result<Expr, String> {
        let expr = normalize_let_target_expr(expr);
        if matches!(&expr, Expr::Sqrt(radicand) if expr_is_empty_sequence(radicand)) {
            let checkpoint = self.pos;
            if let Ok(radicand) = self.parse_required_group_or_atom("let sqrt fallback radicand") {
                return Ok(Expr::Sqrt(Box::new(radicand)));
            }
            self.pos = checkpoint;
        }
        Ok(expr)
    }

    /// Parse a `\\sqrt` let-target while allowing an omitted wrapper to reveal later bracket text.
    fn parse_let_target_sqrt(&mut self) -> Result<Expr, String> {
        let checkpoint = self.pos;
        self.pos += 1 + "sqrt".len();
        let index = match self.parse_optional_bracket_group() {
            Ok(index) => index,
            Err(_) => {
                self.pos = checkpoint;
                return Ok(Expr::RawTex("\\sqrt".to_string()));
            }
        };
        let radicand = match self.parse_required_group_or_atom("square-root radicand") {
            Ok(radicand) => {
                let normalized = normalize_let_target_expr(radicand);
                if expr_is_empty_sequence(&normalized) && self.peek() == Some('[') {
                    self.pos += 1;
                    Expr::Char('[')
                } else {
                    normalized
                }
            }
            Err(_) => {
                self.pos = checkpoint;
                return Ok(Expr::RawTex("\\sqrt".to_string()));
            }
        };
        Ok(if let Some(index) = index {
            Expr::NthRoot {
                index: Box::new(index),
                radicand: Box::new(radicand),
            }
        } else {
            Expr::Sqrt(Box::new(radicand))
        })
    }

    /// Peek one control-word token without consuming it.
    fn peek_next_control_word(&self) -> Option<String> {
        let mut index = self.pos;
        while self.chars.get(index).is_some_and(|ch| ch.is_whitespace()) {
            index += 1;
        }
        if self.chars.get(index) != Some(&'\\') {
            return None;
        }
        let start = index + 1;
        index = start;
        while self.chars.get(index).is_some_and(|ch| ch.is_ascii_alphabetic()) {
            index += 1;
        }
        (index > start).then(|| self.chars[start..index].iter().collect())
    }

    /// Return one `\\let` target command that MathType later replays as a built-in command.
    fn replayable_let_target_command(&self, command: &str) -> Option<String> {
        matches!(command, "sqrt").then(|| command.to_string())
    }

    /// Return true when the next token will stay on MathType's raw fallback path.
    fn next_starts_raw_fallback_command(&self) -> bool {
        if self.peek() != Some('\\') {
            return false;
        }
        let mut index = self.pos + 1;
        while self.chars.get(index).is_some_and(|ch| ch.is_ascii_alphabetic()) {
            index += 1;
        }
        let command = self.chars[self.pos + 1..index].iter().collect::<String>();
        if command.is_empty() || self.macros.contains_key(&command) {
            return false;
        }
        if should_force_raw_simple_command(&command)
            || omitted_let_target_raw_command(&format!("\\{command}"))
            || matches!(command.as_str(), "lvert" | "rvert" | "lVert" | "rVert")
        {
            return true;
        }
        if command_to_char(&command).is_some()
            || command_to_sequence(&command).is_some()
            || command_to_text(&command).is_some()
            || command_specific_to_char(&command).is_some()
            || delimiter_command_char(&command).is_some()
            || big_symbol_command_to_char(&command).is_some()
            || sum_operator_command_to_char(&command).is_some()
            || xarrow_command_kind(&command).is_some()
            || raw_hybrid_xarrow_command(&command)
            || spacing_command_width(&command).is_some()
            || matches!(
                command.as_str(),
                "arg"
                    | "argmax"
                    | "argmin"
                    | "arccos"
                    | "arccot"
                    | "arcctg"
                    | "arccsc"
                    | "arcsin"
                    | "arcsec"
                    | "arctan"
                    | "arctg"
                    | "ch"
                    | "cos"
                    | "cosec"
                    | "cosh"
                    | "cot"
                    | "cotg"
                    | "coth"
                    | "csc"
                    | "ctg"
                    | "cth"
                    | "deg"
                    | "det"
                    | "dim"
                    | "exp"
                    | "gcd"
                    | "hom"
                    | "injlim"
                    | "ker"
                    | "lg"
                    | "lim"
                    | "inf"
                    | "liminf"
                    | "limsup"
                    | "ln"
                    | "log"
                    | "max"
                    | "min"
                    | "plim"
                    | "projlim"
                    | "sec"
                    | "sh"
                    | "sin"
                    | "sinh"
                    | "sup"
                    | "tan"
                    | "tanh"
                    | "tg"
                    | "th"
                    | "Pr"
            )
        {
            return false;
        }
        true
    }

    /// Consume let-target commands that MathType keeps raw instead of as fence glyphs.
    fn consume_raw_let_target_command(&mut self) -> Option<String> {
        for command in ["lvert", "rvert", "lVert", "rVert"] {
            if self.starts_command(command) {
                self.pos += 1 + command.len();
                return Some(format!("\\{command}"));
            }
        }
        None
    }

    /// Consume one standalone raw command in a `\let` target without stealing following spaces.
    ///
    /// Bug-fix: MathType keeps source spaces after raw let-target commands such
    /// as `\bb` and `\choose`, so routing them through the generic unsupported
    /// parser would consume the separator too early.
    fn consume_standalone_raw_let_command(&mut self) -> Option<String> {
        if self.peek() != Some('\\') {
            return None;
        }
        let start = self.pos + 1;
        let mut index = start;
        while self.chars.get(index).is_some_and(|ch| ch.is_ascii_alphabetic()) {
            index += 1;
        }
        if index == start {
            return None;
        }
        let command = self.chars[start..index].iter().collect::<String>();
        if matches!(command.as_str(), "sqrt") {
            // Bug-fix: `\let\foo\sqrt` keeps MathType's native root template visible
            // instead of collapsing `\sqrt` into the surrounding raw fallback run.
            return None;
        }
        if self.macros.contains_key(&command)
            || command_to_char(&command).is_some()
            || command_to_sequence(&command).is_some()
            || command_to_text(&command).is_some()
            || command_specific_to_char(&command).is_some()
            || delimiter_command_char(&command).is_some()
            || big_symbol_command_to_char(&command).is_some()
            || sum_operator_command_to_char(&command).is_some()
            || xarrow_command_kind(&command).is_some()
            || raw_hybrid_xarrow_command(&command)
            || spacing_command_width(&command).is_some()
            || matches!(
                command.as_str(),
                "arg"
                    | "argmax"
                    | "argmin"
                    | "arccos"
                    | "arccot"
                    | "arcctg"
                    | "arccsc"
                    | "arcsin"
                    | "arcsec"
                    | "arctan"
                    | "arctg"
                    | "ch"
                    | "cos"
                    | "cosec"
                    | "cosh"
                    | "cot"
                    | "cotg"
                    | "coth"
                    | "csc"
                    | "ctg"
                    | "cth"
                    | "deg"
                    | "det"
                    | "dim"
                    | "exp"
                    | "gcd"
                    | "hom"
                    | "injlim"
                    | "ker"
                    | "lg"
                    | "lim"
                    | "inf"
                    | "liminf"
                    | "limsup"
                    | "ln"
                    | "log"
                    | "max"
                    | "min"
                    | "plim"
                    | "projlim"
                    | "sec"
                    | "sh"
                    | "sin"
                    | "sinh"
                    | "sup"
                    | "tan"
                    | "tanh"
                    | "tg"
                    | "th"
                    | "Pr"
            )
        {
            return None;
        }
        self.pos = index;
        Some(format!("\\{command}"))
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

    /// Parse `\middle` while leaving supported visible delimiters in the atom stream.
    ///
    /// Bug-fix: MathType preserves raw `\middle` but lets the following visible
    /// delimiter serialize on the normal visible path, so consuming both tokens
    /// here would create a nested sequence shape that does not match the passing
    /// raw/visible/raw hybrids.
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
        let checkpoint = self.pos;
        let delimiter = match self.parse_left_right_delimiter("middle delimiter") {
            Ok(delimiter) => delimiter,
            Err(_) => {
                self.pos = checkpoint;
                return Ok(Expr::RawTex("\\middle".to_string()));
            }
        };
        Ok(match delimiter {
            LeftRightDelimiter::Char('.') => Expr::RawTex("\\middle".to_string()),
            LeftRightDelimiter::Char(_) | LeftRightDelimiter::Command { .. } => {
                self.pos = checkpoint;
                Expr::RawTex("\\middle".to_string())
            }
            LeftRightDelimiter::RawCommand(command) => {
                Expr::RawTex(format!("\\middle\\{command}"))
            }
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
        let closing_delimiter = if delimiter == '{' { '}' } else { delimiter };
        let start = self.pos;
        while self.peek().is_some_and(|ch| ch != closing_delimiter) {
            self.pos += 1;
        }
        if self.peek() != Some(closing_delimiter) {
            self.pos = start;
            return Ok(Expr::RawTex("\\verb".to_string()));
        }
        let content = self.chars[start..self.pos].iter().collect::<String>();
        self.pos += 1;
        let visible = if delimiter == '{' {
            // Bug-fix: MathType keeps the malformed `\verb{a}` shell raw but
            // renders only the interior literal visibly, without showing braces.
            self.parse_visible_wrapper_text(&content)?
        } else {
            self.parse_visible_wrapper_text(&format!("{delimiter}{content}{delimiter}"))?
        };
        Ok(raw_prefix_expr("verb", visible))
    }

    /// Parse `\not` as a relation overlay instead of collapsing it to Unicode.
    ///
    /// MathType TeX Input keeps the base relation glyph and adds the same `embNOT`
    /// decoration in MTEF, so we preserve the inner relation expression here.
    /// Delimiter structures that do not accept `embNOT` fall back to MathType's
    /// generic up-strike template instead of dropping the visible target.
    pub(super) fn parse_not_relation(&mut self) -> Result<Expr, String> {
        let relation = self.parse_required_group_or_atom("not relation")?;
        if relation_char(&relation).is_none() {
            return Ok(match relation_uses_not_strike_template(&relation) {
                true => Expr::Strike {
                    kind: StrikeKind::Up,
                    content: Box::new(relation),
                },
                false => relation,
            });
        }
        Ok(Expr::NotRelation(Box::new(relation)))
    }

    /// Parse `\quantity` / `\qty` wrappers with MathType's mixed raw-prefix behavior.
    ///
    /// Bug-fix: MathType keeps `\quantity` raw but drops following size hints such
    /// as `\big(`, while `\qty\Bigg{}\Bigg[]` preserves only the first empty-group
    /// size hint inside the raw prefix and still renders the later delimiters visibly.
    pub(super) fn parse_physics_quantity_command(
        &mut self,
        command: &str,
    ) -> Result<Expr, String> {
        let checkpoint = self.pos;
        let starred = self.consume_optional_star();
        let mut raw_prefix = format!("\\{command}");
        let mut body_size_command = self.consume_optional_ignored_delimiter_size_command();
        if command == "qty"
            && body_size_command.is_some()
            && self.consume_empty_group()
        {
            raw_prefix.push('\\');
            raw_prefix.push_str(body_size_command.expect("checked qty size hint"));
            body_size_command = self.consume_optional_ignored_delimiter_size_command();
        }

        let Some(mut body) = self.parse_physics_auto_brace_body(command)? else {
            self.pos = checkpoint;
            let mut raw = raw_prefix;
            if starred {
                raw.push('*');
            }
            if let Some(size_command) = body_size_command {
                raw.push('\\');
                raw.push_str(size_command);
            }
            return Ok(Expr::RawTex(raw));
        };
        if matches!(body, PhysicsAutoBraceBody::Visible(_)) {
            if let Some(size_command) = body_size_command {
                raw_prefix.push('\\');
                raw_prefix.push_str(size_command);
            }
        } else if body_size_command.is_some() {
            body.mark_opening_delimiter();
        };

        let mut items = Vec::new();
        items.push(Expr::RawTex(raw_prefix));
        if starred {
            items.push(Expr::Char('*'));
        }
        push_visible_items(&mut items, body.into_expr());
        Ok(collapse_single_sequence(Expr::Sequence(items)))
    }

    /// Parse physics wrappers whose size hint stays as a separate raw run.
    ///
    /// Bug-fix: MathType emits `\abs`, `\norm`, and `\comm` as one raw command
    /// run, keeps a following `\Bigg` as another raw run, and only then renders
    /// the visible arguments.
    pub(super) fn parse_physics_raw_size_wrapper_command(
        &mut self,
        command: &str,
        arity: PhysicsAutoBraceArity,
    ) -> Result<Expr, String> {
        let checkpoint = self.pos;
        let starred = self.consume_optional_star();
        let size_command = self.consume_optional_ignored_delimiter_size_command();
        let Some(first) = self.parse_physics_auto_brace_body(command)? else {
            self.pos = checkpoint;
            let mut raw = format!("\\{command}");
            if starred {
                raw.push('*');
            }
            if let Some(size_command) = size_command {
                raw.push('\\');
                raw.push_str(size_command);
            }
            return Ok(Expr::RawTex(raw));
        };

        let mut items = Vec::new();
        items.push(Expr::RawTex(format!("\\{command}")));
        if starred {
            items.push(Expr::Char('*'));
        }
        if let Some(size_command) = size_command {
            items.push(Expr::RawTex(format!("\\{size_command}")));
        }
        push_visible_items(&mut items, first.into_expr());
        if arity == PhysicsAutoBraceArity::Two {
            let second = self.parse_required_group_or_atom(&format!("{command} argument"))?;
            push_visible_items(&mut items, second);
        }
        Ok(collapse_single_sequence(Expr::Sequence(items)))
    }

    /// Parse legacy `\cases{...}` fallback with raw braces and raw top-level separators.
    ///
    /// Bug-fix: MathType keeps `\cases{` and the closing `}` as raw text while
    /// still rendering cell content visibly, instead of dropping the braces.
    pub(super) fn parse_cases_command(&mut self, command: &str) -> Result<Expr, String> {
        let checkpoint = self.pos;
        let raw = match self.parse_raw_group(&format!("{command} content")) {
            Ok(raw) => raw,
            Err(_) => {
                self.pos = checkpoint;
                return Ok(Expr::RawTex(format!("\\{command}")));
            }
        };
        let (mut parts, has_top_level_separator) = self.parse_physics_matrix_raw_content(&raw)?;
        if !has_top_level_separator {
            let mut hybrid = Vec::new();
            append_hybrid_raw(&mut hybrid, format!("\\{command}"));
            hybrid.append(&mut parts);
            return Ok(Expr::HybridLayout(hybrid));
        }
        let mut hybrid = Vec::new();
        append_hybrid_raw(&mut hybrid, format!("\\{command}{{"));
        hybrid.append(&mut parts);
        append_hybrid_raw(&mut hybrid, "}");
        Ok(Expr::HybridLayout(hybrid))
    }

    /// Parse malformed old-TeX `\root ... \of ...` input on MathType's partial raw path.
    ///
    /// Bug-fix: samples such as `\root {3] \of 5` keep the raw `\root {` prefix
    /// and raw ` \of` separator while still rendering the malformed index and
    /// radicand visibly.
    pub(super) fn parse_malformed_root_command(&mut self) -> Result<Expr, String> {
        let checkpoint = self.pos;
        let mut raw_prefix = "\\root".to_string();
        let consumed_ws = self.consume_raw_whitespace();
        let Some(opening) = self.peek() else {
            self.pos = checkpoint;
            return Ok(Expr::RawTex("\\root".to_string()));
        };
        if opening != '{' && opening != '[' {
            self.pos = checkpoint;
            return Ok(Expr::RawTex("\\root".to_string()));
        }
        self.pos += 1;
        if opening == '{' {
            raw_prefix.push_str(&consumed_ws);
            raw_prefix.push(opening);
        }

        let segment_start = self.pos;
        while self.pos < self.chars.len() && !self.starts_command("of") {
            self.pos += 1;
        }
        if !self.starts_command("of") {
            self.pos = checkpoint;
            return Ok(Expr::RawTex("\\root".to_string()));
        }

        let mut raw_segment = self.chars[segment_start..self.pos].iter().collect::<String>();
        let trailing_ws = take_trailing_ascii_whitespace(&mut raw_segment);
        let mut raw_separator_prefix = String::new();
        if opening == '[' && raw_segment.ends_with('}') {
            raw_segment.pop();
            raw_separator_prefix.push('}');
        }
        let visible_index = if opening == '[' {
            format!("[{raw_segment}")
        } else {
            raw_segment.clone()
        };
        let index = self.parse_visible_wrapper_text(&visible_index)?;
        self.pos += 1 + "of".len();
        let radicand = self.parse_switch_content("root radicand")?;

        let mut items = vec![Expr::RawTex(raw_prefix)];
        push_visible_items(&mut items, index);
        items.push(Expr::RawTex(format!("{raw_separator_prefix}{trailing_ws}\\of")));
        push_visible_items(&mut items, radicand);
        Ok(collapse_single_sequence(Expr::Sequence(items)))
    }

    /// Parse physics matrix-like wrappers into MathType's hybrid raw-text plus LINE layout.
    pub(super) fn parse_physics_matrix_command(&mut self, command: &str) -> Result<Expr, String> {
        let checkpoint = self.pos;
        let starred = self.consume_optional_star();
        let body = match self.parse_physics_matrix_body(command)? {
            Some(body) => body,
            None => {
                self.pos = checkpoint;
                return Ok(Expr::RawTex(if starred {
                    format!("\\{command}*")
                } else {
                    format!("\\{command}")
                }));
            }
        };

        let mut parts = Vec::new();
        append_hybrid_raw(&mut parts, format!("\\{command}"));
        match body {
            PhysicsMatrixBody::Braced(raw) => {
                let (mut content_parts, has_top_level_separator) =
                    self.parse_physics_matrix_raw_content(&raw)?;
                if has_top_level_separator {
                    if starred {
                        append_hybrid_line(&mut parts, Expr::Char('*'));
                    }
                    append_hybrid_raw(&mut parts, "{");
                    parts.append(&mut content_parts);
                    append_hybrid_raw(&mut parts, "}");
                } else {
                    if starred {
                        prepend_hybrid_visible_prefix(&mut content_parts, Expr::Char('*'));
                    }
                    parts.append(&mut content_parts);
                }
            }
            PhysicsMatrixBody::Delimited {
                left,
                right,
                raw,
            } => {
                let (mut content_parts, _) = self.parse_physics_matrix_raw_content(&raw)?;
                let mut prefix = vec![Expr::Char(left)];
                if starred {
                    prefix.insert(0, Expr::Char('*'));
                }
                insert_hybrid_visible_prefix(
                    &mut content_parts,
                    collapse_single_sequence(Expr::Sequence(prefix)),
                );
                append_hybrid_visible_suffix(&mut content_parts, Expr::Char(right));
                parts.append(&mut content_parts);
            }
            PhysicsMatrixBody::Bare(content) => {
                let mut content_parts = Vec::new();
                append_expr_as_hybrid(&mut content_parts, content);
                if starred {
                    prepend_hybrid_visible_prefix(&mut content_parts, Expr::Char('*'));
                }
                parts.append(&mut content_parts);
            }
        }
        Ok(Expr::HybridLayout(parts))
    }

    /// Parse old-TeX matrix macros such as `\array{...}` on MathType's hybrid fallback path.
    ///
    /// Bug-fix: MathType preserves the command shell of legacy matrix macros as raw TeX,
    /// but still parses top-level cell content into visible LINE records.
    pub(super) fn parse_old_tex_matrix_command(&mut self, command: &str) -> Result<Expr, String> {
        let checkpoint = self.pos;
        let raw = match self.parse_raw_group(&format!("{command} content")) {
            Ok(raw) => raw,
            Err(_) => {
                self.pos = checkpoint;
                return Ok(Expr::RawTex(format!("\\{command}")));
            }
        };

        let (mut content_parts, keeps_braces) = self.parse_old_tex_matrix_raw_content(&raw)?;
        let mut parts = Vec::new();
        append_hybrid_raw(
            &mut parts,
            if keeps_braces {
                format!("\\{command}{{")
            } else {
                format!("\\{command}")
            },
        );
        parts.append(&mut content_parts);
        if keeps_braces {
            append_hybrid_raw(&mut parts, "}");
        }
        Ok(Expr::HybridLayout(parts))
    }

    /// Parse old-TeX alignment wrappers such as `\eqalignno{...}` on the same hybrid path.
    ///
    /// Bug-fix: MathType keeps the balanced wrapper braces and raw top-level `&`
    /// separators for these commands, instead of flattening them into a plain raw
    /// prefix followed by visible cells.
    pub(super) fn parse_old_tex_alignment_command(
        &mut self,
        command: &str,
    ) -> Result<Expr, String> {
        let checkpoint = self.pos;
        let raw = match self.parse_raw_group(&format!("{command} content")) {
            Ok(raw) => raw,
            Err(_) => {
                self.pos = checkpoint;
                return Ok(Expr::RawTex(format!("\\{command}")));
            }
        };

        let (mut content_parts, keeps_braces) = self.parse_old_tex_matrix_raw_content(&raw)?;
        let mut parts = Vec::new();
        append_hybrid_raw(
            &mut parts,
            if keeps_braces {
                format!("\\{command}{{")
            } else {
                format!("\\{command}")
            },
        );
        parts.append(&mut content_parts);
        if keeps_braces {
            append_hybrid_raw(&mut parts, "}");
        }
        Ok(Expr::HybridLayout(parts))
    }

    /// Parse one visible physics-wrapper body after any command-local size hint.
    fn parse_physics_auto_brace_body(
        &mut self,
        command: &str,
    ) -> Result<Option<PhysicsAutoBraceBody>, String> {
        self.skip_ws();
        match self.peek() {
            Some('{') => Ok(Some(PhysicsAutoBraceBody::Visible(
                self.parse_visible_wrapper_group(&format!("{command} content"))?,
            ))),
            Some('(') => self
                .parse_physics_auto_brace_delimited_body('(', ')', command)
                .map(|expr| Some(PhysicsAutoBraceBody::Delimited(expr))),
            Some('[') => self
                .parse_physics_auto_brace_delimited_body('[', ']', command)
                .map(|expr| Some(PhysicsAutoBraceBody::Delimited(expr))),
            Some('|') => self
                .parse_physics_auto_brace_delimited_body('|', '|', command)
                .map(|expr| Some(PhysicsAutoBraceBody::Delimited(expr))),
            Some(_) => Ok(Some(PhysicsAutoBraceBody::Visible(
                self.parse_switch_content(&format!("{command} content"))?,
            ))),
            None => Ok(None),
        }
    }

    /// Parse one delimiter-wrapped physics body while keeping the fence glyphs visible.
    fn parse_physics_auto_brace_delimited_body(
        &mut self,
        left: char,
        right: char,
        command: &str,
    ) -> Result<Expr, String> {
        let raw = self.parse_physics_delimited_body(left, right, command)?;
        let content = self.parse_visible_wrapper_text(&raw)?;
        let mut items = vec![Expr::Char(left)];
        push_visible_items(&mut items, content);
        items.push(Expr::Char(right));
        Ok(collapse_single_sequence(Expr::Sequence(items)))
    }

    /// Parse helper commands such as `\xmat` / `\zmat` that concatenate braced groups visibly.
    pub(super) fn parse_physics_multi_group_command(
        &mut self,
        command: &str,
        group_count: usize,
    ) -> Result<Expr, String> {
        let checkpoint = self.pos;
        let starred = self.consume_optional_star();
        let mut parts = Vec::new();
        append_hybrid_raw(&mut parts, format!("\\{command}"));
        let mut visible = Vec::new();
        if starred {
            visible.push(Expr::Char('*'));
        }
        for index in 0..group_count {
            let group = match self.parse_visible_wrapper_group(&format!(
                "{command} argument {}",
                index + 1
            )) {
                Ok(group) => group,
                Err(_) => {
                    self.pos = checkpoint;
                    return Ok(Expr::RawTex(if starred {
                        format!("\\{command}*")
                    } else {
                        format!("\\{command}")
                    }));
                }
            };
            push_visible_items(&mut visible, group);
        }
        append_hybrid_line(
            &mut parts,
            collapse_single_sequence(Expr::Sequence(visible)),
        );
        Ok(Expr::HybridLayout(parts))
    }

    /// Read the argument syntax used by one physics matrix-like wrapper command.
    fn parse_physics_matrix_body(
        &mut self,
        command: &str,
    ) -> Result<Option<PhysicsMatrixBody>, String> {
        self.skip_ws();
        match self.peek() {
            Some('{') => Ok(Some(PhysicsMatrixBody::Braced(self.parse_raw_group(
                &format!("{command} content"),
            )?))),
            Some('(') => Ok(Some(PhysicsMatrixBody::Delimited {
                left: '(',
                right: ')',
                raw: self.parse_physics_delimited_body('(', ')', command)?,
            })),
            Some('[') => Ok(Some(PhysicsMatrixBody::Delimited {
                left: '[',
                right: ']',
                raw: self.parse_physics_delimited_body('[', ']', command)?,
            })),
            Some('|') => Ok(Some(PhysicsMatrixBody::Delimited {
                left: '|',
                right: '|',
                raw: self.parse_physics_delimited_body('|', '|', command)?,
            })),
            Some(_) => Ok(Some(PhysicsMatrixBody::Bare(
                self.parse_switch_content(&format!("{command} content"))?,
            ))),
            None => Ok(None),
        }
    }

    /// Parse one visible-delimiter body used by physics matrix wrappers such as `\mqty(...)`.
    fn parse_physics_delimited_body(
        &mut self,
        left: char,
        right: char,
        command: &str,
    ) -> Result<String, String> {
        self.skip_ws();
        if self.peek() != Some(left) {
            return Err(format!("expected {left} after \\{command}"));
        }
        self.pos += 1;
        let start = self.pos;
        let mut brace_depth = 0usize;
        let mut same_delimiter_depth = 0usize;
        while self.pos < self.chars.len() {
            let ch = self.chars[self.pos];
            match ch {
                '{' => brace_depth += 1,
                '}' => brace_depth = brace_depth.saturating_sub(1),
                _ if brace_depth == 0 && left == '|' && ch == '|' => {
                    let end = self.pos;
                    self.pos += 1;
                    return Ok(self.chars[start..end].iter().collect());
                }
                _ if brace_depth == 0 && ch == left && left != '|' => {
                    same_delimiter_depth += 1;
                }
                _ if brace_depth == 0 && ch == right && left != '|' => {
                    if same_delimiter_depth == 0 {
                        let end = self.pos;
                        self.pos += 1;
                        return Ok(self.chars[start..end].iter().collect());
                    }
                    same_delimiter_depth -= 1;
                }
                _ => {}
            }
            self.pos += 1;
        }
        Err(format!("unterminated physics delimiter for \\{command}"))
    }

    /// Split one raw matrix-like body on top-level `&` and drop top-level `\\` separators.
    fn parse_physics_matrix_raw_content(
        &self,
        raw: &str,
    ) -> Result<(Vec<HybridPart>, bool), String> {
        let chars = raw.chars().collect::<Vec<_>>();
        let mut parts = Vec::new();
        let mut segment = String::new();
        let mut brace_depth = 0usize;
        let mut paren_depth = 0usize;
        let mut bracket_depth = 0usize;
        let mut bar_open = false;
        let mut saw_top_level_separator = false;
        let mut index = 0usize;
        while index < chars.len() {
            let ch = chars[index];
            if brace_depth == 0
                && paren_depth == 0
                && bracket_depth == 0
                && !bar_open
                && ch == '\\'
                && chars.get(index + 1) == Some(&'\\')
            {
                saw_top_level_separator = true;
                let had_trailing_ws = segment.chars().last().is_some_and(|ch| ch.is_ascii_whitespace());
                index += 2;
                while chars.get(index).is_some_and(|ch| ch.is_ascii_whitespace()) {
                    index += 1;
                }
                // Bug-fix: when a top-level `\\` is followed by `&`, MathType keeps at most one
                // separator space across the removed row break instead of concatenating both sides.
                if !had_trailing_ws && chars.get(index) == Some(&'&') {
                    segment.push(' ');
                }
                continue;
            }
            if brace_depth == 0
                && paren_depth == 0
                && bracket_depth == 0
                && !bar_open
                && ch == '&'
            {
                saw_top_level_separator = true;
                let trailing_ws = take_trailing_ascii_whitespace(&mut segment);
                self.append_physics_matrix_segment(&mut parts, &segment)?;
                segment.clear();
                append_hybrid_raw(&mut parts, format!("{trailing_ws}&"));
                index += 1;
                continue;
            }
            match ch {
                '{' => brace_depth += 1,
                '}' => brace_depth = brace_depth.saturating_sub(1),
                '(' if brace_depth == 0 && !bar_open => paren_depth += 1,
                ')' if brace_depth == 0 && !bar_open => {
                    paren_depth = paren_depth.saturating_sub(1)
                }
                '[' if brace_depth == 0 && !bar_open => bracket_depth += 1,
                ']' if brace_depth == 0 && !bar_open => {
                    bracket_depth = bracket_depth.saturating_sub(1)
                }
                '|' if brace_depth == 0 && paren_depth == 0 && bracket_depth == 0 => {
                    bar_open = !bar_open;
                }
                _ => {}
            }
            segment.push(ch);
            index += 1;
        }
        self.append_physics_matrix_segment(&mut parts, &segment)?;
        Ok((parts, saw_top_level_separator))
    }

    /// Split one old-TeX matrix body into visible cells plus raw separator fragments.
    ///
    /// Bug-fix: legacy macros such as `\array{a&b\\ c&d}` do not keep plain top-level
    /// row breaks as raw text, but they do preserve `\\[...]`, `\\\cr`, `\\\hline`,
    /// and `\\\hdashline`.
    fn parse_old_tex_matrix_raw_content(
        &self,
        raw: &str,
    ) -> Result<(Vec<HybridPart>, bool), String> {
        let chars = raw.chars().collect::<Vec<_>>();
        let mut parts = Vec::new();
        let mut segment = String::new();
        let mut brace_depth = 0usize;
        let mut paren_depth = 0usize;
        let mut bracket_depth = 0usize;
        let mut bar_open = false;
        let mut keeps_braces = false;
        let mut index = 0usize;

        while index < chars.len() {
            let ch = chars[index];
            if brace_depth == 0
                && paren_depth == 0
                && bracket_depth == 0
                && !bar_open
                && ch == '\\'
                && chars.get(index + 1) == Some(&'\\')
            {
                if let Some((raw_break, next_index)) =
                    parse_old_tex_matrix_row_break_raw(&chars, index + 2)
                {
                    self.append_physics_matrix_segment(&mut parts, &segment)?;
                    segment.clear();
                    append_hybrid_raw(&mut parts, raw_break);
                    keeps_braces = true;
                    index = next_index;
                    continue;
                }

                keeps_braces = true;
                index += 2;
                while chars.get(index).is_some_and(|ch| ch.is_ascii_whitespace()) {
                    index += 1;
                }
                continue;
            }
            if brace_depth == 0
                && paren_depth == 0
                && bracket_depth == 0
                && !bar_open
                && ch == '\\'
                && char_slice_starts_with(chars.as_slice(), index + 1, "cr")
            {
                self.append_physics_matrix_segment(&mut parts, &segment)?;
                segment.clear();
                append_hybrid_raw(&mut parts, "\\cr");
                index += 3;
                while chars.get(index).is_some_and(|ch| ch.is_ascii_whitespace()) {
                    index += 1;
                }
                continue;
            }
            if brace_depth == 0
                && paren_depth == 0
                && bracket_depth == 0
                && !bar_open
                && ch == '&'
            {
                keeps_braces = true;
                let trailing_ws = take_trailing_ascii_whitespace(&mut segment);
                self.append_physics_matrix_segment(&mut parts, &segment)?;
                segment.clear();
                append_hybrid_raw(&mut parts, format!("{trailing_ws}&"));
                index += 1;
                continue;
            }
            match ch {
                '{' => brace_depth += 1,
                '}' => brace_depth = brace_depth.saturating_sub(1),
                '(' if brace_depth == 0 && !bar_open => paren_depth += 1,
                ')' if brace_depth == 0 && !bar_open => {
                    paren_depth = paren_depth.saturating_sub(1)
                }
                '[' if brace_depth == 0 && !bar_open => bracket_depth += 1,
                ']' if brace_depth == 0 && !bar_open => {
                    bracket_depth = bracket_depth.saturating_sub(1)
                }
                '|' if brace_depth == 0 && paren_depth == 0 && bracket_depth == 0 => {
                    bar_open = !bar_open;
                }
                _ => {}
            }
            segment.push(ch);
            index += 1;
        }

        self.append_physics_matrix_segment(&mut parts, &segment)?;
        Ok((parts, keeps_braces))
    }

    /// Parse one matrix cell fragment and flatten nested raw-prefix hybrids into the outer run.
    fn append_physics_matrix_segment(
        &self,
        parts: &mut Vec<HybridPart>,
        segment: &str,
    ) -> Result<(), String> {
        if segment.trim().is_empty() {
            return Ok(());
        }
        let expr = self.parse_visible_wrapper_text(segment)?;
        append_expr_as_hybrid(parts, expr);
        Ok(())
    }

    /// Preserve MathType TeX Input fallback for unsupported control words.
    pub(super) fn parse_unsupported_command(&mut self, command: String) -> Result<Expr, String> {
        let mut raw = format!("\\{command}");
        if command == "right" {
            let checkpoint = self.pos;
            self.skip_ws();
            if let Ok((_delimiter, raw_delimiter)) =
                self.parse_left_right_delimiter_with_raw("right delimiter")
            {
                raw.push_str(&raw_delimiter);
                return Ok(Expr::RawTex(raw));
            }
            self.pos = checkpoint;
        }
        let mut consumed_size_hint = false;
        loop {
            let checkpoint = self.pos;
            self.skip_ws();
            let mut matched = false;
            for size_command in [
                "big", "Big", "bigg", "Bigg", "bigl", "Bigl", "bigr", "Bigr", "bigm", "Bigm",
                "biggl", "Biggl", "biggr", "Biggr", "biggm", "Biggm",
            ] {
                if self.starts_command(size_command) {
                    self.pos += 1 + size_command.len();
                    raw.push('\\');
                    raw.push_str(size_command);
                    consumed_size_hint = true;
                    matched = true;
                    break;
                }
            }
            if !matched {
                self.pos = checkpoint;
                break;
            }
        }
        let mut args = Vec::new();
        loop {
            let checkpoint = self.pos;
            self.skip_ws();
            if self.peek() == Some('{') {
                self.pos += 1;
                match self.parse_sequence(Some('}')) {
                    Ok(argument) => {
                        args.push(argument);
                        self.skip_ws();
                        continue;
                    }
                    Err(_) => {
                        self.pos = checkpoint;
                        break;
                    }
                }
            }
            self.pos = checkpoint;
            if self.peek() == Some('[') {
                self.skip_ws();
                match self.parse_optional_bracket_group() {
                    Ok(Some(argument)) => {
                        args.push(Expr::Sequence(vec![
                            Expr::Char('['),
                            argument,
                            Expr::Char(']'),
                        ]));
                        self.skip_ws();
                        continue;
                    }
                    Ok(None) => {
                        self.pos = checkpoint;
                        break;
                    }
                    Err(_) => {
                        self.pos = checkpoint;
                        break;
                    }
                }
            }
            self.pos = checkpoint;
            break;
        }
        if args.is_empty() && consumed_size_hint && !self.switch_content_stops() {
            args.push(self.parse_switch_content(&format!("{command} content"))?);
        }
        if args.is_empty() {
            if raw_command_preserves_trailing_space(&command) {
                raw.push_str(&self.consume_raw_whitespace());
            }
            Ok(Expr::RawTex(raw))
        } else {
            Ok(raw_prefix_sequence_with_raw(raw, args))
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
                        placement: LimitPlacement::NoLimits,
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

    /// Flatten repeated old-TeX `\over` separators into MathType's raw fallback form.
    ///
    /// Bug-fix: `1 \over 2 \over 3` does not nest native fractions in MathType.
    /// It keeps each ` \over` token raw between the visible operands.
    pub(super) fn parse_repeated_over_fallback(
        &mut self,
        left_items: Vec<Expr>,
        until: Option<char>,
    ) -> Result<Expr, String> {
        let mut items = left_items;
        loop {
            items.push(Expr::RawTex(" \\over".to_string()));
            let operand = self.parse_required_group_or_atom("over operand")?;
            push_visible_items(&mut items, operand);

            let checkpoint = self.pos;
            let _ = self.consume_raw_whitespace();
            if matches!(self.consume_infix_command(), Some(InfixCommand::Over)) {
                continue;
            }
            self.pos = checkpoint;
            let tail = self.parse_sequence(until)?;
            push_visible_items(&mut items, tail);
            return Ok(collapse_single_sequence(Expr::Sequence(items)));
        }
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
            while self.peek() == Some('\'') {
                self.pos += 1;
                atom = append_postfix_prime(atom);
            }
            if let Some(expr) = self.parse_bracket_prime_group()? {
                atom = match atom {
                    Expr::Sequence(mut items) => {
                        push_visible_items(&mut items, expr);
                        Expr::Sequence(items)
                    }
                    other => {
                        let mut items = vec![other];
                        push_visible_items(&mut items, expr);
                        Expr::Sequence(items)
                    }
                };
                continue;
            }
            let consumed_ws = self.consume_raw_whitespace();
            if let Some(modifier) = self.consume_limits_modifier() {
                limit_modifier = Some(modifier);
                continue;
            }
            match self.peek() {
                Some('_') => {
                    self.pos += 1;
                    let checkpoint = self.pos;
                    match self.parse_script_arg() {
                        Ok(sub) => {
                            let active_modifier = limit_modifier.take();
                            if let Some(modifier) = active_modifier {
                                if !expr_supports_limits_modifier(&atom)
                                    || expr_has_attached_scripts(&atom)
                                {
                                    atom = raw_limits_script_suffix_expr(atom, modifier, "_", sub);
                                    continue;
                                }
                            }
                            if let Some(expr) =
                                raw_group_script_suffix_expr(atom.clone(), "_", sub.clone())
                            {
                                atom = expr;
                                continue;
                            }
                            if let Some(expr) =
                                append_raw_script_followup(atom.clone(), "_", sub.clone())
                            {
                                atom = expr;
                                continue;
                            }
                            if let Some(expr) = delimited_bodyless_big_op_outer_script_expr(
                                atom.clone(),
                                "_",
                                sub.clone(),
                            ) {
                                atom = expr;
                                continue;
                            }
                            if let Some(raw) = repeated_script_marker_raw('_', &sub) {
                                atom = raw_script_suffix_expr(atom, raw);
                                continue;
                            }
                            atom = merge_script(atom, Some(sub), None, active_modifier);
                        }
                        Err(_) => {
                            // Bug-fix: MathType keeps a dangling `_` as raw text after the
                            // visible base instead of rejecting the whole formula.
                            self.pos = checkpoint;
                            atom = Expr::Sequence(vec![atom, Expr::RawTex("_".to_string())]);
                        }
                    }
                }
                Some('^') => {
                    self.pos += 1;
                    if self.peek() == Some('\'') {
                        self.pos += 1;
                        atom = raw_script_suffix_expr(atom, "^'");
                        continue;
                    }
                    if let Some(expr) =
                        self.parse_raw_malformed_superscript_suffix(atom.clone())?
                    {
                        atom = expr;
                        continue;
                    }
                    if let Some(raw) = self.parse_raw_prime_script_group()? {
                        atom = raw_superscript_suffix_expr(atom, &raw);
                        continue;
                    }
                    let checkpoint = self.pos;
                    match self.parse_script_arg() {
                        Ok(sup) => {
                            let active_modifier = limit_modifier.take();
                            if let Some(modifier) = active_modifier {
                                if !expr_supports_limits_modifier(&atom)
                                    || expr_has_attached_scripts(&atom)
                                {
                                    atom = raw_limits_script_suffix_expr(atom, modifier, "^", sup);
                                    continue;
                                }
                            }
                            if let Some(expr) =
                                raw_group_script_suffix_expr(atom.clone(), "^", sup.clone())
                            {
                                atom = expr;
                                continue;
                            }
                            if let Some(expr) =
                                append_raw_script_followup(atom.clone(), "^", sup.clone())
                            {
                                atom = expr;
                                continue;
                            }
                            if let Some(expr) = delimited_bodyless_big_op_outer_script_expr(
                                atom.clone(),
                                "^",
                                sup.clone(),
                            ) {
                                atom = expr;
                                continue;
                            }
                            if let Some(raw) = repeated_script_marker_raw('^', &sup) {
                                atom = raw_script_suffix_expr(atom, raw);
                                continue;
                            }
                            atom = merge_script(atom, None, Some(sup), active_modifier);
                        }
                        Err(_) => {
                            // Bug-fix: MathType keeps a dangling `^` as raw text after the
                            // visible base instead of rejecting the whole formula.
                            self.pos = checkpoint;
                            atom = Expr::Sequence(vec![atom, Expr::RawTex("^".to_string())]);
                        }
                    }
                }
                _ => {
                    if let Some(modifier) = limit_modifier.take() {
                        if !expr_supports_limits_modifier(&atom) {
                            atom = raw_limits_modifier_expr(atom, modifier);
                        }
                    }
                    if !consumed_ws.is_empty() {
                        if self.starts_row_separator()
                            && self
                                .active_unsupported_envs
                                .last()
                                .is_some_and(|name| unsupported_environment_keeps_row_separator(name))
                        {
                            // Bug-fix: unsupported environments such as `gather*`
                            // preserve the source space immediately before a raw
                            // top-level `\\`, so leave that whitespace for the
                            // outer fallback parser instead of deferring it here.
                            self.pos -= consumed_ws.chars().count();
                            break;
                        }
                        // MathType keeps this source-space inside the next raw-text fallback run.
                        self.pending_raw_ws.push_str(&consumed_ws);
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
    #[allow(dead_code)]
    pub(super) fn parse_sequence_until_right(&mut self) -> Result<Expr, String> {
        let mut items = Vec::new();
        loop {
            let leading_ws = self.consume_raw_whitespace();
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
            let atom = with_leading_raw_space(self.parse_complete_atom()?, &leading_ws);
            if expr_is_empty_sequence(&atom) {
                continue;
            }
            if expr_is_mathtype_translation_failed(&atom) {
                return Ok(atom);
            }
            items.push(atom);
        }
        Ok(Expr::Sequence(items))
    }

    /// Parse one `\left` body, returning whether a matching `\right` was found.
    pub(super) fn parse_sequence_until_right_or_eof(&mut self) -> Result<(Expr, bool), String> {
        let mut items = Vec::new();
        loop {
            let leading_ws = self.consume_raw_whitespace();
            if self.pos >= self.chars.len() {
                return Ok((Expr::Sequence(items), false));
            }
            if self.starts_command("right") {
                return Ok((Expr::Sequence(items), true));
            }
            if let Some(infix) = self.consume_infix_command() {
                let left = Expr::Sequence(items);
                let (right, terminated) = self.parse_sequence_until_right_or_eof()?;
                return Ok((infix_expr(infix, left, right), terminated));
            }
            let atom = with_leading_raw_space(self.parse_complete_atom()?, &leading_ws);
            if expr_is_empty_sequence(&atom) {
                continue;
            }
            if expr_is_mathtype_translation_failed(&atom) {
                return Ok((atom, false));
            }
            items.push(atom);
        }
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

    /// Parse `\right` and keep the exact raw token text for partial fallback.
    pub(super) fn parse_right_delimiter_spec_with_raw(
        &mut self,
    ) -> Result<(LeftRightDelimiter, String), String> {
        let start = self.pos;
        let delimiter = self.parse_right_delimiter_spec()?;
        let raw: String = self.chars[start..self.pos].iter().collect();
        Ok((delimiter, raw))
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
                if let Some(ch) = command_specific_to_char(&command) {
                    return Ok(LeftRightDelimiter::Command { command, ch });
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

    /// Parse one delimiter and also return the exact raw source text it consumed.
    pub(super) fn parse_left_right_delimiter_with_raw(
        &mut self,
        label: &str,
    ) -> Result<(LeftRightDelimiter, String), String> {
        let start = self.pos;
        let delimiter = self.parse_left_right_delimiter(label)?;
        let raw: String = self.chars[start..self.pos].iter().collect();
        Ok((delimiter, raw))
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

    /// Consume one ignored delimiter-size hint such as `\Bigg` when present.
    fn consume_optional_ignored_delimiter_size_command(&mut self) -> Option<&'static str> {
        self.skip_ws();
        for command in [
            "big", "Big", "bigg", "Bigg", "bigl", "Bigl", "bigr", "Bigr", "bigm", "Bigm",
            "biggl", "Biggl", "biggr", "Biggr", "biggm", "Biggm",
        ] {
            if self.starts_command(command) {
                self.pos += 1 + command.len();
                return Some(command);
            }
        }
        None
    }

    /// Collapse one standalone delimiter-size hint into MathType's marked fence glyph.
    ///
    /// Bug-fix: commands such as `\Bigg[` and `\Biggr]` do not vanish completely.
    /// MathType keeps a line-marker byte in front of the visible delimiter.
    pub(super) fn parse_ignored_delimiter_size_command(&mut self) -> Result<Expr, String> {
        let checkpoint = self.pos;
        match self.parse_delimiter_char("size-hinted delimiter") {
            Ok(ch) => Ok(Expr::MarkedChar(ch)),
            Err(_) => {
                self.pos = checkpoint;
                Ok(Expr::Sequence(Vec::new()))
            }
        }
    }

    /// Consume one empty braced group used by `\qty\Bigg{}`-style wrappers.
    ///
    /// Bug-fix: MathType folds this empty group into the preceding raw `\qty\Bigg`
    /// prefix instead of rendering a visible empty argument.
    fn consume_empty_group(&mut self) -> bool {
        self.skip_ws();
        if self.peek() != Some('{') {
            return false;
        }
        let checkpoint = self.pos;
        self.pos += 1;
        match self.parse_sequence(Some('}')) {
            Ok(content) if expr_is_empty_sequence(&content) => true,
            _ => {
                self.pos = checkpoint;
                false
            }
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
            let atom = self.parse_complete_atom()?;
            if expr_is_empty_sequence(&atom) {
                self.skip_ws();
                continue;
            }
            if expr_is_mathtype_translation_failed(&atom) {
                return Ok(atom);
            }
            items.push(atom);
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
        let checkpoint = self.pos;
        let _ = self.consume_optional_star();
        if self.parse_raw_group("hspace width").is_err() {
            self.pos = checkpoint;
            let starred = self.consume_optional_star();
            if self.parse_required_group_or_atom("hspace width").is_ok() {
                // Bug-fix: malformed `\hspace 2pt` still consumes one width atom in
                // MathType, so the command itself disappears and only the trailing
                // source characters remain visible.
                return Ok(Expr::Sequence(Vec::new()));
            }
            self.pos = checkpoint;
            return Ok(Expr::RawTex(if starred {
                "\\hspace*".to_string()
            } else {
                "\\hspace".to_string()
            }));
        }
        Ok(Expr::Sequence(Vec::new()))
    }

    /// Parse \cline as a MathType-ignored table-rule hint when a span is present.
    pub(super) fn parse_cline_content(&mut self) -> Result<Expr, String> {
        let checkpoint = self.pos;
        if self.parse_raw_group("cline span").is_err() {
            self.pos = checkpoint;
            return Ok(Expr::RawTex("\\cline".to_string()));
        }
        Ok(Expr::Sequence(Vec::new()))
    }

    /// Parse `\ce{...}` with MathType's hybrid isotope-prefix fallback.
    ///
    /// Bug-fix: mhchem isotope prefixes such as `\ce{^{227}_{90}Th+}` keep the
    /// opening `\ce{^` shell raw in MathType, and some subscript forms also keep
    /// a raw `_` marker before the visible native script slot.
    pub(super) fn parse_ce_content(&mut self) -> Result<Expr, String> {
        let checkpoint = self.pos;
        let content = match self.parse_raw_group("ce content") {
            Ok(content) => content,
            Err(_) => {
                self.pos = checkpoint;
                return Ok(Expr::RawTex("\\ce".to_string()));
            }
        };
        if let Some(expr) = self.parse_ce_isotope_prefix(&content)? {
            return Ok(expr);
        }
        Ok(raw_prefix_expr("ce", self.parse_visible_wrapper_text(&content)?))
    }

    /// Parse the leading isotope-script shell that MathType keeps partially raw in `\ce{...}`.
    fn parse_ce_isotope_prefix(&self, content: &str) -> Result<Option<Expr>, String> {
        if !content.starts_with('^') {
            return Ok(None);
        }
        let Some((_sup_raw, sup_expr, mut index, _sup_braced)) =
            parse_ce_script_token(self, content, 1)?
        else {
            return Ok(None);
        };

        let mut items = vec![Expr::RawTex("\\ce{^".to_string())];
        if content.as_bytes().get(index) == Some(&b'_') {
            let Some((sub_raw, sub_expr, next_index, sub_braced)) =
                parse_ce_script_token(self, content, index + 1)?
            else {
                return Ok(None);
            };
            push_visible_items(&mut items, sup_expr);
            if sub_braced || !ce_compact_subscript_token(&sub_raw) {
                items.push(Expr::RawTex("_".to_string()));
                push_visible_items(&mut items, sub_expr);
            } else {
                let (first_sub, trailing_sub) = split_compact_ce_subscript(self, &sub_raw)?;
                items.push(empty_base_sub_script(first_sub));
                if let Some(trailing_sub) = trailing_sub {
                    push_visible_items(&mut items, trailing_sub);
                }
            }
            index = next_index;
        } else {
            push_visible_items(&mut items, sup_expr);
        }

        if let Some(rest) = content.get(index..) {
            if !rest.is_empty() {
                push_visible_items(&mut items, self.parse_visible_wrapper_text(rest)?);
            }
        }
        items.push(Expr::RawTex("}".to_string()));
        Ok(Some(collapse_single_sequence(Expr::Sequence(items))))
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

    /// Recover malformed superscript groups whose prime shell stays raw in MathType.
    ///
    /// Bug-fix: inputs such as `^{'_{a}}`, `^{'^{a}}`, and `^{a^{'}}` do not
    /// become nested native scripts in MathType. It preserves the malformed
    /// prime/script shell as raw TeX while still rendering the simple visible
    /// payload in between when one exists.
    pub(super) fn parse_raw_malformed_superscript_suffix(
        &mut self,
        base: Expr,
    ) -> Result<Option<Expr>, String> {
        self.skip_ws();
        if self.peek() != Some('{') {
            return Ok(None);
        }
        let checkpoint = self.pos;
        let raw = self.parse_raw_group("malformed superscript")?;
        if let Some(inner) = raw.strip_prefix("'_{").and_then(|rest| rest.strip_suffix('}')) {
            if is_raw_prime_script_group(inner) {
                return Ok(Some(raw_superscript_suffix_expr(base, &raw)));
            }
            let visible = Parser::new(inner).parse()?;
            return Ok(Some(raw_script_hybrid_suffix_expr(
                base,
                "^{'_",
                visible,
                "}",
            )));
        }
        if let Some(inner) = raw.strip_prefix("'^{").and_then(|rest| rest.strip_suffix('}')) {
            if is_raw_prime_script_group(inner) {
                return Ok(Some(raw_superscript_suffix_expr(base, &raw)));
            }
            let visible = Parser::new(inner).parse()?;
            return Ok(Some(raw_script_hybrid_suffix_expr(
                base,
                "^{'^",
                visible,
                "}",
            )));
        }
        if let Some((visible_prefix, raw_tail)) = split_prime_superscript_tail(&raw) {
            let visible = Parser::new(visible_prefix).parse()?;
            let mut raw_suffix = format!("^{{{raw_tail}}}");
            raw_suffix.push('}');
            return Ok(Some(raw_script_hybrid_suffix_expr(
                base,
                "^{",
                visible,
                &raw_suffix,
            )));
        }
        self.pos = checkpoint;
        Ok(None)
    }

    /// Parse bracketed derivative markers such as `[']` without normalizing them away.
    pub(super) fn parse_bracket_prime_group(&mut self) -> Result<Option<Expr>, String> {
        if self.peek() != Some('[') {
            return Ok(None);
        }
        let checkpoint = self.pos;
        let raw = match self.parse_raw_square_group_without_skip("prime bracket") {
            Ok(raw) => raw,
            Err(_) => {
                // Bug-fix: bracketed-prime recovery is optional. Unterminated or
                // ordinary `[...]` input must fall back to the normal parser path
                // instead of rejecting the whole formula.
                self.pos = checkpoint;
                return Ok(None);
            }
        };
        if is_raw_prime_script_group(&raw) {
            let prime_count = raw.chars().filter(|ch| *ch == '\'').count();
            let prime_expr = if prime_count == 1 {
                Expr::CommandSymbol {
                    command: "prime".to_string(),
                    ch: '\u{2032}',
                }
            } else {
                Expr::Sequence(
                    std::iter::repeat_n(
                        Expr::CommandSymbol {
                            command: "prime".to_string(),
                            ch: '\u{2032}',
                        },
                        prime_count,
                    )
                    .collect(),
                )
            };
            let mut items = vec![Expr::Char('[')];
            items.push(Expr::Script {
                base: Box::new(Expr::Sequence(Vec::new())),
                sub: None,
                sup: Some(Box::new(prime_expr)),
            });
            items.push(Expr::Char(']'));
            Ok(Some(collapse_single_sequence(Expr::Sequence(items))))
        } else {
            self.pos = checkpoint;
            Ok(None)
        }
    }

    /// Return raw text inside one bracketed group without interpreting its contents.
    #[allow(dead_code)]
    fn parse_raw_square_group(&mut self, label: &str) -> Result<String, String> {
        self.skip_ws();
        self.parse_raw_square_group_without_skip(label)
    }

    /// Return raw text inside one bracketed group without skipping leading source whitespace.
    ///
    /// Bug-fix: optional bracket-prime parsing must not eat ordinary inter-atom spaces,
    /// because MathType keeps that whitespace inside the next raw fallback run.
    fn parse_raw_square_group_without_skip(&mut self, label: &str) -> Result<String, String> {
        if self.peek() != Some('[') {
            return Err(format!("expected bracketed {label}"));
        }
        self.pos += 1;
        let start = self.pos;
        let mut depth = 1usize;
        while let Some(ch) = self.next() {
            match ch {
                '[' => depth += 1,
                ']' => {
                    depth -= 1;
                    if depth == 0 {
                        let end = self.pos - 1;
                        return Ok(self.chars[start..end].iter().collect());
                    }
                }
                _ => {}
            }
        }
        Err(format!("unterminated bracketed {label}"))
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

/// Split a superscript group into visible prefix plus one prime-only nested superscript tail.
fn split_prime_superscript_tail(raw: &str) -> Option<(&str, &str)> {
    let tail_start = raw.rfind("^{")?;
    let visible_prefix = &raw[..tail_start];
    let tail = raw
        .get(tail_start + 2..raw.len().checked_sub(1)?)
        .filter(|_| raw.ends_with('}'))?;
    if visible_prefix.is_empty() || !is_raw_prime_script_group(tail) {
        return None;
    }
    Some((visible_prefix, tail))
}

/// Return the visible fence glyph for `\left...\right` pairs that still stay native.
fn left_right_visible_char(delimiter: &LeftRightDelimiter) -> Option<char> {
    match delimiter {
        LeftRightDelimiter::Char(ch) => Some(*ch),
        LeftRightDelimiter::Command { ch, .. } => Some(*ch),
        LeftRightDelimiter::RawCommand(_) => None,
    }
}

/// Return one preserved raw row-break fragment used by old-TeX matrix macros.
///
/// Plain top-level `\\` separators are removed, but MathType keeps width-specifying
/// `\\[...]` and control-word tails such as `\\\cr` as raw TeX.
fn parse_old_tex_matrix_row_break_raw(chars: &[char], start: usize) -> Option<(String, usize)> {
    let mut index = start;
    while chars.get(index).is_some_and(|ch| ch.is_ascii_whitespace()) {
        index += 1;
    }

    if chars.get(index) == Some(&'[') {
        return Some(("\\\\".to_string(), index));
    }

    if chars.get(index) == Some(&'\\') {
        for command in ["hdashline", "cr"] {
            if char_slice_starts_with(chars, index + 1, command) {
                let mut raw = String::from("\\\\");
                raw.push('\\');
                raw.push_str(command);
                let mut next_index = index + 1 + command.len();
                while chars
                    .get(next_index)
                    .is_some_and(|ch| ch.is_ascii_whitespace())
                {
                    next_index += 1;
                }
                return Some((raw, next_index));
            }
        }
        if char_slice_starts_with(chars, index + 1, "hline") {
            let mut next_index = index + 1 + "hline".len();
            while chars
                .get(next_index)
                .is_some_and(|ch| ch.is_ascii_whitespace())
            {
                next_index += 1;
            }
            return Some(("\\\\".to_string(), next_index));
        }
    }

    None
}

/// Return true when a character slice matches one ASCII control-word suffix.
fn char_slice_starts_with(chars: &[char], start: usize, text: &str) -> bool {
    text.chars()
        .enumerate()
        .all(|(offset, ch)| chars.get(start + offset) == Some(&ch))
}

/// Drop standalone wrapper commands that MathType swallows when they appear as `\let` targets.
fn normalize_let_target_expr(expr: Expr) -> Expr {
    match expr {
        Expr::RawTex(raw) if omitted_let_target_raw_command(&raw) => Expr::Sequence(Vec::new()),
        Expr::Sqrt(radicand) => Expr::Sqrt(Box::new(normalize_let_target_expr(*radicand))),
        Expr::Font { kind, content } => Expr::Font {
            kind,
            content: Box::new(normalize_let_target_expr(*content)),
        },
        Expr::Style { kind, content } => Expr::Style {
            kind,
            content: Box::new(normalize_let_target_expr(*content)),
        },
        Expr::Sequence(items) => Expr::Sequence(
            items.into_iter().map(normalize_let_target_expr).collect(),
        ),
        other => other,
    }
}

/// Return true for raw wrapper commands that disappear as standalone `\let` targets.
fn omitted_let_target_raw_command(raw: &str) -> bool {
    matches!(
        raw,
        "\\textbf"
            | "\\mathbf"
            | "\\bold"
            | "\\boldsymbol"
            | "\\textit"
            | "\\mathit"
            | "\\mathrm"
            | "\\textsf"
            | "\\mathsf"
            | "\\texttt"
            | "\\mathtt"
            | "\\textnormal"
            | "\\mathnormal"
    )
}

/// Keep track of the source form used by one physics matrix-like wrapper.
enum PhysicsMatrixBody {
    Braced(String),
    Delimited { left: char, right: char, raw: String },
    Bare(Expr),
}

/// Keep track of whether one physics wrapper consumes one or two visible arguments.
#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) enum PhysicsAutoBraceArity {
    One,
    Two,
}

/// Distinguish delimiter-wrapped bodies from plain visible content in physics wrappers.
enum PhysicsAutoBraceBody {
    Delimited(Expr),
    Visible(Expr),
}

impl PhysicsAutoBraceBody {
    /// Return the visible parser expression after the wrapper-specific raw decision is made.
    fn into_expr(self) -> Expr {
        match self {
            Self::Delimited(expr) | Self::Visible(expr) => expr,
        }
    }

    /// Mark the opening delimiter when a dropped size hint still leaves MathType's line marker.
    fn mark_opening_delimiter(&mut self) {
        let Self::Delimited(Expr::Sequence(items)) = self else {
            return;
        };
        if let Some(Expr::Char(ch)) = items.first_mut() {
            let ch = *ch;
            *items.first_mut().expect("delimited body has a first item") = Expr::MarkedChar(ch);
        }
    }
}

/// Parse one leading mhchem isotope token after `^` or `_`.
fn parse_ce_script_token(
    parser: &Parser,
    content: &str,
    start: usize,
) -> Result<Option<(String, Expr, usize, bool)>, String> {
    let bytes = content.as_bytes();
    if start >= bytes.len() {
        return Ok(None);
    }
    if bytes[start] == b'{' {
        let mut depth = 1usize;
        let mut index = start + 1;
        while index < bytes.len() {
            match bytes[index] {
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        let raw = content[start + 1..index].to_string();
                        let expr = parser.parse_visible_wrapper_text(&raw)?;
                        return Ok(Some((raw, expr, index + 1, true)));
                    }
                }
                _ => {}
            }
            index += 1;
        }
        return Ok(None);
    }

    let token_end = ce_unbraced_script_end(content, start);
    if token_end == start {
        return Ok(None);
    }
    let raw = content[start..token_end].to_string();
    let expr = parser.parse_visible_wrapper_text(&raw)?;
    Ok(Some((raw, expr, token_end, false)))
}

/// Return the end of one unbraced mhchem isotope token.
fn ce_unbraced_script_end(content: &str, start: usize) -> usize {
    let bytes = content.as_bytes();
    let mut index = start;
    if bytes.get(index).is_some_and(|byte| matches!(*byte, b'+' | b'-')) {
        index += 1;
    }
    let digit_start = index;
    while bytes.get(index).is_some_and(|byte| byte.is_ascii_digit()) {
        index += 1;
    }
    if index > digit_start {
        return index;
    }
    while bytes
        .get(index)
        .is_some_and(|byte| byte.is_ascii_alphanumeric())
    {
        index += 1;
    }
    index
}

/// Return true when MathType folds one subscript into the same empty-base script shell.
fn ce_compact_subscript_token(token: &str) -> bool {
    !token.is_empty() && token.bytes().all(|byte| byte.is_ascii_alphanumeric())
}

/// Build the empty-base subscript shell MathType uses for compact mhchem isotope tails.
fn empty_base_sub_script(sub: Expr) -> Expr {
    Expr::Script {
        base: Box::new(Expr::Sequence(Vec::new())),
        sub: Some(Box::new(sub)),
        sup: None,
    }
}

/// Split one compact mhchem subscript into the scripted first glyph plus any visible tail.
fn split_compact_ce_subscript(
    parser: &Parser,
    token: &str,
) -> Result<(Expr, Option<Expr>), String> {
    let mut chars = token.chars();
    let first = chars
        .next()
        .ok_or_else(|| "compact ce subscript token is empty".to_string())?;
    let first_expr = parser.parse_visible_wrapper_text(&first.to_string())?;
    let rest: String = chars.collect();
    let trailing = if rest.is_empty() {
        None
    } else {
        Some(parser.parse_visible_wrapper_text(&rest)?)
    };
    Ok((first_expr, trailing))
}

/// Remove trailing ASCII whitespace so top-level `&` stays in MathType's raw run.
fn take_trailing_ascii_whitespace(text: &mut String) -> String {
    let split = text.trim_end_matches(char::is_whitespace).len();
    text.split_off(split)
}

/// Return true when `\not` falls back to MathType's generic strike template.
fn relation_uses_not_strike_template(expr: &Expr) -> bool {
    match expr {
        Expr::Delimited { .. } | Expr::OneSidedDelimited { .. } => true,
        Expr::Sequence(items) => matches!(items.as_slice(), [item] if relation_uses_not_strike_template(item)),
        _ => false,
    }
}

/// Preserve outer scripts on fenced bodyless big operators as raw/native hybrids.
///
/// Bug-fix: MathType does not keep `\left( \sum_1^n \right)^{2}` on the native
/// delimited-script path. It preserves the fence and outer script shell as raw
/// TeX while still rendering the standalone operator glyph and its inner limits
/// visibly.
fn delimited_bodyless_big_op_outer_script_expr(
    base: Expr,
    marker: &str,
    script: Expr,
) -> Option<Expr> {
    let (left, right, kind, lower, upper) = delimited_bodyless_big_op_parts(&base)?;
    let mut items = vec![Expr::RawTex(format!("\\left{left}"))];
    items.push(bodyless_big_op_visible_expr(kind));
    if let Some(lower) = lower {
        items.push(Expr::RawTex("_".to_string()));
        push_visible_items(&mut items, lower);
    }
    if let Some(upper) = upper {
        items.push(Expr::RawTex("^".to_string()));
        push_visible_items(&mut items, upper);
    }
    items.push(Expr::RawTex(format!(" \\right{right}{marker}")));
    push_visible_items(&mut items, script);
    Some(collapse_single_sequence(Expr::Sequence(items)))
}

/// Return the fence and limit parts for one delimited bodyless big operator.
fn delimited_bodyless_big_op_parts(
    expr: &Expr,
) -> Option<(char, char, BigOpKind, Option<Expr>, Option<Expr>)> {
    match expr {
        Expr::Delimited {
            left,
            right,
            content,
        } => match content.as_ref() {
            Expr::BigOp {
                kind,
                lower,
                upper,
                body: None,
                placement: LimitPlacement::Limits,
            } if lower.is_some() || upper.is_some() => Some((
                *left,
                *right,
                *kind,
                lower.as_deref().cloned(),
                upper.as_deref().cloned(),
            )),
            Expr::Sequence(items) => match items.as_slice() {
                [item] => delimited_bodyless_big_op_parts(&Expr::Delimited {
                    left: *left,
                    right: *right,
                    content: Box::new(item.clone()),
                }),
                _ => None,
            },
            _ => None,
        },
        Expr::Sequence(items) => match items.as_slice() {
            [item] => delimited_bodyless_big_op_parts(item),
            _ => None,
        },
        _ => None,
    }
}

/// Return the visible standalone glyph expression for one bodyless big operator.
fn bodyless_big_op_visible_expr(kind: BigOpKind) -> Expr {
    match kind {
        BigOpKind::Sum => Expr::SumOperatorSymbol('\u{2211}'),
        BigOpKind::Product => Expr::BigSymbol('\u{220f}'),
        BigOpKind::Coproduct => Expr::BigSymbol('\u{2210}'),
        BigOpKind::Union => Expr::BigSymbol('\u{22c3}'),
        BigOpKind::Intersection => Expr::BigSymbol('\u{22c2}'),
    }
}



