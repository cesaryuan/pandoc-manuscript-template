use mitex::convert_math;
use ratex_layout::{LayoutOptions, layout, to_display_list};
use ratex_parser::parse;
use ratex_svg::{SvgOptions, render_to_svg};
use ratex_types::math_style::MathStyle;
use typst_as_lib::{TypstEngine, typst_kit_options::TypstKitFontOptions};
use typst_layout::PagedDocument;
use typst_library::layout::{Frame, FrameItem};

const RATEX_SAFETY_PADDING_EM: f64 = 0.02;
const WORD_HALF_POINT_PT: f64 = 0.5;
const TYPST_FORMULA_LABEL: &str = "latex2wmf-formula";
const TYPST_MATH_FONT_NAME: &str = "XITS Math";
const XITS_MATH_FONT: &[u8] = include_bytes!("../assets/fonts/XITSMath-Regular.otf");

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SvgBackend {
    Ratex,
    Typst,
}

impl SvgBackend {
    /// Parse the user-facing backend name.
    pub(crate) fn parse(value: &str) -> Result<Self, String> {
        match value.trim().to_ascii_lowercase().as_str() {
            "ratex" => Ok(Self::Ratex),
            "typst" | "typst-as-lib" => Ok(Self::Typst),
            _ => Err(format!(
                "unsupported SVG backend {value:?}; expected ratex or typst"
            )),
        }
    }

    /// Return the canonical backend name stored in metadata and cache keys.
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Ratex => "ratex",
            Self::Typst => "typst",
        }
    }

    /// Return the mathematical font family used by this deterministic backend.
    pub(crate) fn math_font(self) -> &'static str {
        match self {
            Self::Ratex => "KaTeX",
            Self::Typst => TYPST_MATH_FONT_NAME,
        }
    }
}

/// Formula context controlling TeX text-style versus display-style layout.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FormulaStyle {
    Inline,
    Display,
}

impl FormulaStyle {
    /// Parse the formula context passed by the DOCX integration.
    pub(crate) fn parse(value: &str) -> Result<Self, String> {
        match value.trim().to_ascii_lowercase().as_str() {
            "inline" | "text" => Ok(Self::Inline),
            "display" | "block" => Ok(Self::Display),
            _ => Err(format!(
                "unsupported math style {value:?}; expected inline or display"
            )),
        }
    }

    /// Return the canonical name written to metadata.
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Inline => "inline",
            Self::Display => "display",
        }
    }

    /// Map the formula context to RaTeX's TeX math style.
    fn ratex_style(self) -> MathStyle {
        match self {
            Self::Inline => MathStyle::Text,
            Self::Display => MathStyle::Display,
        }
    }
}

pub(crate) struct RenderedSvg {
    pub(crate) svg: String,
    pub(crate) width_pt: f64,
    pub(crate) height_pt: f64,
    pub(crate) baseline_from_bottom_pt: f64,
    pub(crate) baseline_source: &'static str,
}

/// Render LaTeX math through the selected self-contained SVG backend.
pub(crate) fn render_formula_svg(
    latex: &str,
    font_size_pt: f64,
    backend: SvgBackend,
    formula_style: FormulaStyle,
) -> Result<RenderedSvg, String> {
    match backend {
        SvgBackend::Ratex => render_ratex_svg(latex, font_size_pt, formula_style),
        SvgBackend::Typst => render_typst_svg(latex, font_size_pt, formula_style),
    }
}

/// Remove the outer math delimiters written by the existing Python pipeline.
fn strip_math_delimiters(latex: &str) -> String {
    let mut text = latex.trim();
    if text.starts_with("$$") && text.ends_with("$$") && text.len() >= 4 {
        text = &text[2..text.len() - 2];
    } else if text.starts_with('$') && text.ends_with('$') && text.len() >= 2 {
        text = &text[1..text.len() - 1];
    }
    text.trim().to_string()
}

