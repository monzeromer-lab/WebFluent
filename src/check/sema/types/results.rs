//! Results and error enums: what a call that may fail gives, and the enums
//! that say how it failed.
//!
//! An *error enum* names the statuses an endpoint answers with as cases of
//! their own, each with its typed body, and keeps one case, `http`, for
//! everything else:
//!
//! ```wf
//! enum CreateUserError {
//!     unprocessableContent(problem: ValidationProblem) on 422
//!     http(error: HttpError)
//! }
//! ```

use super::{Checker, Type, World};
use crate::syntax::parser::ast::{EnumCase, EnumDecl, Expr, Pattern, Span, TypeRef};

/// The case every error enum keeps for what it does not name.
pub(super) const HTTP_CASE: &str = "http";

impl World<'_> {
    /// Whether `name` is an error enum: one that names a status, or keeps
    /// an `http` case for what goes wrong between the page and the server.
    pub(super) fn is_error_enum(&self, name: &str) -> bool {
        self.enums.get(name).is_some_and(|e| claims_error_enum(e))
    }
}

/// Whether an enum says it is an error enum.
fn claims_error_enum(e: &EnumDecl) -> bool {
    e.cases
        .iter()
        .any(|c| c.status.is_some() || is_http_case(c))
}

/// `http(error: HttpError)`: the case an error enum keeps for the rest.
fn is_http_case(c: &EnumCase) -> bool {
    c.name == HTTP_CASE
        && matches!(c.fields.as_slice(), [f] if matches!(&f.ty, TypeRef::Named(n) if n == super::ty::HTTP_ERROR))
}

impl Checker<'_, '_> {
    /// `N12`: an error enum keeps exactly one `http(error: HttpError)`
    /// case; every other case carries one value and names its status; and
    /// no status is named twice.
    pub(super) fn error_enum_shape(&mut self, e: &EnumDecl) {
        if !claims_error_enum(e) {
            return;
        }
        let http_cases = e.cases.iter().filter(|c| is_http_case(c)).count();
        if http_cases != 1 {
            self.error(
                e.span,
                "N12",
                format!("`{}` names a status, so it is an error enum, and keeps no `http` case", e.name),
                "Add `http(error: HttpError)`: what goes wrong that no case names — no network, a timeout, another status — arrives there",
            );
        }
        let mut seen: Vec<(u16, &str)> = Vec::new();
        for case in &e.cases {
            if is_http_case(case) {
                if let Some(status) = case.status {
                    self.error(
                        e.span,
                        "N12",
                        format!("`{}.http` stands for every status no other case names, so it names none: `on {status}`", e.name),
                        "Take the `on …` off the `http` case",
                    );
                }
                continue;
            }
            match (case.status, case.fields.len()) {
                (None, _) => self.error(
                    e.span,
                    "N12",
                    format!("`{}.{}` names no status", e.name, case.name),
                    &format!("Say which status it stands for: `{}(…) on 404`", case.name),
                ),
                (Some(_), n) if n != 1 => self.error(
                    e.span,
                    "N12",
                    format!(
                        "`{}.{}` carries {n} values; a case of an error enum carries one, the body of its status",
                        e.name, case.name
                    ),
                    &format!("Give it one: `{}(problem: Problem)`", case.name),
                ),
                (Some(status), _) => {
                    if let Some((_, first)) = seen.iter().find(|(s, _)| *s == status) {
                        self.error(
                            e.span,
                            "N12",
                            format!("`{}.{}` and `{}.{first}` both stand for {status}", e.name, case.name, e.name),
                            "A status is one case: keep one of them",
                        );
                    } else {
                        seen.push((status, &case.name));
                    }
                }
            }
        }
    }

    /// What an endpoint fails with, held to what a request's failure may
    /// be: an error enum, or `HttpError` itself. `what` names the endpoint.
    pub(super) fn result_error(&mut self, ty: &Type, span: Span, what: &str) {
        let Type::Result(_, error) = ty else {
            return;
        };
        match &**error {
            Type::Enum(name) if name == super::ty::HTTP_ERROR || self.world.is_error_enum(name) => {}
            Type::Any | Type::Error => {}
            other => self.error(
                span,
                "N12",
                format!("{what} fails with `{other}`, which is no error enum"),
                "An error enum names its statuses and keeps `http(error: HttpError)`; or write `T!` for a call that fails only with an `HttpError`",
            ),
        }
    }
}

