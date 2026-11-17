//! A service's contract, as 7.0 writes it: groups of endpoints, each
//! endpoint's path written `at "users/{id}"` with every `{name}` one of its
//! parameters (`N01`), where each other parameter goes, and the reads a
//! write makes stale — `invalidates [list, byId(id)]`, every name and
//! argument checked (`N05`).

use super::{Checker, Expr};
use crate::diagnostics::fixes::Plan;
use crate::syntax::parser::ast::{ApiDecl, Endpoint, ParamLocation};

/// The `{name}`s a path names, in order.
pub(super) fn path_names(path: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = path;
    while let Some(open) = rest.find('{') {
        let Some(close) = rest[open..].find('}') else {
            break;
        };
        out.push(rest[open + 1..open + close].trim().to_string());
        rest = &rest[open + close + 1..];
    }
    out
}

impl Checker<'_, '_> {
    /// A service's `invalidations { }` and `pagination { }`: what a
    /// contract written elsewhere (a generated one) cannot say, said beside
    /// it. Each names an endpoint of one of the service's groups; a write's
    /// targets are reads of its own group, asked with the write's own
    /// parameters (`N05`). What each says is then checked at the endpoint,
    /// as if written there.
    pub(super) fn service_blocks(&mut self, a: &ApiDecl) {
        if a.name.contains('.') && (!a.invalidations.is_empty() || !a.pagination.is_empty()) {
            let service = a.name.split('.').next().unwrap_or_default();
            self.error(
                a.span,
                "N05",
                format!("`api {}` is a group: what its endpoints make stale and how they page is said on them, or in `api {service}`", a.name),
                "Write `invalidates [ … ]` and `pages …` on the endpoint, or move the block to the service",
            );
            return;
        }
        let groups: Vec<String> = self
            .world
            .apis
            .keys()
            .filter_map(|k| k.strip_prefix(&format!("{}.", a.name)).map(str::to_string))
            .collect();
        let known = |cx: &Self, full: &str| -> Option<&Endpoint> {
            let (group, name) = full.split_once('.')?;
            cx.world.endpoint(&format!("{}.{group}", a.name), name)
        };
        let all: Vec<String> = groups
            .iter()
            .flat_map(|g| {
                self.world.apis[format!("{}.{g}", a.name).as_str()]
                    .endpoints
                    .iter()
                    .map(move |e| format!("{g}.{}", e.name))
            })
            .collect();
        let all_refs: Vec<&str> = all.iter().map(String::as_str).collect();
        for inv in &a.invalidations {
            self.current_span = inv.span;
            let Some(write) = known(self, &inv.endpoint) else {
                self.error_naming(
                    "N05",
                    format!(
                        "`{}` names `{}`, which no group of it has",
                        a.name, inv.endpoint
                    ),
                    "Name an endpoint as `group.endpoint`",
                    &inv.endpoint,
                    &all_refs,
                    false,
                );
                continue;
            };
            if matches!(write.method.as_str(), "GET" | "HEAD") {
                self.error(
                    inv.span,
                    "N05",
                    format!(
                        "`{}` is a read: it changes nothing, so it makes nothing stale",
                        inv.endpoint
                    ),
                    "Name what goes stale on the write that changes it",
                );
                continue;
            }
            let own: Vec<&str> = write.params.iter().map(|p| p.name.as_str()).collect();
            for p in &inv.params {
                if !own.contains(&p.as_str()) {
                    self.error(
                        inv.span,
                        "N05",
                        format!("`{}` takes no parameter `{p}`", inv.endpoint),
                        &format!(
                            "It takes {}",
                            if own.is_empty() {
                                "none".to_string()
                            } else {
                                own.iter()
                                    .map(|o| format!("`{o}`"))
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            }
                        ),
                    );
                }
            }
            let group = inv.endpoint.split('.').next().unwrap_or_default();
            for target in &inv.targets {
                if target.name.split('.').next() != Some(group) {
                    self.error(
                        inv.span,
                        "N05",
                        format!(
                            "`{}` invalidates `{}`, a read of another group",
                            inv.endpoint, target.name
                        ),
                        "A write makes stale the reads of its own group",
                    );
                    continue;
                }
                if known(self, &target.name).is_none() {
                    self.error_naming(
                        "N05",
                        format!(
                            "`{}` invalidates `{}`, which no group of `{}` has",
                            inv.endpoint, target.name, a.name
                        ),
                        "Name a read as `group.endpoint`",
                        &target.name,
                        &all_refs,
                        false,
                    );
                    continue;
                }
                for arg in &target.args {
                    if !inv.params.contains(arg) && !own.contains(&arg.as_str()) {
                        self.error(
                            inv.span,
                            "N05",
                            format!("`{}` invalidates `{}({arg})`: an argument is one of the write's own parameters", inv.endpoint, target.name),
                            &format!("Name it after the write: `{}({arg}) invalidates [ … ]`", inv.endpoint),
                        );
                    }
                }
            }
        }
        for (endpoint, _, span) in &a.pagination {
            self.current_span = *span;
            if known(self, endpoint).is_none() {
                self.error_naming(
                    "N14",
                    format!("`{}` pages `{endpoint}`, which no group of it has", a.name),
                    "Name a read as `group.endpoint`",
                    endpoint,
                    &all_refs,
                    false,
                );
            }
        }
    }

