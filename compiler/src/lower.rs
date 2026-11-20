//! Lowering from the checked AST to Core IR (`spec/core-ir.md` §8).
//!
//! Untrusted: its output is re-checked by `coreir` (`spec/core-ir.md` §3.4).
//! Covers the slice subset only (`spec/contracts.md` §7); everything else is
//! rejected with `E0401`. Operations are emitted in evaluation order, one
//! `let` per operation; nothing is folded, reordered or dropped.

use std::collections::BTreeMap;

use coreir::ast as ir;

use crate::ast::*;
use crate::check::Checked;
use crate::diag::{Code, Diagnostic, Span};
use crate::types::{IntTy, Ty};

type R<T> = Result<T, Diagnostic>;

/// Lower a file that passed `check::check`.
pub fn lower(file: &File, checked: &Checked) -> R<ir::Program> {
    let mut fns = Vec::new();
    for item in &file.items {
        match &item.kind {
            ItemKind::Fn(f) => fns.push(
                Lower {
                    types: &checked.types,
                    names: BTreeMap::new(),
                    counts: BTreeMap::new(),
                    locals: BTreeMap::new(),
                    binders: Vec::new(),
                    temps: 0,
                    ret: ir::Ty::Unit,
                }
                .func(f)?,
            ),
            _ => return Err(outside(item.span, "`type` items")),
        }
    }
    Ok(ir::Program { fns })
}

fn outside(span: Span, what: &str) -> Diagnostic {
    Diagnostic::new(
        Code::E0401,
        span,
        format!("not in the slice subset (spec/contracts.md §7): {what}"),
    )
}

fn scalar_of(t: &Ty) -> Option<ir::Scalar> {
    Some(match t {
        Ty::Int(IntTy::U32) => ir::Scalar::U32,
        Ty::Int(IntTy::U64) => ir::Scalar::U64,
        Ty::Int(IntTy::Usize) => ir::Scalar::Usize,
        Ty::Bool => ir::Scalar::Bool,
        _ => return None,
    })
}

fn bin_op(op: BinOp) -> Option<ir::BinOp> {
    ir::BinOp::from_name(match op {
        BinOp::Add => "add",
        BinOp::Sub => "sub",
        BinOp::Mul => "mul",
        BinOp::Div => "div",
        BinOp::Rem => "rem",
        BinOp::BitAnd => "band",
        BinOp::BitOr => "bor",
        BinOp::BitXor => "bxor",
        BinOp::Shl => "shl",
        BinOp::Shr => "shr",
        BinOp::Eq => "eq",
        BinOp::Ne => "ne",
        BinOp::Lt => "lt",
        BinOp::Le => "le",
        BinOp::Gt => "gt",
        BinOp::Ge => "ge",
        BinOp::And | BinOp::Or | BinOp::Implies => return None,
    })
}

fn spec_op(op: BinOp) -> Option<ir::SpecOp> {
    ir::SpecOp::from_name(match op {
        BinOp::Add => "+",
        BinOp::Sub => "-",
        BinOp::Mul => "*",
        BinOp::Div => "/",
        BinOp::Rem => "%",
        BinOp::Eq => "=",
        BinOp::Ne => "!=",
        BinOp::Lt => "<",
        BinOp::Le => "<=",
        BinOp::Gt => ">",
        BinOp::Ge => ">=",
        BinOp::And => "and",
        BinOp::Or => "or",
        BinOp::Implies => "=>",
        _ => return None,
    })
}

fn always_returns(ss: &[ir::Stmt]) -> bool {
    match ss.last() {
        Some(ir::Stmt::Return(_)) => true,
        Some(ir::Stmt::If(_, t, e)) => always_returns(t) && always_returns(e),
        _ => false,
    }
}

