use super::*;

/// Return true when MathType starts the equation body with a MATRIX record.
pub(super) fn expr_starts_with_top_matrix(expr: &Expr) -> bool {
    match expr {
        Expr::Environment {
            kind:
                EnvironmentKind::Array
                | EnvironmentKind::Align
                | EnvironmentKind::Aligned
                | EnvironmentKind::AlignAt
                | EnvironmentKind::AlignedAt
                | EnvironmentKind::Split,
            ..
        } => true,
        Expr::Limit { .. } => true,
        Expr::Underset { .. } => true,
        Expr::Matrix { .. } => true,
        Expr::Style { content, .. } => expr_starts_with_top_matrix(content),
        Expr::Sequence(items) => items.first().is_some_and(expr_starts_with_top_matrix),
        _ => false,
    }
}

/// Return true for top-level matrices wrapped in a fence template.
pub(super) fn expr_starts_with_top_fenced_matrix(expr: &Expr) -> bool {
    match expr {
        Expr::Matrix {
            kind:
                MatrixKind::Parenthesized
                | MatrixKind::Bracketed
                | MatrixKind::Braced
                | MatrixKind::Barred
                | MatrixKind::DoubleBarred,
            ..
        } => true,
        Expr::Style { content, .. } => expr_starts_with_top_fenced_matrix(content),
        Expr::Sequence(items) => items
            .first()
            .is_some_and(expr_starts_with_top_fenced_matrix),
        _ => false,
    }
}

/// Return true when MathType writes a top-level PILE template before line color.
pub(super) fn expr_starts_with_top_pile_template(expr: &Expr) -> bool {
    match expr {
        Expr::Pile { .. } => true,
        Expr::Style { content, .. } => expr_starts_with_top_pile_template(content),
        Expr::Sequence(items) => items
            .first()
            .is_some_and(expr_starts_with_top_pile_template),
        _ => false,
    }
}

/// Return true when a line-leading style switch owns the visible size transition.
pub(super) fn expr_starts_with_top_style(expr: &Expr) -> bool {
    match expr {
        Expr::Style { .. } => true,
        Expr::Sequence(items) => items.first().is_some_and(expr_starts_with_top_style),
        _ => false,
    }
}

/// Return true when the first visible top-level node is a color wrapper.
pub(super) fn expr_starts_with_top_color(expr: &Expr) -> bool {
    match expr {
        Expr::Color { .. } => true,
        Expr::Sequence(items) => items.first().is_some_and(expr_starts_with_top_color),
        _ => false,
    }
}

/// Return true when MathType starts this environment as raw TeX fallback text.
pub(super) fn expr_starts_with_environment_fallback(expr: &Expr) -> bool {
    match expr {
        Expr::Environment {
            kind: EnvironmentKind::Aligned | EnvironmentKind::Split,
            ..
        } => true,
        Expr::Style { content, .. } => expr_starts_with_environment_fallback(content),
        Expr::Sequence(items) => items
            .first()
            .is_some_and(expr_starts_with_environment_fallback),
        _ => false,
    }
}

/// Return true when MathType starts a line with raw unsupported TeX text.
pub(super) fn expr_starts_with_raw_tex(expr: &Expr) -> bool {
    match expr {
        Expr::RawTex(_) => true,
        Expr::Style { content, .. } => expr_starts_with_raw_tex(content),
        Expr::Sequence(items) => items.first().is_some_and(expr_starts_with_raw_tex),
        Expr::Script { base, .. } => expr_starts_with_raw_tex(base),
        _ => false,
    }
}

/// Return true when a line starts with a big-symbol command carrying scripts.
pub(super) fn expr_starts_with_big_symbol_script_base(expr: &Expr) -> bool {
    match expr {
        Expr::Script { base, .. } => matches!(base.as_ref(), Expr::BigSymbol(_)),
        Expr::Style { content, .. } => expr_starts_with_big_symbol_script_base(content),
        Expr::Sequence(items) => items
            .first()
            .is_some_and(expr_starts_with_big_symbol_script_base),
        _ => false,
    }
}

