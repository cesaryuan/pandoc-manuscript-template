use super::*;

/// Build the AST node for a TeX infix command after both sides are parsed.
pub(super) fn infix_expr(command: InfixCommand, left: Expr, right: Expr) -> Expr {
    match command {
        InfixCommand::Over => Expr::Fraction(Box::new(left), Box::new(right)),
        InfixCommand::Above(command_expr) => raw_infix_expr(left, command_expr, right),
        InfixCommand::Atop => raw_infix_expr(left, Expr::RawTex(" \\atop".to_string()), right),
        InfixCommand::Choose => Expr::Pile {
            kind: PileKind::Parenthesized,
            upper: Box::new(left),
            lower: Box::new(right),
        },
        InfixCommand::Brace => raw_infix_expr(left, Expr::RawTex("\\brace".to_string()), right),
        InfixCommand::Brack => raw_infix_expr(left, Expr::RawTex("\\brack".to_string()), right),
    }
}

/// Preserve an old-TeX infix command between its visible left and right operands.
pub(super) fn raw_infix_expr(left: Expr, command: Expr, right: Expr) -> Expr {
    Expr::Sequence(vec![left, command, right])
}

/// Append one superscript suffix that MathType preserves verbatim as raw TeX.
pub(super) fn raw_superscript_suffix_expr(base: Expr, raw: &str) -> Expr {
    Expr::Sequence(vec![base, Expr::RawTex(format!("^{{{raw}}}"))])
}

/// Build native modulo text for TeX's parenthesized modulo operators.
pub(super) fn modulo_parenthesized_expr(command: &str, argument: Expr) -> Expr {
    let content = if command == "pmod" {
        Expr::Sequence(vec![Expr::FunctionName("mod".to_string()), argument])
    } else {
        argument
    };
    Expr::Sequence(vec![
        Expr::Space(0x05),
        Expr::Delimited {
            left: '(',
            right: ')',
            content: Box::new(content),
        },
    ])
}

/// Return the visible relation character from a parsed relation atom.
pub(super) fn relation_char(expr: &Expr) -> Option<char> {
    match expr {
        Expr::Char(ch) | Expr::CommandSymbol { ch, .. } => Some(*ch),
        Expr::Sequence(items) if items.len() == 1 => relation_char(&items[0]),
        _ => None,
    }
}

/// Preserve an unsupported prefix command while keeping its consumed operand visible.
pub(super) fn raw_prefix_expr(command: &str, operand: Expr) -> Expr {
    Expr::Sequence(vec![Expr::RawTex(format!("\\{command}")), operand])
}

/// Preserve a raw command name while rendering each consumed argument as visible follow-up content.
pub(super) fn raw_prefix_sequence(command: &str, args: Vec<Expr>) -> Expr {
    let mut items = Vec::with_capacity(args.len() + 1);
    items.push(Expr::RawTex(format!("\\{command}")));
    items.extend(args);
    Expr::Sequence(items)
}

/// Preserve a precomputed raw prefix while rendering each consumed argument as visible follow-up content.
pub(super) fn raw_prefix_sequence_with_raw(raw_prefix: String, args: Vec<Expr>) -> Expr {
    let mut items = Vec::with_capacity(args.len() + 1);
    items.push(Expr::RawTex(raw_prefix));
    items.extend(args);
    Expr::Sequence(items)
}

/// Return true for unsupported environments that MathType stores as raw begin/end
/// commands plus visible interior tokens instead of one raw body blob.
pub(super) fn unsupported_environment_uses_hybrid_begin_end(name: &str) -> bool {
    matches!(name, "CD")
}

/// Build one visible character sequence used by hybrid begin/end fallbacks.
pub(super) fn visible_text_sequence(text: &str) -> Expr {
    Expr::Sequence(text.chars().map(Expr::Char).collect())
}

/// Flatten one visible wrapper group so raw-prefix hybrids keep MathType's item order.
pub(super) fn push_visible_items(items: &mut Vec<Expr>, expr: Expr) {
    match expr {
        Expr::Sequence(seq) => items.extend(seq),
        other => items.push(other),
    }
}

/// Preserve one probe-known raw left-right control word or one native delimiter char.
pub(super) fn push_left_right_delimiter(
    items: &mut Vec<Expr>,
    side: &str,
    delimiter: LeftRightDelimiter,
) {
    match delimiter {
        LeftRightDelimiter::Char(ch) => items.push(Expr::Char(ch)),
        LeftRightDelimiter::RawCommand(command) => {
            items.push(Expr::RawTex(format!("\\{side}\\{command}")));
        }
    }
}

