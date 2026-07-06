use super::*;

/// Write MathType's big-operator template; the following term is its first slot.
pub(super) fn write_big_op(
    kind: BigOpKind,
    body: Option<&Expr>,
    lower: Option<&Expr>,
    upper: Option<&Expr>,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    if !writer.fallback_environment_active
        && (lower.is_some_and(expr_contains_substack) || upper.is_some_and(expr_contains_substack))
    {
        write_text("(Tex translation failed)", out)?;
        return Ok(WriteState {
            size: current_size,
            color: ColorState::Black,
        });
    }
    let Some(body) = body else {
        if let Some((column_spec, rows)) = lower.and_then(subarray_parts) {
            if upper.is_none() {
                return write_bodyless_big_op_subarray_fallback(
                    kind,
                    column_spec,
                    rows,
                    out,
                    current_size,
                    writer,
                );
            }
        }
        if lower.is_none() && upper.is_none() {
            out.push(0x0d);
            writer.ensure_black_color_def(out);
            color_black(out);
            write_standalone_big_op_glyph(kind, out)?;
            return Ok(WriteState {
                size: current_size,
                color: ColorState::Black,
            });
        }
        return write_standalone_big_op_limits(kind, lower, upper, out, current_size, writer);
    };
    let selector = big_op_selector(kind);
    let variation = if upper.is_some() { 0x70 } else { 0x50 };
    // Only the body shares the template-opening line with the big-operator TMPL.
    // Raw fallback inside lower/upper limits belongs to later slots and should
    // not inject an extra black selector before the template header.
    let needs_leading_black = body.contains_raw_tex();
    if needs_leading_black {
        writer.ensure_black_color_def(out);
        color_black(out);
    }
    out.extend_from_slice(&[0x03, 0x00, selector, variation, 0x00]);
    color_default(out);
    let body_state = write_line(body, out, current_size, writer)?;
    let limit_size = match current_size {
        SizeState::Full => SizeState::Sub,
        SizeState::Sub | SizeState::Sub2 => SizeState::Sub2,
    };
    if upper.is_some() {
        if body_state.size != limit_size || expr_requires_explicit_big_op_limit_restore(body) {
            // Some body templates still require an explicit limit-size restore before the
            // following lower slot even when the reported trailing size already equals it.
            write_size(limit_size, out);
        }
        if body_state.size != limit_size || body_state.color != ColorState::Default {
            color_default(out);
        }
    }
    if upper.is_none() {
        if big_op_body_needs_lower_size_restore(body) {
            write_size(limit_size, out);
        }
        if body_state.color != ColorState::Default {
            color_default(out);
        }
    }
    let lower_state = if let Some(lower) = lower {
        write_line(lower, out, limit_size, writer)?
    } else {
        write_null_line(out);
        WriteState {
            size: limit_size,
            color: ColorState::Default,
        }
    };
    let final_limit_state = if let Some(upper) = upper {
        if lower_state.size != limit_size {
            write_size(limit_size, out);
        }
        if lower_state.color != ColorState::Default {
            color_default(out);
        }
        write_line(upper, out, limit_size, writer)?
    } else if lower.is_some() {
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
    } else {
        if lower_state.size != limit_size {
            write_size(limit_size, out);
        }
        if lower_state.color != ColorState::Default {
            color_default(out);
        }
        write_null_line(out);
        WriteState {
            size: limit_size,
            color: ColorState::Default,
        }
    };
    out.push(0x0d);
    if final_limit_state.color != ColorState::Black {
        color_black(out);
    }
    write_big_op_glyph(kind, out)?;
    out.push(0x00);
    Ok(WriteState {
        size: limit_size,
        color: ColorState::Black,
    })
}

/// Write MathType's fallback-line bodyful big-operator template used inside definitions.
pub(super) fn write_fallback_big_op_body(
    kind: BigOpKind,
    body: &Expr,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    let selector = big_op_selector(kind);
    if body.contains_raw_tex() {
        writer.ensure_black_color_def(out);
        color_black(out);
    }
    out.extend_from_slice(&[0x03, 0x00, selector, 0x40, 0x00]);
    color_default(out);
    out.extend_from_slice(&[0x01, 0x00]);
    color_black(out);
    let previous_line_starts_default = writer.line_starts_default;
    writer.line_starts_default = false;
    write_expr(body, out, current_size, writer)?;
    writer.line_starts_default = previous_line_starts_default;
    out.push(0x00);
    let limit_size = match current_size {
        SizeState::Full => SizeState::Sub,
        SizeState::Sub | SizeState::Sub2 => SizeState::Sub2,
    };
    write_size(limit_size, out);
    out.extend_from_slice(&[0x01, 0x01, 0x01, 0x01, 0x0d]);
    write_big_op_glyph(kind, out)?;
    out.push(0x00);
    Ok(WriteState {
        size: limit_size,
        color: ColorState::Black,
    })
}

