use std::collections::BTreeMap;

use lyon_path::Path;
use lyon_path::iterator::PathIterator;
use lyon_path::math::{Point, point};
use tiny_skia_path::{PathSegment, PathStroker};
use usvg::{FillRule, Node, Paint};

const PLACEABLE_KEY: u32 = 0x9AC6_CDD7;
const UNITS_PER_POINT: f64 = 20.0;
const UNITS_PER_INCH: u16 = 1440;
const MAX_LOGICAL_EXTENT: i16 = 32_000;

const META_EOF: u16 = 0x0000;
const META_SETMAPMODE: u16 = 0x0103;
const META_SETPOLYFILLMODE: u16 = 0x0106;
const META_SELECTOBJECT: u16 = 0x012D;
const META_SETWINDOWORG: u16 = 0x020B;
const META_SETWINDOWEXT: u16 = 0x020C;
const META_CREATEPENINDIRECT: u16 = 0x02FA;
const META_CREATEBRUSHINDIRECT: u16 = 0x02FC;
const META_POLYPOLYGON: u16 = 0x0538;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Rgb {
    red: u8,
    green: u8,
    blue: u8,
}

#[derive(Debug)]
struct Shape {
    contours: Vec<Vec<Point>>,
    fill_rule: FillRule,
    color: Rgb,
}

/// Convert the path-only formula subset of SVG into an Aldus placeable WMF.
pub(crate) fn svg_to_wmf(svg: &str, width_pt: f64, height_pt: f64) -> Result<Vec<u8>, String> {
    validate_dimensions(width_pt, height_pt)?;
    let tree = usvg::Tree::from_str(svg, &usvg::Options::default())
        .map_err(|err| format!("failed to parse generated SVG: {err}"))?;
    let mut shapes = Vec::new();
    collect_group_shapes(tree.root(), &mut shapes)?;
    if shapes.is_empty() {
        return Err("generated SVG contains no supported vector paths".to_string());
    }

    let canvas_width = tree.size().width() as f64;
    let canvas_height = tree.size().height() as f64;
    if canvas_width <= 0.0 || canvas_height <= 0.0 {
        return Err("generated SVG has an empty viewport".to_string());
    }
    let width_units = logical_extent(width_pt)?;
    let height_units = logical_extent(height_pt)?;
    let scale_x = f64::from(width_units) / canvas_width;
    let scale_y = f64::from(height_units) / canvas_height;
    serialize_wmf(&shapes, width_units, height_units, scale_x, scale_y)
}

/// Reject dimensions that cannot be represented by placeable WMF bounds.
fn validate_dimensions(width_pt: f64, height_pt: f64) -> Result<(), String> {
    if !width_pt.is_finite() || !height_pt.is_finite() || width_pt <= 0.0 || height_pt <= 0.0 {
        return Err("formula dimensions must be positive finite point values".to_string());
    }
    logical_extent(width_pt)?;
    logical_extent(height_pt)?;
    Ok(())
}

/// Convert a point extent into the fixed 1/20-point WMF coordinate space.
fn logical_extent(points: f64) -> Result<i16, String> {
    let logical = (points * UNITS_PER_POINT).ceil();
    if logical > f64::from(MAX_LOGICAL_EXTENT) {
        return Err(format!(
            "formula extent {points:.2}pt exceeds the WMF logical-coordinate limit"
        ));
    }
    Ok((logical as i16).max(1))
}

/// Recursively collect supported formula paths and reject raster-only SVG features.
fn collect_group_shapes(group: &usvg::Group, shapes: &mut Vec<Shape>) -> Result<(), String> {
    if group.clip_path().is_some()
        || group.mask().is_some()
        || !group.filters().is_empty()
        || group.opacity() != usvg::Opacity::ONE
    {
        return Err("SVG groups with clipping, masks, filters, or opacity are outside the formula WMF subset".to_string());
    }
    for node in group.children() {
        match node {
            Node::Group(child) => collect_group_shapes(child, shapes)?,
            Node::Path(path) => collect_path_shapes(path, shapes)?,
            Node::Image(_) => {
                return Err("SVG images are outside the formula WMF subset".to_string());
            }
            Node::Text(_) => {
                return Err("SVG text must be outlined before WMF conversion".to_string());
            }
        }
    }
    Ok(())
}

