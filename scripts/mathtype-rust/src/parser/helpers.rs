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

/// Return true when an old-TeX infix command keeps its native meaning in this parse context.
pub(super) fn infix_command_stays_native(command: &InfixCommand, until: Option<char>) -> bool {
    if !matches!(
        command,
        InfixCommand::Over
            | InfixCommand::Above(_)
            | InfixCommand::Atop
            | InfixCommand::Choose
            | InfixCommand::Brace
            | InfixCommand::Brack
    ) {
        return true;
    }
    until == Some('}')
}

/// Preserve one old-TeX infix command as raw source when MathType does not lower it natively.
pub(super) fn raw_infix_command_expr(
    command: InfixCommand,
    left: Expr,
    right: Expr,
    leading_ws: &str,
) -> Expr {
    let command_expr = match command {
        InfixCommand::Over => Expr::RawTex(format!("{leading_ws}\\over")),
        InfixCommand::Above(command_expr) => {
            normalize_old_tex_infix_command_expr(command_expr, leading_ws)
        }
        InfixCommand::Atop => Expr::RawTex(format!("{leading_ws}\\atop")),
        InfixCommand::Choose => Expr::RawTex(format!("{leading_ws}\\choose")),
        InfixCommand::Brace => Expr::RawTex(format!("{leading_ws}\\brace")),
        InfixCommand::Brack => Expr::RawTex(format!("{leading_ws}\\brack")),
    };
    raw_infix_expr(left, command_expr, right)
}

/// Replace the synthetic spacer on a raw old-TeX infix command with the actual source whitespace.
fn normalize_old_tex_infix_command_expr(expr: Expr, leading_ws: &str) -> Expr {
    match expr {
        Expr::RawTex(raw) => Expr::RawTex(format!("{leading_ws}{}", raw.trim_start())),
        Expr::Sequence(items) => Expr::Sequence(
            items
                .into_iter()
                .enumerate()
                .map(|(index, item)| {
                    if index == 0 {
                        normalize_old_tex_infix_command_expr(item, leading_ws)
                    } else {
                        item
                    }
                })
                .collect(),
        ),
        other => other,
    }
}

/// Preserve an old-TeX infix command between its visible left and right operands.
pub(super) fn raw_infix_expr(left: Expr, command: Expr, right: Expr) -> Expr {
    let mut items = Vec::new();
    push_visible_items(&mut items, left);
    items.push(command);
    push_visible_items(&mut items, right);
    Expr::Sequence(items)
}

/// Append one superscript suffix that MathType preserves verbatim as raw TeX.
pub(super) fn raw_superscript_suffix_expr(base: Expr, raw: &str) -> Expr {
    Expr::Sequence(vec![base, Expr::RawTex(format!("^{{{raw}}}"))])
}

/// Preserve a raw script suffix after one already-visible base expression.
pub(super) fn raw_script_suffix_expr(base: Expr, raw: &str) -> Expr {
    match base {
        Expr::Sequence(mut items) => {
            items.push(Expr::RawTex(raw.to_string()));
            Expr::Sequence(items)
        }
        other if expr_is_empty_sequence(&other) => Expr::RawTex(raw.to_string()),
        other => Expr::Sequence(vec![other, Expr::RawTex(raw.to_string())]),
    }
}

/// Preserve a raw script shell while leaving one reparsed visible fragment native.
pub(super) fn raw_script_hybrid_suffix_expr(
    base: Expr,
    raw_prefix: &str,
    visible: Expr,
    raw_suffix: &str,
) -> Expr {
    let mut items = match base {
        Expr::Sequence(items) => items,
        other if expr_is_empty_sequence(&other) => Vec::new(),
        other => vec![other],
    };
    items.push(Expr::RawTex(raw_prefix.to_string()));
    push_visible_items(&mut items, visible);
    if !raw_suffix.is_empty() {
        items.push(Expr::RawTex(raw_suffix.to_string()));
    }
    Expr::Sequence(items)
}

/// Keep one outer group shell raw when the grouped content already starts and ends on a raw path.
pub(super) fn preserve_raw_group_braces(expr: Expr) -> Expr {
    if !expr_keeps_outer_group_braces_raw(&expr) {
        return expr;
    }
    match expr {
        Expr::RawTex(raw) => Expr::RawTex(format!("{{{raw}}}")),
        Expr::HybridLayout(mut parts) => {
            if let Some(HybridPart::Raw(raw)) = parts.first_mut() {
                raw.insert(0, '{');
            } else {
                parts.insert(0, HybridPart::Raw("{".to_string()));
            }
            if let Some(HybridPart::Raw(raw)) = parts.last_mut() {
                raw.push('}');
            } else {
                parts.push(HybridPart::Raw("}".to_string()));
            }
            Expr::HybridLayout(parts)
        }
        Expr::Sequence(mut items) => {
            if let Some(first) = items.first_mut() {
                *first = prepend_raw_group_shell(first.clone(), "{");
            } else {
                items.insert(0, Expr::RawTex("{".to_string()));
            }
            if items.last().is_some_and(item_is_split_end_shell) {
                items.push(Expr::RawTex("}".to_string()));
            } else if let Some(last) = items.last_mut() {
                *last = append_raw_group_shell(last.clone(), "}");
            } else {
                items.push(Expr::RawTex("}".to_string()));
            }
            Expr::Sequence(items)
        }
        other => Expr::Sequence(vec![
            Expr::RawTex("{".to_string()),
            other,
            Expr::RawTex("}".to_string()),
        ]),
    }
}

