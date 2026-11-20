//! Well-formedness of a Core IR program (`spec/core-ir.md` §3.4).
//!
//! These are the facts `vcgen` and `cgen` rely on: unique names, defined
//! before use, well-typed operations and terms, restricted uses of slice
//! values and of `old`/`result`, every path ending in `return`, and an
//! acyclic call graph.

use std::collections::{BTreeMap, BTreeSet};

use crate::ast::*;
use crate::Error;

type R<T> = Result<T, Error>;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Param,
    Let,
    Var,
    Binder,
}

struct Sig {
    params: Vec<Ty>,
    ret: Ty,
}

/// What a spec term may use (§6, contracts.md §5.3).
#[derive(Clone, Copy)]
struct Ctx {
    result: bool,
    old: bool,
    in_old: bool,
    /// Only parameters and binders (contracts).
    params_only: bool,
}

pub fn check(p: &Program) -> R<()> {
    let mut sigs: BTreeMap<String, Sig> = BTreeMap::new();
    for f in &p.fns {
        let sig = Sig {
            params: f.params.iter().map(|(_, t)| t.clone()).collect(),
            ret: f.ret.clone(),
        };
        if sigs.insert(f.name.clone(), sig).is_some() {
            return Err(err(&f.name, "function defined more than once"));
        }
    }
    let mut calls: BTreeMap<&str, BTreeSet<String>> = BTreeMap::new();
    for f in &p.fns {
        let mut c = Checker {
            sigs: &sigs,
            func: &f.name,
            ret: f.ret.clone(),
            names: BTreeSet::new(),
            scopes: vec![Vec::new()],
            head_vars: None,
            callees: BTreeSet::new(),
        };
        c.func_body(f)?;
        calls.insert(&f.name, c.callees);
    }
    acyclic(&calls)
}

fn err(func: &str, msg: impl AsRef<str>) -> Error {
    Error::new(0, format!("fn {func}: {}", msg.as_ref()))
}

fn acyclic(calls: &BTreeMap<&str, BTreeSet<String>>) -> R<()> {
    // Depth-first search with colors: 1 = on the stack, 2 = done.
    fn visit<'a>(
        f: &'a str,
        calls: &'a BTreeMap<&str, BTreeSet<String>>,
        color: &mut BTreeMap<&'a str, u8>,
    ) -> R<()> {
        match color.get(f) {
            Some(1) => return Err(err(f, "recursion (the call graph must be acyclic)")),
            Some(_) => return Ok(()),
            None => {}
        }
        color.insert(f, 1);
        for g in calls.get(f).into_iter().flatten() {
            visit(g, calls, color)?;
        }
        color.insert(f, 2);
        Ok(())
    }
    let mut color = BTreeMap::new();
    for f in calls.keys() {
        visit(f, calls, &mut color)?;
    }
    Ok(())
}

struct Checker<'a> {
    sigs: &'a BTreeMap<String, Sig>,
    func: &'a str,
    ret: Ty,
    /// Every name defined so far in this function (uniqueness).
    names: BTreeSet<String>,
    scopes: Vec<Vec<(String, Ty, Kind)>>,
    /// Inside a loop `head`: the `var`s declared there.
    head_vars: Option<BTreeSet<String>>,
    callees: BTreeSet<String>,
}

