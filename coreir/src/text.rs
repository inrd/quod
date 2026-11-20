//! Canonical text form (`spec/core-ir.md` §3).
//!
//! A text is accepted only if printing the parsed program gives back the same
//! bytes, so canonicality needs no rules beyond the printer.

use crate::ast::*;
use crate::check::check;
use crate::Error;

pub const HEADER: &str = "(quod-core 0)";

type R<T> = Result<T, Error>;

/// Parse a canonical Core IR text and check its well-formedness.
pub fn parse(src: &str) -> R<Program> {
    let program = read(src)?;
    let printed = print(&program);
    if printed != src {
        let line = first_difference(src, &printed);
        return Err(Error::new(
            line,
            "not in canonical form (spec/core-ir.md §3.2)",
        ));
    }
    check(&program)?;
    Ok(program)
}

fn first_difference(a: &str, b: &str) -> usize {
    let mut la = a.split('\n');
    let mut lb = b.split('\n');
    let mut n = 1;
    loop {
        match (la.next(), lb.next()) {
            (Some(x), Some(y)) if x == y => n += 1,
            _ => return n,
        }
    }
}

// ------------------------------------------------------------------ reading

enum Sx {
    W(String, usize),
    L(Vec<Sx>, usize),
}

impl Sx {
    fn line(&self) -> usize {
        match self {
            Sx::W(_, l) | Sx::L(_, l) => *l,
        }
    }

    fn word(&self) -> R<&str> {
        match self {
            Sx::W(w, _) => Ok(w),
            Sx::L(_, l) => Err(Error::new(*l, "expected a word, found a list")),
        }
    }

    fn head(&self) -> Option<&str> {
        match self {
            Sx::L(items, _) => match items.first() {
                Some(Sx::W(w, _)) => Some(w),
                _ => None,
            },
            Sx::W(..) => None,
        }
    }

    /// Children after the head word of a list `(head …)`.
    fn form(&self, head: &str) -> R<&[Sx]> {
        match self {
            Sx::L(items, _) if self.head() == Some(head) => Ok(&items[1..]),
            _ => Err(Error::new(self.line(), format!("expected `({head} …)`"))),
        }
    }

    /// Children of `(head …)` with exactly `n` of them.
    fn form_n(&self, head: &str, n: usize) -> R<&[Sx]> {
        let rest = self.form(head)?;
        if rest.len() != n {
            return Err(Error::new(
                self.line(),
                format!("`({head} …)` takes {n} argument(s)"),
            ));
        }
        Ok(rest)
    }
}

fn read_tree(src: &str) -> R<Vec<Sx>> {
    let mut stack: Vec<(Vec<Sx>, usize)> = vec![(Vec::new(), 0)];
    let mut line = 1;
    let mut chars = src.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        match c {
            '\n' => line += 1,
            ' ' => {}
            '(' => stack.push((Vec::new(), line)),
            ')' => {
                if stack.len() == 1 {
                    return Err(Error::new(line, "unbalanced `)`"));
                }
                let (items, l) = stack.pop().expect("stack");
                stack.last_mut().expect("stack").0.push(Sx::L(items, l));
            }
            c if c.is_ascii_graphic() => {
                let mut end = i + 1;
                while let Some(&(j, d)) = chars.peek() {
                    if d == '(' || d == ')' || !d.is_ascii_graphic() {
                        break;
                    }
                    end = j + 1;
                    chars.next();
                }
                let top = stack.last_mut().expect("stack");
                top.0.push(Sx::W(src[i..end].to_string(), line));
            }
            _ => return Err(Error::new(line, "unexpected character")),
        }
    }
    if stack.len() != 1 {
        return Err(Error::new(line, "unclosed `(`"));
    }
    Ok(stack.pop().expect("stack").0)
}

fn read(src: &str) -> R<Program> {
    let trees = read_tree(src)?;
    let Some((first, fns)) = trees.split_first() else {
        return Err(Error::new(1, format!("expected `{HEADER}`")));
    };
    let v = first.form_n("quod-core", 1)?;
    if v[0].word()? != "0" {
        return Err(Error::new(1, "unsupported Core IR version"));
    }
    let fns = fns.iter().map(read_fn).collect::<R<Vec<_>>>()?;
    Ok(Program { fns })
}

