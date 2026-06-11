#[derive(Clone, Debug)]
pub(crate) enum Expr {
    Sequence(Vec<Expr>),
    Char(char),
    Space(u8),
    FunctionName(String),
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
    BigOp {
        kind: BigOpKind,
        lower: Option<Box<Expr>>,
        upper: Option<Box<Expr>>,
        body: Option<Box<Expr>>,
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
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AccentKind {
    Bar,
    Hat,
    WideHat,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BigOpKind {
    Sum,
    Product,
}
