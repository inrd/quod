//! Recursive-descent LL(1) parser for `spec/grammar-draft.md`.
//!
//! Each choice is made on the current token only. Stops at the first error.

use crate::ast::*;
use crate::diag::{Code, Diagnostic, Span};
use crate::lexer::{lex, Comment, Kw, Punct, TokKind, Token};

type PResult<T> = Result<T, Diagnostic>;

pub fn parse(src: &str) -> PResult<File> {
    let lexed = lex(src)?;
    let mut p = Parser {
        toks: lexed.tokens,
        pos: 0,
        comments: lexed.comments,
        cpos: 0,
    };
    p.file()
}

struct Parser {
    toks: Vec<Token>,
    pos: usize,
    comments: Vec<Comment>,
    cpos: usize,
}

impl Parser {
    // ---- token helpers ----

    fn peek(&self) -> &TokKind {
        &self.toks[self.pos].kind
    }

    fn span(&self) -> Span {
        self.toks[self.pos].span
    }

    fn prev_span(&self) -> Span {
        self.toks[self.pos.saturating_sub(1)].span
    }

    fn prev_line(&self) -> usize {
        self.toks[self.pos.saturating_sub(1)].line
    }

    fn bump(&mut self) -> Token {
        let t = self.toks[self.pos].clone();
        if self.pos + 1 < self.toks.len() {
            self.pos += 1;
        }
        t
    }

    fn at(&self, p: Punct) -> bool {
        *self.peek() == TokKind::Punct(p)
    }

    fn at_kw(&self, k: Kw) -> bool {
        *self.peek() == TokKind::Kw(k)
    }

    fn eat(&mut self, p: Punct) -> bool {
        if self.at(p) {
            self.bump();
            true
        } else {
            false
        }
    }

