//! Abstract syntax tree for the phase-1 core.
//!
//! Spans and comments are carried for diagnostics and formatting but are not
//! part of AST identity: compare ASTs with [`crate::dump::dump`].

use crate::diag::Span;
use crate::lexer::{Comment, IntLit};

#[derive(Clone, Debug)]
pub struct Ident {
    pub name: String,
    pub span: Span,
}

#[derive(Clone, Debug, Default)]
pub struct Comments {
    /// Own-line comments printed above the node.
    pub leading: Vec<Comment>,
    /// Comment at the end of the node's last line.
    pub trailing: Option<Comment>,
}

#[derive(Clone, Debug)]
pub struct File {
    pub items: Vec<Item>,
    /// Comments after the last item.
    pub trailing: Vec<Comment>,
}

#[derive(Clone, Debug)]
pub struct Item {
    pub comments: Comments,
    pub is_pub: bool,
    pub kind: ItemKind,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum ItemKind {
    Fn(FnDecl),
    Type(TypeDecl),
    Mod(Ident),
    Use(Vec<Ident>),
}

#[derive(Clone, Debug)]
pub enum GenericParam {
    Type(Ident),
    Const(Ident, Type),
}

#[derive(Clone, Debug)]
pub struct TypeDecl {
    pub name: Ident,
    pub generics: Vec<GenericParam>,
    pub fields: Vec<Field>,
    /// Comments before the closing `}` not attached to a field.
    pub trailing: Vec<Comment>,
}

#[derive(Clone, Debug)]
pub struct Field {
    pub comments: Comments,
    pub name: Ident,
    pub ty: Type,
}

#[derive(Clone, Debug)]
pub struct Param {
    pub name: Ident,
    pub ty: Type,
}

#[derive(Clone, Debug)]
pub struct FnDecl {
    pub name: Ident,
    pub generics: Vec<GenericParam>,
    pub params: Vec<Param>,
    pub ret: Type,
    pub effects: Vec<Ident>,
    pub body: Block,
}

#[derive(Clone, Debug)]
pub enum GenericArg {
    Type(Type),
    Int(IntLit),
}

#[derive(Clone, Debug)]
pub enum ConstExpr {
    Int(IntLit),
    Name(Ident),
}

#[derive(Clone, Debug)]
pub enum Type {
    /// Named type, including scalars (`u32`, `bool`) and generic instances.
    Named {
        path: Vec<Ident>,
        args: Vec<GenericArg>,
        span: Span,
    },
    /// `[T; N]`
    Array {
        elem: Box<Type>,
        len: ConstExpr,
        span: Span,
    },
    /// `[T]`, only valid directly under `&`.
    Slice { elem: Box<Type>, span: Span },
    /// `&T` / `&mut T`
    Ref {
        mutable: bool,
        inner: Box<Type>,
        span: Span,
    },
    /// `()`
    Unit { span: Span },
}

#[derive(Clone, Debug)]
pub struct Block {
    /// Comment after `{` on the same line.
    pub open: Option<Comment>,
    pub stmts: Vec<Stmt>,
    /// Comments before the closing `}` not attached to a statement.
    pub trailing: Vec<Comment>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct Stmt {
    pub comments: Comments,
    pub kind: StmtKind,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompoundOp {
    Add,
    Sub,
    Mul,
}

impl CompoundOp {
    pub fn as_str(self) -> &'static str {
        match self {
            CompoundOp::Add => "+=",
            CompoundOp::Sub => "-=",
            CompoundOp::Mul => "*=",
        }
    }
}

#[derive(Clone, Debug)]
pub enum StmtKind {
    Let {
        mutable: bool,
        name: Ident,
        ty: Type,
        init: Expr,
    },
    If(IfStmt),
    For {
        var: Ident,
        ty: Type,
        start: Expr,
        end: Expr,
        body: Block,
    },
    Return(Expr),
    Break,
    Continue,
    Block(Block),
    Assign {
        place: Expr,
        value: Expr,
    },
    Compound {
        op: CompoundOp,
        place: Expr,
        value: Expr,
    },
    Expr(Expr),
}

#[derive(Clone, Debug)]
pub struct IfStmt {
    pub cond: Expr,
    pub then: Block,
    pub els: Option<Else>,
}

#[derive(Clone, Debug)]
pub enum Else {
    If(Box<IfStmt>),
    Block(Block),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnOp {
    Neg,
    Not,
}

impl UnOp {
    pub fn as_str(self) -> &'static str {
        match self {
            UnOp::Neg => "-",
            UnOp::Not => "!",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
}

impl BinOp {
    pub fn as_str(self) -> &'static str {
        match self {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Div => "/",
            BinOp::Rem => "%",
            BinOp::BitAnd => "&",
            BinOp::BitOr => "|",
            BinOp::BitXor => "^",
            BinOp::Shl => "<<",
            BinOp::Shr => ">>",
            BinOp::Eq => "==",
            BinOp::Ne => "!=",
            BinOp::Lt => "<",
            BinOp::Le => "<=",
            BinOp::Gt => ">",
            BinOp::Ge => ">=",
            BinOp::And => "&&",
            BinOp::Or => "||",
        }
    }
}

/// Path in expression position: `a::b`, optionally `::<args>` at the end.
#[derive(Clone, Debug)]
pub struct ExprPath {
    pub segments: Vec<Ident>,
    pub generic_args: Option<Vec<GenericArg>>,
}

#[derive(Clone, Debug)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum ExprKind {
    Int(IntLit),
    Bool(bool),
    Unit,
    Hole(String),
    Path(ExprPath),
    StructLit {
        path: ExprPath,
        fields: Vec<(Ident, Expr)>,
    },
    Paren(Box<Expr>),
    Array(Vec<Expr>),
    Field(Box<Expr>, Ident),
    Index(Box<Expr>, Box<Expr>),
    /// `e[lo..hi]`; valid only under `&`/`&mut` (checked by the type checker).
    Slice(Box<Expr>, Box<Expr>, Box<Expr>),
    /// `&e` (`false`) or `&mut e` (`true`).
    Ref(bool, Box<Expr>),
    Call(Box<Expr>, Vec<Expr>),
    Cast(Box<Expr>, Ident),
    Unary(UnOp, Box<Expr>),
    Binary(BinOp, Box<Expr>, Box<Expr>),
}
