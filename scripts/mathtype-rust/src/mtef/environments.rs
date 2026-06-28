use super::*;

/// Write a matrix environment as a real MTEF MATRIX wrapped in its fence template.
pub(super) fn write_matrix(
    kind: MatrixKind,
    rows: &[Vec<Expr>],
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    let Some((left, right, selector)) = (match kind {
        MatrixKind::Plain => None,
        MatrixKind::Small => {
            write_size(current_size, out);
            return write_plain_matrix_record(rows, out, current_size, writer);
        }
        MatrixKind::Parenthesized => Some(('(', ')', 0x01)),
        MatrixKind::Bracketed => Some(('[', ']', 0x03)),
        MatrixKind::Braced => Some(('{', '}', 0x02)),
        MatrixKind::Barred => Some(('|', '|', 0x04)),
        MatrixKind::DoubleBarred => Some(('\u{2016}', '\u{2016}', 0x05)),
    }) else {
        return write_plain_matrix_record(rows, out, current_size, writer);
    };
    write_fenced_matrix(selector, left, right, rows, out, current_size, writer)?;
    Ok(WriteState {
        size: current_size,
        color: ColorState::Black,
    })
}

/// Write alignment-like environments using MTEF layout records, not fixture bytes.
pub(super) fn write_environment(
    kind: EnvironmentKind,
    rows: &[Vec<Expr>],
    trivia: &EnvironmentTrivia,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    match kind {
        EnvironmentKind::Array => write_array_environment(rows, trivia, out, current_size, writer),
        EnvironmentKind::Align => {
            if let Some(cell) = transparent_failure_environment_cell(rows) {
                write_expr(cell, out, current_size, writer)
            } else {
                write_align_matrix_record(rows, out, current_size, writer)
            }
        }
        EnvironmentKind::AlignAt => write_align_matrix_record(rows, out, current_size, writer),
        EnvironmentKind::Split => write_environment_fallback(
            EnvironmentFallbackSpec {
                name: "split",
                separator: "&",
                end_command: "\\end",
                rows,
                trivia,
            },
            out,
            current_size,
            writer,
        ),
        // MathType's TeX Input keeps the `aligned` wrapper as raw fallback text
        // even when the environment contains only one visible cell, so do not
        // collapse it into the inner expression here.
        EnvironmentKind::Aligned => write_environment_fallback(
            EnvironmentFallbackSpec {
                name: "aligned",
                separator: "&",
                end_command: "\\end",
                rows,
                trivia,
            },
            out,
            current_size,
            writer,
        ),
        EnvironmentKind::AlignedAt => write_align_matrix_record(rows, out, current_size, writer),
        EnvironmentKind::Gather => write_environment_fallback(
            EnvironmentFallbackSpec {
                name: "gather",
                separator: "",
                end_command: "\\end",
                rows,
                trivia,
            },
            out,
            current_size,
            writer,
        ),
        EnvironmentKind::Gathered => write_environment_fallback(
            EnvironmentFallbackSpec {
                name: "gathered",
                separator: "",
                end_command: "\\end",
                rows,
                trivia,
            },
            out,
            current_size,
            writer,
        ),
        EnvironmentKind::Cases => {
            write_left_fenced_matrix(rows, out, current_size, writer)?;
            Ok(WriteState {
                size: current_size,
                color: ColorState::Black,
            })
        }
        EnvironmentKind::RightCases => {
            write_right_fenced_matrix(rows, out, current_size, writer)?;
            Ok(WriteState {
                size: current_size,
                color: ColorState::Black,
            })
        }
    }
}

/// Write MathType''s mixed native/raw array environment form.
fn write_array_environment(
    rows: &[Vec<Expr>],
    trivia: &EnvironmentTrivia,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    write_plain_matrix_record_with_row_leading(
        rows,
        trivia.column_spec.as_deref(),
        &trivia.row_leading,
        out,
        current_size,
        writer,
    )
}

struct EnvironmentFallbackSpec<'a> {
    name: &'a str,
    separator: &'a str,
    end_command: &'a str,
    rows: &'a [Vec<Expr>],
    trivia: &'a EnvironmentTrivia,
}