impl Checker<'_, '_> {
    /// `try e`: the value of the result `e`, or its error returned from the
    /// action — which must declare an error that one can become (`N03`).
    /// In a handler, the error is reported where the page reports errors.
    pub(super) fn try_expr(
        &mut self,
        e: &crate::syntax::parser::ast::Expr,
        expected: Option<&Type>,
    ) -> Type {
        // What the value goes to is the value of the result.
        let wanted = expected.map(|t| Type::Result(Box::new(t.clone()), Box::new(Type::Any)));
        let ty = self.infer(e, wanted.as_ref());
        let text = super::expr_text(e);
        let (value, error) = match ty {
            Type::Result(value, error) => (*value, *error),
            Type::Promise(inner) if matches!(*inner, Type::Result(..)) => {
                self.error_at_current(
                    "T17",
                    format!("`{text}` is a promise of a result until it is awaited"),
                    &format!("Write `try await {text}`"),
                );
                return Type::Error;
            }
            Type::Any | Type::Error => return ty,
            other => {
                self.error_at_current(
                    "N03",
                    format!("`try` takes a result, but `{text}` is `{other}`"),
                    "`try` is for what may fail — a call declared `-> T ! E` or `-> T!`, a `decode`",
                );
                return Type::Error;
            }
        };
        match self.declared_returns.last().cloned() {
            // In an action: the error goes up as the action's own.
            Some(Some(Type::Result(_, into))) => self.try_converts(&error, &into, &text),
            Some(declared) => {
                let says = declared.map_or_else(
                    || "declares nothing it returns".to_string(),
                    |d| format!("declares `{d}`, which cannot fail"),
                );
                self.error_at_current(
                    "N03",
                    format!(
                        "`try` returns `{text}`'s error from the action, and the action {says}"
                    ),
                    &format!(
                        "Say what it fails with: `-> {}`, or `match` the result here",
                        Type::Result(Box::new(value.clone()), Box::new(error.clone()))
                    ),
                );
            }
            // In a handler: the error is reported, not returned.
            None if self.in_handler > 0 => {}
            None => self.error_at_current(
                "N03",
                format!("`try` returns `{text}`'s error from an action, and this is none"),
                "Call it in an action that declares what it fails with, or `match` the result",
            ),
        }
        value
    }

    /// `N03`: an error of `from` passed up as an error of `into`. The same
    /// error goes as it is; an `HttpError` into an error enum's `http`
    /// case; another error enum only through its `http` case, so any other
    /// case of it must be handled first.
    fn try_converts(&mut self, from: &Type, into: &Type, text: &str) {
        let (Type::Enum(from_name), Type::Enum(into_name)) = (from, into) else {
            if !from.assignable_to(into) {
                self.error_at_current(
                    "N03",
                    format!("`{text}` fails with `{from}`, and the action fails with `{into}`"),
                    "Handle it here with `match`, or declare the action's error to fit it",
                );
            }
            return;
        };
        if from_name == into_name {
            return;
        }
        let http = super::ty::HTTP_ERROR;
        if from_name == http && self.world.is_error_enum(into_name) {
            return;
        }
        if !self.world.is_error_enum(from_name)
            || (into_name != http && !self.world.is_error_enum(into_name))
        {
            self.error_at_current(
                "N03",
                format!(
                    "`{text}` fails with `{from_name}`, and the action fails with `{into_name}`"
                ),
                "Handle it here with `match`, or declare the action's error to fit it",
            );
            return;
        }
        let into_cases: Vec<String> = if into_name == http {
            Vec::new()
        } else {
            self.world
                .enums
                .get(into_name.as_str())
                .map(|e| e.case_names())
                .unwrap_or_default()
        };
        let unhandled: Vec<String> = self
            .world
            .enums
            .get(from_name.as_str())
            .map(|e| e.case_names())
            .unwrap_or_default()
            .into_iter()
            .filter(|c| c != HTTP_CASE && !into_cases.contains(c))
            .collect();
        if !unhandled.is_empty() {
            self.error_at_current(
                "N03",
                format!(
                    "`{text}` may fail with {}, which `{into_name}` has no case for",
                    unhandled
                        .iter()
                        .map(|c| format!("`.{c}`"))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                "Handle those here with `match`; only its `http` case passes up as it is",
            );
        }
    }
}

impl Checker<'_, '_> {
    /// The cases of a value of `ty`, each with the types of its payload:
    /// an enum's, or a result's `.ok` and `.err`.
    pub(super) fn cases_of(&self, ty: &Type) -> Option<Vec<(String, Vec<Type>)>> {
        match ty.unwrapped() {
            Type::Result(value, error) => Some(vec![
                (
                    "ok".to_string(),
                    super::stmt::result_payload("ok", &value, &error),
                ),
                ("err".to_string(), vec![*error]),
            ]),
            Type::Resource(value, failed) => Some(super::read::read_cases(&value, &failed)),
            Type::Enum(name) => {
                let decl = self.world.enums.get(name.as_str())?;
                Some(
                    decl.case_names()
                        .into_iter()
                        .map(|c| {
                            let payload = self
                                .payload_of(&name, &c)
                                .unwrap_or_default()
                                .into_iter()
                                .map(|(_, t)| t)
                                .collect();
                            (c, payload)
                        })
                        .collect(),
                )
            }
            _ => None,
        }
    }

