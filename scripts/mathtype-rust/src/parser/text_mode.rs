use crate::ast::{Expr, StrikeKind};

/// Parse the supported command subset that MathType accepts inside \text{...}.
pub(super) fn parse_content(raw: &str) -> Expr {
    let chars: Vec<char> = raw.chars().collect();
    let mut pos = 0usize;
    let mut text = String::new();
    let mut items = Vec::new();

    while pos < chars.len() {
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
            if let Some(ch) = take_text_circled_char(&chars, &mut pos).and_then(circled_char) {
                text.push(ch);
                continue;
            }
            flush_text(&mut text, &mut items);
            items.push(Expr::RawTex("\\textcircled".to_string()));
            continue;
        }

        if command == "sout" {
            if let Some(group) = take_text_group(&chars, &mut pos) {
                flush_text(&mut text, &mut items);
                items.push(Expr::Strike {
                    kind: StrikeKind::Horizontal,
                    content: Box::new(parse_content(&group)),
                });
                continue;
            }
        }

        if let Some(mark) = text_accent_combining_mark(&command) {
            if let Some((argument, raw_suffix)) = take_text_accent_argument(&chars, &mut pos) {
                if let Some(argument_text) = plain_text_content(&argument) {
                    text.push_str(&argument_text);
                    text.push(mark);
                    continue;
                }
                flush_text(&mut text, &mut items);
                items.push(Expr::RawTex(format!("\\{command}{raw_suffix}")));
                continue;
            }
        }

        if let Some(ch) = text_command_char(&command) {
            text.push(ch);
        } else {
            flush_text(&mut text, &mut items);
            items.push(Expr::RawTex(format!("\\{command}")));
        }
    }

    flush_text(&mut text, &mut items);
    sequence_or_single(items)
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
fn take_text_accent_argument(chars: &[char], pos: &mut usize) -> Option<(String, String)> {
    while *pos < chars.len() && chars[*pos].is_whitespace() {
        *pos += 1;
    }
    if chars.get(*pos) == Some(&'{') {
        let group = take_text_group(chars, pos)?;
        let raw_suffix = format!("{{{group}}}");
        return Some((group, raw_suffix));
    }
    let ch = chars.get(*pos).copied()?;
    *pos += 1;
    let argument = ch.to_string();
    Some((argument.clone(), argument))
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

/// Return the Unicode enclosed alphanumeric form for \textcircled.
fn circled_char(ch: char) -> Option<char> {
    match ch {
        'A'..='Z' => char::from_u32(0x24b6 + (ch as u32 - 'A' as u32)),
        'a'..='z' => char::from_u32(0x24d0 + (ch as u32 - 'a' as u32)),
        '0' => Some('\u{24ea}'),
        '1'..='9' => char::from_u32(0x2460 + (ch as u32 - '1' as u32)),
        _ => None,
    }
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

/// Return plain text from a text-mode accent argument when no layout is needed.
fn plain_text_content(raw: &str) -> Option<String> {
    match parse_content(raw) {
        Expr::Text(text) => Some(text),
        Expr::Sequence(items) => {
            let mut text = String::new();
            for item in items {
                match item {
                    Expr::Text(item_text) => text.push_str(&item_text),
                    _ => return None,
                }
            }
            Some(text)
        }
        _ => None,
    }
}

/// Return the Unicode combining mark for LaTeX text-mode accent commands.
fn text_accent_combining_mark(command: &str) -> Option<char> {
    Some(match command {
        "'" => '\u{0301}',
        "`" => '\u{0300}',
        "^" => '\u{0302}',
        "~" => '\u{0303}',
        "=" => '\u{0304}',
        "u" => '\u{0306}',
        "." => '\u{0307}',
        "\"" => '\u{0308}',
        "r" => '\u{030a}',
        "H" => '\u{030b}',
        "v" => '\u{030c}',
        _ => return None,
    })
}

/// Return a text-mode command's literal character when MathType need not see TeX.
fn text_command_char(command: &str) -> Option<char> {
    Some(match command {
        "%" => '%',
        "#" => '#',
        "&" => '&',
        "_" | "textunderscore" => '_',
        "$" | "textdollar" => '$',
        "{" | "textbraceleft" => '{',
        "}" | "textbraceright" => '}',
        "textendash" => '\u{2013}',
        "textemdash" => '\u{2014}',
        "textasciitilde" => '~',
        "textasciicircum" => '^',
        "textellipsis" => '\u{2026}',
        "textquoteleft" => '\u{2018}',
        "textquoteright" => '\u{2019}',
        "textquotedblleft" => '\u{201c}',
        "textquotedblright" => '\u{201d}',
        "textless" => '<',
        "textgreater" => '>',
        "textbar" => '|',
        "textbardbl" => '\u{2016}',
        "textbackslash" => '\\',
        "textsterling" => '\u{00a3}',
        "textdegree" => '\u{00b0}',
        "textregistered" => '\u{00ae}',
        "textdagger" => '\u{2020}',
        "textdaggerdbl" => '\u{2021}',
        "P" => '\u{00b6}',
        "S" | "sect" => '\u{00a7}',
        "OE" => '\u{0152}',
        "oe" => '\u{0153}',
        "O" => '\u{00d8}',
        "o" => '\u{00f8}',
        "ss" => '\u{00df}',
        "AA" => '\u{00c5}',
        "aa" => '\u{00e5}',
        "AE" => '\u{00c6}',
        "ae" => '\u{00e6}',
        "i" => '\u{0131}',
        "j" => '\u{0237}',
        _ => return None,
    })
}
