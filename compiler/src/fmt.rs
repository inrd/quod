//! Canonical formatter (`spec/format.md`).

use crate::ast::*;
use crate::lexer::Comment;

const INDENT: &str = "    ";

/// Format a parsed file canonically.
pub fn format_file(file: &File) -> String {
    let mut out = String::new();
    for (i, item) in file.items.iter().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        leading(&mut out, &item.comments.leading, 0);
        out.push_str(&item_text(item));
        line_end(&mut out, &item.comments.trailing);
    }
    if !file.trailing.is_empty() {
        if !file.items.is_empty() {
            out.push('\n');
        }
        closing_comments(&mut out, &file.trailing, 0);
    }
    out
}

fn indent(out: &mut String, level: usize) {
    for _ in 0..level {
        out.push_str(INDENT);
    }
}

fn comment_text(c: &Comment) -> String {
    format!("//{}", c.text)
}

fn leading(out: &mut String, comments: &[Comment], level: usize) {
    for c in comments {
        indent(out, level);
        out.push_str(&comment_text(c));
        out.push('\n');
        if c.blank_after {
            out.push('\n');
        }
    }
}

/// Comments before a closing brace or EOF: no blank line after the last one.
fn closing_comments(out: &mut String, comments: &[Comment], level: usize) {
    for (i, c) in comments.iter().enumerate() {
        indent(out, level);
        out.push_str(&comment_text(c));
        out.push('\n');
        if c.blank_after && i + 1 < comments.len() {
            out.push('\n');
        }
    }
}

fn line_end(out: &mut String, trailing: &Option<Comment>) {
    if let Some(c) = trailing {
        out.push(' ');
        out.push_str(&comment_text(c));
    }
    out.push('\n');
}

fn item_text(item: &Item) -> String {
    let mut s = String::new();
    if item.is_pub {
        s.push_str("pub ");
    }
    match &item.kind {
        ItemKind::Fn(f) => {
            s.push_str("fn ");
            s.push_str(&f.name.name);
            s.push_str(&generic_params(&f.generics));
            s.push('(');
            s.push_str(&join(
                f.params
                    .iter()
                    .map(|p| format!("{}: {}", p.name.name, ty(&p.ty))),
            ));
            s.push_str(") -> ");
            s.push_str(&ty(&f.ret));
            s.push('\n');
            s.push_str(INDENT);
            if f.effects.is_empty() {
                s.push_str("effects {}");
            } else {
                s.push_str("effects { ");
                s.push_str(&join(f.effects.iter().map(|e| e.name.clone())));
                s.push_str(" }");
            }
            s.push('\n');
            s.push_str(&block(&f.body, 0));
        }
        ItemKind::Type(t) => {
            s.push_str("type ");
            s.push_str(&t.name.name);
            s.push_str(&generic_params(&t.generics));
            s.push_str(" {\n");
            for f in &t.fields {
                leading(&mut s, &f.comments.leading, 1);
                indent(&mut s, 1);
                s.push_str(&format!("{}: {},", f.name.name, ty(&f.ty)));
                line_end(&mut s, &f.comments.trailing);
            }
            closing_comments(&mut s, &t.trailing, 1);
            s.push('}');
        }
        ItemKind::Mod(m) => {
            s.push_str("mod ");
            s.push_str(&m.name);
            s.push(';');
        }
        ItemKind::Use(p) => {
            s.push_str("use ");
            s.push_str(&path(p));
            s.push(';');
        }
    }
    s
}

fn join(parts: impl IntoIterator<Item = String>) -> String {
    parts.into_iter().collect::<Vec<_>>().join(", ")
}

fn path(p: &[Ident]) -> String {
    p.iter()
        .map(|i| i.name.as_str())
        .collect::<Vec<_>>()
        .join("::")
}

fn generic_params(g: &[GenericParam]) -> String {
    if g.is_empty() {
        return String::new();
    }
    let parts = g.iter().map(|p| match p {
        GenericParam::Type(i) => i.name.clone(),
        GenericParam::Const(i, t) => format!("const {}: {}", i.name, ty(t)),
    });
    format!("<{}>", join(parts))
}

fn generic_args(g: &[GenericArg]) -> String {
    let parts = g.iter().map(|a| match a {
        GenericArg::Type(t) => ty(t),
        GenericArg::Int(l) => l.text.clone(),
    });
    format!("<{}>", join(parts))
}

