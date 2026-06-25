use crate::ast::*;

/// Strip math delimiters and keep multiline source records stable for MathType.
pub(crate) fn normalize_latex(input: &str) -> String {
    let text = input.trim().trim_start_matches('\u{feff}').trim();
    let (body, stripped_delimiters) =
        if text.starts_with("$$") && text.ends_with("$$") && text.len() >= 4 {
            (text[2..text.len() - 2].trim(), true)
        } else if text.starts_with('$') && text.ends_with('$') && text.len() >= 2 {
            (text[1..text.len() - 1].trim(), true)
        } else {
            (text, false)
        };
    if stripped_delimiters {
        // Existing MathType references were regenerated from stripped snippets with CRLF.
        body.replace("\r\n", "\n")
            .replace('\r', "\n")
            .replace('\n', "\r\n")
    } else {
        // File-based helper input without outer math delimiters preserves source line endings.
        body.to_string()
    }
}

pub(crate) struct Parser {
    chars: Vec<char>,
    pos: usize,
}

impl Parser {
    /// Create a parser for the currently supported TeX math subset.
    pub(crate) fn new(input: &str) -> Self {
        Self {
            chars: input.chars().collect(),
            pos: 0,
        }
    }

    /// Parse the full formula and reject trailing unsupported syntax.
    pub(crate) fn parse(mut self) -> Result<Expr, String> {
        let expr = self.parse_sequence(None)?;
        self.skip_ws();
        if self.pos != self.chars.len() {
            return Err(format!("unexpected character {:?}", self.peek()));
        }
        Ok(expr)
    }

    /// Parse a sequence until an optional closing delimiter is reached.
    fn parse_sequence(&mut self, until: Option<char>) -> Result<Expr, String> {
        let mut items = Vec::new();
        loop {
            self.skip_ws();
            if self.pos >= self.chars.len() || until.is_some_and(|end| self.peek() == Some(end)) {
                break;
            }
            let atom = self.parse_complete_atom()?;
            items.push(atom);
        }
        if let Some(end) = until {
            self.expect(end)?;
        }
        Ok(Expr::Sequence(items))
    }

    /// Parse one atom, including a small set of control words.
    fn parse_atom(&mut self) -> Result<Expr, String> {
        self.skip_ws();
        match self.peek() {
            Some('{') => {
                self.pos += 1;
                self.parse_sequence(Some('}'))
            }
            Some('\\') => self.parse_command(),
            Some(ch) if ch != '}' => {
                self.pos += 1;
                Ok(Expr::Char(ch))
            }
            other => Err(format!("expected atom, found {other:?}")),
        }
    }

    /// Parse subscript/superscript arguments, accepting either groups or atoms.
    fn parse_script_arg(&mut self) -> Result<Expr, String> {
        self.skip_ws();
        if self.peek() == Some('{') {
            self.pos += 1;
            self.parse_sequence(Some('}'))
        } else {
            self.parse_atom()
        }
    }

