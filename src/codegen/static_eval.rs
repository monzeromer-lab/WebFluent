//! Resolving what a page's data *is* at build time, so the static paint can
//! include it.
//!
//! The static renderer used to emit `<!--wf-for-->` wherever a list appeared and
//! leave the rows to JavaScript. For seeded data — a store's initial state, a
//! literal array in the page — the values are sitting in the AST, so the rows
//! could have been painted all along. Leaving them out meant a static site's most
//! substantial content was absent from the HTML: nothing for a crawler to index,
//! and nothing for the browser to paint until the bundle had downloaded, parsed
//! and run, which is exactly the work Largest Contentful Paint measures.
//!
//! What cannot be known at build time — a fetched collection, anything derived
//! from user input — stays a placeholder. The rule is that this module never
//! guesses: it returns `None` and the renderer falls back to the client.

use std::collections::HashMap;

use crate::parser::ast::{BinOp, Declaration, Expr, Program, Statement, StatementKind, UnaryOp};

/// A value the compiler could work out for itself.
#[derive(Debug, Clone, PartialEq)]
pub enum Static {
    Str(String),
    Num(f64),
    Bool(bool),
    List(Vec<Static>),
    Map(Vec<(String, Static)>),
    Null,
}

impl Static {
    /// The text this value renders as.
    pub fn to_text(&self) -> String {
        match self {
            Static::Str(s) => s.clone(),
            Static::Num(n) => {
                if *n == (*n as i64) as f64 {
                    format!("{}", *n as i64)
                } else {
                    format!("{}", n)
                }
            }
            Static::Bool(b) => format!("{}", b),
            Static::Null => String::new(),
            // A list or map has no sensible text form; the renderer should be
            // iterating it, not printing it.
            Static::List(_) | Static::Map(_) => String::new(),
        }
    }

    /// JavaScript truthiness, so a resolved `if` takes the same branch the
    /// runtime would.
    pub fn truthy(&self) -> bool {
        match self {
            Static::Bool(b) => *b,
            Static::Num(n) => *n != 0.0 && !n.is_nan(),
            Static::Str(s) => !s.is_empty(),
            Static::List(items) => !items.is_empty(),
            Static::Map(_) => true,
            Static::Null => false,
        }
    }
}

/// Names in scope at build time, innermost last.
#[derive(Debug, Clone, Default)]
pub struct Scope {
    frames: Vec<HashMap<String, Static>>,
}

impl Scope {
    /// The scope a page starts with: every store's seeded state, keyed
    /// `Store.field`, plus the page's own literal `state` declarations.
    ///
    /// Only initial values. A store field a user action has since changed is not
    /// knowable here, and the hydrating client will correct it.
    pub fn from_program(program: &Program, page_body: &[Statement]) -> Self {
        let mut frame = HashMap::new();

        for decl in &program.declarations {
            if let Declaration::Store(store) = decl {
                let mut fields = Vec::new();
                for stmt in &store.body {
                    if let StatementKind::State(s) = &stmt.kind {
                        if let Some(v) = eval(&s.value, &Scope::default()) {
                            fields.push((s.name.clone(), v));
                        }
                    }
                }
                frame.insert(store.name.clone(), Static::Map(fields));
            }
        }

        let mut scope = Scope {
            frames: vec![frame],
        };
        scope.push_state(page_body);
        scope
    }

    /// Add the literal `state` declarations of a statement list.
    pub fn push_state(&mut self, stmts: &[Statement]) {
        let mut frame = HashMap::new();
        for stmt in stmts {
            if let StatementKind::State(s) = &stmt.kind {
                if let Some(v) = eval(&s.value, self) {
                    frame.insert(s.name.clone(), v);
                }
            }
        }
        self.frames.push(frame);
    }

    /// Bind one name, for a loop body.
    pub fn with(&self, name: &str, value: Static) -> Self {
        let mut next = self.clone();
        let mut frame = HashMap::new();
        frame.insert(name.to_string(), value);
        next.frames.push(frame);
        next
    }

    fn get(&self, name: &str) -> Option<&Static> {
        self.frames.iter().rev().find_map(|f| f.get(name))
    }
}