/// Return true for control-word delimiters that MathType stores raw under left-right fences.
pub(super) fn raw_left_right_delimiter_command(command: &str) -> bool {
    matches!(command, "lt" | "gt")
}

/// Split a raw-only macro replacement so MathType-style parameter markers keep their raw `#` bytes.
pub(super) fn render_raw_only_macro_replacement(
    replacement: &str,
    parser: &Parser,
) -> Result<Vec<Expr>, String> {
    let chars = replacement.chars().collect::<Vec<_>>();
    let mut args = Vec::new();
    let mut start = 0usize;
    let mut index = 0usize;
    while index + 1 < chars.len() {
        if chars[index] == '#' && chars[index + 1] == '1' {
            if start < index {
                let chunk = chars[start..index].iter().collect::<String>();
                if !chunk.is_empty() {
                    args.push(parser.parse_visible_wrapper_text(&chunk)?);
                }
            }
            args.push(Expr::RawTex("#".to_string()));
            start = index + 1;
            index += 2;
            continue;
        }
        index += 1;
    }
    if start < chars.len() {
        let chunk = chars[start..].iter().collect::<String>();
        if !chunk.is_empty() {
            args.push(parser.parse_visible_wrapper_text(&chunk)?);
        }
    }
    Ok(args)
}

/// Return true when MathType keeps one following source-space inside the raw command run.
pub(super) fn raw_command_preserves_trailing_space(command: &str) -> bool {
    matches!(command, "allowbreak")
}

/// Preserve spaces that MathType keeps inside raw-text fallback runs before unsupported commands.
pub(super) fn with_leading_raw_space(expr: Expr, had_leading_ws: bool) -> Expr {
    if !had_leading_ws {
        return expr;
    }
    match expr {
        Expr::RawTex(text) => Expr::RawTex(format!(" {text}")),
        Expr::Sequence(mut items) => {
            if let Some(Expr::RawTex(text)) = items.first_mut() {
                text.insert(0, ' ');
            }
            Expr::Sequence(items)
        }
        other => other,
    }
}

/// MathType's raw fallback CHAR runs serialize line breaks as literal `n` bytes.
pub(super) fn normalize_environment_fallback_whitespace(raw: &str) -> String {
    raw.chars()
        .map(|ch| match ch {
            '\r' | '\n' => 'n',
            other => other,
        })
        .collect()
}

/// Mirror MathType TeX Input's fallback behavior for direct literals MathType stores as raw fragments.
pub(super) fn parse_literal_char(ch: char, had_leading_ws: bool) -> Result<Expr, String> {
    if let Some(fragments) = literal_raw_text_override(ch) {
        return Ok(literal_override_expr(fragments, had_leading_ws));
    }
    if ch.is_ascii() {
        return Ok(Expr::Char(ch));
    }
    let encoded = encode_mathtype_text(&ch.to_string())?;
    if encoded.iter().all(|byte| *byte == b'?') {
        let items = encoded.iter().map(|_| Expr::Char('?')).collect::<Vec<_>>();
        return Ok(match items.as_slice() {
            [item] => item.clone(),
            _ => Expr::Sequence(items),
        });
    }
    Ok(with_leading_raw_space(
        split_encoded_literal_bytes(&encoded),
        had_leading_ws,
    ))
}

/// Split one ANSI-encoded MathType literal into raw bytes plus visible ASCII fragments.
///
/// Some direct Unicode literals become mixed byte streams such as `0xA8 0x49`, where
/// MathType keeps the high byte raw but renders the trailing ASCII byte visibly.
pub(super) fn split_encoded_literal_bytes(bytes: &[u8]) -> Expr {
    let mut items = Vec::new();
    let mut raw = Vec::new();
    for &byte in bytes {
        if byte.is_ascii() {
            if !raw.is_empty() {
                items.push(Expr::RawTex(raw.drain(..).map(char::from).collect()));
            }
            items.push(Expr::Char(byte as char));
        } else {
            raw.push(byte);
        }
    }
    if !raw.is_empty() {
        items.push(Expr::RawTex(raw.into_iter().map(char::from).collect()));
    }
    match items.as_slice() {
        [item] => item.clone(),
        _ => Expr::Sequence(items),
    }
}

/// Build one parser expression from generated direct-literal fallback fragments.
pub(super) fn literal_override_expr(
    fragments: &[LiteralOverrideFragment],
    had_leading_ws: bool,
) -> Expr {
    let mut items = Vec::with_capacity(fragments.len());
    for fragment in fragments {
        match fragment {
            LiteralOverrideFragment::Raw(bytes) => {
                let mut raw = bytes.iter().copied().map(char::from).collect::<String>();
                if had_leading_ws && items.is_empty() {
                    raw.insert(0, ' ');
                }
                items.push(Expr::RawTex(raw));
            }
            LiteralOverrideFragment::Char(ch) => items.push(Expr::Char(*ch)),
        }
    }
    match items.len() {
        0 => Expr::Sequence(Vec::new()),
        1 => items
            .into_iter()
            .next()
            .expect("one literal override fragment"),
        _ => Expr::Sequence(items),
    }
}