/// Convert one normalized SVG path's fill and stroke into filled WMF shapes.
fn collect_path_shapes(path: &usvg::Path, shapes: &mut Vec<Shape>) -> Result<(), String> {
    if !path.is_visible() {
        return Ok(());
    }
    let transform = path.abs_transform();
    if let Some(fill) = path.fill() {
        let color = solid_color(fill.paint(), fill.opacity())?;
        let lyon_path = to_lyon_path(path.data(), transform)?;
        let contours = flatten_contours(&lyon_path, 0.05);
        if !contours.is_empty() {
            shapes.push(Shape {
                contours,
                fill_rule: fill.rule(),
                color,
            });
        }
    }
    if let Some(stroke) = path.stroke() {
        let color = solid_color(stroke.paint(), stroke.opacity())?;
        let outline = PathStroker::new()
            .stroke(path.data(), &stroke.to_tiny_skia(), 1.0)
            .ok_or_else(|| "failed to outline an SVG formula stroke".to_string())?;
        let lyon_path = to_lyon_path(&outline, transform)?;
        let contours = flatten_contours(&lyon_path, 0.05);
        if !contours.is_empty() {
            shapes.push(Shape {
                contours,
                fill_rule: FillRule::NonZero,
                color,
            });
        }
    }
    Ok(())
}

/// Accept only opaque solid colors because WMF has no alpha or gradient model.
fn solid_color(paint: &Paint, opacity: usvg::Opacity) -> Result<Rgb, String> {
    if opacity != usvg::Opacity::ONE {
        return Err("transparent SVG paint is outside the formula WMF subset".to_string());
    }
    match paint {
        Paint::Color(color) => Ok(Rgb {
            red: color.red,
            green: color.green,
            blue: color.blue,
        }),
        _ => Err("SVG gradients and patterns are outside the formula WMF subset".to_string()),
    }
}

/// Translate normalized tiny-skia segments into a transformed lyon path.
fn to_lyon_path(
    source: &tiny_skia_path::Path,
    transform: tiny_skia_path::Transform,
) -> Result<Path, String> {
    let mut builder = Path::builder();
    let mut active = false;
    for segment in source.segments() {
        match segment {
            PathSegment::MoveTo(value) => {
                if active {
                    builder.end(false);
                }
                builder.begin(mapped_point(value, transform));
                active = true;
            }
            PathSegment::LineTo(value) => {
                ensure_active(active)?;
                builder.line_to(mapped_point(value, transform));
            }
            PathSegment::QuadTo(control, end) => {
                ensure_active(active)?;
                builder.quadratic_bezier_to(
                    mapped_point(control, transform),
                    mapped_point(end, transform),
                );
            }
            PathSegment::CubicTo(control1, control2, end) => {
                ensure_active(active)?;
                builder.cubic_bezier_to(
                    mapped_point(control1, transform),
                    mapped_point(control2, transform),
                    mapped_point(end, transform),
                );
            }
            PathSegment::Close => {
                ensure_active(active)?;
                builder.end(true);
                active = false;
            }
        }
    }
    if active {
        builder.end(false);
    }
    Ok(builder.build())
}

/// Guard malformed paths whose drawing segment appears before a move command.
fn ensure_active(active: bool) -> Result<(), String> {
    if active {
        Ok(())
    } else {
        Err("SVG path segment appeared before its initial move command".to_string())
    }
}

/// Apply a complete SVG transform before paths are flattened for WMF.
fn mapped_point(value: tiny_skia_path::Point, transform: tiny_skia_path::Transform) -> Point {
    let mut value = value;
    transform.map_point(&mut value);
    point(value.x, value.y)
}