fn read_fn(sx: &Sx) -> R<Fn> {
    let parts = sx.form_n("fn", 6)?;
    let name = read_name(&parts[0])?;
    let params = parts[1]
        .form("params")?
        .iter()
        .map(|p| match p {
            Sx::L(items, _) if items.len() == 2 => Ok((read_name(&items[0])?, read_ty(&items[1])?)),
            _ => Err(Error::new(p.line(), "expected `(name type)`")),
        })
        .collect::<R<Vec<_>>>()?;
    let ret = read_ty(&parts[2].form_n("ret", 1)?[0])?;
    let requires = read_terms(parts[3].form("requires")?)?;
    let ensures = read_terms(parts[4].form("ensures")?)?;
    let body = read_stmts(parts[5].form("body")?)?;
    Ok(Fn {
        name,
        params,
        ret,
        requires,
        ensures,
        body,
    })
}

fn read_name(sx: &Sx) -> R<String> {
    let w = sx.word()?;
    if !is_name(w) {
        return Err(Error::new(sx.line(), format!("`{w}` is not a valid name")));
    }
    Ok(w.to_string())
}

/// `ident`, `ident.N` (N ≥ 1) or `$N`; numbers in decimal without leading zeros.
pub fn is_name(w: &str) -> bool {
    let number = |s: &str, min: u64| {
        !s.is_empty()
            && s.bytes().all(|b| b.is_ascii_digit())
            && !(s.len() > 1 && s.starts_with('0'))
            && s.parse::<u64>().is_ok_and(|n| n >= min)
    };
    if let Some(n) = w.strip_prefix('$') {
        return number(n, 0);
    }
    let (ident, suffix) = match w.split_once('.') {
        Some((i, n)) => (i, Some(n)),
        None => (w, None),
    };
    let mut bytes = ident.bytes();
    let ident_ok = bytes
        .next()
        .is_some_and(|b| b.is_ascii_alphabetic() || b == b'_')
        && bytes.all(|b| b.is_ascii_alphanumeric() || b == b'_')
        && !matches!(ident, "true" | "false" | "result");
    ident_ok && suffix.is_none_or(|n| number(n, 1))
}

fn read_scalar(sx: &Sx) -> R<Scalar> {
    let w = sx.word()?;
    Scalar::from_name(w).ok_or_else(|| Error::new(sx.line(), format!("unknown type `{w}`")))
}

fn read_ty(sx: &Sx) -> R<Ty> {
    match sx {
        Sx::W(w, _) if w == "unit" => Ok(Ty::Unit),
        Sx::W(..) => Ok(Ty::Scalar(read_scalar(sx)?)),
        Sx::L(..) => match sx.head() {
            Some("array") => {
                let a = sx.form_n("array", 2)?;
                let n = read_u64(&a[1])?;
                Ok(Ty::Array(read_scalar(&a[0])?, n))
            }
            Some(h @ ("ref" | "mut")) => {
                let inner = &sx.form_n(h, 1)?[0];
                let elem = read_scalar(&inner.form_n("slice", 1)?[0])?;
                Ok(Ty::Slice {
                    mutable: h == "mut",
                    elem,
                })
            }
            _ => Err(Error::new(sx.line(), "unknown type form")),
        },
    }
}

fn read_u64(sx: &Sx) -> R<u64> {
    let w = sx.word()?;
    if !w.bytes().all(|b| b.is_ascii_digit()) {
        return Err(Error::new(sx.line(), format!("`{w}` is not a number")));
    }
    w.parse()
        .map_err(|_| Error::new(sx.line(), format!("`{w}` is out of range")))
}

/// Machine literal `123u32`.
fn read_lit(sx: &Sx) -> R<(u64, Scalar)> {
    let w = sx.word()?;
    let split = w.find(|c: char| !c.is_ascii_digit()).unwrap_or(w.len());
    let (digits, suffix) = w.split_at(split);
    let ty = Scalar::from_name(suffix)
        .filter(|s| s.is_int() && !digits.is_empty())
        .ok_or_else(|| Error::new(sx.line(), format!("`{w}` is not a machine literal")))?;
    let v = digits
        .parse()
        .map_err(|_| Error::new(sx.line(), format!("`{w}` is out of range")))?;
    Ok((v, ty))
}

fn read_atom(sx: &Sx) -> R<Atom> {
    let w = sx.word()?;
    Ok(match w {
        "true" => Atom::Bool(true),
        "false" => Atom::Bool(false),
        _ if w.starts_with(|c: char| c.is_ascii_digit()) => {
            let (v, t) = read_lit(sx)?;
            Atom::Lit(v, t)
        }
        _ => Atom::Name(read_name(sx)?),
    })
}

