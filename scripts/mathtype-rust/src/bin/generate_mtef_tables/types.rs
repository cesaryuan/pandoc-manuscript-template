#[derive(Clone, Copy)]
pub(super) struct Target {
    pub(super) name: &'static str,
    pub(super) category: Category,
    pub(super) formula: &'static str,
    pub(super) selector: Selector,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) enum Category {
    MathCal,
    MathBb,
    MathFrak,
    Special,
    RawLiteral,
    CommandSpecific,
    CommandAlias,
    SumOperatorAlias,
    Operator,
    BigOperator,
    SupportedSnippet,
}

#[derive(Clone, Copy)]
pub(super) enum Selector {
    FontPos {
        ch: char,
        typeface: u8,
        font_pos: u8,
    },
    PlainChar {
        ch: char,
        typeface: u8,
        mtcode: u16,
    },
    VisibleCharIndex {
        ch: char,
        index: usize,
    },
    RawTextRun {
        ch: char,
        index: usize,
    },
    LastChar {
        ch: char,
    },
    LastInferredChar,
    NamedMtCode {
        ch: char,
        name: &'static str,
    },
    BigOperator {
        name: &'static str,
    },
}

#[derive(Clone, Copy, Debug)]
pub(super) struct CharRecord {
    pub(super) offset: usize,
    pub(super) options: u8,
    pub(super) typeface: u8,
    pub(super) mtcode: u16,
    pub(super) font_pos: Option<u8>,
    pub(super) explicit_font: Option<ExplicitFont>,
    pub(super) font_style_selector: Option<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ExplicitFont {
    EuclidMathOne,
    EuclidMathTwo,
}

/// Return a stable category name for probe manifests.
pub(super) fn category_name(category: Category) -> &'static str {
    match category {
        Category::MathCal => "mathcal",
        Category::MathBb => "mathbb",
        Category::MathFrak => "mathfrak",
        Category::Special => "special",
        Category::RawLiteral => "raw_literal",
        Category::CommandSpecific => "command_specific",
        Category::CommandAlias => "command_alias",
        Category::SumOperatorAlias => "sum_operator_alias",
        Category::Operator => "operator",
        Category::BigOperator => "big_operator",
        Category::SupportedSnippet => "supported_snippet",
    }
}