/// Flatten Bézier curves into the polygon contours supported by classic WMF.
fn flatten_contours(path: &Path, tolerance: f32) -> Vec<Vec<Point>> {
    let mut contours = Vec::new();
    let mut current = Vec::new();
    for event in path.iter().flattened(tolerance) {
        match event {
            lyon_path::Event::Begin { at } => current.push(at),
            lyon_path::Event::Line { to, .. } => current.push(to),
            lyon_path::Event::End { first, close, .. } => {
                if close && current.last().copied() != Some(first) {
                    current.push(first);
                }
                deduplicate_adjacent(&mut current);
                if current.len() >= 3 {
                    contours.push(std::mem::take(&mut current));
                } else {
                    current.clear();
                }
            }
            lyon_path::Event::Quadratic { .. } | lyon_path::Event::Cubic { .. } => {
                unreachable!("flattened lyon paths contain only line events")
            }
        }
    }
    contours
}

/// Remove duplicate adjacent vertices that can make Word reject a polygon.
fn deduplicate_adjacent(points: &mut Vec<Point>) {
    points.dedup_by(|left, right| left.x == right.x && left.y == right.y);
    if points.len() > 1 && points.first() == points.last() {
        points.pop();
    }
}

/// Serialize collected shapes into a placeable WMF with explicit window mapping.
fn serialize_wmf(
    shapes: &[Shape],
    width: i16,
    height: i16,
    scale_x: f64,
    scale_y: f64,
) -> Result<Vec<u8>, String> {
    let mut colors = BTreeMap::new();
    for shape in shapes {
        let next_index = 1_u16
            .checked_add(colors.len() as u16)
            .ok_or_else(|| "too many WMF brush colors".to_string())?;
        colors.entry(shape.color).or_insert(next_index);
    }

    let mut records = Vec::new();
    push_record(&mut records, META_SETMAPMODE, &[8]);
    push_record(&mut records, META_SETWINDOWORG, &[0, 0]);
    push_record(
        &mut records,
        META_SETWINDOWEXT,
        &[height as u16, width as u16],
    );
    // Object 0 is a null pen, ensuring adjacent filled contours have no outline.
    push_record(&mut records, META_CREATEPENINDIRECT, &[5, 0, 0, 0, 0]);
    push_record(&mut records, META_SELECTOBJECT, &[0]);
    for color in colors.keys() {
        let colorref =
            u32::from(color.red) | (u32::from(color.green) << 8) | (u32::from(color.blue) << 16);
        push_record(
            &mut records,
            META_CREATEBRUSHINDIRECT,
            &[0, colorref as u16, (colorref >> 16) as u16, 0],
        );
    }

    for shape in shapes {
        let fill_mode = match shape.fill_rule {
            FillRule::EvenOdd => 1,
            FillRule::NonZero => 2,
        };
        push_record(&mut records, META_SETPOLYFILLMODE, &[fill_mode]);
        push_record(
            &mut records,
            META_SELECTOBJECT,
            &[*colors.get(&shape.color).expect("shape color was indexed")],
        );
        let parameters = polygon_parameters(&shape.contours, scale_x, scale_y)?;
        push_record(&mut records, META_POLYPOLYGON, &parameters);
    }
    push_record(&mut records, META_EOF, &[]);

    let max_record_words = max_record_words(&records)?;
    let file_size_words = 9_u32
        .checked_add((records.len() / 2) as u32)
        .ok_or_else(|| "WMF file size overflow".to_string())?;
    let mut output = placeable_header(width, height);
    push_u16(&mut output, 1);
    push_u16(&mut output, 9);
    push_u16(&mut output, 0x0300);
    push_u32(&mut output, file_size_words);
    push_u16(&mut output, (colors.len() + 1) as u16);
    push_u32(&mut output, max_record_words);
    push_u16(&mut output, 0);
    output.extend_from_slice(&records);
    Ok(output)
}

/// Encode a WMF PolyPolygon parameter block from scaled SVG contours.
fn polygon_parameters(
    contours: &[Vec<Point>],
    scale_x: f64,
    scale_y: f64,
) -> Result<Vec<u16>, String> {
    let polygon_count = u16::try_from(contours.len())
        .map_err(|_| "too many contours in one SVG path".to_string())?;
    let mut parameters = Vec::new();
    parameters.push(polygon_count);
    for contour in contours {
        parameters.push(
            u16::try_from(contour.len())
                .map_err(|_| "too many vertices in one SVG contour".to_string())?,
        );
    }
    for contour in contours {
        for vertex in contour {
            parameters.push(scaled_coordinate(f64::from(vertex.x) * scale_x)? as u16);
            parameters.push(scaled_coordinate(f64::from(vertex.y) * scale_y)? as u16);
        }
    }
    Ok(parameters)
}