/// Render directly with RaTeX and retain its exact layout baseline/depth.
fn render_ratex_svg(
    latex: &str,
    font_size_pt: f64,
    formula_style: FormulaStyle,
) -> Result<RenderedSvg, String> {
    let formula = strip_math_delimiters(latex);
    let ast = parse(&formula).map_err(|err| format!("RaTeX parse error: {err}"))?;
    let layout_options = LayoutOptions::default().with_style(formula_style.ratex_style());
    let layout_box = layout(&ast, &layout_options);
    let display_list = to_display_list(&layout_box);
    let content_width_pt = display_list.width * font_size_pt;
    let content_height_pt = display_list.height * font_size_pt;
    let content_depth_pt = display_list.depth.max(0.0) * font_size_pt;
    let minimum_padding_pt = font_size_pt * RATEX_SAFETY_PADDING_EM;

    // Expand whitespace, never the formula paths, so the WMF box and its two
    // baseline-side extents land exactly on Word's half-point grid.
    let baseline_from_top_pt = ceil_to_half_point(content_height_pt + minimum_padding_pt);
    let baseline_from_bottom_pt = ceil_to_half_point(content_depth_pt + minimum_padding_pt);
    let width_pt = ceil_to_half_point(content_width_pt + 2.0 * minimum_padding_pt);
    let height_pt = baseline_from_top_pt + baseline_from_bottom_pt;
    let horizontal_padding_pt = (width_pt - content_width_pt) / 2.0;
    let top_padding_pt = baseline_from_top_pt - content_height_pt;
    let options = SvgOptions {
        font_size: font_size_pt,
        padding: 0.0,
        stroke_width: (font_size_pt / 26.6667).max(0.25),
        embed_glyphs: true,
        font_dir: String::new(),
    };
    if width_pt <= 0.0 || height_pt <= 0.0 {
        return Err("RaTeX produced an empty formula box".to_string());
    }
    let raw_svg = render_to_svg(&display_list, &options);
    Ok(RenderedSvg {
        svg: wrap_svg_with_padding(
            &raw_svg,
            width_pt,
            height_pt,
            horizontal_padding_pt,
            top_padding_pt,
        )?,
        width_pt,
        height_pt,
        baseline_from_bottom_pt,
        baseline_source: "ratex-layout-depth",
    })
}

/// Round a positive point extent upward to Word's half-point grid.
fn ceil_to_half_point(value: f64) -> f64 {
    let rounded = ((value / WORD_HALF_POINT_PT) - 1e-9).ceil() * WORD_HALF_POINT_PT;
    // Rust preserves negative zero through ceil; normalize it so JSON does not
    // report a visually confusing `-0.0pt` baseline for zero-depth glyphs.
    if rounded == 0.0 { 0.0 } else { rounded }
}

/// Rewrap a formula SVG with independent horizontal and top padding.
fn wrap_svg_with_padding(
    raw_svg: &str,
    width_pt: f64,
    height_pt: f64,
    horizontal_padding_pt: f64,
    top_padding_pt: f64,
) -> Result<String, String> {
    let body_start = raw_svg
        .find('>')
        .map(|index| index + 1)
        .ok_or_else(|| "generated formula SVG is missing its opening element".to_string())?;
    let body_end = raw_svg
        .rfind("</svg>")
        .ok_or_else(|| "generated formula SVG is missing its closing element".to_string())?;
    if body_start > body_end {
        return Err("generated formula SVG has an invalid element order".to_string());
    }
    // f64's Display form preserves round-trip precision, avoiding another
    // coordinate quantization before the final 1/20-point WMF conversion.
    let width = width_pt.to_string();
    let height = height_pt.to_string();
    let translate_x = horizontal_padding_pt.to_string();
    let translate_y = top_padding_pt.to_string();
    let body = &raw_svg[body_start..body_end];
    Ok(format!(
        // Typst outlines reuse glyphs through xlink:href, so the new root must
        // retain that namespace even though RaTeX paths do not require it.
        r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" viewBox="0 0 {width} {height}" width="{width}pt" height="{height}pt"><g transform="translate({translate_x} {translate_y})">{body}</g></svg>"#
    ))
}