    /// Parse supported LaTeX commands that map directly to MTEF templates.
    fn parse_command(&mut self) -> Result<Expr, String> {
        self.expect('\\')?;
        let start = self.pos;
        while self.peek().is_some_and(|ch| ch.is_ascii_alphabetic()) {
            self.pos += 1;
        }
        let command: String = self.chars[start..self.pos].iter().collect();
        match command.as_str() {
            "frac" => {
                let numerator = self.parse_required_group("fraction numerator")?;
                let denominator = self.parse_required_group("fraction denominator")?;
                Ok(Expr::Fraction(Box::new(numerator), Box::new(denominator)))
            }
            "cfrac" => {
                let numerator = self.parse_required_group("continued fraction numerator")?;
                let denominator = self.parse_required_group("continued fraction denominator")?;
                Ok(Expr::Fraction(Box::new(numerator), Box::new(denominator)))
            }
            "sqrt" => {
                let index = self.parse_optional_bracket_group()?;
                let radicand = self.parse_required_group("square-root radicand")?;
                Ok(if let Some(index) = index {
                    Expr::NthRoot {
                        index: Box::new(index),
                        radicand: Box::new(radicand),
                    }
                } else {
                    Expr::Sqrt(Box::new(radicand))
                })
            }
            "sum" => Ok(Expr::BigOp {
                kind: BigOpKind::Sum,
                lower: None,
                upper: None,
                body: None,
            }),
            "prod" => Ok(Expr::BigOp {
                kind: BigOpKind::Product,
                lower: None,
                upper: None,
                body: None,
            }),
            "int" => Ok(Expr::Integral {
                kind: IntegralKind::Single,
            }),
            "iint" => Ok(Expr::Integral {
                kind: IntegralKind::Double,
            }),
            "oint" => Ok(Expr::Integral {
                kind: IntegralKind::Contour,
            }),
            "binom" => {
                let upper = self.parse_required_group("binomial upper")?;
                let lower = self.parse_required_group("binomial lower")?;
                Ok(Expr::Binomial(Box::new(upper), Box::new(lower)))
            }
            "begin" => {
                let name = self.parse_raw_group("environment name")?;
                self.parse_environment(&name)
            }
            "left" => {
                let left = self.parse_delimiter_char("left delimiter")?;
                let content = self.parse_sequence_until_right()?;
                let right = self.parse_right_delimiter()?;
                Ok(Expr::Delimited {
                    left,
                    right,
                    content: Box::new(content),
                })
            }
            "quad" => Ok(Expr::Space(0x05)),
            "qquad" => Ok(Expr::Space(0x06)),
            "!" => Ok(Expr::Space(0x02)),
            "," => Ok(Expr::Space(0x08)),
            "text" => Ok(Expr::Text(self.parse_raw_group("text content")?)),
            "color" => {
                let name = self.parse_raw_group("color name")?;
                Ok(Expr::Color {
                    name,
                    content: Box::new(self.parse_required_group("colored content")?),
                })
            }
            "operatorname" => Ok(Expr::FunctionName(self.parse_raw_group("operator name")?)),
            "arg" | "exp" | "ln" | "log" | "max" | "min" | "lim" | "sup" | "Pr" => {
                Ok(Expr::FunctionName(command))
            }
            "mathbf" => Ok(Expr::Font {
                kind: FontKind::Bold,
                content: Box::new(self.parse_required_group("mathbf content")?),
            }),
            "mathcal" => Ok(Expr::Font {
                kind: FontKind::MathCal,
                content: Box::new(self.parse_required_group("mathcal content")?),
            }),
            "mathsf" => Ok(Expr::Font {
                kind: FontKind::MathSf,
                content: Box::new(self.parse_required_group("mathsf content")?),
            }),
            "mathbb" => Ok(Expr::Font {
                kind: FontKind::MathBb,
                content: Box::new(self.parse_required_group("mathbb content")?),
            }),
            "mathscr" => Ok(Expr::Font {
                kind: FontKind::MathScr,
                content: Box::new(self.parse_required_group("mathscr content")?),
            }),
            "bar" => Ok(Expr::Accent {
                kind: AccentKind::Bar,
                content: Box::new(self.parse_required_group("bar content")?),
            }),
            "hat" => Ok(Expr::Accent {
                kind: AccentKind::Hat,
                content: Box::new(self.parse_required_group("hat content")?),
            }),
            "widehat" => Ok(Expr::Accent {
                kind: AccentKind::WideHat,
                content: Box::new(self.parse_required_group("widehat content")?),
            }),
            "vec" => Ok(Expr::Accent {
                kind: AccentKind::Vec,
                content: Box::new(self.parse_required_group("vec content")?),
            }),
            "overbrace" | "underbrace" => self.parse_required_group("brace content"),
            "stackrel" => {
                let upper = self.parse_required_group("stackrel upper")?;
                let lower = self.parse_required_group("stackrel lower")?;
                Ok(Expr::Script {
                    base: Box::new(lower),
                    sub: None,
                    sup: Some(Box::new(upper)),
                })
            }
            "xrightarrow" => {
                let label = self.parse_required_group("arrow label")?;
                Ok(Expr::Sequence(vec![Expr::Char('→'), label]))
            }
            _ if command_to_char(&command).is_some() => {
                Ok(Expr::Char(command_to_char(&command).unwrap()))
            }
            "" => {
                let ch = self
                    .next()
                    .ok_or_else(|| "dangling backslash".to_string())?;
                if ch == '!' {
                    return Ok(Expr::Space(0x02));
                }
                if ch == ',' {
                    return Ok(Expr::Space(0x08));
                }
                Ok(Expr::Char(ch))
            }
            _ => Err(format!("unsupported LaTeX command: \\{command}")),
        }
    }

