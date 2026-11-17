//! Function calls: built-ins, project scripts, `format`, and the arguments each takes.

use super::{BUILT_IN_FUNCTIONS, Checker, Expr, HoleKind, Scalar, Type, expr_text, jsdoc_type};

impl Checker<'_, '_> {
    pub(super) fn infer_arg(&mut self, args: &[Expr], i: usize, expected: Option<&Type>) -> Type {
        match args.get(i) {
            Some(e) => {
                let ty = self.infer(e, expected);
                // A function handed to a list's method takes no more
                // arguments than the method gives it: `sort` hands two.
                if let (Type::Func(given, _), Some(Type::Func(offered, _))) = (&ty, expected)
                    && given.len() > offered.len()
                {
                    self.error_at_current(
                        "T10",
                        format!(
                            "`{}` takes {} argument{}, but it is given {}",
                            expr_text(e),
                            given.len(),
                            if given.len() == 1 { "" } else { "s" },
                            offered.len()
                        ),
                        "Name only the arguments the method passes",
                    );
                }
                ty
            }
            // An argument left out — `sort()` with no comparator — is not a
            // value of unknown type: there is none.
            None => expected.cloned().unwrap_or(Type::Null),
        }
    }

    /// T12 for each argument that is a `Secret`, handed to `to`.
    pub(super) fn secret_args(&mut self, to: &str, args: &[Expr]) {
        for arg in args {
            if self.infer_quiet(arg).unwrapped() == Type::Scalar(Scalar::Secret) {
                self.error_at_current(
                    "T12",
                    format!("`{}` is a `Secret`, and `{to}` would give it to anyone who opens the page", expr_text(arg)),
                    "A secret goes to the server only — an `api`'s headers, a request body — never into the browser's own hands",
                );
            }
        }
    }

    /// The `fetch(…)` call in the expression being checked, rewritten as
    /// `http.<verb>(…)` with its `method:` argument gone: the bytes it spans
    /// and what goes there.
    fn fetch_with_method(&self, verb: &str) -> Option<(usize, usize, String)> {
        let source = self.source?;
        let (from, to) = (
            self.current_span.start as usize,
            (self.current_span.end as usize).min(source.len()),
        );
        let text = source.get(from..to)?;
        let at = text.find("fetch(")?;
        // The call's closing parenthesis, strings skipped.
        let (mut depth, mut quoted, mut close) = (0usize, false, None);
        for (i, c) in text[at + 5..].char_indices() {
            match c {
                '"' => quoted = !quoted,
                '(' if !quoted => depth += 1,
                ')' if !quoted => {
                    depth -= 1;
                    if depth == 0 {
                        close = Some(at + 5 + i);
                        break;
                    }
                }
                _ => {}
            }
        }
        let close = close?;
        let inner = &text[at + 6..close];
        // `method: "POST"` with the comma before or after it.
        let key = inner.find("method:")?;
        let value_end = key + inner[key..].find('"')? + 1;
        let value_end = value_end + inner[value_end..].find('"')? + 1;
        let (cut_from, cut_to) = match inner[value_end..].trim_start().strip_prefix(',') {
            Some(rest) => (key, inner.len() - rest.trim_start().len()),
            None => (
                inner[..key]
                    .trim_end()
                    .strip_suffix(',')
                    .map_or(key, str::len),
                value_end,
            ),
        };
        let rewritten = format!("http.{verb}({}{})", &inner[..cut_from], &inner[cut_to..]);
        Some((from + at, from + close + 1, rewritten))
    }