/// Work out an expression's value, or `None` if it depends on something only the
/// running page knows.
pub fn eval(expr: &Expr, scope: &Scope) -> Option<Static> {
    match expr {
        Expr::StringLiteral(s) => Some(Static::Str(s.clone())),
        Expr::NumberLiteral(n) => Some(Static::Num(*n)),
        Expr::BoolLiteral(b) => Some(Static::Bool(*b)),
        Expr::Null => Some(Static::Null),

        Expr::Identifier(name) => scope.get(name).cloned(),

        Expr::ListLiteral(items) => items
            .iter()
            .map(|e| eval(e, scope))
            .collect::<Option<Vec<_>>>()
            .map(Static::List),

        Expr::MapLiteral(entries) => entries
            .iter()
            .map(|(k, v)| eval(v, scope).map(|v| (k.clone(), v)))
            .collect::<Option<Vec<_>>>()
            .map(Static::Map),

        Expr::InterpolatedString(parts) => {
            use crate::parser::ast::StringPart;
            let mut out = String::new();
            for part in parts {
                match part {
                    StringPart::Literal(s) => out.push_str(s),
                    StringPart::Expression(e) => out.push_str(&eval(e, scope)?.to_text()),
                }
            }
            Some(Static::Str(out))
        }

        Expr::PropertyAccess(base, prop) => {
            let base = eval(base, scope)?;
            match (&base, prop.as_str()) {
                (Static::List(items), "length") => Some(Static::Num(items.len() as f64)),
                (Static::Str(s), "length") => Some(Static::Num(s.chars().count() as f64)),
                (Static::Map(fields), _) => fields
                    .iter()
                    .find(|(k, _)| k == prop)
                    .map(|(_, v)| v.clone()),
                _ => None,
            }
        }

        Expr::IndexAccess(base, index) => {
            let base = eval(base, scope)?;
            let index = eval(index, scope)?;
            match (base, index) {
                (Static::List(items), Static::Num(n)) => items.get(n as usize).cloned(),
                (Static::Map(fields), Static::Str(k)) => fields
                    .iter()
                    .find(|(key, _)| *key == k)
                    .map(|(_, v)| v.clone()),
                _ => None,
            }
        }

        Expr::UnaryOp(op, inner) => {
            let v = eval(inner, scope)?;
            match op {
                UnaryOp::Not => Some(Static::Bool(!v.truthy())),
                UnaryOp::Neg => match v {
                    Static::Num(n) => Some(Static::Num(-n)),
                    _ => None,
                },
            }
        }

        Expr::BinaryOp(left, op, right) => {
            let l = eval(left, scope)?;
            let r = eval(right, scope)?;
            binary(&l, op, &r)
        }

        // A call could do anything; the compiler does not run user code.
        Expr::MethodCall(..) | Expr::FunctionCall(..) | Expr::Lambda(..) => None,
    }
}