/// Return true when MathType lets an explicit accent template choose its own first color.
pub(super) fn expr_starts_with_explicit_accent_template(expr: &Expr) -> bool {
    match expr {
        Expr::Accent { kind, .. } => matches!(
            kind,
            AccentKind::Acute | AccentKind::Grave | AccentKind::Check
        ),
        Expr::Style { content, .. } => expr_starts_with_explicit_accent_template(content),
        Expr::Sequence(items) => items
            .first()
            .is_some_and(expr_starts_with_explicit_accent_template),
        _ => false,
    }
}

/// Return true when a line/sequence item starts with a tmSUMOP scripted glyph.
pub(super) fn expr_starts_with_sum_operator_script_base(expr: &Expr) -> bool {
    match expr {
        Expr::Script { base, .. } => matches!(base.as_ref(), Expr::SumOperatorSymbol(_)),
        Expr::Style { content, .. } => expr_starts_with_sum_operator_script_base(content),
        Expr::Sequence(items) => items
            .first()
            .is_some_and(expr_starts_with_sum_operator_script_base),
        _ => false,
    }
}

/// Return true when a bodyless big operator with limits owns the first color/template order.
pub(super) fn expr_starts_with_bodyless_big_op_script(expr: &Expr) -> bool {
    match expr {
        Expr::BigOp {
            body: None,
            lower,
            upper,
            ..
        } => lower.is_some() || upper.is_some(),
        Expr::Style { content, .. } => expr_starts_with_bodyless_big_op_script(content),
        Expr::Sequence(items) => items
            .first()
            .is_some_and(expr_starts_with_bodyless_big_op_script),
        _ => false,
    }
}

/// Return true when MathType prefixes a standalone large glyph with a line marker.
pub(super) fn expr_starts_with_standalone_big_glyph(expr: &Expr) -> bool {
    match expr {
        Expr::BigSymbol(_) | Expr::SumOperatorSymbol(_) => true,
        Expr::Style { content, .. } => expr_starts_with_standalone_big_glyph(content),
        Expr::Sequence(items) => items
            .first()
            .is_some_and(expr_starts_with_standalone_big_glyph),
        _ => false,
    }
}

/// Return true when a slotless BigOp branch must own its marker/color order.
pub(super) fn expr_starts_with_slotless_big_op(expr: &Expr) -> bool {
    match expr {
        Expr::BigOp {
            lower: None,
            upper: None,
            body: None,
            ..
        } => true,
        Expr::Style { content, .. } => expr_starts_with_slotless_big_op(content),
        Expr::Sequence(items) => items.first().is_some_and(expr_starts_with_slotless_big_op),
        _ => false,
    }
}

/// Return true when MathType emits Euclid Math One before the first line def.
pub(super) fn expr_starts_with_euclid_math_one(expr: &Expr) -> bool {
    match expr {
        Expr::Char(ch) | Expr::BigSymbol(ch) | Expr::SumOperatorSymbol(ch) => {
            encoding::special_char(*ch)
                .is_some_and(|entry| entry.explicit_font == Some(ExplicitFont::EuclidMathOne))
        }
        Expr::CommandSymbol { command, ch } => encoding::command_specific_char(command)
            .map(|entry| entry.explicit_font)
            .or_else(|| encoding::special_char(*ch).map(|entry| entry.explicit_font))
            .is_some_and(|font| font == Some(ExplicitFont::EuclidMathOne)),
        Expr::Font {
            kind: FontKind::MathCal | FontKind::MathScr,
            content,
        } => first_plain_char(content).is_some_and(|ch| {
            encoding::mathcal_char(ch).is_ok_and(|entry| {
                entry.font_pos.is_some() && entry.typeface == EXPLICIT_FONT_NEG_1
            })
        }),
        // Accent templates such as `\hat{\mathcal O}` open their own TMPL bytes first,
        // so their nested explicit font definitions must stay inside the accent slot.
        Expr::Font { content, .. } | Expr::Style { content, .. } => {
            expr_starts_with_euclid_math_one(content)
        }
        Expr::Sequence(items) => items.first().is_some_and(expr_starts_with_euclid_math_one),
        Expr::Script { base, .. } => expr_starts_with_euclid_math_one(base),
        _ => false,
    }
}