/// Return true for parser-produced non-visible layout placeholders.
pub(super) fn expr_is_empty_sequence(expr: &Expr) -> bool {
    matches!(expr, Expr::Sequence(items) if items.is_empty())
}

/// Drop one redundant wrapper sequence when a script group contains exactly one item.
pub(super) fn collapse_single_sequence(expr: Expr) -> Expr {
    match expr {
        Expr::Sequence(mut items) if items.len() == 1 => items.remove(0),
        other => other,
    }
}

/// Return true when a braced superscript contains only apostrophes and spaces.
pub(super) fn is_raw_prime_script_group(raw: &str) -> bool {
    !raw.is_empty()
        && raw.chars().any(|ch| ch == '\'')
        && raw.chars().all(|ch| matches!(ch, '\'' | ' '))
}

/// Map extensible-arrow commands that share MathType's x-arrow template.
pub(super) fn xarrow_command_kind(command: &str) -> Option<XArrowKind> {
    match command {
        "xleftarrow" => Some(XArrowKind::Left),
        "xrightarrow" => Some(XArrowKind::Right),
        "xLeftarrow" => Some(XArrowKind::DoubleLeft),
        "xRightarrow" => Some(XArrowKind::DoubleRight),
        "xhookleftarrow" => Some(XArrowKind::HookLeft),
        "xhookrightarrow" => Some(XArrowKind::HookRight),
        "xtwoheadleftarrow" => Some(XArrowKind::TwoHeadLeft),
        "xtwoheadrightarrow" => Some(XArrowKind::TwoHeadRight),
        "xmapsto" => Some(XArrowKind::Mapsto),
        "xlongequal" => Some(XArrowKind::LongEqual),
        "xtofrom" => Some(XArrowKind::ToFrom),
        _ => None,
    }
}

/// Return true for x-arrow variants that MathType keeps as raw command text plus visible labels.
pub(super) fn raw_hybrid_xarrow_command(command: &str) -> bool {
    matches!(
        command,
        "xLeftarrow"
            | "xRightarrow"
            | "xhookleftarrow"
            | "xhookrightarrow"
            | "xtwoheadleftarrow"
            | "xtwoheadrightarrow"
            | "xmapsto"
            | "xlongequal"
            | "xtofrom"
    )
}

/// Return the MathType logical-size style represented by a TeX style switch.
pub(super) fn style_command_kind(command: &str) -> StyleKind {
    match command {
        "displaystyle" => StyleKind::Display,
        "textstyle" => StyleKind::Text,
        "scriptstyle" => StyleKind::Script,
        "scriptscriptstyle" => StyleKind::ScriptScript,
        _ => unreachable!("style_command_kind is only called for style switches"),
    }
}

/// Build a blackboard-bold single-letter alias such as \R or \Complex.
pub(super) fn blackboard_letter(ch: char) -> Expr {
    Expr::Font {
        kind: FontKind::MathBb,
        content: Box::new(Expr::Char(ch)),
    }
}

/// Return true when a parsed expression already matches MathType's plain-text failure placeholder.
pub(super) fn expr_is_mathtype_translation_failed(expr: &Expr) -> bool {
    match expr {
        Expr::Text(text) => text == MATHTYPE_TEXT_TRANSLATION_FAILED,
        // \\substack currently becomes a one-item sequence around the failure text,
        // and MathType collapses the surrounding scripted formula when that happens.
        Expr::Sequence(items) => {
            matches!(items.as_slice(), [item] if expr_is_mathtype_translation_failed(item))
        }
        Expr::Style { content, .. } => expr_is_mathtype_translation_failed(content),
        _ => false,
    }
}