fn binary(l: &Static, op: &BinOp, r: &Static) -> Option<Static> {
    use BinOp::*;
    match op {
        Add => match (l, r) {
            (Static::Num(a), Static::Num(b)) => Some(Static::Num(a + b)),
            // `+` on anything with a string is concatenation, as in JavaScript.
            (Static::Str(_), _) | (_, Static::Str(_)) => {
                Some(Static::Str(format!("{}{}", l.to_text(), r.to_text())))
            }
            _ => None,
        },
        Sub | Mul | Div | Mod => {
            let (Static::Num(a), Static::Num(b)) = (l, r) else {
                return None;
            };
            Some(Static::Num(match op {
                Sub => a - b,
                Mul => a * b,
                Div => {
                    if *b == 0.0 {
                        return None;
                    }
                    a / b
                }
                _ => {
                    if *b == 0.0 {
                        return None;
                    }
                    a % b
                }
            }))
        }
        Eq => Some(Static::Bool(l == r)),
        Neq => Some(Static::Bool(l != r)),
        Lt | Gt | Lte | Gte => {
            let (Static::Num(a), Static::Num(b)) = (l, r) else {
                return None;
            };
            Some(Static::Bool(match op {
                Lt => a < b,
                Gt => a > b,
                Lte => a <= b,
                _ => a >= b,
            }))
        }
        And => Some(Static::Bool(l.truthy() && r.truthy())),
        Or => Some(Static::Bool(l.truthy() || r.truthy())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    fn program(src: &str) -> Program {
        let tokens = Lexer::new(src, "<t>").tokenize().expect("lex");
        Parser::new(tokens, "<t>").parse().expect("parse")
    }

    fn page_scope(src: &str) -> (Program, Scope) {
        let p = program(src);
        let body = p
            .declarations
            .iter()
            .find_map(|d| {
                if let Declaration::Page(pg) = d {
                    Some(pg.body.clone())
                } else {
                    None
                }
            })
            .unwrap_or_default();
        let scope = Scope::from_program(&p, &body);
        (p, scope)
    }

    fn expr_of(src: &str) -> Expr {
        // Borrow a page's first Text argument as a way to parse an expression.
        let p = program(&format!("Page P (path: \"/\") {{ Text({src}) }}"));
        let Declaration::Page(page) = &p.declarations[0] else {
            unreachable!()
        };
        let StatementKind::UIElement(ui) = &page.body[0].kind else {
            unreachable!()
        };
        match &ui.args[0] {
            crate::parser::ast::Arg::Positional(e) => e.clone(),
            _ => unreachable!(),
        }
    }

    #[test]
    fn literals_resolve() {
        let s = Scope::default();
        assert_eq!(eval(&expr_of("\"hi\""), &s), Some(Static::Str("hi".into())));
        assert_eq!(eval(&expr_of("42"), &s), Some(Static::Num(42.0)));
        assert_eq!(eval(&expr_of("true"), &s), Some(Static::Bool(true)));
    }

    #[test]
    fn a_store_s_seeded_state_is_in_scope() {
        let (_, scope) =
            page_scope("Store S { state rows = [1, 2, 3] }\nPage P (path: \"/\") { Text(\"x\") }");
        let value = eval(&expr_of("S.rows"), &scope).expect("S.rows should resolve");
        assert_eq!(
            value,
            Static::List(vec![Static::Num(1.0), Static::Num(2.0), Static::Num(3.0)])
        );
    }

    #[test]
    fn length_resolves_so_a_count_can_be_painted() {
        let (_, scope) =
            page_scope("Store S { state rows = [1, 2] }\nPage P (path: \"/\") { Text(\"x\") }");
        assert_eq!(
            eval(&expr_of("S.rows.length"), &scope),
            Some(Static::Num(2.0))
        );
    }

    #[test]
    fn a_page_s_own_state_is_in_scope() {
        let (_, scope) = page_scope("Page P (path: \"/\") { state n = 7\n Text(\"x\") }");
        assert_eq!(eval(&expr_of("n"), &scope), Some(Static::Num(7.0)));
    }

    #[test]
    fn map_fields_resolve_through_property_access() {
        let (_, scope) = page_scope(
            "Store S { state user = { name: \"Monzer\", age: 30 } }\nPage P (path: \"/\") { Text(\"x\") }",
        );
        assert_eq!(
            eval(&expr_of("S.user.name"), &scope),
            Some(Static::Str("Monzer".into()))
        );
    }

    #[test]
    fn arithmetic_and_comparison_resolve() {
        let s = Scope::default();
        assert_eq!(eval(&expr_of("2 + 3"), &s), Some(Static::Num(5.0)));
        assert_eq!(eval(&expr_of("2 > 3"), &s), Some(Static::Bool(false)));
        assert_eq!(
            eval(&expr_of("\"a\" + \"b\""), &s),
            Some(Static::Str("ab".into()))
        );
    }

    /// The compiler does not run user code, and it does not pretend to know what
    /// a call returns.
    #[test]
    fn anything_unknowable_returns_none_rather_than_a_guess() {
        let s = Scope::default();
        assert_eq!(eval(&expr_of("unknownThing"), &s), None);
        assert_eq!(eval(&expr_of("items.filter(x => x)"), &s), None);
        assert_eq!(
            eval(&expr_of("1 / 0"), &s),
            None,
            "no infinity in the output"
        );
    }

    #[test]
    fn a_loop_binding_shadows_the_outer_scope() {
        let (_, scope) = page_scope("Page P (path: \"/\") { state x = 1\n Text(\"y\") }");
        let inner = scope.with("x", Static::Str("bound".into()));
        assert_eq!(
            eval(&expr_of("x"), &inner),
            Some(Static::Str("bound".into()))
        );
        assert_eq!(
            eval(&expr_of("x"), &scope),
            Some(Static::Num(1.0)),
            "outer is intact"
        );
    }

    #[test]
    fn truthiness_follows_javascript() {
        assert!(!Static::Str(String::new()).truthy());
        assert!(Static::Str("x".into()).truthy());
        assert!(!Static::Num(0.0).truthy());
        assert!(!Static::List(vec![]).truthy());
        assert!(Static::List(vec![Static::Null]).truthy());
        assert!(!Static::Null.truthy());
    }
}