/// Return true when MathType emits Euclid Math Two before the first line color.
pub(super) fn expr_starts_with_euclid_math_two(expr: &Expr) -> bool {
    match expr {
        Expr::Char(ch) | Expr::BigSymbol(ch) | Expr::SumOperatorSymbol(ch) => {
            encoding::special_char(*ch)
                .is_some_and(|entry| entry.explicit_font == Some(ExplicitFont::EuclidMathTwo))
        }
        Expr::CommandSymbol { command, ch } => encoding::command_specific_char(command)
            .map(|entry| entry.explicit_font)
            .or_else(|| encoding::special_char(*ch).map(|entry| entry.explicit_font))
            .is_some_and(|font| font == Some(ExplicitFont::EuclidMathTwo)),
        Expr::Font {
            kind: FontKind::MathBb,
            content,
        } => first_plain_char(content).is_some_and(|ch| {
            encoding::mathbb_char(ch).is_ok_and(|entry| {
                entry.font_pos.is_some() && entry.typeface == EXPLICIT_FONT_NEG_1
            })
        }),
        // Keep explicit Euclid fonts inside accent templates instead of pulling
        // them ahead of the opening accent TMPL record.
        Expr::Font { content, .. } | Expr::Style { content, .. } => {
            expr_starts_with_euclid_math_two(content)
        }
        Expr::Sequence(items) => items.first().is_some_and(expr_starts_with_euclid_math_two),
        Expr::Script { base, .. } => expr_starts_with_euclid_math_two(base),
        _ => false,
    }
}

/// Return true when MathType emits Euclid Fraktur before selecting line color.
pub(super) fn expr_starts_with_euclid_fraktur(expr: &Expr) -> bool {
    match expr {
        Expr::Font {
            kind: FontKind::MathFrak,
            content,
        } => first_plain_char(content).is_some_and(|ch| {
            encoding::mathfrak_char(ch).is_ok_and(|entry| {
                entry.font_pos.is_some() && entry.typeface == EXPLICIT_FONT_NEG_1
            })
        }),
        Expr::Style { content, .. } => expr_starts_with_euclid_fraktur(content),
        Expr::Sequence(items) => items.first().is_some_and(expr_starts_with_euclid_fraktur),
        Expr::Script { base, .. } => expr_starts_with_euclid_fraktur(base),
        _ => false,
    }
}

/// Return true for algorithm-indent formulas that contain only spacing commands.
pub(super) fn expr_is_only_spaces(expr: &Expr) -> bool {
    match expr {
        Expr::Space(_) => true,
        Expr::Style { content, .. } => expr_is_only_spaces(content),
        Expr::Sequence(items) => !items.is_empty() && items.iter().all(expr_is_only_spaces),
        _ => false,
    }
}

/// Write a pure spacing formula without color records, matching MathType output.
pub(super) fn write_only_spaces(expr: &Expr, out: &mut Vec<u8>) -> Result<(), String> {
    match expr {
        Expr::Space(width) => out.extend_from_slice(&[0x02, 0x00, FN_SPACE, *width, 0xef]),
        Expr::Sequence(items) => {
            for item in items {
                write_only_spaces(item, out)?;
            }
        }
        _ => return Err("internal error: non-space expression in write_only_spaces".to_string()),
    }
    Ok(())
}

/// Return true when a LINE starts with a layout object that already controls its own opening.
pub(super) fn expr_starts_with_line_layout_object(expr: &Expr) -> bool {
    match expr {
        Expr::Matrix { .. }
        | Expr::Pile { .. }
        | Expr::Environment { .. }
        | Expr::Stackrel { .. }
        | Expr::Underset { .. }
        | Expr::XArrow { .. } => true,
        Expr::Style { content, .. } => expr_starts_with_line_layout_object(content),
        Expr::Sequence(items) => items
            .first()
            .is_some_and(expr_starts_with_line_layout_object),
        _ => false,
    }
}