    /// Bind the names `p` binds, each with the type of its place in a value
    /// of `ty`, and report a pattern no value of `ty` can match.
    pub(super) fn bind_pattern(&mut self, p: &Pattern, ty: &Type, span: Span) {
        match p {
            Pattern::Bind(name) => self.bind_fixed(name, ty.clone(), span, super::ARM),
            Pattern::Wildcard => {}
            Pattern::Literal(e) => {
                let given = self.infer(e, Some(ty));
                if super::disjoint(&given, &ty.unwrapped()) {
                    self.error(
                        span,
                        "T14",
                        format!(
                            "`{}` is `{given}`, so it never matches a `{ty}`",
                            super::expr_text(e)
                        ),
                        "Match a value of the place's type",
                    );
                }
            }
            Pattern::Case(case, inner) => {
                if ty.is_any() {
                    for p in inner {
                        self.bind_pattern(p, ty, span);
                    }
                    return;
                }
                let Some(cases) = self.cases_of(ty) else {
                    self.error(
                        span,
                        "T11",
                        format!("`.{case}` matches a case, but this place is `{ty}`"),
                        "A case pattern matches an enum's value or a result",
                    );
                    for p in inner {
                        self.bind_pattern(p, &Type::Error, span);
                    }
                    return;
                };
                let Some((_, payload)) = cases.iter().find(|(c, _)| c == case) else {
                    self.error(
                        span,
                        "T02",
                        format!("`{ty}` has no case `.{case}`"),
                        &format!(
                            "`{ty}` is one of {}",
                            cases
                                .iter()
                                .map(|(c, _)| format!("`.{c}`"))
                                .collect::<Vec<_>>()
                                .join(", ")
                        ),
                    );
                    for p in inner {
                        self.bind_pattern(p, &Type::Error, span);
                    }
                    return;
                };
                if inner.len() > payload.len() {
                    self.error(
                        span,
                        "T10",
                        format!(
                            "`.{case}` carries {} value{}, but the pattern has {}",
                            payload.len(),
                            if payload.len() == 1 { "" } else { "s" },
                            inner.len()
                        ),
                        "Match one place for each value it carries",
                    );
                }
                let payload = payload.clone();
                for (i, p) in inner.iter().enumerate() {
                    let place = payload.get(i).cloned().unwrap_or(Type::Error);
                    self.bind_pattern(p, &place, span);
                }
            }
        }
    }

    /// Whether arms whose patterns are `rows` — each a pattern per place of
    /// a payload of `types` — take every value: a row that takes anything,
    /// or one place whose patterns between them take every value while the
    /// other places take anything.
    pub(super) fn covers(&self, rows: &[Vec<&Pattern>], types: &[Type]) -> bool {
        let any = |p: &&Pattern| p.is_irrefutable();
        if rows.iter().any(|r| r.iter().all(any)) {
            return true;
        }
        (0..types.len()).any(|j| {
            let column: Vec<&Pattern> = rows
                .iter()
                .filter(|r| {
                    r.iter()
                        .enumerate()
                        .all(|(i, p)| i == j || p.is_irrefutable())
                })
                .filter_map(|r| r.get(j).copied())
                .collect();
            !column.is_empty() && self.covers_place(&column, &types[j])
        })
    }

