//! Function calls: built-ins, project scripts, `format`, and the arguments each takes.

use super::{BUILT_IN_FUNCTIONS, Checker, Expr, Scalar, Type, expr_text, jsdoc_type};

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
            None => Type::Any,
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

    pub(super) fn function_call(&mut self, name: &str, args: &[Expr]) -> Type {
        if self.lookup(name).is_none() {
            match name {
                "alert" | "confirm" | "prompt" => self.secret_args(name, args),
                // In a request's address it is in every log the request
                // passes; as a header or body it is where it belongs.
                "fetch" => self.secret_args("a fetch's address", &args[..args.len().min(1)]),
                _ => {}
            }
        }
        if self.lookup(name).is_none()
            && let Some((file, script)) = self.world.scripts.get(name).copied()
        {
            return self.script_call(file, script, args);
        }
        match self.lookup(name) {
            Some(Type::Func(params, ret)) => {
                self.call_args(&params, args, &format!("`{name}`"));
                *ret
            }
            Some(_) => {
                for a in args {
                    self.infer(a, None);
                }
                Type::Any
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
            None => {
                for a in args {
                    self.infer(a, None);
                }
                if !self.is_known_name(name) && !BUILT_IN_FUNCTIONS.contains(&name) {
                    self.unresolved(name, true);
                }
                match name {
                    "String" | "t" => Type::String,
                    "Number" => Type::Number,
                    "Bool" | "Boolean" => Type::Bool,
                    // Where a `Uuid` comes from.
                    "uuid" => Type::Scalar(Scalar::Uuid),
                    // `keyIs(e, "mod+s")`: whether a key event is that
                    // combination; `keyLabel("mod+s")`: how it is written for
                    // the reader, `⌘S` or `Ctrl+S`.
                    "keyIs" => Type::Bool,
                    "keyLabel" => Type::String,
                    _ => Type::Any,
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
            NameKind::Value => {
                // A value a script declares may hold a function the scanner
                // could not see; the browser decides.
                for a in args {
                    self.infer(a, None);
                }
                return Type::Any;
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
        // back; without one, everything is `Any`.
        let doc = script
            .doc
            .as_deref()
            .map(crate::check::jsdoc::parse)
            .unwrap_or_default();
        for (i, arg) in args.iter().enumerate() {
            let param = params.get(i).or_else(|| params.last().filter(|p| p.rest));
            let wanted = param
                .and_then(|p| doc.param(&p.name))
                .and_then(|d| d.ty.as_deref())
                .map_or(Type::Any, jsdoc_type);
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
        match &script.kind {
            // A class's call is the instance, which the checker takes as
            // given.
            NameKind::Class { .. } => Type::Any,
            _ => doc.returns.as_deref().map_or(Type::Any, jsdoc_type),
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
            let wanted = params.get(i).cloned().unwrap_or(Type::Any);
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
