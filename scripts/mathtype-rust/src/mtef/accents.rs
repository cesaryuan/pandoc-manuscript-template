use super::*;
use crate::typeface::FN_EXPAND;
use std::collections::HashMap;

/// Write simple MathType embellishments such as \bar{I} and \hat{P}.
pub(super) fn write_accent_expr(
    kind: AccentKind,
    expr: &Expr,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    if matches!(
        kind,
        AccentKind::Acute | AccentKind::Grave | AccentKind::Check
    ) {
        return write_explicit_accent_template(kind, expr, out, current_size, writer);
    }
    if let Some((font_kind, ch)) = single_font_char(expr) {
        match (kind, font_kind) {
            (
                AccentKind::Bar
                | AccentKind::Hat
                | AccentKind::WideHat
                | AccentKind::Breve
                | AccentKind::Dot
                | AccentKind::Ddot
                | AccentKind::Dddot
                | AccentKind::Ddddot
                | AccentKind::Tilde
                | AccentKind::UnderTilde,
                None,
            ) => {
                write_embellished_char(ch, &[kind], out, writer)?;
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
            (AccentKind::Bar, Some(FontKind::Bold)) => {
                return write_bar_template(BarTemplateKind::Over, expr, out, current_size, writer);
            }
            _ => {}
        }
    }
    if kind == AccentKind::UnderTilde {
        return write_under_tilde_run(expr, out, current_size, writer);
    }
    if let Some(ch) = widehat_bar_char(expr) {
        write_embellished_char(ch, &[AccentKind::Bar, AccentKind::Hat], out, writer)?;
        return Ok(WriteState {
            size: current_size,
            color: ColorState::Black,
        });
    }
    if matches!(kind, AccentKind::Hat | AccentKind::WideHat) {
        return write_hat_template(expr, out, current_size, writer);
    }
    write_expr(expr, out, current_size, writer)
}

/// Write simple multi-character \utilde content with one documented embU_TILDE per char.
fn write_under_tilde_run(
    expr: &Expr,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    let chars = simple_char_run(expr)
        .ok_or_else(|| "\\utilde currently supports only simple character runs".to_string())?;
    for ch in chars {
        write_embellished_char(ch, &[AccentKind::UnderTilde], out, writer)?;
    }
    Ok(WriteState {
        size: current_size,
        color: ColorState::Black,
    })
}

/// Extract a parser-vetted run of plain characters for per-character embellishments.
fn simple_char_run(expr: &Expr) -> Option<Vec<char>> {
    match expr {
        Expr::Char(ch) => Some(vec![*ch]),
        Expr::Sequence(items) if !items.is_empty() => {
            let mut chars = Vec::new();
            for item in items {
                chars.extend(simple_char_run(item)?);
            }
            Some(chars)
        }
        _ => None,
    }
}

/// Write MathType's vector-arrow template for arrow accents over/under content.
pub(super) fn write_arrow_accent_template(
    kind: ArrowAccentKind,
    under: bool,
    expr: &Expr,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    if under && expr_is_lim_function(expr) {
        return write_lim_arrow_template(kind, expr, out, current_size, writer);
    }
    if !under {
        if let Some((font_kind, ch, repeat_count)) = repeated_arrow_accent_char(expr, kind, under) {
            let embellishment = match kind {
                ArrowAccentKind::Right => 0x0b,
                ArrowAccentKind::Left => 0x0c,
                ArrowAccentKind::LeftRight => 0x0d,
            };
            let embellishments =
                std::iter::repeat_n(embellishment, repeat_count).collect::<Vec<_>>();
            match font_kind {
                None => {
                    write_embellished_char_codes(ch, &embellishments, out, writer)?;
                    return Ok(WriteState {
                        size: current_size,
                        color: ColorState::Black,
                    });
                }
                Some(FontKind::RomanText) => {
                    // Bug-fix: one-character text-font vectors such as
                    // `\mathrm{\vec{a}}` stay on MathType's compact EMBELL path.
                    write_font_embellished_char_codes(
                        FontKind::RomanText,
                        ch,
                        &embellishments,
                        out,
                        writer,
                    )?;
                    return Ok(WriteState {
                        size: current_size,
                        color: ColorState::Black,
                    });
                }
                _ => {}
            }
        }
    }
    out.extend_from_slice(&[0x03, 0x00, 0x1f, arrow_accent_variation(kind, under), 0x00]);
    color_default(out);
    let line_state = write_line(expr, out, current_size, writer)?;
    if line_state.size != current_size {
        write_size(current_size, out);
    }
    if line_state.color != ColorState::Black {
        color_black(out);
    }
    write_delimiter_glyph(arrow_accent_glyph(kind, under), out)?;
    out.push(0x00);
    Ok(WriteState {
        size: current_size,
        color: ColorState::Black,
    })
}

/// Write one font-wrapped compact embellishment without opening a full accent template.
pub(super) fn write_font_embellished_char(
    kind: FontKind,
    ch: char,
    embellishment: u8,
    out: &mut Vec<u8>,
    writer: &mut MtefWriter,
) -> Result<(), String> {
    write_font_embellished_char_codes(kind, ch, &[embellishment], out, writer)
}

/// Write one font-wrapped compact embellishment run without opening a full accent template.
pub(super) fn write_font_embellished_char_codes(
    kind: FontKind,
    ch: char,
    embellishments: &[u8],
    out: &mut Vec<u8>,
    writer: &mut MtefWriter,
) -> Result<(), String> {
    let code = ch as u32;
    if code > u16::MAX as u32 {
        return Err(format!("font accent character is outside BMP: {ch}"));
    }
    match kind {
        FontKind::RomanText => {
            write_styled_table_char_with_embellishments(
                FN_TEXT,
                code as u16,
                None,
                None,
                embellishments,
                out,
                writer,
            );
            Ok(())
        }
        _ => Err(format!("unsupported font embellishment path for {kind:?}")),
    }
}

/// Return one single-character arrow-accent stack, preserving font information and repeat count.
fn repeated_arrow_accent_char(
    expr: &Expr,
    kind: ArrowAccentKind,
    under: bool,
) -> Option<(Option<FontKind>, char, usize)> {
    match expr {
        Expr::Char(ch) => Some((None, *ch, 1)),
        Expr::Sequence(items) if items.len() == 1 => {
            repeated_arrow_accent_char(&items[0], kind, under)
        }
        Expr::Font {
            kind: font_kind,
            content,
        } => repeated_arrow_accent_char(content, kind, under)
            .map(|(_, ch, count)| (Some(*font_kind), ch, count)),
        Expr::ArrowAccent {
            kind: inner_kind,
            under: inner_under,
            content,
        } if *inner_kind == kind && *inner_under == under => {
            repeated_arrow_accent_char(content, kind, under)
                .map(|(font_kind, ch, count)| (font_kind, ch, count + 1))
        }
        _ => None,
    }
}

/// Write the MathType structure used by `\varinjlim` and `\varprojlim`.
fn write_lim_arrow_template(
    kind: ArrowAccentKind,
    expr: &Expr,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    out.extend_from_slice(&[0x03, 0x00, 0x1f, arrow_accent_variation(kind, true), 0x00]);
    let line_state = write_line(expr, out, current_size, writer)?;
    if line_state.size != current_size {
        write_size(current_size, out);
    }
    color_default(out);
    let lim_arrow = match kind {
        ArrowAccentKind::Left => '\u{20d6}',
        ArrowAccentKind::Right => '\u{20d7}',
        _ => return Err("unsupported lim arrow accent".to_string()),
    };
    write_delimiter_glyph(lim_arrow, out)?;
    out.push(0x00);
    Ok(WriteState {
        size: current_size,
        // Bug-fix: `\varinjlim` / `\varprojlim` leave MathType on the
        // template's default-color path, so following inline content selects
        // black explicitly instead of inheriting it from the arrow glyph slot.
        color: ColorState::Default,
    })
}

/// Return MathType's tmVEC variation bits for an arrow accent.
fn arrow_accent_variation(kind: ArrowAccentKind, under: bool) -> u8 {
    let direction = match kind {
        ArrowAccentKind::Left => 0x01,
        ArrowAccentKind::Right => 0x02,
        ArrowAccentKind::LeftRight => 0x03,
    };
    direction | (u8::from(under) * 0x04)
}

/// Return the expanding combining glyph MathType appends after a tmVEC slot.
fn arrow_accent_glyph(kind: ArrowAccentKind, under: bool) -> char {
    match (kind, under) {
        (ArrowAccentKind::Left, false) => '\u{20d6}',
        (ArrowAccentKind::Right, false) => '\u{20d7}',
        (ArrowAccentKind::LeftRight, false) => '\u{20e1}',
        (ArrowAccentKind::Left, true) => '\u{20ee}',
        (ArrowAccentKind::Right, true) => '\u{20ef}',
        (ArrowAccentKind::LeftRight, true) => '\u{034d}',
    }
}

/// Collapse one bodyless operator into the visible glyph form template slots expect.
pub(super) fn bodyless_operator_template_content(expr: &Expr) -> Option<Expr> {
    match unwrap_single_sequence(expr) {
        Expr::BigOp {
            kind,
            lower: None,
            upper: None,
            body: None,
            ..
        } => Some(match kind {
            BigOpKind::Sum => Expr::SumOperatorSymbol('\u{2211}'),
            BigOpKind::Product => Expr::BigSymbol('\u{220f}'),
            BigOpKind::Coproduct => Expr::BigSymbol('\u{2210}'),
            BigOpKind::Union => Expr::BigSymbol('\u{22c3}'),
            BigOpKind::Intersection => Expr::BigSymbol('\u{22c2}'),
        }),
        Expr::Integral { kind } => Some(Expr::Integral { kind: *kind }),
        Expr::IntegralOp {
            kind,
            lower: None,
            upper: None,
            body: None,
            ..
        } => Some(Expr::Integral { kind: *kind }),
        _ => None,
    }
}

/// Write long overline/underline templates for multi-character content.
pub(super) fn write_bar_template(
    kind: BarTemplateKind,
    content: &Expr,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    if let Some((None, ch)) = single_font_char(content) {
        // Bug-fix: one-character overline/underline shorthands use a compact EMBELL record
        // instead of opening the full bar template that multi-character content needs.
        match kind {
            BarTemplateKind::Over => write_embellished_char(ch, &[AccentKind::Bar], out, writer)?,
            BarTemplateKind::Under => write_embellished_char_with_code(ch, 0x1d, out, writer)?,
        }
        return Ok(WriteState {
            size: current_size,
            color: ColorState::Black,
        });
    }
    let simplified_content = bodyless_operator_template_content(content);
    let content = simplified_content.as_ref().unwrap_or(content);
    let selector = match kind {
        BarTemplateKind::Under => 0x0c,
        BarTemplateKind::Over => 0x0d,
    };
    out.extend_from_slice(&[0x03, 0x00, selector, 0x00, 0x00]);
    if !expr_is_lim_function(content) {
        color_default(out);
    }
    if expr_is_empty_sequence(content) {
        // Bug-fix: MathType writes an empty overline/underline slot as a null line
        // without forcing an extra black selection on the way back out.
        write_null_line(out);
        out.push(0x00);
        return Ok(WriteState {
            size: current_size,
            color: ColorState::Default,
        });
    }
    let content_state = if let Some((prefix, followup)) = split_bar_template_raw_followup(content) {
        // Bug-fix: raw definition fallbacks split by `RawBoundary` only keep the
        // definition prefix inside the bar slot; MathType resumes the raw
        // replacement text after the template closes.
        let prefix_expr = Expr::RawTex(prefix.to_string());
        let state = write_line(&prefix_expr, out, current_size, writer)?;
        out.push(0x00);
        write_raw_tex_text(followup, out)?;
        return Ok(WriteState {
            size: state.size,
            color: ColorState::Default,
        });
    } else {
        write_line(content, out, current_size, writer)?
    };
    out.push(0x00);
    Ok(content_state)
}

/// Split a raw-definition fallback so the replacement suffix can leave the bar slot.
fn split_bar_template_raw_followup(expr: &Expr) -> Option<(&str, &str)> {
    let Expr::Sequence(items) = expr else {
        return None;
    };
    match items.as_slice() {
        [Expr::RawTex(prefix), Expr::RawBoundary, Expr::RawTex(followup)] => {
            Some((prefix.as_str(), followup.as_str()))
        }
        _ => None,
    }
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

/// Return a single plain character inside an accent payload.
pub(super) fn single_accent_char(expr: &Expr) -> Option<char> {
    match expr {
        Expr::Char(ch) => Some(*ch),
        Expr::Sequence(items) if items.len() == 1 => single_accent_char(&items[0]),
        _ => None,
    }
}

/// Write MathType's compact form for `\mathbf{\hat{C}}`-style bold accents.
pub(super) fn write_bold_embellished_char(
    ch: char,
    kind: AccentKind,
    out: &mut Vec<u8>,
) -> Result<(), String> {
    let code = ch as u32;
    if code > u16::MAX as u32 {
        return Err(format!("bold accent character is outside BMP: {ch}"));
    }
    write_table_char_with_embellishments(
        FN_VECTOR,
        code as u16,
        None,
        &[embellishment_code(kind)],
        out,
    );
    Ok(())
}

/// Return the first plain character inside a font command before emitting color.
pub(super) fn first_plain_char(expr: &Expr) -> Option<char> {
    match expr {
        Expr::Char(ch) => Some(*ch),
        Expr::Sequence(items) => items.first().and_then(first_plain_char),
        Expr::Style { content, .. } => first_plain_char(content),
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
            euclid_fraktur_defined: false,
            sans_serif_defined: false,
            euclid_math_one_typeface: EXPLICIT_FONT_NEG_1,
            euclid_math_two_typeface: EXPLICIT_FONT_NEG_2,
            euclid_fraktur_typeface: EXPLICIT_FONT_NEG_1,
            sans_serif_typeface: EXPLICIT_FONT_NEG_1,
            black_color_defined: true,
            sans_serif_group_active: false,
            typewriter_group_active: false,
            big_symbol_line_marker_pending: false,
            suppress_next_pile_color_default: false,
            suppress_next_pile_black_selector: false,
            suppress_next_stackrel_color_default: false,
            emit_top_fenced_matrix_color: false,
            suppress_next_style_restore: false,
            force_next_no_limits_integral_style_restore: false,
            suppress_next_limit_restore: false,
            emit_top_color_selector_one: false,
            next_color_selector: 1,
            named_color_selectors: HashMap::new(),
            pending_raw_follow_selector: None,
            top_sequence_starts_default: false,
            fallback_environment_active: false,
            suppress_next_line_black: false,
            line_starts_default: false,
            parent_sequence_has_previous_sibling: false,
            parent_sequence_previous_was_raw: false,
            suppress_next_marked_char_marker: false,
        },
    )?;
    out.push(0x00);
    out.extend_from_slice(&[0x02, 0x00, FN_EXPAND, 0x02, 0x03, 0x00]);
    Ok(())
}

/// Write MathType's general hat template for non-embellished content.
fn write_hat_template(
    expr: &Expr,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    out.extend_from_slice(&[0x03, 0x00, 0x21, 0x00, 0x00]);
    color_default(out);
    let line_state = write_line(expr, out, current_size, writer)?;
    if line_state.size != current_size {
        write_size(current_size, out);
    }
    if line_state.color != ColorState::Black {
        color_black(out);
    }
    out.extend_from_slice(&[0x02, 0x00, FN_EXPAND, 0x02, 0x03]);
    out.push(0x00);
    Ok(WriteState {
        size: current_size,
        color: ColorState::Black,
    })
}

/// Return MathType's EMBELL subtype for single-character accent commands.
fn embellishment_code(kind: AccentKind) -> u8 {
    match kind {
        AccentKind::Dot => 0x02,
        AccentKind::Ddot => 0x03,
        AccentKind::Dddot => 0x04,
        AccentKind::Tilde => 0x08,
        AccentKind::Hat | AccentKind::WideHat => 0x09,
        AccentKind::Bar => 0x11,
        AccentKind::Breve => 0x14,
        AccentKind::Ddddot => 0x18,
        AccentKind::UnderTilde => 0x1e,
        AccentKind::Acute | AccentKind::Grave | AccentKind::Check => {
            unreachable!("explicit accent templates do not use EMBELL records")
        }
    }
}

/// MathType's `embNOT` overlay subtype used by `\not <relation>`.
const EMBELL_NOT: u8 = 0x0a;
pub(super) const EMBELL_PRIME: u8 = 0x05;
pub(super) const EMBELL_DOUBLE_PRIME: u8 = 0x06;

/// Write a negated relation as the base relation glyph plus MathType's `embNOT`.
pub(super) fn write_not_relation(
    relation: &Expr,
    out: &mut Vec<u8>,
    writer: &mut MtefWriter,
) -> Result<(), String> {
    match relation {
        Expr::Char(ch) => write_embellished_char_with_code(*ch, EMBELL_NOT, out, writer),
        Expr::CommandSymbol { command, ch } => {
            write_command_symbol_with_embellishments(command, *ch, &[EMBELL_NOT], out, writer)
        }
        Expr::OneSidedDelimited {
            delimiter, content, ..
        } if expr_is_empty_sequence(content) => {
            // Bug-fix: MathType keeps `\not\left(\right.` on the native overlay
            // path by slashing the visible fence glyph directly.
            write_embellished_char_with_code(*delimiter, EMBELL_NOT, out, writer)
        }
        Expr::Sequence(items) if items.len() == 1 => write_not_relation(&items[0], out, writer),
        Expr::Style { content, .. } => write_not_relation(content, out, writer),
        _ => Err("internal error: unsupported relation payload in write_not_relation".to_string()),
    }
}

/// Return the visible accent glyph MathType places in the upper template slot.
fn explicit_accent_mtcode(kind: AccentKind) -> u16 {
    match kind {
        AccentKind::Acute => 0x00b4,
        AccentKind::Grave => 0x0060,
        AccentKind::Check => 0x02c7,
        _ => unreachable!("not an explicit accent template"),
    }
}

/// Write MathType's explicit accent template used by \acute, \grave, and \check.
fn write_explicit_accent_template(
    kind: AccentKind,
    expr: &Expr,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    out.extend_from_slice(&[0x03, 0x00, 0x17, 0x20, 0x00]);
    let lower_state = write_line(expr, out, current_size, writer)?;
    let stack_size = match current_size {
        SizeState::Full => SizeState::Sub,
        SizeState::Sub | SizeState::Sub2 => SizeState::Sub2,
    };
    if lower_state.size != stack_size {
        write_size(stack_size, out);
    }
    color_default(out);
    write_null_line(out);
    write_explicit_accent_line(kind, out);
    out.push(0x00);
    Ok(WriteState {
        size: stack_size,
        color: ColorState::Black,
    })
}

/// Write the upper glyph line in MathType's explicit accent template.
fn write_explicit_accent_line(kind: AccentKind, out: &mut Vec<u8>) {
    out.extend_from_slice(&[0x01, 0x00]);
    color_black(out);
    write_table_char(FN_FUNCTION, explicit_accent_mtcode(kind), None, out);
    out.push(0x00);
}

/// Write a CHAR record with one or more embellishments attached.
fn write_embellished_char(
    ch: char,
    kinds: &[AccentKind],
    out: &mut Vec<u8>,
    writer: &mut MtefWriter,
) -> Result<(), String> {
    let codes = kinds
        .iter()
        .map(|kind| embellishment_code(*kind))
        .collect::<Vec<_>>();
    write_embellished_char_codes(ch, &codes, out, writer)
}

/// Write a CHAR record with one explicitly probed EMBELL subtype attached.
pub(super) fn write_embellished_char_with_code(
    ch: char,
    embellishment: u8,
    out: &mut Vec<u8>,
    writer: &mut MtefWriter,
) -> Result<(), String> {
    write_embellished_char_codes(ch, &[embellishment], out, writer)
}

/// Write a CHAR record with one or more raw EMBELL subtype bytes attached.
pub(super) fn write_embellished_char_codes(
    ch: char,
    embellishments: &[u8],
    out: &mut Vec<u8>,
    writer: &mut MtefWriter,
) -> Result<(), String> {
    let style = resolve_char_style(ch)?;
    write_styled_table_char_with_embellishments(
        style.typeface,
        style.mtcode,
        style.font_pos,
        style.explicit_font,
        embellishments,
        out,
        writer,
    );
    Ok(())
}

/// Return true for punctuation MathType writes with the function style.
pub(super) fn is_function_char(ch: char) -> bool {
    matches!(
        ch,
        '!' | '(' | ')' | '[' | ']' | '{' | '}' | '|' | ',' | '.' | ':' | ';' | '/' | '@'
    )
}

/// Return true for BMP Unicode blocks that primarily hold math symbols.
pub(super) fn is_math_symbol_char(ch: char) -> bool {
    matches!(
        ch as u32,
        0x2030..=0x203f
            | 0x2100..=0x214f
            | 0x2190..=0x21ff
            | 0x2200..=0x22ff
            | 0x2300..=0x23ff
            | 0x2571
            | 0x2572
            | 0x25a0..=0x25ff
            | 0x2600..=0x26ff
            | 0x27c0..=0x27ff
            | 0x2900..=0x2aff
    )
}