fn read_atoms(sxs: &[Sx]) -> R<Vec<Atom>> {
    sxs.iter().map(read_atom).collect()
}

/// `op.T` → (`op`, `T`).
fn split_op(w: &str) -> Option<(&str, Scalar)> {
    let (op, t) = w.split_once('.')?;
    Some((op, Scalar::from_name(t)?))
}

/// A list's head word and the items after it.
fn parts(sx: &Sx) -> Option<(&str, &[Sx])> {
    match sx {
        Sx::L(items, _) => Some((sx.head()?, &items[1..])),
        Sx::W(..) => None,
    }
}

fn bad_form(sx: &Sx, what: &str) -> Error {
    let head = sx.head().unwrap_or("?");
    Error::new(
        sx.line(),
        format!("unknown {what} or wrong arity: `({head} …)`"),
    )
}

fn read_rhs(sx: &Sx) -> R<Rhs> {
    let Some((head, args)) = parts(sx) else {
        return Ok(Rhs::Atom(read_atom(sx)?));
    };
    let a = read_atom;
    Ok(match (head, args) {
        ("len", [n]) => Rhs::Len(read_name(n)?),
        ("load", [n, i]) => Rhs::Load(read_name(n)?, a(i)?),
        ("array", [t, es @ ..]) => Rhs::Array(read_scalar(t)?, read_atoms(es)?),
        ("view" | "mview", [b, lo, hi]) => Rhs::View {
            mutable: head == "mview",
            base: read_name(b)?,
            lo: a(lo)?,
            hi: a(hi)?,
        },
        ("call", [f, xs @ ..]) => Rhs::Call(read_name(f)?, read_atoms(xs)?),
        _ => match (split_op(head), args) {
            (Some(("not", t)), [x]) => Rhs::Not(t, a(x)?),
            (Some((op, t)), [x, y]) if BinOp::from_name(op).is_some() => {
                Rhs::Bin(BinOp::from_name(op).expect("op"), t, a(x)?, a(y)?)
            }
            _ => return Err(bad_form(sx, "operation")),
        },
    })
}

fn read_stmts(sxs: &[Sx]) -> R<Vec<Stmt>> {
    sxs.iter().map(read_stmt).collect()
}

fn read_stmt(sx: &Sx) -> R<Stmt> {
    let (head, args) = parts(sx).ok_or_else(|| bad_form(sx, "statement"))?;
    Ok(match (head, args) {
        ("let", [n, t, r]) => Stmt::Let(read_name(n)?, read_ty(t)?, read_rhs(r)?),
        ("var", [n, t, r]) => Stmt::Var(read_name(n)?, read_ty(t)?, read_rhs(r)?),
        ("set", [n, x]) => Stmt::Set(read_name(n)?, read_atom(x)?),
        ("store", [n, i, v]) => Stmt::Store(read_name(n)?, read_atom(i)?, read_atom(v)?),
        ("call", [f, xs @ ..]) => Stmt::Call(read_name(f)?, read_atoms(xs)?),
        ("if", [c, t, e]) => Stmt::If(
            read_atom(c)?,
            read_stmts(t.form("then")?)?,
            read_stmts(e.form("else")?)?,
        ),
        ("loop", [h, c, inv @ .., d, b]) => Stmt::Loop(Box::new(Loop {
            head: read_stmts(h.form("head")?)?,
            cond: read_atom(&c.form_n("cond", 1)?[0])?,
            invariants: inv
                .iter()
                .map(|i| read_term(&i.form_n("invariant", 1)?[0]))
                .collect::<R<Vec<_>>>()?,
            decreases: read_term(&d.form_n("decreases", 1)?[0])?,
            body: read_stmts(b.form("body")?)?,
        })),
        ("return", []) => Stmt::Return(None),
        ("return", [x]) => Stmt::Return(Some(read_atom(x)?)),
        _ => return Err(bad_form(sx, "statement")),
    })
}

fn read_terms(sxs: &[Sx]) -> R<Vec<Term>> {
    sxs.iter().map(read_term).collect()
}