/// Return true when any nested node changes MathType's active color selection.
pub(super) fn expr_contains_color_change(expr: &Expr) -> bool {
    match expr {
        Expr::Color { .. } => true,
        Expr::Style { content, .. } => expr_contains_color_change(content),
        Expr::Sequence(items) => items.iter().any(expr_contains_color_change),
        Expr::Matrix { rows, .. } | Expr::Environment { rows, .. } => rows
            .iter()
            .flat_map(|row| row.iter())
            .any(expr_contains_color_change),
        Expr::Delimited { content, .. }
        | Expr::Font { content, .. }
        | Expr::Accent { content, .. }
        | Expr::BarTemplate { content, .. } => expr_contains_color_change(content),
        Expr::Fraction(numerator, denominator) => {
            expr_contains_color_change(numerator) || expr_contains_color_change(denominator)
        }
        Expr::Sqrt(radicand) => expr_contains_color_change(radicand),
        Expr::Script { base, sub, sup } => {
            expr_contains_color_change(base)
                || sub.as_deref().is_some_and(expr_contains_color_change)
                || sup.as_deref().is_some_and(expr_contains_color_change)
        }
        Expr::BigOp {
            body, lower, upper, ..
        } => {
            body.as_deref().is_some_and(expr_contains_color_change)
                || lower.as_deref().is_some_and(expr_contains_color_change)
                || upper.as_deref().is_some_and(expr_contains_color_change)
        }
        Expr::Pile { upper, lower, .. } | Expr::Stackrel { upper, lower } => {
            expr_contains_color_change(upper) || expr_contains_color_change(lower)
        }
        Expr::Brace {
            content,
            annotation,
            ..
        } => {
            expr_contains_color_change(content)
                || annotation
                    .as_deref()
                    .is_some_and(expr_contains_color_change)
        }
        Expr::Underset { lower, base } => {
            expr_contains_color_change(lower) || expr_contains_color_change(base)
        }
        Expr::XArrow { label, under, .. } => {
            expr_contains_color_change(label)
                || under.as_deref().is_some_and(expr_contains_color_change)
        }
        Expr::Substack { rows } | Expr::Subarray { rows, .. } => rows
            .iter()
            .flat_map(|row| row.iter())
            .any(expr_contains_color_change),
        _ => false,
    }
}

/// Return true when the first visible node is a stackrel-like template.
pub(super) fn expr_starts_with_stackrel(expr: &Expr) -> bool {
    match expr {
        Expr::Stackrel { .. } => true,
        Expr::Style { content, .. } => expr_starts_with_stackrel(content),
        Expr::Sequence(items) => items.first().is_some_and(expr_starts_with_stackrel),
        _ => false,
    }
}

/// Return true when the first visible node opens MathType's `\underset` tmLIM template.
pub(super) fn expr_starts_with_underset(expr: &Expr) -> bool {
    match expr {
        Expr::Underset { .. } => true,
        Expr::Style { content, .. } => expr_starts_with_underset(content),
        Expr::Sequence(items) => items.first().is_some_and(expr_starts_with_underset),
        _ => false,
    }
}

/// Return true when a sqrt radicand needs an explicit nth-index size restore after a fraction.
pub(super) fn expr_requires_explicit_sqrt_index_restore(expr: &Expr) -> bool {
    match expr {
        Expr::Fraction(_, denominator) => expr_ends_with_big_op(denominator),
        Expr::Style { content, .. } => expr_requires_explicit_sqrt_index_restore(content),
        Expr::Sequence(items) => items
            .last()
            .is_some_and(expr_requires_explicit_sqrt_index_restore),
        _ => false,
    }
}

/// Return true when the trailing visible node is a large-operator template.
pub(super) fn expr_ends_with_big_op(expr: &Expr) -> bool {
    match expr {
        Expr::BigOp { .. } => true,
        Expr::Style { content, .. } => expr_ends_with_big_op(content),
        Expr::Sequence(items) => items.last().is_some_and(expr_ends_with_big_op),
        _ => false,
    }
}

/// Return true when any nested visible node uses a large-operator template.
pub(super) fn expr_contains_big_op(expr: &Expr) -> bool {
    match expr {
        Expr::BigOp { .. } => true,
        Expr::Style { content, .. }
        | Expr::Font { content, .. }
        | Expr::Accent { content, .. }
        | Expr::BarTemplate { content, .. }
        | Expr::Delimited { content, .. } => expr_contains_big_op(content),
        Expr::Sequence(items) => items.iter().any(expr_contains_big_op),
        Expr::Fraction(numerator, denominator) => {
            expr_contains_big_op(numerator) || expr_contains_big_op(denominator)
        }
        Expr::Sqrt(radicand) => expr_contains_big_op(radicand),
        Expr::Script { base, sub, sup } => {
            expr_contains_big_op(base)
                || sub.as_deref().is_some_and(expr_contains_big_op)
                || sup.as_deref().is_some_and(expr_contains_big_op)
        }
        Expr::Pile { upper, lower, .. } | Expr::Stackrel { upper, lower } => {
            expr_contains_big_op(upper) || expr_contains_big_op(lower)
        }
        Expr::Brace {
            content,
            annotation,
            ..
        } => {
            expr_contains_big_op(content) || annotation.as_deref().is_some_and(expr_contains_big_op)
        }
        Expr::Underset { lower, base } => expr_contains_big_op(lower) || expr_contains_big_op(base),
        Expr::XArrow { label, under, .. } => {
            expr_contains_big_op(label) || under.as_deref().is_some_and(expr_contains_big_op)
        }
        Expr::Matrix { rows, .. }
        | Expr::Environment { rows, .. }
        | Expr::Subarray { rows, .. }
        | Expr::Substack { rows } => rows
            .iter()
            .flat_map(|row| row.iter())
            .any(expr_contains_big_op),
        _ => false,
    }
}