/// Preserve `_ { ... }` / `^ { ... }` as one raw shell when the grouped payload already uses one.
pub(super) fn raw_group_script_suffix_expr(
    base: Expr,
    marker: &str,
    operand: Expr,
) -> Option<Expr> {
    expr_keeps_outer_group_braces_raw(&operand)
        .then(|| raw_script_hybrid_suffix_expr(base, &format!("{marker}{{"), operand, "}"))
}

/// Return true when MathType keeps one surrounding `{...}` pair on the raw fallback path.
fn expr_keeps_outer_group_braces_raw(expr: &Expr) -> bool {
    expr_starts_with_raw_group_shell(expr) && expr_ends_with_raw_group_shell(expr)
}

/// Return true when an expression starts with one structural raw group shell.
///
/// Bug-fix: a plain raw control word like `\angln` should not force outer
/// braces onto the raw path, but begin/end fallback shells such as
/// `{\begin{eqnarray}...\end{eqnarray}}` still do in MathType.
fn expr_starts_with_raw_group_shell(expr: &Expr) -> bool {
    match expr {
        Expr::RawTex(_) => false,
        Expr::HybridLayout(parts) => matches!(parts.first(), Some(HybridPart::Raw(_))),
        Expr::DefaultColor(content)
        | Expr::Marked(content)
        | Expr::Style { content, .. }
        | Expr::Font { content, .. }
        | Expr::Color { content, .. } => expr_starts_with_raw_group_shell(content),
        Expr::Sequence(items) => items.first().is_some_and(item_starts_raw_group_shell),
        _ => false,
    }
}

/// Return true when an expression ends with one structural raw group shell.
fn expr_ends_with_raw_group_shell(expr: &Expr) -> bool {
    match expr {
        Expr::RawTex(_) => false,
        Expr::HybridLayout(parts) => matches!(parts.last(), Some(HybridPart::Raw(_))),
        Expr::DefaultColor(content)
        | Expr::Marked(content)
        | Expr::Style { content, .. }
        | Expr::Font { content, .. }
        | Expr::Color { content, .. } => expr_ends_with_raw_group_shell(content),
        Expr::Sequence(items) => items.last().is_some_and(item_ends_raw_group_shell),
        _ => false,
    }
}

/// Return true when one leading item is the raw start of a structural group shell.
fn item_starts_raw_group_shell(expr: &Expr) -> bool {
    match expr {
        Expr::RawTex(raw) => raw == "\\begin",
        Expr::HybridLayout(parts) => matches!(parts.first(), Some(HybridPart::Raw(_))),
        Expr::DefaultColor(content)
        | Expr::Marked(content)
        | Expr::Style { content, .. }
        | Expr::Font { content, .. }
        | Expr::Color { content, .. } => item_starts_raw_group_shell(content),
        Expr::Sequence(items) => items.first().is_some_and(item_starts_raw_group_shell),
        _ => false,
    }
}

/// Return true when one trailing item is the raw end of a structural group shell.
fn item_ends_raw_group_shell(expr: &Expr) -> bool {
    match expr {
        Expr::RawTex(raw) => raw == "\\end" || raw.ends_with('}'),
        Expr::HybridLayout(parts) => matches!(parts.last(), Some(HybridPart::Raw(_))),
        Expr::DefaultColor(content)
        | Expr::Marked(content)
        | Expr::Style { content, .. }
        | Expr::Font { content, .. }
        | Expr::Color { content, .. } => item_ends_raw_group_shell(content),
        Expr::Sequence(items) => {
            item_is_split_end_shell(expr) || items.last().is_some_and(item_ends_raw_group_shell)
        }
        _ => false,
    }
}

/// Return true when one expression stores `\end` and the visible environment name separately.
fn item_is_split_end_shell(expr: &Expr) -> bool {
    matches!(expr, Expr::Sequence(items) if matches!(items.as_slice(), [Expr::RawTex(raw), _] if raw == "\\end"))
}

/// Merge one opening brace into the first structural raw shell when possible.
fn prepend_raw_group_shell(expr: Expr, prefix: &str) -> Expr {
    match expr {
        Expr::RawTex(raw) if raw == "\\begin" => Expr::RawTex(format!("{prefix}{raw}")),
        Expr::HybridLayout(mut parts) => {
            if let Some(HybridPart::Raw(raw)) = parts.first_mut() {
                raw.insert_str(0, prefix);
            }
            Expr::HybridLayout(parts)
        }
        Expr::DefaultColor(content) => {
            Expr::DefaultColor(Box::new(prepend_raw_group_shell(*content, prefix)))
        }
        Expr::Marked(content) => Expr::Marked(Box::new(prepend_raw_group_shell(*content, prefix))),
        Expr::Style { kind, content } => Expr::Style {
            kind,
            content: Box::new(prepend_raw_group_shell(*content, prefix)),
        },
        Expr::Font { kind, content } => Expr::Font {
            kind,
            content: Box::new(prepend_raw_group_shell(*content, prefix)),
        },
        Expr::Color { name, content } => Expr::Color {
            name,
            content: Box::new(prepend_raw_group_shell(*content, prefix)),
        },
        Expr::Sequence(mut items) => {
            if let Some(first) = items.first_mut() {
                *first = prepend_raw_group_shell(first.clone(), prefix);
            }
            Expr::Sequence(items)
        }
        other => other,
    }
}