    /// Whether `patterns` between them take every value of `ty`.
    fn covers_place(&self, patterns: &[&Pattern], ty: &Type) -> bool {
        if patterns.iter().any(|p| p.is_irrefutable()) {
            return true;
        }
        if ty.unwrapped() == Type::Bool {
            let has = |b: bool| {
                patterns
                    .iter()
                    .any(|p| matches!(p, Pattern::Literal(Expr::BoolLiteral(x)) if *x == b))
            };
            return has(true) && has(false);
        }
        let Some(cases) = self.cases_of(ty) else {
            return false;
        };
        cases.iter().all(|(case, payload)| {
            let rows: Vec<Vec<&Pattern>> = patterns
                .iter()
                .filter_map(|p| match p {
                    Pattern::Case(c, inner) if c == case => Some(inner.iter().collect()),
                    _ => None,
                })
                .collect();
            !rows.is_empty() && self.covers(&rows, payload)
        })
    }
}

impl Checker<'_, '_> {
    /// `http.post(url, body: draft)`: a request of the built-in service, a
    /// promise of `T!`. `T` is what the answer goes to — a `let`'s type, a
    /// declared `return`, a read's annotation — and a `get` that goes to
    /// nothing that says is `N11`.
    pub(super) fn http_call(&mut self, verb: &str, args: &[Expr], expected: Option<&Type>) -> Type {
        let mut url = None;
        for arg in args {
            match arg {
                Expr::MapLiteral(pairs) => {
                    for (key, value) in pairs {
                        let key = key.trim_matches('"');
                        match key {
                            // What is sent is JSON, or plain data that is.
                            "body" => {
                                let json = Type::Enum(super::ty::JSON.to_string());
                                let given = self.infer(value, Some(&json));
                                self.expect(
                                    &given,
                                    &json,
                                    self.current_span,
                                    &format!("the body of `http.{verb}`"),
                                );
                            }
                            "headers" | "timeout" | "retry" | "cache" | "as" | "credentials"
                            | "signal" => {
                                self.infer(value, None);
                            }
                            other => {
                                self.infer(value, None);
                                self.error_at_current(
                                    "T10",
                                    format!("`http.{verb}` takes no `{other}`"),
                                    "It takes the address, then `body:`, `headers:`, `timeout:`, `retry:` and `cache:`",
                                );
                            }
                        }
                    }
                }
                other if url.is_none() => {
                    let given = self.infer(other, Some(&Type::String));
                    self.expect(
                        &given,
                        &Type::String,
                        self.current_span,
                        &format!("the address of `http.{verb}`"),
                    );
                    url = Some(());
                }
                other => {
                    self.infer(other, None);
                    self.error_at_current(
                        "T10",
                        format!("`http.{verb}` takes one address, then what it sends by name"),
                        &format!("Write `http.{verb}(url, body: value)`"),
                    );
                }
            }
        }
        if url.is_none() {
            self.error_at_current("C01", format!("`http.{verb}` needs the address to ask"), "");
        }
        // `as: .text` reads the answer as text.
        let as_text = args.iter().any(|a| {
            matches!(a, Expr::MapLiteral(p) if p.iter().any(|(k, v)| k == "as" && matches!(v, Expr::EnumCase(c) if c == "text")))
        });
        let answer = match expected.map(Type::unwrapped) {
            None if as_text => Some(Type::String),
            Some(Type::Promise(inner)) => match *inner {
                Type::Result(value, _) => Some(*value),
                other => Some(other),
            },
            Some(Type::Result(value, _)) => Some(*value),
            other => other,
        };
        let answer = match answer {
            Some(t) if !t.is_any() => {
                self.boundary_type(&t, &format!("what `http.{verb}` answers"));
                t
            }
            Some(t) if t == Type::Error => t,
            // A write whose answer nothing reads needs no type for it.
            _ if verb != "get" => Type::Null,
            _ => {
                self.error_at_current(
                    "N11",
                    format!("`http.{verb}` says nothing of what comes back"),
                    "Name its type where the answer goes: `let rows: [Row] = try await http.get(…)`, or `resource rows: [Row] = http.get(…)`",
                );
                Type::Error
            }
        };
        Type::Promise(Box::new(Type::Result(
            Box::new(answer),
            Box::new(Type::Enum(super::ty::HTTP_ERROR.to_string())),
        )))
    }

    /// `N11`: what crosses to a service is a type a program names, never
    /// `Any` or `Map`.
    pub(super) fn boundary_type(&mut self, ty: &Type, what: &str) {
        if open_type(ty) {
            self.error_at_current(
                "N11",
                format!("{what} is `{ty}`, which says nothing of what it holds"),
                "Name it: a `type` with its fields, a dictionary `[String: T]`, or `Json` for JSON whose shape is open",
            );
        }
    }

