use crate::ast::Expr;
use crate::mathtype_ansi::encode_mathtype_text;

/// Parse the supported command subset that MathType accepts inside \text{...}.
pub(super) fn parse_content(raw: &str) -> Expr {
    let chars: Vec<char> = raw.chars().collect();
    let mut pos = 0usize;
    let mut text = String::new();
    let mut items = Vec::new();

    while pos < chars.len() {
        // MathType drops a leading text-mode space when the visible content starts
        // with a literal `$...$` fragment inside `\text{...}`.
        if chars[pos].is_whitespace() && text.is_empty() && chars.get(pos + 1) == Some(&'$') {
            pos += 1;
            continue;
        }
        if chars[pos] == '$' {
            if let Some(end) = find_text_math_fragment_end(&chars, pos + 1) {
                // Bug-fix: MathType drops the literal text-mode space that sits
                // immediately before one `$...$` fragment inside `\text{...}`.
                while text.ends_with(char::is_whitespace) {
                    text.pop();
                }
                flush_text(&mut text, &mut items);
                items.push(Expr::Char('$'));
                push_text_math_fragment_items(&chars[(pos + 1)..end], &mut items);
                items.push(Expr::Char('$'));
                pos = end + 1;
                continue;
            }
        }
        if chars[pos] != '\\' {
            text.push(chars[pos]);
            pos += 1;
            continue;
        }

        pos += 1;
        let name_start = pos;
        while pos < chars.len() && chars[pos].is_ascii_alphabetic() {
            pos += 1;
        }
        let command = if name_start == pos {
            chars.get(pos).map(|ch| {
                pos += 1;
                ch.to_string()
            })
        } else {
            Some(chars[name_start..pos].iter().collect())
        };
        let Some(command) = command else {
            text.push('\\');
            continue;
        };

        if command == "textcircled" {
            flush_text(&mut text, &mut items);
            items.push(Expr::RawTex("\\textcircled".to_string()));
            if let Some(ch) = take_text_circled_char(&chars, &mut pos) {
                items.push(Expr::Text(ch.to_string()));
            }
            continue;
        }

        if command == "sout" {
            if let Some(group) = take_text_group(&chars, &mut pos) {
                flush_text(&mut text, &mut items);
                items.push(Expr::RawTex("\\sout".to_string()));
                items.push(parse_content(&group));
                continue;
            }
        }

        if is_text_accent_command(&command) {
            if let Some(argument) = take_text_accent_expr(&chars, &mut pos) {
                flush_text(&mut text, &mut items);
                items.push(text_accent_expr(&command, argument));
                continue;
            }
        }

        if let Some(ch) = text_literal_char(&command) {
            text.push(ch);
        } else if let Some(expr) = text_direct_expr(&command) {
            flush_text(&mut text, &mut items);
            items.push(expr);
        } else {
            flush_text(&mut text, &mut items);
            items.push(Expr::RawTex(format!("\\{command}")));
        }
    }

    flush_text(&mut text, &mut items);
    sequence_or_single(items)
}

/// Find the closing `$` for one visible text-mode math fragment.
fn find_text_math_fragment_end(chars: &[char], mut pos: usize) -> Option<usize> {
    while pos < chars.len() {
        if chars[pos] == '$' {
            return Some(pos);
        }
        pos += 1;
    }
    None
}

/// Push the visible items MathType keeps from one `$...$` fragment inside `\text{...}`.
fn push_text_math_fragment_items(chars: &[char], items: &mut Vec<Expr>) {
    let mut text = String::new();
    for &ch in chars {
        if ch == '?' {
            flush_text(&mut text, items);
            items.push(Expr::Char('?'));
            continue;
        }
        if matches!(ch, '<' | '>') {
            flush_text(&mut text, items);
            items.push(Expr::Char(ch));
            continue;
        }
        if ch.is_ascii() {
            text.push(ch);
            continue;
        }
        match encode_mathtype_text(&ch.to_string()) {
            Ok(bytes) if bytes.iter().all(|byte| *byte == b'?') => {
                flush_text(&mut text, items);
                for _ in 0..bytes.len() {
                    items.push(Expr::Char('?'));
                }
            }
            _ => text.push(ch),
        }
    }
    flush_text(&mut text, items);
}