/// Emulate MathType TeX Input's fallback for unsupported alignment environments.
///
/// In practice this is how we match MathType for `aligned`: keep native support
/// in our AST/MTEF writer, but use the same fallback byte pattern MathType
/// emits because its TeX Input translator does not accept `aligned` directly.
fn write_environment_fallback(
    spec: EnvironmentFallbackSpec<'_>,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    write_raw_tex_text("\\begin", out)?;
    writer.ensure_black_color_def(out);
    let mut wrote_name_color = false;
    for ch in spec.name.chars() {
        if !wrote_name_color {
            color_black(out);
            wrote_name_color = true;
        }
        write_char(ch, out, writer)?;
    }
    let mut state = WriteState {
        size: current_size,
        color: ColorState::Black,
    };
    let mut skip_fallback_end_command = false;
    for (row_index, row) in spec.rows.iter().enumerate() {
        let row_prefix = spec
            .trivia
            .row_leading
            .get(row_index)
            .map(String::as_str)
            .unwrap_or("");
        let separator_prefixes = spec
            .trivia
            .separator_leading
            .get(row_index)
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        let row_starts_with_separator =
            row.first().is_some_and(expr_is_empty_sequence) && row.len() > 1;
        for (cell_index, cell) in row.iter().enumerate() {
            if row_index > 0 && cell_index == 0 && expr_is_empty_sequence(cell) {
                continue;
            }
            let mut after_raw_separator = false;
            let needs_row_leading_prefix =
                row_index > 0 && cell_index == 0 && !row_prefix.is_empty();
            if cell_index > 0 {
                if state.size != current_size {
                    write_size(current_size, out);
                    state.size = current_size;
                }
                if state.color != ColorState::Default {
                    color_default(out);
                }
                let separator_prefix = if row_starts_with_separator && cell_index == 1 {
                    row_prefix
                } else {
                    let explicit_prefix = separator_prefixes
                        .get(cell_index.saturating_sub(1))
                        .map(String::as_str)
                        .unwrap_or("");
                    if explicit_prefix.is_empty() && cell_index == 1 {
                        row_prefix
                    } else {
                        explicit_prefix
                    }
                };
                if !separator_prefix.is_empty() {
                    write_raw_tex_text(separator_prefix, out)?;
                }
                write_raw_tex_text(spec.separator, out)?;
                after_raw_separator = true;
                if !expr_starts_with_space(cell) && !expr_starts_with_line_layout_object(cell) {
                    color_black(out);
                }
            } else if row_index > 0 && cell_index == 0 && spec.separator.is_empty() {
                if state.color != ColorState::Default {
                    color_default(out);
                }
                if !row_prefix.is_empty() {
                    write_raw_tex_text(row_prefix, out)?;
                }
                write_raw_tex_text("\\\\", out)?;
                after_raw_separator = true;
                if !expr_starts_with_space(cell) && !expr_starts_with_line_layout_object(cell) {
                    color_black(out);
                }
            } else if needs_row_leading_prefix {
                if state.color != ColorState::Default {
                    color_default(out);
                }
                write_raw_tex_text(row_prefix, out)?;
                after_raw_separator = true;
                if !expr_starts_with_space(cell) && !expr_starts_with_line_layout_object(cell) {
                    color_black(out);
                }
            }
            if !after_raw_separator
                && row_index > 0
                && state.color != ColorState::Default
                && expr_starts_with_environment_fallback(cell)
            {
                color_default(out);
                state.color = ColorState::Default;
            }
            // MathType re-selects the inherited/default color before a split
            // continuation row starts with tmLIM via `\underset{...}{\lim}`.
            // Without this row-boundary reset, the template opens under the
            // prior black selection from the previous row and the remaining
            // bytes drift out of sync.
            if spec.name == "split"
                && !after_raw_separator
                && row_index > 0
                && expr_starts_with_underset(cell)
            {
                // MathType emits an explicit default-color selector here even
                // when the logical state is already default. The byte stream is
                // therefore edge-triggered by the split-row transition, not by
                // our tracked ColorState alone.
                color_default(out);
                state.color = ColorState::Default;
            }
            let is_last_cell = row_index + 1 == spec.rows.len() && cell_index + 1 == row.len();
            if is_last_cell && matches!(spec.name, "aligned" | "split") {
                if let Some((kind, substack_rows)) = bodyless_big_op_substack_parts(cell) {
                    let trailing_end_raw = format!("{row_prefix}{}", spec.end_command);
                    let was_active = writer.fallback_environment_active;
                    writer.fallback_environment_active = true;
                    let result = write_big_op_substack_environment_fallback(
                        kind,
                        substack_rows,
                        &trailing_end_raw,
                        out,
                        current_size,
                        writer,
                    );
                    writer.fallback_environment_active = was_active;
                    state = result?;
                    skip_fallback_end_command = true;
                    continue;
                }
            }
            state = if spec.name == "split" && !after_raw_separator {
                if let Some(nested_state) =
                    write_nested_aligned_fallback(cell, out, current_size, writer)?
                {
                    nested_state
                } else if expr_starts_with_space(cell) {
                    let was_active = writer.fallback_environment_active;
                    writer.fallback_environment_active = true;
                    let result =
                        write_fallback_cell_after_default_space(cell, out, current_size, writer);
                    writer.fallback_environment_active = was_active;
                    result?
                } else {
                    let was_active = writer.fallback_environment_active;
                    writer.fallback_environment_active = true;
                    let result = write_expr(cell, out, current_size, writer);
                    writer.fallback_environment_active = was_active;
                    result?
                }
            } else if after_raw_separator && expr_starts_with_space(cell) {
                let was_active = writer.fallback_environment_active;
                writer.fallback_environment_active = true;
                let result =
                    write_fallback_cell_after_default_space(cell, out, current_size, writer);
                writer.fallback_environment_active = was_active;
                result?
            } else {
                let was_active = writer.fallback_environment_active;
                writer.fallback_environment_active = true;
                let result = write_expr(cell, out, current_size, writer);
                writer.fallback_environment_active = was_active;
                result?
            };
        }
    }
    if state.size != current_size {
        write_size(current_size, out);
    }
    if !skip_fallback_end_command {
        color_default(out);
        let end_prefix = if spec.trivia.end_leading.is_empty() {
            spec.trivia
                .row_leading
                .last()
                .map(String::as_str)
                .unwrap_or("")
        } else {
            &spec.trivia.end_leading
        };
        if !end_prefix.is_empty() {
            write_raw_tex_text(end_prefix, out)?;
        }
        write_raw_tex_text(spec.end_command, out)?;
        color_black(out);
        for ch in spec.name.chars() {
            write_char(ch, out, writer)?;
        }
        return Ok(WriteState {
            size: current_size,
            color: ColorState::Black,
        });
    }
    for ch in spec.name.chars() {
        write_char(ch, out, writer)?;
    }
    Ok(WriteState {
        size: current_size,
        color: ColorState::Black,
    })
}