/// Return true when MathType restates sub-size before a lower-limit slot.
fn big_op_body_needs_lower_size_restore(expr: &Expr) -> bool {
    match expr {
        Expr::Char(_) => true,
        Expr::Fraction(numerator, _) => expr_has_template_structure(numerator),
        Expr::Font {
            kind: FontKind::Bold,
            content,
        } => expr_is_single_char(content, '1'),
        Expr::Script {
            base, sup: Some(_), ..
        } => matches!(base.as_ref(), Expr::Delimited { .. }),
        Expr::Delimited { .. } => true,
        Expr::Sequence(items) => items
            .last()
            .is_some_and(big_op_body_needs_lower_size_restore),
        _ => false,
    }
}

/// Return true for a possibly wrapped one-character expression.
fn expr_is_single_char(expr: &Expr, expected: char) -> bool {
    match expr {
        Expr::Char(ch) => *ch == expected,
        Expr::Sequence(items) => {
            matches!(items.as_slice(), [item] if expr_is_single_char(item, expected))
        }
        _ => false,
    }
}

/// Return true when a fraction numerator contains MathType template structure.
fn expr_has_template_structure(expr: &Expr) -> bool {
    match expr {
        Expr::Script { .. }
        | Expr::Fraction(_, _)
        | Expr::Sqrt(_)
        | Expr::Accent { .. }
        | Expr::BarTemplate { .. }
        | Expr::Delimited { .. }
        | Expr::BigOp { .. }
        | Expr::Integral { .. }
        | Expr::Pile { .. }
        | Expr::Matrix { .. }
        | Expr::Environment { .. } => true,
        Expr::Style { content, .. } | Expr::Font { content, .. } => {
            expr_has_template_structure(content)
        }
        Expr::Sequence(items) => items.iter().any(expr_has_template_structure),
        _ => false,
    }
}

/// Write a big-operator template whose body slot is intentionally empty.
fn write_standalone_big_op_limits(
    kind: BigOpKind,
    lower: Option<&Expr>,
    upper: Option<&Expr>,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    let glyph_name = bodyless_big_op_glyph_name(kind);
    write_bodyless_big_op_template(glyph_name, lower, upper, out, current_size, writer)
}

/// Return one parsed substack node through the usual thin wrappers.
fn substack_rows(expr: &Expr) -> Option<&[Vec<Expr>]> {
    match expr {
        Expr::Substack { rows } => Some(rows),
        Expr::Style { content, .. } => substack_rows(content),
        Expr::Sequence(items) => match items.as_slice() {
            [item] => substack_rows(item),
            _ => None,
        },
        _ => None,
    }
}

/// Return a bodyless big operator that still carries a `\substack` lower limit.
pub(super) fn bodyless_big_op_substack_parts(expr: &Expr) -> Option<(BigOpKind, &[Vec<Expr>])> {
    match expr {
        Expr::BigOp {
            kind,
            lower,
            upper: None,
            body: None,
            ..
        } => lower
            .as_deref()
            .and_then(substack_rows)
            .map(|rows| (*kind, rows)),
        Expr::Style { content, .. } => bodyless_big_op_substack_parts(content),
        Expr::Sequence(items) => match items.as_slice() {
            [item] => bodyless_big_op_substack_parts(item),
            _ => None,
        },
        _ => None,
    }
}

/// Return true when an expression still contains one parsed `\substack`.
fn expr_contains_substack(expr: &Expr) -> bool {
    match expr {
        Expr::Substack { .. } => true,
        Expr::Style { content, .. } => expr_contains_substack(content),
        Expr::Sequence(items) => items.iter().any(expr_contains_substack),
        Expr::Script { base, sub, sup } => {
            expr_contains_substack(base)
                || sub.as_deref().is_some_and(expr_contains_substack)
                || sup.as_deref().is_some_and(expr_contains_substack)
        }
        _ => false,
    }
}

