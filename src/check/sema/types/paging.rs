//! How an endpoint answers more than once: a page at a time (`pages
//! .number(param: page)`, `.cursor(…)`, `.link`), or a line at a time
//! (`streamed`). What a read of either holds is the list it gathers.

use super::ty::Type;
use super::{Checker, Expr, answer_of, expr_text};
use crate::syntax::parser::ast::{ApiDecl, Endpoint};

/// The styles `pages` takes, each with what it names.
const STYLES: &[(&str, &[&str])] = &[
    ("number", &["param", "items"]),
    ("cursor", &["param", "next", "items"]),
    ("link", &["items"]),
];

impl Checker<'_, '_> {
    /// `pages` and `streamed` on one endpoint: a read of it, its
    /// parameter, and how a page's items and the next page are found
    /// (`N14`).
    pub(super) fn paging(&mut self, a: &ApiDecl, e: &Endpoint) {
        let what = format!("{}.{}", a.name, e.name);
        let reads = matches!(e.method.as_str(), "GET" | "HEAD");
        let answer = e
            .returns
            .as_ref()
            .map(|t| answer_of(self.world.resolve(Type::from_ref(t))));
        if e.streamed() {
            if !reads {
                self.error(
                    e.span,
                    "N14",
                    format!("`{what}` is a `{}` marked `streamed`: only a read is gathered a line at a time", e.method),
                    "Mark a `get` `streamed`",
                );
            }
            if let Some(ty) = &answer
                && !matches!(ty, Type::List(_) | Type::Error | Type::Any)
            {
                self.error(
                    e.span,
                    "N14",
                    format!("`{what}` is `streamed`, so it answers a list, one item a line, but it says `{ty}`"),
                    &format!("Say what each line is: `-> [{ty}]! streamed`"),
                );
            }
            if e.pages().is_some() {
                self.error(
                    e.span,
                    "N14",
                    format!("`{what}` is `streamed` and `pages`: a streamed answer is one request, however long"),
                    "Keep one of the two",
                );
            }
        }
        let Some(paging) = e.pages() else {
            return;
        };
        if !reads {
            self.error(
                e.span,
                "N14",
                format!(
                    "`{what}` is a `{}`: only a read is gathered a page at a time",
                    e.method
                ),
                "Say how a `get` pages",
            );
        }
        let Some((_, takes)) = STYLES.iter().find(|(s, _)| *s == paging.style) else {
            self.error(
                e.span,
                "N14",
                format!(
                    "`pages {}` is not a way to page",
                    expr_text(paging.written)
                ),
                "Write `pages .number(param: page)`, `pages .cursor(param: after, next: p => p.next)` or `pages .link`",
            );
            return;
        };
        for (name, _) in &paging.args {
            if !takes.contains(name) {
                self.error(
                    e.span,
                    "N14",
                    format!("`pages .{}` takes no `{name}:`", paging.style),
                    &format!(
                        "It takes {}",
                        takes
                            .iter()
                            .map(|t| format!("`{t}:`"))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                );
            }
        }
        // The parameter the next page is asked with, which the first call
        // need not give.
        if takes.contains(&"param") {
            match paging.param() {
                None => self.error(
                    e.span,
                    "N14",
                    format!(
                        "`pages .{}` on `{what}` names no parameter the page goes in",
                        paging.style
                    ),
                    &format!(
                        "Name one of its parameters: `pages .{}(param: …)`",
                        paging.style
                    ),
                ),
                Some(param) => match e.params.iter().find(|p| p.name == param) {
                    None => {
                        let params: Vec<&str> = e.params.iter().map(|p| p.name.as_str()).collect();
                        self.error_naming(
                            "N14",
                            format!("`pages .{}(param: {param})`, but `{what}` has no parameter `{param}`", paging.style),
                            &format!("Give it one: `{}({param}: {})`", e.name, if paging.style == "number" { "Number = 1" } else { "String?" }),
                            param,
                            &params,
                            false,
                        );
                    }
                    Some(p) => {
                        let ty = self.world.resolve(Type::from_ref(&p.prop_type));
                        if p.default.is_none() && !matches!(ty, Type::Optional(_)) {
                            self.error(
                                e.span,
                                "N14",
                                format!("`{param}` of `{what}` must be given, so the first page cannot be asked for without it"),
                                &format!("Give it a default, or let it be null: `{param}: {}`", if paging.style == "number" { "Number = 1".to_string() } else { format!("{ty}?") }),
                            );
                        }
                        if paging.style == "number"
                            && !matches!(ty.unwrapped(), Type::Number | Type::Error | Type::Any)
                        {
                            self.error(
                                e.span,
                                "N14",
                                format!(
                                    "`pages .number` counts pages in `{param}`, which is `{ty}`"
                                ),
                                &format!("Make it a number: `{param}: Number = 1`"),
                            );
                        }
                    }
                },
            }
        }
        let Some(answer) = answer else {
            return;
        };
        if paging.style == "cursor" {
            match paging.arg("next") {
                None => self.error(
                    e.span,
                    "N14",
                    format!(
                        "`pages .cursor` on `{what}` says nothing of where the next page starts"
                    ),
                    "Name it from a page: `next: p => p.nextCursor`",
                ),
                Some(next) => {
                    self.current_span = e.span;
                    let wanted = Type::Func(vec![answer], Box::new(Type::Any));
                    if let Type::Func(_, cursor) = self.infer(next, Some(&wanted))
                        && matches!(*cursor, Type::List(_) | Type::Bool)
                    {
                        self.error(
                            e.span,
                            "N14",
                            format!("`next:` of `{what}` gives `{cursor}`, which is not where a page starts"),
                            "Give the cursor the server sent, or null at the end: `next: p => p.nextCursor`",
                        );
                    }
                }
            }
        }
        self.paged_items(e, false);
    }

    /// What a paged read of `e` gathers: its `items:`, or the answer when
    /// that is a list. `None` for an endpoint that does not page. `quiet`
    /// keeps what it finds unsaid, for a read that asks again.
    pub(super) fn paged_items(&mut self, e: &Endpoint, quiet: bool) -> Option<Type> {
        let paging = e.pages()?;
        let answer = answer_of(self.world.resolve(Type::from_ref(e.returns.as_ref()?)));
        let items = match paging.arg("items") {
            Some(items) => {
                let wanted = Type::Func(vec![answer], Box::new(Type::Any));
                let got = if quiet {
                    self.infer_quiet_as(items, Some(&wanted))
                } else {
                    self.current_span = e.span;
                    self.infer(items, Some(&wanted))
                };
                match got {
                    Type::Func(_, list) => *list,
                    _ => Type::Error,
                }
            }
            None => answer,
        };
        match items {
            Type::List(_) | Type::Error | Type::Any => Some(items),
            other => {
                if !quiet {
                    let what = if paging.arg("items").is_some() {
                        format!("`items:` gives `{other}`")
                    } else {
                        format!("each page is `{other}`")
                    };
                    self.error(
                        e.span,
                        "N14",
                        format!("A page is a list of what it holds, but {what}"),
                        "Say where a page's items are: `items: p => p.posts`",
                    );
                }
                Some(Type::Error)
            }
        }
    }

    /// `for await line in Logs.tail(id: id)`: what each line of the
    /// streamed answer is, or why it is not one.
    pub(super) fn streamed_item(&mut self, iterable: &Expr) -> Type {
        let streamed = match iterable {
            Expr::MethodCall(obj, method, _) => self
                .world
                .api_of(obj)
                .and_then(|api| self.world.endpoint(&api, method))
                .filter(|e| e.streamed()),
            _ => None,
        };
        let ty = self.infer(iterable, None);
        if streamed.is_none() {
            if !matches!(ty, Type::Error) {
                self.error_at_current(
                    "N14",
                    format!("`for await` walks the lines of a `streamed` endpoint, and `{}` is not a call of one", expr_text(iterable)),
                    "Mark the endpoint `streamed` — `get tail(id: String) at \"logs/{id}\" -> [Line]! streamed` — or loop over a list with `for`",
                );
            }
            return Type::Error;
        }
        if !self.async_ok {
            self.error_at_current(
                "T17",
                "`for await` is written in an action or a handler, which can wait".to_string(),
                "Walk it in an action, or read it with a `resource`, which grows as the lines arrive",
            );
        }
        let answer = match ty {
            Type::Promise(inner) => answer_of(*inner),
            other => answer_of(other),
        };
        match answer {
            Type::List(item) => *item,
            _ => Type::Error,
        }
    }
}