    pub(super) fn function_call(&mut self, name: &str, args: &[Expr]) -> Type {
        if self.lookup(name).is_none() {
            // In a request's address it is in every log the request
            // passes; as a header or body it is where it belongs.
            if name == "fetch" {
                self.secret_args("a fetch's address", &args[..args.len().min(1)]);
            }
        }
        // `fetch(…)` was 6.x's request; 7.0's is the built-in `http`
        // service, whose answer is a result. The browser's own is
        // `window.fetch`.
        if name == "fetch" && self.lookup(name).is_none() && !self.world.scripts.contains_key(name)
        {
            self.error_at_current(
                "N08",
                "`fetch(…)` is gone: a request is `http.get(…)`, `http.post(…)`, whose answer is a result".to_string(),
                "Write `let rows: [Row] = try await http.get(url)` in an action that says what it fails with, or `window.fetch` for the browser's own",
            );
            let method = args.iter().find_map(|a| match a {
                Expr::MapLiteral(p) => p.iter().find(|(k, _)| k == "method").map(|(_, v)| v),
                _ => None,
            });
            match method {
                None => self.plan_last(
                    "Write `http.get`".to_string(),
                    crate::diagnostics::fixes::Plan::Rename {
                        from: "fetch".to_string(),
                        to: "http.get".to_string(),
                    },
                ),
                // `fetch(url, method: "POST", body: b)` is `http.post(url,
                // body: b)`: the verb is the method, and the argument goes.
                Some(Expr::StringLiteral(m))
                    if crate::syntax::parser::ast::HTTP_VERBS
                        .contains(&m.to_lowercase().as_str()) =>
                {
                    if let Some((start, end, text)) = self.fetch_with_method(&m.to_lowercase()) {
                        self.plan_last(
                            format!("Write `http.{}`", m.to_lowercase()),
                            crate::diagnostics::fixes::Plan::Replace { start, end, text },
                        );
                    }
                }
                Some(_) => {}
            }
        }
        if self.lookup(name).is_none()
            && let Some((file, script)) = self.world.scripts.get(name).copied()
        {
            return self.script_call(file, script, args);
        }
        // A function of the browser's, by the browser table.
        if self.lookup(name).is_none()
            && let Some(ty) = self.browser_function(name, args)
        {
            return ty;
        }
        match self.lookup(name) {
            Some(Type::Func(params, ret)) => {
                self.call_args(&params, args, &format!("`{name}`"));
                if self.own_actions.contains_key(name) {
                    self.note_passed(self.decl, name, &params, args);
                }
                *ret
            }
            Some(other) => {
                for a in args {
                    self.infer(a, None);
                }
                self.loose(&other, HoleKind::MapField, format!("{name}(…)"))
            }
            None if name == "format" => self.format_call(args),
            None if name == "setTheme" => {
                if args.len() != 1 {
                    self.error_at_current(
                        "T10",
                        format!("`setTheme` takes one of `\"light\"`, `\"dark\"` or `\"system\"`, but {} arguments are given", args.len()),
                        "",
                    );
                }
                for a in args {
                    let ty = self.infer(a, Some(&Type::String));
                    if !ty.assignable_to(&Type::String) {
                        self.error_at_current(
                            "T01",
                            format!(
                                "`setTheme` takes a `String`, but `{}` is `{ty}`",
                                expr_text(a)
                            ),
                            "One of `\"light\"`, `\"dark\"` or `\"system\"`",
                        );
                    }
                }
                Type::Null
            }
            None if name == "ago" => {
                if args.is_empty() || args.len() > 2 {
                    self.error_at_current(
                        "T10",
                        format!("`ago` takes a date, but {} arguments are given", args.len()),
                        "`ago(date)`, or `ago(date, now)` against a moment of your own",
                    );
                }
                for a in args {
                    self.infer(a, None);
                }
                Type::String
            }
            None if crate::registry::browser::BROWSER_GLOBALS.contains(&name)
                && !BUILT_IN_FUNCTIONS.contains(&name) =>
            {
                for a in args {
                    self.infer(a, None);
                }
                self.untyped_browser(&format!("{name}(…)"), name, &[])
            }
            // `optimistic(Todos.items, items => …)`: the change takes what
            // the holder holds and gives it back, so its parameter is typed.
            None if name == "optimistic" && args.len() == 2 => {
                let held = self.infer(&args[0], None);
                let change = Type::Func(vec![held.clone()], Box::new(held));
                let given = self.infer(&args[1], Some(&change));
                if !given.assignable_to(&change) {
                    self.error_at_current(
                        "T01",
                        format!("`optimistic`'s change is `{given}`, but `{change}` is wanted: it takes what `{}` holds and gives it back", expr_text(&args[0])),
                        "Write the change as a function of the value: `items => items.filter(i => i.id != id)`",
                    );
                }
                Type::Null
            }
            None => {
                for a in args {
                    self.infer(a, None);
                }
                let unresolved = !self.is_known_name(name) && !BUILT_IN_FUNCTIONS.contains(&name);
                if unresolved {
                    self.unresolved(name, true);
                }
                match name {
                    "String" | "t" => Type::String,
                    "Number" => Type::Number,
                    "Bool" | "Boolean" => Type::Bool,
                    // Where a `Uuid` comes from.
                    "uuid" => Type::Scalar(Scalar::Uuid),
                    // `now(every: 1.seconds)`: the time, kept current that often.
                    "now" => Type::Scalar(Scalar::DateTime),
                    // `keyIs(e, "mod+s")`: whether a key event is that
                    // combination; `keyLabel("mod+s")`: how it is written for
                    // the reader, `⌘S` or `Ctrl+S`.
                    "keyIs" => Type::Bool,
                    "keyLabel" => Type::String,
                    // `await ask(…)`: the answer, yes or no.
                    "ask" => Type::Bool,
                    // What returns nothing.
                    "log" | "navigate" | "setLocale" | "beacon" | "replayAnimation"
                    | "optimistic" | "announce" => Type::Null,
                    // The allow-listed markup.
                    "sanitize" => Type::String,
                    _ if unresolved => Type::Error,
                    _ if BUILT_IN_FUNCTIONS.contains(&name) => {
                        self.hole(HoleKind::BuiltinFunction, format!("{name}(…)"))
                    }
                    _ => self.hole(HoleKind::BrowserGlobal, format!("{name}(…)")),
                }
            }
        }
    }