/// Return one parsed subarray environment, allowing the usual thin wrappers.
fn subarray_parts(expr: &Expr) -> Option<(&str, &[Vec<Expr>])> {
    match expr {
        Expr::Subarray { column_spec, rows } => Some((column_spec.as_str(), rows)),
        Expr::Style { content, .. } => subarray_parts(content),
        Expr::Sequence(items) => match items.as_slice() {
            [item] => subarray_parts(item),
            _ => None,
        },
        _ => None,
    }
}

/// Write MathType's bodyless tmSUM template when `\substack` appears inside another
/// unsupported environment such as `aligned`.
pub(super) fn write_big_op_substack_environment_fallback(
    kind: BigOpKind,
    rows: &[Vec<Expr>],
    trailing_end_raw: &str,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    let selector = big_op_selector(kind);
    let variation = 0x50;
    let body = Expr::RawTex(trailing_end_raw.to_string());
    let script_size = match current_size {
        SizeState::Full => SizeState::Sub,
        SizeState::Sub | SizeState::Sub2 => SizeState::Sub2,
    };
    out.extend_from_slice(&[0x03, 0x00, selector, variation, 0x00]);
    color_default(out);
    let body_state = write_line(&body, out, current_size, writer)?;
    if body_state.size != script_size {
        write_size(script_size, out);
    }
    let lower_state = write_substack_lower_slot(rows, out, script_size, writer)?;
    if lower_state.size != script_size {
        write_size(script_size, out);
    }
    write_null_line(out);
    out.push(0x0d);
    write_big_op_glyph(kind, out)?;
    out.push(0x00);
    Ok(WriteState {
        size: script_size,
        color: ColorState::Black,
    })
}

/// Convert one-column `\substack` rows into a plain pile expression.
fn substack_pile_expr(rows: &[Vec<Expr>]) -> Expr {
    match rows {
        [] => Expr::Sequence(Vec::new()),
        [row] => subarray_row_expr(row),
        [first, second] => Expr::Pile {
            kind: PileKind::Plain,
            upper: Box::new(subarray_row_expr(first)),
            lower: Box::new(subarray_row_expr(second)),
        },
        [first, rest @ ..] => Expr::Pile {
            kind: PileKind::Plain,
            upper: Box::new(subarray_row_expr(first)),
            lower: Box::new(substack_pile_expr(rest)),
        },
    }
}

/// Write the lower slot pile MathType uses for two-row `\substack` limits.
fn write_substack_lower_slot(
    rows: &[Vec<Expr>],
    out: &mut Vec<u8>,
    script_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    if rows.len() == 2 {
        out.extend_from_slice(&[0x04, 0x00, 0x02, 0x01]);
        let upper_state = write_line(&subarray_row_expr(&rows[0]), out, script_size, writer)?;
        if upper_state.size != script_size {
            write_size(script_size, out);
        }
        color_default(out);
        let lower_state = write_line(&subarray_row_expr(&rows[1]), out, script_size, writer)?;
        if lower_state.size != script_size {
            write_size(script_size, out);
        }
        out.push(0x00);
        return Ok(WriteState {
            size: script_size,
            color: ColorState::Default,
        });
    }
    write_line(&substack_pile_expr(rows), out, script_size, writer)
}

/// Write MathType's hybrid subarray fallback used after bodyless big-operator glyphs.
fn write_bodyless_big_op_subarray_fallback(
    kind: BigOpKind,
    column_spec: &str,
    rows: &[Vec<Expr>],
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    let script_size = match current_size {
        SizeState::Full => SizeState::Sub,
        SizeState::Sub | SizeState::Sub2 => SizeState::Sub2,
    };
    out.push(0x0d);
    writer.ensure_black_color_def(out);
    color_black(out);
    write_standalone_big_op_glyph(kind, out)?;
    write_size(current_size, out);
    color_default(out);
    write_raw_tex_text(r"_{\begin", out)?;
    color_black(out);
    let first_line = subarray_first_line_expr(column_spec, rows.first());
    write_expr(&first_line, out, script_size, writer)?;
    for row in rows.iter().skip(1) {
        color_default(out);
        write_raw_tex_text("\\\\", out)?;
        color_black(out);
        let row_expr = subarray_row_expr(row);
        write_expr(&row_expr, out, script_size, writer)?;
    }
    color_default(out);
    write_raw_tex_text(r"\end", out)?;
    color_black(out);
    let end_name = owned_char_sequence("subarray");
    write_expr(&end_name, out, script_size, writer)?;
    color_default(out);
    write_raw_tex_text("}", out)?;
    Ok(WriteState {
        size: script_size,
        color: ColorState::Default,
    })
}