/// Merge one closing brace into the last structural raw shell when possible.
fn append_raw_group_shell(expr: Expr, suffix: &str) -> Expr {
    match expr {
        Expr::RawTex(raw) if raw == "\\end" || raw.ends_with('}') => {
            Expr::RawTex(format!("{raw}{suffix}"))
        }
        Expr::Sequence(mut items) if matches!(items.as_slice(), [Expr::RawTex(raw), _] if raw == "\\end") =>
        {
            items.push(Expr::RawTex(suffix.to_string()));
            Expr::Sequence(items)
        }
        Expr::HybridLayout(mut parts) => {
            if let Some(HybridPart::Raw(raw)) = parts.last_mut() {
                raw.push_str(suffix);
            }
            Expr::HybridLayout(parts)
        }
        Expr::DefaultColor(content) => {
            Expr::DefaultColor(Box::new(append_raw_group_shell(*content, suffix)))
        }
        Expr::Marked(content) => Expr::Marked(Box::new(append_raw_group_shell(*content, suffix))),
        Expr::Style { kind, content } => Expr::Style {
            kind,
            content: Box::new(append_raw_group_shell(*content, suffix)),
        },
        Expr::Font { kind, content } => Expr::Font {
            kind,
            content: Box::new(append_raw_group_shell(*content, suffix)),
        },
        Expr::Color { name, content } => Expr::Color {
            name,
            content: Box::new(append_raw_group_shell(*content, suffix)),
        },
        Expr::Sequence(mut items) => {
            if let Some(last) = items.last_mut() {
                *last = append_raw_group_shell(last.clone(), suffix);
            }
            Expr::Sequence(items)
        }
        other => other,
    }
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

/// Collapse repeated identical arrow accents that MathType stores only once.
pub(super) fn normalize_arrow_accent_expr(
    kind: ArrowAccentKind,
    under: bool,
    content: Expr,
) -> Expr {
    match content {
        Expr::Sequence(items) if items.len() == 1 => {
            normalize_arrow_accent_expr(kind, under, items.into_iter().next().expect("len checked"))
        }
        other => Expr::ArrowAccent {
            kind,
            under,
            content: Box::new(other),
        },
    }
}

/// Return the visible relation character from a parsed relation atom.
pub(super) fn relation_char(expr: &Expr) -> Option<char> {
    match expr {
        Expr::Char(ch) | Expr::CommandSymbol { ch, .. } => Some(*ch),
        Expr::Sequence(items) if items.len() == 1 => relation_char(&items[0]),
        _ => None,
    }
}

/// Return one raw closing delimiter token that should stay attached to a malformed command shell.
pub(super) fn malformed_closing_delimiter_raw(expr: &Expr) -> Option<&str> {
    match expr {
        Expr::RawTex(raw) if matches!(raw.as_str(), "}" | "]" | ")") => Some(raw.as_str()),
        Expr::Sequence(items) if items.len() == 1 => malformed_closing_delimiter_raw(&items[0]),
        _ => None,
    }
}

/// Return one raw fallback suffix when a script marker is immediately followed by itself.
///
/// Bug-fix: MathType keeps `^^` and `__` as raw source text after the visible
/// base instead of treating the second marker as a literal scripted character.
pub(super) fn repeated_script_marker_raw(marker: char, expr: &Expr) -> Option<&'static str> {
    match (marker, expr) {
        ('^', Expr::Char('^')) => Some("^^"),
        ('_', Expr::Char('_')) => Some("__"),
        _ => None,
    }
}

/// Append one postfix apostrophe to the current expression using MathType's prime semantics.
pub(super) fn append_postfix_prime(expr: Expr) -> Expr {
    match expr {
        Expr::Script { base, sub, sup } => Expr::Script {
            base,
            sub,
            sup: Some(Box::new(append_prime_to_content(
                sup.map(|value| *value)
                    .unwrap_or_else(|| Expr::Sequence(Vec::new())),
            ))),
        },
        Expr::Sequence(mut items) => {
            items.push(Expr::Char('\''));
            Expr::Sequence(items)
        }
        other => Expr::Sequence(vec![other, Expr::Char('\'')]),
    }
}

/// Extend one existing script content expression with a trailing apostrophe.
fn append_prime_to_content(expr: Expr) -> Expr {
    match expr {
        Expr::Sequence(mut items) => {
            items.push(Expr::Char('\''));
            Expr::Sequence(items)
        }
        other if expr_is_empty_sequence(&other) => Expr::Char('\''),
        other => Expr::Sequence(vec![other, Expr::Char('\'')]),
    }
}