    /// A call to a function or class a project script declares: as many
    /// arguments as it takes — fewer where the last have defaults, any
    /// number past a `...rest`.
    pub(super) fn script_call(
        &mut self,
        file: &str,
        script: &crate::syntax::js_scan::Name,
        args: &[Expr],
    ) -> Type {
        use crate::syntax::js_scan::NameKind;
        let name = script.name.as_str();
        let params = match &script.kind {
            NameKind::Function { params, .. } | NameKind::Class { params, .. } => params,
            // A library's global (`meta.scripts`' `globals`) says nothing of
            // its types: it takes and gives JSON, matched or decoded where it
            // is read.
            NameKind::Value if file == "webfluent.app.json" => {
                let json = Type::Enum(super::ty::JSON.to_string());
                for a in args {
                    self.infer(a, Some(&json));
                }
                return json;
            }
            NameKind::Value => {
                // A value a script declares may hold a function the scanner
                // could not see; the browser decides.
                for a in args {
                    self.infer(a, None);
                }
                return self.hole(HoleKind::Script, format!("{name}(…)"));
            }
        };
        let documented = script
            .doc
            .as_deref()
            .map(crate::check::jsdoc::parse)
            .unwrap_or_default();
        // A default in the code or `[name]` in the comment: either says a
        // call may leave it out.
        let optional = |p: &crate::syntax::js_scan::Param| {
            p.optional || documented.param(&p.name).is_some_and(|d| d.optional)
        };
        let rest = params.last().is_some_and(|p| p.rest);
        let fixed = params.iter().filter(|p| !p.rest).count();
        let required = params
            .iter()
            .rposition(|p| !optional(p) && !p.rest)
            .map_or(0, |i| i + 1);
        if args.len() < required || (!rest && args.len() > fixed) {
            let takes = match (required == fixed, rest) {
                (true, false) => format!("{fixed}"),
                (false, false) => format!("{required} to {fixed}"),
                (_, true) => format!("at least {required}"),
            };
            let shape = params
                .iter()
                .map(|p| {
                    if p.rest {
                        format!("...{}", p.name)
                    } else if optional(p) {
                        format!("{}?", p.name)
                    } else {
                        p.name.clone()
                    }
                })
                .collect::<Vec<_>>()
                .join(", ");
            self.error_at_current(
                "T10",
                format!(
                    "`{name}` takes {takes} argument{}, but {} {} given",
                    if takes == "1" { "" } else { "s" },
                    args.len(),
                    if args.len() == 1 { "is" } else { "are" }
                ),
                &format!(
                    "It is `{name}({shape})`, at {file}:{}:{}",
                    script.line, script.col
                ),
            );
        }
        // What its doc comment says each parameter is, and what it gives
        // back. A function with none says nothing of either (`T31`), and
        // its fix writes one from this call.
        let doc = script.doc.as_deref().map(crate::check::jsdoc::parse);
        let class = matches!(script.kind, NameKind::Class { .. });
        if doc.is_none() && !class {
            let given: Vec<Type> = args.iter().map(|a| self.infer_quiet(a)).collect();
            let block = jsdoc_block(params, &given);
            self.error_at_current(
                "T31",
                format!("`{name}` has no JSDoc, so nothing says what it takes or gives back"),
                &format!(
                    "Say it above the function, at {file}:{}: `/** @param {{number}} max … @returns {{string}} */`; the fix writes one from this call, `@returns {{*}}` for you to narrow",
                    script.line
                ),
            );
            self.plan_last(
                format!("Write `{name}`'s JSDoc from this call"),
                crate::diagnostics::fixes::Plan::InsertAtLine {
                    file: file.to_string(),
                    line: script.line,
                    text: block,
                },
            );
        }
        let doc = doc.unwrap_or_default();
        let unsaid = |cx: &mut Self, text: &str, what: &str| {
            for problem in super::jsdoc::jsdoc_problems(text) {
                cx.error_at_current(
                    "T31",
                    format!("{what} of `{name}`: {problem}"),
                    &format!("At {file}:{}", script.line),
                );
            }
        };
        for (i, arg) in args.iter().enumerate() {
            let param = params.get(i).or_else(|| params.last().filter(|p| p.rest));
            let written = param
                .and_then(|p| doc.param(&p.name))
                .and_then(|d| d.ty.as_deref());
            if let (Some(text), Some(p)) = (written, param) {
                unsaid(self, text, &format!("`{}`", p.name));
            }
            let wanted = written.map_or(Type::Any, |t| self.script_type(jsdoc_type(t)));
            if wanted.is_any() {
                let param = param.map_or("…", |p| p.name.as_str());
                self.hole(HoleKind::Script, format!("{name}({param})"));
            }
            let given = self.infer(arg, Some(&wanted));
            if !given.assignable_to(&wanted) {
                self.error_at_current(
                    "T01",
                    format!(
                        "`{}` of `{name}` is `{wanted}`, but `{}` is `{given}`",
                        param.map_or("an argument", |p| p.name.as_str()),
                        expr_text(arg)
                    ),
                    &format!(
                        "As its doc comment says, at {file}:{}:{}",
                        script.line, script.col
                    ),
                );
            }
        }
        let result = match &script.kind {
            // A class's call is an instance of it, its methods typed by
            // their JSDoc.
            NameKind::Class { .. } => Type::Host(name.to_string()),
            // A function with JSDoc and no `@returns` gives nothing back.
            _ if script.doc.is_some() => match doc.returns.as_deref() {
                Some(text) => {
                    unsaid(self, text, "`@returns`");
                    self.script_type(jsdoc_type(text))
                }
                None => Type::Null,
            },
            _ => Type::Any,
        };
        if result.is_any() {
            self.hole(HoleKind::Script, format!("{name}(…)"))
        } else {
            result
        }
    }