/// Write standalone `subarray` using MathType's mixed raw/native fallback shell.
pub(super) fn write_subarray_fallback(
    column_spec: &str,
    rows: &[Vec<Expr>],
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    // Bug-fix: standalone `subarray` stays on MathType's fallback path instead
    // of collapsing to a native MATRIX, but it still renders the rows visibly.
    write_raw_tex_text("\\begin", out)?;
    writer.ensure_black_color_def(out);
    color_black(out);
    let first_line = subarray_first_line_expr(column_spec, rows.first());
    write_expr(&first_line, out, current_size, writer)?;
    for row in rows.iter().skip(1) {
        color_default(out);
        write_raw_tex_text("\\\\", out)?;
        color_black(out);
        let row_expr = subarray_row_expr(row);
        write_expr(&row_expr, out, current_size, writer)?;
    }
    color_default(out);
    write_raw_tex_text("\\end", out)?;
    color_black(out);
    let end_name = owned_char_sequence("subarray");
    write_expr(&end_name, out, current_size, writer)?;
    Ok(WriteState {
        size: current_size,
        color: ColorState::Black,
    })
}

fn subarray_first_line_expr(column_spec: &str, row: Option<&Vec<Expr>>) -> Expr {
    let mut items = char_sequence_items("subarray");
    items.extend(char_sequence_items(column_spec));
    if let Some(row) = row {
        append_subarray_row_items(&mut items, row);
    }
    Expr::Sequence(items)
}

/// Build one visible row line inside MathType's subarray fallback.
fn subarray_row_expr(row: &[Expr]) -> Expr {
    let mut items = Vec::new();
    append_subarray_row_items(&mut items, row);
    Expr::Sequence(items)
}

/// Build owned Expr::Char items from plain text without introducing text-style typefaces.
fn char_sequence_items(text: &str) -> Vec<Expr> {
    text.chars().map(Expr::Char).collect()
}

/// Flatten parser-produced one-cell row wrappers so subarray fallback lines do not emit spurious empty LINE records.
fn append_subarray_row_items(items: &mut Vec<Expr>, row: &[Expr]) {
    for cell in row {
        match unwrap_single_sequence(cell) {
            Expr::Sequence(nested) => items.extend(nested.iter().cloned()),
            other => items.push(other.clone()),
        }
    }
}

/// Build a sequence of character expressions from plain text.
fn owned_char_sequence(text: &str) -> Expr {
    Expr::Sequence(char_sequence_items(text))
}

/// Write the Sigma/Pi glyph MathType appends at the end of a big-op template.
fn write_big_op_glyph(kind: BigOpKind, out: &mut Vec<u8>) -> Result<(), String> {
    let name = big_op_glyph_name(kind);
    write_named_big_operator_glyph(name, out)
}

/// Write MathType's standalone glyph form for big operators without slots.
pub(super) fn write_standalone_big_op_glyph(
    kind: BigOpKind,
    out: &mut Vec<u8>,
) -> Result<(), String> {
    let name = standalone_big_op_glyph_name(kind);
    write_named_big_operator_glyph(name, out)
}

/// Return MathType's template selector for large operators with limits.
fn big_op_selector(kind: BigOpKind) -> u8 {
    match kind {
        BigOpKind::Sum => 0x10,
        BigOpKind::Product => 0x11,
        BigOpKind::Coproduct => 0x12,
        BigOpKind::Union => 0x13,
        BigOpKind::Intersection => 0x14,
    }
}

/// Return the generated glyph-table key for a large operator.
fn big_op_glyph_name(kind: BigOpKind) -> &'static str {
    match kind {
        BigOpKind::Sum => "sum",
        BigOpKind::Product => "product",
        BigOpKind::Coproduct => "coproduct",
        BigOpKind::Union => "union",
        BigOpKind::Intersection => "intersection",
    }
}

