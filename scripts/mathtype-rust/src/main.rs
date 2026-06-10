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

#[derive(Clone, Debug)]
enum Expr {
    Sequence(Vec<Expr>),
    Char(char),
    Fraction(Box<Expr>, Box<Expr>),
    Sqrt(Box<Expr>),
    Script {
        base: Box<Expr>,
        sub: Option<Box<Expr>>,
        sup: Option<Box<Expr>>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SizeState {
    Full,
    Sub,
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
    let expr = Parser::new(&latex).parse()?;
    let mtef = write_mtef(&latex, &expr)?;
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
            "" => {
                let ch = self.next().ok_or_else(|| "dangling backslash".to_string())?;
                Ok(Expr::Char(ch))
            }
            _ => Err(format!("unsupported LaTeX command: \\{command}")),
        }
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
    write_equation_body(expr, &mut out)?;
    Ok(out)
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
    out.extend_from_slice(&[
        0x0a, 0x01, 0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0f, 0x01,
    ]);
    write_expr(expr, out, SizeState::Full)?;
    out.extend_from_slice(&[0x00, 0x00]);
    Ok(())
}

/// Write an expression in MathType's record order for the supported subset.
fn write_expr(expr: &Expr, out: &mut Vec<u8>, current_size: SizeState) -> Result<SizeState, String> {
    let next_size = match expr {
        Expr::Sequence(items) => {
            let mut state = current_size;
            for item in items {
                if state != current_size {
                    write_size(current_size, out);
                    state = current_size;
                }
                state = write_expr(item, out, state)?;
            }
            state
        }
        Expr::Char(ch) => {
            write_char(*ch, out)?;
            current_size
        }
        Expr::Fraction(numerator, denominator) => write_fraction(numerator, denominator, out, current_size)?,
        Expr::Sqrt(radicand) => write_sqrt(radicand, out, current_size)?,
        Expr::Script { base, sub, sup } => write_script(base, sub.as_deref(), sup.as_deref(), out, current_size)?,
    };
    Ok(next_size)
}

/// Write one MTEF CHAR record using MathType's simple font/style choices.
fn write_char(ch: char, out: &mut Vec<u8>) -> Result<(), String> {
    let code = ch as u32;
    if code > u16::MAX as u32 {
        return Err(format!("character is outside BMP and not yet supported: {ch}"));
    }

    out.push(0x02);
    if is_symbol_char(ch) {
        out.push(0x04);
        out.push(0x86);
        write_u16(code as u16, out);
        out.push(code as u8);
    } else {
        out.push(0x00);
        out.push(if ch.is_ascii_digit() { 0x88 } else { 0x83 });
        write_u16(code as u16, out);
    }
    Ok(())
}

/// Return true for operators MathType stores through Symbol font positions.
fn is_symbol_char(ch: char) -> bool {
    matches!(
        ch,
        '+' | '-' | '=' | '<' | '>' | '(' | ')' | '[' | ']' | ',' | '.' | ':' | ';' | '/' | '*'
    )
}

/// Write a MathType fraction template with numerator and denominator slots.
fn write_fraction(
    numerator: &Expr,
    denominator: &Expr,
    out: &mut Vec<u8>,
    current_size: SizeState,
) -> Result<SizeState, String> {
    out.extend_from_slice(&[0x03, 0x00, 0x0b, 0x00, 0x00]);
    color_default(out);
    let numerator_size = write_line(numerator, out, current_size)?;
    if numerator_size != current_size {
        write_size(current_size, out);
    } else {
        color_default(out);
    }
    let denominator_size = write_line(denominator, out, current_size)?;
    out.push(0x00);
    Ok(denominator_size)
}

/// Write a square-root template with a null nth-root index slot.
fn write_sqrt(radicand: &Expr, out: &mut Vec<u8>, current_size: SizeState) -> Result<SizeState, String> {
    out.extend_from_slice(&[0x03, 0x00, 0x0a, 0x00, 0x00]);
    color_default(out);
    let radicand_size = write_line(radicand, out, current_size)?;
    if radicand_size != SizeState::Sub {
        write_size(SizeState::Sub, out);
    }
    write_null_line(out);
    out.push(0x00);
    Ok(SizeState::Sub)
}

/// Write a postfix script template; selectors match MathType sub/sup variants.
fn write_script(
    base: &Expr,
    sub: Option<&Expr>,
    sup: Option<&Expr>,
    out: &mut Vec<u8>,
    current_size: SizeState,
) -> Result<SizeState, String> {
    let base_size = write_expr(base, out, current_size)?;
    if base_size != current_size {
        write_size(current_size, out);
    }
    color_default(out);
    let selector = match (sub.is_some(), sup.is_some()) {
        (true, false) => 0x1b,
        (false, true) => 0x1c,
        (true, true) => 0x1d,
        (false, false) => return Ok(current_size),
    };
    out.extend_from_slice(&[0x03, 0x00, selector, 0x00, 0x00, 0x0b]);
    match (sub, sup) {
        (Some(sub), None) => {
            write_line(sub, out, SizeState::Sub)?;
            color_default(out);
            write_null_line(out);
        }
        (None, Some(sup)) => {
            write_null_line(out);
            write_line(sup, out, SizeState::Sub)?;
        }
        (Some(sub), Some(sup)) => {
            write_line(sub, out, SizeState::Sub)?;
            color_default(out);
            write_line(sup, out, SizeState::Sub)?;
        }
        (None, None) => {}
    }
    out.push(0x00);
    Ok(SizeState::Sub)
}

/// Write a non-null LINE record with MathType's black color selection inside.
fn write_line(expr: &Expr, out: &mut Vec<u8>, current_size: SizeState) -> Result<SizeState, String> {
    out.extend_from_slice(&[0x01, 0x00, 0x0f, 0x01]);
    let final_size = write_expr(expr, out, current_size)?;
    out.push(0x00);
    Ok(final_size)
}

/// Write MathType's compact placeholder line for absent script slots.
fn write_null_line(out: &mut Vec<u8>) {
    out.extend_from_slice(&[0x01, 0x01]);
}

/// Select the inherited/default color, used by MathType before template slots.
fn color_default(out: &mut Vec<u8>) {
    out.extend_from_slice(&[0x0f, 0x00]);
}

/// Emit a compact MathType size record when template slots need restoration.
fn write_size(size: SizeState, out: &mut Vec<u8>) {
    out.push(match size {
        SizeState::Full => 0x0a,
        SizeState::Sub => 0x0b,
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