    /// A type a script's JSDoc names, as the program knows it: a record or
    /// an enum of the program's — a `@typedef` among them — a class of a
    /// script's, or an object of the browser's; anything else is unknown.
    pub(super) fn script_type(&self, ty: Type) -> Type {
        match ty {
            Type::Host(name) => {
                if self.world.types.contains_key(name.as_str())
                    || self.world.enums.contains_key(name.as_str())
                {
                    self.world.resolve(Type::Record(name))
                } else if super::host::is_browser_object(&name)
                    || matches!(
                        self.world.scripts.get(name.as_str()),
                        Some((_, n)) if matches!(n.kind, crate::syntax::js_scan::NameKind::Class { .. })
                    )
                {
                    Type::Host(name)
                } else {
                    Type::Any
                }
            }
            Type::List(t) => Type::list(self.script_type(*t)),
            Type::Optional(t) => Type::optional(self.script_type(*t)),
            Type::Dict(k, v) => Type::Dict(k, Box::new(self.script_type(*v))),
            Type::Promise(t) => Type::Promise(Box::new(self.script_type(*t))),
            Type::Func(params, ret) => Type::Func(
                params.into_iter().map(|p| self.script_type(p)).collect(),
                Box::new(self.script_type(*ret)),
            ),
            Type::Shape(fields) => Type::Shape(
                fields
                    .into_iter()
                    .map(|(n, t)| (n, self.script_type(t)))
                    .collect(),
            ),
            other => other,
        }
    }