/// Round one SVG coordinate into the signed 16-bit WMF range.
fn scaled_coordinate(value: f64) -> Result<i16, String> {
    if !value.is_finite() || value < f64::from(i16::MIN) || value > f64::from(i16::MAX) {
        return Err(format!("SVG coordinate {value} exceeds the WMF range"));
    }
    Ok(value.round() as i16)
}

/// Append one standard WMF record and its word-sized parameter array.
fn push_record(output: &mut Vec<u8>, function: u16, parameters: &[u16]) {
    push_u32(output, (3 + parameters.len()) as u32);
    push_u16(output, function);
    for parameter in parameters {
        push_u16(output, *parameter);
    }
}

/// Scan serialized records for the standard-header max-record field.
fn max_record_words(records: &[u8]) -> Result<u32, String> {
    let mut offset = 0;
    let mut maximum = 0;
    while offset + 6 <= records.len() {
        let words = u32::from_le_bytes(
            records[offset..offset + 4]
                .try_into()
                .expect("record size slice has four bytes"),
        );
        if words < 3 {
            return Err("generated an invalid WMF record size".to_string());
        }
        maximum = maximum.max(words);
        offset = offset
            .checked_add(words as usize * 2)
            .ok_or_else(|| "WMF record offset overflow".to_string())?;
    }
    if offset != records.len() {
        return Err("generated a misaligned WMF record stream".to_string());
    }
    Ok(maximum)
}

/// Build the 22-byte Aldus placeable header and checksum.
fn placeable_header(width: i16, height: i16) -> Vec<u8> {
    let mut header = Vec::with_capacity(22);
    push_u32(&mut header, PLACEABLE_KEY);
    push_u16(&mut header, 0);
    push_u16(&mut header, 0);
    push_u16(&mut header, 0);
    push_u16(&mut header, width as u16);
    push_u16(&mut header, height as u16);
    push_u16(&mut header, UNITS_PER_INCH);
    push_u32(&mut header, 0);
    let checksum = header.chunks_exact(2).take(10).fold(0_u16, |value, word| {
        value ^ u16::from_le_bytes([word[0], word[1]])
    });
    push_u16(&mut header, checksum);
    header
}

/// Append a little-endian 16-bit value.
fn push_u16(output: &mut Vec<u8>, value: u16) {
    output.extend_from_slice(&value.to_le_bytes());
}

/// Append a little-endian 32-bit value.
fn push_u32(output: &mut Vec<u8>, value: u32) {
    output.extend_from_slice(&value.to_le_bytes());
}

#[cfg(test)]
mod tests {
    use super::{META_SETWINDOWEXT, META_SETWINDOWORG, PLACEABLE_KEY, svg_to_wmf};

    /// Keep the generated WMF compatible with Word's PDF replay path.
    #[test]
    fn simple_path_has_placeable_and_window_records() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="10pt" height="10pt" viewBox="0 0 10 10"><path d="M1 1 L9 1 L9 9 L1 9 Z"/></svg>"#;
        let bytes = svg_to_wmf(svg, 10.0, 10.0).expect("simple SVG should convert");
        assert_eq!(
            u32::from_le_bytes(bytes[0..4].try_into().unwrap()),
            PLACEABLE_KEY
        );
        assert!(
            bytes
                .windows(2)
                .any(|word| word == META_SETWINDOWORG.to_le_bytes())
        );
        assert!(
            bytes
                .windows(2)
                .any(|word| word == META_SETWINDOWEXT.to_le_bytes())
        );
    }

    /// Reject unsupported raster content instead of silently producing blank previews.
    #[test]
    fn images_are_rejected() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10"><image width="10" height="10" href="data:image/png;base64,iVBORw0KGgo="/></svg>"#;
        assert!(svg_to_wmf(svg, 10.0, 10.0).is_err());
    }
}