/// Return the generated glyph-table key for slotless large operators.
fn standalone_big_op_glyph_name(kind: BigOpKind) -> &'static str {
    match kind {
        BigOpKind::Union => "standalone_union",
        BigOpKind::Intersection => "standalone_intersection",
        _ => big_op_glyph_name(kind),
    }
}

/// Return the generated glyph-table key for bodyless scripted big operators.
pub(super) fn bodyless_big_op_glyph_name(kind: BigOpKind) -> &'static str {
    match kind {
        BigOpKind::Sum => "bodyless_sum",
        BigOpKind::Product => "bodyless_product",
        BigOpKind::Coproduct => "bodyless_coproduct",
        BigOpKind::Union => "bodyless_union",
        BigOpKind::Intersection => "bodyless_intersection",
    }
}

/// Write standalone big-symbol commands whose glyph bytes are learned by probes.
pub(super) fn write_big_symbol_char(
    ch: char,
    out: &mut Vec<u8>,
    writer: &mut MtefWriter,
) -> Result<(), String> {
    if let Some(name) = big_symbol_glyph_name(ch) {
        write_named_big_operator_glyph(name, out)
    } else {
        write_char(ch, out, writer)
    }
}

/// Return the generated glyph-table key for standalone big-symbol aliases.
fn big_symbol_glyph_name(ch: char) -> Option<&'static str> {
    match ch {
        '\u{22c1}' => Some("bigvee"),
        '\u{22c0}' => Some("bigwedge"),
        _ => None,
    }
}

