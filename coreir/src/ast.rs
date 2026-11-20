//! Core IR syntax (`spec/core-ir.md` §3.3).

/// An enum whose variants have fixed spellings in the text form.
macro_rules! named_enum {
    ($(#[$m:meta])* $name:ident, $table:ident { $($v:ident = $s:literal),* $(,)? }) => {
        $(#[$m])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
        pub enum $name { $($v),* }

        pub const $table: &[($name, &str)] = &[$(($name::$v, $s)),*];

        impl $name {
            pub fn name(self) -> &'static str {
                $table.iter().find(|(o, _)| *o == self).map_or("?", |(_, n)| *n)
            }

            pub fn from_name(s: &str) -> Option<$name> {
                $table.iter().find(|(_, n)| *n == s).map(|(o, _)| *o)
            }
        }
    };
}

named_enum!(Scalar, SCALARS { U32 = "u32", U64 = "u64", Usize = "usize", Bool = "bool" });

impl Scalar {
    pub fn is_int(self) -> bool {
        self != Scalar::Bool
    }

    /// Width for static checks; `usize` counts as 32 bits (ADR-0005).
    pub fn portable_bits(self) -> u32 {
        match self {
            Scalar::U64 => 64,
            Scalar::Bool => 1,
            _ => 32,
        }
    }

    /// Largest value in the portable range.
    pub fn portable_max(self) -> u64 {
        match self.portable_bits() {
            64 => u64::MAX,
            b => (1u64 << b) - 1,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Ty {
    Scalar(Scalar),
    Array(Scalar, u64),
    Slice {
        mutable: bool,
        elem: Scalar,
    },
    Unit,
    /// Spec-only unbounded integer (terms only).
    Int,
}

impl Ty {
    pub fn int_scalar(&self) -> Option<Scalar> {
        match self {
            Ty::Scalar(s) if s.is_int() => Some(*s),
            _ => None,
        }
    }

    /// Element type of an array or slice.
    pub fn elem(&self) -> Option<Scalar> {
        match self {
            Ty::Array(e, _) | Ty::Slice { elem: e, .. } => Some(*e),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Atom {
    Name(String),
    Lit(u64, Scalar),
    Bool(bool),
}

named_enum!(
    /// Binary code operations (`op.T`, §5) and bitwise spec operations (§6).
    BinOp, BIN_OPS {
        Add = "add", Sub = "sub", Mul = "mul", Div = "div", Rem = "rem",
        Band = "band", Bor = "bor", Bxor = "bxor", Shl = "shl", Shr = "shr",
        Eq = "eq", Ne = "ne", Lt = "lt", Le = "le", Gt = "gt", Ge = "ge",
    }
);

impl BinOp {
    pub fn is_bitwise(self) -> bool {
        matches!(
            self,
            BinOp::Band | BinOp::Bor | BinOp::Bxor | BinOp::Shl | BinOp::Shr
        )
    }

    pub fn is_comparison(self) -> bool {
        matches!(
            self,
            BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Rhs {
    Atom(Atom),
    Bin(BinOp, Scalar, Atom, Atom),
    /// `not.T`
    Not(Scalar, Atom),
    Len(String),
    Load(String, Atom),
    Array(Scalar, Vec<Atom>),
    View {
        mutable: bool,
        base: String,
        lo: Atom,
        hi: Atom,
    },
    Call(String, Vec<Atom>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Stmt {
    Let(String, Ty, Rhs),
    Var(String, Ty, Rhs),
    Set(String, Atom),
    Store(String, Atom, Atom),
    Call(String, Vec<Atom>),
    If(Atom, Vec<Stmt>, Vec<Stmt>),
    Loop(Box<Loop>),
    Return(Option<Atom>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Loop {
    pub head: Vec<Stmt>,
    pub cond: Atom,
    pub invariants: Vec<Term>,
    pub decreases: Term,
    pub body: Vec<Stmt>,
}

named_enum!(
    /// Spec operators on `int` and `bool` (§6).
    SpecOp, SPEC_OPS {
        Add = "+", Sub = "-", Mul = "*", Div = "/", Rem = "%",
        Eq = "=", Ne = "!=", Lt = "<", Le = "<=", Gt = ">", Ge = ">=",
        And = "and", Or = "or", Implies = "=>",
    }
);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Term {
    Name(String),
    Result,
    Lit(u64, Scalar),
    Int(i128),
    Bool(bool),
    /// `(int t)`
    Lift(Box<Term>),
    Len(Box<Term>),
    Sel(Box<Term>, Box<Term>),
    Old(Box<Term>),
    Op(SpecOp, Box<Term>, Box<Term>),
    Not(Box<Term>),
    /// Bitwise `band.T`, `bor.T`, `bxor.T`, `shl.T`, `shr.T`.
    Bit(BinOp, Scalar, Box<Term>, Box<Term>),
    /// `bnot.T`
    BitNot(Scalar, Box<Term>),
    Quant(Box<Quant>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Quant {
    pub forall: bool,
    pub var: String,
    pub lo: Term,
    pub hi: Term,
    pub body: Term,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fn {
    pub name: String,
    pub params: Vec<(String, Ty)>,
    pub ret: Ty,
    pub requires: Vec<Term>,
    pub ensures: Vec<Term>,
    pub body: Vec<Stmt>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Program {
    pub fns: Vec<Fn>,
}