/// Continue one already-raw script fallback with another script marker plus visible operand.
pub(super) fn append_raw_script_followup(base: Expr, marker: &str, operand: Expr) -> Option<Expr> {
    let Expr::Sequence(mut items) = base else {
        return None;
    };
    if let Some(Expr::RawTex(raw)) = items.last_mut() {
        if !raw_script_fallback_suffix(raw) {
            return None;
        }
        raw.push_str(marker);
        push_visible_items(&mut items, operand);
        return Some(Expr::Sequence(items));
    }
    if items.len() >= 2 {
        if let Expr::RawTex(raw) = &items[items.len() - 2] {
            if raw_limits_script_operand_suffix(raw) {
                items.push(Expr::RawTex(marker.to_string()));
                push_visible_items(&mut items, operand);
                return Some(Expr::Sequence(items));
            }
        }
    }
    None
}

/// Return true when one raw tail already represents a script-fallback suffix.
fn raw_script_fallback_suffix(raw: &str) -> bool {
    raw.starts_with('^') || raw.starts_with('_')
}

/// Return true when one raw tail already owns the visible operand of `\limits_` / `\nolimits_`.
fn raw_limits_script_operand_suffix(raw: &str) -> bool {
    raw.ends_with("\\limits_") || raw.ends_with("\\nolimits_")
}
/// Return true when a parsed base can consume `\limits` / `\nolimits` natively.
pub(super) fn expr_supports_limits_modifier(expr: &Expr) -> bool {
    match expr {
        Expr::BigOp { .. }
        | Expr::Integral { .. }
        | Expr::IntegralOp { .. }
        | Expr::Limit { .. }
        | Expr::MathOp { .. } => true,
        Expr::FunctionName(name) => matches!(name.as_str(), "lim" | "sup"),
        Expr::Sequence(items) => items.last().is_some_and(expr_supports_limits_modifier),
        _ => false,
    }
}

/// Return true once a limit-aware base already owns at least one parsed script slot.
pub(super) fn expr_has_attached_scripts(expr: &Expr) -> bool {
    match expr {
        Expr::BigOp { lower, upper, .. }
        | Expr::IntegralOp { lower, upper, .. }
        | Expr::Limit { lower, upper, .. }
        | Expr::MathOp { lower, upper, .. } => lower.is_some() || upper.is_some(),
        Expr::Sequence(items) => items.last().is_some_and(expr_has_attached_scripts),
        Expr::Script { .. } => true,
        _ => false,
    }
}

/// Preserve one unsupported `\limits` / `\nolimits` modifier plus its visible script operand.
pub(super) fn raw_limits_script_suffix_expr(
    base: Expr,
    modifier: LimitModifier,
    marker: &str,
    operand: Expr,
) -> Expr {
    let modifier_raw = match modifier {
        LimitModifier::Limits => "\\limits",
        LimitModifier::NoLimits => "\\nolimits",
    };
    let mut items = match base {
        Expr::Sequence(items) => items,
        other => vec![other],
    };
    items.push(Expr::RawTex(format!("{modifier_raw}{marker}")));
    push_visible_items(&mut items, operand);
    Expr::Sequence(items)
}

/// Preserve one unsupported standalone `\limits` / `\nolimits` token.
pub(super) fn raw_limits_modifier_expr(base: Expr, modifier: LimitModifier) -> Expr {
    let modifier_raw = match modifier {
        LimitModifier::Limits => "\\limits",
        LimitModifier::NoLimits => "\\nolimits",
    };
    match base {
        Expr::Sequence(mut items) => {
            items.push(Expr::RawTex(modifier_raw.to_string()));
            Expr::Sequence(items)
        }
        other => Expr::Sequence(vec![other, Expr::RawTex(modifier_raw.to_string())]),
    }
}

/// Preserve an unsupported prefix command while keeping its consumed operand visible.
pub(super) fn raw_prefix_expr(command: &str, operand: Expr) -> Expr {
    let mut items = Vec::with_capacity(2);
    items.push(Expr::RawTex(format!("\\{command}")));
    push_visible_items(&mut items, operand);
    Expr::Sequence(items)
}

/// Preserve a raw command name while rendering each consumed argument as visible follow-up content.
pub(super) fn raw_prefix_sequence(command: &str, args: Vec<Expr>) -> Expr {
    let mut items = Vec::with_capacity(args.len() + 1);
    items.push(Expr::RawTex(format!("\\{command}")));
    for arg in args {
        push_visible_items(&mut items, arg);
    }
    Expr::Sequence(items)
}

/// Preserve a precomputed raw prefix while rendering each consumed argument as visible follow-up content.
pub(super) fn raw_prefix_sequence_with_raw(raw_prefix: String, args: Vec<Expr>) -> Expr {
    let mut items = Vec::with_capacity(args.len() + 1);
    items.push(Expr::RawTex(raw_prefix));
    for arg in args {
        push_visible_items(&mut items, arg);
    }
    Expr::Sequence(items)
}

/// Append one raw-text fragment to a hybrid layout, merging adjacent raw runs.
pub(super) fn append_hybrid_raw(parts: &mut Vec<HybridPart>, raw: impl Into<String>) {
    let raw = raw.into();
    if raw.is_empty() {
        return;
    }
    match parts.last_mut() {
        Some(HybridPart::Raw(existing)) => existing.push_str(&raw),
        _ => parts.push(HybridPart::Raw(raw)),
    }
}