/// Write nested aligned environments inside split using MathType's split fallback separators.
fn write_nested_aligned_fallback(
    expr: &Expr,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<Option<WriteState>, String> {
    let Some((rows, trivia)) = nested_aligned_rows(expr) else {
        return Ok(None);
    };
    write_environment_fallback(
        EnvironmentFallbackSpec {
            name: "aligned",
            separator: "&",
            end_command: "\\end",
            rows,
            trivia,
        },
        out,
        current_size,
        writer,
    )?;
    Ok(Some(WriteState {
        size: current_size,
        color: ColorState::Black,
    }))
}

/// Return rows plus fallback trivia for one aligned environment wrapped by split.
fn nested_aligned_rows(expr: &Expr) -> Option<(&[Vec<Expr>], &EnvironmentTrivia)> {
    match expr {
        Expr::Environment {
            kind: EnvironmentKind::Aligned,
            rows,
            trivia,
        } => Some((rows, trivia)),
        Expr::Style { content, .. } => nested_aligned_rows(content),
        Expr::Sequence(items) => match items.as_slice() {
            [item] => nested_aligned_rows(item),
            _ => None,
        },
        _ => None,
    }
}

/// Return the only visible cell when MathType treats an environment wrapper as transparent.
fn transparent_environment_cell(rows: &[Vec<Expr>]) -> Option<&Expr> {
    let [row] = rows else {
        return None;
    };
    let [cell] = row.as_slice() else {
        return None;
    };
    Some(cell)
}

/// Return the only cell when MathType collapses the whole environment into one failure message.
fn transparent_failure_environment_cell(rows: &[Vec<Expr>]) -> Option<&Expr> {
    let cell = transparent_environment_cell(rows)?;
    expr_is_translation_failed_placeholder(cell).then_some(cell)
}

/// Return true for the exact MathType failure placeholder, allowing thin wrappers.
pub(super) fn expr_is_translation_failed_placeholder(expr: &Expr) -> bool {
    match expr {
        Expr::Text(text) => text == "(Tex translation failed)",
        Expr::Style { content, .. } => expr_is_translation_failed_placeholder(content),
        Expr::Sequence(items) => {
            matches!(items.as_slice(), [item] if expr_is_translation_failed_placeholder(item))
        }
        Expr::Environment { rows, .. } => {
            transparent_environment_cell(rows).is_some_and(expr_is_translation_failed_placeholder)
        }
        _ => false,
    }
}

/// Return true when a fallback cell starts with TeX spacing such as \quad.
fn expr_starts_with_space(expr: &Expr) -> bool {
    match expr {
        Expr::Space(_) => true,
        Expr::Style { content, .. } => expr_starts_with_space(content),
        Expr::Sequence(items) => items.first().is_some_and(expr_starts_with_space),
        _ => false,
    }
}

/// Write a fallback aligned cell after raw separators already selected default color.
fn write_fallback_cell_after_default_space(
    expr: &Expr,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    if let Some((width, rest)) = leading_space_rest(expr) {
        write_space_without_color(width, out);
        if rest.is_empty() {
            return Ok(WriteState {
                size: current_size,
                color: ColorState::Default,
            });
        }
        color_black(out);
        let rest_expr = Expr::Sequence(rest.to_vec());
        return write_expr(&rest_expr, out, current_size, writer);
    }
    write_expr(expr, out, current_size, writer)
}

/// Write align rows, padding omitted leading alignment cells on continuation rows.
fn write_align_matrix_record(
    rows: &[Vec<Expr>],
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    let col_count = rows.iter().map(Vec::len).max().unwrap_or(0);
    let padded = rows
        .iter()
        .map(|row| {
            if row.len() + 1 == col_count {
                let mut padded_row = Vec::with_capacity(col_count);
                padded_row.push(Expr::Sequence(Vec::new()));
                padded_row.extend(row.iter().cloned());
                padded_row
            } else {
                row.clone()
            }
        })
        .collect::<Vec<_>>();
    write_matrix_record(&padded, out, current_size, writer)
}

/// Write a two-sided fence template whose main slot is a MATRIX record.
fn write_fenced_matrix(
    selector: u8,
    left: char,
    right: char,
    rows: &[Vec<Expr>],
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<(), String> {
    if writer.emit_top_fenced_matrix_color {
        writer.emit_top_fenced_matrix_color = false;
        writer.ensure_black_color_def(out);
        color_black(out);
    }
    out.extend_from_slice(&[0x03, 0x00, selector, 0x03, 0x00]);
    color_default(out);
    write_matrix_slot_line(rows, out, current_size, writer, MatrixHeaderStyle::Fenced)?;
    write_delimiter_glyph_pair(left, right, out)?;
    out.push(0x00);
    Ok(())
}

/// Write \begin{cases} as MathType's left-brace fence around a MATRIX.
fn write_left_fenced_matrix(
    rows: &[Vec<Expr>],
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<(), String> {
    out.extend_from_slice(&[0x03, 0x00, 0x02, 0x01, 0x00]);
    color_default(out);
    write_matrix_slot_line(rows, out, current_size, writer, MatrixHeaderStyle::Cases)?;
    write_delimiter_glyph('{', out)?;
    out.push(0x00);
    Ok(())
}

/// Write right-braced cases as a MATRIX inside a right-only fence template.
fn write_right_fenced_matrix(
    rows: &[Vec<Expr>],
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<(), String> {
    out.extend_from_slice(&[0x03, 0x00, 0x02, 0x02, 0x00]);
    color_default(out);
    write_matrix_slot_line(rows, out, current_size, writer, MatrixHeaderStyle::Cases)?;
    write_delimiter_glyph('}', out)?;
    out.push(0x00);
    Ok(())
}

/// Write a template slot line that contains only one MATRIX object.
fn write_matrix_slot_line(
    rows: &[Vec<Expr>],
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
    header_style: MatrixHeaderStyle,
) -> Result<(), String> {
    out.extend_from_slice(&[0x01, 0x00]);
    let matrix_state =
        write_matrix_record_with_header(rows, out, current_size, writer, header_style)?;
    out.push(0x00);
    if header_style == MatrixHeaderStyle::Fenced && matrix_state.size != current_size {
        write_size(current_size, out);
        color_black(out);
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum MatrixHeaderStyle {
    Cases,
    Plain,
    Fenced,
}

/// Write MathType's MATRIX record and one LINE object for each cell.
fn write_matrix_record(
    rows: &[Vec<Expr>],
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    write_matrix_record_with_header(rows, out, current_size, writer, MatrixHeaderStyle::Cases)
}

/// Write standalone matrix/array environments using MathType's centered columns.
fn write_plain_matrix_record(
    rows: &[Vec<Expr>],
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    write_matrix_record_with_header(rows, out, current_size, writer, MatrixHeaderStyle::Plain)
}

/// Write a plain array matrix while injecting probe-backed raw row controls.
fn write_plain_matrix_record_with_row_leading(
    rows: &[Vec<Expr>],
    column_spec: Option<&str>,
    row_leading: &[String],
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    let row_count = u8::try_from(rows.len()).map_err(|_| "matrix has too many rows".to_string())?;
    let col_count = rows.iter().map(Vec::len).max().unwrap_or(0);
    let col_count =
        u8::try_from(col_count).map_err(|_| "matrix has too many columns".to_string())?;

    let column_style = array_column_style(column_spec);
    out.extend_from_slice(&[0x05, 0x00, 0x01, column_style, 0x01, row_count, col_count]);
    out.extend(std::iter::repeat_n(0x00, partition_byte_count(row_count)));
    out.extend(std::iter::repeat_n(0x00, partition_byte_count(col_count)));
    let mut cell_ordinal = 0usize;
    let mut previous_cell_was_empty = false;
    let mut previous_cell_forces_default = false;
    let total_cells = rows.len() * col_count as usize;
    let mut final_cell_state = WriteState {
        size: current_size,
        color: ColorState::Default,
    };
    for (row_index, row) in rows.iter().enumerate() {
        let row_prefix = row_leading.get(row_index).map(String::as_str).unwrap_or("");
        for col_index in 0..col_count as usize {
            let prefix_cell = row_index > 0 && col_index == 0 && !row_prefix.is_empty();
            if cell_ordinal > 0
                && !previous_cell_was_empty
                && (final_cell_state.color != ColorState::Default || previous_cell_forces_default)
            {
                // Array cells inherit default color unless the previous cell left an explicit
                // non-default selection behind, so avoid emitting redundant selectors here.
                color_default(out);
            }
            let is_last_cell = cell_ordinal + 1 == total_cells;
            if let Some(cell) = row.get(col_index) {
                previous_cell_was_empty = expr_is_empty_sequence(cell);
                previous_cell_forces_default = expr_contains_color_change(cell);
                if prefix_cell {
                    let prefixed = row_prefixed_cell_expr(cell, row_prefix);
                    final_cell_state =
                        write_matrix_cell_line(&prefixed, out, current_size, writer)?;
                } else {
                    final_cell_state = write_matrix_cell_line(cell, out, current_size, writer)?;
                }
            } else {
                previous_cell_was_empty = true;
                previous_cell_forces_default = false;
                write_empty_matrix_cell_line(out);
                final_cell_state = WriteState {
                    size: current_size,
                    color: ColorState::Default,
                };
            }
            if final_cell_state.size != current_size && !is_last_cell {
                write_size(current_size, out);
            }
            cell_ordinal += 1;
        }
    }
    out.push(0x00);
    Ok(final_cell_state)
}

/// Return MathType's MATRIX column style byte for simple array column specs.
fn array_column_style(column_spec: Option<&str>) -> u8 {
    match column_spec.map(str::trim) {
        // MathType marks a one-column `{r}` array as right-aligned in the MATRIX header.
        Some("r") => 0x02,
        _ => 0x01,
    }
}

/// Merge an array row's raw prefix into the first visible cell line.
fn row_prefixed_cell_expr(cell: &Expr, prefix: &str) -> Expr {
    match cell {
        Expr::Sequence(items) => {
            let mut merged = Vec::with_capacity(items.len() + 1);
            merged.push(Expr::RawTex(prefix.to_string()));
            merged.extend(items.clone());
            Expr::Sequence(merged)
        }
        other => Expr::Sequence(vec![Expr::RawTex(prefix.to_string()), other.clone()]),
    }
}

/// Write a MATRIX record; fenced matrices use MathType's centered column style byte.
fn write_matrix_record_with_header(
    rows: &[Vec<Expr>],
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
    header_style: MatrixHeaderStyle,
) -> Result<WriteState, String> {
    let row_count = u8::try_from(rows.len()).map_err(|_| "matrix has too many rows".to_string())?;
    let col_count = rows.iter().map(Vec::len).max().unwrap_or(0);
    let col_count =
        u8::try_from(col_count).map_err(|_| "matrix has too many columns".to_string())?;

    let column_style = match header_style {
        MatrixHeaderStyle::Cases => 0x00,
        MatrixHeaderStyle::Plain => 0x01,
        MatrixHeaderStyle::Fenced => 0x01,
    };
    out.extend_from_slice(&[0x05, 0x00, 0x01, column_style, 0x01, row_count, col_count]);
    out.extend(std::iter::repeat_n(0x00, partition_byte_count(row_count)));
    out.extend(std::iter::repeat_n(0x00, partition_byte_count(col_count)));
    let mut cell_ordinal = 0usize;
    let mut previous_cell_was_empty = false;
    let mut previous_cell_forces_default = false;
    let total_cells = rows.len() * col_count as usize;
    let mut final_cell_state = WriteState {
        size: current_size,
        color: ColorState::Default,
    };
    for row in rows {
        for col_index in 0..col_count as usize {
            if header_style != MatrixHeaderStyle::Fenced
                && cell_ordinal > 0
                && !previous_cell_was_empty
                && (final_cell_state.color != ColorState::Default || previous_cell_forces_default)
            {
                // Plain matrix cells only need an explicit default-color restore when the
                // preceding cell changed the active selector.
                color_default(out);
            }
            let is_last_cell = cell_ordinal + 1 == total_cells;
            if let Some(cell) = row.get(col_index) {
                previous_cell_was_empty = expr_is_empty_sequence(cell);
                previous_cell_forces_default = expr_contains_color_change(cell);
                final_cell_state = write_matrix_cell_line(cell, out, current_size, writer)?;
            } else {
                previous_cell_was_empty = true;
                previous_cell_forces_default = false;
                write_empty_matrix_cell_line(out);
                final_cell_state = WriteState {
                    size: current_size,
                    color: ColorState::Default,
                };
            }
            if final_cell_state.size != current_size
                && !(header_style == MatrixHeaderStyle::Fenced && is_last_cell)
            {
                write_size(current_size, out);
            } else if header_style == MatrixHeaderStyle::Fenced
                && !is_last_cell
                && !previous_cell_was_empty
            {
                color_default(out);
            }
            cell_ordinal += 1;
        }
    }
    out.push(0x00);
    Ok(final_cell_state)
}

/// Return the packed two-bit partition-byte count for MATRIX dividers.
fn partition_byte_count(count: u8) -> usize {
    (count as usize + 4) / 4
}

/// Write one MATRIX cell line, preserving MathType's empty-cell form.
fn write_matrix_cell_line(
    expr: &Expr,
    out: &mut Vec<u8>,
    current_size: SizeState,
    writer: &mut MtefWriter,
) -> Result<WriteState, String> {
    if expr_is_empty_sequence(expr) {
        write_empty_matrix_cell_line(out);
        return Ok(WriteState {
            size: current_size,
            color: ColorState::Default,
        });
    }
    write_line(expr, out, current_size, writer)
}

/// Write the non-null but object-empty LINE MathType uses for empty cells.
pub(super) fn write_empty_matrix_cell_line(out: &mut Vec<u8>) {
    out.extend_from_slice(&[0x01, 0x00, 0x00]);
}

/// Return true for parser-produced empty cells in alignment environments.
pub(super) fn expr_is_empty_sequence(expr: &Expr) -> bool {
    matches!(expr, Expr::Sequence(items) if items.is_empty())
}
