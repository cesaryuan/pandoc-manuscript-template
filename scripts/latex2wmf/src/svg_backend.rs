use mitex::convert_math;
use ratex_layout::{LayoutOptions, layout, to_display_list};
use ratex_parser::parse;
use ratex_svg::{SvgOptions, render_to_svg};
use ratex_types::math_style::MathStyle;
use typst_as_lib::{TypstEngine, typst_kit_options::TypstKitFontOptions};
use typst_layout::PagedDocument;

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
    let options = SvgOptions {
        font_size: font_size_pt,
        padding: 0.0,
        stroke_width: (font_size_pt / 26.6667).max(0.25),
        embed_glyphs: true,
        font_dir: String::new(),
    };
    let width_pt = display_list.width * font_size_pt;
    let height_pt = (display_list.height + display_list.depth) * font_size_pt;
    if width_pt <= 0.0 || height_pt <= 0.0 {
        return Err("RaTeX produced an empty formula box".to_string());
    }
    Ok(RenderedSvg {
        svg: render_to_svg(&display_list, &options),
        width_pt,
        height_pt,
        baseline_from_bottom_pt: (display_list.depth * font_size_pt).max(0.0),
        baseline_source: "ratex-layout-depth",
    })
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
        "#set page(width: auto, height: auto, margin: 0pt, fill: none)\n#set text(size: {font_size_pt}pt)\n{math_source}"
    );
    let engine = TypstEngine::builder()
        .main_file(source)
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
    let width_pt = page.frame.width().to_pt();
    let height_pt = page.frame.height().to_pt();
    if width_pt <= 0.0 || height_pt <= 0.0 {
        return Err("Typst produced an empty formula page".to_string());
    }

    // Typst's exported page frame does not retain the inline box baseline.
    // A 0.2-em math depth matches its default math axis closely enough for
    // Word placement while keeping the limitation explicit in the metadata.
    let baseline_from_bottom_pt = (font_size_pt * 0.2).min(height_pt);
    Ok(RenderedSvg {
        svg: typst_svg::svg(page, &typst_svg::SvgOptions::default()),
        width_pt,
        height_pt,
        baseline_from_bottom_pt,
        baseline_source: "typst-0.2em-estimate",
    })
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
}
