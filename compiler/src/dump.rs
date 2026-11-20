//! Structural S-expression dump of the AST, without spans or comments.
//!
//! Used for AST equality (`parse(format(x)) == parse(x)`) and `quodc parse`.
//! Deliberately independent of the formatter's printer.

use crate::ast::*;
use crate::lexer::IntLit;

pub fn dump(file: &File) -> String {
    let mut out = String::new();
    for item in &file.items {
        out.push_str(&item_sx(item));
        out.push('\n');
    }
    out
}

fn list(head: &str, parts: impl IntoIterator<Item = String>) -> String {
    let mut s = format!("({head}");
    for p in parts {
        s.push(' ');
        s.push_str(&p);
    }
    s.push(')');
    s
}

fn int(l: &IntLit) -> String {
    match l.suffix {
        Some(sfx) => format!("{}{}", l.value, sfx.as_str()),
        None => l.value.to_string(),
    }
}

fn path(p: &[Ident]) -> String {
    p.iter()
        .map(|i| i.name.as_str())
        .collect::<Vec<_>>()
        .join("::")
}

fn item_sx(item: &Item) -> String {
    let vis = if item.is_pub { "pub" } else { "priv" };
    match &item.kind {
        ItemKind::Fn(f) => list(
            "fn",
            [
                vis.to_string(),
                f.name.name.clone(),
                list("generics", f.generics.iter().map(generic_param)),
                list(
                    "params",
                    f.params.iter().map(|p| list(&p.name.name, [ty(&p.ty)])),
                ),
                list("ret", [ty(&f.ret)]),
                list("effects", f.effects.iter().map(|e| e.name.clone())),
                list("requires", f.requires.iter().map(expr)),
                list("ensures", f.ensures.iter().map(expr)),
                block(&f.body),
            ],
        ),
        ItemKind::Type(t) => list(
            "type",
            [
                vis.to_string(),
                t.name.name.clone(),
                list("generics", t.generics.iter().map(generic_param)),
                list(
                    "fields",
                    t.fields.iter().map(|f| list(&f.name.name, [ty(&f.ty)])),
                ),
            ],
        ),
        ItemKind::Mod(m) => list("mod", [vis.to_string(), m.name.clone()]),
        ItemKind::Use(p) => list("use", [vis.to_string(), path(p)]),
    }
}

fn generic_param(g: &GenericParam) -> String {
    match g {
        GenericParam::Type(i) => list("type", [i.name.clone()]),
        GenericParam::Const(i, t) => list("const", [i.name.clone(), ty(t)]),
    }
}

fn generic_arg(g: &GenericArg) -> String {
    match g {
        GenericArg::Type(t) => ty(t),
        GenericArg::Int(l) => list("int", [int(l)]),
    }
}

fn ty(t: &Type) -> String {
    match t {
        Type::Named { path: p, args, .. } => {
            if args.is_empty() {
                path(p)
            } else {
                list(
                    "inst",
                    std::iter::once(path(p)).chain(args.iter().map(generic_arg)),
                )
            }
        }
        Type::Array { elem, len, .. } => {
            let n = match len {
                ConstExpr::Int(l) => int(l),
                ConstExpr::Name(i) => i.name.clone(),
            };
            list("array", [ty(elem), n])
        }
        Type::Slice { elem, .. } => list("slice", [ty(elem)]),
        Type::Ref { mutable, inner, .. } => {
            list(if *mutable { "ref-mut" } else { "ref" }, [ty(inner)])
        }
        Type::Unit { .. } => "unit".to_string(),
    }
}

fn block(b: &Block) -> String {
    list("block", b.stmts.iter().map(stmt))
}

fn stmt(s: &Stmt) -> String {
    match &s.kind {
        StmtKind::Let {
            mutable,
            name,
            ty: t,
            init,
        } => list(
            if *mutable { "let-mut" } else { "let" },
            [name.name.clone(), ty(t), expr(init)],
        ),
        StmtKind::If(i) => if_stmt(i),
        StmtKind::For {
            var,
            ty: t,
            start,
            end,
            body,
        } => list(
            "for",
            [var.name.clone(), ty(t), expr(start), expr(end), block(body)],
        ),
        StmtKind::While {
            cond,
            invariants,
            decreases,
            body,
        } => list(
            "while",
            [
                expr(cond),
                list("invariants", invariants.iter().map(expr)),
                list("decreases", [expr(decreases)]),
                block(body),
            ],
        ),
        StmtKind::Return(e) => list("return", [expr(e)]),
        StmtKind::Break => "(break)".to_string(),
        StmtKind::Continue => "(continue)".to_string(),
        StmtKind::Block(b) => block(b),
        StmtKind::Assign { place, value } => list("assign", [expr(place), expr(value)]),
        StmtKind::Compound { op, place, value } => list(op.as_str(), [expr(place), expr(value)]),
        StmtKind::Expr(e) => list("expr", [expr(e)]),
    }
}

fn if_stmt(i: &IfStmt) -> String {
    let mut parts = vec![expr(&i.cond), block(&i.then)];
    match &i.els {
        None => {}
        Some(Else::If(e)) => parts.push(if_stmt(e)),
        Some(Else::Block(b)) => parts.push(block(b)),
    }
    list("if", parts)
}

fn expr_path(p: &ExprPath) -> String {
    match &p.generic_args {
        None => path(&p.segments),
        Some(args) => list(
            "inst",
            std::iter::once(path(&p.segments)).chain(args.iter().map(generic_arg)),
        ),
    }
}

fn expr(e: &Expr) -> String {
    match &e.kind {
        ExprKind::Int(l) => int(l),
        ExprKind::Bool(b) => b.to_string(),
        ExprKind::Unit => "()".to_string(),
        ExprKind::Hole(h) => format!("?{h}"),
        ExprKind::Path(p) => expr_path(p),
        ExprKind::StructLit { path: p, fields } => list(
            "struct",
            std::iter::once(expr_path(p))
                .chain(fields.iter().map(|(n, v)| list(&n.name, [expr(v)]))),
        ),
        ExprKind::Paren(inner) => list("paren", [expr(inner)]),
        ExprKind::Array(es) => list("array", es.iter().map(expr)),
        ExprKind::Field(b, f) => list("field", [expr(b), f.name.clone()]),
        ExprKind::Index(b, i) => list("index", [expr(b), expr(i)]),
        ExprKind::Slice(b, lo, hi) => list("slice", [expr(b), expr(lo), expr(hi)]),
        ExprKind::Ref(m, b) => list(if *m { "ref-mut" } else { "ref" }, [expr(b)]),
        ExprKind::Call(f, args) => list(
            "call",
            std::iter::once(expr(f)).chain(args.iter().map(expr)),
        ),
        ExprKind::Cast(b, t) => list("as", [expr(b), t.name.clone()]),
        ExprKind::Unary(op, b) => list(op.as_str(), [expr(b)]),
        ExprKind::Binary(op, l, r) => list(op.as_str(), [expr(l), expr(r)]),
        ExprKind::Result => "(result)".to_string(),
        ExprKind::Old(inner) => list("old", [expr(inner)]),
        ExprKind::Quant(q) => list(
            q.keyword(),
            [
                q.var.name.clone(),
                ty(&q.ty),
                expr(&q.lo),
                expr(&q.hi),
                expr(&q.body),
            ],
        ),
    }
}
