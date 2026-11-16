//! What a request passes through: a service's `middleware [ … ]` and the
//! `middleware name(req, next) { … }` a project declares. A middleware is
//! handed the request and the rest of the chain, and returns what the
//! chain returns — a result of the answer, as JSON, or an `HttpError`.
//! It replaced a service's `headers { }` and its `on request`, `on
//! response` and `on error` hooks, which are refused here.

use super::{Checker, Expr, Type, expr_text, ty};
use crate::syntax::parser::ast::{ApiDecl, MiddlewareDecl};

/// What a middleware is handed: the request on its way out.
pub(super) fn request_type() -> Type {
    Type::Record("Request".to_string())
}

/// What the chain returns: the answer, still JSON, or what went wrong.
pub(super) fn answer_type() -> Type {
    Type::Result(
        Box::new(Type::Enum(ty::JSON.to_string())),
        Box::new(Type::Enum(ty::HTTP_ERROR.to_string())),
    )
}

/// `next`: the rest of the chain.
pub(super) fn next_type() -> Type {
    Type::Func(
        vec![request_type()],
        Box::new(Type::Promise(Box::new(answer_type()))),
    )
}

/// A middleware: `(Request, next) -> answer`.
pub(super) fn middleware_type() -> Type {
    Type::Func(
        vec![request_type(), next_type()],
        Box::new(Type::Promise(Box::new(answer_type()))),
    )
}

/// The middleware every service may name without declaring it, and what
/// each takes: `bearer(token)`, `header(name, value)`,
/// `refreshOn401(action)`, `requestId`, `log`.
const BUILT_IN: &[&str] = &["bearer", "header", "refreshOn401", "requestId", "log"];

impl Checker<'_, '_> {
    /// `middleware name(req, next) { … }`: a handler whose two names are
    /// the request and the rest of the chain, and whose every `return`
    /// hands back an answer.
    pub(super) fn middleware(&mut self, m: &MiddlewareDecl) {
        self.push_scope();
        self.current_span = m.handler.span;
        self.handler_names(
            &m.handler,
            &[request_type(), next_type()],
            "a middleware is handed the request and the rest of the chain",
        );
        self.returns.push(Vec::new());
        self.declared_returns.push(None);
        self.async_ok = true;
        self.in_handler += 1;
        self.statements(&m.handler.body, super::Body::Imperative);
        self.in_handler -= 1;
        self.async_ok = false;
        let returned = self.returns.pop().unwrap_or_default();
        self.declared_returns.pop();
        self.pop_scope();
        let answer = answer_type();
        let resolved = self.world.resolve(answer.clone());
        if returned.is_empty() {
            self.error(
                m.span,
                "T10",
                format!(
                    "`middleware {}` returns nothing, so its request never answers",
                    m.name
                ),
                "Return what the rest of the chain returns: `return await next(req)`",
            );
        }
        for given in returned {
            if !given.assignable_to(&resolved) && !given.assignable_to(&answer) {
                self.error(
                    m.span,
                    "T01",
                    format!("`middleware {}` returns `{given}`, but a middleware returns an answer: what `next` returns", m.name),
                    "Return `await next(req)`, or what it returned",
                );
            }
        }
    }