/// Does a code expression contain a call?
fn has_call(e: &Expr) -> bool {
    match &e.kind {
        ExprKind::Call(..) => true,
        ExprKind::Paren(x)
        | ExprKind::Unary(_, x)
        | ExprKind::Field(x, _)
        | ExprKind::Cast(x, _)
        | ExprKind::Ref(_, x)
        | ExprKind::Old(x) => has_call(x),
        ExprKind::Binary(_, a, b) | ExprKind::Index(a, b) => has_call(a) || has_call(b),
        ExprKind::Slice(a, b, c) => has_call(a) || has_call(b) || has_call(c),
        ExprKind::Array(es) => es.iter().any(has_call),
        _ => false,
    }
}

struct Lower<'a> {
    types: &'a BTreeMap<(usize, usize), Ty>,
    /// Source name → Core IR name of the local currently in scope.
    names: BTreeMap<String, String>,
    /// Declarations so far per source name (for `name.N`).
    counts: BTreeMap<String, usize>,
    /// Core IR name → type.
    locals: BTreeMap<String, ir::Ty>,
    /// Quantifier binders in scope (keep their source names).
    binders: Vec<String>,
    temps: usize,
    ret: ir::Ty,
}

impl Lower<'_> {
    fn ty(&self, e: &Expr) -> &Ty {
        self.types
            .get(&(e.span.start, e.span.end))
            .expect("checked expression has a type")
    }

    fn scalar(&self, e: &Expr) -> R<ir::Scalar> {
        scalar_of(self.ty(e)).ok_or_else(|| outside(e.span, "values of this type"))
    }

    fn declare(&mut self, src: &str, t: ir::Ty) -> String {
        let n = self.counts.entry(src.to_string()).or_insert(0);
        let core = if *n == 0 {
            src.to_string()
        } else {
            format!("{src}.{n}")
        };
        *n += 1;
        self.names.insert(src.to_string(), core.clone());
        self.locals.insert(core.clone(), t);
        core
    }

    fn name(&self, src: &str) -> String {
        self.names
            .get(src)
            .cloned()
            .unwrap_or_else(|| src.to_string())
    }

    fn temp(&mut self, t: ir::Ty, rhs: ir::Rhs, out: &mut Vec<ir::Stmt>) -> ir::Atom {
        let n = format!("${}", self.temps);
        self.temps += 1;
        self.locals.insert(n.clone(), t.clone());
        out.push(ir::Stmt::Let(n.clone(), t, rhs));
        ir::Atom::Name(n)
    }

    // ---------------------------------------------------------------- types

    fn ast_scalar(&self, t: &Type) -> Option<ir::Scalar> {
        match t {
            Type::Named { path, args, .. } if path.len() == 1 && args.is_empty() => {
                ir::Scalar::from_name(&path[0].name)
            }
            _ => None,
        }
    }

    /// Parameter, return or `let` type, as allowed by the slice.
    fn ast_ty(&self, t: &Type, param: bool) -> R<ir::Ty> {
        let span = match t {
            Type::Named { span, .. }
            | Type::Array { span, .. }
            | Type::Slice { span, .. }
            | Type::Ref { span, .. }
            | Type::Unit { span } => *span,
        };
        if let Some(s) = self.ast_scalar(t) {
            return Ok(ir::Ty::Scalar(s));
        }
        match t {
            Type::Unit { .. } => Ok(ir::Ty::Unit),
            Type::Ref { mutable, inner, .. } if param => match &**inner {
                Type::Slice { elem, .. } => match self.ast_scalar(elem) {
                    Some(e) => Ok(ir::Ty::Slice {
                        mutable: *mutable,
                        elem: e,
                    }),
                    None => Err(outside(span, "slices of this element type")),
                },
                _ => Err(outside(span, "references other than `&[T]`/`&mut [T]`")),
            },
            Type::Array {
                elem,
                len: ConstExpr::Int(n),
                ..
            } if !param => match self.ast_scalar(elem) {
                Some(e) => Ok(ir::Ty::Array(e, n.value)),
                None => Err(outside(span, "arrays of this element type")),
            },
            _ => Err(outside(span, "types like this")),
        }
    }

    // ------------------------------------------------------------ functions

    fn func(mut self, f: &FnDecl) -> R<ir::Fn> {
        let mut params = Vec::new();
        for p in &f.params {
            let t = self.ast_ty(&p.ty, true)?;
            params.push((self.declare(&p.name.name, t.clone()), t));
        }
        self.ret = self.ast_ty(&f.ret, false)?;
        if matches!(self.ret, ir::Ty::Array(..)) {
            return Err(outside(f.name.span, "array return types"));
        }
        let requires = f.requires.iter().map(|c| self.term(c)).collect::<R<_>>()?;
        let ensures = f.ensures.iter().map(|c| self.term(c)).collect::<R<_>>()?;
        let mut body = Vec::new();
        self.stmts(&f.body.stmts, &mut body)?;
        if self.ret == ir::Ty::Unit && !always_returns(&body) {
            body.push(ir::Stmt::Return(None));
        }
        Ok(ir::Fn {
            name: f.name.name.clone(),
            params,
            ret: self.ret,
            requires,
            ensures,
            body,
        })
    }

    // ----------------------------------------------------------- statements

    fn stmts(&mut self, ss: &[Stmt], out: &mut Vec<ir::Stmt>) -> R<()> {
        ss.iter().try_for_each(|s| self.stmt(s, out))
    }

    fn block(&mut self, b: &Block) -> R<Vec<ir::Stmt>> {
        let mut out = Vec::new();
        self.stmts(&b.stmts, &mut out)?;
        Ok(out)
    }

    fn stmt(&mut self, s: &Stmt, out: &mut Vec<ir::Stmt>) -> R<()> {
        match &s.kind {
            StmtKind::Let {
                mutable,
                name,
                ty,
                init,
            } => {
                let t = self.ast_ty(ty, false)?;
                let a = self.code(init, out)?;
                let n = self.declare(&name.name, t.clone());
                out.push(if *mutable {
                    ir::Stmt::Var(n, t, ir::Rhs::Atom(a))
                } else {
                    ir::Stmt::Let(n, t, ir::Rhs::Atom(a))
                });
            }
            StmtKind::If(i) => self.if_stmt(i, out)?,
            StmtKind::While {
                cond,
                invariants,
                decreases,
                body,
            } => {
                if has_call(cond) {
                    return Err(outside(cond.span, "calls in a `while` condition"));
                }
                let invariants = invariants.iter().map(|c| self.term(c)).collect::<R<_>>()?;
                let decreases = self.term(decreases)?;
                let mut head = Vec::new();
                let cond = self.code(cond, &mut head)?;
                let body = self.block(body)?;
                out.push(ir::Stmt::Loop(Box::new(ir::Loop {
                    head,
                    cond,
                    invariants,
                    decreases,
                    body,
                })));
            }
            StmtKind::Return(e) => {
                if self.ret == ir::Ty::Unit {
                    match &e.kind {
                        ExprKind::Unit => {}
                        ExprKind::Call(f, args) => self.call_stmt(f, args, out)?,
                        _ => return Err(outside(e.span, "unit-valued expressions")),
                    }
                    out.push(ir::Stmt::Return(None));
                } else {
                    let a = self.code(e, out)?;
                    out.push(ir::Stmt::Return(Some(a)));
                }
            }
            StmtKind::Block(b) => self.stmts(&b.stmts, out)?,
            StmtKind::Assign { place, value } => match &place.kind {
                ExprKind::Path(p) => {
                    let v = self.code(value, out)?;
                    out.push(ir::Stmt::Set(self.name(&p.segments[0].name), v));
                }
                ExprKind::Index(b, i) => {
                    let base = self.base(b)?;
                    let i = self.code(i, out)?;
                    let v = self.code(value, out)?;
                    out.push(ir::Stmt::Store(base, i, v));
                }
                _ => return Err(outside(place.span, "assignments to fields")),
            },
            StmtKind::Compound { op, place, value } => {
                let s = self.scalar(place)?;
                let op = match op {
                    CompoundOp::Add => ir::BinOp::Add,
                    CompoundOp::Sub => ir::BinOp::Sub,
                    CompoundOp::Mul => ir::BinOp::Mul,
                };
                match &place.kind {
                    ExprKind::Path(p) => {
                        let x = self.name(&p.segments[0].name);
                        let v = self.code(value, out)?;
                        let rhs = ir::Rhs::Bin(op, s, ir::Atom::Name(x.clone()), v);
                        let r = self.temp(ir::Ty::Scalar(s), rhs, out);
                        out.push(ir::Stmt::Set(x, r));
                    }
                    ExprKind::Index(b, i) => {
                        let base = self.base(b)?;
                        let i = self.code(i, out)?;
                        let old = self.temp(
                            ir::Ty::Scalar(s),
                            ir::Rhs::Load(base.clone(), i.clone()),
                            out,
                        );
                        let v = self.code(value, out)?;
                        let r = self.temp(ir::Ty::Scalar(s), ir::Rhs::Bin(op, s, old, v), out);
                        out.push(ir::Stmt::Store(base, i, r));
                    }
                    _ => return Err(outside(place.span, "assignments to fields")),
                }
            }
            StmtKind::Expr(e) => match &e.kind {
                ExprKind::Call(f, args) => self.call_stmt(f, args, out)?,
                _ => return Err(outside(e.span, "expression statements other than calls")),
            },
            StmtKind::For { .. } => return Err(outside(s.span, "`for` loops")),
            StmtKind::Break | StmtKind::Continue => {
                return Err(outside(s.span, "`break` and `continue`"))
            }
        }
        Ok(())
    }

    fn if_stmt(&mut self, i: &IfStmt, out: &mut Vec<ir::Stmt>) -> R<()> {
        let c = self.code(&i.cond, out)?;
        let then = self.block(&i.then)?;
        let els = match &i.els {
            None => Vec::new(),
            Some(Else::Block(b)) => self.block(b)?,
            Some(Else::If(e)) => {
                let mut v = Vec::new();
                self.if_stmt(e, &mut v)?;
                v
            }
        };
        out.push(ir::Stmt::If(c, then, els));
        Ok(())
    }

    // ---------------------------------------------------------------- code

    /// Base of `.len`, `[i]`, `[lo..hi]`: a local or parameter name.
    fn base(&self, b: &Expr) -> R<String> {
        match &b.kind {
            ExprKind::Path(p) => Ok(self.name(&p.segments[0].name)),
            _ => Err(outside(b.span, "indexing expressions other than names")),
        }
    }

    fn fn_name(&self, f: &Expr) -> String {
        match &f.kind {
            ExprKind::Path(p) => p.segments[0].name.clone(),
            _ => unreachable!("checked call target"),
        }
    }

    fn args(&mut self, args: &[Expr], out: &mut Vec<ir::Stmt>) -> R<Vec<ir::Atom>> {
        let mut atoms = Vec::new();
        for a in args {
            atoms.push(match &a.kind {
                ExprKind::Ref(m, inner) => match &inner.kind {
                    ExprKind::Slice(b, lo, hi) => {
                        let base = self.base(b)?;
                        let elem = self.locals[&base].elem().expect("sequence");
                        let lo = self.code(lo, out)?;
                        let hi = self.code(hi, out)?;
                        let t = ir::Ty::Slice { mutable: *m, elem };
                        let rhs = ir::Rhs::View {
                            mutable: *m,
                            base,
                            lo,
                            hi,
                        };
                        self.temp(t, rhs, out)
                    }
                    _ => return Err(outside(a.span, "references other than slices `&p[lo..hi]`")),
                },
                ExprKind::Path(p)
                    if matches!(
                        self.locals.get(&self.name(&p.segments[0].name)),
                        Some(ir::Ty::Slice { .. })
                    ) =>
                {
                    ir::Atom::Name(self.name(&p.segments[0].name))
                }
                _ => self.code(a, out)?,
            });
        }
        Ok(atoms)
    }

    fn call_stmt(&mut self, f: &Expr, args: &[Expr], out: &mut Vec<ir::Stmt>) -> R<()> {
        let atoms = self.args(args, out)?;
        out.push(ir::Stmt::Call(self.fn_name(f), atoms));
        Ok(())
    }

    /// Lower a code expression; returns the atom holding its value.
    fn code(&mut self, e: &Expr, out: &mut Vec<ir::Stmt>) -> R<ir::Atom> {
        Ok(match &e.kind {
            ExprKind::Int(l) => ir::Atom::Lit(l.value, self.scalar(e)?),
            ExprKind::Bool(b) => ir::Atom::Bool(*b),
            ExprKind::Path(p) => ir::Atom::Name(self.name(&p.segments[0].name)),
            ExprKind::Paren(x) => self.code(x, out)?,
            ExprKind::Binary(op @ (BinOp::And | BinOp::Or), l, r) => {
                // Short-circuit: `$r` starts as `l`; `r` runs only if needed.
                let a = self.code(l, out)?;
                let n = format!("${}", self.temps);
                self.temps += 1;
                let boolean = ir::Ty::Scalar(ir::Scalar::Bool);
                self.locals.insert(n.clone(), boolean.clone());
                out.push(ir::Stmt::Var(n.clone(), boolean, ir::Rhs::Atom(a.clone())));
                let mut branch = Vec::new();
                let b = self.code(r, &mut branch)?;
                branch.push(ir::Stmt::Set(n.clone(), b));
                out.push(if *op == BinOp::And {
                    ir::Stmt::If(a, branch, Vec::new())
                } else {
                    ir::Stmt::If(a, Vec::new(), branch)
                });
                ir::Atom::Name(n)
            }
            ExprKind::Binary(op, l, r) => {
                let s = self.scalar(l)?;
                let op = bin_op(*op).ok_or_else(|| outside(e.span, "operators like this"))?;
                let a = self.code(l, out)?;
                let b = self.code(r, out)?;
                let t = ir::Ty::Scalar(self.scalar(e)?);
                self.temp(t, ir::Rhs::Bin(op, s, a, b), out)
            }
            ExprKind::Unary(UnOp::Not, x) => {
                let s = self.scalar(x)?;
                if s == ir::Scalar::Usize {
                    return Err(outside(e.span, "`!` on `usize` values"));
                }
                let a = self.code(x, out)?;
                self.temp(ir::Ty::Scalar(s), ir::Rhs::Not(s, a), out)
            }
            ExprKind::Field(b, f) if f.name == "len" => {
                let base = self.base(b)?;
                self.temp(ir::Ty::Scalar(ir::Scalar::Usize), ir::Rhs::Len(base), out)
            }
            ExprKind::Index(b, i) => {
                let base = self.base(b)?;
                let i = self.code(i, out)?;
                let t = ir::Ty::Scalar(self.scalar(e)?);
                self.temp(t, ir::Rhs::Load(base, i), out)
            }
            ExprKind::Call(f, args) => {
                let atoms = self.args(args, out)?;
                let t = ir::Ty::Scalar(self.scalar(e)?);
                self.temp(t, ir::Rhs::Call(self.fn_name(f), atoms), out)
            }
            ExprKind::Array(es) => {
                let Ty::Array(elem, n) = self.ty(e).clone() else {
                    unreachable!("array literal has an array type")
                };
                let s = scalar_of(&elem)
                    .ok_or_else(|| outside(e.span, "arrays of this element type"))?;
                let atoms = es.iter().map(|x| self.code(x, out)).collect::<R<_>>()?;
                self.temp(ir::Ty::Array(s, n), ir::Rhs::Array(s, atoms), out)
            }
            ExprKind::Unary(UnOp::Neg, _) => {
                return Err(outside(e.span, "negation (signed types)"))
            }
            ExprKind::Cast(..) => return Err(outside(e.span, "casts")),
            ExprKind::StructLit { .. } | ExprKind::Field(..) => {
                return Err(outside(e.span, "structs"))
            }
            _ => return Err(outside(e.span, "expressions like this")),
        })
    }

    // ---------------------------------------------------------------- terms

    /// A term, lifted to `int` if it has a machine-integer type.
    fn lifted(&mut self, e: &Expr) -> R<ir::Term> {
        let t = self.term(e)?;
        Ok(match self.ty(e) {
            Ty::Int(_) => ir::Term::Lift(Box::new(t)),
            _ => t,
        })
    }

    /// Sequence term: a name, or `old` of a name.
    fn seq(&mut self, b: &Expr) -> R<ir::Term> {
        match &b.kind {
            ExprKind::Path(_) => self.term(b),
            ExprKind::Old(x) if matches!(x.kind, ExprKind::Path(_)) => {
                Ok(ir::Term::Old(Box::new(self.term(x)?)))
            }
            ExprKind::Paren(x) => self.seq(x),
            _ => Err(outside(b.span, "sequence expressions other than names")),
        }
    }

    fn term(&mut self, e: &Expr) -> R<ir::Term> {
        let b = Box::new;
        Ok(match &e.kind {
            ExprKind::Int(l) => match self.ty(e) {
                Ty::MathInt => ir::Term::Int(i128::from(l.value)),
                _ => ir::Term::Lit(l.value, self.scalar(e)?),
            },
            ExprKind::Bool(v) => ir::Term::Bool(*v),
            ExprKind::Path(p) => {
                let n = &p.segments[0].name;
                if self.binders.contains(n) {
                    ir::Term::Name(n.clone())
                } else {
                    ir::Term::Name(self.name(n))
                }
            }
            ExprKind::Paren(x) => self.term(x)?,
            ExprKind::Result => ir::Term::Result,
            ExprKind::Old(x) => ir::Term::Old(b(self.term(x)?)),
            ExprKind::Field(x, f) if f.name == "len" => ir::Term::Len(b(self.seq(x)?)),
            ExprKind::Index(s, i) => ir::Term::Sel(b(self.seq(s)?), b(self.lifted(i)?)),
            ExprKind::Unary(UnOp::Neg, x) => match &x.kind {
                ExprKind::Int(l) => ir::Term::Int(-i128::from(l.value)),
                _ => ir::Term::Op(ir::SpecOp::Sub, b(ir::Term::Int(0)), b(self.lifted(x)?)),
            },
            ExprKind::Unary(UnOp::Not, x) => match self.scalar(x)? {
                ir::Scalar::Bool => ir::Term::Not(b(self.term(x)?)),
                ir::Scalar::Usize => return Err(outside(e.span, "`!` on `usize` values")),
                s => ir::Term::BitNot(s, b(self.term(x)?)),
            },
            ExprKind::Binary(op, l, r) => {
                let bitwise = matches!(
                    op,
                    BinOp::BitAnd | BinOp::BitOr | BinOp::BitXor | BinOp::Shl | BinOp::Shr
                );
                if bitwise {
                    let s = self.scalar(e)?;
                    if *op == BinOp::Shl && s == ir::Scalar::Usize {
                        return Err(outside(e.span, "`<<` on `usize` in spec contexts"));
                    }
                    let op = bin_op(*op).expect("bitwise op");
                    ir::Term::Bit(op, s, b(self.term(l)?), b(self.term(r)?))
                } else {
                    let sop = spec_op(*op).expect("spec op");
                    let logical = matches!(op, BinOp::And | BinOp::Or | BinOp::Implies)
                        || *self.ty(l) == Ty::Bool;
                    if logical {
                        ir::Term::Op(sop, b(self.term(l)?), b(self.term(r)?))
                    } else {
                        ir::Term::Op(sop, b(self.lifted(l)?), b(self.lifted(r)?))
                    }
                }
            }
            ExprKind::Quant(q) => {
                let lo = self.lifted(&q.lo)?;
                let hi = self.lifted(&q.hi)?;
                self.binders.push(q.var.name.clone());
                let body = self.term(&q.body);
                self.binders.pop();
                ir::Term::Quant(Box::new(ir::Quant {
                    forall: q.forall,
                    var: q.var.name.clone(),
                    lo,
                    hi,
                    body: body?,
                }))
            }
            _ => return Err(outside(e.span, "spec expressions like this")),
        })
    }
}