/// Append one visible LINE payload to a hybrid layout, skipping empty wrapper sequences.
pub(super) fn append_hybrid_line(parts: &mut Vec<HybridPart>, expr: Expr) {
    let expr = collapse_single_sequence(expr);
    if expr_is_empty_sequence(&expr) {
        return;
    }
    parts.push(HybridPart::Line(Box::new(expr)));
}

/// Flatten a parser expression into MathType's hybrid raw-text and LINE parts.
pub(super) fn append_expr_as_hybrid(parts: &mut Vec<HybridPart>, expr: Expr) {
    match expr {
        Expr::HybridLayout(inner) => {
            for part in inner {
                match part {
                    HybridPart::Raw(raw) => append_hybrid_raw(parts, raw),
                    HybridPart::Line(expr) => append_hybrid_line(parts, *expr),
                }
            }
        }
        Expr::RawTex(raw) => append_hybrid_raw(parts, raw),
        Expr::Sequence(items) => {
            let mut visible = Vec::new();
            for item in items {
                match item {
                    Expr::RawTex(raw) => {
                        flush_hybrid_visible_items(parts, &mut visible);
                        append_hybrid_raw(parts, raw);
                    }
                    Expr::HybridLayout(inner) => {
                        flush_hybrid_visible_items(parts, &mut visible);
                        append_expr_as_hybrid(parts, Expr::HybridLayout(inner));
                    }
                    other => visible.push(other),
                }
            }
            flush_hybrid_visible_items(parts, &mut visible);
        }
        other => append_hybrid_line(parts, other),
    }
}

/// Prepend visible prefix glyphs to the first LINE in one hybrid fallback run.
pub(super) fn prepend_hybrid_visible_prefix(parts: &mut Vec<HybridPart>, prefix: Expr) {
    let mut prefix_items = Vec::new();
    push_visible_items(&mut prefix_items, prefix);
    if prefix_items.is_empty() {
        return;
    }
    for part in parts.iter_mut() {
        if let HybridPart::Line(expr) = part {
            let mut items = prefix_items.clone();
            push_visible_items(&mut items, (**expr).clone());
            *expr = Box::new(collapse_single_sequence(Expr::Sequence(items)));
            return;
        }
    }
    append_hybrid_line(
        parts,
        collapse_single_sequence(Expr::Sequence(prefix_items)),
    );
}

/// Insert visible prefix glyphs before a hybrid run that starts with raw command text.
pub(super) fn insert_hybrid_visible_prefix(parts: &mut Vec<HybridPart>, prefix: Expr) {
    if matches!(parts.first(), Some(HybridPart::Raw(_))) {
        let mut prefix_items = Vec::new();
        push_visible_items(&mut prefix_items, prefix);
        if prefix_items.is_empty() {
            return;
        }
        parts.insert(
            0,
            HybridPart::Line(Box::new(collapse_single_sequence(Expr::Sequence(
                prefix_items,
            )))),
        );
        return;
    }
    prepend_hybrid_visible_prefix(parts, prefix);
}

/// Append visible suffix glyphs to the last LINE in one hybrid fallback run.
pub(super) fn append_hybrid_visible_suffix(parts: &mut Vec<HybridPart>, suffix: Expr) {
    let mut suffix_items = Vec::new();
    push_visible_items(&mut suffix_items, suffix);
    if suffix_items.is_empty() {
        return;
    }
    if matches!(parts.last(), Some(HybridPart::Raw(_))) {
        append_hybrid_line(
            parts,
            collapse_single_sequence(Expr::Sequence(suffix_items)),
        );
        return;
    }
    for part in parts.iter_mut().rev() {
        if let HybridPart::Line(expr) = part {
            let mut items = Vec::new();
            push_visible_items(&mut items, (**expr).clone());
            items.extend(suffix_items);
            *expr = Box::new(collapse_single_sequence(Expr::Sequence(items)));
            return;
        }
    }
    append_hybrid_line(
        parts,
        collapse_single_sequence(Expr::Sequence(suffix_items)),
    );
}

/// Flush pending visible items into one hybrid LINE part.
fn flush_hybrid_visible_items(parts: &mut Vec<HybridPart>, visible: &mut Vec<Expr>) {
    if visible.is_empty() {
        return;
    }
    append_hybrid_line(
        parts,
        collapse_single_sequence(Expr::Sequence(std::mem::take(visible))),
    );
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
        LeftRightDelimiter::Command { command, ch } => {
            items.push(Expr::CommandSymbol { command, ch });
        }
        LeftRightDelimiter::RawCommand(command) => {
            items.push(Expr::RawTex(format!("\\{side}\\{command}")));
        }
    }
}