pub fn ty(t: &Type) -> String {
    match t {
        Type::Named { path: p, args, .. } => {
            if args.is_empty() {
                path(p)
            } else {
                format!("{}{}", path(p), generic_args(args))
            }
        }
        Type::Array { elem, len, .. } => {
            let n = match len {
                ConstExpr::Int(l) => l.text.clone(),
                ConstExpr::Name(i) => i.name.clone(),
            };
            format!("[{}; {}]", ty(elem), n)
        }
        Type::Slice { elem, .. } => format!("[{}]", ty(elem)),
        Type::Ref { mutable, inner, .. } => {
            format!("&{}{}", if *mutable { "mut " } else { "" }, ty(inner))
        }
        Type::Unit { .. } => "()".to_string(),
    }
}

/// A block whose `{` continues the current line and whose `}` is at `level`.
fn block(b: &Block, level: usize) -> String {
    if b.open.is_none() && b.stmts.is_empty() && b.trailing.is_empty() {
        return "{}".to_string();
    }
    let mut s = String::from("{");
    line_end(&mut s, &b.open);
    for st in &b.stmts {
        leading(&mut s, &st.comments.leading, level + 1);
        indent(&mut s, level + 1);
        s.push_str(&stmt(st, level + 1));
        line_end(&mut s, &st.comments.trailing);
    }
    closing_comments(&mut s, &b.trailing, level + 1);
    indent(&mut s, level);
    s.push('}');
    s
}

fn stmt(st: &Stmt, level: usize) -> String {
    match &st.kind {
        StmtKind::Let {
            mutable,
            name,
            ty: t,
            init,
        } => format!(
            "let {}{}: {} = {};",
            if *mutable { "mut " } else { "" },
            name.name,
            ty(t),
            expr(init)
        ),
        StmtKind::If(i) => if_stmt(i, level),
        StmtKind::For {
            var,
            ty: t,
            start,
            end,
            body,
        } => format!(
            "for {}: {} in {}..{} {}",
            var.name,
            ty(t),
            expr(start),
            expr(end),
            block(body, level)
        ),
        StmtKind::Return(e) => format!("return {};", expr(e)),
        StmtKind::Break => "break;".to_string(),
        StmtKind::Continue => "continue;".to_string(),
        StmtKind::Block(b) => block(b, level),
        StmtKind::Assign { place, value } => format!("{} = {};", expr(place), expr(value)),
        StmtKind::Compound { op, place, value } => {
            format!("{} {} {};", expr(place), op.as_str(), expr(value))
        }
        StmtKind::Expr(e) => format!("{};", expr(e)),
    }
}

fn if_stmt(i: &IfStmt, level: usize) -> String {
    let mut s = format!("if {} {}", expr(&i.cond), block(&i.then, level));
    match &i.els {
        None => {}
        Some(Else::If(e)) => {
            s.push_str(" else ");
            s.push_str(&if_stmt(e, level));
        }
        Some(Else::Block(b)) => {
            s.push_str(" else ");
            s.push_str(&block(b, level));
        }
    }
    s
}

fn expr_path(p: &ExprPath) -> String {
    match &p.generic_args {
        None => path(&p.segments),
        Some(args) => format!("{}::{}", path(&p.segments), generic_args(args)),
    }
}

pub fn expr(e: &Expr) -> String {
    match &e.kind {
        ExprKind::Int(l) => l.text.clone(),
        ExprKind::Bool(b) => b.to_string(),
        ExprKind::Unit => "()".to_string(),
        ExprKind::Hole(h) => format!("?{h}"),
        ExprKind::Path(p) => expr_path(p),
        ExprKind::StructLit { path: p, fields } => format!(
            "{}.{{ {} }}",
            expr_path(p),
            join(
                fields
                    .iter()
                    .map(|(n, v)| format!("{}: {}", n.name, expr(v)))
            )
        ),
        ExprKind::Paren(inner) => format!("({})", expr(inner)),
        ExprKind::Array(es) => format!("[{}]", join(es.iter().map(expr))),
        ExprKind::Field(b, f) => format!("{}.{}", expr(b), f.name),
        ExprKind::Index(b, i) => format!("{}[{}]", expr(b), expr(i)),
        ExprKind::Slice(b, lo, hi) => format!("{}[{}..{}]", expr(b), expr(lo), expr(hi)),
        ExprKind::Ref(mutable, b) => {
            let inner = expr(b);
            if *mutable {
                format!("&mut {inner}")
            } else if inner.starts_with('&') {
                // `&&` would lex as one token.
                format!("& {inner}")
            } else {
                format!("&{inner}")
            }
        }
        ExprKind::Call(f, args) => format!("{}({})", expr(f), join(args.iter().map(expr))),
        ExprKind::Cast(b, t) => format!("{} as {}", expr(b), t.name),
        ExprKind::Unary(op, b) => format!("{}{}", op.as_str(), expr(b)),
        ExprKind::Binary(op, l, r) => format!("{} {} {}", expr(l), op.as_str(), expr(r)),
    }
}