    /// Parse one atom, its scripts, and any operand required by big operators.
    fn parse_complete_atom(&mut self) -> Result<Expr, String> {
        let mut atom = self.parse_atom_with_scripts()?;
        if matches!(atom, Expr::BigOp { body: None, .. }) {
            let body = self.parse_big_op_operand()?;
            if let Expr::BigOp {
                body: body_slot, ..
            } = &mut atom
            {
                *body_slot = Some(Box::new(body));
            }
        } else if matches!(atom, Expr::IntegralOp { body: None, .. }) {
            let body = self.parse_big_op_operand()?;
            if let Expr::IntegralOp {
                body: body_slot, ..
            } = &mut atom
            {
                *body_slot = Some(Box::new(body));
            }
        }
        Ok(atom)
    }

    /// Parse an atom followed by optional subscript/superscript records.
    fn parse_atom_with_scripts(&mut self) -> Result<Expr, String> {
        let mut atom = self.parse_atom()?;
        loop {
            self.skip_ws();
            match self.peek() {
                Some('_') => {
                    self.pos += 1;
                    let sub = self.parse_script_arg()?;
                    atom = merge_script(atom, Some(sub), None);
                }
                Some('^') => {
                    self.pos += 1;
                    let sup = self.parse_script_arg()?;
                    atom = merge_script(atom, None, Some(sup));
                }
                _ => break,
            }
        }
        Ok(atom)
    }

    /// Parse the term MathType places in the first slot of a big-op template.
    fn parse_big_op_operand(&mut self) -> Result<Expr, String> {
        self.skip_ws();
        if self.big_op_operand_stops() {
            return Err("expected big-operator operand".to_string());
        }
        self.parse_complete_atom()
    }

    /// Stop a big-op operand at top-level separators; nested groups parse themselves.
    fn big_op_operand_stops(&self) -> bool {
        self.pos >= self.chars.len()
            || self.starts_command("right")
            || matches!(self.peek(), Some('}' | ',' | '+' | '-' | '=' | '&'))
    }

    /// Parse a dynamic delimiter body until the paired \right command.
    fn parse_sequence_until_right(&mut self) -> Result<Expr, String> {
        let mut items = Vec::new();
        loop {
            self.skip_ws();
            if self.pos >= self.chars.len() {
                return Err("unterminated \\left...\\right group".to_string());
            }
            if self.starts_command("right") {
                break;
            }
            let atom = self.parse_complete_atom()?;
            items.push(atom);
        }
        Ok(Expr::Sequence(items))
    }

    /// Parse rows and columns for the supported \begin...\end environments.
    fn parse_environment(&mut self, name: &str) -> Result<Expr, String> {
        let kind = match name {
            "align" => EnvironmentKind::Align,
            "aligned" => EnvironmentKind::Aligned,
            "cases" => EnvironmentKind::Cases,
            "pmatrix" => {
                return Ok(Expr::Matrix {
                    kind: MatrixKind::Parenthesized,
                    rows: self.parse_environment_rows(name)?,
                })
            }
            "bmatrix" => {
                return Ok(Expr::Matrix {
                    kind: MatrixKind::Bracketed,
                    rows: self.parse_environment_rows(name)?,
                })
            }
            other => return Err(format!("unsupported LaTeX environment: {other}")),
        };
        Ok(Expr::Environment {
            kind,
            rows: self.parse_environment_rows(name)?,
        })
    }

