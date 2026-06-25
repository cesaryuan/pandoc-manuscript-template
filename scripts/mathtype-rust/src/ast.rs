#[derive(Clone, Debug)]
pub(crate) enum Expr {
    Sequence(Vec<Expr>),
    Char(char),
    Space(u8),
    FunctionName(String),
    Text(String),
    Color {
        name: String,
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
    Binomial(Box<Expr>, Box<Expr>),
    Matrix {
        kind: MatrixKind,
        rows: Vec<Vec<Expr>>,
    },
    Environment {
        kind: EnvironmentKind,
        rows: Vec<Vec<Expr>>,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FontKind {
    Bold,
    MathCal,
    MathSf,
    MathBb,
    MathScr,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AccentKind {
    Bar,
    Hat,
    WideHat,
    Vec,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BigOpKind {
    Sum,
    Product,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum IntegralKind {
    Single,
    Double,
    Contour,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MatrixKind {
    Parenthesized,
    Bracketed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EnvironmentKind {
    Align,
    Aligned,
    Cases,
}