fn read_term(sx: &Sx) -> R<Term> {
    let Some((head, args)) = parts(sx) else {
        let w = sx.word()?;
        return Ok(match w {
            "result" => Term::Result,
            "true" => Term::Bool(true),
            "false" => Term::Bool(false),
            _ if w.starts_with(|c: char| c.is_ascii_digit())
                && w.contains(|c: char| c.is_ascii_alphabetic()) =>
            {
                let (v, t) = read_lit(sx)?;
                Term::Lit(v, t)
            }
            _ if w.starts_with(|c: char| c.is_ascii_digit() || c == '-') => Term::Int(
                w.parse()
                    .ok()
                    .filter(|v: &i128| v.unsigned_abs() <= u128::from(u64::MAX))
                    .ok_or_else(|| {
                        Error::new(sx.line(), format!("`{w}` is not an `int` literal"))
                    })?,
            ),
            _ => Term::Name(read_name(sx)?),
        });
    };
    let t = |x: &Sx| read_term(x).map(Box::new);
    Ok(match (head, args) {
        ("int", [x]) => Term::Lift(t(x)?),
        ("len", [x]) => Term::Len(t(x)?),
        ("old", [x]) => Term::Old(t(x)?),
        ("not", [x]) => Term::Not(t(x)?),
        ("sel", [s, k]) => Term::Sel(t(s)?, t(k)?),
        ("forall" | "exists", [v, lo, hi, body]) => Term::Quant(Box::new(Quant {
            forall: head == "forall",
            var: read_name(v)?,
            lo: read_term(lo)?,
            hi: read_term(hi)?,
            body: read_term(body)?,
        })),
        (_, [x, y]) if SpecOp::from_name(head).is_some() => {
            Term::Op(SpecOp::from_name(head).expect("op"), t(x)?, t(y)?)
        }
        _ => match (split_op(head), args) {
            (Some(("bnot", s)), [x]) => Term::BitNot(s, t(x)?),
            (Some((op, s)), [x, y]) if BinOp::from_name(op).is_some_and(BinOp::is_bitwise) => {
                Term::Bit(BinOp::from_name(op).expect("op"), s, t(x)?, t(y)?)
            }
            _ => return Err(bad_form(sx, "term")),
        },
    })
}

// ----------------------------------------------------------------- printing

/// Layout tree: a one-line list, or a block form (§3.2) whose children each
/// take their own line.
enum Doc {
    W(String),
    L(Vec<Doc>),
    B(Vec<Doc>, Vec<Doc>),
}

fn w(s: impl Into<String>) -> Doc {
    Doc::W(s.into())
}

fn render(d: &Doc, indent: usize, out: &mut String) {
    match d {
        Doc::W(s) => out.push_str(s),
        Doc::L(items) | Doc::B(items, _) => {
            out.push('(');
            for (i, x) in items.iter().enumerate() {
                if i > 0 {
                    out.push(' ');
                }
                render(x, indent, out);
            }
            if let Doc::B(_, children) = d {
                for c in children {
                    out.push('\n');
                    out.push_str(&" ".repeat(indent + 2));
                    render(c, indent + 2, out);
                }
            }
            out.push(')');
        }
    }
}

/// Print a program in canonical form.
pub fn print(p: &Program) -> String {
    let mut out = String::from(HEADER);
    out.push('\n');
    for f in &p.fns {
        out.push('\n');
        render(&fn_doc(f), 0, &mut out);
        out.push('\n');
    }
    out
}

fn block(head: &str, children: Vec<Doc>) -> Doc {
    Doc::B(vec![w(head)], children)
}

fn fn_doc(f: &Fn) -> Doc {
    let params = std::iter::once(w("params"))
        .chain(
            f.params
                .iter()
                .map(|(n, t)| Doc::L(vec![w(n.clone()), ty_doc(t)])),
        )
        .collect();
    Doc::B(
        vec![w("fn"), w(f.name.clone())],
        vec![
            Doc::L(params),
            Doc::L(vec![w("ret"), ty_doc(&f.ret)]),
            block("requires", f.requires.iter().map(term_doc).collect()),
            block("ensures", f.ensures.iter().map(term_doc).collect()),
            block("body", stmts_doc(&f.body)),
        ],
    )
}

fn ty_doc(t: &Ty) -> Doc {
    match t {
        Ty::Scalar(s) => w(s.name()),
        Ty::Unit => w("unit"),
        Ty::Int => w("int"),
        Ty::Array(s, n) => Doc::L(vec![w("array"), w(s.name()), w(n.to_string())]),
        Ty::Slice { mutable, elem } => Doc::L(vec![
            w(if *mutable { "mut" } else { "ref" }),
            Doc::L(vec![w("slice"), w(elem.name())]),
        ]),
    }
}