    /// Parse an environment into rows split by & and \\ separators.
    fn parse_environment_rows(&mut self, name: &str) -> Result<Vec<Vec<Expr>>, String> {
        let mut rows = Vec::new();
        loop {
            if self.starts_command("end") {
                self.consume_end_environment(name)?;
                break;
            }
            let mut cells = Vec::new();
            loop {
                let cell = self.parse_sequence_until_environment_stop(name)?;
                cells.push(cell);
                self.skip_ws();
                if self.peek() == Some('&') {
                    self.pos += 1;
                    continue;
                }
                if self.consume_row_separator() {
                    break;
                }
                if self.starts_command("end") {
                    self.consume_end_environment(name)?;
                    return Ok(rows_with_row(rows, cells));
                }
                return Err(format!("expected &, \\\\, or \\end{{{name}}}"));
            }
            rows.push(cells);
        }
        Ok(rows)
    }

    /// Parse a cell until an environment separator appears at the current nesting level.
    fn parse_sequence_until_environment_stop(&mut self, name: &str) -> Result<Expr, String> {
        let mut items = Vec::new();
        loop {
            self.skip_ws();
            if self.pos >= self.chars.len()
                || self.peek() == Some('&')
                || self.starts_command("end")
                || self.starts_row_separator()
            {
                break;
            }
            let _ = name;
            items.push(self.parse_complete_atom()?);
        }
        Ok(Expr::Sequence(items))
    }

    /// Return true when the input is at a row separator command.
    fn starts_row_separator(&self) -> bool {
        self.peek() == Some('\\') && self.chars.get(self.pos + 1) == Some(&'\\')
    }

    /// Consume a row separator command if present.
    fn consume_row_separator(&mut self) -> bool {
        if self.starts_row_separator() {
            self.pos += 2;
            true
        } else {
            false
        }
    }

    /// Consume \end{name} for the currently parsed environment.
    fn consume_end_environment(&mut self, expected: &str) -> Result<(), String> {
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

    /// Consume the \right command and return its visible delimiter.
    fn parse_right_delimiter(&mut self) -> Result<char, String> {
        self.expect('\\')?;
        let start = self.pos;
        while self.peek().is_some_and(|ch| ch.is_ascii_alphabetic()) {
            self.pos += 1;
        }
        let command: String = self.chars[start..self.pos].iter().collect();
        if command != "right" {
            return Err(format!("expected \\right, found \\{command}"));
        }
        self.parse_delimiter_char("right delimiter")
    }

    /// Parse one delimiter after \left or \right, including escaped braces.
    fn parse_delimiter_char(&mut self, label: &str) -> Result<char, String> {
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
            return self.next().ok_or_else(|| format!("expected {label}"));
        }
        self.next().ok_or_else(|| format!("expected {label}"))
    }

    /// Parse an optional bracketed group such as the index in \sqrt[n]{...}.
    fn parse_optional_bracket_group(&mut self) -> Result<Option<Expr>, String> {
        self.skip_ws();
        if self.peek() != Some('[') {
            return Ok(None);
        }
        self.pos += 1;
        Ok(Some(self.parse_sequence(Some(']'))?))
    }

