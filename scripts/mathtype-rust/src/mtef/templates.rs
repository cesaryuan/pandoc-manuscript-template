use super::*;

/// Write MathType's two-row pile, optionally wrapped in a delimiter pair.
pub(super) fn write_pile(
    kind: PileKind,
    upper: &Expr,
    lower: &Expr,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    if kind == PileKind::Binom {
        return write_binom_pile(upper, lower, out, current_size, writer);
    }
    let suppress_initial_color = writer.suppress_next_pile_color_default;
    if suppress_initial_color {
        writer.suppress_next_pile_color_default = false;
    }
    let delimiters = pile_delimiters(kind);
    if delimiters.is_some() {
        writer.ensure_black_color_def(out);
        color_black(out);
    } else if suppress_initial_color {
        writer.ensure_black_color_def(out);
    } else {
        color_default(out);
    }
    if let Some((left, right)) = delimiters {
        out.extend_from_slice(&[0x03, 0x00, delimiter_selector(left, right)?, 0x03, 0x00]);
        color_default(out);
    }
    out.extend_from_slice(&[0x04, 0x00, 0x02, 0x01]);
    let upper_state = write_line(upper, out, current_size, writer)?;
    if upper_state.size != current_size {
        write_size(current_size, out);
    }
    color_default(out);
    let lower_state = write_line(lower, out, current_size, writer)?;
    if lower_state.size != current_size {
        write_size(current_size, out);
    }
    out.push(0x00);
    if let Some((left, right)) = delimiters {
        write_delimiter_glyph_pair(left, right, out)?;
        out.push(0x00);
    }
    Ok(WriteState {
        size: current_size,
        color: ColorState::Default,
    })
}
/// Return the optional delimiter pair around a two-row pile.
fn pile_delimiters(kind: PileKind) -> Option<(char, char)> {
    match kind {
        PileKind::Plain | PileKind::Binom => None,
        PileKind::Parenthesized => Some(('(', ')')),
    }
}

/// Write MathType's overbrace/underbrace template with an optional annotation slot.
pub(super) fn write_brace_template(
    kind: BraceKind,
    content: &Expr,
    annotation: Option<&Expr>,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    let variation = match kind {
        BraceKind::Over => 0x01,
        BraceKind::Under => 0x00,
    };
    let glyph = match kind {
        BraceKind::Over => 0xfe37,
        BraceKind::Under => 0xfe38,
    };
    write_horizontal_fence_template(
        HorizontalFenceTemplate {
            selector: 0x18,
            variation,
            glyph,
            content,
            annotation,
        },
        out,
        current_size,
        writer,
    )
}

struct HorizontalFenceTemplate<'a> {
    selector: u8,
    variation: u8,
    glyph: u16,
    content: &'a Expr,
    annotation: Option<&'a Expr>,
}

/// Write a horizontal brace/bracket HFence template described by MathType's selector table.
fn write_horizontal_fence_template(
    template: HorizontalFenceTemplate<'_>,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    out.extend_from_slice(&[0x03, 0x00, template.selector, template.variation, 0x00]);
    color_default(out);
    let content_state = write_line(template.content, out, current_size, writer)?;
    let annotation_size = match current_size {
        SizeState::Full => SizeState::Sub,
        SizeState::Sub | SizeState::Sub2 => SizeState::Sub2,
    };
    if template.annotation.is_some() || content_state.size != annotation_size {
        write_size(annotation_size, out);
    }
    color_default(out);
    let annotation_state = if let Some(annotation) = template.annotation {
        write_line(annotation, out, annotation_size, writer)?
    } else {
        write_empty_matrix_cell_line(out);
        WriteState {
            size: annotation_size,
            color: ColorState::Default,
        }
    };
    if annotation_state.size != current_size {
        write_size(current_size, out);
    }
    if template.annotation.is_none() {
        color_black(out);
    }
    write_expanding_glyph(template.glyph, out);
    out.push(0x00);
    Ok(WriteState {
        size: current_size,
        color: ColorState::Black,
    })
}

/// Write \stackrel as MathType's above/below stacking template.
pub(super) fn write_stackrel(
    upper: &Expr,
    lower: &Expr,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    let suppress_initial_color = writer.suppress_next_stackrel_color_default;
    if suppress_initial_color {
        writer.suppress_next_stackrel_color_default = false;
    } else {
        color_default(out);
    }
    out.extend_from_slice(&[0x03, 0x00, 0x17, 0x20, 0x00]);
    let lower_state = write_line(lower, out, current_size, writer)?;
    let stack_size = match current_size {
        SizeState::Full => SizeState::Sub,
        SizeState::Sub | SizeState::Sub2 => SizeState::Sub2,
    };
    if lower_state.size != stack_size {
        write_size(stack_size, out);
    }
    color_default(out);
    write_null_line(out);
    let upper_state = write_line(upper, out, stack_size, writer)?;
    out.push(0x00);
    Ok(WriteState {
        size: upper_state.size,
        color: upper_state.color,
    })
}

/// Write \underset using MathType's tmLIM lower-slot template.
pub(super) fn write_underset(
    lower: &Expr,
    base: &Expr,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    out.extend_from_slice(&[0x03, 0x00, 0x17, 0x10, 0x00]);
    let base_state = write_line(base, out, current_size, writer)?;
    let stack_size = match current_size {
        SizeState::Full => SizeState::Sub,
        SizeState::Sub | SizeState::Sub2 => SizeState::Sub2,
    };
    if base_state.size != stack_size {
        write_size(stack_size, out);
    }
    color_default(out);
    let lower_state = write_line(lower, out, stack_size, writer)?;
    if lower_state.size != stack_size {
        write_size(stack_size, out);
    }
    color_default(out);
    write_null_line(out);
    out.push(0x00);
    Ok(WriteState {
        size: stack_size,
        color: ColorState::Default,
    })
}

