//! Phase-1 name resolution and type checking (`spec/typing.md`).
//!
//! Stops at the first error. On success, returns the unverified-checks report:
//! every runtime trap site that is not statically trivial (§11).

use std::collections::{BTreeMap, BTreeSet};

use crate::ast::*;
use crate::diag::{Code, Diagnostic, Span};
use crate::types::{IntTy, Ty};

type CResult<T> = Result<T, Diagnostic>;

/// Kind of runtime trap (`spec/typing.md` §11).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TrapKind {
    Overflow,
    Negate,
    DivZero,
    DivOverflow,
    Shift,
    Cast,
    Bounds,
    Slice,
    // Contract clauses, unverified until `vcgen` (`spec/contracts.md` §6).
    Requires,
    Ensures,
    Invariant,
    Decreases,
}

impl TrapKind {
    pub fn as_str(self) -> &'static str {
        match self {
            TrapKind::Overflow => "overflow",
            TrapKind::Negate => "negate",
            TrapKind::DivZero => "div-zero",
            TrapKind::DivOverflow => "div-overflow",
            TrapKind::Shift => "shift",
            TrapKind::Cast => "cast",
            TrapKind::Bounds => "bounds",
            TrapKind::Slice => "slice",
            TrapKind::Requires => "requires",
            TrapKind::Ensures => "ensures",
            TrapKind::Invariant => "invariant",
            TrapKind::Decreases => "decreases",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Trap {
    pub span: Span,
    pub func: String,
    pub kind: TrapKind,
    /// Operand type(s), e.g. `u32` or `u64 -> u32`; empty for clauses.
    pub detail: String,
}

#[derive(Clone, Debug)]
pub struct Checked {
    /// Trap sites in source order.
    pub traps: Vec<Trap>,
    /// Type of every checked expression, keyed by its span (start, end).
    /// Used by lowering (`spec/core-ir.md` §8).
    pub types: BTreeMap<(usize, usize), Ty>,
}

/// Render the unverified-checks report.
pub fn render_report(file: &str, src: &str, traps: &[Trap]) -> String {
    let mut out = format!("unverified checks: {}\n", traps.len());
    for t in traps {
        let (line, col) = crate::diag::line_col(src, t.span.start);
        out.push_str(&format!(
            "{file}:{line}:{col}: {}: {}",
            t.func,
            t.kind.as_str()
        ));
        if !t.detail.is_empty() {
            out.push_str(": ");
            out.push_str(&t.detail);
        }
        out.push('\n');
    }
    out
}

/// Names that items and locals may not take (§3).
const BUILTIN_TYPES: &[&str] = &[
    "u8", "u16", "u32", "u64", "usize", "i8", "i16", "i32", "i64", "isize", "bool", "int", "nat",
];

fn err(code: Code, span: Span, msg: impl Into<String>) -> Diagnostic {
    Diagnostic::new(code, span, msg)
}

pub fn check(file: &File) -> CResult<Checked> {
    let mut c = Checker::default();
    c.collect_items(file)?;
    c.struct_fields(file)?;
    c.struct_cycles(file)?;
    c.signatures(file)?;
    for item in &file.items {
        if let ItemKind::Fn(f) = &item.kind {
            c.check_fn(f)?;
        }
    }
    c.recursion()?;
    let mut traps = c.traps;
    traps.sort_by_key(|t| (t.span.start, t.kind));
    Ok(Checked {
        traps,
        types: c.types,
    })
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ItemRef {
    Type,
    Fn,
}

#[derive(Clone)]
struct FnSig {
    params: Vec<Ty>,
    ret: Ty,
}

struct Local {
    name: String,
    ty: Ty,
    mutable: bool,
    /// Exists in the function's entry state (parameter or quantifier
    /// binder), so `old` may read it.
    at_entry: bool,
}

/// What a spec context allows (`spec/contracts.md` §5.3).
#[derive(Clone, Copy)]
struct SpecCtx {
    /// Clause keyword, for messages.
    clause: &'static str,
    result: bool,
    old: bool,
    in_old: bool,
}

/// A place: its root local and whether it is assignable (§7).
struct PlaceInfo {
    root: String,
    assignable: bool,
}

#[derive(Default)]
struct Checker {
    items: BTreeMap<String, ItemRef>,
    /// Source order of items, for deterministic traversal.
    struct_order: Vec<(String, Span)>,
    fn_order: Vec<String>,
    structs: BTreeMap<String, Vec<(String, Ty)>>,
    fns: BTreeMap<String, FnSig>,
    /// Caller -> callees with call-site spans, in source order.
    calls: BTreeMap<String, Vec<(String, Span)>>,
    traps: Vec<Trap>,
    // Per-function state.
    cur: String,
    ret: Ty,
    scopes: Vec<Vec<Local>>,
    loop_depth: usize,
    /// Set while checking a contract clause.
    spec: Option<SpecCtx>,
    types: BTreeMap<(usize, usize), Ty>,
}

impl Checker {
    // ---------------------------------------------------------------- items

    fn collect_items(&mut self, file: &File) -> CResult<()> {
        for item in &file.items {
            let (name, kind, generic) =
                match &item.kind {
                    ItemKind::Mod(_) | ItemKind::Use(_) => return Err(err(
                        Code::E0318,
                        item.span,
                        "`mod` and `use` are not available in phase 1 (single-file compilation)",
                    )),
                    ItemKind::Fn(f) => (&f.name, ItemRef::Fn, !f.generics.is_empty()),
                    ItemKind::Type(t) => (&t.name, ItemRef::Type, !t.generics.is_empty()),
                };
            if generic {
                return Err(err(
                    Code::E0318,
                    name.span,
                    format!(
                        "generic parameters on `{}` are not available in phase 1",
                        name.name
                    ),
                ));
            }
            if BUILTIN_TYPES.contains(&name.name.as_str()) {
                return Err(err(
                    Code::E0302,
                    name.span,
                    format!("`{}` is a built-in type name", name.name),
                ));
            }
            if self.items.contains_key(&name.name) {
                return Err(err(
                    Code::E0302,
                    name.span,
                    format!("`{}` is defined more than once", name.name),
                ));
            }
            self.items.insert(name.name.clone(), kind);
            match kind {
                ItemRef::Type => self.struct_order.push((name.name.clone(), name.span)),
                ItemRef::Fn => self.fn_order.push(name.name.clone()),
            }
        }
        Ok(())
    }

    fn struct_fields(&mut self, file: &File) -> CResult<()> {
        for item in &file.items {
            let ItemKind::Type(t) = &item.kind else {
                continue;
            };
            let mut fields: Vec<(String, Ty)> = Vec::new();
            for f in &t.fields {
                if fields.iter().any(|(n, _)| *n == f.name.name) {
                    return Err(err(
                        Code::E0302,
                        f.name.span,
                        format!("field `{}` is declared more than once", f.name.name),
                    ));
                }
                fields.push((f.name.name.clone(), self.resolve_ty(&f.ty, false)?));
            }
            self.structs.insert(t.name.name.clone(), fields);
        }
        Ok(())
    }

    fn struct_cycles(&self, _file: &File) -> CResult<()> {
        fn by_value(t: &Ty, out: &mut Vec<String>) {
            match t {
                Ty::Struct(n) => out.push(n.clone()),
                Ty::Array(e, _) => by_value(e, out),
                _ => {}
            }
        }
        for (start, span) in &self.struct_order {
            // DFS for a path start -> ... -> start.
            let mut stack: Vec<(String, Vec<String>)> = vec![(start.clone(), vec![start.clone()])];
            let mut seen = BTreeSet::new();
            while let Some((cur, path)) = stack.pop() {
                let mut next = Vec::new();
                for (_, ty) in &self.structs[&cur] {
                    by_value(ty, &mut next);
                }
                for n in next.into_iter().rev() {
                    let mut p = path.clone();
                    p.push(n.clone());
                    if n == *start {
                        return Err(err(
                            Code::E0324,
                            *span,
                            format!(
                                "type `{start}` contains itself by value ({})",
                                p.join(" -> ")
                            ),
                        ));
                    }
                    if seen.insert(n.clone()) {
                        stack.push((n, p));
                    }
                }
            }
        }
        Ok(())
    }

    fn signatures(&mut self, file: &File) -> CResult<()> {
        for item in &file.items {
            let ItemKind::Fn(f) = &item.kind else {
                continue;
            };
            if let Some(e) = f.effects.first() {
                return Err(err(
                    Code::E0318,
                    e.span,
                    "phase 1 accepts only `effects {}` (effects arrive in phase 4)",
                ));
            }
            let mut params = Vec::new();
            for (i, p) in f.params.iter().enumerate() {
                if f.params[..i].iter().any(|q| q.name.name == p.name.name) {
                    return Err(err(
                        Code::E0302,
                        p.name.span,
                        format!("parameter `{}` is declared more than once", p.name.name),
                    ));
                }
                params.push(self.resolve_ty(&p.ty, true)?);
            }
            let ret = self.resolve_ty(&f.ret, false)?;
            self.fns.insert(f.name.name.clone(), FnSig { params, ret });
        }
        Ok(())
    }

    fn resolve_ty(&self, t: &Type, allow_ref: bool) -> CResult<Ty> {
        match t {
            Type::Named { path, args, span } => {
                if path.len() > 1 {
                    return Err(err(
                        Code::E0318,
                        *span,
                        "paths with `::` need modules, which are not available in phase 1",
                    ));
                }
                if !args.is_empty() {
                    return Err(err(
                        Code::E0318,
                        *span,
                        "generic arguments are not available in phase 1",
                    ));
                }
                let name = &path[0].name;
                if let Some(i) = IntTy::from_name(name) {
                    return Ok(Ty::Int(i));
                }
                match name.as_str() {
                    "bool" => Ok(Ty::Bool),
                    "int" => Err(err(
                        Code::E0326,
                        *span,
                        "`int` is spec-only; it can only be a quantifier binder type",
                    )),
                    "nat" => Err(err(Code::E0318, *span, "`nat` is not available yet")),
                    _ => match self.items.get(name) {
                        Some(ItemRef::Type) => Ok(Ty::Struct(name.clone())),
                        Some(ItemRef::Fn) => Err(err(
                            Code::E0303,
                            *span,
                            format!("`{name}` is a function, not a type"),
                        )),
                        None => Err(err(Code::E0301, *span, format!("unknown type `{name}`"))),
                    },
                }
            }
            Type::Array { elem, len, span } => {
                let elem = self.resolve_ty(elem, false)?;
                let n = match len {
                    ConstExpr::Name(i) => return Err(err(
                        Code::E0318,
                        i.span,
                        "named array lengths are not available in phase 1; use an integer literal",
                    )),
                    ConstExpr::Int(l) => l,
                };
                if n.suffix.is_some_and(|s| s.as_str() != "usize") {
                    return Err(err(
                        Code::E0322,
                        *span,
                        "array length must be unsuffixed or `usize`",
                    ));
                }
                if n.value == 0 || n.value > (1u64 << 63) - 1 {
                    return Err(err(
                        Code::E0322,
                        *span,
                        format!(
                            "array length must be between 1 and 2^63 - 1, found {}",
                            n.value
                        ),
                    ));
                }
                Ok(Ty::Array(Box::new(elem), n.value))
            }
            Type::Slice { elem, .. } => Ok(Ty::Slice(Box::new(self.resolve_ty(elem, false)?))),
            Type::Ref {
                mutable,
                inner,
                span,
            } => {
                if !allow_ref {
                    return Err(err(
                        Code::E0319,
                        *span,
                        "reference types are only allowed as parameter types in phase 1",
                    ));
                }
                Ok(Ty::Ref(*mutable, Box::new(self.resolve_ty(inner, false)?)))
            }
            Type::Unit { .. } => Ok(Ty::Unit),
        }
    }

    fn recursion(&self) -> CResult<()> {
        for start in &self.fn_order {
            let mut seen = BTreeSet::new();
            let mut path: Vec<(String, Span)> = Vec::new();
            if self.cycle_from(start, start, &mut seen, &mut path) {
                let names: Vec<&str> = std::iter::once(start.as_str())
                    .chain(path.iter().map(|(n, _)| n.as_str()))
                    .collect();
                return Err(err(
                    Code::E0317,
                    path[0].1,
                    format!(
                        "recursion is not available in phase 1 (needs `decreases`, phase 2): {}",
                        names.join(" -> ")
                    ),
                ));
            }
        }
        Ok(())
    }

    fn cycle_from(
        &self,
        start: &str,
        cur: &str,
        seen: &mut BTreeSet<String>,
        path: &mut Vec<(String, Span)>,
    ) -> bool {
        for (callee, span) in self.calls.get(cur).map(Vec::as_slice).unwrap_or(&[]) {
            path.push((callee.clone(), *span));
            if callee == start {
                return true;
            }
            if seen.insert(callee.clone()) && self.cycle_from(start, callee, seen, path) {
                return true;
            }
            path.pop();
        }
        false
    }

    // ------------------------------------------------------------ functions

    fn check_fn(&mut self, f: &FnDecl) -> CResult<()> {
        let sig = self.fns[&f.name.name].clone();
        self.cur = f.name.name.clone();
        self.ret = sig.ret.clone();
        self.scopes = vec![Vec::new()];
        self.loop_depth = 0;
        self.calls.entry(self.cur.clone()).or_default();
        self.spec = None;
        for (p, ty) in f.params.iter().zip(sig.params) {
            self.declare(&p.name, ty, false)?;
            self.mark_at_entry();
        }
        for c in &f.requires {
            self.clause(c, TrapKind::Requires, false, false)?;
        }
        let has_result = self.ret != Ty::Unit;
        for c in &f.ensures {
            self.clause(c, TrapKind::Ensures, has_result, true)?;
        }
        let returns = self.block(&f.body)?;
        if self.ret != Ty::Unit && !returns {
            let end = Span::new(f.body.span.end.saturating_sub(1), f.body.span.end);
            return Err(err(
                Code::E0313,
                end,
                format!(
                    "function `{}` must return a `{}` on every path",
                    f.name.name, self.ret
                ),
            ));
        }
        Ok(())
    }

    fn declare(&mut self, name: &Ident, ty: Ty, mutable: bool) -> CResult<()> {
        let n = &name.name;
        let clash = if BUILTIN_TYPES.contains(&n.as_str()) {
            Some("a built-in type name")
        } else if self.items.contains_key(n) {
            Some("an item name")
        } else if self.lookup(n).is_some() {
            Some("already a local in scope (no shadowing)")
        } else {
            None
        };
        if let Some(why) = clash {
            return Err(err(Code::E0302, name.span, format!("`{n}` is {why}")));
        }
        self.scopes.last_mut().expect("scope").push(Local {
            name: n.clone(),
            ty,
            mutable,
            at_entry: false,
        });
        Ok(())
    }

    /// The local declared last exists at function entry.
    fn mark_at_entry(&mut self) {
        if let Some(l) = self.scopes.last_mut().and_then(|s| s.last_mut()) {
            l.at_entry = true;
        }
    }

    fn lookup(&self, name: &str) -> Option<&Local> {
        self.scopes
            .iter()
            .rev()
            .flat_map(|s| s.iter())
            .find(|l| l.name == name)
    }

    fn trap(&mut self, kind: TrapKind, span: Span, detail: impl Into<String>) {
        self.traps.push(Trap {
            span,
            func: self.cur.clone(),
            kind,
            detail: detail.into(),
        });
    }

    // ----------------------------------------------------------- statements

    /// Returns true if the block always diverges (returns, breaks or continues).
    fn block(&mut self, b: &Block) -> CResult<bool> {
        self.scopes.push(Vec::new());
        let mut diverges = false;
        for s in &b.stmts {
            if diverges {
                return Err(err(Code::E0314, s.span, "unreachable statement"));
            }
            diverges = self.stmt(s)?;
        }
        self.scopes.pop();
        Ok(diverges)
    }

    fn stmt(&mut self, s: &Stmt) -> CResult<bool> {
        match &s.kind {
            StmtKind::Let {
                mutable,
                name,
                ty,
                init,
            } => {
                let ty = self.resolve_ty(ty, false)?;
                self.check(init, &ty)?;
                self.declare(name, ty, *mutable)?;
                Ok(false)
            }
            StmtKind::If(i) => self.if_stmt(i),
            StmtKind::For {
                var,
                ty,
                start,
                end,
                body,
            } => {
                let t = self.resolve_ty(ty, false)?;
                if t.int().is_none() {
                    return Err(err(
                        Code::E0306,
                        type_span(ty),
                        format!("`for` variable must have an integer type, found `{t}`"),
                    ));
                }
                self.check(start, &t)?;
                self.check(end, &t)?;
                self.scopes.push(Vec::new());
                self.declare(var, t, false)?;
                self.loop_depth += 1;
                self.block(body)?;
                self.loop_depth -= 1;
                self.scopes.pop();
                Ok(false)
            }
            StmtKind::While {
                cond,
                invariants,
                decreases,
                body,
            } => {
                self.check(cond, &Ty::Bool)?;
                for c in invariants {
                    self.clause(c, TrapKind::Invariant, false, true)?;
                }
                self.clause(decreases, TrapKind::Decreases, false, true)?;
                self.loop_depth += 1;
                self.block(body)?;
                self.loop_depth -= 1;
                Ok(false)
            }
            StmtKind::Return(e) => {
                let ret = self.ret.clone();
                self.check(e, &ret)?;
                Ok(true)
            }
            StmtKind::Break | StmtKind::Continue => {
                if self.loop_depth == 0 {
                    let kw = if matches!(s.kind, StmtKind::Break) {
                        "break"
                    } else {
                        "continue"
                    };
                    return Err(err(Code::E0315, s.span, format!("`{kw}` outside a loop")));
                }
                Ok(true)
            }
            StmtKind::Block(b) => self.block(b),
            StmtKind::Assign { place, value } => {
                let t = self.place(place)?;
                self.check(value, &t)?;
                Ok(false)
            }
            StmtKind::Compound { op, place, value } => {
                let t = self.place(place)?;
                let Some(i) = t.int() else {
                    return Err(err(
                        Code::E0306,
                        place.span,
                        format!("`{}` needs an integer place, found `{t}`", op.as_str()),
                    ));
                };
                self.check(value, &t)?;
                self.trap(TrapKind::Overflow, place.span.to(value.span), i.name());
                Ok(false)
            }
            StmtKind::Expr(e) => {
                if !matches!(e.kind, ExprKind::Call(..)) {
                    return Err(err(
                        Code::E0316,
                        e.span,
                        "expression statement must be a call returning `()`",
                    ));
                }
                let t = self.infer(e, None)?;
                if t != Ty::Unit {
                    return Err(err(
                        Code::E0316,
                        e.span,
                        format!("result of type `{t}` is discarded"),
                    ));
                }
                Ok(false)
            }
        }
    }

    fn if_stmt(&mut self, i: &IfStmt) -> CResult<bool> {
        self.check(&i.cond, &Ty::Bool)?;
        let then = self.block(&i.then)?;
        let els = match &i.els {
            None => false,
            Some(Else::If(e)) => self.if_stmt(e)?,
            Some(Else::Block(b)) => self.block(b)?,
        };
        Ok(then && els && i.els.is_some())
    }

    // --------------------------------------------------------------- places

    /// Is `e` a path naming a local of reference type? Returns its mutability.
    fn ref_local(&self, e: &Expr) -> Option<bool> {
        let ExprKind::Path(p) = &e.kind else {
            return None;
        };
        if p.segments.len() != 1 || p.generic_args.is_some() {
            return None;
        }
        match &self.lookup(&p.segments[0].name)?.ty {
            Ty::Ref(m, _) => Some(*m),
            _ => None,
        }
    }

    fn place_info(&self, e: &Expr) -> Option<PlaceInfo> {
        match &e.kind {
            ExprKind::Path(p) if p.segments.len() == 1 && p.generic_args.is_none() => {
                let l = self.lookup(&p.segments[0].name)?;
                Some(PlaceInfo {
                    root: l.name.clone(),
                    assignable: l.mutable && !matches!(l.ty, Ty::Ref(..)),
                })
            }
            ExprKind::Field(b, _) | ExprKind::Index(b, _) | ExprKind::Slice(b, _, _) => {
                if let Some(m) = self.ref_local(b) {
                    let ExprKind::Path(p) = &b.kind else {
                        unreachable!()
                    };
                    Some(PlaceInfo {
                        root: p.segments[0].name.clone(),
                        assignable: m,
                    })
                } else {
                    self.place_info(b)
                }
            }
            _ => None,
        }
    }

    /// Is `e` a `.len` access on an array or slice? (Probe without recording traps.)
    fn is_len(&mut self, e: &Expr) -> bool {
        let ExprKind::Field(b, f) = &e.kind else {
            return false;
        };
        if f.name != "len" {
            return false;
        }
        let n = self.traps.len();
        let t = self.infer_base(b).map(deref);
        self.traps.truncate(n);
        matches!(t, Ok(Ty::Array(..) | Ty::Slice(..)))
    }

    /// Check an assignment target; returns its type.
    fn place(&mut self, e: &Expr) -> CResult<Ty> {
        let Some(info) = self.place_info(e) else {
            // Report unknown names etc. first.
            self.infer(e, None)?;
            return Err(err(
                Code::E0308,
                e.span,
                "left side of an assignment must be a local, field or element",
            ));
        };
        if self.is_len(e) {
            return Err(err(Code::E0308, e.span, "`.len` is read-only"));
        }
        if !info.assignable {
            return Err(err(
                Code::E0308,
                e.span,
                format!(
                    "cannot assign through `{}`: not `let mut`, or reached through `&` instead of `&mut`",
                    info.root
                ),
            ));
        }
        self.infer(e, None)
    }

    // ---------------------------------------------------------- expressions

    fn check(&mut self, e: &Expr, expected: &Ty) -> CResult<()> {
        let t = self.infer(e, Some(expected))?;
        if t != *expected {
            return Err(err(
                Code::E0303,
                e.span,
                format!("expected `{expected}`, found `{t}`"),
            ));
        }
        Ok(())
    }

    /// Type of an expression used as the base of `.f`, `.len`, `[i]`:
    /// reference locals are allowed here (one implicit dereference, §8.2).
    fn infer_base(&mut self, b: &Expr) -> CResult<Ty> {
        if self.ref_local(b).is_some() {
            let ExprKind::Path(p) = &b.kind else {
                unreachable!()
            };
            return Ok(self.lookup(&p.segments[0].name).expect("local").ty.clone());
        }
        self.infer(b, None)
    }

    fn int_literal_ty(
        &self,
        l: &crate::lexer::IntLit,
        span: Span,
        exp: Option<&Ty>,
    ) -> CResult<IntTy> {
        if let Some(s) = l.suffix {
            return Ok(IntTy::from_name(s.as_str()).expect("suffix is an int type"));
        }
        match exp {
            Some(Ty::Int(i)) => Ok(*i),
            Some(t) => Err(err(
                Code::E0303,
                span,
                format!("expected `{t}`, found integer literal `{}`", l.text),
            )),
            None => Err(err(
                Code::E0304,
                span,
                format!(
                    "cannot infer the type of `{}`; add a suffix (e.g. `{}u32`)",
                    l.text, l.text
                ),
            )),
        }
    }

    fn range_check(&self, v: i128, t: IntTy, span: Span, what: &str) -> CResult<()> {
        if t.fits(v) {
            return Ok(());
        }
        let note = if t.is_pointer_sized() {
            " (static checks use the 32-bit range portable to every target)"
        } else {
            ""
        };
        Err(err(
            Code::E0305,
            span,
            format!("{what} {v} does not fit `{}`{note}", t.name()),
        ))
    }

    fn infer(&mut self, e: &Expr, exp: Option<&Ty>) -> CResult<Ty> {
        let t = self.infer_kind(e, exp)?;
        self.types.insert((e.span.start, e.span.end), t.clone());
        Ok(t)
    }

    fn infer_kind(&mut self, e: &Expr, exp: Option<&Ty>) -> CResult<Ty> {
        match &e.kind {
            ExprKind::Int(l) => {
                let t = self.int_literal_ty(l, e.span, exp)?;
                self.range_check(i128::from(l.value), t, e.span, "literal")?;
                Ok(Ty::Int(t))
            }
            ExprKind::Bool(_) => Ok(Ty::Bool),
            ExprKind::Unit => Ok(Ty::Unit),
            ExprKind::Hole(h) => match exp {
                Some(t) => {
                    let locals: Vec<String> = self
                        .scopes
                        .iter()
                        .flat_map(|s| s.iter())
                        .map(|l| format!("{}: {}", l.name, l.ty))
                        .collect();
                    let scope = if locals.is_empty() {
                        "none".to_string()
                    } else {
                        locals.join(", ")
                    };
                    Err(err(
                        Code::E0321,
                        e.span,
                        format!("unfilled hole `?{h}`: expected `{t}`; locals in scope: {scope}"),
                    ))
                }
                None => Err(err(
                    Code::E0304,
                    e.span,
                    format!("hole `?{h}` needs an expected type"),
                )),
            },
            ExprKind::Path(p) => self.path_value(p, e.span),
            ExprKind::StructLit { path, fields } => self.struct_lit(path, fields, e.span),
            ExprKind::Paren(inner) => self.infer(inner, exp),
            ExprKind::Array(es) => self.array_lit(es, exp, e.span),
            ExprKind::Field(b, f) => {
                let bt = deref(self.infer_base(b)?);
                match &bt {
                    Ty::Struct(n) => {
                        let fields = &self.structs[n];
                        fields
                            .iter()
                            .find(|(fname, _)| *fname == f.name)
                            .map(|(_, t)| t.clone())
                            .ok_or_else(|| {
                                err(
                                    Code::E0311,
                                    f.span,
                                    format!("type `{n}` has no field `{}`", f.name),
                                )
                            })
                    }
                    Ty::Array(..) | Ty::Slice(..) if f.name == "len" => Ok(Ty::Int(IntTy::Usize)),
                    Ty::Array(..) | Ty::Slice(..) => Err(err(
                        Code::E0311,
                        f.span,
                        format!(
                            "`{bt}` has no field `{}` (arrays and slices only have `.len`)",
                            f.name
                        ),
                    )),
                    _ => Err(err(
                        Code::E0306,
                        f.span,
                        format!("type `{bt}` has no fields"),
                    )),
                }
            }
            ExprKind::Index(b, i) => {
                let bt = deref(self.infer_base(b)?);
                let (elem, len) = seq_parts(&bt).ok_or_else(|| {
                    err(
                        Code::E0306,
                        b.span,
                        format!("cannot index a value of type `{bt}`"),
                    )
                })?;
                self.check(i, &Ty::Int(IntTy::Usize))?;
                match (len, lit_value(i)) {
                    (Some(n), Some(v)) => {
                        if v >= i128::from(n) {
                            return Err(err(
                                Code::E0305,
                                i.span,
                                format!("index {v} is out of bounds for length {n}"),
                            ));
                        }
                    }
                    _ => self.trap(TrapKind::Bounds, e.span, bt.to_string()),
                }
                Ok(elem)
            }
            ExprKind::Slice(..) => Err(err(
                Code::E0325,
                e.span,
                "range index `p[lo..hi]` is only allowed directly under `&` or `&mut`",
            )),
            ExprKind::Ref(..) => Err(err(
                Code::E0319,
                e.span,
                "references can only be created as call arguments in phase 1",
            )),
            ExprKind::Call(f, args) => self.call(f, args),
            ExprKind::Cast(b, target) => self.cast(b, target, e.span),
            ExprKind::Unary(UnOp::Neg, inner) => self.neg(inner, exp, e.span),
            ExprKind::Unary(UnOp::Not, inner) => {
                let t = match exp {
                    Some(t @ (Ty::Int(_) | Ty::Bool)) => {
                        self.check(inner, t)?;
                        t.clone()
                    }
                    _ => self.infer(inner, None)?,
                };
                match t {
                    Ty::Int(_) | Ty::Bool => Ok(t),
                    _ => Err(err(
                        Code::E0306,
                        e.span,
                        format!("`!` needs `bool` or an integer, found `{t}`"),
                    )),
                }
            }
            ExprKind::Binary(BinOp::Implies, ..) => Err(spec_only(e.span, "`==>`")),
            ExprKind::Binary(op, l, r) => self.binary(*op, l, r, exp, e.span),
            ExprKind::Result => Err(spec_only(e.span, "`result`")),
            ExprKind::Old(_) => Err(spec_only(e.span, "`old`")),
            ExprKind::Quant(q) => Err(spec_only(e.span, &format!("`{}`", q.keyword()))),
        }
    }

    fn path_value(&mut self, p: &ExprPath, span: Span) -> CResult<Ty> {
        if p.generic_args.is_some() {
            return Err(err(
                Code::E0318,
                span,
                "generic arguments are not available in phase 1",
            ));
        }
        if p.segments.len() > 1 {
            return Err(err(
                Code::E0318,
                span,
                "paths with `::` need modules, which are not available in phase 1",
            ));
        }
        let name = &p.segments[0].name;
        if let Some(l) = self.lookup(name) {
            if matches!(l.ty, Ty::Ref(..)) {
                return Err(err(
                    Code::E0319,
                    span,
                    format!("reference `{name}` can only be used as `{name}.f`, `{name}[i]`, `{name}.len`, or passed on as a call argument"),
                ));
            }
            return Ok(l.ty.clone());
        }
        match self.items.get(name) {
            Some(ItemRef::Fn) => Err(err(
                Code::E0303,
                span,
                format!("`{name}` is a function; functions are not values (call it: `{name}(…)`)"),
            )),
            Some(ItemRef::Type) => Err(err(
                Code::E0303,
                span,
                format!("`{name}` is a type, not a value (struct literal: `{name}.{{ … }}`)"),
            )),
            None if BUILTIN_TYPES.contains(&name.as_str()) => Err(err(
                Code::E0303,
                span,
                format!("`{name}` is a type, not a value"),
            )),
            None => Err(err(Code::E0301, span, format!("unknown name `{name}`"))),
        }
    }

    fn struct_lit(&mut self, path: &ExprPath, fields: &[(Ident, Expr)], span: Span) -> CResult<Ty> {
        if path.generic_args.is_some() {
            return Err(err(
                Code::E0318,
                span,
                "generic arguments are not available in phase 1",
            ));
        }
        if path.segments.len() > 1 {
            return Err(err(
                Code::E0318,
                span,
                "paths with `::` need modules, which are not available in phase 1",
            ));
        }
        let name = &path.segments[0];
        match self.items.get(&name.name) {
            Some(ItemRef::Type) => {}
            Some(ItemRef::Fn) => {
                return Err(err(
                    Code::E0303,
                    name.span,
                    format!("`{}` is a function, not a type", name.name),
                ))
            }
            None => {
                return Err(err(
                    Code::E0301,
                    name.span,
                    format!("unknown type `{}`", name.name),
                ))
            }
        }
        let decl = self.structs[&name.name].clone();
        let given: Vec<&str> = fields.iter().map(|(n, _)| n.name.as_str()).collect();
        let wanted: Vec<&str> = decl.iter().map(|(n, _)| n.as_str()).collect();
        if given != wanted {
            return Err(err(
                Code::E0312,
                span,
                format!(
                    "struct literal must list every field of `{}` once, in declaration order: {}",
                    name.name,
                    wanted.join(", ")
                ),
            ));
        }
        for ((_, v), (_, t)) in fields.iter().zip(&decl) {
            self.check(v, t)?;
        }
        Ok(Ty::Struct(name.name.clone()))
    }

    fn array_lit(&mut self, es: &[Expr], exp: Option<&Ty>, span: Span) -> CResult<Ty> {
        if let Some(Ty::Array(t, n)) = exp {
            if es.len() as u64 != *n {
                return Err(err(
                    Code::E0323,
                    span,
                    format!("expected {n} elements, found {}", es.len()),
                ));
            }
            for x in es {
                self.check(x, t)?;
            }
            return Ok(Ty::Array(t.clone(), *n));
        }
        let Some((first, rest)) = es.split_first() else {
            return Err(err(
                Code::E0322,
                span,
                "array literals need at least one element",
            ));
        };
        let t = self.infer(first, None)?;
        for x in rest {
            self.check(x, &t)?;
        }
        Ok(Ty::Array(Box::new(t), es.len() as u64))
    }

    fn cast(&mut self, b: &Expr, target: &Ident, span: Span) -> CResult<Ty> {
        let Some(to) = IntTy::from_name(&target.name) else {
            let known = target.name == "bool" || self.items.contains_key(&target.name);
            return Err(if known {
                err(
                    Code::E0307,
                    target.span,
                    format!(
                        "cannot cast to `{}`: `as` converts between integer types only",
                        target.name
                    ),
                )
            } else {
                err(
                    Code::E0301,
                    target.span,
                    format!("unknown type `{}`", target.name),
                )
            });
        };
        let from = self.infer(b, None)?;
        let Some(from_i) = from.int() else {
            return Err(err(
                Code::E0307,
                b.span,
                format!("cannot cast `{from}`: `as` converts between integer types only"),
            ));
        };
        if let Some(v) = lit_value(b) {
            self.range_check(v, to, span, "value")?;
        } else if !from_i.widens_to(to) {
            self.trap(
                TrapKind::Cast,
                span,
                format!("{} -> {}", from_i.name(), to.name()),
            );
        }
        Ok(Ty::Int(to))
    }

    fn neg(&mut self, inner: &Expr, exp: Option<&Ty>, span: Span) -> CResult<Ty> {
        // `-` directly on a literal is folded (§4.1).
        if let ExprKind::Int(l) = &inner.kind {
            let t = self.int_literal_ty(l, span, exp)?;
            if !t.signed() {
                return Err(err(
                    Code::E0306,
                    span,
                    format!("cannot negate unsigned `{}`", t.name()),
                ));
            }
            self.range_check(-i128::from(l.value), t, span, "literal")?;
            return Ok(Ty::Int(t));
        }
        let t = match exp {
            Some(Ty::Int(i)) => {
                self.check(inner, &Ty::Int(*i))?;
                Ty::Int(*i)
            }
            _ => self.infer(inner, None)?,
        };
        match t.int() {
            Some(i) if i.signed() => {
                if let Some(v) = lit_value(inner) {
                    self.range_check(-v, i, span, "value")?;
                } else {
                    self.trap(TrapKind::Negate, span, i.name());
                }
                Ok(t)
            }
            _ => Err(err(
                Code::E0306,
                span,
                format!("cannot negate `{t}` (only signed integers)"),
            )),
        }
    }

    /// Two operands of the same type, no expected type (§4.2).
    fn pair(&mut self, l: &Expr, r: &Expr) -> CResult<Ty> {
        match (needs_expected(l), needs_expected(r)) {
            (true, true) => Err(err(
                Code::E0304,
                l.span,
                "cannot infer the operand type; add a suffix to one literal",
            )),
            (true, false) => {
                let t = self.infer(r, None)?;
                self.check(l, &t)?;
                Ok(t)
            }
            _ => {
                let t = self.infer(l, None)?;
                self.check(r, &t)?;
                Ok(t)
            }
        }
    }

    fn binary(
        &mut self,
        op: BinOp,
        l: &Expr,
        r: &Expr,
        exp: Option<&Ty>,
        span: Span,
    ) -> CResult<Ty> {
        use BinOp::*;
        match op {
            And | Or => {
                self.check(l, &Ty::Bool)?;
                self.check(r, &Ty::Bool)?;
                Ok(Ty::Bool)
            }
            Eq | Ne | Lt | Le | Gt | Ge => {
                let t = self.pair(l, r)?;
                let ok = match op {
                    Eq | Ne => matches!(t, Ty::Int(_) | Ty::Bool),
                    _ => matches!(t, Ty::Int(_)),
                };
                if !ok {
                    return Err(err(
                        Code::E0306,
                        span,
                        format!("`{}` is not defined for `{t}`", op.as_str()),
                    ));
                }
                Ok(Ty::Bool)
            }
            _ => {
                let t = match exp {
                    Some(Ty::Int(i)) => {
                        let t = Ty::Int(*i);
                        self.check(l, &t)?;
                        self.check(r, &t)?;
                        t
                    }
                    _ => self.pair(l, r)?,
                };
                let Some(i) = t.int() else {
                    let hint = if t == Ty::Bool && matches!(op, BitAnd | BitOr | BitXor) {
                        " (for `bool` use `&&`, `||` or `!=`)"
                    } else {
                        ""
                    };
                    return Err(err(
                        Code::E0306,
                        span,
                        format!(
                            "`{}` needs integer operands, found `{t}`{hint}",
                            op.as_str()
                        ),
                    ));
                };
                self.arith_traps(op, i, l, r, span)?;
                Ok(t)
            }
        }
    }

    fn arith_traps(&mut self, op: BinOp, t: IntTy, l: &Expr, r: &Expr, span: Span) -> CResult<()> {
        use BinOp::*;
        let (a, b) = (lit_value(l), lit_value(r));
        let constant = |v: Option<i128>| -> CResult<()> {
            match v {
                Some(v) if t.fits(v) => Ok(()),
                _ => Err(err(
                    Code::E0305,
                    span,
                    format!("constant expression overflows `{}`", t.name()),
                )),
            }
        };
        match op {
            Add | Sub | Mul => match (a, b) {
                (Some(a), Some(b)) => constant(match op {
                    Add => a.checked_add(b),
                    Sub => a.checked_sub(b),
                    _ => a.checked_mul(b),
                })?,
                _ => self.trap(TrapKind::Overflow, span, t.name()),
            },
            Div | Rem => {
                match b {
                    Some(0) => return Err(err(Code::E0305, r.span, "division by zero")),
                    Some(_) => {}
                    None => self.trap(TrapKind::DivZero, span, t.name()),
                }
                if t.signed() {
                    let (min, _) = t.portable_range();
                    match (a, b) {
                        (Some(a), Some(b)) if a == min && b == -1 => {
                            return Err(err(
                                Code::E0305,
                                span,
                                format!("`{}::MIN / -1` overflows", t.name()),
                            ))
                        }
                        (Some(_), Some(_)) => {}
                        _ => self.trap(TrapKind::DivOverflow, span, t.name()),
                    }
                }
            }
            Shl | Shr => {
                let bits = i128::from(t.min_bits());
                let amount_ok = match b {
                    Some(n) if n < 0 || n >= bits => {
                        return Err(err(
                            Code::E0305,
                            r.span,
                            format!("shift amount {n} out of range for `{}`", t.name()),
                        ))
                    }
                    Some(_) => true,
                    None => false,
                };
                let lossless_ok = match (op, a, b) {
                    (Shr, _, _) => true,
                    (_, Some(a), Some(n)) => {
                        constant(a.checked_mul(1i128 << n))?;
                        true
                    }
                    _ => false,
                };
                if !amount_ok || !lossless_ok {
                    self.trap(TrapKind::Shift, span, t.name());
                }
            }
            _ => {} // & | ^ are total
        }
        Ok(())
    }

    // ------------------------------------------------------- spec contexts
    //
    // `spec/contracts.md` §5. Spec expressions record no traps: their
    // well-formedness conditions are obligations, and the whole clause is one
    // report entry.

    fn clause(&mut self, e: &Expr, kind: TrapKind, result: bool, old: bool) -> CResult<()> {
        self.spec = Some(SpecCtx {
            clause: kind.as_str(),
            result,
            old,
            in_old: false,
        });
        let exp = if kind == TrapKind::Decreases {
            Ty::MathInt
        } else {
            Ty::Bool
        };
        let t = self.spec_expr(e, Some(&exp));
        self.spec = None;
        let t = t?;
        let ok = if kind == TrapKind::Decreases {
            t.is_integer()
        } else {
            t == Ty::Bool
        };
        if !ok {
            let want = if kind == TrapKind::Decreases {
                "an integer"
            } else {
                "`bool`"
            };
            return Err(err(
                Code::E0303,
                e.span,
                format!("`{}` needs {want}, found `{t}`", kind.as_str()),
            ));
        }
        self.trap(kind, e.span, "");
        Ok(())
    }

    fn ctx(&self) -> SpecCtx {
        self.spec.expect("inside a spec context")
    }

    fn spec_check(&mut self, e: &Expr, expected: &Ty) -> CResult<()> {
        let t = self.spec_expr(e, Some(expected))?;
        if t != *expected {
            return Err(err(
                Code::E0303,
                e.span,
                format!("expected `{expected}`, found `{t}`"),
            ));
        }
        Ok(())
    }

    /// An integer operand (machine type or `int`), lifted by the caller.
    fn spec_int(&mut self, e: &Expr, what: &str) -> CResult<Ty> {
        let t = self.spec_expr(e, None)?;
        if !t.is_integer() {
            return Err(err(
                Code::E0306,
                e.span,
                format!("{what} needs an integer, found `{t}`"),
            ));
        }
        Ok(t)
    }

    /// Base of `.f`, `.len`, `[i]` in a spec context (one implicit deref).
    fn spec_base(&mut self, b: &Expr) -> CResult<Ty> {
        if self.ref_local(b).is_some() {
            let ExprKind::Path(p) = &b.kind else {
                unreachable!()
            };
            // Reference locals are parameters, so `old` may read them.
            return Ok(deref(
                self.lookup(&p.segments[0].name).expect("local").ty.clone(),
            ));
        }
        self.spec_expr(b, None).map(deref)
    }

    fn spec_expr(&mut self, e: &Expr, exp: Option<&Ty>) -> CResult<Ty> {
        let t = self.spec_kind(e, exp)?;
        self.types.insert((e.span.start, e.span.end), t.clone());
        Ok(t)
    }

    fn spec_kind(&mut self, e: &Expr, exp: Option<&Ty>) -> CResult<Ty> {
        match &e.kind {
            ExprKind::Int(l) => match (l.suffix, exp) {
                (None, Some(Ty::Int(_))) | (Some(_), _) => self.infer(e, exp),
                (None, _) => Ok(Ty::MathInt),
            },
            ExprKind::Bool(_) | ExprKind::Unit | ExprKind::Hole(_) => self.infer(e, exp),
            ExprKind::Path(p) => {
                let t = self.path_value(p, e.span)?;
                let ctx = self.ctx();
                if ctx.in_old {
                    let l = self.lookup(&p.segments[0].name).expect("local");
                    if !l.at_entry {
                        return Err(err(
                            Code::E0327,
                            e.span,
                            format!(
                                "`old` can only read parameters; `{}` does not exist at function entry",
                                l.name
                            ),
                        ));
                    }
                }
                Ok(t)
            }
            ExprKind::Paren(inner) => self.spec_expr(inner, exp),
            ExprKind::Field(b, f) => {
                let bt = self.spec_base(b)?;
                match &bt {
                    Ty::Array(..) | Ty::Slice(..) if f.name == "len" => Ok(Ty::Int(IntTy::Usize)),
                    Ty::Struct(n) => self.structs[n]
                        .iter()
                        .find(|(fname, _)| *fname == f.name)
                        .map(|(_, t)| t.clone())
                        .ok_or_else(|| {
                            err(
                                Code::E0311,
                                f.span,
                                format!("type `{n}` has no field `{}`", f.name),
                            )
                        }),
                    Ty::Array(..) | Ty::Slice(..) => Err(err(
                        Code::E0311,
                        f.span,
                        format!(
                            "`{bt}` has no field `{}` (arrays and slices only have `.len`)",
                            f.name
                        ),
                    )),
                    _ => Err(err(
                        Code::E0306,
                        f.span,
                        format!("type `{bt}` has no fields"),
                    )),
                }
            }
            ExprKind::Index(b, i) => {
                let bt = self.spec_base(b)?;
                let (elem, len) = seq_parts(&bt).ok_or_else(|| {
                    err(
                        Code::E0306,
                        b.span,
                        format!("cannot index a value of type `{bt}`"),
                    )
                })?;
                self.spec_int(i, "an index")?;
                if let Some(v) = lit_value(i) {
                    if v < 0 || len.is_some_and(|n| v >= i128::from(n)) {
                        return Err(err(
                            Code::E0305,
                            i.span,
                            format!("index {v} is out of bounds for `{bt}`"),
                        ));
                    }
                }
                Ok(elem)
            }
            ExprKind::Slice(..) => self.infer(e, exp),
            ExprKind::Ref(..) => Err(err(
                Code::E0319,
                e.span,
                "references cannot be created in spec contexts",
            )),
            ExprKind::Call(..) => Err(err(
                Code::E0329,
                e.span,
                "calls are not allowed in spec contexts (there is no `spec fn` yet)",
            )),
            ExprKind::Cast(..) => Err(err(
                Code::E0318,
                e.span,
                "casts in spec contexts are not available yet",
            )),
            ExprKind::Array(_) | ExprKind::StructLit { .. } => Err(err(
                Code::E0318,
                e.span,
                "array and struct literals in spec contexts are not available yet",
            )),
            ExprKind::Unary(UnOp::Neg, inner) => {
                self.spec_int(inner, "`-`")?;
                Ok(Ty::MathInt)
            }
            ExprKind::Unary(UnOp::Not, inner) => {
                let t = match exp {
                    Some(t @ (Ty::Int(_) | Ty::Bool)) => {
                        self.spec_check(inner, t)?;
                        t.clone()
                    }
                    _ => self.spec_expr(inner, None)?,
                };
                match t {
                    Ty::Int(_) | Ty::Bool => Ok(t),
                    _ => Err(err(
                        Code::E0306,
                        e.span,
                        format!("`!` needs `bool` or a machine integer, found `{t}`"),
                    )),
                }
            }
            ExprKind::Binary(op, l, r) => self.spec_binary(*op, l, r, exp, e.span),
            ExprKind::Result => {
                let ctx = self.ctx();
                if !ctx.result || ctx.in_old {
                    let why = if ctx.in_old {
                        "inside `old`".to_string()
                    } else if ctx.clause == "ensures" {
                        "in a function returning `()`".to_string()
                    } else {
                        format!("in `{}`", ctx.clause)
                    };
                    return Err(err(
                        Code::E0328,
                        e.span,
                        format!("`result` is only allowed in `ensures` of a function with a result, not {why}"),
                    ));
                }
                Ok(self.ret.clone())
            }
            ExprKind::Old(inner) => {
                let ctx = self.ctx();
                if !ctx.old || ctx.in_old {
                    let why = if ctx.in_old {
                        "`old` cannot be nested"
                    } else {
                        "`old` is not allowed in `requires` (it already reads the entry state)"
                    };
                    return Err(err(Code::E0327, e.span, why));
                }
                self.spec = Some(SpecCtx {
                    in_old: true,
                    ..ctx
                });
                let t = self.spec_expr(inner, exp);
                self.spec = Some(ctx);
                t
            }
            ExprKind::Quant(q) => {
                let is_int = matches!(&q.ty, Type::Named { path, args, .. }
                    if path.len() == 1 && args.is_empty() && path[0].name == "int");
                if !is_int {
                    return Err(err(
                        Code::E0318,
                        type_span(&q.ty),
                        format!(
                            "quantifier binders must have type `int`, found `{}`",
                            crate::fmt::ty(&q.ty)
                        ),
                    ));
                }
                self.spec_int(&q.lo, "a quantifier range")?;
                self.spec_int(&q.hi, "a quantifier range")?;
                self.scopes.push(Vec::new());
                let r = self.declare(&q.var, Ty::MathInt, false).and_then(|()| {
                    self.mark_at_entry();
                    self.spec_check(&q.body, &Ty::Bool)
                });
                self.scopes.pop();
                r?;
                Ok(Ty::Bool)
            }
        }
    }

    fn spec_binary(
        &mut self,
        op: BinOp,
        l: &Expr,
        r: &Expr,
        exp: Option<&Ty>,
        span: Span,
    ) -> CResult<Ty> {
        use BinOp::*;
        match op {
            And | Or | Implies => {
                self.spec_check(l, &Ty::Bool)?;
                self.spec_check(r, &Ty::Bool)?;
                Ok(Ty::Bool)
            }
            Eq | Ne | Lt | Le | Gt | Ge => {
                let lt = self.spec_expr(l, None)?;
                let rt = self.spec_expr(r, None)?;
                let ok = (lt.is_integer() && rt.is_integer())
                    || (matches!(op, Eq | Ne) && lt == Ty::Bool && rt == Ty::Bool);
                if !ok {
                    let msg = if lt == rt || lt.is_integer() == rt.is_integer() {
                        format!("`{}` is not defined for `{lt}` and `{rt}`", op.as_str())
                    } else {
                        format!("cannot compare `{lt}` with `{rt}`")
                    };
                    return Err(err(Code::E0306, span, msg));
                }
                Ok(Ty::Bool)
            }
            Add | Sub | Mul | Div | Rem => {
                self.spec_int(l, &format!("`{}`", op.as_str()))?;
                self.spec_int(r, &format!("`{}`", op.as_str()))?;
                if matches!(op, Div | Rem) && lit_value(r) == Some(0) {
                    return Err(err(Code::E0305, r.span, "division by zero"));
                }
                Ok(Ty::MathInt)
            }
            BitAnd | BitOr | BitXor | Shl | Shr => {
                // Machine type, bit-vector meaning; unsuffixed literals take
                // the other operand's type (`spec/contracts.md` §5.1).
                let t = match (needs_expected(l), needs_expected(r), exp) {
                    (true, true, Some(t @ Ty::Int(_))) => {
                        self.spec_check(l, t)?;
                        self.spec_check(r, t)?;
                        t.clone()
                    }
                    (true, true, _) => {
                        return Err(err(
                            Code::E0304,
                            l.span,
                            "cannot infer the operand type; add a suffix to one literal",
                        ))
                    }
                    (true, false, _) => {
                        let t = self.spec_bit_operand(op, r, span)?;
                        self.spec_check(l, &t)?;
                        t
                    }
                    _ => {
                        let t = self.spec_bit_operand(op, l, span)?;
                        self.spec_check(r, &t)?;
                        t
                    }
                };
                let i = t.int().expect("machine integer");
                if matches!(op, Shl | Shr) {
                    if let Some(n) = lit_value(r) {
                        if n < 0 || n >= i128::from(i.min_bits()) {
                            return Err(err(
                                Code::E0305,
                                r.span,
                                format!("shift amount {n} out of range for `{}`", i.name()),
                            ));
                        }
                    }
                }
                Ok(t)
            }
        }
    }

    /// First operand of a bitwise operator: must be a machine integer.
    fn spec_bit_operand(&mut self, op: BinOp, e: &Expr, span: Span) -> CResult<Ty> {
        let t = self.spec_expr(e, None)?;
        if t.int().is_none() {
            return Err(err(
                Code::E0306,
                span,
                format!(
                    "`{}` needs machine-integer operands of one type, found `{t}` (bitwise operators do not lift to `int`)",
                    op.as_str()
                ),
            ));
        }
        Ok(t)
    }

    // ---------------------------------------------------------------- calls

    fn call(&mut self, f: &Expr, args: &[Expr]) -> CResult<Ty> {
        let ExprKind::Path(p) = &f.kind else {
            return Err(err(
                Code::E0310,
                f.span,
                "call target must be a function name",
            ));
        };
        if p.generic_args.is_some() {
            return Err(err(
                Code::E0318,
                f.span,
                "generic arguments are not available in phase 1",
            ));
        }
        if p.segments.len() > 1 {
            return Err(err(
                Code::E0318,
                f.span,
                "paths with `::` need modules, which are not available in phase 1",
            ));
        }
        let name = &p.segments[0].name;
        if self.lookup(name).is_some() || self.items.get(name) == Some(&ItemRef::Type) {
            return Err(err(
                Code::E0310,
                f.span,
                format!("`{name}` is not a function"),
            ));
        }
        let Some(sig) = self.fns.get(name).cloned() else {
            return Err(err(
                Code::E0301,
                f.span,
                format!("unknown function `{name}`"),
            ));
        };
        if sig.params.len() != args.len() {
            return Err(err(
                Code::E0309,
                f.span,
                format!(
                    "`{name}` takes {} argument(s), found {}",
                    sig.params.len(),
                    args.len()
                ),
            ));
        }
        self.calls
            .entry(self.cur.clone())
            .or_default()
            .push((name.clone(), f.span));
        for (a, pt) in args.iter().zip(&sig.params) {
            let t = match &a.kind {
                ExprKind::Ref(m, inner) => self.make_ref(*m, inner, a.span)?,
                _ if self.ref_local(a).is_some() => {
                    let ExprKind::Path(p) = &a.kind else {
                        unreachable!()
                    };
                    self.lookup(&p.segments[0].name).expect("local").ty.clone()
                }
                _ => {
                    self.check(a, pt)?;
                    continue;
                }
            };
            if t != *pt {
                return Err(err(
                    Code::E0303,
                    a.span,
                    format!("expected `{pt}`, found `{t}`"),
                ));
            }
        }
        self.aliasing(args)?;
        Ok(sig.ret)
    }

    fn make_ref(&mut self, m: bool, inner: &Expr, span: Span) -> CResult<Ty> {
        let target = match &inner.kind {
            ExprKind::Slice(b, _, _) => b.as_ref(),
            _ => inner,
        };
        let info = self.place_info(inner).ok_or_else(|| {
            err(
                Code::E0308,
                span,
                "can only take a reference to a local, field or element",
            )
        })?;
        if self.is_len(target) {
            return Err(err(Code::E0308, span, "cannot take a reference to `.len`"));
        }
        if m && !info.assignable {
            return Err(err(
                Code::E0308,
                span,
                format!(
                    "cannot take `&mut` through `{}`: not `let mut`, or reached through `&` instead of `&mut`",
                    info.root
                ),
            ));
        }
        let ExprKind::Slice(b, lo, hi) = &inner.kind else {
            let t = self.infer(inner, None)?;
            return Ok(Ty::Ref(m, Box::new(t)));
        };
        let bt = deref(self.infer_base(b)?);
        let (elem, len) = seq_parts(&bt).ok_or_else(|| {
            err(
                Code::E0306,
                b.span,
                format!("cannot slice a value of type `{bt}`"),
            )
        })?;
        let usize_t = Ty::Int(IntTy::Usize);
        self.check(lo, &usize_t)?;
        self.check(hi, &usize_t)?;
        match (len, lit_value(lo), lit_value(hi)) {
            (Some(n), Some(l), Some(h)) => {
                if l > h || h > i128::from(n) {
                    return Err(err(
                        Code::E0305,
                        inner.span,
                        format!("range {l}..{h} is out of bounds for length {n}"),
                    ));
                }
            }
            _ => self.trap(TrapKind::Slice, inner.span, bt.to_string()),
        }
        Ok(Ty::Ref(m, Box::new(Ty::Slice(Box::new(elem)))))
    }

    /// `&mut` arguments must not share a root local with any other argument (§8.4).
    fn aliasing(&self, args: &[Expr]) -> CResult<()> {
        for (i, a) in args.iter().enumerate() {
            let root = match &a.kind {
                ExprKind::Ref(true, inner) => self.place_info(inner).map(|p| p.root),
                _ if self.ref_local(a) == Some(true) => {
                    let ExprKind::Path(p) = &a.kind else {
                        unreachable!()
                    };
                    Some(p.segments[0].name.clone())
                }
                _ => None,
            };
            let Some(root) = root else { continue };
            for (j, other) in args.iter().enumerate() {
                if j != i && mentions(other, &root) {
                    return Err(err(
                        Code::E0320,
                        other.span,
                        format!(
                            "`{root}` is mutably borrowed by argument {} of this call",
                            i + 1
                        ),
                    ));
                }
            }
        }
        Ok(())
    }
}

fn deref(t: Ty) -> Ty {
    match t {
        Ty::Ref(_, inner) => *inner,
        t => t,
    }
}

/// Element type and static length (arrays only) of an indexable type.
fn seq_parts(t: &Ty) -> Option<(Ty, Option<u64>)> {
    match t {
        Ty::Array(e, n) => Some(((**e).clone(), Some(*n))),
        Ty::Slice(e) => Some(((**e).clone(), None)),
        _ => None,
    }
}

/// Value of an integer literal, possibly negated or parenthesized.
fn lit_value(e: &Expr) -> Option<i128> {
    match &e.kind {
        ExprKind::Int(l) => Some(i128::from(l.value)),
        ExprKind::Unary(UnOp::Neg, x) => lit_value(x).map(|v| -v),
        ExprKind::Paren(x) => lit_value(x),
        _ => None,
    }
}

/// Expressions that cannot synthesize a type (§4.2).
fn needs_expected(e: &Expr) -> bool {
    match &e.kind {
        ExprKind::Int(l) => l.suffix.is_none(),
        ExprKind::Hole(_) => true,
        ExprKind::Unary(_, x) | ExprKind::Paren(x) => needs_expected(x),
        ExprKind::Array(es) => es.first().is_none_or(needs_expected),
        _ => false,
    }
}

/// Does `e` mention local `name` anywhere?
fn mentions(e: &Expr, name: &str) -> bool {
    match &e.kind {
        ExprKind::Path(p) => p.segments.len() == 1 && p.segments[0].name == name,
        ExprKind::Int(_) | ExprKind::Bool(_) | ExprKind::Unit | ExprKind::Hole(_) => false,
        ExprKind::StructLit { fields, .. } => fields.iter().any(|(_, v)| mentions(v, name)),
        ExprKind::Paren(x)
        | ExprKind::Field(x, _)
        | ExprKind::Cast(x, _)
        | ExprKind::Unary(_, x) => mentions(x, name),
        ExprKind::Ref(_, x) => mentions(x, name),
        ExprKind::Array(es) => es.iter().any(|x| mentions(x, name)),
        ExprKind::Index(a, b) | ExprKind::Binary(_, a, b) => mentions(a, name) || mentions(b, name),
        ExprKind::Slice(a, b, c) => mentions(a, name) || mentions(b, name) || mentions(c, name),
        ExprKind::Call(f, args) => mentions(f, name) || args.iter().any(|x| mentions(x, name)),
        ExprKind::Result => false,
        ExprKind::Old(x) => mentions(x, name),
        ExprKind::Quant(q) => {
            mentions(&q.lo, name) || mentions(&q.hi, name) || mentions(&q.body, name)
        }
    }
}

fn spec_only(span: Span, what: &str) -> Diagnostic {
    err(
        Code::E0326,
        span,
        format!("{what} is only allowed in `requires`, `ensures`, `invariant` and `decreases`"),
    )
}

fn type_span(t: &Type) -> Span {
    match t {
        Type::Named { span, .. }
        | Type::Array { span, .. }
        | Type::Slice { span, .. }
        | Type::Ref { span, .. }
        | Type::Unit { span } => *span,
    }
}