/// Convert LaTeX with MiTeX, compile through typst-as-lib, and export one SVG page.
fn render_typst_svg(
    latex: &str,
    font_size_pt: f64,
    formula_style: FormulaStyle,
) -> Result<RenderedSvg, String> {
    let formula = strip_math_delimiters(latex);
    let typst_math = convert_math(&formula, None)
        .map_err(|err| format!("MiTeX LaTeX-to-Typst conversion failed: {err}"))?;
    let math_source = match formula_style {
        FormulaStyle::Inline => format!("${typst_math}$"),
        FormulaStyle::Display => format!("$ {typst_math} $"),
    };
    let source = format!(
        "#set page(width: auto, height: auto, margin: 0pt, fill: none)\n\
         #set text(size: {font_size_pt}pt)\n\
         #show math.equation: set text(font: \"{TYPST_MATH_FONT_NAME}\")\n\
         #box[{math_source}] <{TYPST_FORMULA_LABEL}>"
    );
    let engine = TypstEngine::builder()
        .main_file(source)
        // Bundle XITS Math so wheels render identically without system font lookup.
        .fonts([XITS_MATH_FONT])
        .search_fonts_with(
            TypstKitFontOptions::default()
                .include_system_fonts(false)
                .include_embedded_fonts(true),
        )
        .build();
    let compiled = engine.compile::<PagedDocument>();
    let document = compiled.output.map_err(|err| {
        format!(
            "typst-as-lib compilation failed: {err}; warnings={:?}",
            compiled.warnings
        )
    })?;
    if document.pages().len() != 1 {
        return Err(format!(
            "Typst formula compilation produced {} pages instead of one",
            document.pages().len()
        ));
    }
    let page = &document.pages()[0];
    let formula_frame = find_labeled_typst_formula_frame(&page.frame)
        .ok_or_else(|| "Typst formula frame label was not preserved during layout".to_string())?;
    if !formula_frame.has_baseline() {
        return Err("Typst formula frame does not expose a layout baseline".to_string());
    }
    let content_width_pt = formula_frame.width().to_pt();
    let content_height_pt = formula_frame.height().to_pt();
    if content_width_pt <= 0.0 || content_height_pt <= 0.0 {
        return Err("Typst produced an empty formula frame".to_string());
    }

    // Read the labelled box's actual descent before the page compositor drops
    // child baselines; this keeps Word placement tied to Typst's font layout.
    let content_depth_pt = formula_frame.descent().to_pt().max(0.0).abs();
    let baseline_from_top_pt = ceil_to_half_point(content_height_pt - content_depth_pt);
    let baseline_from_bottom_pt = ceil_to_half_point(content_depth_pt);
    let width_pt = ceil_to_half_point(content_width_pt);
    let height_pt = baseline_from_top_pt + baseline_from_bottom_pt;
    let horizontal_padding_pt = (width_pt - content_width_pt) / 2.0;
    let top_padding_pt = baseline_from_top_pt - (content_height_pt - content_depth_pt);
    let mut formula_page = page.clone();
    formula_page.frame = formula_frame.clone();
    let raw_svg = typst_svg::svg(&formula_page, &typst_svg::SvgOptions::default());
    Ok(RenderedSvg {
        // Add only transparent canvas space so Typst paths keep their scale
        // while the shared WMF dimensions remain exactly representable.
        svg: wrap_svg_with_padding(
            &raw_svg,
            width_pt,
            height_pt,
            horizontal_padding_pt,
            top_padding_pt,
        )?,
        width_pt,
        height_pt,
        baseline_from_bottom_pt,
        baseline_source: "typst-frame-descent",
    })
}