    fn eat_kw(&mut self, k: Kw) -> bool {
        if self.at_kw(k) {
            self.bump();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, p: Punct) -> PResult<Span> {
        if self.at(p) {
            Ok(self.bump().span)
        } else {
            Err(self.unexpected(&format!("`{}`", p.as_str())))
        }
    }

    fn expect_kw(&mut self, k: Kw) -> PResult<Span> {
        if self.at_kw(k) {
            Ok(self.bump().span)
        } else {
            Err(self.unexpected(&format!("`{}`", k.as_str())))
        }
    }

    /// Expect `>`, splitting `>>` or `>=` when closing generic arguments.
    fn expect_gt(&mut self) -> PResult<Span> {
        let rest = match self.peek() {
            TokKind::Punct(Punct::Gt) => return Ok(self.bump().span),
            TokKind::Punct(Punct::Shr) => Punct::Gt,
            TokKind::Punct(Punct::Ge) => Punct::Eq,
            _ => return Err(self.unexpected("`>`")),
        };
        let tok = &mut self.toks[self.pos];
        let gt = Span::new(tok.span.start, tok.span.start + 1);
        tok.kind = TokKind::Punct(rest);
        tok.span.start += 1;
        Ok(gt)
    }

    fn ident(&mut self) -> PResult<Ident> {
        match self.peek().clone() {
            TokKind::Ident(name) => {
                let span = self.bump().span;
                Ok(Ident { name, span })
            }
            _ => Err(self.unexpected("identifier")),
        }
    }

    fn unexpected(&self, expected: &str) -> Diagnostic {
        let span = self.span();
        match self.peek() {
            TokKind::Reserved(w) => Diagnostic::new(
                Code::E0202,
                span,
                format!("`{w}` is reserved for phase 2+ and not available in phase 1"),
            ),
            TokKind::Kw(Kw::While) => Diagnostic::new(
                Code::E0206,
                span,
                "`while` loops arrive in phase 2 (with `decreases`); use a bounded `for` loop",
            ),
            found => Diagnostic::new(
                Code::E0201,
                span,
                format!("expected {expected}, found {}", found.describe()),
            ),
        }
    }

    // ---- comment helpers ----

    /// Take every pending comment that starts before byte offset `pos`.
    fn comments_before(&mut self, pos: usize) -> Vec<Comment> {
        let mut out = Vec::new();
        while self.cpos < self.comments.len() && self.comments[self.cpos].span.start < pos {
            out.push(self.comments[self.cpos].clone());
            self.cpos += 1;
        }
        out
    }

    /// Take the next comment if it trails code on `line`.
    fn trailing_on(&mut self, line: usize) -> Option<Comment> {
        let c = self.comments.get(self.cpos)?;
        if c.line == line && !c.own_line {
            self.cpos += 1;
            Some(c.clone())
        } else {
            None
        }
    }

    /// Finish a node: comments left inside it move to `leading`; then take a trailing comment.
    fn finish_comments(&mut self, leading: &mut Vec<Comment>) -> Option<Comment> {
        let end = self.prev_span().end;
        leading.extend(self.comments_before(end));
        let line = self.prev_line();
        self.trailing_on(line)
    }

    // ---- items ----

    fn file(&mut self) -> PResult<File> {
        let mut items = Vec::new();
        while *self.peek() != TokKind::Eof {
            items.push(self.item()?);
        }
        let trailing = self.comments_before(usize::MAX);
        Ok(File { items, trailing })
    }

    fn item(&mut self) -> PResult<Item> {
        let start = self.span();
        let mut leading = self.comments_before(start.start);
        let is_pub = self.eat_kw(Kw::Pub);
        let kind = match self.peek() {
            TokKind::Kw(Kw::Fn) => ItemKind::Fn(self.fn_decl()?),
            TokKind::Kw(Kw::Type) => ItemKind::Type(self.type_decl()?),
            TokKind::Kw(Kw::Mod) => {
                self.bump();
                let name = self.ident()?;
                self.expect(Punct::Semi)?;
                ItemKind::Mod(name)
            }
            TokKind::Kw(Kw::Use) => {
                self.bump();
                let mut path = vec![self.ident()?];
                while self.eat(Punct::ColonColon) {
                    path.push(self.ident()?);
                }
                self.expect(Punct::Semi)?;
                ItemKind::Use(path)
            }
            _ => return Err(self.unexpected("item (`fn`, `type`, `mod` or `use`)")),
        };
        let span = start.to(self.prev_span());
        let trailing = self.finish_comments(&mut leading);
        Ok(Item {
            comments: Comments { leading, trailing },
            is_pub,
            kind,
            span,
        })
    }

    fn generic_params(&mut self) -> PResult<Vec<GenericParam>> {
        let mut out = Vec::new();
        if !self.eat(Punct::Lt) {
            return Ok(out);
        }
        loop {
            if self.eat_kw(Kw::Const) {
                let name = self.ident()?;
                self.expect(Punct::Colon)?;
                out.push(GenericParam::Const(name, self.ty()?));
            } else {
                out.push(GenericParam::Type(self.ident()?));
            }
            if !self.eat(Punct::Comma) {
                break;
            }
        }
        self.expect_gt()?;
        Ok(out)
    }

    fn type_decl(&mut self) -> PResult<TypeDecl> {
        self.expect_kw(Kw::Type)?;
        let name = self.ident()?;
        let generics = self.generic_params()?;
        self.expect(Punct::LBrace)?;
        let mut fields = Vec::new();
        loop {
            let mut leading = self.comments_before(self.span().start);
            let fname = self.ident()?;
            self.expect(Punct::Colon)?;
            let ty = self.ty()?;
            let more = self.eat(Punct::Comma);
            let trailing = self.finish_comments(&mut leading);
            fields.push(Field {
                comments: Comments { leading, trailing },
                name: fname,
                ty,
            });
            if !more || self.at(Punct::RBrace) {
                break;
            }
        }
        let trailing = self.comments_before(self.span().start);
        self.expect(Punct::RBrace)?;
        Ok(TypeDecl {
            name,
            generics,
            fields,
            trailing,
        })
    }

    fn fn_decl(&mut self) -> PResult<FnDecl> {
        self.expect_kw(Kw::Fn)?;
        let name = self.ident()?;
        let generics = self.generic_params()?;
        self.expect(Punct::LParen)?;
        let mut params = Vec::new();
        if !self.at(Punct::RParen) {
            loop {
                let pname = self.ident()?;
                self.expect(Punct::Colon)?;
                params.push(Param {
                    name: pname,
                    ty: self.ty()?,
                });
                if !self.eat(Punct::Comma) {
                    break;
                }
            }
        }
        self.expect(Punct::RParen)?;
        self.expect(Punct::Arrow)?;
        let ret = self.ty()?;
        self.expect_kw(Kw::Effects)?;
        self.expect(Punct::LBrace)?;
        let mut effects = Vec::new();
        if !self.at(Punct::RBrace) {
            loop {
                effects.push(self.ident()?);
                if !self.eat(Punct::Comma) {
                    break;
                }
            }
        }
        self.expect(Punct::RBrace)?;
        let body = self.block()?;
        Ok(FnDecl {
            name,
            generics,
            params,
            ret,
            effects,
            body,
        })
    }

    // ---- types ----

    fn ty(&mut self) -> PResult<Type> {
        let start = self.span();
        match self.peek() {
            TokKind::Punct(Punct::LParen) => {
                self.bump();
                let end = self.expect(Punct::RParen)?;
                Ok(Type::Unit {
                    span: start.to(end),
                })
            }
            TokKind::Punct(Punct::LBracket) => {
                let t = self.bracket_type()?;
                if let Type::Slice { span, .. } = t {
                    return Err(Diagnostic::new(
                        Code::E0201,
                        span,
                        "slice type `[T]` must be behind a reference: `&[T]`",
                    ));
                }
                Ok(t)
            }
            TokKind::Punct(Punct::Amp) => {
                self.bump();
                let mutable = self.eat_kw(Kw::Mut);
                let inner = if self.at(Punct::LBracket) {
                    self.bracket_type()?
                } else {
                    self.ty()?
                };
                Ok(Type::Ref {
                    mutable,
                    inner: Box::new(inner),
                    span: start.to(self.prev_span()),
                })
            }
            TokKind::Ident(_) => {
                let mut path = vec![self.ident()?];
                while self.eat(Punct::ColonColon) {
                    path.push(self.ident()?);
                }
                let args = if self.at(Punct::Lt) {
                    self.generic_args()?
                } else {
                    Vec::new()
                };
                Ok(Type::Named {
                    path,
                    args,
                    span: start.to(self.prev_span()),
                })
            }
            _ => Err(self.unexpected("type")),
        }
    }

    /// `"[" type ( ";" const_expr "]" | "]" )`
    fn bracket_type(&mut self) -> PResult<Type> {
        let start = self.expect(Punct::LBracket)?;
        let elem = Box::new(self.ty()?);
        if self.eat(Punct::Semi) {
            let len = match self.peek().clone() {
                TokKind::Int(l) => {
                    self.bump();
                    ConstExpr::Int(l)
                }
                TokKind::Ident(_) => ConstExpr::Name(self.ident()?),
                _ => return Err(self.unexpected("array length (integer literal or constant name)")),
            };
            let end = self.expect(Punct::RBracket)?;
            Ok(Type::Array {
                elem,
                len,
                span: start.to(end),
            })
        } else if self.at(Punct::RBracket) {
            let end = self.bump().span;
            Ok(Type::Slice {
                elem,
                span: start.to(end),
            })
        } else {
            Err(self.unexpected("`;` or `]`"))
        }
    }

    fn generic_args(&mut self) -> PResult<Vec<GenericArg>> {
        self.expect(Punct::Lt)?;
        let mut out = Vec::new();
        loop {
            if let TokKind::Int(l) = self.peek().clone() {
                self.bump();
                out.push(GenericArg::Int(l));
            } else {
                out.push(GenericArg::Type(self.ty()?));
            }
            if !self.eat(Punct::Comma) {
                break;
            }
        }
        self.expect_gt()?;
        Ok(out)
    }

    // ---- statements ----

    fn block(&mut self) -> PResult<Block> {
        let start = self.expect(Punct::LBrace)?;
        let open = self.trailing_on(self.prev_line());
        let mut stmts = Vec::new();
        while !self.at(Punct::RBrace) {
            if *self.peek() == TokKind::Eof {
                return Err(self.unexpected("`}`"));
            }
            stmts.push(self.stmt()?);
        }
        let trailing = self.comments_before(self.span().start);
        let end = self.bump().span;
        Ok(Block {
            open,
            stmts,
            trailing,
            span: start.to(end),
        })
    }

    fn stmt(&mut self) -> PResult<Stmt> {
        let start = self.span();
        let mut leading = self.comments_before(start.start);
        let kind = match self.peek() {
            TokKind::Kw(Kw::Let) => {
                self.bump();
                let mutable = self.eat_kw(Kw::Mut);
                let name = self.ident()?;
                self.expect(Punct::Colon)?;
                let ty = self.ty()?;
                self.expect(Punct::Eq)?;
                let init = self.expr()?;
                self.expect(Punct::Semi)?;
                StmtKind::Let {
                    mutable,
                    name,
                    ty,
                    init,
                }
            }
            TokKind::Kw(Kw::If) => StmtKind::If(self.if_stmt()?),
            TokKind::Kw(Kw::For) => {
                self.bump();
                let var = self.ident()?;
                self.expect(Punct::Colon)?;
                let ty = self.ty()?;
                self.expect_kw(Kw::In)?;
                let lo = self.expr()?;
                self.expect(Punct::DotDot)?;
                let hi = self.expr()?;
                let body = self.block()?;
                StmtKind::For {
                    var,
                    ty,
                    start: lo,
                    end: hi,
                    body,
                }
            }
            TokKind::Kw(Kw::Return) => {
                self.bump();
                let e = self.expr()?;
                self.expect(Punct::Semi)?;
                StmtKind::Return(e)
            }
            TokKind::Kw(Kw::Break) => {
                self.bump();
                self.expect(Punct::Semi)?;
                StmtKind::Break
            }
            TokKind::Kw(Kw::Continue) => {
                self.bump();
                self.expect(Punct::Semi)?;
                StmtKind::Continue
            }
            TokKind::Punct(Punct::LBrace) => StmtKind::Block(self.block()?),
            _ => {
                let place = self.expr()?;
                let op = match self.peek() {
                    TokKind::Punct(Punct::Eq) => Some(None),
                    TokKind::Punct(Punct::PlusEq) => Some(Some(CompoundOp::Add)),
                    TokKind::Punct(Punct::MinusEq) => Some(Some(CompoundOp::Sub)),
                    TokKind::Punct(Punct::StarEq) => Some(Some(CompoundOp::Mul)),
                    TokKind::Punct(Punct::Semi) => None,
                    _ => return Err(self.unexpected("`=`, `+=`, `-=`, `*=` or `;`")),
                };
                let kind = match op {
                    None => StmtKind::Expr(place),
                    Some(op) => {
                        self.bump();
                        let value = self.expr()?;
                        match op {
                            None => StmtKind::Assign { place, value },
                            Some(op) => StmtKind::Compound { op, place, value },
                        }
                    }
                };
                self.expect(Punct::Semi)?;
                kind
            }
        };
        let span = start.to(self.prev_span());
        let trailing = self.finish_comments(&mut leading);
        Ok(Stmt {
            comments: Comments { leading, trailing },
            kind,
            span,
        })
    }

    fn if_stmt(&mut self) -> PResult<IfStmt> {
        self.expect_kw(Kw::If)?;
        let cond = self.expr()?;
        let then = self.block()?;
        let els = if self.eat_kw(Kw::Else) {
            if self.at_kw(Kw::If) {
                Some(Else::If(Box::new(self.if_stmt()?)))
            } else if self.at(Punct::LBrace) {
                Some(Else::Block(self.block()?))
            } else {
                return Err(self.unexpected("`if` or `{`"));
            }
        } else {
            None
        };
        Ok(IfStmt { cond, then, els })
    }

    // ---- expressions ----

    pub fn expr(&mut self) -> PResult<Expr> {
        self.or_expr()
    }

    fn or_expr(&mut self) -> PResult<Expr> {
        let (mut lhs, mut lhs_chain) = self.and_expr()?;
        while self.at(Punct::OrOr) {
            let op_span = self.bump().span;
            let (rhs, rhs_chain) = self.and_expr()?;
            if lhs_chain || rhs_chain {
                return Err(mixed_logic(op_span));
            }
            lhs = binary(BinOp::Or, lhs, rhs);
            lhs_chain = false;
        }
        Ok(lhs)
    }

    /// Returns the expression and whether it is an unparenthesized `&&` chain.
    fn and_expr(&mut self) -> PResult<(Expr, bool)> {
        let mut lhs = self.cmp_expr()?;
        let mut chain = false;
        while self.at(Punct::AndAnd) {
            self.bump();
            let rhs = self.cmp_expr()?;
            lhs = binary(BinOp::And, lhs, rhs);
            chain = true;
        }
        if chain && self.at(Punct::OrOr) {
            return Err(mixed_logic(self.span()));
        }
        Ok((lhs, chain))
    }

    fn cmp_expr(&mut self) -> PResult<Expr> {
        let lhs = self.arith_expr()?;
        let Some(op) = cmp_op(self.peek()) else {
            return Ok(lhs);
        };
        self.bump();
        let rhs = self.arith_expr()?;
        if cmp_op(self.peek()).is_some() {
            return Err(Diagnostic::new(
                Code::E0204,
                self.span(),
                "comparisons cannot be chained; combine with `&&`, e.g. `a < b && b < c`",
            ));
        }
        Ok(binary(op, lhs, rhs))
    }

    fn arith_expr(&mut self) -> PResult<Expr> {
        let mut lhs = self.unary()?;
        let mut first: Option<BinOp> = None;
        while let Some(op) = arith_op(self.peek()) {
            let op_span = self.bump().span;
            match first {
                None => first = Some(op),
                Some(f) if f != op => {
                    return Err(Diagnostic::new(
                        Code::E0203,
                        op_span,
                        format!(
                            "operators `{}` and `{}` cannot be mixed without parentheses",
                            f.as_str(),
                            op.as_str()
                        ),
                    ));
                }
                Some(_) => {}
            }
            let rhs = self.unary()?;
            lhs = binary(op, lhs, rhs);
        }
        Ok(lhs)
    }

    fn unary(&mut self) -> PResult<Expr> {
        let op = match self.peek() {
            TokKind::Punct(Punct::Minus) => UnOp::Neg,
            TokKind::Punct(Punct::Bang) => UnOp::Not,
            TokKind::Punct(Punct::Amp) => {
                let start = self.bump().span;
                let mutable = self.eat_kw(Kw::Mut);
                let e = self.unary()?;
                let span = start.to(e.span);
                return Ok(Expr {
                    kind: ExprKind::Ref(mutable, Box::new(e)),
                    span,
                });
            }
            _ => return self.cast_expr(),
        };
        let start = self.bump().span;
        let e = self.unary()?;
        let span = start.to(e.span);
        Ok(Expr {
            kind: ExprKind::Unary(op, Box::new(e)),
            span,
        })
    }

    fn cast_expr(&mut self) -> PResult<Expr> {
        let e = self.postfix()?;
        if self.eat_kw(Kw::As) {
            let target = self.ident()?;
            let span = e.span.to(target.span);
            return Ok(Expr {
                kind: ExprKind::Cast(Box::new(e), target),
                span,
            });
        }
        Ok(e)
    }

    fn postfix(&mut self) -> PResult<Expr> {
        let mut e = self.primary()?;
        loop {
            match self.peek() {
                TokKind::Punct(Punct::Dot) => {
                    self.bump();
                    let f = self.ident()?;
                    let span = e.span.to(f.span);
                    e = Expr {
                        kind: ExprKind::Field(Box::new(e), f),
                        span,
                    };
                }
                TokKind::Punct(Punct::LBracket) => {
                    self.bump();
                    let i = self.expr()?;
                    let hi = if self.eat(Punct::DotDot) {
                        Some(self.expr()?)
                    } else {
                        None
                    };
                    let end = self.expect(Punct::RBracket)?;
                    let span = e.span.to(end);
                    let kind = match hi {
                        None => ExprKind::Index(Box::new(e), Box::new(i)),
                        Some(hi) => ExprKind::Slice(Box::new(e), Box::new(i), Box::new(hi)),
                    };
                    e = Expr { kind, span };
                }
                TokKind::Punct(Punct::LParen) => {
                    self.bump();
                    let args = self.args(Punct::RParen)?;
                    let end = self.expect(Punct::RParen)?;
                    let span = e.span.to(end);
                    e = Expr {
                        kind: ExprKind::Call(Box::new(e), args),
                        span,
                    };
                }
                _ => return Ok(e),
            }
        }
    }

    fn args(&mut self, close: Punct) -> PResult<Vec<Expr>> {
        let mut out = Vec::new();
        if self.at(close) {
            return Ok(out);
        }
        loop {
            out.push(self.expr()?);
            if !self.eat(Punct::Comma) {
                return Ok(out);
            }
        }
    }

    fn primary(&mut self) -> PResult<Expr> {
        let start = self.span();
        let kind = match self.peek().clone() {
            TokKind::Int(l) => {
                self.bump();
                ExprKind::Int(l)
            }
            TokKind::Kw(Kw::True) => {
                self.bump();
                ExprKind::Bool(true)
            }
            TokKind::Kw(Kw::False) => {
                self.bump();
                ExprKind::Bool(false)
            }
            TokKind::Hole(h) => {
                self.bump();
                ExprKind::Hole(h)
            }
            TokKind::Punct(Punct::LParen) => {
                self.bump();
                if self.eat(Punct::RParen) {
                    ExprKind::Unit
                } else {
                    let e = self.expr()?;
                    self.expect(Punct::RParen)?;
                    ExprKind::Paren(Box::new(e))
                }
            }
            TokKind::Punct(Punct::LBracket) => {
                self.bump();
                let elems = self.args(Punct::RBracket)?;
                self.expect(Punct::RBracket)?;
                ExprKind::Array(elems)
            }
            TokKind::Ident(_) => {
                let path = self.expr_path()?;
                if self.eat(Punct::DotLBrace) {
                    let mut fields = Vec::new();
                    loop {
                        let name = self.ident()?;
                        self.expect(Punct::Colon)?;
                        fields.push((name, self.expr()?));
                        if !self.eat(Punct::Comma) || self.at(Punct::RBrace) {
                            break;
                        }
                    }
                    self.expect(Punct::RBrace)?;
                    ExprKind::StructLit { path, fields }
                } else {
                    ExprKind::Path(path)
                }
            }
            _ => return Err(self.unexpected("expression")),
        };
        Ok(Expr {
            kind,
            span: start.to(self.prev_span()),
        })
    }

    fn expr_path(&mut self) -> PResult<ExprPath> {
        let mut segments = vec![self.ident()?];
        while self.eat(Punct::ColonColon) {
            if self.at(Punct::Lt) {
                let args = self.generic_args()?;
                return Ok(ExprPath {
                    segments,
                    generic_args: Some(args),
                });
            }
            segments.push(self.ident()?);
        }
        Ok(ExprPath {
            segments,
            generic_args: None,
        })
    }
}

fn binary(op: BinOp, lhs: Expr, rhs: Expr) -> Expr {
    let span = lhs.span.to(rhs.span);
    Expr {
        kind: ExprKind::Binary(op, Box::new(lhs), Box::new(rhs)),
        span,
    }
}

fn mixed_logic(span: Span) -> Diagnostic {
    Diagnostic::new(
        Code::E0205,
        span,
        "`&&` and `||` cannot be mixed without parentheses",
    )
}

fn cmp_op(t: &TokKind) -> Option<BinOp> {
    Some(match t {
        TokKind::Punct(Punct::EqEq) => BinOp::Eq,
        TokKind::Punct(Punct::Ne) => BinOp::Ne,
        TokKind::Punct(Punct::Lt) => BinOp::Lt,
        TokKind::Punct(Punct::Le) => BinOp::Le,
        TokKind::Punct(Punct::Gt) => BinOp::Gt,
        TokKind::Punct(Punct::Ge) => BinOp::Ge,
        _ => return None,
    })
}

fn arith_op(t: &TokKind) -> Option<BinOp> {
    Some(match t {
        TokKind::Punct(Punct::Plus) => BinOp::Add,
        TokKind::Punct(Punct::Minus) => BinOp::Sub,
        TokKind::Punct(Punct::Star) => BinOp::Mul,
        TokKind::Punct(Punct::Slash) => BinOp::Div,
        TokKind::Punct(Punct::Percent) => BinOp::Rem,
        TokKind::Punct(Punct::Amp) => BinOp::BitAnd,
        TokKind::Punct(Punct::Pipe) => BinOp::BitOr,
        TokKind::Punct(Punct::Caret) => BinOp::BitXor,
        TokKind::Punct(Punct::Shl) => BinOp::Shl,
        TokKind::Punct(Punct::Shr) => BinOp::Shr,
        _ => return None,
    })
}