/// Write MathType's extensible arrow template with an upper label slot.
pub(super) fn write_xarrow(
    kind: XArrowKind,
    label: &Expr,
    under: Option<&Expr>,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    let direction_bit = xarrow_direction_bit(kind);
    let variation = direction_bit | 0x04 | (u8::from(under.is_some()) * 0x08);
    // MathType selects black before opening tmARROW, then switches the label
    // slot back to default color inside the template.
    if !writer.parent_sequence_has_previous_sibling {
        writer.ensure_black_color_def(out);
        color_black(out);
    }
    out.extend_from_slice(&[0x03, 0x00, 0x0e, variation, 0x00]);
    let label_size = match current_size {
        SizeState::Full => SizeState::Sub,
        SizeState::Sub | SizeState::Sub2 => SizeState::Sub2,
    };
    write_size(label_size, out);
    color_default(out);
    let label_state = write_line(label, out, label_size, writer)?;
    if label_state.size != label_size {
        write_size(label_size, out);
    }
    if let Some(under) = under {
        color_default(out);
        let under_state = write_line(under, out, label_size, writer)?;
        if under_state.size != label_size {
            write_size(label_size, out);
        }
    } else {
        write_null_line(out);
    }
    if current_size != label_size {
        write_size(current_size, out);
    }
    write_expanding_glyph(xarrow_glyph(kind), out);
    out.push(0x00);
    Ok(WriteState {
        size: current_size,
        color: ColorState::Black,
    })
}

/// Return MathType's left/right variation bit for an extensible arrow command.
fn xarrow_direction_bit(kind: XArrowKind) -> u8 {
    match kind {
        XArrowKind::Left
        | XArrowKind::DoubleLeft
        | XArrowKind::HookLeft
        | XArrowKind::TwoHeadLeft => 0x10,
        XArrowKind::Right
        | XArrowKind::DoubleRight
        | XArrowKind::HookRight
        | XArrowKind::TwoHeadRight
        | XArrowKind::Mapsto
        | XArrowKind::LongEqual
        | XArrowKind::ToFrom => 0x20,
    }
}

/// Return the expandable glyph code used for one x-arrow variant.
fn xarrow_glyph(kind: XArrowKind) -> u16 {
    match kind {
        XArrowKind::Left => 0x2190,
        XArrowKind::Right => 0x2192,
        XArrowKind::DoubleLeft | XArrowKind::DoubleRight => 0x21d2,
        XArrowKind::HookLeft | XArrowKind::HookRight => 0x21aa,
        XArrowKind::TwoHeadLeft | XArrowKind::TwoHeadRight => 0x21a0,
        XArrowKind::Mapsto => 0x21a6,
        XArrowKind::LongEqual => 0x003d,
        XArrowKind::ToFrom => 0x21c4,
    }
}

/// Write a MathType fraction template with numerator and denominator slots.
pub(super) fn write_fraction(
    numerator: &Expr,
    denominator: &Expr,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    write_fraction_with_variation(0x00, numerator, denominator, out, current_size, writer)
}

/// Write one fraction template with an explicit MathType variation byte.
fn write_fraction_with_variation(
    variation: u8,
    numerator: &Expr,
    denominator: &Expr,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    write_fraction_with_variation_options(
        variation,
        numerator,
        denominator,
        out,
        current_size,
        writer,
        true,
    )
}

/// Write one fraction template while optionally suppressing the leading default-color selector.
fn write_fraction_with_variation_options(
    variation: u8,
    numerator: &Expr,
    denominator: &Expr,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
    emit_initial_default_color: bool,
) -> Result<WriteState, String> {
    out.extend_from_slice(&[0x03, 0x00, 0x0b, variation, 0x00]);
    if emit_initial_default_color {
        color_default(out);
    }
    let numerator_state = write_line(numerator, out, current_size, writer)?;
    if numerator_state.size != current_size {
        write_size(current_size, out);
        if emit_initial_default_color && numerator_state.color != ColorState::Default {
            color_default(out);
        }
    } else if emit_initial_default_color {
        color_default(out);
    }
    let denominator_state = write_line(denominator, out, current_size, writer)?;
    out.push(0x00);
    Ok(denominator_state)
}

/// Write a square-root template with a null nth-root index slot.
pub(super) fn write_sqrt(
    radicand: &Expr,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    out.extend_from_slice(&[0x03, 0x00, 0x0a, 0x00, 0x00]);
    color_default(out);
    let radicand_state = write_line(radicand, out, current_size, writer)?;
    if radicand_state.size != SizeState::Sub || expr_requires_explicit_sqrt_index_restore(radicand)
    {
        // Fractions whose denominator ends with a large-operator template can report sub size
        // without emitting the nth-root slot restore MathType writes before the empty index line.
        write_size(SizeState::Sub, out);
    }
    if radicand_state.color != ColorState::Black {
        color_black(out);
    }
    write_null_line(out);
    out.push(0x00);
    Ok(WriteState {
        size: SizeState::Sub,
        color: ColorState::Black,
    })
}

/// Write MathType's overstrike template used by cancel-like commands.
pub(super) fn write_strike_template(
    kind: StrikeKind,
    content: &Expr,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    let variation = match kind {
        StrikeKind::Horizontal => 0x01,
        StrikeKind::Up => 0x02,
        StrikeKind::Down => 0x04,
        StrikeKind::Both => 0x06,
    };
    out.extend_from_slice(&[0x03, 0x00, 0x24, variation, 0x00]);
    color_default(out);
    let content_state = write_line(content, out, current_size, writer)?;
    out.push(0x00);
    Ok(content_state)
}