/// Consume the single argument form used by \textcircled a.
fn take_text_circled_char(chars: &[char], pos: &mut usize) -> Option<char> {
    while *pos < chars.len() && chars[*pos].is_whitespace() {
        *pos += 1;
    }
    if chars.get(*pos) == Some(&'{') {
        return take_text_group(chars, pos).and_then(|group| {
            let mut chars = group.chars();
            let ch = chars.next()?;
            chars.next().is_none().then_some(ch)
        });
    }
    let ch = chars.get(*pos).copied()?;
    *pos += 1;
    Some(ch)
}

/// Consume the argument form used by text-mode accent commands such as \'{a}.
fn take_text_accent_expr(chars: &[char], pos: &mut usize) -> Option<Expr> {
    while *pos < chars.len() && chars[*pos].is_whitespace() {
        *pos += 1;
    }
    if chars.get(*pos) == Some(&'{') {
        let group = take_text_group(chars, pos)?;
        return Some(parse_content(&group));
    }
    let ch = chars.get(*pos).copied()?;
    *pos += 1;
    Some(Expr::Text(ch.to_string()))
}

/// Consume a balanced text-mode group after a command such as \sout.
fn take_text_group(chars: &[char], pos: &mut usize) -> Option<String> {
    while *pos < chars.len() && chars[*pos].is_whitespace() {
        *pos += 1;
    }
    if chars.get(*pos) != Some(&'{') {
        return None;
    }
    *pos += 1;
    let start = *pos;
    let mut depth = 1usize;
    while *pos < chars.len() {
        match chars[*pos] {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    let end = *pos;
                    *pos += 1;
                    return Some(chars[start..end].iter().collect());
                }
            }
            _ => {}
        }
        *pos += 1;
    }
    None
}

/// Move accumulated text into the expression list without creating empty runs.
fn flush_text(text: &mut String, items: &mut Vec<Expr>) {
    if !text.is_empty() {
        items.push(Expr::Text(std::mem::take(text)));
    }
}

/// Collapse one-item helper output to keep the AST compact.
fn sequence_or_single(mut items: Vec<Expr>) -> Expr {
    if items.len() == 1 {
        items.pop().unwrap()
    } else {
        Expr::Sequence(items)
    }
}

/// Return true when a text-mode command consumes one accent argument.
fn is_text_accent_command(command: &str) -> bool {
    matches!(
        command,
        "'" | "`" | "^" | "~" | "=" | "u" | "." | "\"" | "r" | "H" | "v"
    )
}

/// Preserve MathType's text-mode accent command while keeping the visible argument native.
fn text_accent_expr(command: &str, argument: Expr) -> Expr {
    match command {
        "'" => Expr::Sequence(vec![
            Expr::RawTex("\\".to_string()),
            text_empty_base_superscript(Expr::CommandSymbol {
                command: "prime".to_string(),
                ch: '\u{2032}',
            }),
            argument,
        ]),
        "^" => Expr::Sequence(vec![
            Expr::RawTex("\\".to_string()),
            text_empty_base_superscript(argument),
        ]),
        "\"" => raw_backslash_sequence(Expr::RawTex("\"".to_string()), argument),
        "." => raw_backslash_sequence(Expr::Char('.'), argument),
        "=" => raw_backslash_sequence(Expr::Text("=".to_string()), argument),
        "~" => raw_backslash_sequence(Expr::Char('~'), argument),
        _ => Expr::Sequence(vec![Expr::RawTex(format!("\\{command}")), argument]),
    }
}

/// Preserve one raw backslash prefix before a visible accent marker and its argument.
fn raw_backslash_sequence(marker: Expr, argument: Expr) -> Expr {
    Expr::Sequence(vec![Expr::RawTex("\\".to_string()), marker, argument])
}

/// Build the empty-base superscript shape MathType uses for text-mode accent forms.
fn text_empty_base_superscript(sup: Expr) -> Expr {
    Expr::Script {
        base: Box::new(Expr::Sequence(Vec::new())),
        sub: None,
        sup: Some(Box::new(sup)),
    }
}

/// Return a text-mode command's literal escape character when MathType need not see TeX.
fn text_literal_char(command: &str) -> Option<char> {
    Some(match command {
        "%" => '%',
        "#" => '#',
        "&" => '&',
        "_" => '_',
        "$" => '$',
        "{" => '{',
        "}" => '}',
        _ => return None,
    })
}

/// Return one text-mode command that MathType emits directly as a visible expression.
fn text_direct_expr(command: &str) -> Option<Expr> {
    Some(match command {
        "AA" => Expr::Char('\u{00c5}'),
        "O" => Expr::Char('\u{2205}'),
        "P" => Expr::Char('\u{00b6}'),
        "S" | "sect" => Expr::Char('\u{00a7}'),
        _ => return None,
    })
}