/// Find the labelled formula box that preserves Typst's layout baseline.
fn find_labeled_typst_formula_frame(frame: &Frame) -> Option<&Frame> {
    for (_, item) in frame.items() {
        let FrameItem::Group(group) = item else {
            continue;
        };
        if group
            .label
            .is_some_and(|label| label.resolve().as_str() == TYPST_FORMULA_LABEL)
        {
            return Some(&group.frame);
        }
        if let Some(found) = find_labeled_typst_formula_frame(&group.frame) {
            return Some(found);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{FormulaStyle, SvgBackend, render_formula_svg, strip_math_delimiters};

    /// Accept both inline and display delimiters produced by upstream callers.
    #[test]
    fn strips_outer_math_delimiters() {
        assert_eq!(strip_math_delimiters(" $$ x + y $$ "), "x + y");
        assert_eq!(strip_math_delimiters("$x$"), "x");
    }

    /// Prove the default backend emits outlined, non-empty SVG geometry.
    #[test]
    fn ratex_renders_formula_svg() {
        let rendered = render_formula_svg(
            r"\frac{1}{2}",
            12.0,
            SvgBackend::Ratex,
            FormulaStyle::Display,
        )
        .expect("RaTeX rendering should work");
        assert!(rendered.svg.contains("<path"));
        assert!(rendered.width_pt > 0.0);
        assert!(rendered.height_pt > rendered.baseline_from_bottom_pt);
    }

    /// Keep every backend's public dimensions exactly on the half-point grid.
    #[test]
    fn both_backends_use_half_point_formula_boxes() {
        for backend in [SvgBackend::Ratex, SvgBackend::Typst] {
            let rendered = render_formula_svg("x_i", 12.0, backend, FormulaStyle::Inline)
                .expect("formula should render on either backend");
            assert_eq!((rendered.width_pt * 2.0).fract(), 0.0);
            assert_eq!((rendered.height_pt * 2.0).fract(), 0.0);
            assert_eq!((rendered.baseline_from_bottom_pt * 2.0).fract(), 0.0);
        }
    }

    /// Use Typst's formula frame rather than a fixed em-based baseline guess.
    #[test]
    fn typst_baseline_tracks_formula_depth() {
        let simple = render_formula_svg("b", 12.0, SvgBackend::Typst, FormulaStyle::Inline)
            .expect("simple Typst formula should render");
        let fraction = render_formula_svg(
            r"\frac{x_i}{y_j}",
            12.0,
            SvgBackend::Typst,
            FormulaStyle::Display,
        )
        .expect("Typst fraction should render");

        assert_eq!(simple.baseline_source, "typst-frame-descent");
        assert!(fraction.baseline_from_bottom_pt > simple.baseline_from_bottom_pt);
    }

    /// Keep inline fractions compact instead of applying display-style layout.
    #[test]
    fn ratex_inline_fraction_is_shorter_than_display_fraction() {
        let inline = render_formula_svg(
            r"\frac{1}{2}",
            12.0,
            SvgBackend::Ratex,
            FormulaStyle::Inline,
        )
        .expect("inline fraction should render");
        let display = render_formula_svg(
            r"\frac{1}{2}",
            12.0,
            SvgBackend::Ratex,
            FormulaStyle::Display,
        )
        .expect("display fraction should render");

        assert!(inline.height_pt < display.height_pt);
    }

    /// Keep a zero-depth ascender glyph inside a padded, baseline-aware box.
    #[test]
    fn ratex_single_ascender_has_safety_depth() {
        let rendered = render_formula_svg("b", 12.0, SvgBackend::Ratex, FormulaStyle::Inline)
            .expect("single ascender should render");

        assert_eq!(rendered.width_pt, 6.0);
        assert_eq!(rendered.height_pt, 9.5);
        assert_eq!(rendered.baseline_from_bottom_pt, 0.5);
        assert!(rendered.svg.contains("viewBox=\"0 0 6 9.5\""));
    }
}