fn atom_doc(a: &Atom) -> Doc {
    match a {
        Atom::Name(n) => w(n.clone()),
        Atom::Lit(v, t) => w(format!("{v}{}", t.name())),
        Atom::Bool(b) => w(b.to_string()),
    }
}

fn list(head: impl Into<String>, rest: impl IntoIterator<Item = Doc>) -> Doc {
    Doc::L(std::iter::once(w(head)).chain(rest).collect())
}

fn rhs_doc(r: &Rhs) -> Doc {
    match r {
        Rhs::Atom(a) => atom_doc(a),
        Rhs::Bin(op, t, a, b) => list(
            format!("{}.{}", op.name(), t.name()),
            [atom_doc(a), atom_doc(b)],
        ),
        Rhs::Not(t, a) => list(format!("not.{}", t.name()), [atom_doc(a)]),
        Rhs::Len(n) => list("len", [w(n.clone())]),
        Rhs::Load(n, i) => list("load", [w(n.clone()), atom_doc(i)]),
        Rhs::Array(t, es) => list(
            "array",
            std::iter::once(w(t.name())).chain(es.iter().map(atom_doc)),
        ),
        Rhs::View {
            mutable,
            base,
            lo,
            hi,
        } => list(
            if *mutable { "mview" } else { "view" },
            [w(base.clone()), atom_doc(lo), atom_doc(hi)],
        ),
        Rhs::Call(f, args) => list(
            "call",
            std::iter::once(w(f.clone())).chain(args.iter().map(atom_doc)),
        ),
    }
}

fn stmts_doc(ss: &[Stmt]) -> Vec<Doc> {
    ss.iter().map(stmt_doc).collect()
}

fn stmt_doc(s: &Stmt) -> Doc {
    match s {
        Stmt::Let(n, t, r) => list("let", [w(n.clone()), ty_doc(t), rhs_doc(r)]),
        Stmt::Var(n, t, r) => list("var", [w(n.clone()), ty_doc(t), rhs_doc(r)]),
        Stmt::Set(n, a) => list("set", [w(n.clone()), atom_doc(a)]),
        Stmt::Store(n, i, v) => list("store", [w(n.clone()), atom_doc(i), atom_doc(v)]),
        Stmt::Call(f, args) => list(
            "call",
            std::iter::once(w(f.clone())).chain(args.iter().map(atom_doc)),
        ),
        Stmt::If(c, t, e) => Doc::B(
            vec![w("if"), atom_doc(c)],
            vec![block("then", stmts_doc(t)), block("else", stmts_doc(e))],
        ),
        Stmt::Loop(l) => {
            let mut children = vec![
                block("head", stmts_doc(&l.head)),
                list("cond", [atom_doc(&l.cond)]),
            ];
            children.extend(
                l.invariants
                    .iter()
                    .map(|i| list("invariant", [term_doc(i)])),
            );
            children.push(list("decreases", [term_doc(&l.decreases)]));
            children.push(block("body", stmts_doc(&l.body)));
            block("loop", children)
        }
        Stmt::Return(None) => list("return", []),
        Stmt::Return(Some(a)) => list("return", [atom_doc(a)]),
    }
}

fn term_doc(t: &Term) -> Doc {
    match t {
        Term::Name(n) => w(n.clone()),
        Term::Result => w("result"),
        Term::Lit(v, s) => w(format!("{v}{}", s.name())),
        Term::Int(v) => w(v.to_string()),
        Term::Bool(b) => w(b.to_string()),
        Term::Lift(x) => list("int", [term_doc(x)]),
        Term::Len(x) => list("len", [term_doc(x)]),
        Term::Old(x) => list("old", [term_doc(x)]),
        Term::Not(x) => list("not", [term_doc(x)]),
        Term::Sel(s, k) => list("sel", [term_doc(s), term_doc(k)]),
        Term::Op(op, a, b) => list(op.name(), [term_doc(a), term_doc(b)]),
        Term::Bit(op, s, a, b) => list(
            format!("{}.{}", op.name(), s.name()),
            [term_doc(a), term_doc(b)],
        ),
        Term::BitNot(s, x) => list(format!("bnot.{}", s.name()), [term_doc(x)]),
        Term::Quant(q) => list(
            if q.forall { "forall" } else { "exists" },
            [
                w(q.var.clone()),
                term_doc(&q.lo),
                term_doc(&q.hi),
                term_doc(&q.body),
            ],
        ),
    }
}