impl Checker<'_> {
    fn err(&self, msg: impl AsRef<str>) -> Error {
        err(self.func, msg)
    }

    fn func_body(&mut self, f: &Fn) -> R<()> {
        if !matches!(f.ret, Ty::Unit)
            && f.ret.int_scalar().is_none()
            && f.ret != Ty::Scalar(Scalar::Bool)
        {
            return Err(self.err("return type must be a scalar or `unit`"));
        }
        for (n, t) in &f.params {
            if !matches!(t, Ty::Scalar(_) | Ty::Slice { .. }) {
                return Err(self.err(format!("parameter `{n}` must be a scalar or a slice")));
            }
            self.define(n, t.clone(), Kind::Param)?;
        }
        let entry = Ctx {
            result: false,
            old: false,
            in_old: false,
            params_only: true,
        };
        for t in &f.requires {
            self.clause(t, entry, &Ty::Scalar(Scalar::Bool), "requires")?;
        }
        let post = Ctx {
            result: f.ret != Ty::Unit,
            old: true,
            ..entry
        };
        for t in &f.ensures {
            self.clause(t, post, &Ty::Scalar(Scalar::Bool), "ensures")?;
        }
        if !self.block(&f.body)? {
            return Err(self.err("a path through the body does not end in `return`"));
        }
        Ok(())
    }

    // ---------------------------------------------------------------- names

    fn define(&mut self, n: &str, t: Ty, k: Kind) -> R<()> {
        if !self.names.insert(n.to_string()) {
            return Err(self.err(format!("`{n}` is defined more than once")));
        }
        self.scopes
            .last_mut()
            .expect("scope")
            .push((n.to_string(), t, k));
        Ok(())
    }

    fn lookup(&self, n: &str) -> R<(Ty, Kind)> {
        self.scopes
            .iter()
            .rev()
            .flat_map(|s| s.iter().rev())
            .find(|(m, _, _)| m == n)
            .map(|(_, t, k)| (t.clone(), *k))
            .ok_or_else(|| self.err(format!("`{n}` is not defined here")))
    }

    // ----------------------------------------------------------- statements

    /// Check a statement list in a new scope; true if it always returns.
    fn block(&mut self, ss: &[Stmt]) -> R<bool> {
        self.scopes.push(Vec::new());
        let mut returns = false;
        for s in ss {
            if returns {
                return Err(self.err("statement after `return`"));
            }
            returns = self.stmt(s)?;
        }
        self.scopes.pop();
        Ok(returns)
    }

    fn in_head(&self, what: &str) -> R<()> {
        if self.head_vars.is_some() {
            return Err(self.err(format!("`{what}` is not allowed in a loop `head`")));
        }
        Ok(())
    }

    fn stmt(&mut self, s: &Stmt) -> R<bool> {
        match s {
            Stmt::Let(n, t, r) => {
                if matches!(t, Ty::Slice { .. }) != matches!(r, Rhs::View { .. }) {
                    return Err(self.err(format!(
                        "`{n}`: slice values are defined only by `view`/`mview`"
                    )));
                }
                self.typed_rhs(n, t, r)?;
                self.define(n, t.clone(), Kind::Let)?;
            }
            Stmt::Var(n, t, r) => {
                if !matches!(t, Ty::Scalar(_) | Ty::Array(..)) {
                    return Err(self.err(format!("`var {n}` must be a scalar or an array")));
                }
                self.typed_rhs(n, t, r)?;
                self.define(n, t.clone(), Kind::Var)?;
                if let Some(h) = &mut self.head_vars {
                    h.insert(n.clone());
                }
            }
            Stmt::Set(n, a) => {
                let (t, k) = self.lookup(n)?;
                if k != Kind::Var || !matches!(t, Ty::Scalar(_)) {
                    return Err(self.err(format!("`set {n}`: target must be a scalar `var`")));
                }
                if self.head_vars.as_ref().is_some_and(|h| !h.contains(n)) {
                    return Err(self.err(format!(
                        "`set {n}` in a loop `head` must target a `var` declared there"
                    )));
                }
                self.expect_atom(a, &t)?;
            }
            Stmt::Store(n, i, v) => {
                self.in_head("store")?;
                let (t, k) = self.lookup(n)?;
                let ok = match t {
                    Ty::Array(..) => k == Kind::Var,
                    Ty::Slice { mutable, .. } => mutable && k == Kind::Param,
                    _ => false,
                };
                if !ok {
                    return Err(self.err(format!(
                        "`store {n}`: target must be a `var` array or a `mut` slice parameter"
                    )));
                }
                self.expect_atom(i, &Ty::Scalar(Scalar::Usize))?;
                self.expect_atom(v, &Ty::Scalar(t.elem().expect("sequence")))?;
            }
            Stmt::Call(f, args) => {
                self.in_head("call")?;
                if self.call(f, args)? != Ty::Unit {
                    return Err(self.err(format!("call statement: `{f}` does not return `unit`")));
                }
            }
            Stmt::If(c, t, e) => {
                self.expect_atom(c, &Ty::Scalar(Scalar::Bool))?;
                let a = self.block(t)?;
                let b = self.block(e)?;
                return Ok(a && b);
            }
            Stmt::Loop(l) => {
                self.in_head("loop")?;
                // Clauses hold before `head` runs, so they cannot see its names.
                let ctx = Ctx {
                    result: false,
                    old: true,
                    in_old: false,
                    params_only: false,
                };
                for i in &l.invariants {
                    self.clause(i, ctx, &Ty::Scalar(Scalar::Bool), "invariant")?;
                }
                self.clause(&l.decreases, ctx, &Ty::Int, "decreases")?;
                self.scopes.push(Vec::new());
                self.head_vars = Some(BTreeSet::new());
                let head = self.block_no_scope(&l.head);
                self.head_vars = None;
                if head? {
                    return Err(self.err("loop `head` cannot return"));
                }
                self.expect_atom(&l.cond, &Ty::Scalar(Scalar::Bool))?;
                self.block(&l.body)?;
                self.scopes.pop();
            }
            Stmt::Return(a) => {
                self.in_head("return")?;
                match (a, &self.ret) {
                    (None, Ty::Unit) => {}
                    (Some(a), t) if *t != Ty::Unit => {
                        let t = t.clone();
                        self.expect_atom(a, &t)?;
                    }
                    _ => return Err(self.err("`return` does not match the return type")),
                }
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Head statements share the loop's scope (the condition reads them).
    fn block_no_scope(&mut self, ss: &[Stmt]) -> R<bool> {
        for s in ss {
            if self.stmt(s)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    // ---------------------------------------------------------------- atoms

    fn atom(&self, a: &Atom) -> R<Ty> {
        match a {
            Atom::Bool(_) => Ok(Ty::Scalar(Scalar::Bool)),
            Atom::Lit(v, t) => {
                self.fits(*v, *t)?;
                Ok(Ty::Scalar(*t))
            }
            Atom::Name(n) => Ok(self.lookup(n)?.0),
        }
    }

    fn fits(&self, v: u64, t: Scalar) -> R<()> {
        if v > t.portable_max() {
            return Err(self.err(format!("literal {v} does not fit `{}`", t.name())));
        }
        Ok(())
    }

    fn expect_atom(&self, a: &Atom, want: &Ty) -> R<()> {
        let t = self.atom(a)?;
        if t != *want || matches!(t, Ty::Slice { .. } | Ty::Array(..)) {
            return Err(self.err(format!(
                "operand {a:?} must have type {want:?}, found {t:?}"
            )));
        }
        Ok(())
    }

    fn typed_rhs(&mut self, n: &str, t: &Ty, r: &Rhs) -> R<()> {
        let got = self.rhs(r)?;
        if got != *t {
            return Err(self.err(format!("`{n}` is declared {t:?} but its value is {got:?}")));
        }
        Ok(())
    }

    fn seq(&self, n: &str) -> R<Ty> {
        let (t, _) = self.lookup(n)?;
        if t.elem().is_none() {
            return Err(self.err(format!("`{n}` is not an array or slice")));
        }
        Ok(t)
    }

    fn rhs(&mut self, r: &Rhs) -> R<Ty> {
        let usize_t = Ty::Scalar(Scalar::Usize);
        Ok(match r {
            Rhs::Atom(a) => {
                let t = self.atom(a)?;
                if !matches!(t, Ty::Scalar(_) | Ty::Array(..)) {
                    return Err(self.err("only scalars and arrays can be copied"));
                }
                t
            }
            Rhs::Bin(op, s, a, b) => {
                let want = Ty::Scalar(*s);
                let ok = s.is_int() || matches!(op, BinOp::Eq | BinOp::Ne);
                if !ok {
                    return Err(self.err(format!(
                        "`{}` is not defined on `{}`",
                        op.name(),
                        s.name()
                    )));
                }
                self.expect_atom(a, &want)?;
                self.expect_atom(b, &want)?;
                if op.is_comparison() {
                    Ty::Scalar(Scalar::Bool)
                } else {
                    want
                }
            }
            Rhs::Not(s, a) => {
                if *s == Scalar::Usize {
                    return Err(self.err("`not.usize` is outside the slice (width-dependent)"));
                }
                self.expect_atom(a, &Ty::Scalar(*s))?;
                Ty::Scalar(*s)
            }
            Rhs::Len(n) => {
                self.seq(n)?;
                usize_t
            }
            Rhs::Load(n, i) => {
                let t = self.seq(n)?;
                self.expect_atom(i, &usize_t)?;
                Ty::Scalar(t.elem().expect("sequence"))
            }
            Rhs::Array(s, es) => {
                if es.is_empty() {
                    return Err(self.err("array literal needs at least one element"));
                }
                for e in es {
                    self.expect_atom(e, &Ty::Scalar(*s))?;
                }
                Ty::Array(*s, es.len() as u64)
            }
            Rhs::View {
                mutable,
                base,
                lo,
                hi,
            } => {
                let (t, k) = self.lookup(base)?;
                let ok = match &t {
                    Ty::Array(..) => !mutable || k == Kind::Var,
                    Ty::Slice { mutable: m, .. } => k == Kind::Param && (*m || !mutable),
                    _ => false,
                };
                if !ok {
                    return Err(self.err(format!("cannot take a view of `{base}` here")));
                }
                self.expect_atom(lo, &usize_t)?;
                self.expect_atom(hi, &usize_t)?;
                Ty::Slice {
                    mutable: *mutable,
                    elem: t.elem().expect("sequence"),
                }
            }
            Rhs::Call(f, args) => {
                self.in_head("call")?;
                let t = self.call(f, args)?;
                if t == Ty::Unit {
                    return Err(self.err(format!("`{f}` returns `unit`; use a call statement")));
                }
                t
            }
        })
    }

    fn call(&mut self, f: &str, args: &[Atom]) -> R<Ty> {
        let sig = self
            .sigs
            .get(f)
            .ok_or_else(|| self.err(format!("call to unknown function `{f}`")))?;
        if sig.params.len() != args.len() {
            return Err(self.err(format!("`{f}` takes {} argument(s)", sig.params.len())));
        }
        for (a, want) in args.iter().zip(&sig.params) {
            let t = self.atom(a)?;
            if t != *want {
                return Err(self.err(format!(
                    "argument {a:?} to `{f}` must be {want:?}, found {t:?}"
                )));
            }
        }
        self.callees.insert(f.to_string());
        Ok(sig.ret.clone())
    }

    // ---------------------------------------------------------------- terms

    fn clause(&mut self, t: &Term, ctx: Ctx, want: &Ty, what: &str) -> R<()> {
        let got = self.term(t, ctx)?;
        let ok = if *want == Ty::Int {
            got == Ty::Int
        } else {
            got == *want
        };
        if !ok {
            return Err(self.err(format!("`{what}` must have type {want:?}, found {got:?}")));
        }
        Ok(())
    }

    fn term(&mut self, t: &Term, ctx: Ctx) -> R<Ty> {
        let boolean = Ty::Scalar(Scalar::Bool);
        Ok(match t {
            Term::Name(n) => {
                let (ty, k) = self.lookup(n)?;
                if (ctx.in_old || ctx.params_only) && !matches!(k, Kind::Param | Kind::Binder) {
                    return Err(self.err(format!(
                        "`{n}` cannot be read here (only parameters and binders)"
                    )));
                }
                ty
            }
            Term::Result => {
                if !ctx.result || ctx.in_old {
                    return Err(
                        self.err("`result` is only allowed in `ensures` of a non-`unit` function")
                    );
                }
                self.ret.clone()
            }
            Term::Lit(v, s) => {
                self.fits(*v, *s)?;
                Ty::Scalar(*s)
            }
            Term::Int(_) => Ty::Int,
            Term::Bool(_) => boolean,
            Term::Lift(x) => {
                if self.term(x, ctx)?.int_scalar().is_none() {
                    return Err(self.err("`int` applies to machine integers only"));
                }
                Ty::Int
            }
            Term::Len(s) => {
                self.seq_term(s, ctx)?;
                Ty::Scalar(Scalar::Usize)
            }
            Term::Sel(s, k) => {
                let e = self.seq_term(s, ctx)?;
                self.want_term(k, ctx, &Ty::Int)?;
                Ty::Scalar(e)
            }
            Term::Old(x) => {
                if !ctx.old || ctx.in_old {
                    return Err(
                        self.err("`old` is not allowed here (not in `requires`, not nested)")
                    );
                }
                self.term(
                    x,
                    Ctx {
                        in_old: true,
                        ..ctx
                    },
                )?
            }
            Term::Op(op, a, b) => self.spec_op(*op, a, b, ctx)?,
            Term::Not(x) => {
                self.want_term(x, ctx, &boolean)?;
                boolean
            }
            Term::Bit(op, s, a, b) => {
                self.bit_scalar(op.name(), *s, *op == BinOp::Shl)?;
                self.want_term(a, ctx, &Ty::Scalar(*s))?;
                self.want_term(b, ctx, &Ty::Scalar(*s))?;
                Ty::Scalar(*s)
            }
            Term::BitNot(s, x) => {
                self.bit_scalar("bnot", *s, true)?;
                self.want_term(x, ctx, &Ty::Scalar(*s))?;
                Ty::Scalar(*s)
            }
            Term::Quant(q) => {
                self.want_term(&q.lo, ctx, &Ty::Int)?;
                self.want_term(&q.hi, ctx, &Ty::Int)?;
                // Binders may be reused by other quantifiers, never shadow.
                if self.lookup(&q.var).is_ok() {
                    return Err(self.err(format!("binder `{}` shadows a name in scope", q.var)));
                }
                self.scopes
                    .push(vec![(q.var.clone(), Ty::Int, Kind::Binder)]);
                let r = self.want_term(&q.body, ctx, &boolean);
                self.scopes.pop();
                r?;
                boolean
            }
        })
    }

    /// Bitwise terms need a machine integer; width-dependent ones exclude `usize`.
    fn bit_scalar(&self, op: &str, s: Scalar, width_dependent: bool) -> R<()> {
        if !s.is_int() || (width_dependent && s == Scalar::Usize) {
            return Err(self.err(format!("`{op}.{}` is not available in terms", s.name())));
        }
        Ok(())
    }

    fn want_term(&mut self, t: &Term, ctx: Ctx, want: &Ty) -> R<()> {
        let got = self.term(t, ctx)?;
        if got != *want {
            return Err(self.err(format!("term must have type {want:?}, found {got:?}")));
        }
        Ok(())
    }

    /// A sequence term: an array or slice name, possibly under `old`.
    fn seq_term(&mut self, s: &Term, ctx: Ctx) -> R<Scalar> {
        if !matches!(s, Term::Name(_) | Term::Old(_)) {
            return Err(self.err("sequence terms are names or `old` of a name"));
        }
        self.term(s, ctx)?
            .elem()
            .ok_or_else(|| self.err("expected an array or slice"))
    }

    fn spec_op(&mut self, op: SpecOp, a: &Term, b: &Term, ctx: Ctx) -> R<Ty> {
        use SpecOp::*;
        let boolean = Ty::Scalar(Scalar::Bool);
        let (operand, result) = match op {
            Add | Sub | Mul | Div | Rem => (Ty::Int, Ty::Int),
            Lt | Le | Gt | Ge => (Ty::Int, boolean),
            And | Or | Implies => (boolean.clone(), boolean),
            Eq | Ne => {
                let t = self.term(a, ctx)?;
                if t != Ty::Int && t != boolean {
                    return Err(self.err("`=`/`!=` compare `int` or `bool` terms"));
                }
                (t, boolean)
            }
        };
        if !matches!(op, Eq | Ne) {
            self.want_term(a, ctx, &operand)?;
        }
        self.want_term(b, ctx, &operand)?;
        Ok(result)
    }
}