    /// `api Backend.users { … }`: a group of a service's endpoints, which
    /// the service reaches, and which has no transport of its own.
    pub(super) fn group_of(&mut self, a: &ApiDecl) {
        let Some((service, group)) = a.name.split_once('.') else {
            return;
        };
        if !self.world.apis.contains_key(service) {
            self.error(
                a.span,
                "T13",
                format!(
                    "`api {}` is a group of `{service}`, and nothing declares `api {service}`",
                    a.name
                ),
                &format!(
                    "Declare the service it belongs to: `api {service}(base: \"/api\") {{ … }}`"
                ),
            );
        }
        if !a.settings.is_empty() || !a.headers.is_empty() || !a.hooks.is_empty() {
            self.error(
                a.span,
                "N08",
                format!("`api {}` sets how requests are sent, which is its service's to say", a.name),
                &format!("Move `base:`, `timeout:`, `retry:` and `middleware [ … ]` to `api {service}`; `{service}.{group}` is reached through it"),
            );
        }
    }

    /// One endpoint's contract: its path, where its parameters go, and
    /// what it makes stale.
    pub(super) fn contract(&mut self, a: &ApiDecl, e: &Endpoint) {
        let what = format!("{}.{}", a.name, e.name);
        // `at` is where it is, written: a path made from the name was a
        // guess.
        if !e.path_written {
            self.error(
                e.span,
                "N08",
                format!("`{what}` says nothing of where it is: 7.0 asks for `at`"),
                &format!(
                    "Write the path the server answers it at: `{} {}(…) at \"{}\" -> …`",
                    e.method.to_lowercase(),
                    e.name,
                    e.name
                ),
            );
            self.plan_last(
                format!("Write `at \"{}\"`", e.name),
                Plan::InsertBefore {
                    anchor: "->".to_string(),
                    text: format!("at \"{}\" ", e.name),
                },
            );
        }
        // 6.x's `:name`, which `{name}` replaced.
        for segment in e.path.split('/') {
            if let Some(name) = segment.strip_prefix(':') {
                self.error(
                    e.span,
                    "N08",
                    format!(
                        "`{what}` is at `{}`: a path names a parameter `{{{name}}}`, not `:{name}`",
                        e.path
                    ),
                    &format!("Write `{{{name}}}`, as OpenAPI does"),
                );
                self.plan_last(
                    format!("Write `{{{name}}}`"),
                    Plan::Rename {
                        from: format!(":{name}"),
                        to: format!("{{{name}}}"),
                    },
                );
            }
        }
        let params: Vec<&str> = e.params.iter().map(|p| p.name.as_str()).collect();
        let named = path_names(&e.path);
        for name in &named {
            if !params.contains(&name.as_str()) {
                let hint = if params.is_empty() {
                    format!("Give it the parameter: `{}({name}: String)`", e.name)
                } else {
                    format!(
                        "Name a parameter `{name}`; it takes {}",
                        params
                            .iter()
                            .map(|p| format!("`{p}`"))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                };
                self.error(
                    e.span,
                    "N01",
                    format!(
                        "`{what}` is at `{}`, and no parameter fills `{{{name}}}`",
                        e.path
                    ),
                    &hint,
                );
            }
        }
        // A parameter has one place: the path, a header, a cookie.
        for (name, location) in &e.locations {
            if named.iter().any(|n| n == name) {
                let place = match location {
                    ParamLocation::Header(h) => format!("header `{h}`"),
                    ParamLocation::Cookie(_) => "a cookie".to_string(),
                    ParamLocation::Query(q) => format!("the query as `{q}`"),
                };
                self.error(
                    e.span,
                    "N01",
                    format!("`{name}` of `{what}` is in the path and in {place}: it has one place"),
                    "Leave it in the path, or take `{…}` out of it",
                );
            }
        }
        self.invalidates(a, e, &what);
    }

    /// `invalidates [list, byId(id)]`: each a read of the same group, each
    /// argument a parameter of the write and one the read takes (`N05`).
    fn invalidates(&mut self, a: &ApiDecl, e: &Endpoint, what: &str) {
        let Some((_, list)) = e.settings.iter().find(|(k, _)| k == "invalidates") else {
            return;
        };
        if matches!(e.method.as_str(), "GET" | "HEAD") {
            self.error(
                e.span,
                "N05",
                format!("`{what}` is a read: it changes nothing, so it makes nothing stale"),
                "Name what goes stale on the write that changes it",
            );
        }
        let Expr::ListLiteral(items) = list else {
            return;
        };
        let reads: Vec<&Endpoint> = a
            .endpoints
            .iter()
            .filter(|x| matches!(x.method.as_str(), "GET" | "HEAD"))
            .collect();
        let read_names: Vec<&str> = reads.iter().map(|r| r.name.as_str()).collect();
        let own: Vec<&str> = e.params.iter().map(|p| p.name.as_str()).collect();
        for item in items {
            let (name, args) = match item {
                Expr::Identifier(n) => (n.as_str(), &[][..]),
                Expr::FunctionCall(n, args) => (n.as_str(), args.as_slice()),
                other => {
                    self.error(
                        e.span,
                        "N05",
                        format!(
                            "`{what}` invalidates `{}`, which names no read",
                            super::expr_text(other)
                        ),
                        "Name a read of the same group: `invalidates [list, byId(id)]`",
                    );
                    continue;
                }
            };
            let Some(read) = reads.iter().find(|r| r.name == name) else {
                self.error_naming(
                    "N05",
                    format!(
                        "`{what}` invalidates `{name}`, which `{}` has no read of",
                        a.name
                    ),
                    &format!(
                        "It may name {}",
                        if read_names.is_empty() {
                            "a `get` of the same group".to_string()
                        } else {
                            read_names
                                .iter()
                                .map(|n| format!("`{n}`"))
                                .collect::<Vec<_>>()
                                .join(", ")
                        }
                    ),
                    name,
                    &read_names,
                    false,
                );
                continue;
            };
            if args.len() > read.params.len() {
                self.error(
                    e.span,
                    "N05",
                    format!(
                        "`{what}` invalidates `{name}` with {} argument(s), but `{name}` takes {}",
                        args.len(),
                        read.params.len()
                    ),
                    "Name only what the read takes",
                );
            }
            for arg in args {
                match arg {
                    // `byId(id)`: the write's `id`, as the read's first
                    // argument.
                    Expr::Identifier(n) if own.contains(&n.as_str()) => {}
                    other => {
                        self.error(
                            e.span,
                            "N05",
                            format!("`{what}` invalidates `{name}({})`: an argument is one of `{}`'s own parameters", super::expr_text(other), e.name),
                            &format!(
                                "It takes {}",
                                if own.is_empty() {
                                    "none".to_string()
                                } else {
                                    own.iter().map(|p| format!("`{p}`")).collect::<Vec<_>>().join(", ")
                                }
                            ),
                        );
                    }
                }
            }
        }
    }
}