    /// `N11` for a type as written: `Map` and `Any` in a service's contract
    /// or a read's annotation, each with the fix that writes `Json`.
    pub(super) fn boundary_written(&mut self, ty: &TypeRef, span: Span, what: &str) {
        let found = match ty {
            TypeRef::Map => Some("Map"),
            TypeRef::Any => Some("Any"),
            TypeRef::List(inner) | TypeRef::Optional(inner) | TypeRef::Refined(inner, _) => {
                return self.boundary_written(inner, span, what);
            }
            TypeRef::Dict(k, v) => {
                self.boundary_written(k, span, what);
                return self.boundary_written(v, span, what);
            }
            TypeRef::Result(v, e) => {
                self.boundary_written(v, span, what);
                if let Some(e) = e {
                    self.boundary_written(e, span, what);
                }
                return;
            }
            _ => None,
        };
        if let Some(word) = found {
            self.error(
                span,
                "N11",
                format!("{what} is `{word}`, which says nothing of what it holds"),
                "Name it: a `type` with its fields, a dictionary `[String: T]`, or `Json` for JSON whose shape is open",
            );
            self.plan_last(
                "Write `Json`".to_string(),
                crate::diagnostics::fixes::Plan::Rename {
                    from: word.to_string(),
                    to: "Json".to_string(),
                },
            );
        }
    }
}

/// Whether `ty` holds an `Any` or a `Map` anywhere.
fn open_type(ty: &Type) -> bool {
    match ty {
        Type::Any | Type::Map => true,
        Type::List(t) | Type::Optional(t) | Type::Promise(t) => open_type(t),
        Type::Dict(k, v) | Type::Result(k, v) => open_type(k) || open_type(v),
        _ => false,
    }
}

impl Checker<'_, '_> {
    /// `N08` and `N11` for a resource: `fetch(…)` is gone, and a read of
    /// `http` says what it holds.
    pub(super) fn resource_source(
        &mut self,
        r: &crate::syntax::parser::ast::ResourceDecl,
        span: Span,
    ) {
        use crate::syntax::parser::ast::RequestSource;
        if let Some(ty) = &r.ty {
            self.boundary_written(ty, span, &format!("`{}`", r.name));
        }
        match &r.source {
            RequestSource::Fetch => {
                let method = r.options.iter().find(|o| o.key == "method");
                self.error(
                    span,
                    "N08",
                    format!("`resource {} = fetch(…)`: `fetch` is the browser's, and a read is `http.get(…)`", r.name),
                    "Write `http.get(url, …)` — or `http.post`, `http.put` for another method — with the type it holds: `resource rows: [Row] = http.get(…)`",
                );
                if method.is_none() {
                    self.plan_last(
                        "Write `http.get`".to_string(),
                        crate::diagnostics::fixes::Plan::Rename {
                            from: "fetch".to_string(),
                            to: "http.get".to_string(),
                        },
                    );
                }
            }
            RequestSource::Http(verb) if r.ty.is_none() => self.error(
                span,
                "N11",
                format!(
                    "`resource {}` says nothing of what `http.{verb}` holds",
                    r.name
                ),
                &format!(
                    "Name its type: `resource {}: [Row] = http.{verb}(…)`",
                    r.name
                ),
            ),
            _ => {}
        }
    }

    /// `N08` for an endpoint written as 6.x wrote one: one that does not
    /// say how it fails (`-> T`, no result), or types its failures with
    /// `errors { }` rather than an error enum.
    pub(super) fn endpoint_form(
        &mut self,
        api: &crate::syntax::parser::ast::ApiDecl,
        e: &crate::syntax::parser::ast::Endpoint,
    ) {
        let what = format!("{}.{}", api.name, e.name);
        if !e.errors.is_empty() {
            self.error(
                e.span,
                "N08",
                format!("`{what}` types its failures with `errors {{ }}`, which 7.0 replaced with an error enum"),
                "Declare `enum NameError { unprocessableContent(problem: T) on 422  http(error: HttpError) }` and write `-> T ! NameError`",
            );
            return;
        }
        if matches!(e.returns, Some(TypeRef::Result(..))) {
            return;
        }
        let mut fixed = e.clone();
        fixed.returns = Some(TypeRef::Result(
            Box::new(e.returns.clone().unwrap_or(TypeRef::Unit)),
            None,
        ));
        let written = crate::syntax::print::endpoint(&fixed);
        self.error(
            e.span,
            "N08",
            format!("`{what}` does not say how it fails: a call of it answers with a result"),
            &format!("Write `{}`", crate::syntax::print::endpoint(&fixed)),
        );
        self.plan_last(
            format!("Write `{written}`"),
            crate::diagnostics::fixes::Plan::Replace {
                start: e.span.start as usize,
                end: e.span.end as usize,
                text: written,
            },
        );
    }
}
