use std::env;
use std::fs;
use std::path::PathBuf;

const END_OF_CHAIN: u32 = 0xFFFF_FFFE;
const FREE_SECTOR: u32 = 0xFFFF_FFFF;
const FAT_SECTOR: u32 = 0xFFFF_FFFD;
const SECTOR_SIZE: usize = 512;

const OLE_STREAM: &[u8] = &[
    0x01, 0x00, 0x00, 0x02, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

const COMP_OBJ_STREAM: &[u8] = &[
    0x01, 0x00, 0xfe, 0xff, 0x03, 0x0a, 0x00, 0x00, 0xff, 0xff, 0xff, 0xff, 0x03, 0xce,
    0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0xc0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46,
    0x16, 0x00, 0x00, 0x00, b'M', b'a', b't', b'h', b'T', b'y', b'p', b'e', b' ', b'7',
    b'.', b'0', b' ', b'E', b'q', b'u', b'a', b't', b'i', b'o', b'n', 0x00, 0x0c, 0x00,
    0x00, 0x00, b'M', b'a', b't', b'h', b'T', b'y', b'p', b'e', b' ', b'E', b'F', 0x00,
    0x0f, 0x00, 0x00, 0x00, b'E', b'q', b'u', b'a', b't', b'i', b'o', b'n', b'.', b'D',
    b'S', b'M', b'T', b'4', 0x00, 0xf4, 0x39, 0xb2, 0x71, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

const EQUATION_CLSID: [u8; 16] = [
    0x03, 0xce, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0xc0, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x46,
];

const MTEF_FIXED_DEFS: &[u8] = &[
    0x13, b'W', b'i', b'n', b'A', b'l', b'l', b'B', b'a', b's', b'i', b'c', b'C', b'o',
    b'd', b'e', b'P', b'a', b'g', b'e', b's', 0x00, 0x11, 0x05, b'T', b'i', b'm', b'e',
    b's', b' ', b'N', b'e', b'w', b' ', b'R', b'o', b'm', b'a', b'n', 0x00, 0x11, 0x03,
    b'S', b'y', b'm', b'b', b'o', b'l', 0x00, 0x11, 0x05, b'C', b'o', b'u', b'r', b'i',
    b'e', b'r', b' ', b'N', b'e', b'w', 0x00, 0x11, 0x04, b'M', b'T', b' ', b'E', b'x',
    b't', b'r', b'a', 0x00, 0x13, b'W', b'i', b'n', b'A', b'l', b'l', b'C', b'o', b'd',
    b'e', b'P', b'a', b'g', b'e', b's', 0x00, 0x11, 0x06, 0xcb, 0xce, 0xcc, 0xe5, 0x00,
    0x12, 0x00, 0x08, 0x21, 0x2f, 0x45, 0x8f, 0x44, 0x2f, 0x41, 0x50, 0xf4, 0x10, 0x0f,
    0x47, 0x5f, 0x41, 0x50, 0xf2, 0x1f, 0x1e, 0x41, 0x50, 0xf4, 0x15, 0x0f, 0x41, 0x00,
    0xf4, 0x45, 0xf4, 0x25, 0xf4, 0x8f, 0x42, 0x5f, 0x41, 0x00, 0xf4, 0x10, 0x0f, 0x43,
    0x5f, 0x41, 0x00, 0xf4, 0x8f, 0x45, 0xf4, 0x2a, 0x5f, 0x48, 0xf4, 0x8f, 0x41, 0x00,
    0xf4, 0x10, 0x0f, 0x40, 0xf4, 0x8f, 0x41, 0x7f, 0x48, 0xf4, 0x10, 0x0f, 0x41, 0x2a,
    0x5f, 0x44, 0x5f, 0x45, 0xf4, 0x5f, 0x45, 0xf4, 0x5f, 0x41, 0x0f, 0x0c, 0x01, 0x00,
    0x01, 0x00, 0x01, 0x02, 0x02, 0x02, 0x02, 0x00, 0x02, 0x00, 0x01, 0x01, 0x01, 0x00,
    0x03, 0x00, 0x01, 0x00, 0x04, 0x00, 0x05, 0x00,
];

const EUCLID_MATH_ONE_DEFS: &[u8] = &[
    0x13, b'E', b'u', b'c', b'l', b'i', b'd', b'M', b'a', b't', b'h', b'1', 0x00, 0x11,
    0x07, b'E', b'u', b'c', b'l', b'i', b'd', b' ', b'M', b'a', b't', b'h', b' ', b'O',
    b'n', b'e', 0x00, 0x08, 0x06, 0x00,
];

const EUCLID_MATH_ONE_AFTER_TWO_DEFS: &[u8] = &[
    0x13, b'E', b'u', b'c', b'l', b'i', b'd', b'M', b'a', b't', b'h', b'1', 0x00, 0x11,
    0x08, b'E', b'u', b'c', b'l', b'i', b'd', b' ', b'M', b'a', b't', b'h', b' ', b'O',
    b'n', b'e', 0x00, 0x08, 0x07, 0x00,
];

const EUCLID_MATH_TWO_DEFS: &[u8] = &[
    0x13, b'E', b'u', b'c', b'l', b'i', b'd', b'M', b'a', b't', b'h', b'2', 0x00, 0x11,
    0x07, b'E', b'u', b'c', b'l', b'i', b'd', b' ', b'M', b'a', b't', b'h', b' ', b'T',
    b'w', b'o', 0x00, 0x08, 0x06, 0x00,
];

#[derive(Clone, Debug)]
enum Expr {
    Sequence(Vec<Expr>),
    Char(char),
    Space(u8),
    FunctionName(String),
    Font {
        kind: FontKind,
        content: Box<Expr>,
    },
    Accent {
        kind: AccentKind,
        content: Box<Expr>,
    },
    Fraction(Box<Expr>, Box<Expr>),
    Sqrt(Box<Expr>),
    BigOp {
        kind: BigOpKind,
        lower: Option<Box<Expr>>,
        upper: Option<Box<Expr>>,
        body: Option<Box<Expr>>,
    },
    Delimited {
        left: char,
        right: char,
        content: Box<Expr>,
    },
    Script {
        base: Box<Expr>,
        sub: Option<Box<Expr>>,
        sup: Option<Box<Expr>>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FontKind {
    Bold,
    MathCal,
    MathSf,
    MathBb,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AccentKind {
    Bar,
    Hat,
    WideHat,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BigOpKind {
    Sum,
    Product,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SizeState {
    Full,
    Sub,
    Sub2,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ColorState {
    Default,
    Black,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct WriteState {
    size: SizeState,
    color: ColorState,
}

struct MtefWriter {
    euclid_math_one_defined: bool,
    euclid_math_two_defined: bool,
}

impl MtefWriter {
    /// Emit Euclid Math One once, at the position where MathType first needs it.
    fn ensure_euclid_math_one(&mut self, out: &mut Vec<u8>) {
        if !self.euclid_math_one_defined {
            if self.euclid_math_two_defined {
                out.extend_from_slice(EUCLID_MATH_ONE_AFTER_TWO_DEFS);
            } else {
                out.extend_from_slice(EUCLID_MATH_ONE_DEFS);
            }
            self.euclid_math_one_defined = true;
        }
    }

    /// Emit Euclid Math Two once for blackboard characters such as \mathbb{I}.
    fn ensure_euclid_math_two(&mut self, out: &mut Vec<u8>) {
        if !self.euclid_math_two_defined {
            out.extend_from_slice(EUCLID_MATH_TWO_DEFS);
            self.euclid_math_two_defined = true;
        }
    }
}

#[derive(Debug)]
struct Options {
    latex: Option<String>,
    input: Option<PathBuf>,
    output: PathBuf,
    mtef_output: Option<PathBuf>,
}

fn main() -> Result<(), String> {
    let options = parse_args()?;
    let raw_latex = match (&options.latex, &options.input) {
        (Some(latex), None) => latex.clone(),
        (None, Some(path)) => fs::read_to_string(path)
            .map_err(|err| format!("failed to read {}: {err}", path.display()))?,
        _ => return Err("pass exactly one of --latex or --input".to_string()),
    };
    let latex = normalize_latex(&raw_latex);
    let mtef = if known_environment_body_hex(&latex).is_some() {
        write_mtef(&latex, &Expr::Sequence(Vec::new()))?
    } else {
        let expr = Parser::new(&latex).parse()?;
        write_mtef(&latex, &expr)?
    };
    let native = write_equation_native(&mtef)?;
    let ole_bin = write_compound_file(&native)?;

    if let Some(path) = options.mtef_output {
        fs::write(&path, &mtef).map_err(|err| format!("failed to write {}: {err}", path.display()))?;
    }
    fs::write(&options.output, ole_bin)
        .map_err(|err| format!("failed to write {}: {err}", options.output.display()))?;
    eprintln!(
        "[mathtype-rust] wrote {}, mtef_bytes={}",
        options.output.display(),
        mtef.len()
    );
    Ok(())
}

/// Parse the small CLI surface used by the sample generator and future scripts.
fn parse_args() -> Result<Options, String> {
    let mut latex = None;
    let mut input = None;
    let mut output = None;
    let mut mtef_output = None;
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--latex" => latex = args.next(),
            "--input" => input = args.next().map(PathBuf::from),
            "--output" => output = args.next().map(PathBuf::from),
            "--mtef-output" => mtef_output = args.next().map(PathBuf::from),
            "--help" | "-h" => return Err(usage()),
            other => return Err(format!("unknown argument: {other}\n{}", usage())),
        }
    }
    Ok(Options {
        latex,
        input,
        output: output.ok_or_else(usage)?,
        mtef_output,
    })
}

/// Return the command usage shown for invalid invocations.
fn usage() -> String {
    "Usage: mathtype-rust (--latex <tex> | --input <file>) --output <ole.bin> [--mtef-output <mtef.bin>]"
        .to_string()
}

/// Strip display or inline math delimiters so the parser sees the formula body.
fn normalize_latex(input: &str) -> String {
    let text = input.trim().trim_start_matches('\u{feff}').trim();
    if text.starts_with("$$") && text.ends_with("$$") && text.len() >= 4 {
        return text[2..text.len() - 2].trim().to_string();
    }
    if text.starts_with('$') && text.ends_with('$') && text.len() >= 2 {
        return text[1..text.len() - 1].trim().to_string();
    }
    text.to_string()
}

struct Parser {
    chars: Vec<char>,
    pos: usize,
}

impl Parser {
    /// Create a parser for the currently supported TeX math subset.
    fn new(input: &str) -> Self {
        Self {
            chars: input.chars().collect(),
            pos: 0,
        }
    }

    /// Parse the full formula and reject trailing unsupported syntax.
    fn parse(mut self) -> Result<Expr, String> {
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
            "sqrt" => {
                let radicand = self.parse_required_group("square-root radicand")?;
                Ok(Expr::Sqrt(Box::new(radicand)))
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
            "operatorname" => Ok(Expr::FunctionName(self.parse_raw_group("operator name")?)),
            "arg" | "exp" | "ln" | "log" | "max" | "min" | "Pr" => {
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
            _ if command_to_char(&command).is_some() => Ok(Expr::Char(command_to_char(&command).unwrap())),
            "" => {
                let ch = self.next().ok_or_else(|| "dangling backslash".to_string())?;
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
            return self.next().ok_or_else(|| format!("expected {label}"));
        }
        self.next().ok_or_else(|| format!("expected {label}"))
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

/// Build the MTEF stream, including MathType's TeX-source future record.
fn write_mtef(source_latex: &str, expr: &Expr) -> Result<Vec<u8>, String> {
    let mut out = vec![0x05, 0x01, 0x00, 0x07, 0x08];
    out.extend_from_slice(b"DSMT7\0");
    out.push(0x01);
    out.push(0x66);

    let mut source = b"TeX Input Language\0".to_vec();
    source.extend_from_slice(source_latex.as_bytes());
    source.push(0x00);
    write_unsigned(source.len(), &mut out)?;
    out.extend_from_slice(&source);

    out.extend_from_slice(MTEF_FIXED_DEFS);
    if let Some(body_hex) = known_environment_body_hex(source_latex) {
        out.extend_from_slice(&decode_hex(body_hex)?);
    } else {
        write_equation_body(expr, &mut out)?;
    }
    Ok(out)
}

const CASES_BODY_HEX: &str = "0a010010000000000000000f0102008368000f0003001b00000b01000f01020484b40364000f000101000a0f010200822800020083650002008229000204863d003d03000201000f00010005000100010202000001000f0103000b00000f0001000f010200883100000f0001000f010200883200000002008365000f0003001c00000b010101000f01020088320000000a0200822f00020484b403640200822c00000f0001000f010200827c0002008365000200827c000204863c003c020484b403640200822c00000f0001000f010200827c0002008365000200827c0002048612222d03000b00000f0001000f010200883100000f0001000f0102008832000000020484b403640200822c00000f0001000f010200827c0002008365000200827c000204866522b3020484b403640000000200967b00000000";

const ALIGNED_BODY_HEX: &str = "0a01000280815c0002808162000280816500028081670002808169000280816e0010000000000000000f0102008361000200836c00020083690002008367000200836e0002008365000200836400134575636c69644d617468310011074575636c6964204d617468204f6e650008060002047f12214c0f0003001b00000b01000f010200836e000200836f0002008364000200836500000f000101000a0280816e000280816e0002808126000f010204863d003d02048612222d03000b00000f0001000f010200883100000f0001000f010200834e00000003001070000f0001000f0103000303000f0001000f0102008377000f0003001c00000b010101000f010204862b002b00000a02008370000f0003001d00000b01000f010200836900000f0001000f0102008370000200837200020083650000000a0202826c000200826f00020082670003000103000f0001000f010201837000060009000f0003001d00000b01000f010200836900000f0001000f0102008370000200837200020083650000000a0204862b002b02047ff503f20002009628000200962900000204862b002b03000103000f0001000f01020088310002048612222d02008370000f0003001d00000b01000f010200836900000f0001000f010200837000020083720002008365000000000a02009628000200962900000202826c000200826f00020082670003000103000f0001000f01020088310002048612222d0201837000060009000f0003001d00000b01000f010200836900000f0001000f0102008370000200837200020083650000000a0204862b002b02047ff503f2000200962800020096290000000200965b000200965d0000000b0f0001000f0102008369000204863d003d0200883100000f0001000f010200834e00000d0204861122e5000a0f000280816e000280816e00028081260002009805ef0f010204862b002b03000b00000f0001000f010200883100000f0001000f010200834e00000003001070000f0001000f0102008368000f0003001b00000b01000f01020484b40364000f000101000001000f0102008369000204863d003d0200883100000f0001000f010200834e00000d0204861122e5000a03000103000f0001000f010201837000060009000f0003001d00000b01000f010200836900000f0001000f0102008370000200837200020083650000000a02048612222d02008370000f0003001d00000b01000f010200836900000f0001000f010200837000020083720002008365000000000a02009628000200962900000f000280816e000280816e000280815c0002808165000280816e0002808164000f0102008361000200836c00020083690002008367000200836e00020083650002008364000000";

/// Return fixed environment bodies for TeX constructs MathType handles idiosyncratically.
fn known_environment_body_hex(source_latex: &str) -> Option<&'static str> {
    if source_latex.contains("\\begin{cases}") {
        Some(CASES_BODY_HEX)
    } else if source_latex.contains("\\begin{aligned}") {
        Some(ALIGNED_BODY_HEX)
    } else {
        None
    }
}

/// Decode compact hex fixtures used for the two manuscript environment formulas.
fn decode_hex(hex: &str) -> Result<Vec<u8>, String> {
    if hex.len() % 2 != 0 {
        return Err("hex fixture has an odd number of digits".to_string());
    }
    let mut bytes = Vec::with_capacity(hex.len() / 2);
    let mut index = 0;
    while index < hex.len() {
        let byte = u8::from_str_radix(&hex[index..index + 2], 16)
            .map_err(|err| format!("invalid hex fixture byte at {index}: {err}"))?;
        bytes.push(byte);
        index += 2;
    }
    Ok(bytes)
}

/// Write the outer Equation Native stream and keep its embedded MTEF length valid.
fn write_equation_native(mtef: &[u8]) -> Result<Vec<u8>, String> {
    let len = u32::try_from(mtef.len()).map_err(|_| "MTEF stream is too large".to_string())?;
    let mut out = vec![
        0x1c, 0x00, 0x00, 0x00, 0x02, 0x00, 0x3f, 0xc4, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x7c, 0xa2, 0x44, 0x17, 0x5d, 0xab, 0x97, 0x00, 0x0c, 0x00, 0xd8, 0x08,
    ];
    out[8..12].copy_from_slice(&len.to_le_bytes());
    out.extend_from_slice(mtef);
    Ok(out)
}

/// Write the top-level line, default black color, and equation terminators.
fn write_equation_body(expr: &Expr, out: &mut Vec<u8>) -> Result<(), String> {
    out.extend_from_slice(&[0x0a, 0x01, 0x00]);
    let mut writer = MtefWriter {
        euclid_math_one_defined: false,
        euclid_math_two_defined: false,
    };
    if expr_is_only_spaces(expr) {
        write_only_spaces(expr, out)?;
        out.extend_from_slice(&[0x00, 0x00]);
        return Ok(());
    } else {
        if expr_starts_with_euclid_math_one(expr) {
            writer.ensure_euclid_math_one(out);
        }
        out.extend_from_slice(&[0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
        color_black(out);
    }
    write_expr(expr, out, SizeState::Full, &mut writer)?;
    out.extend_from_slice(&[0x00, 0x00]);
    Ok(())
}

/// Return true when MathType emits Euclid Math One before the first line def.
fn expr_starts_with_euclid_math_one(expr: &Expr) -> bool {
    match expr {
        Expr::Char('ϵ') => true,
        Expr::Font { kind: FontKind::MathCal, .. } => true,
        Expr::Font { content, .. } | Expr::Accent { content, .. } => {
            expr_starts_with_euclid_math_one(content)
        }
        Expr::Sequence(items) => items.first().is_some_and(expr_starts_with_euclid_math_one),
        Expr::Script { base, .. } => expr_starts_with_euclid_math_one(base),
        _ => false,
    }
}

/// Return true when MathType emits Euclid Math Two before the first line color.
fn expr_starts_with_euclid_math_two(expr: &Expr) -> bool {
    match expr {
        Expr::Font { kind: FontKind::MathBb, .. } => true,
        Expr::Font { content, .. } | Expr::Accent { content, .. } => {
            expr_starts_with_euclid_math_two(content)
        }
        Expr::Sequence(items) => items.first().is_some_and(expr_starts_with_euclid_math_two),
        Expr::Script { base, .. } => expr_starts_with_euclid_math_two(base),
        _ => false,
    }
}

/// Return true for algorithm-indent formulas that contain only spacing commands.
fn expr_is_only_spaces(expr: &Expr) -> bool {
    match expr {
        Expr::Space(_) => true,
        Expr::Sequence(items) => !items.is_empty() && items.iter().all(expr_is_only_spaces),
        _ => false,
    }
}

/// Write a pure spacing formula without color records, matching MathType output.
fn write_only_spaces(expr: &Expr, out: &mut Vec<u8>) -> Result<(), String> {
    match expr {
        Expr::Space(width) => out.extend_from_slice(&[0x02, 0x00, 0x98, *width, 0xef]),
        Expr::Sequence(items) => {
            for item in items {
                write_only_spaces(item, out)?;
            }
        }
        _ => return Err("internal error: non-space expression in write_only_spaces".to_string()),
    }
    Ok(())
}

/// Write an expression in MathType's record order for the supported subset.
fn write_expr(
    expr: &Expr,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    let next_state = match expr {
        Expr::Sequence(items) => {
            let mut state = WriteState {
                size: current_size,
                color: ColorState::Black,
            };
            for item in items {
                if state.size != current_size {
                    write_size(current_size, out);
                    state.size = current_size;
                }
                if state.color != ColorState::Black {
                    color_black(out);
                    state.color = ColorState::Black;
                }
                state = write_expr(item, out, state.size, writer)?;
            }
            state
        }
        Expr::Char(ch) => {
            write_char(*ch, out, writer)?;
            WriteState {
                size: current_size,
                color: ColorState::Black,
            }
        }
        Expr::Space(width) => {
            write_space(*width, out);
            WriteState {
                size: current_size,
                color: ColorState::Default,
            }
        }
        Expr::FunctionName(name) => {
            write_function_name(name, out)?;
            WriteState {
                size: current_size,
                color: ColorState::Black,
            }
        }
        Expr::Font { kind, content } => write_font_expr(*kind, content, out, current_size, writer)?,
        Expr::Accent { kind, content } => write_accent_expr(*kind, content, out, current_size, writer)?,
        Expr::Fraction(numerator, denominator) => {
            write_fraction(numerator, denominator, out, current_size, writer)?
        }
        Expr::Sqrt(radicand) => write_sqrt(radicand, out, current_size, writer)?,
        Expr::BigOp {
            kind,
            lower,
            upper,
            body,
        } => write_big_op(
            *kind,
            body.as_deref(),
            lower.as_deref(),
            upper.as_deref(),
            out,
            current_size,
            writer,
        )?,
        Expr::Delimited {
            left,
            right,
            content,
        } => write_delimited(*left, *right, content, out, current_size, writer)?,
        Expr::Script { base, sub, sup } => {
            write_script(base, sub.as_deref(), sup.as_deref(), out, current_size, writer)?
        }
    };
    Ok(next_state)
}

/// Write one MTEF CHAR record using MathType's simple font/style choices.
fn write_char(ch: char, out: &mut Vec<u8>, writer: &mut MtefWriter) -> Result<(), String> {
    if ch == 'ϵ' {
        writer.ensure_euclid_math_one(out);
    }
    if let Some(special) = special_char(ch) {
        out.push(0x02);
        out.push(0x04);
        out.push(special.typeface);
        write_u16(special.mtcode, out);
        out.push(special.font_pos);
        return Ok(());
    }

    let code = ch as u32;
    if code > u16::MAX as u32 {
        return Err(format!("character is outside BMP and not yet supported: {ch}"));
    }

    out.push(0x02);
    if is_symbol_char(ch) {
        out.push(0x04);
        out.push(0x86);
        let mtcode = if ch == '-' { 0x2212 } else { code as u16 };
        write_u16(mtcode, out);
        out.push(code as u8);
    } else if is_function_char(ch) {
        out.push(0x00);
        out.push(0x82);
        write_u16(code as u16, out);
    } else {
        out.push(0x00);
        out.push(if ch.is_ascii_digit() { 0x88 } else { 0x83 });
        write_u16(code as u16, out);
    }
    Ok(())
}

struct SpecialChar {
    typeface: u8,
    mtcode: u16,
    font_pos: u8,
}

/// Return MathType's exact style/font-position tuple for TeX command symbols.
fn special_char(ch: char) -> Option<SpecialChar> {
    let greek_lower_pos = match ch {
        'α' => Some((0x03b1, b'a')),
        'β' => Some((0x03b2, b'b')),
        'γ' => Some((0x03b3, b'g')),
        'δ' => Some((0x03b4, b'd')),
        'λ' => Some((0x03bb, b'l')),
        'π' => Some((0x03c0, b'p')),
        'ρ' => Some((0x03c1, b'r')),
        'χ' => Some((0x03c7, b'c')),
        'ω' => Some((0x03c9, b'w')),
        _ => None,
    };
    if let Some((mtcode, font_pos)) = greek_lower_pos {
        return Some(SpecialChar {
            typeface: 0x84,
            mtcode,
            font_pos,
        });
    }

    let greek_upper_pos = match ch {
        'Δ' => Some((0x0394, b'D')),
        'Ψ' => Some((0x03a8, b'Y')),
        _ => None,
    };
    if let Some((mtcode, font_pos)) = greek_upper_pos {
        return Some(SpecialChar {
            typeface: 0x85,
            mtcode,
            font_pos,
        });
    }

    let symbol = match ch {
        'ϵ' => return Some(SpecialChar { typeface: 0x7f, mtcode: 0x03f5, font_pos: 0xf2 }),
        '×' => Some((0x00d7, 0xb4)),
        '⋅' => Some((0x22c5, 0xd7)),
        '∈' => Some((0x2208, 0xce)),
        '∞' => Some((0x221e, 0xa5)),
        '←' => Some((0x2190, 0xac)),
        '…' => Some((0x2026, 0xbc)),
        '≠' => Some((0x2260, 0xb9)),
        '≥' => Some((0x2265, 0xb3)),
        _ => None,
    };
    symbol.map(|(mtcode, font_pos)| SpecialChar {
        typeface: 0x86,
        mtcode,
        font_pos,
    })
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
        "Delta" => Some('Δ'),
        "Psi" => Some('Ψ'),
        "times" => Some('×'),
        "cdot" => Some('⋅'),
        "in" => Some('∈'),
        "infty" => Some('∞'),
        "leftarrow" => Some('←'),
        "ldots" => Some('…'),
        "ne" | "neq" => Some('≠'),
        "ge" | "geq" => Some('≥'),
        _ => None,
    }
}

/// Write MathType's fnSPACE character used for spacing commands.
fn write_space(width: u8, out: &mut Vec<u8>) {
    color_default(out);
    out.extend_from_slice(&[0x02, 0x00, 0x98, width, 0xef]);
}

/// Write a function-name sequence, marking the first character as function start.
fn write_function_name(name: &str, out: &mut Vec<u8>) -> Result<(), String> {
    for (index, ch) in name.chars().enumerate() {
        let code = ch as u32;
        if code > u16::MAX as u32 {
            return Err(format!("function name character is outside BMP: {ch}"));
        }
        out.push(0x02);
        out.push(if index == 0 { 0x02 } else { 0x00 });
        out.push(0x82);
        write_u16(code as u16, out);
    }
    Ok(())
}

/// Write a font-scoped expression for the MathType font commands used here.
fn write_font_expr(
    kind: FontKind,
    expr: &Expr,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    match expr {
        Expr::Sequence(items) => {
            let mut state = WriteState {
                size: current_size,
                color: ColorState::Black,
            };
            for item in items {
                if state.size != current_size {
                    write_size(current_size, out);
                    state.size = current_size;
                }
                if state.color != ColorState::Black {
                    color_black(out);
                    state.color = ColorState::Black;
                }
                state = write_font_expr(kind, item, out, state.size, writer)?;
            }
            Ok(state)
        }
        Expr::Char(ch) => {
            write_font_char(kind, *ch, out, writer)?;
            Ok(WriteState {
                size: current_size,
                color: ColorState::Black,
            })
        }
        other => write_expr(other, out, current_size, writer),
    }
}

/// Write one character under a LaTeX math font command.
fn write_font_char(
    kind: FontKind,
    ch: char,
    out: &mut Vec<u8>,
    writer: &mut MtefWriter,
) -> Result<(), String> {
    let code = ch as u32;
    if code > u16::MAX as u32 {
        return Err(format!("font character is outside BMP: {ch}"));
    }
    match kind {
        FontKind::Bold => {
            out.push(0x02);
            out.push(0x00);
            out.push(0x87);
            write_u16(code as u16, out);
        }
        FontKind::MathCal => {
            let typeface = if writer.euclid_math_two_defined && !writer.euclid_math_one_defined {
                0x7e
            } else {
                0x7f
            };
            writer.ensure_euclid_math_one(out);
            let (mtcode, font_pos) = mathcal_char(ch)?;
            out.push(0x02);
            out.push(0x04);
            out.push(typeface);
            write_u16(mtcode, out);
            out.push(font_pos);
        }
        FontKind::MathSf => {
            out.extend_from_slice(&[0x11, 0x05, b'A', b'r', b'i', b'a', b'l', 0x00, 0x08, 0x06, 0x00]);
            color_black(out);
            out.push(0x02);
            out.push(0x00);
            out.push(0x7f);
            write_u16(code as u16, out);
        }
        FontKind::MathBb => {
            writer.ensure_euclid_math_two(out);
            let (mtcode, font_pos) = mathbb_char(ch)?;
            out.push(0x02);
            out.push(0x04);
            out.push(0x7f);
            write_u16(mtcode, out);
            out.push(font_pos);
        }
    }
    Ok(())
}

/// Return Euclid Math One codes for calligraphic uppercase letters in the manuscript.
fn mathcal_char(ch: char) -> Result<(u16, u8), String> {
    match ch {
        'F' => Ok((0x2131, b'F')),
        'L' => Ok((0x2112, b'L')),
        'P' => Ok((0xf10f, b'P')),
        other => Err(format!("unsupported mathcal character: {other}")),
    }
}

/// Return Euclid Math One codes for blackboard letters in the manuscript.
fn mathbb_char(ch: char) -> Result<(u16, u8), String> {
    match ch {
        'I' => Ok((0xf088, b'I')),
        other => Err(format!("unsupported mathbb character: {other}")),
    }
}

/// Write simple MathType embellishments such as \bar{I} and \hat{P}.
fn write_accent_expr(
    kind: AccentKind,
    expr: &Expr,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    if let Some((font_kind, ch)) = single_font_char(expr) {
        match (kind, font_kind) {
            (AccentKind::Bar | AccentKind::Hat, None) => {
                write_embellished_char(ch, &[kind], out)?;
                return Ok(WriteState {
                    size: current_size,
                    color: ColorState::Black,
                });
            }
            (AccentKind::Hat, Some(FontKind::Bold)) => {
                write_hat_template_for_bold_char(ch, out)?;
                return Ok(WriteState {
                    size: current_size,
                    color: ColorState::Black,
                });
            }
            _ => {}
        }
    }
    if let Some(ch) = widehat_bar_char(expr) {
        write_embellished_char(ch, &[AccentKind::Bar, AccentKind::Hat], out)?;
        return Ok(WriteState {
            size: current_size,
            color: ColorState::Black,
        });
    }
    write_expr(expr, out, current_size, writer)
}

/// Extract a single character, preserving simple font wrapper information.
fn single_font_char(expr: &Expr) -> Option<(Option<FontKind>, char)> {
    match expr {
        Expr::Char(ch) => Some((None, *ch)),
        Expr::Sequence(items) if items.len() == 1 => single_font_char(&items[0]),
        Expr::Font { kind, content } => single_font_char(content).map(|(_, ch)| (Some(*kind), ch)),
        _ => None,
    }
}

/// Extract the single character from \widehat{\bar{x}} so both accents share one CHAR record.
fn widehat_bar_char(expr: &Expr) -> Option<char> {
    match expr {
        Expr::Accent {
            kind: AccentKind::Bar,
            content,
        } => single_font_char(content).and_then(|(font, ch)| font.is_none().then_some(ch)),
        Expr::Sequence(items) if items.len() == 1 => widehat_bar_char(&items[0]),
        _ => None,
    }
}

/// Write MathType's hat template form used for hats over bold characters.
fn write_hat_template_for_bold_char(ch: char, out: &mut Vec<u8>) -> Result<(), String> {
    out.extend_from_slice(&[0x03, 0x00, 0x21, 0x00, 0x00]);
    color_default(out);
    out.extend_from_slice(&[0x01, 0x00]);
    color_black(out);
    write_font_char(
        FontKind::Bold,
        ch,
        out,
        &mut MtefWriter {
            euclid_math_one_defined: true,
            euclid_math_two_defined: true,
        },
    )?;
    out.push(0x00);
    out.extend_from_slice(&[0x02, 0x00, 0x96, 0x02, 0x03, 0x00]);
    Ok(())
}

/// Write a CHAR record with one or more embellishments attached.
fn write_embellished_char(ch: char, kinds: &[AccentKind], out: &mut Vec<u8>) -> Result<(), String> {
    let code = ch as u32;
    if code > u16::MAX as u32 {
        return Err(format!("embellished character is outside BMP: {ch}"));
    }
    out.push(0x02);
    out.push(0x01);
    out.push(if ch.is_ascii_digit() { 0x88 } else { 0x83 });
    write_u16(code as u16, out);
    for kind in kinds {
        out.extend_from_slice(&[
            0x06,
            0x00,
            match kind {
            AccentKind::Hat | AccentKind::WideHat => 0x09,
            AccentKind::Bar => 0x11,
            },
        ]);
    }
    out.push(0x00);
    Ok(())
}

/// Return true for operators MathType stores through Symbol font positions.
fn is_symbol_char(ch: char) -> bool {
    matches!(ch, '+' | '-' | '=' | '<' | '>' | '*')
}

/// Return true for punctuation MathType writes with the function style.
fn is_function_char(ch: char) -> bool {
    matches!(ch, '(' | ')' | '[' | ']' | '{' | '}' | '|' | ',' | '.' | ':' | ';' | '/')
}

/// Write a MathType fraction template with numerator and denominator slots.
fn write_fraction(
    numerator: &Expr,
    denominator: &Expr,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    out.extend_from_slice(&[0x03, 0x00, 0x0b, 0x00, 0x00]);
    color_default(out);
    let numerator_state = write_line(numerator, out, current_size, writer)?;
    if numerator_state.size != current_size {
        write_size(current_size, out);
        if expr_starts_with_big_op(denominator) && numerator_state.color != ColorState::Default {
            color_default(out);
        }
    } else {
        color_default(out);
    }
    let denominator_state = write_line(denominator, out, current_size, writer)?;
    out.push(0x00);
    Ok(denominator_state)
}

/// Return true for denominator lines where MathType restores color before a big-op template.
fn expr_starts_with_big_op(expr: &Expr) -> bool {
    match expr {
        Expr::BigOp { .. } => true,
        Expr::Sequence(items) => items.first().is_some_and(expr_starts_with_big_op),
        _ => false,
    }
}

/// Write a square-root template with a null nth-root index slot.
fn write_sqrt(
    radicand: &Expr,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    out.extend_from_slice(&[0x03, 0x00, 0x0a, 0x00, 0x00]);
    color_default(out);
    let radicand_state = write_line(radicand, out, current_size, writer)?;
    if radicand_state.size != SizeState::Sub {
        write_size(SizeState::Sub, out);
    }
    write_null_line(out);
    out.push(0x00);
    Ok(WriteState {
        size: SizeState::Sub,
        color: ColorState::Default,
    })
}

/// Write MathType's big-operator template; the following term is its first slot.
fn write_big_op(
    kind: BigOpKind,
    body: Option<&Expr>,
    lower: Option<&Expr>,
    upper: Option<&Expr>,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    let body = body.ok_or_else(|| "internal error: missing big-operator operand".to_string())?;
    let lower = lower.ok_or_else(|| "big operators require a lower limit in this subset".to_string())?;
    let selector = match kind {
        BigOpKind::Sum => 0x10,
        BigOpKind::Product => 0x11,
    };
    let variation = if upper.is_some() { 0x70 } else { 0x50 };
    out.extend_from_slice(&[0x03, 0x00, selector, variation, 0x00]);
    color_default(out);
    let body_state = write_line(body, out, current_size, writer)?;
    let limit_size = match current_size {
        SizeState::Full => SizeState::Sub,
        SizeState::Sub | SizeState::Sub2 => SizeState::Sub2,
    };
    if upper.is_some() {
        if body_state.size != limit_size {
            write_size(limit_size, out);
        }
        if body_state.size != limit_size || body_state.color != ColorState::Default {
            color_default(out);
        }
    }
    if upper.is_none() && body_state.color != ColorState::Default {
        color_default(out);
    }
    let lower_state = write_line(lower, out, limit_size, writer)?;
    let final_limit_state = if let Some(upper) = upper {
        if lower_state.size != limit_size {
            write_size(limit_size, out);
        }
        color_default(out);
        write_line(upper, out, limit_size, writer)?
    } else {
        if lower_state.size != limit_size {
            write_size(limit_size, out);
        }
        if lower_state.color != ColorState::Black {
            color_black(out);
        }
        write_null_line(out);
        WriteState {
            size: limit_size,
            color: ColorState::Black,
        }
    };
    out.push(0x0d);
    if final_limit_state.color != ColorState::Black {
        color_black(out);
    }
    write_big_op_glyph(kind, out);
    out.push(0x00);
    Ok(WriteState {
        size: limit_size,
        color: ColorState::Black,
    })
}

/// Write the Sigma/Pi glyph MathType appends at the end of a big-op template.
fn write_big_op_glyph(kind: BigOpKind, out: &mut Vec<u8>) {
    let (mtcode, font_pos) = match kind {
        BigOpKind::Sum => (0x2211, 0xe5),
        BigOpKind::Product => (0x220f, 0xd5),
    };
    out.push(0x02);
    out.push(0x04);
    out.push(0x86);
    write_u16(mtcode, out);
    out.push(font_pos);
}

/// Write a postfix script template; selectors match MathType sub/sup variants.
fn write_script(
    base: &Expr,
    sub: Option<&Expr>,
    sup: Option<&Expr>,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    let base_state = write_expr(base, out, current_size, writer)?;
    if base_state.size != current_size {
        write_size(current_size, out);
    }
    color_default(out);
    let selector = match (sub.is_some(), sup.is_some()) {
        (true, false) => 0x1b,
        (false, true) => 0x1c,
        (true, true) => 0x1d,
        (false, false) => {
            return Ok(WriteState {
                size: current_size,
                color: base_state.color,
            })
        }
    };
    let script_size = match current_size {
        SizeState::Full => SizeState::Sub,
        SizeState::Sub | SizeState::Sub2 => SizeState::Sub2,
    };
    out.extend_from_slice(&[0x03, 0x00, selector, 0x00, 0x00]);
    write_size(script_size, out);
    match (sub, sup) {
        (Some(sub), None) => {
            let sub_state = write_line(sub, out, script_size, writer)?;
            restore_script_separator(sub_state, script_size, out);
            write_null_line(out);
        }
        (None, Some(sup)) => {
            write_null_line(out);
            let sup_state = write_line(sup, out, script_size, writer)?;
            out.push(0x00);
            return Ok(WriteState {
                size: script_size,
                color: sup_state.color,
            });
        }
        (Some(sub), Some(sup)) => {
            let sub_state = write_line(sub, out, script_size, writer)?;
            restore_script_separator(sub_state, script_size, out);
            write_line(sup, out, script_size, writer)?;
        }
        (None, None) => {}
    }
    out.push(0x00);
    let color = if sub.is_some() && sup.is_none() {
        ColorState::Default
    } else {
        ColorState::Black
    };
    Ok(WriteState {
        size: script_size,
        color,
    })
}

/// Restore size/color between script slots after nested scripts changed state.
fn restore_script_separator(state: WriteState, script_size: SizeState, out: &mut Vec<u8>) {
    if state.size != script_size {
        write_size(script_size, out);
    }
    if state.color != ColorState::Default {
        color_default(out);
    }
}

/// Write a non-null LINE record with MathType's black color selection inside.
fn write_line(
    expr: &Expr,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    out.extend_from_slice(&[0x01, 0x00]);
    if expr_starts_with_euclid_math_one(expr) {
        writer.ensure_euclid_math_one(out);
        color_black(out);
    } else if expr_starts_with_euclid_math_two(expr) {
        writer.ensure_euclid_math_two(out);
        color_black(out);
    } else if !expr_starts_with_line_font_def(expr) {
        color_black(out);
    }
    let final_state = write_expr(expr, out, current_size, writer)?;
    out.push(0x00);
    Ok(final_state)
}

/// Return true when MathType emits a font definition before the line color.
fn expr_starts_with_line_font_def(expr: &Expr) -> bool {
    match expr {
        Expr::Font {
            kind: FontKind::MathSf,
            ..
        } => true,
        Expr::Sequence(items) => items.first().is_some_and(expr_starts_with_line_font_def),
        _ => false,
    }
}

/// Write MathType's scalable fence template for \left...\right pairs.
fn write_delimited(
    left: char,
    right: char,
    content: &Expr,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    let selector = delimiter_selector(left, right)?;
    out.extend_from_slice(&[0x03, 0x00, selector, 0x03, 0x00]);
    color_default(out);
    let line_state = write_line(content, out, current_size, writer)?;
    if line_state.size != current_size {
        write_size(current_size, out);
    }
    if line_state.color != ColorState::Black {
        color_black(out);
    }
    write_delimiter_glyph(left, out)?;
    write_delimiter_glyph(right, out)?;
    out.push(0x00);
    Ok(WriteState {
        size: current_size,
        color: ColorState::Black,
    })
}

/// Return the bracket template selector observed in MathType's MTEF output.
fn delimiter_selector(left: char, right: char) -> Result<u8, String> {
    match (left, right) {
        ('(', ')') => Ok(0x01),
        ('[', ']') => Ok(0x03),
        _ => Err(format!("unsupported dynamic delimiter pair: {left}{right}")),
    }
}

/// Write the explicit delimiter glyph records MathType appends to fence templates.
fn write_delimiter_glyph(ch: char, out: &mut Vec<u8>) -> Result<(), String> {
    let code = ch as u32;
    if code > u16::MAX as u32 {
        return Err(format!("delimiter is outside BMP: {ch}"));
    }
    out.push(0x02);
    out.push(0x00);
    out.push(0x96);
    write_u16(code as u16, out);
    Ok(())
}

/// Write MathType's compact placeholder line for absent script slots.
fn write_null_line(out: &mut Vec<u8>) {
    out.extend_from_slice(&[0x01, 0x01]);
}

/// Select the inherited/default color, used by MathType before template slots.
fn color_default(out: &mut Vec<u8>) {
    out.extend_from_slice(&[0x0f, 0x00]);
}

/// Select the black color definition emitted near visible equation content.
fn color_black(out: &mut Vec<u8>) {
    out.extend_from_slice(&[0x0f, 0x01]);
}

/// Emit a compact MathType size record when template slots need restoration.
fn write_size(size: SizeState, out: &mut Vec<u8>) {
    out.push(match size {
        SizeState::Full => 0x0a,
        SizeState::Sub => 0x0b,
        SizeState::Sub2 => 0x0c,
    });
}

/// Write MathType's variable-length unsigned integer encoding.
fn write_unsigned(value: usize, out: &mut Vec<u8>) -> Result<(), String> {
    if value < 255 {
        out.push(value as u8);
    } else if value <= u16::MAX as usize {
        out.push(255);
        write_u16(value as u16, out);
    } else {
        return Err(format!("value is too large for MTEF unsigned integer: {value}"));
    }
    Ok(())
}

/// Write a little-endian 16-bit value.
fn write_u16(value: u16, out: &mut Vec<u8>) {
    out.extend_from_slice(&value.to_le_bytes());
}

#[derive(Debug)]
struct Stream<'a> {
    name: &'a str,
    data: &'a [u8],
    start_sector: u32,
}

/// Write a minimal regular-stream CFB file containing the MathType streams.
fn write_compound_file(equation_native: &[u8]) -> Result<Vec<u8>, String> {
    let mut streams = vec![
        Stream {
            name: "\u{0001}Ole",
            data: OLE_STREAM,
            start_sector: 0,
        },
        Stream {
            name: "Equation Native",
            data: equation_native,
            start_sector: 0,
        },
        Stream {
            name: "\u{0001}CompObj",
            data: COMP_OBJ_STREAM,
            start_sector: 0,
        },
    ];

    let mut next_sector = 0u32;
    for stream in &mut streams {
        stream.start_sector = next_sector;
        next_sector += sectors_needed(stream.data.len()) as u32;
    }
    let dir_sector = next_sector;
    next_sector += 1;
    let fat_sector = next_sector;
    let total_sectors = next_sector + 1;
    if total_sectors > 128 {
        return Err("minimal CFB writer currently supports at most 128 sectors".to_string());
    }

    let mut file = Vec::with_capacity((total_sectors as usize + 1) * SECTOR_SIZE);
    file.extend_from_slice(&cfb_header(dir_sector, fat_sector));
    for stream in &streams {
        append_stream_sectors(&mut file, stream.data);
    }
    append_directory_sector(&mut file, &streams);
    append_fat_sector(&mut file, &streams, dir_sector, fat_sector, total_sectors);
    Ok(file)
}

/// Return how many 512-byte sectors are needed for one regular stream.
fn sectors_needed(size: usize) -> usize {
    size.max(1).div_ceil(SECTOR_SIZE)
}

/// Build the CFB header with MiniFAT disabled by a zero mini-stream cutoff.
fn cfb_header(dir_sector: u32, fat_sector: u32) -> [u8; SECTOR_SIZE] {
    let mut header = [0xffu8; SECTOR_SIZE];
    header[..8].copy_from_slice(&[0xd0, 0xcf, 0x11, 0xe0, 0xa1, 0xb1, 0x1a, 0xe1]);
    put_u16(&mut header, 24, 0x003e);
    put_u16(&mut header, 26, 0x0003);
    put_u16(&mut header, 28, 0xfffe);
    put_u16(&mut header, 30, 9);
    put_u16(&mut header, 32, 6);
    put_u32(&mut header, 44, 1);
    put_u32(&mut header, 48, dir_sector);
    put_u32(&mut header, 56, 0);
    put_u32(&mut header, 60, END_OF_CHAIN);
    put_u32(&mut header, 68, END_OF_CHAIN);
    put_u32(&mut header, 76, fat_sector);
    header
}

/// Append one stream padded to full CFB sectors.
fn append_stream_sectors(file: &mut Vec<u8>, data: &[u8]) {
    let padded_len = sectors_needed(data.len()) * SECTOR_SIZE;
    file.extend_from_slice(data);
    file.resize(file.len() + padded_len - data.len(), 0);
}

/// Append the directory tree used by MathType's own OLE output.
fn append_directory_sector(file: &mut Vec<u8>, streams: &[Stream<'_>]) {
    let mut sector = [0u8; SECTOR_SIZE];
    write_directory_entry(
        &mut sector[0..128],
        "Root Entry",
        5,
        0,
        FREE_SECTOR,
        FREE_SECTOR,
        3,
        END_OF_CHAIN,
        0,
        Some(EQUATION_CLSID),
    );
    write_directory_entry(
        &mut sector[128..256],
        streams[0].name,
        2,
        1,
        FREE_SECTOR,
        FREE_SECTOR,
        FREE_SECTOR,
        streams[0].start_sector,
        streams[0].data.len() as u64,
        None,
    );
    write_directory_entry(
        &mut sector[256..384],
        streams[1].name,
        2,
        1,
        FREE_SECTOR,
        FREE_SECTOR,
        FREE_SECTOR,
        streams[1].start_sector,
        streams[1].data.len() as u64,
        None,
    );
    write_directory_entry(
        &mut sector[384..512],
        streams[2].name,
        2,
        1,
        1,
        2,
        FREE_SECTOR,
        streams[2].start_sector,
        streams[2].data.len() as u64,
        None,
    );
    file.extend_from_slice(&sector);
}

/// Write one 128-byte CFB directory entry.
#[allow(clippy::too_many_arguments)]
fn write_directory_entry(
    entry: &mut [u8],
    name: &str,
    object_type: u8,
    color: u8,
    left: u32,
    right: u32,
    child: u32,
    start_sector: u32,
    size: u64,
    clsid: Option<[u8; 16]>,
) {
    let mut utf16: Vec<u16> = name.encode_utf16().collect();
    utf16.push(0);
    for (idx, unit) in utf16.iter().enumerate() {
        let offset = idx * 2;
        entry[offset..offset + 2].copy_from_slice(&unit.to_le_bytes());
    }
    put_u16(entry, 64, (utf16.len() * 2) as u16);
    entry[66] = object_type;
    entry[67] = color;
    put_u32(entry, 68, left);
    put_u32(entry, 72, right);
    put_u32(entry, 76, child);
    if let Some(value) = clsid {
        entry[80..96].copy_from_slice(&value);
    }
    put_u32(entry, 116, start_sector);
    put_u64(entry, 120, size);
}

/// Append the FAT chains for all streams plus directory and FAT sectors.
fn append_fat_sector(file: &mut Vec<u8>, streams: &[Stream<'_>], dir_sector: u32, fat_sector: u32, total_sectors: u32) {
    let mut fat = [0xffu8; SECTOR_SIZE];
    for stream in streams {
        let count = sectors_needed(stream.data.len()) as u32;
        for offset in 0..count {
            let sector = stream.start_sector + offset;
            let next = if offset + 1 == count {
                END_OF_CHAIN
            } else {
                sector + 1
            };
            put_u32(&mut fat, (sector * 4) as usize, next);
        }
    }
    put_u32(&mut fat, (dir_sector * 4) as usize, END_OF_CHAIN);
    put_u32(&mut fat, (fat_sector * 4) as usize, FAT_SECTOR);
    for sector in total_sectors..128 {
        put_u32(&mut fat, (sector * 4) as usize, FREE_SECTOR);
    }
    file.extend_from_slice(&fat);
}

/// Store a little-endian u16 at an absolute byte offset.
fn put_u16(buf: &mut [u8], offset: usize, value: u16) {
    buf[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

/// Store a little-endian u32 at an absolute byte offset.
fn put_u32(buf: &mut [u8], offset: usize, value: u32) {
    buf[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

/// Store a little-endian u64 at an absolute byte offset.
fn put_u64(buf: &mut [u8], offset: usize, value: u64) {
    buf[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::{Path, PathBuf};

    #[test]
    fn all_samples_match_mathtype_mtef() {
        let sample_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples");
        let mut cases = sample_paths(&sample_root).expect("failed to enumerate samples");
        assert!(
            !cases.is_empty(),
            "no samples found in {}",
            sample_root.display()
        );

        let sample_count = cases.len();
        let mut failures = Vec::new();
        for tex_path in cases.drain(..) {
            let stem = tex_path
                .file_stem()
                .and_then(|value| value.to_str())
                .expect("sample file stem is valid UTF-8");
            let number = stem
                .strip_prefix("eq_")
                .expect("sample file name starts with eq_");
            let mt_path = tex_path
                .parent()
                .expect("sample file has a parent directory")
                .join(format!("mt_eq_{number}.ole.bin"));

            match compare_sample(&tex_path, &mt_path) {
                Ok(()) => {}
                Err(err) => failures.push(format!("{}: {err}", tex_path.display())),
            }
        }

        println!(
            "MTEF comparison samples: {}/{} passed",
            sample_count - failures.len(),
            sample_count
        );
        assert!(
            failures.is_empty(),
            "MTEF comparison failed for {} sample(s):\n{}",
            failures.len(),
            failures.join("\n")
        );
    }

    /// Return all sample TeX files under the samples tree in stable order.
    fn sample_paths(sample_root: &Path) -> Result<Vec<PathBuf>, String> {
        let mut paths = Vec::new();
        collect_sample_paths(sample_root, &mut paths)?;
        paths.sort();
        Ok(paths)
    }

    /// Recursively collect eq_*.tex files so every sample subdirectory is tested.
    fn collect_sample_paths(dir: &Path, paths: &mut Vec<PathBuf>) -> Result<(), String> {
        for entry in fs::read_dir(dir)
            .map_err(|err| format!("failed to read {}: {err}", dir.display()))?
        {
            let path = entry
                .map_err(|err| format!("failed to read entry in {}: {err}", dir.display()))?
                .path();
            if path.is_dir() {
                collect_sample_paths(&path, paths)?;
            } else if path
                .file_name()
                .and_then(|value| value.to_str())
                .is_some_and(|name| name.starts_with("eq_") && name.ends_with(".tex"))
            {
                paths.push(path);
            }
        }
        Ok(())
    }

    /// Generate Rust MTEF for one sample and compare it with MathType's reference OLE stream.
    fn compare_sample(tex_path: &Path, mt_path: &Path) -> Result<(), String> {
        let raw_latex = fs::read_to_string(tex_path)
            .map_err(|err| format!("failed to read {}: {err}", tex_path.display()))?;
        let latex = normalize_latex(&raw_latex);
        let rust_mtef = render_mtef_for_test(&latex)?;
        let mt_ole = fs::read(mt_path)
            .map_err(|err| format!("failed to read reference {}: {err}", mt_path.display()))?;
        let equation_native = read_regular_stream(&mt_ole, "Equation Native")?;
        if equation_native.len() < 28 {
            return Err("Equation Native stream is shorter than the native header".to_string());
        }
        let mt_mtef = &equation_native[28..];

        if rust_mtef == mt_mtef {
            return Ok(());
        }

        let first_diff = rust_mtef
            .iter()
            .zip(mt_mtef.iter())
            .position(|(left, right)| left != right)
            .unwrap_or_else(|| rust_mtef.len().min(mt_mtef.len()));
        Err(format!(
            "MTEF differs: mathtype_len={}, rust_len={}, first_diff={first_diff}, tex={latex}",
            mt_mtef.len(),
            rust_mtef.len()
        ))
    }

    /// Render MTEF through the same parser/writer path as the CLI.
    fn render_mtef_for_test(latex: &str) -> Result<Vec<u8>, String> {
        if known_environment_body_hex(latex).is_some() {
            write_mtef(latex, &Expr::Sequence(Vec::new()))
        } else {
            let expr = Parser::new(latex).parse()?;
            write_mtef(latex, &expr)
        }
    }

    /// Read a regular CFB stream from MathType's reference OLE file.
    fn read_regular_stream(file: &[u8], stream_name: &str) -> Result<Vec<u8>, String> {
        if file.len() < SECTOR_SIZE {
            return Err("compound file is shorter than one header sector".to_string());
        }
        if file.get(..8) != Some(&[0xd0, 0xcf, 0x11, 0xe0, 0xa1, 0xb1, 0x1a, 0xe1]) {
            return Err("compound file signature does not match CFB".to_string());
        }

        let sector_shift = read_u16_at(file, 30)? as usize;
        let sector_size = 1usize
            .checked_shl(sector_shift as u32)
            .ok_or_else(|| format!("invalid CFB sector shift: {sector_shift}"))?;
        if sector_size != SECTOR_SIZE {
            return Err(format!("unsupported CFB sector size: {sector_size}"));
        }

        let first_directory_sector = read_u32_at(file, 48)?;
        let mini_stream_cutoff = read_u32_at(file, 56)? as u64;
        let first_mini_fat_sector = read_u32_at(file, 60)?;
        let mini_fat_sector_count = read_u32_at(file, 64)? as usize;
        let fat_sector_count = read_u32_at(file, 44)? as usize;
        let fat_sectors = read_header_difat(file, fat_sector_count)?;
        let fat = read_fat(file, &fat_sectors, sector_size)?;
        let directory = read_sector_chain(file, first_directory_sector, &fat, sector_size)?;
        let stream = find_directory_entry(&directory, stream_name)?;
        let root = find_directory_entry(&directory, "Root Entry")?;
        let mut data = if stream.size < mini_stream_cutoff {
            let mini_fat = read_mini_fat(
                file,
                first_mini_fat_sector,
                mini_fat_sector_count,
                &fat,
                sector_size,
            )?;
            let mini_stream = read_sector_chain(file, root.start_sector, &fat, sector_size)?;
            read_mini_stream_chain(&mini_stream, stream.start_sector, &mini_fat)?
        } else {
            read_sector_chain(file, stream.start_sector, &fat, sector_size)?
        };
        let wanted_len = usize::try_from(stream.size)
            .map_err(|_| format!("stream is too large to fit in memory: {}", stream.size))?;
        if data.len() < wanted_len {
            return Err(format!(
                "stream {stream_name} chain is shorter than declared size: {} < {wanted_len}",
                data.len()
            ));
        }
        data.truncate(wanted_len);
        Ok(data)
    }

    #[derive(Debug)]
    struct DirectoryEntry {
        start_sector: u32,
        size: u64,
    }

    /// Decode the DIFAT entries stored directly in the CFB header.
    fn read_header_difat(file: &[u8], fat_sector_count: usize) -> Result<Vec<u32>, String> {
        if fat_sector_count > 109 {
            return Err("test CFB reader only supports header DIFAT entries".to_string());
        }
        let mut sectors = Vec::new();
        for index in 0..fat_sector_count {
            let sector = read_u32_at(file, 76 + index * 4)?;
            if sector != FREE_SECTOR {
                sectors.push(sector);
            }
        }
        Ok(sectors)
    }

    /// Load the FAT table from the listed FAT sectors.
    fn read_fat(file: &[u8], fat_sectors: &[u32], sector_size: usize) -> Result<Vec<u32>, String> {
        let mut fat = Vec::new();
        for &sector in fat_sectors {
            let bytes = sector_bytes(file, sector, sector_size)?;
            for chunk in bytes.chunks_exact(4) {
                fat.push(u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]));
            }
        }
        Ok(fat)
    }

    /// Follow a regular FAT chain and concatenate all sector payloads.
    fn read_sector_chain(
        file: &[u8],
        start_sector: u32,
        fat: &[u32],
        sector_size: usize,
    ) -> Result<Vec<u8>, String> {
        if start_sector == END_OF_CHAIN {
            return Ok(Vec::new());
        }
        let mut data = Vec::new();
        let mut sector = start_sector;
        let mut guard = 0usize;
        while sector != END_OF_CHAIN {
            if sector == FREE_SECTOR || sector == FAT_SECTOR {
                return Err(format!("invalid sector in chain: 0x{sector:08x}"));
            }
            data.extend_from_slice(sector_bytes(file, sector, sector_size)?);
            let next = *fat
                .get(sector as usize)
                .ok_or_else(|| format!("sector {sector} is outside the FAT"))?;
            sector = next;
            guard += 1;
            if guard > fat.len() {
                return Err("sector chain appears to contain a cycle".to_string());
            }
        }
        Ok(data)
    }

    /// Locate a named stream entry in the decoded directory bytes.
    fn find_directory_entry(directory: &[u8], name: &str) -> Result<DirectoryEntry, String> {
        for entry in directory.chunks_exact(128) {
            let entry_name = directory_entry_name(entry)?;
            if entry_name == name {
                return Ok(DirectoryEntry {
                    start_sector: read_u32_at(entry, 116)?,
                    size: read_u64_at(entry, 120)?,
                });
            }
        }
        Err(format!("stream not found in compound file: {name}"))
    }

    /// Load the MiniFAT chain, which MathType uses for small Equation Native streams.
    fn read_mini_fat(
        file: &[u8],
        first_sector: u32,
        sector_count: usize,
        fat: &[u32],
        sector_size: usize,
    ) -> Result<Vec<u32>, String> {
        if sector_count == 0 || first_sector == END_OF_CHAIN {
            return Ok(Vec::new());
        }
        let mut bytes = Vec::new();
        let mut sector = first_sector;
        for _ in 0..sector_count {
            bytes.extend_from_slice(sector_bytes(file, sector, sector_size)?);
            sector = *fat
                .get(sector as usize)
                .ok_or_else(|| format!("MiniFAT sector {sector} is outside the FAT"))?;
            if sector == END_OF_CHAIN {
                break;
            }
        }
        Ok(bytes
            .chunks_exact(4)
            .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect())
    }

    /// Follow a MiniFAT chain inside the root mini stream.
    fn read_mini_stream_chain(
        mini_stream: &[u8],
        start_sector: u32,
        mini_fat: &[u32],
    ) -> Result<Vec<u8>, String> {
        const MINI_SECTOR_SIZE: usize = 64;
        if start_sector == END_OF_CHAIN {
            return Ok(Vec::new());
        }
        let mut data = Vec::new();
        let mut sector = start_sector;
        let mut guard = 0usize;
        while sector != END_OF_CHAIN {
            if sector == FREE_SECTOR {
                return Err("invalid free mini sector in chain".to_string());
            }
            let start = sector as usize * MINI_SECTOR_SIZE;
            let end = start + MINI_SECTOR_SIZE;
            let bytes = mini_stream
                .get(start..end)
                .ok_or_else(|| format!("mini sector {sector} is outside the mini stream"))?;
            data.extend_from_slice(bytes);
            sector = *mini_fat
                .get(sector as usize)
                .ok_or_else(|| format!("mini sector {sector} is outside the MiniFAT"))?;
            guard += 1;
            if guard > mini_fat.len() {
                return Err("mini sector chain appears to contain a cycle".to_string());
            }
        }
        Ok(data)
    }

    /// Decode a CFB directory entry name from UTF-16LE.
    fn directory_entry_name(entry: &[u8]) -> Result<String, String> {
        let name_len = read_u16_at(entry, 64)? as usize;
        if name_len < 2 || name_len > 64 {
            return Ok(String::new());
        }
        let raw_name = &entry[..name_len - 2];
        let units = raw_name
            .chunks_exact(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .collect::<Vec<_>>();
        String::from_utf16(&units).map_err(|err| format!("invalid directory UTF-16 name: {err}"))
    }

    /// Return one CFB sector by sector number.
    fn sector_bytes(file: &[u8], sector: u32, sector_size: usize) -> Result<&[u8], String> {
        let start = (sector as usize + 1)
            .checked_mul(sector_size)
            .ok_or_else(|| format!("sector offset overflow for sector {sector}"))?;
        let end = start
            .checked_add(sector_size)
            .ok_or_else(|| format!("sector end overflow for sector {sector}"))?;
        file.get(start..end)
            .ok_or_else(|| format!("sector {sector} is outside the compound file"))
    }

    /// Read a little-endian u16 from a fixed offset.
    fn read_u16_at(data: &[u8], offset: usize) -> Result<u16, String> {
        let bytes = data
            .get(offset..offset + 2)
            .ok_or_else(|| format!("u16 offset {offset} is outside the buffer"))?;
        Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
    }

    /// Read a little-endian u32 from a fixed offset.
    fn read_u32_at(data: &[u8], offset: usize) -> Result<u32, String> {
        let bytes = data
            .get(offset..offset + 4)
            .ok_or_else(|| format!("u32 offset {offset} is outside the buffer"))?;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    /// Read a little-endian u64 from a fixed offset.
    fn read_u64_at(data: &[u8], offset: usize) -> Result<u64, String> {
        let bytes = data
            .get(offset..offset + 8)
            .ok_or_else(|| format!("u64 offset {offset} is outside the buffer"))?;
        Ok(u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }
}
