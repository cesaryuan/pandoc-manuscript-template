#[derive(Clone, Debug)]
pub(crate) enum Expr {
    Sequence(Vec<Expr>),
    Char(char),
    /// Preserve MathType's rare "line marker + visible CHAR" form for standalone glyph hints.
    MarkedChar(char),
    CommandSymbol {
        command: String,
        ch: char,
    },
    BigSymbol(char),
    SumOperatorSymbol(char),
    RawTex(String),
    Space(u8),
    FunctionName(String),
    Text(String),
    Color {
        name: String,
        content: Box<Expr>,
    },
    Style {
        kind: StyleKind,
        content: Box<Expr>,
    },
    Font {
        kind: FontKind,
        content: Box<Expr>,
    },
    Accent {
        kind: AccentKind,
        content: Box<Expr>,
    },
    ArrowAccent {
        kind: ArrowAccentKind,
        under: bool,
        content: Box<Expr>,
    },
    BarTemplate {
        kind: BarTemplateKind,
        content: Box<Expr>,
    },
    Strike {
        kind: StrikeKind,
        content: Box<Expr>,
    },
    NotRelation(Box<Expr>),
    Fraction(Box<Expr>, Box<Expr>),
    Sqrt(Box<Expr>),
    NthRoot {
        index: Box<Expr>,
        radicand: Box<Expr>,
    },
    BigOp {
        kind: BigOpKind,
        lower: Option<Box<Expr>>,
        upper: Option<Box<Expr>>,
        body: Option<Box<Expr>>,
    },
    Limit {
        name: String,
        lower: Option<Box<Expr>>,
        upper: Option<Box<Expr>>,
    },
    Integral {
        kind: IntegralKind,
    },
    IntegralOp {
        kind: IntegralKind,
        lower: Option<Box<Expr>>,
        upper: Option<Box<Expr>>,
        body: Option<Box<Expr>>,
    },
    Pile {
        kind: PileKind,
        upper: Box<Expr>,
        lower: Box<Expr>,
    },
    Brace {
        kind: BraceKind,
        content: Box<Expr>,
        annotation: Option<Box<Expr>>,
    },
    Stackrel {
        upper: Box<Expr>,
        lower: Box<Expr>,
    },
    Underset {
        lower: Box<Expr>,
        base: Box<Expr>,
    },
    XArrow {
        kind: XArrowKind,
        label: Box<Expr>,
        under: Option<Box<Expr>>,
    },
    Substack {
        rows: Vec<Vec<Expr>>,
    },
    Subarray {
        column_spec: String,
        rows: Vec<Vec<Expr>>,
    },
    Matrix {
        kind: MatrixKind,
        rows: Vec<Vec<Expr>>,
    },
    Environment {
        kind: EnvironmentKind,
        rows: Vec<Vec<Expr>>,
        trivia: EnvironmentTrivia,
    },
    Delimited {
        left: char,
        right: char,
        content: Box<Expr>,
    },
    Script {
        base: Box<Expr>,
        sub: Option<Box<Expr>>,
        sup: Option<Box<Expr>>,
    },
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct EnvironmentTrivia {
    pub(crate) row_leading: Vec<String>,
    pub(crate) separator_leading: Vec<Vec<String>>,
    pub(crate) end_leading: String,
}

impl Expr {
    /// Return true when this expression still depends on MathType raw TeX fallback.
    #[allow(dead_code)]
    pub(crate) fn contains_raw_tex(&self) -> bool {
        match self {
            Expr::RawTex(_) => true,
            Expr::Sequence(items) => items.iter().any(Expr::contains_raw_tex),
            Expr::Color { content, .. }
            | Expr::Style { content, .. }
            | Expr::Font { content, .. }
            | Expr::Accent { content, .. }
            | Expr::ArrowAccent { content, .. }
            | Expr::BarTemplate { content, .. }
            | Expr::Strike { content, .. }
            | Expr::NotRelation(content)
            | Expr::Sqrt(content)
            | Expr::Delimited { content, .. } => content.contains_raw_tex(),
            Expr::Script { base, sub, sup } => {
                base.contains_raw_tex()
                    || sub.as_deref().is_some_and(Expr::contains_raw_tex)
                    || sup.as_deref().is_some_and(Expr::contains_raw_tex)
            }
            Expr::XArrow { label, under, .. } => {
                label.contains_raw_tex() || under.as_deref().is_some_and(Expr::contains_raw_tex)
            }
            Expr::Fraction(left, right)
            | Expr::Stackrel {
                upper: left,
                lower: right,
            }
            | Expr::Underset {
                lower: left,
                base: right,
            }
            | Expr::Pile {
                upper: left,
                lower: right,
                ..
            } => left.contains_raw_tex() || right.contains_raw_tex(),
            Expr::NthRoot { index, radicand } => {
                index.contains_raw_tex() || radicand.contains_raw_tex()
            }
            Expr::BigOp {
                lower, upper, body, ..
            }
            | Expr::IntegralOp {
                lower, upper, body, ..
            } => {
                lower.as_deref().is_some_and(Expr::contains_raw_tex)
                    || upper.as_deref().is_some_and(Expr::contains_raw_tex)
                    || body.as_deref().is_some_and(Expr::contains_raw_tex)
            }
            Expr::Limit { lower, upper, .. } => {
                lower.as_deref().is_some_and(Expr::contains_raw_tex)
                    || upper.as_deref().is_some_and(Expr::contains_raw_tex)
            }
            Expr::Brace {
                content,
                annotation,
                ..
            } => {
                content.contains_raw_tex()
                    || annotation.as_deref().is_some_and(Expr::contains_raw_tex)
            }
            Expr::Substack { rows }
            | Expr::Matrix { rows, .. }
            | Expr::Subarray { rows, .. }
            | Expr::Environment { rows, .. } => rows
                .iter()
                .flat_map(|row| row.iter())
                .any(Expr::contains_raw_tex),
            Expr::Char(_)
            | Expr::MarkedChar(_)
            | Expr::CommandSymbol { .. }
            | Expr::BigSymbol(_)
            | Expr::SumOperatorSymbol(_)
            | Expr::Space(_)
            | Expr::FunctionName(_)
            | Expr::Text(_)
            | Expr::Integral { .. } => false,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum StyleKind {
    Display,
    Text,
    Script,
    ScriptScript,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FontKind {
    Bold,
    RomanText,
    TypewriterText,
    MathCal,
    MathSf,
    MathBb,
    MathScr,
    MathFrak,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AccentKind {
    Bar,
    Hat,
    WideHat,
    Breve,
    Dot,
    Ddot,
    Dddot,
    Ddddot,
    Tilde,
    UnderTilde,
    Acute,
    Grave,
    Check,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ArrowAccentKind {
    Left,
    Right,
    LeftRight,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BarTemplateKind {
    Over,
    Under,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum StrikeKind {
    Horizontal,
    Up,
    Down,
    Both,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum XArrowKind {
    Left,
    Right,
    DoubleLeft,
    DoubleRight,
    HookLeft,
    HookRight,
    TwoHeadLeft,
    TwoHeadRight,
    Mapsto,
    LongEqual,
    ToFrom,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BigOpKind {
    Sum,
    Product,
    Coproduct,
    Union,
    Intersection,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum IntegralKind {
    Single,
    Double,
    Triple,
    Contour,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BraceKind {
    Over,
    Under,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PileKind {
    Plain,
    Parenthesized,
    Binom,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MatrixKind {
    Plain,
    Small,
    Parenthesized,
    Bracketed,
    Braced,
    Barred,
    DoubleBarred,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EnvironmentKind {
    Array,
    Align,
    Split,
    Aligned,
    AlignAt,
    AlignedAt,
    Cases,
    RightCases,
    Gather,
    Gathered,
}
