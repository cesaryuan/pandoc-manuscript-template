use super::*;

impl Parser {
    /// Return true when the remaining input starts with a specific control word.
    pub(super) fn starts_command(&self, expected: &str) -> bool {
        if self.peek() != Some('\\') {
            return false;
        }
        let mut index = self.pos + 1;
        for expected_char in expected.chars() {
            if self.chars.get(index) != Some(&expected_char) {
                return false;
            }
            index += 1;
        }
        !self
            .chars
            .get(index)
            .is_some_and(|ch| ch.is_ascii_alphabetic())
    }

    /// Return true when the remaining input is exactly \end{expected}.
    pub(super) fn starts_end_environment(&self, expected: &str) -> bool {
        if !self.starts_command("end") {
            return false;
        }
        let mut index = self.pos + "\\end".len();
        if self.chars.get(index) != Some(&'{') {
            return false;
        }
        index += 1;
        for expected_char in expected.chars() {
            if self.chars.get(index) != Some(&expected_char) {
                return false;
            }
            index += 1;
        }
        self.chars.get(index) == Some(&'}')
    }

    /// Consume old TeX infix commands that map to existing native templates.
    pub(super) fn consume_infix_command(&mut self) -> Option<InfixCommand> {
        let infix = if self.starts_command("over") {
            InfixCommand::Over
        } else if self.starts_command("above") {
            self.expect('\\').ok()?;
            while self.peek().is_some_and(|ch| ch.is_ascii_alphabetic()) {
                self.pos += 1;
            }
            let thickness = self.parse_raw_group("above line thickness").ok()?;
            let thickness_expr = self.parse_visible_wrapper_text(&thickness).ok()?;
            return Some(InfixCommand::Above(Expr::Sequence(vec![
                Expr::RawTex(" \\above".to_string()),
                thickness_expr,
            ])));
        } else if self.starts_command("atop") {
            InfixCommand::Atop
        } else if self.starts_command("choose") {
            InfixCommand::Choose
        } else if self.starts_command("brace") {
            InfixCommand::Brace
        } else if self.starts_command("brack") {
            InfixCommand::Brack
        } else {
            return None;
        };
        self.expect('\\').ok()?;
        while self.peek().is_some_and(|ch| ch.is_ascii_alphabetic()) {
            self.pos += 1;
        }
        Some(infix)
    }

    /// Consume a single required character.
    pub(super) fn expect(&mut self, expected: char) -> Result<(), String> {
        match self.next() {
            Some(actual) if actual == expected => Ok(()),
            other => Err(format!("expected {expected:?}, found {other:?}")),
        }
    }

    /// Return the current character without consuming it.
    pub(super) fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    /// Consume and return the current character.
    pub(super) fn next(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.pos += 1;
        Some(ch)
    }

    /// Ignore whitespace, matching MathType's treatment for simple TeX input.
    pub(super) fn skip_ws(&mut self) {
        self.consume_ws();
    }

    /// Ignore whitespace and report whether at least one space was consumed.
    pub(super) fn consume_ws(&mut self) -> bool {
        let start = self.pos;
        while self.peek().is_some_and(char::is_whitespace) {
            self.pos += 1;
        }
        self.pos != start
    }

    /// Consume one deferred space that belongs to the next raw fallback token.
    pub(super) fn take_pending_raw_ws(&mut self) -> bool {
        let pending = self.pending_raw_ws;
        self.pending_raw_ws = false;
        pending
    }

    /// Consume raw source whitespace so fallback environments can reproduce
    /// MathType's row-leading trivia instead of one fixed separator prefix.
    pub(super) fn consume_raw_whitespace(&mut self) -> String {
        let start = self.pos;
        while self.peek().is_some_and(char::is_whitespace) {
            self.pos += 1;
        }
        self.chars[start..self.pos].iter().collect()
    }

    /// Recover separator-adjacent source whitespace for fallback environments.
    ///
    /// This is a bug fix for rows like `a &= b`, where MathType preserves the
    /// literal space before `&` in raw fallback runs.
    pub(super) fn recover_environment_separator_prefix(&self) -> String {
        let mut start = self.pos;
        while start > 0 && self.chars[start - 1].is_whitespace() {
            if matches!(self.chars[start - 1], '\r' | '\n') {
                break;
            }
            start -= 1;
        }
        self.chars[start..self.pos].iter().collect()
    }
}