/// Return true when a big-operator body needs an explicit size restore before the lower slot.
pub(super) fn expr_requires_explicit_big_op_limit_restore(expr: &Expr) -> bool {
    expr_contains_big_op(expr) || expr_is_superscript_with_nested_script(expr)
}

/// Return true for superscripts whose payload itself ends in another script template.
pub(super) fn expr_is_superscript_with_nested_script(expr: &Expr) -> bool {
    match expr {
        Expr::Script {
            sub: None,
            sup: Some(sup),
            ..
        } => expr_ends_with_script(sup),
        Expr::Style { content, .. } => expr_is_superscript_with_nested_script(content),
        Expr::Sequence(items) => items
            .last()
            .is_some_and(expr_is_superscript_with_nested_script),
        _ => false,
    }
}

/// Return true when the trailing visible node is a script template.
pub(super) fn expr_ends_with_script(expr: &Expr) -> bool {
    match expr {
        Expr::Script { .. } => true,
        Expr::Style { content, .. } => expr_ends_with_script(content),
        Expr::Sequence(items) => items.last().is_some_and(expr_ends_with_script),
        _ => false,
    }
}

/// Return true when an item starts with MathType's empty-base superscript template form.
pub(super) fn expr_starts_with_empty_base_superscript(expr: &Expr) -> bool {
    match expr {
        Expr::Script { base, sub, sup } => {
            sub.is_none() && sup.is_some() && expr_is_empty_sequence(base)
        }
        Expr::Style { content, .. } => expr_starts_with_empty_base_superscript(content),
        Expr::Sequence(items) => items
            .first()
            .is_some_and(expr_starts_with_empty_base_superscript),
        _ => false,
    }
}

/// Return true when the first visible node writes its own line-opening state bytes.
pub(super) fn expr_starts_with_self_opening(expr: &Expr) -> bool {
    expr_starts_with_line_layout_object(expr)
        || expr_starts_with_line_font_def(expr)
        || expr_starts_with_split_function_name(expr)
        || expr_starts_with_raw_tex(expr)
        || expr_starts_with_explicit_accent_template(expr)
        || expr_starts_with_lim_decoration(expr)
        || expr_starts_with_sum_operator_script_base(expr)
        || expr_starts_with_empty_base_script(expr)
}

/// Return true when MathType opens a script template directly because the base is empty.
pub(super) fn expr_starts_with_empty_base_script(expr: &Expr) -> bool {
    match expr {
        Expr::Script { base, .. } => expr_is_empty_sequence(base),
        Expr::Style { content, .. } => expr_starts_with_empty_base_script(content),
        Expr::Sequence(items) => items
            .first()
            .is_some_and(expr_starts_with_empty_base_script),
        _ => false,
    }
}

/// Return true for MathType's `var*lim` wrappers, which open with their own template bytes.
pub(super) fn expr_starts_with_lim_decoration(expr: &Expr) -> bool {
    match expr {
        Expr::BarTemplate { content, .. } => expr_is_lim_function(content),
        Expr::ArrowAccent {
            kind: ArrowAccentKind::Left | ArrowAccentKind::Right,
            under: true,
            content,
        } => expr_is_lim_function(content),
        Expr::Style { content, .. } => expr_starts_with_lim_decoration(content),
        Expr::Sequence(items) => items.first().is_some_and(expr_starts_with_lim_decoration),
        _ => false,
    }
}

/// Return true when the expression is just the visible `lim` function token.
pub(super) fn expr_is_lim_function(expr: &Expr) -> bool {
    match expr {
        Expr::FunctionName(name) => name == "lim",
        Expr::Sequence(items) if items.len() == 1 => expr_is_lim_function(&items[0]),
        _ => false,
    }
}