/// Return true for control-word delimiters that MathType stores raw under left-right fences.
pub(super) fn raw_left_right_delimiter_command(command: &str) -> bool {
    matches!(command, "lt" | "gt" | "lgroup" | "rgroup")
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
        if chars[index] == '#' && matches!(chars[index + 1], '1' | '2') {
            if start < index {
                let mut chunk = chars[start..index].iter().collect::<String>();
                let trailing_ws = take_trailing_ascii_whitespace(&mut chunk);
                if !chunk.is_empty() {
                    args.push(parser.parse_visible_wrapper_text(&chunk)?);
                }
                if !trailing_ws.is_empty() {
                    args.push(Expr::RawTex(format!("{trailing_ws}#")));
                } else {
                    args.push(Expr::RawTex("#".to_string()));
                }
            } else {
                args.push(Expr::RawTex("#".to_string()));
            }
            args.push(Expr::Char(chars[index + 1]));
            start = index + 2;
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

/// Remove trailing ASCII whitespace so raw-only macro markers keep MathType's spacing.
fn take_trailing_ascii_whitespace(text: &mut String) -> String {
    let split = text.trim_end_matches(char::is_whitespace).len();
    text.split_off(split)
}

/// Return true when MathType keeps one following source-space inside the raw command run.
pub(super) fn raw_command_preserves_trailing_space(command: &str) -> bool {
    matches!(command, "allowbreak")
}

/// Preserve exact source whitespace that MathType keeps inside raw-text fallback runs.
pub(super) fn with_leading_raw_space(expr: Expr, leading_ws: &str) -> Expr {
    if leading_ws.is_empty() {
        return expr;
    }
    match expr {
        Expr::RawTex(text) => Expr::RawTex(format!("{leading_ws}{text}")),
        Expr::DefaultColor(content) => {
            Expr::DefaultColor(Box::new(with_leading_raw_space(*content, leading_ws)))
        }
        Expr::Marked(content) => {
            Expr::Marked(Box::new(with_leading_raw_space(*content, leading_ws)))
        }
        Expr::Color { name, content } => Expr::Color {
            name,
            content: Box::new(with_leading_raw_space(*content, leading_ws)),
        },
        Expr::Style { kind, content } => Expr::Style {
            kind,
            content: Box::new(with_leading_raw_space(*content, leading_ws)),
        },
        Expr::Font { kind, content } => Expr::Font {
            kind,
            content: Box::new(with_leading_raw_space(*content, leading_ws)),
        },
        Expr::HybridLayout(mut parts) => {
            if let Some(HybridPart::Raw(text)) = parts.first_mut() {
                text.insert_str(0, leading_ws);
            }
            Expr::HybridLayout(parts)
        }
        Expr::Sequence(mut items) => {
            if let Some(first) = items.first_mut() {
                *first = with_leading_raw_space(first.clone(), leading_ws);
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
pub(super) fn parse_literal_char(ch: char, leading_ws: &str) -> Result<Expr, String> {
    if let Some(fragments) = literal_raw_text_override(ch) {
        return Ok(literal_override_expr(fragments, leading_ws));
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
        leading_ws,
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
    leading_ws: &str,
) -> Expr {
    let mut items = Vec::with_capacity(fragments.len());
    for fragment in fragments {
        match fragment {
            LiteralOverrideFragment::Raw(bytes) => {
                let mut raw = bytes.iter().copied().map(char::from).collect::<String>();
                if !leading_ws.is_empty() && items.is_empty() {
                    raw.insert_str(0, leading_ws);
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

/// Merge adjacent raw fallback fragments into the same Sequence slot unless a RawBoundary splits them.
///
/// Bug-fix: MathType often keeps consecutive direct-literal fallback bytes and their source spaces
/// inside one raw-text run, while the parser can naturally build them as several sibling RawTex nodes.
pub(super) fn merge_adjacent_raw_tex(expr: Expr) -> Expr {
    match expr {
        Expr::Sequence(items) => {
            let mut merged = Vec::with_capacity(items.len());
            for item in items.into_iter().map(merge_adjacent_raw_tex) {
                match item {
                    Expr::RawTex(raw) => {
                        if let Some(Expr::RawTex(previous)) = merged.last_mut() {
                            previous.push_str(&raw);
                        } else {
                            merged.push(Expr::RawTex(raw));
                        }
                    }
                    other => merged.push(other),
                }
            }
            Expr::Sequence(merged)
        }
        Expr::DefaultColor(content) => {
            Expr::DefaultColor(Box::new(merge_adjacent_raw_tex(*content)))
        }
        Expr::Marked(content) => Expr::Marked(Box::new(merge_adjacent_raw_tex(*content))),
        Expr::Color { name, content } => Expr::Color {
            name,
            content: Box::new(merge_adjacent_raw_tex(*content)),
        },
        Expr::Style { kind, content } => Expr::Style {
            kind,
            content: Box::new(merge_adjacent_raw_tex(*content)),
        },
        Expr::Font { kind, content } => Expr::Font {
            kind,
            content: Box::new(merge_adjacent_raw_tex(*content)),
        },
        Expr::Accent { kind, content } => Expr::Accent {
            kind,
            content: Box::new(merge_adjacent_raw_tex(*content)),
        },
        Expr::ArrowAccent {
            kind,
            under,
            content,
        } => Expr::ArrowAccent {
            kind,
            under,
            content: Box::new(merge_adjacent_raw_tex(*content)),
        },
        Expr::BarTemplate { kind, content } => Expr::BarTemplate {
            kind,
            content: Box::new(merge_adjacent_raw_tex(*content)),
        },
        Expr::Strike { kind, content } => Expr::Strike {
            kind,
            content: Box::new(merge_adjacent_raw_tex(*content)),
        },
        Expr::NotRelation(content) => Expr::NotRelation(Box::new(merge_adjacent_raw_tex(*content))),
        Expr::Fraction(left, right) => Expr::Fraction(
            Box::new(merge_adjacent_raw_tex(*left)),
            Box::new(merge_adjacent_raw_tex(*right)),
        ),
        Expr::Sqrt(content) => Expr::Sqrt(Box::new(merge_adjacent_raw_tex(*content))),
        Expr::NthRoot { index, radicand } => Expr::NthRoot {
            index: Box::new(merge_adjacent_raw_tex(*index)),
            radicand: Box::new(merge_adjacent_raw_tex(*radicand)),
        },
        Expr::BigOp {
            kind,
            lower,
            upper,
            body,
            placement,
        } => Expr::BigOp {
            kind,
            lower: lower.map(|expr| Box::new(merge_adjacent_raw_tex(*expr))),
            upper: upper.map(|expr| Box::new(merge_adjacent_raw_tex(*expr))),
            body: body.map(|expr| Box::new(merge_adjacent_raw_tex(*expr))),
            placement,
        },
        Expr::FallbackBigOp { kind, body } => Expr::FallbackBigOp {
            kind,
            body: Box::new(merge_adjacent_raw_tex(*body)),
        },
        Expr::Limit {
            name,
            lower,
            upper,
            placement,
        } => Expr::Limit {
            name,
            lower: lower.map(|expr| Box::new(merge_adjacent_raw_tex(*expr))),
            upper: upper.map(|expr| Box::new(merge_adjacent_raw_tex(*expr))),
            placement,
        },
        Expr::MathOp {
            content,
            lower,
            upper,
            placement,
            leading_space,
        } => Expr::MathOp {
            content: Box::new(merge_adjacent_raw_tex(*content)),
            lower: lower.map(|expr| Box::new(merge_adjacent_raw_tex(*expr))),
            upper: upper.map(|expr| Box::new(merge_adjacent_raw_tex(*expr))),
            placement,
            leading_space,
        },
        Expr::IntegralOp {
            kind,
            lower,
            upper,
            body,
            placement,
        } => Expr::IntegralOp {
            kind,
            lower: lower.map(|expr| Box::new(merge_adjacent_raw_tex(*expr))),
            upper: upper.map(|expr| Box::new(merge_adjacent_raw_tex(*expr))),
            body: body.map(|expr| Box::new(merge_adjacent_raw_tex(*expr))),
            placement,
        },
        Expr::Pile { kind, upper, lower } => Expr::Pile {
            kind,
            upper: Box::new(merge_adjacent_raw_tex(*upper)),
            lower: Box::new(merge_adjacent_raw_tex(*lower)),
        },
        Expr::Brace {
            kind,
            content,
            annotation,
        } => Expr::Brace {
            kind,
            content: Box::new(merge_adjacent_raw_tex(*content)),
            annotation: annotation.map(|expr| Box::new(merge_adjacent_raw_tex(*expr))),
        },
        Expr::Stackrel { upper, lower } => Expr::Stackrel {
            upper: Box::new(merge_adjacent_raw_tex(*upper)),
            lower: Box::new(merge_adjacent_raw_tex(*lower)),
        },
        Expr::Underset { lower, base } => Expr::Underset {
            lower: Box::new(merge_adjacent_raw_tex(*lower)),
            base: Box::new(merge_adjacent_raw_tex(*base)),
        },
        Expr::XArrow { kind, label, under } => Expr::XArrow {
            kind,
            label: Box::new(merge_adjacent_raw_tex(*label)),
            under: under.map(|expr| Box::new(merge_adjacent_raw_tex(*expr))),
        },
        Expr::Substack { rows } => Expr::Substack {
            rows: rows
                .into_iter()
                .map(|row| row.into_iter().map(merge_adjacent_raw_tex).collect())
                .collect(),
        },
        Expr::Subarray { column_spec, rows } => Expr::Subarray {
            column_spec,
            rows: rows
                .into_iter()
                .map(|row| row.into_iter().map(merge_adjacent_raw_tex).collect())
                .collect(),
        },
        Expr::Matrix { kind, rows } => Expr::Matrix {
            kind,
            rows: rows
                .into_iter()
                .map(|row| row.into_iter().map(merge_adjacent_raw_tex).collect())
                .collect(),
        },
        Expr::Environment { kind, rows, trivia } => Expr::Environment {
            kind,
            rows: rows
                .into_iter()
                .map(|row| row.into_iter().map(merge_adjacent_raw_tex).collect())
                .collect(),
            trivia,
        },
        Expr::Delimited {
            left,
            right,
            content,
        } => Expr::Delimited {
            left,
            right,
            content: Box::new(merge_adjacent_raw_tex(*content)),
        },
        Expr::OneSidedDelimited {
            side,
            delimiter,
            content,
        } => Expr::OneSidedDelimited {
            side,
            delimiter,
            content: Box::new(merge_adjacent_raw_tex(*content)),
        },
        Expr::Script { base, sub, sup } => Expr::Script {
            base: Box::new(merge_adjacent_raw_tex(*base)),
            sub: sub.map(|expr| Box::new(merge_adjacent_raw_tex(*expr))),
            sup: sup.map(|expr| Box::new(merge_adjacent_raw_tex(*expr))),
        },
        Expr::HybridLayout(parts) => Expr::HybridLayout(parts),
        other => other,
    }
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

/// Build MathType's four-sign sequence for `\iiiint`.
pub(super) fn quadruple_integral_expr() -> Expr {
    Expr::Sequence(vec![
        Expr::Integral {
            kind: IntegralKind::Single,
        },
        Expr::DefaultColor(Box::new(Expr::Char('\u{ef01}'))),
        Expr::Integral {
            kind: IntegralKind::Single,
        },
        Expr::DefaultColor(Box::new(Expr::Char('\u{ef01}'))),
        Expr::Integral {
            kind: IntegralKind::Single,
        },
        Expr::DefaultColor(Box::new(Expr::Char('\u{ef01}'))),
        Expr::Integral {
            kind: IntegralKind::Single,
        },
    ])
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
            lower: Some(lower),
            upper: None,
            body: None,
            placement: LimitPlacement::Limits,
        } if sub.is_some() => repeated_bodyless_big_op_script_expr(kind, "_", *lower, sub.unwrap()),
        Expr::BigOp {
            kind,
            lower: None,
            upper: Some(upper),
            body: None,
            placement: LimitPlacement::Limits,
        } if sup.is_some() => repeated_bodyless_big_op_script_expr(kind, "^", *upper, sup.unwrap()),
        Expr::BigOp {
            kind,
            lower,
            upper,
            body,
            placement,
        } => Expr::BigOp {
            kind,
            lower: sub.map(Box::new).or(lower),
            upper: sup.map(Box::new).or(upper),
            body,
            placement: match limit_modifier {
                Some(LimitModifier::Limits) => LimitPlacement::Limits,
                Some(LimitModifier::NoLimits) => LimitPlacement::NoLimits,
                None => placement,
            },
        },
        Expr::Integral { kind } => Expr::IntegralOp {
            kind,
            lower: sub.map(Box::new),
            upper: sup.map(Box::new),
            body: None,
            placement: match limit_modifier {
                Some(LimitModifier::Limits) => LimitPlacement::Limits,
                Some(LimitModifier::NoLimits) | None => LimitPlacement::NoLimits,
            },
        },
        Expr::IntegralOp {
            kind,
            lower,
            upper,
            body,
            placement,
        } => Expr::IntegralOp {
            kind,
            lower: sub.map(Box::new).or(lower),
            upper: sup.map(Box::new).or(upper),
            body,
            placement: match limit_modifier {
                Some(LimitModifier::Limits) => LimitPlacement::Limits,
                Some(LimitModifier::NoLimits) => LimitPlacement::NoLimits,
                None => placement,
            },
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
            placement: LimitPlacement::Limits,
        },
        Expr::Limit {
            name,
            lower,
            upper,
            placement,
        } => Expr::Limit {
            name,
            lower: sub.map(Box::new).or(lower),
            upper: sup.map(Box::new).or(upper),
            placement: match limit_modifier {
                Some(LimitModifier::Limits) => LimitPlacement::Limits,
                Some(LimitModifier::NoLimits) => LimitPlacement::NoLimits,
                None => placement,
            },
        },
        Expr::MathOp {
            content,
            lower,
            upper,
            placement,
            leading_space,
        } => Expr::MathOp {
            content,
            lower: sub.map(Box::new).or(lower),
            upper: sup.map(Box::new).or(upper),
            placement: match limit_modifier {
                Some(LimitModifier::Limits) => LimitPlacement::Limits,
                Some(LimitModifier::NoLimits) => LimitPlacement::NoLimits,
                None => placement,
            },
            leading_space,
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
        } if sub.is_some() && old_sub.is_some() => Expr::Script {
            base: Box::new(Expr::Script {
                base,
                sub: old_sub,
                sup: old_sup,
            }),
            sub: sub.map(Box::new),
            sup: None,
        },
        Expr::Script {
            base,
            sub: old_sub,
            sup: old_sup,
        } if sup.is_some() && old_sup.is_some() => Expr::Script {
            base: Box::new(Expr::Script {
                base,
                sub: old_sub,
                sup: old_sup,
            }),
            sub: None,
            sup: sup.map(Box::new),
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

/// Preserve repeated same-direction scripts on bodyless big operators as raw markers plus visible slots.
///
/// Bug-fix: MathType does not nest a second `_`/`^` onto a standalone `\sum`-style template.
/// It falls back to the visible operator glyph followed by raw script markers and visible script payloads.
fn repeated_bodyless_big_op_script_expr(
    kind: BigOpKind,
    marker: &str,
    old_script: Expr,
    new_script: Expr,
) -> Expr {
    let mut items = vec![
        bodyless_big_op_visible_expr(kind),
        Expr::RawTex(marker.to_string()),
    ];
    push_visible_items(&mut items, old_script);
    items.push(Expr::RawTex(marker.to_string()));
    push_visible_items(&mut items, new_script);
    Expr::Sequence(items)
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
            | "biggl"
            | "Biggl"
            | "biggr"
            | "Biggr"
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