/// Preserve MathType's postfix script template shape by merging repeated scripts.
pub(super) fn merge_script(
    base: Expr,
    sub: Option<Expr>,
    sup: Option<Expr>,
    limit_modifier: Option<LimitModifier>,
) -> Expr {
    if sub
        .as_ref()
        .is_some_and(expr_is_mathtype_translation_failed)
        || sup
            .as_ref()
            .is_some_and(expr_is_mathtype_translation_failed)
    {
        return Expr::Text(MATHTYPE_TEXT_TRANSLATION_FAILED.to_string());
    }
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
        Expr::FunctionName(name)
            if matches!(name.as_str(), "lim" | "sup")
                && limit_modifier == Some(LimitModifier::NoLimits) =>
        {
            Expr::Script {
                base: Box::new(Expr::FunctionName(name)),
                sub: sub.map(Box::new),
                sup: sup.map(Box::new),
            }
        }
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
        Expr::Brace {
            kind,
            content,
            annotation,
        } if (kind == BraceKind::Under && sub.is_some())
            || (kind == BraceKind::Over && sup.is_some()) =>
        {
            Expr::Brace {
                kind,
                content,
                annotation: sub.or(sup).map(Box::new).or(annotation),
            }
        }
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

/// Map delimiter commands used after \left and \right to visible fence characters.
pub(super) fn delimiter_command_char(command: &str) -> Option<char> {
    DELIMITER_COMMAND_CHARS
        .iter()
        .find(|entry| entry.command == command)
        .map(|entry| entry.ch)
        .or_else(|| command_to_char(command).filter(|ch| is_dynamic_delimiter_char(*ch)))
}

/// Return true for ordinary command aliases that can also act as delimiters.
pub(super) fn is_dynamic_delimiter_char(ch: char) -> bool {
    matches!(
        ch,
        '(' | ')'
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

/// Map TeX spacing commands onto the fnSPACE bytes already verified by samples.
pub(super) fn spacing_command_width(command: &str) -> Option<u8> {
    match command {
        "quad" => Some(0x05),
        "qquad" => Some(0x06),
        "medspace" => Some(0x02),
        "thickspace" => Some(0x04),
        "thinspace" | "space" | "nobreakspace" => Some(0x08),
        "negthinspace" | "negmedspace" | "negthickspace" => Some(0x01),
        _ => None,
    }
}

/// Map one-character TeX spacing escapes onto MathType's fnSPACE widths.
pub(super) fn escaped_single_char_space(ch: char) -> Option<u8> {
    match ch {
        '!' => Some(0x01),
        ',' => Some(0x08),
        ':' | '>' => Some(0x02),
        ';' => Some(0x04),
        _ => None,
    }
}

/// Return true for delimiter-size hints that MathType drops while keeping the following fence.
pub(super) fn ignored_delimiter_size_command(command: &str) -> bool {
    matches!(
        command,
        "big"
            | "Big"
            | "bigg"
            | "Bigg"
            | "bigl"
            | "Bigl"
            | "bigr"
            | "Bigr"
            | "bigm"
            | "Bigm"
            | "biggl"
            | "Biggl"
            | "biggr"
            | "Biggr"
            | "biggm"
            | "Biggm"
    )
}

/// Return escaped single-character commands that MathType stores as visible math glyphs.
pub(super) fn escaped_single_char_math_char(ch: char) -> Option<char> {
    match ch {
        '#' | '%' | '&' | '_' => Some(ch),
        _ => None,
    }
}

/// Map no-argument LaTeX commands to the Unicode symbol MathType stores.
pub(super) fn command_to_char(command: &str) -> Option<char> {
    TEX_COMMAND_CHARS
        .iter()
        .find(|entry| entry.command == command)
        .map(|entry| entry.ch)
}

/// Map no-argument LaTeX commands to short visible symbol sequences.
pub(super) fn command_to_sequence(command: &str) -> Option<&'static [char]> {
    TEX_COMMAND_SEQUENCES
        .iter()
        .find(|entry| entry.command == command)
        .map(|entry| entry.chars)
}

/// Map no-argument LaTeX commands to text-style characters.
pub(super) fn command_to_text(command: &str) -> Option<&'static str> {
    TEX_COMMAND_TEXTS
        .iter()
        .find(|entry| entry.command == command)
        .map(|entry| entry.text)
}

/// Map MathType big-symbol commands to the base glyph they write.
pub(super) fn big_symbol_command_to_char(command: &str) -> Option<char> {
    BIG_SYMBOL_COMMAND_CHARS
        .iter()
        .find(|entry| entry.command == command)
        .map(|entry| entry.ch)
}

/// Map MathType tmSUMOP commands to the base glyph they write.
pub(super) fn sum_operator_command_to_char(command: &str) -> Option<char> {
    SUM_OPERATOR_COMMAND_CHARS
        .iter()
        .find(|entry| entry.command == command)
        .map(|entry| entry.ch)
}

/// Return the logical character for source-command-aware symbols.
pub(super) fn command_specific_to_char(command: &str) -> Option<char> {
    COMMAND_SPECIFIC_CHARS
        .iter()
        .find(|entry| entry.command == command)
        .map(|entry| entry.ch)
}
