//! The handles a program is given: a form, a `ref:`, an image, and the browser's values.

use super::{Arg, ComponentRef, Expr, Statement, StatementKind, Type, global_type};

pub fn form_type() -> Type {
    Type::Shape(vec![
        ("valid".to_string(), Type::Bool),
        // Whether an async rule is still asking.
        ("pending".to_string(), Type::Bool),
        // The message of every field that has one, and the fields the
        // reader has left, each by the control's name.
        ("errors".to_string(), Type::Map),
        ("touched".to_string(), Type::Map),
        ("values".to_string(), Type::Map),
        ("element".to_string(), Type::Any),
        (
            "reset".to_string(),
            Type::Func(Vec::new(), Box::new(Type::Null)),
        ),
        (
            "submit".to_string(),
            Type::Func(Vec::new(), Box::new(Type::Null)),
        ),
        // What a 422 said, by field name.
        (
            "apply".to_string(),
            Type::Func(vec![Type::Map], Box::new(Type::Null)),
        ),
    ])
}

/// The names every `Form(bind: name)` in `stmts` declares, at any depth.
pub fn form_names(stmts: &[Statement]) -> Vec<String> {
    fn walk(stmts: &[Statement], out: &mut Vec<String>) {
        for stmt in stmts {
            if let StatementKind::UIElement(el) = &stmt.kind {
                if matches!(&el.component, ComponentRef::BuiltIn(n) if n == "Form") {
                    for arg in &el.args {
                        if let Arg::Named(k, Expr::Identifier(name)) = arg
                            && k == "bind"
                            && !out.contains(name)
                        {
                            out.push(name.clone());
                        }
                    }
                }
                walk(&el.children, out);
                for fill in &el.slot_fills {
                    walk(&fill.body, out);
                }
            }
            for body in stmt.kind.bodies() {
                walk(body, out);
            }
        }
    }
    let mut out = Vec::new();
    walk(stmts, &mut out);
    out
}

/// The names every `ref: name` in `stmts` declares, at any depth.
pub fn ref_names(stmts: &[Statement]) -> Vec<String> {
    fn walk(stmts: &[Statement], out: &mut Vec<String>) {
        for stmt in stmts {
            if let StatementKind::UIElement(el) = &stmt.kind {
                for arg in &el.args {
                    if let Arg::Named(k, Expr::Identifier(name)) = arg
                        && k == "ref"
                        && !out.contains(name)
                    {
                        out.push(name.clone());
                    }
                }
                walk(&el.children, out);
                for fill in &el.slot_fills {
                    walk(&fill.body, out);
                }
            }
            for body in stmt.kind.bodies() {
                walk(body, out);
            }
        }
    }
    let mut out = Vec::new();
    walk(stmts, &mut out);
    out
}

/// What an `image` name is: the asset the build made of the picture —
/// `media::Asset::as_json`, which a test holds to this.
pub fn image_type() -> Type {
    Type::Shape(vec![
        ("src".to_string(), Type::String),
        ("width".to_string(), Type::Number),
        ("height".to_string(), Type::Number),
        ("color".to_string(), Type::String),
        ("placeholder".to_string(), Type::String),
        ("srcset".to_string(), Type::String),
        ("sources".to_string(), Type::list(Type::Map)),
    ])
}

/// The type of a name every program can read — `viewport`, `network`,
/// `now` — for a tool to describe and complete.
pub fn browser_value_type(name: &str) -> Option<Type> {
    crate::registry::browser::BROWSER_VALUES
        .contains(&name)
        .then(|| global_type(name))
}
