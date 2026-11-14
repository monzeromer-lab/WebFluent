//! Method calls on lists, strings, numbers, records, stores and the language's own types.

use super::{
    Checker, Expr, LIST_METHODS, NUMBER_METHODS, SECRET_SINKS, STRING_METHODS, Scalar, Type,
    expr_text, keeps_present, match_arm, narrowed_names, scalar_method, scalar_methods_hint,
};

impl Checker<'_, '_> {
    pub(super) fn method_call(&mut self, obj: &Expr, method: &str, args: &[Expr]) -> Type {
        self.method_call_on(obj, method, args, false)
    }

    /// A method call; `optional` when written `?.`, so a base that may be
    /// null is what the operator is for, not a fault.
    pub(super) fn method_call_on(
        &mut self,
        obj: &Expr,
        method: &str,
        args: &[Expr],
        optional: bool,
    ) -> Type {
        // `if let x = e { a } else { b }` as a value: `x` is the non-null
        // value in `a`.
        // The head of a `match` expression's arms is checked as a whole:
        // every case covered, none twice.
        if (method == "__if" || method == "__iflet") && !self.in_chain {
            self.check_match_chain(obj, method, args);
        }
        if method == "__exhaustive" {
            return Type::Never;
        }
        if method == "__iflet"
            && args.len() == 2
            && let Expr::Lambda(name, then_expr) = &args[0]
        {
            let value_ty = self.infer(obj, None);
            self.push_scope();
            self.narrow(name, value_ty.unwrapped());
            let was = std::mem::replace(&mut self.in_chain, false);
            let then_ty = self.infer(then_expr, None);
            self.pop_scope();
            self.in_chain = match_arm(&args[1]).is_some();
            let else_ty = self.infer(&args[1], None);
            self.in_chain = was;
            return Type::join(then_ty, else_ty);
        }
        // `match` over an enum as a value: `__is` asks for a case, `__payload`
        // for the payload where the case is the one asked for.
        if method == "__case" && args.is_empty() {
            self.infer(obj, None);
            return Type::String;
        }
        if (method == "__is" || method == "__payload")
            && args.len() == 1
            && let Expr::StringLiteral(case) = &args[0]
        {
            let subject = self.infer(obj, None);
            let fields = match subject.unwrapped() {
                Type::Enum(name) => {
                    let known = self.world.enums.get(name.as_str()).map(|e| e.case_names());
                    let fields = self.payload_of(&name, case);
                    if fields.is_none()
                        && let Some(cases) = known
                    {
                        self.error_at_current(
                            "T02",
                            format!("`{name}` has no case `.{case}`"),
                            &format!(
                                "`{name}` takes {}",
                                cases
                                    .iter()
                                    .map(|c| format!(".{c}"))
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            ),
                        );
                        self.plan_nearest_case(case, &cases);
                    }
                    fields
                }
                _ => None,
            };
            return if method == "__is" {
                Type::Bool
            } else {
                match fields {
                    Some(fields) if fields.len() == 1 => Type::optional(fields[0].1.clone()),
                    Some(fields) if fields.len() > 1 => Type::optional(Type::Map),
                    _ => Type::Any,
                }
            };
        }
        // `if c { a } else { b }` as a value: `c` narrows `a`, as it does a
        // branch; the value is what both arms fit.
        if method == "__if" && args.len() == 2 {
            let cond_ty = self.infer(obj, Some(&Type::Bool));
            self.condition(&cond_ty, obj, self.current_span, "if");
            self.push_scope();
            for name in narrowed_names(obj) {
                if let Some(Type::Optional(inner)) = self.lookup(&name) {
                    self.narrow(&name, *inner);
                }
            }
            let was = std::mem::replace(&mut self.in_chain, false);
            let then_ty = self.infer(&args[0], None);
            self.pop_scope();
            self.in_chain = match_arm(&args[1]).is_some();
            let else_ty = self.infer(&args[1], None);
            self.in_chain = was;
            return Type::join(then_ty, else_ty);
        }
        // A secret handed to the browser itself — logged, stored, written
        // into the document — is a secret anyone at the keyboard can read.
        if let Expr::Identifier(global) = obj
            && SECRET_SINKS.contains(&global.as_str())
            && self.lookup(global).is_none()
        {
            self.secret_args(&format!("{global}.{method}"), args);
        }
        let obj_ty = self.infer(obj, None);
        // `Backend.users(page: 2)`: an endpoint of a service, whose
        // arguments are its parameters and whose value is what it returns.
        if let Type::Api(name) = &obj_ty {
            let api = name.clone();
            return match self.world.endpoint(&api, method) {
                Some(endpoint) => {
                    let wanted: Vec<(String, Type)> = endpoint
                        .params
                        .iter()
                        .map(|p| {
                            (
                                p.name.clone(),
                                self.world.resolve(Type::from_ref(&p.prop_type)),
                            )
                        })
                        .collect();
                    let returns = match &endpoint.returns {
                        Some(ty) => self.world.resolve(Type::from_ref(ty)),
                        None => Type::Null,
                    };
                    for arg in args {
                        // The arguments arrive as one map of names.
                        if let Expr::MapLiteral(pairs) = arg {
                            for (key, value) in pairs {
                                let key = key.trim_matches('"');
                                let want = wanted.iter().find(|(n, _)| n == key).map(|(_, t)| t);
                                let given = self.infer(value, want);
                                match want {
                                    Some(want) if !given.assignable_to(want) => {
                                        self.error_at_current(
                                            "T01",
                                            format!("`{key}:` on `{api}.{method}` is `{given}`, but `{want}` is wanted"),
                                            "",
                                        );
                                    }
                                    // `cache`, `on`, `timeout`, `signal` and
                                    // `paginate` are the call's own, not the
                                    // service's.
                                    None if !matches!(
                                        key,
                                        "cache"
                                            | "on"
                                            | "timeout"
                                            | "signal"
                                            | "headers"
                                            | "as"
                                            | "retry"
                                            | "paginate"
                                    ) =>
                                    {
                                        let names: Vec<String> =
                                            wanted.iter().map(|(n, _)| format!("`{n}`")).collect();
                                        self.error_at_current(
                                            "T10",
                                            format!("`{api}.{method}` has no parameter `{key}`"),
                                            &format!("It takes {}", names.join(", ")),
                                        );
                                    }
                                    _ => {}
                                }
                            }
                        } else {
                            let given = self.infer(arg, wanted.first().map(|(_, t)| t));
                            // An endpoint takes its arguments by name, as one
                            // map: `signUp(email, password)` sent the first
                            // value as that map, and the second nowhere.
                            let positional = args
                                .iter()
                                .filter(|a| !matches!(a, Expr::MapLiteral(_)))
                                .count();
                            if positional > 1
                                || matches!(given, Type::String | Type::Number | Type::Bool)
                            {
                                let names: Vec<String> =
                                    wanted.iter().map(|(n, _)| format!("{n}: …")).collect();
                                self.error_at_current(
                                    "T10",
                                    format!("`{api}.{method}` takes its arguments by name"),
                                    &if names.is_empty() {
                                        format!("`{api}.{method}` takes no arguments")
                                    } else {
                                        format!("Write `{api}.{method}({})`", names.join(", "))
                                    },
                                );
                                break;
                            }
                        }
                    }
                    returns
                }
                // `Backend.invalidate()`: forget everything the service's
                // cache holds — on logout, say. It returns nothing.
                None if method == "invalidate" => {
                    for arg in args {
                        self.infer(arg, None);
                    }
                    if !args.is_empty() {
                        self.error_at_current(
                            "T10",
                            format!("`{api}.invalidate()` takes no arguments"),
                            &format!("It forgets everything the service's cache holds; to forget one endpoint's, call `{api}.endpoint.invalidate()`"),
                        );
                    }
                    Type::Null
                }
                None => {
                    let names: Vec<String> = self
                        .world
                        .apis
                        .get(api.as_str())
                        .map(|a| {
                            a.endpoints
                                .iter()
                                .map(|e| format!("`{}`", e.name))
                                .collect()
                        })
                        .unwrap_or_default();
                    let known: Vec<String> = self
                        .world
                        .apis
                        .get(api.as_str())
                        .map(|a| a.endpoints.iter().map(|e| e.name.clone()).collect())
                        .unwrap_or_default();
                    let known: Vec<&str> = known.iter().map(String::as_str).collect();
                    self.error_naming(
                        "T06",
                        format!("`{api}` has no endpoint `{method}`"),
                        &format!("Its endpoints are {}", names.join(", ")),
                        method,
                        &known,
                        true,
                    );
                    Type::Any
                }
            };
        }
        if let Type::Optional(_) = obj_ty
            && !optional
        {
            self.error_at_current(
                "T04",
                format!("`{}` may be null, so `.{method}()` may fail", expr_text(obj)),
                "Unwrap it first: `if let x = value { … }`, `value ?? fallback`, `value?.{method}()`, or a check for `!= null`",
            );
        }
        let obj_ty = obj_ty.unwrapped();
        if let Type::Promise(inner) = &obj_ty
            && !matches!(method, "then" | "catch" | "finally")
        {
            for a in args {
                self.infer(a, None);
            }
            self.error_at_current(
                "T17",
                format!(
                    "`{}` is an async action's result, a promise of `{inner}`; it has no `{method}` until it is awaited",
                    expr_text(obj)
                ),
                "`await` it inside an action or a handler",
            );
            self.plan_await(obj);
            return Type::Any;
        }
        match &obj_ty {
            // A shape's field that is a function: `form.reset()`.
            Type::Shape(fields)
                if matches!(
                    fields.iter().find(|(n, _)| n == method),
                    Some((_, Type::Func(..)))
                ) =>
            {
                let Some((_, Type::Func(params, ret))) =
                    fields.iter().find(|(n, _)| n == method).cloned()
                else {
                    unreachable!("matched a function field")
                };
                self.call_args(&params, args, &format!("`{}`", expr_text(obj)));
                *ret
            }
            Type::Store(name) => {
                let member = self
                    .world
                    .stores
                    .get(name.as_str())
                    .and_then(|m| m.get(method).cloned());
                match member {
                    Some(Type::Func(params, ret)) => {
                        self.call_args(&params, args, &format!("`{name}.{method}`"));
                        *ret
                    }
                    Some(_) => {
                        for a in args {
                            self.infer(a, None);
                        }
                        Type::Any
                    }
                    None => {
                        if let Some(members) = self.world.stores.get(name.as_str()) {
                            let mut known: Vec<String> = members.keys().cloned().collect();
                            known.sort();
                            let known: Vec<&str> = known.iter().map(String::as_str).collect();
                            self.error_naming(
                                "T06",
                                format!("`{name}` has no action `{method}`"),
                                "",
                                method,
                                &known,
                                true,
                            );
                        }
                        for a in args {
                            self.infer(a, None);
                        }
                        Type::Any
                    }
                }
            }
            Type::List(item) => {
                let item = (**item).clone();
                // JavaScript hands each item, its index and the list itself.
                let lambda = |ret: Type| {
                    Type::Func(
                        vec![item.clone(), Type::Number, Type::list(item.clone())],
                        Box::new(ret),
                    )
                };
                match method {
                    "map" => {
                        let f = self.infer_arg(args, 0, Some(&lambda(Type::Any)));
                        let ret = match f {
                            Type::Func(_, ret) => *ret,
                            _ => Type::Any,
                        };
                        Type::list(ret)
                    }
                    "filter" | "slice" | "concat" | "reverse" => {
                        for (i, _) in args.iter().enumerate() {
                            self.infer_arg(args, i, Some(&lambda(Type::Bool)));
                        }
                        // `xs.filter(x => x != null)` keeps what is there:
                        // a `[T?]` becomes a `[T]`.
                        if method == "filter" && args.first().is_some_and(keeps_present) {
                            Type::list(item.unwrapped())
                        } else {
                            Type::list(item)
                        }
                    }
                    // A comparator is handed two elements, not an element and
                    // an index: `(a, b) => a.age - b.age`.
                    "sort" => {
                        self.infer_arg(
                            args,
                            0,
                            Some(&Type::Func(
                                vec![item.clone(), item.clone()],
                                Box::new(Type::Number),
                            )),
                        );
                        Type::list(item)
                    }
                    "find" => {
                        self.infer_arg(args, 0, Some(&lambda(Type::Bool)));
                        Type::optional(item)
                    }
                    "some" | "every" | "includes" => {
                        self.infer_arg(args, 0, Some(&lambda(Type::Bool)));
                        Type::Bool
                    }
                    // What is pushed must fit the list.
                    "push" => {
                        for (i, arg) in args.iter().enumerate() {
                            let ty = self.infer_arg(args, i, Some(&item));
                            if !ty.assignable_to(&item) {
                                self.error_at_current(
                                    "T01",
                                    format!(
                                        "`{}` is `{ty}`, but `{item}` is wanted",
                                        expr_text(arg)
                                    ),
                                    "",
                                );
                            }
                        }
                        Type::Number
                    }
                    "findIndex" | "indexOf" | "sum" => {
                        for (i, _) in args.iter().enumerate() {
                            self.infer_arg(args, i, Some(&lambda(Type::Bool)));
                        }
                        Type::Number
                    }
                    "join" => {
                        self.infer_arg(args, 0, None);
                        Type::String
                    }
                    "reduce" => {
                        let init = self.infer_arg(args, 1, None);
                        self.infer_arg(
                            args,
                            0,
                            Some(&Type::Func(
                                vec![init.clone(), item.clone(), Type::Number, Type::list(item)],
                                Box::new(init.clone()),
                            )),
                        );
                        init
                    }
                    "toString" => Type::String,
                    "sortBy" => {
                        self.infer_arg(args, 0, Some(&lambda(Type::Any)));
                        Type::list(item)
                    }
                    "groupBy" => {
                        self.infer_arg(args, 0, Some(&lambda(Type::Any)));
                        Type::Map
                    }
                    "unique" | "take" => {
                        for (i, _) in args.iter().enumerate() {
                            self.infer_arg(args, i, None);
                        }
                        Type::list(item)
                    }
                    "first" | "last" => Type::optional(item),
                    "flatMap" => {
                        let f = self.infer_arg(args, 0, Some(&lambda(Type::Any)));
                        match f {
                            Type::Func(_, ret) => match *ret {
                                Type::List(inner) => Type::list(*inner),
                                _ => Type::list(Type::Any),
                            },
                            _ => Type::list(Type::Any),
                        }
                    }
                    _ => {
                        for a in args {
                            self.infer(a, None);
                        }
                        if !LIST_METHODS.contains(&method) {
                            self.unknown_method(&obj_ty, method, LIST_METHODS);
                        }
                        Type::Any
                    }
                }
            }
            Type::String => {
                for a in args {
                    self.infer(a, None);
                }
                match method {
                    "toLowerCase" | "toUpperCase" | "trim" | "trimStart" | "trimEnd"
                    | "replace" | "replaceAll" | "slice" | "substring" | "charAt" | "at"
                    | "toString" | "padStart" | "padEnd" | "repeat" | "normalize"
                    | "capitalize" | "truncate" | "dedent" => Type::String,
                    "indexOf" | "lastIndexOf" | "length" | "charCodeAt" | "search"
                    | "localeCompare" => Type::Number,
                    "includes" | "startsWith" | "endsWith" => Type::Bool,
                    "split" | "lines" | "words" => Type::list(Type::String),
                    // `"x".match(/…/)`: the matches, or null.
                    "match" => Type::optional(Type::list(Type::String)),
                    m if STRING_METHODS.contains(&m) => Type::Any,
                    _ => {
                        self.unknown_method(&obj_ty, method, STRING_METHODS);
                        Type::Any
                    }
                }
            }
            // The language's own types: a date's arithmetic, money's, a
            // URL's parts, a colour's mixing.
            Type::Scalar(Scalar::Secret) => {
                for a in args {
                    self.infer(a, None);
                }
                self.error_at_current(
                    "T12",
                    format!(
                        "`{}` is a `Secret`, which has no `{method}()`: it is not read or changed",
                        expr_text(obj)
                    ),
                    "Hand it on whole — to `api` headers, or a request body",
                );
                Type::Any
            }
            Type::Scalar(scalar) => {
                for a in args {
                    self.infer(a, None);
                }
                let scalar = *scalar;
                if let Some(ty) = scalar_method(scalar, method) {
                    ty
                } else {
                    self.error_at_current(
                        "T05",
                        format!("`{}` has no method `{method}`", scalar.name()),
                        &scalar_methods_hint(scalar),
                    );
                    Type::Any
                }
            }
            Type::Regex => {
                for a in args {
                    self.infer(a, None);
                }
                match method {
                    "test" => Type::Bool,
                    "exec" => Type::optional(Type::list(Type::String)),
                    _ => {
                        self.error_at_current(
                            "T05",
                            format!("a regular expression has no method `{method}`"),
                            "It has `test(text)` and `exec(text)`; a string has `match`, `replace`, `split` and `search`",
                        );
                        Type::Any
                    }
                }
            }
            Type::Number => {
                for a in args {
                    self.infer(a, None);
                }
                match method {
                    "toFixed" | "toString" | "toPrecision" | "toExponential" | "toLocaleString" => {
                        Type::String
                    }
                    "valueOf" => Type::Number,
                    _ => {
                        self.unknown_method(&obj_ty, method, NUMBER_METHODS);
                        Type::Any
                    }
                }
            }
            Type::Bool => {
                for a in args {
                    self.infer(a, None);
                }
                match method {
                    "toString" => Type::String,
                    "valueOf" => Type::Bool,
                    _ => {
                        self.unknown_method(&obj_ty, method, &["toString", "valueOf"]);
                        Type::Any
                    }
                }
            }
            Type::Record(name) => {
                for a in args {
                    self.infer(a, None);
                }
                self.error_at_current(
                    "T05",
                    format!("`{name}` has no method `{method}`; a record holds fields"),
                    "",
                );
                Type::Any
            }
            _ => {
                for a in args {
                    self.infer(a, None);
                }
                Type::Any
            }
        }
    }
}