/// Write a postfix script template; selectors match MathType sub/sup variants.
pub(super) fn write_script(
    base: &Expr,
    sub: Option<&Expr>,
    sup: Option<&Expr>,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    if let Expr::SumOperatorSymbol(ch) = base {
        return write_sum_operator_script(*ch, sub, sup, out, current_size, writer);
    }
    let empty_base = expr_is_empty_sequence(base);
    let base_state = if empty_base {
        WriteState {
            size: current_size,
            color: ColorState::Black,
        }
    } else {
        write_script_base(base, out, current_size, writer)?
    };
    if !empty_base
        && (matches!(base, Expr::BigSymbol(_))
            || base_state.size != current_size
            || bar_template_bodyless_operator_needs_script_restore(base))
    {
        write_size(current_size, out);
    }
    if !empty_base
        && (!writer.line_starts_default || writer.parent_sequence_has_previous_sibling)
        && base_state.color != ColorState::Default
        || (empty_base
            && writer.parent_sequence_has_previous_sibling
            && !writer.parent_sequence_previous_was_raw)
    {
        // Bug-fix: MathType restores the inherited/default color before entering
        // tmSCRIPT after inline visible marker content such as `#1^2`, but it
        // does not repeat that selector when a nested base script already ended
        // on the default-color path.
        color_default(out);
    }
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
    if script_size != current_size {
        // Bug-fix: deeply nested scripts already running at Sub2 do not emit
        // a redundant second Sub2 size byte before the first script slot.
        write_size(script_size, out);
    }
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

/// Return true when a bar/underline template around a bodyless operator still leaves
/// MathType in script size before an outer tmSCRIPT opens.
fn bar_template_bodyless_operator_needs_script_restore(expr: &Expr) -> bool {
    match expr {
        Expr::BarTemplate { content, .. } => bodyless_operator_template_content(content).is_some(),
        Expr::DefaultColor(content) | Expr::Style { content, .. } => {
            bar_template_bodyless_operator_needs_script_restore(content)
        }
        Expr::Sequence(items) => {
            matches!(items.as_slice(), [item] if bar_template_bodyless_operator_needs_script_restore(item))
        }
        _ => false,
    }
}

/// Write MathType's tmSUMOP template used by \bigsqcup.
fn write_sum_operator_script(
    ch: char,
    sub: Option<&Expr>,
    sup: Option<&Expr>,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    let script_size = match current_size {
        SizeState::Full => SizeState::Sub,
        SizeState::Sub | SizeState::Sub2 => SizeState::Sub2,
    };
    let variation = if sup.is_some() { 0x70 } else { 0x50 };
    out.extend_from_slice(&[0x03, 0x00, 0x16, variation, 0x00]);
    write_null_line(out);
    write_size(script_size, out);
    if let Some(sub) = sub {
        let sub_state = write_line(sub, out, script_size, writer)?;
        restore_script_separator(sub_state, script_size, out);
    } else {
        write_null_line(out);
    }
    if let Some(sup) = sup {
        write_line(sup, out, script_size, writer)?;
    } else {
        write_null_line(out);
    }
    out.push(0x0d);
    color_default(out);
    write_sum_operator_glyph_line(ch, out, writer)?;
    out.push(0x00);
    Ok(WriteState {
        size: script_size,
        color: ColorState::Black,
    })
}

/// Write the glyph slot inside a tmSUMOP template.
fn write_sum_operator_glyph_line(
    ch: char,
    out: &mut Vec<u8>,
    writer: &mut MtefWriter,
) -> Result<(), String> {
    out.extend_from_slice(&[0x01, 0x00]);
    if let Some(special) = encoding::special_char(ch) {
        match special.explicit_font {
            Some(ExplicitFont::EuclidMathOne) => writer.ensure_euclid_math_one(out),
            Some(ExplicitFont::EuclidMathTwo) => writer.ensure_euclid_math_two(out),
            None => {}
        }
    }
    writer.ensure_black_color_def(out);
    color_black(out);
    write_char(ch, out, writer)?;
    out.push(0x00);
    Ok(())
}

/// Write a standalone tmSUMOP glyph when no scripts were attached.
pub(super) fn write_sum_operator_glyph(
    ch: char,
    out: &mut Vec<u8>,
    writer: &mut MtefWriter,
) -> Result<(), String> {
    if ch == '\u{2211}' {
        return write_named_big_operator_glyph("sum", out);
    }
    write_char(ch, out, writer)
}

/// Write MathType's bodyless big-operator limits template, which shares tmSUMOP layout.
fn write_bodyless_big_op_template(
    glyph_name: &str,
    sub: Option<&Expr>,
    sup: Option<&Expr>,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    let script_size = match current_size {
        SizeState::Full => SizeState::Sub,
        SizeState::Sub | SizeState::Sub2 => SizeState::Sub2,
    };
    let variation = if sup.is_some() { 0x70 } else { 0x50 };
    out.extend_from_slice(&[0x03, 0x00, 0x16, variation, 0x00]);
    write_null_line(out);
    write_size(script_size, out);
    if let Some(sub) = sub {
        let sub_state = write_line(sub, out, script_size, writer)?;
        restore_script_separator(sub_state, script_size, out);
    } else {
        write_null_line(out);
    }
    if let Some(sup) = sup {
        write_line(sup, out, script_size, writer)?;
    } else {
        write_null_line(out);
    }
    out.push(0x0d);
    color_default(out);
    write_named_big_operator_glyph_line(glyph_name, out, writer)?;
    out.push(0x00);
    Ok(WriteState {
        size: script_size,
        color: ColorState::Black,
    })
}

/// Write the glyph slot inside MathType's bodyless big-operator limits template.
pub(super) fn write_named_big_operator_glyph_line(
    glyph_name: &str,
    out: &mut Vec<u8>,
    writer: &mut MtefWriter,
) -> Result<(), String> {
    out.extend_from_slice(&[0x01, 0x00]);
    let glyph = encoding::big_operator_glyph(glyph_name)?;
    if let Some(selector) = glyph.font_style_selector {
        out.extend_from_slice(&[0x08, selector, 0x00]);
    }
    writer.ensure_black_color_def(out);
    color_black(out);
    write_styled_table_char(
        glyph.typeface,
        glyph.mtcode,
        Some(glyph.font_pos),
        glyph.explicit_font,
        out,
        writer,
    );
    out.push(0x00);
    Ok(())
}

/// Write a script base before the script template slots are emitted.
fn write_script_base(
    base: &Expr,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    if matches!(base, Expr::BigSymbol(_)) {
        if writer.big_symbol_line_marker_pending {
            writer.big_symbol_line_marker_pending = false;
        } else {
            out.push(0x0d);
        }
    }
    write_expr(base, out, current_size, writer)
}

/// Restore size/color between script slots after nested scripts changed state.
pub(super) fn restore_script_separator(
    state: WriteState,
    script_size: SizeState,
    out: &mut Vec<u8>,
) {
    if state.size != script_size {
        write_size(script_size, out);
    }
    if state.color != ColorState::Default {
        color_default(out);
    }
}