    /// A service's `middleware [ … ]`: each a declared middleware or a
    /// built-in one, given what it takes.
    pub(super) fn middleware_list(&mut self, list: &Expr) {
        let Expr::ListLiteral(items) = list else {
            self.infer(list, None);
            self.error_at_current(
                "T01",
                "`middleware` is a list: `middleware [bearer(Session.token), requestId]`"
                    .to_string(),
                "Name each step in the order a request passes through it",
            );
            return;
        };
        for item in items {
            match item {
                Expr::Identifier(n) if matches!(n.as_str(), "requestId" | "log") => {}
                Expr::Identifier(n) if self.world.consts.get(n) == Some(&middleware_type()) => {}
                Expr::FunctionCall(name, args) if name == "bearer" => {
                    self.middleware_args(name, args, &[Type::String]);
                }
                Expr::FunctionCall(name, args) if name == "header" => {
                    self.middleware_args(name, args, &[Type::String, Type::String]);
                }
                Expr::FunctionCall(name, args) if name == "refreshOn401" => {
                    if args.len() == 1 {
                        let given = self.infer(&args[0], None);
                        if !given.is_any() && !matches!(given, Type::Func(..) | Type::Error) {
                            self.error_at_current(
                                "T01",
                                format!("`refreshOn401` is handed what refreshes the credentials, an action, but `{}` is `{given}`", expr_text(&args[0])),
                                "Hand it the action: `refreshOn401(Session.refresh)`",
                            );
                        }
                    } else {
                        self.middleware_args(name, args, &[Type::Error]);
                    }
                }
                other => {
                    let wrong = match other {
                        Expr::Identifier(n) | Expr::FunctionCall(n, _) => n.clone(),
                        e => expr_text(e),
                    };
                    let mut known: Vec<String> =
                        BUILT_IN.iter().map(|s| (*s).to_string()).collect();
                    known.extend(
                        self.world
                            .consts
                            .iter()
                            .filter(|(_, t)| **t == middleware_type())
                            .map(|(n, _)| n.clone()),
                    );
                    let known: Vec<&str> = known.iter().map(String::as_str).collect();
                    self.error_naming(
                        "T13",
                        format!("`{wrong}` is not a middleware"),
                        "A step is a `middleware` the project declares, or `bearer(token)`, `header(name, value)`, `refreshOn401(action)`, `requestId`, `log`",
                        &wrong,
                        &known,
                        false,
                    );
                }
            }
        }
    }

    fn middleware_args(&mut self, name: &str, args: &[Expr], wanted: &[Type]) {
        if args.len() != wanted.len() {
            self.error_at_current(
                "T10",
                format!(
                    "`{name}` takes {} argument{}, but {} {} given",
                    wanted.len(),
                    if wanted.len() == 1 { "" } else { "s" },
                    args.len(),
                    if args.len() == 1 { "is" } else { "are" }
                ),
                "`bearer(token)`, `header(name, value)`, `refreshOn401(action)`",
            );
        }
        for (arg, want) in args.iter().zip(wanted) {
            let given = self.infer(arg, Some(want));
            if !given.assignable_to(want) {
                self.error_at_current(
                    "T01",
                    format!(
                        "`{name}` takes a `{want}`, but `{}` is `{given}`",
                        expr_text(arg)
                    ),
                    "",
                );
            }
        }
    }

    /// `N08` for 6.x's ways of touching a request: `headers { }` and the
    /// `on request`, `on response` and `on error` hooks, which middleware
    /// replaced.
    pub(super) fn old_transport(&mut self, a: &ApiDecl) {
        if !a.headers.is_empty() {
            let steps: Vec<String> = a
                .headers
                .iter()
                .map(|(k, v)| format!("header(\"{k}\", {})", expr_text(v)))
                .collect();
            self.error(
                a.span,
                "N08",
                format!(
                    "`{}`'s `headers {{ }}` is 6.x's: a request's headers are set by middleware",
                    a.name
                ),
                &format!("Write `middleware [{}]` in its place", steps.join(", ")),
            );
        }
        for hook in &a.hooks {
            let now = match hook.event.as_str() {
                "error" => {
                    "a `middleware` that looks at what `next` returns — `refreshOn401(Session.refresh)` for a token that expired"
                }
                "response" => "a `middleware` that looks at what `next` returns",
                _ => "a `middleware` that hands `next` the request it changed",
            };
            self.error(
                hook.span,
                "N08",
                format!(
                    "`{}`'s `on {}` hook is 6.x's: what touches a request is middleware",
                    a.name, hook.event
                ),
                &format!("Write {now}, and name it in `middleware [ … ]`"),
            );
        }
    }
}