/// Return true when a LINE contains only one bare integral sign without operands or limits.
pub(super) fn expr_starts_with_standalone_integral(expr: &Expr) -> bool {
    match expr {
        Expr::Integral { .. } => true,
        Expr::Style { content, .. } => expr_starts_with_standalone_integral(content),
        Expr::Sequence(items) => {
            items.len() == 1 && expr_starts_with_standalone_integral(&items[0])
        }
        _ => false,
    }
}

/// Return true when a sequence item starts with MathType's command-form binomial template.
pub(super) fn expr_starts_with_binom_pile(expr: &Expr) -> bool {
    match expr {
        Expr::Pile {
            kind: PileKind::Binom,
            ..
        } => true,
        Expr::Style { content, .. } => expr_starts_with_binom_pile(content),
        Expr::Sequence(items) => items.first().is_some_and(expr_starts_with_binom_pile),
        _ => false,
    }
}

/// Return true when the next sequence item begins with MathType's tmLIM template family.
pub(super) fn expr_starts_with_limit(expr: &Expr) -> bool {
    match expr {
        Expr::Limit { .. } => true,
        Expr::Style { content, .. } => expr_starts_with_limit(content),
        Expr::Sequence(items) => items.first().is_some_and(expr_starts_with_limit),
        _ => false,
    }
}

/// Return true when a function name owns its own mid-run black selector ordering.
pub(super) fn expr_starts_with_split_function_name(expr: &Expr) -> bool {
    match expr {
        Expr::FunctionName(name) => split_lim_family_function_name(name).is_some(),
        Expr::Style { content, .. } => expr_starts_with_split_function_name(content),
        Expr::Sequence(items) => items
            .first()
            .is_some_and(expr_starts_with_split_function_name),
        _ => false,
    }
}

/// Return true when the whole LINE consists of one limit template and no following siblings.
pub(super) fn expr_is_standalone_limit(expr: &Expr) -> bool {
    match expr {
        Expr::Limit { .. } => true,
        Expr::Style { content, .. } => expr_is_standalone_limit(content),
        Expr::Sequence(items) => items.len() == 1 && expr_is_standalone_limit(&items[0]),
        _ => false,
    }
}

/// Split off a leading spacing command that MathType writes before line color.
pub(super) fn leading_space_rest(expr: &Expr) -> Option<(u8, &[Expr])> {
    match expr {
        Expr::Space(width) => Some((*width, &[])),
        Expr::Sequence(items) => match items.as_slice() {
            [Expr::Space(width), rest @ ..] => Some((*width, rest)),
            _ => None,
        },
        _ => None,
    }
}

/// Split off top-level spacing, including command-produced nested sequences.
pub(super) fn top_leading_space_rest(expr: &Expr) -> Option<(u8, Vec<Expr>)> {
    match expr {
        Expr::Space(width) => Some((*width, Vec::new())),
        Expr::Sequence(items) => match items.as_slice() {
            [Expr::Space(width), rest @ ..] => Some((*width, rest.to_vec())),
            [Expr::Sequence(inner), outer @ ..] => match inner.as_slice() {
                [Expr::Space(width), inner_rest @ ..] => {
                    let mut rest = Vec::with_capacity(inner_rest.len() + outer.len());
                    rest.extend_from_slice(inner_rest);
                    rest.extend_from_slice(outer);
                    Some((*width, rest))
                }
                _ => None,
            },
            _ => None,
        },
        _ => None,
    }
}

/// Write a spacing CHAR without changing color, used only at the start of a LINE.
pub(super) fn write_space_without_color(width: u8, out: &mut Vec<u8>) {
    out.extend_from_slice(&[0x02, 0x00, FN_SPACE, width, 0xef]);
}

/// Return true when MathType emits a font definition before the line color.
pub(super) fn expr_starts_with_line_font_def(expr: &Expr) -> bool {
    match expr {
        Expr::Font {
            kind: FontKind::MathSf | FontKind::TypewriterText,
            ..
        } => true,
        Expr::Style { content, .. } => expr_starts_with_line_font_def(content),
        Expr::Sequence(items) => items.first().is_some_and(expr_starts_with_line_font_def),
        _ => false,
    }
}