    /// Return true when the remaining input starts with a specific control word.
    fn starts_command(&self, expected: &str) -> bool {
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

    /// Parse a required braced group and report which slot was missing.
    fn parse_required_group(&mut self, label: &str) -> Result<Expr, String> {
        self.skip_ws();
        if self.peek() != Some('{') {
            return Err(format!("expected braced {label}"));
        }
        self.pos += 1;
        self.parse_sequence(Some('}'))
    }

    /// Return raw text inside a simple braced group such as \operatorname{mean}.
    fn parse_raw_group(&mut self, label: &str) -> Result<String, String> {
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

    /// Consume a single required character.
    fn expect(&mut self, expected: char) -> Result<(), String> {
        match self.next() {
            Some(actual) if actual == expected => Ok(()),
            other => Err(format!("expected {expected:?}, found {other:?}")),
        }
    }

    /// Return the current character without consuming it.
    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    /// Consume and return the current character.
    fn next(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.pos += 1;
        Some(ch)
    }

    /// Ignore whitespace, matching MathType's treatment for simple TeX input.
    fn skip_ws(&mut self) {
        while self.peek().is_some_and(char::is_whitespace) {
            self.pos += 1;
        }
    }
}

/// Preserve MathType's postfix script template shape by merging repeated scripts.
fn merge_script(base: Expr, sub: Option<Expr>, sup: Option<Expr>) -> Expr {
    match base {
        Expr::BigOp {
            kind,
            lower,
            upper,
            body,
        } => Expr::BigOp {
            kind,
            lower: sub.map(Box::new).or(lower),
            upper: sup.map(Box::new).or(upper),
            body,
        },
        Expr::Integral { kind } => Expr::IntegralOp {
            kind,
            lower: sub.map(Box::new),
            upper: sup.map(Box::new),
            body: None,
        },
        Expr::IntegralOp {
            kind,
            lower,
            upper,
            body,
        } => Expr::IntegralOp {
            kind,
            lower: sub.map(Box::new).or(lower),
            upper: sup.map(Box::new).or(upper),
            body,
        },
        Expr::FunctionName(name) if matches!(name.as_str(), "lim" | "sup") => Expr::Limit {
            name,
            lower: sub.map(Box::new),
            upper: sup.map(Box::new),
        },
        Expr::Limit { name, lower, upper } => Expr::Limit {
            name,
            lower: sub.map(Box::new).or(lower),
            upper: sup.map(Box::new).or(upper),
        },
        Expr::Script {
            base,
            sub: old_sub,
            sup: old_sup,
        } => Expr::Script {
            base,
            sub: sub.map(Box::new).or(old_sub),
            sup: sup.map(Box::new).or(old_sup),
        },
        other => Expr::Script {
            base: Box::new(other),
            sub: sub.map(Box::new),
            sup: sup.map(Box::new),
        },
    }
}

/// Append a final environment row while keeping ownership straightforward.
fn rows_with_row(mut rows: Vec<Vec<Expr>>, row: Vec<Expr>) -> Vec<Vec<Expr>> {
    rows.push(row);
    rows
}

/// Map delimiter commands used after \left and \right to visible fence characters.
fn delimiter_command_char(command: &str) -> Option<char> {
    match command {
        "{" => Some('{'),
        "}" => Some('}'),
        "|" => Some('|'),
        "langle" => Some('〈'),
        "rangle" => Some('〉'),
        "lfloor" => Some('⌊'),
        "rfloor" => Some('⌋'),
        "lceil" => Some('⌈'),
        "rceil" => Some('⌉'),
        _ => None,
    }
}

/// Map no-argument LaTeX commands to the Unicode symbol MathType stores.
fn command_to_char(command: &str) -> Option<char> {
    match command {
        "alpha" => Some('α'),
        "beta" => Some('β'),
        "gamma" => Some('γ'),
        "delta" => Some('δ'),
        "epsilon" => Some('ϵ'),
        "lambda" => Some('λ'),
        "pi" => Some('π'),
        "rho" => Some('ρ'),
        "chi" => Some('χ'),
        "omega" => Some('ω'),
        "xi" => Some('ξ'),
        "mu" => Some('μ'),
        "phi" => Some('ϕ'),
        "Delta" => Some('Δ'),
        "Psi" => Some('Ψ'),
        "Omega" => Some('Ω'),
        "Lambda" => Some('Λ'),
        "Gamma" => Some('Γ'),
        "times" => Some('×'),
        "cdot" => Some('⋅'),
        "in" => Some('∈'),
        "infty" => Some('∞'),
        "equiv" => Some('≡'),
        "to" => Some('→'),
        "nabla" => Some('∇'),
        "forall" => Some('∀'),
        "oplus" => Some('⊕'),
        "otimes" => Some('⊗'),
        "propto" => Some('∝'),
        "approx" => Some('≈'),
        "partial" => Some('∂'),
        "le" | "leq" => Some('≤'),
        "pm" => Some('±'),
        "circ" => Some('°'),
        "cup" => Some('∪'),
        "dots" => Some('…'),
        "leftarrow" => Some('←'),
        "ldots" => Some('…'),
        "ne" | "neq" => Some('≠'),
        "ge" | "geq" => Some('≥'),
        _ => None,
    }
}