    /// `format(value, .style, option)`: the style is one the runtime knows,
    /// or a date pattern.
    pub(super) fn format_call(&mut self, args: &[Expr]) -> Type {
        const STYLES: [&str; 10] = [
            "number", "integer", "decimal", "currency", "percent", "compact", "date", "time",
            "datetime", "relative",
        ];
        if args.is_empty() || args.len() > 3 {
            self.error_at_current(
                "T10",
                format!(
                    "`format` takes 1 to 3 arguments, but {} are given",
                    args.len()
                ),
                "`format(value)`, `format(value, .style)` or `format(value, .style, option)`",
            );
        }
        for (i, a) in args.iter().enumerate() {
            if i == 1 {
                match a {
                    Expr::EnumCase(style) if !STYLES.contains(&style.as_str()) => {
                        self.error_at_current(
                            "T02",
                            format!("`format` has no style `.{style}`"),
                            &format!(
                                "It takes {}, or a date pattern such as \"yyyy-MM-dd\"",
                                STYLES
                                    .iter()
                                    .map(|s| format!(".{s}"))
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            ),
                        );
                    }
                    Expr::EnumCase(_) | Expr::StringLiteral(_) => {}
                    other => {
                        let ty = self.infer(other, None);
                        if !ty.assignable_to(&Type::String) && !matches!(ty, Type::Case(_)) {
                            self.error_at_current(
                                "T01",
                                format!("the style of `format` is `{ty}`, but a `.style` or a date pattern is wanted"),
                                "",
                            );
                        }
                    }
                }
                continue;
            }
            self.infer(a, None);
        }
        Type::String
    }

    /// [`call_args`](Self::call_args), where a parameter that may be null
    /// may also be left out at the end: `last()` and `last("price")`.
    pub(super) fn call_args_optional(&mut self, params: &[Type], args: &[Expr], what: &str) {
        let required = params
            .iter()
            .rposition(|p| !matches!(p, Type::Optional(_)))
            .map_or(0, |i| i + 1);
        if (required..=params.len()).contains(&args.len()) {
            let params = &params[..args.len()];
            self.call_args(params, args, what);
        } else {
            self.call_args(params, args, what);
        }
    }

    pub(super) fn call_args(&mut self, params: &[Type], args: &[Expr], what: &str) {
        if params.len() != args.len() {
            self.error_at_current(
                "T10",
                format!(
                    "{what} takes {} argument{}, but {} {} given",
                    params.len(),
                    if params.len() == 1 { "" } else { "s" },
                    args.len(),
                    if args.len() == 1 { "is" } else { "are" }
                ),
                "",
            );
        }
        for (i, arg) in args.iter().enumerate() {
            let wanted = params.get(i).cloned().unwrap_or(Type::Error);
            let given = self.infer(arg, Some(&wanted));
            if !given.assignable_to(&wanted) {
                self.error_at_current(
                    "T01",
                    format!(
                        "argument {} of {what} is `{given}`, but `{wanted}` is wanted",
                        i + 1
                    ),
                    "",
                );
            }
        }
    }
}

/// A JSDoc block for a function of `params`, from what one call passes it:
/// each parameter's type as JSDoc writes it, `*` where it is unknown, and a
/// `@returns {*}` for the author to narrow.
fn jsdoc_block(params: &[crate::syntax::js_scan::Param], given: &[Type]) -> String {
    fn written(ty: &Type) -> String {
        match ty {
            Type::String => "string".into(),
            Type::Number => "number".into(),
            Type::Bool => "boolean".into(),
            Type::List(t) => format!("{}[]", written(t)),
            Type::Optional(t) => format!("?{}", written(t)),
            Type::Record(n) => n.clone(),
            Type::Host(n) if n == "Element" => "HTMLElement".into(),
            Type::Dict(_, v) => format!("Object<string, {}>", written(v)),
            _ => "*".into(),
        }
    }
    let mut out = String::from("/**\n");
    for (i, p) in params.iter().enumerate() {
        let ty = given.get(i).map_or_else(|| "*".to_string(), written);
        let name = if p.optional {
            format!("[{}]", p.name)
        } else if p.rest {
            format!("...{}", p.name)
        } else {
            p.name.clone()
        };
        out.push_str(&format!(" * @param {{{ty}}} {name}\n"));
    }
    out.push_str(" * @returns {*}\n */\n");
    out
}
