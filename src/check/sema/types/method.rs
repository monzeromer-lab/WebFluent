//! Method calls on lists, strings, numbers, records, stores and the language's own types.

use super::{
    Checker, Expr, HoleKind, LIST_METHODS, NUMBER_METHODS, SECRET_SINKS, STRING_METHODS, Scalar,
    Type, expr_text, keeps_present, match_arm, narrowed_names, scalar_method, scalar_methods_hint,
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
            return self.join_at(then_ty, else_ty, &|| {
                format!("if let {name} = {} {{ … }}", expr_text(obj))
            });
        }
        // `match` over an enum as a value: `__is` asks for a case, `__payload`
        // for the payload where the case is the one asked for.
        if method == "__case" && args.is_empty() {
            let subject = self.infer(obj, None);
            self.note_json_subject(obj, &subject);
            return Type::String;
        }
        if (method == "__is" || method == "__payload")
            && args.len() == 1
            && let Expr::StringLiteral(case) = &args[0]
        {
            let subject = self.infer(obj, None);
            self.note_json_subject(obj, &subject);
            // A read binds one name in a match expression: the first place
            // of its case — the value, or what went wrong.
            if let Type::Resource(value, failed) = subject.unwrapped() {
                let cases = super::read::read_cases(&value, &failed);
                return match cases.iter().find(|(c, _)| c == case) {
                    _ if method == "__is" => Type::Bool,
                    Some((_, payload)) => payload.first().cloned().unwrap_or(Type::Null),
                    None => {
                        self.error_at_current(
                            "T02",
                            format!(
                                "`{}` is a read; `.{case}` is not one of its cases",
                                expr_text(obj)
                            ),
                            "A read is `.loading`, `.failed(e)` or `.loaded(value)`",
                        );
                        Type::Error
                    }
                };
            }
            let fields = match subject.unwrapped() {
                Type::Result(value, error) => Some(
                    super::stmt::result_payload(case, &value, &error)
                        .into_iter()
                        .map(|t| (String::new(), t))
                        .collect(),
                ),
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
                    _ => match subject.unwrapped() {
                        Type::Enum(_) => Type::Error,
                        other => self.loose(&other, HoleKind::LooseArm, format!(".{case}")),
                    },
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
            return self.join_at(then_ty, else_ty, &|| {
                format!("if {} {{ … }}", expr_text(obj))
            });
        }
        // A secret handed to the browser itself — logged, stored, written
        // into the document — is a secret anyone at the keyboard can read.
        if let Expr::Identifier(global) = obj
            && SECRET_SINKS.contains(&global.as_str())
            && self.lookup(global).is_none()
        {
            self.secret_args(&format!("{global}.{method}"), args);
        }
        // `Json.string("x")`, and `Settings.decode(raw)`: a type's name
        // where a value would be, which nothing in scope takes.
        if let Expr::Identifier(name) = obj {
            if self.is_json_namespace(name) {
                return self.json_case(method, Some(args));
            }
            if method == "decode"
                && self.lookup(name).is_none()
                && !self.world.stores.contains_key(name.as_str())
                && !self.world.apis.contains_key(name.as_str())
                && !self.world.consts.contains_key(name.as_str())
                && self.world.decodable(name)
            {
                return self.decode_call(name, args);
            }
        }
        // `Backend.users.list.update(f)`, `.invalidate()`: what a page does
        // to what it has read of an endpoint, not a call of it.
        if let Expr::PropertyAccess(owner, ep) = obj
            && let Some(api) = self.world.api_of(owner)
            && let Some(endpoint) = self.world.endpoint(&api, ep)
            && matches!(method, "update" | "invalidate" | "prefetch" | "key" | "url")
        {
            return self.endpoint_method(&api, endpoint, method, args);
        }
        let obj_ty = self.infer(obj, None);
        // JSON has no methods: it is matched, or decoded into a type.
        if matches!(obj_ty.unwrapped(), Type::Enum(n) if n == super::ty::JSON) {
            for a in args {
                self.infer(a, None);
            }
            self.json_fault(
                obj,
                &format!("`.{method}(…)` reads it as something with methods"),
            );
            return Type::Error;
        }
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
                    // A call is a request on its way: a promise of what it
                    // returns, read once it is awaited. An endpoint that
                    // declares nothing returns whatever comes.
                    let returns = Type::Promise(Box::new(match &endpoint.returns {
                        Some(ty) => self.world.resolve(Type::from_ref(ty)),
                        None => self.hole(HoleKind::UntypedResource, format!("{api}.{method}")),
                    }));
                    // A parameter with no default that may not be null must
                    // be given: the path, or the server, needs it.
                    let given: Vec<&str> = args
                        .iter()
                        .filter_map(|a| match a {
                            Expr::MapLiteral(pairs) => Some(pairs),
                            _ => None,
                        })
                        .flatten()
                        .map(|(k, _)| k.trim_matches('"'))
                        .inspect(|k| {
                            // A write sent twice may be taken twice: only one
                            // its contract marks `idempotent` is retried.
                            let write = matches!(endpoint.method.as_str(), "POST" | "PATCH");
                            let idempotent = endpoint.settings.iter().any(|(s, _)| s == "idempotent");
                            if *k == "retry" && write && !idempotent {
                                self.warn(
                                    "N09",
                                    format!("`retry:` on `{api}.{method}`, a `{}` its contract does not mark `idempotent`: it is not retried", endpoint.method),
                                    &format!("If the server takes it once whatever the number of tries, mark it: `{} {method}(…) -> … idempotent`", endpoint.method.to_lowercase()),
                                );
                            }
                        })
                        .collect();
                    let positional = args.iter().any(|a| !matches!(a, Expr::MapLiteral(_)));
                    if !positional {
                        let missing: Vec<String> = endpoint
                            .params
                            .iter()
                            .filter(|p| {
                                p.default.is_none()
                                    && !p.optional
                                    && !matches!(
                                        p.prop_type,
                                        crate::syntax::parser::ast::TypeRef::Optional(_)
                                    )
                                    && !given.contains(&p.name.as_str())
                            })
                            .map(|p| format!("`{}:`", p.name))
                            .collect();
                        if !missing.is_empty() {
                            self.error_at_current(
                                "C01",
                                format!("`{api}.{method}` needs {}", missing.join(", ")),
                                "Pass every parameter that has no default and may not be null",
                            );
                        }
                    }
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
                                            | "refresh"
                                            | "placeholder"
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
                            // A value written without a name is the first
                            // parameter, whatever it is called: `byId(id)`.
                            // Any further one has nowhere to go.
                            let positional = args
                                .iter()
                                .filter(|a| !matches!(a, Expr::MapLiteral(_)))
                                .count();
                            let first = wanted.first().map(|(n, t)| (n.clone(), t.clone()));
                            let given = self.infer(arg, first.as_ref().map(|(_, t)| t));
                            if positional > 1 || first.is_none() {
                                let names: Vec<String> =
                                    wanted.iter().map(|(n, _)| format!("{n}: …")).collect();
                                self.error_at_current(
                                    "T10",
                                    format!("`{api}.{method}` takes one argument without its name, its first: the rest by name"),
                                    &if names.is_empty() {
                                        format!("`{api}.{method}` takes no arguments")
                                    } else {
                                        format!("Write `{api}.{method}({})`", names.join(", "))
                                    },
                                );
                                break;
                            }
                            if let Some((name, want)) = first
                                && !given.assignable_to(&want)
                            {
                                self.error_at_current(
                                    "T01",
                                    format!("`{name}` of `{api}.{method}` is `{want}`, but `{}` is `{given}`", expr_text(arg)),
                                    "",
                                );
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
                    Type::Error
                }
            };
        }
        if let Type::Optional(_) = obj_ty
            && !optional
        {
            self.error_at_current(
                "T04",
                format!("`{}` may be null, so `.{method}()` may fail", expr_text(obj)),
                &format!(
                    "Unwrap it first: `if let x = value {{ … }}`, `value ?? fallback`, `value?.{method}()`, or a check for `!= null`"
                ),
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
            return Type::Error;
        }
        // An object the browser hands over, by the browser table.
        if let Type::Host(name) = &obj_ty {
            let name = name.clone();
            return self.host_method(&name, obj, method, args);
        }
        // A connection's handle, by what it declares.
        if let Type::Connection(kind, send, receive) = &obj_ty {
            let member = self.connection_member(*kind, send, receive, obj, method);
            return match member {
                Type::Func(params, ret) => {
                    self.call_args_optional(
                        &params,
                        args,
                        &format!("`{}.{method}`", expr_text(obj)),
                    );
                    *ret
                }
                Type::Error => {
                    for a in args {
                        self.infer(a, None);
                    }
                    Type::Error
                }
                other => {
                    for a in args {
                        self.infer(a, None);
                    }
                    self.error_at_current(
                        "T10",
                        format!(
                            "`{}.{method}` is a `{other}`, not a function",
                            expr_text(obj)
                        ),
                        "Read it without calling it",
                    );
                    Type::Error
                }
            };
        }
        // A resource does what it says it does, and nothing else.
        if let Type::Resource(..) = &obj_ty {
            for a in args {
                self.infer(a, None);
            }
            let known = ["reload", "invalidate", "loadMore", "cancel"];
            if !known.contains(&method) {
                self.error_at_current(
                    "T16",
                    format!(
                        "`{}` is a resource; it has no method `{method}`",
                        expr_text(obj)
                    ),
                    "A resource has `reload()`, `invalidate()`, `loadMore()` and `cancel()`",
                );
                if let Some(near) =
                    crate::diagnostics::fixes::nearest(method, known.iter().copied())
                {
                    self.plan_last(
                        format!("Change to `{near}`"),
                        crate::diagnostics::fixes::Plan::Rename {
                            from: method.to_string(),
                            to: near.to_string(),
                        },
                    );
                }
            } else if !args.is_empty() {
                self.error_at_current(
                    "T10",
                    format!("`{}.{method}()` takes no arguments", expr_text(obj)),
                    "",
                );
            }
            return Type::Null;
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
                        if let Some(&decl) = self.world.store_decl.get(name.as_str()) {
                            self.note_passed(decl, method, &params, args);
                        }
                        *ret
                    }
                    Some(_) => {
                        for a in args {
                            self.infer(a, None);
                        }
                        self.hole(HoleKind::MapField, format!("{name}.{method}()"))
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
                        if self.world.stores.contains_key(name.as_str()) {
                            Type::Error
                        } else {
                            self.hole(HoleKind::StoreOrder, format!("{name}.{method}()"))
                        }
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
                            _ => self
                                .hole(HoleKind::ListMethod, format!("{}.map(…)", expr_text(obj))),
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
                                _ => Type::list(self.hole(
                                    HoleKind::ListMethod,
                                    format!("{}.flatMap(…)", expr_text(obj)),
                                )),
                            },
                            _ => Type::list(self.hole(
                                HoleKind::ListMethod,
                                format!("{}.flatMap(…)", expr_text(obj)),
                            )),
                        }
                    }
                    _ => {
                        for a in args {
                            self.infer(a, None);
                        }
                        if LIST_METHODS.contains(&method) {
                            self.hole(
                                HoleKind::ListMethod,
                                format!("{}.{method}(…)", expr_text(obj)),
                            )
                        } else {
                            self.unknown_method(&obj_ty, method, LIST_METHODS);
                            Type::Error
                        }
                    }
                }
            }
            // A dictionary's own: its keys, values and entries, whether it
            // has a key, and a copy with one more or one fewer.
            Type::Dict(key, value) => {
                let (key, value) = ((**key).clone(), (**value).clone());
                let dict = obj_ty.clone();
                let (params, ret): (Vec<Type>, Type) = match method {
                    "keys" => (Vec::new(), Type::list(key)),
                    "values" => (Vec::new(), Type::list(value)),
                    "entries" => (
                        Vec::new(),
                        Type::list(Type::Shape(vec![
                            ("key".to_string(), key),
                            ("value".to_string(), value),
                        ])),
                    ),
                    "has" => (vec![key], Type::Bool),
                    "with" => (vec![key, value], dict),
                    "without" => (vec![key], dict),
                    _ => {
                        for a in args {
                            self.infer(a, None);
                        }
                        self.unknown_method(
                            &obj_ty,
                            method,
                            &["keys", "values", "entries", "has", "with", "without"],
                        );
                        return Type::Error;
                    }
                };
                self.call_args(&params, args, &format!("`{}.{method}`", expr_text(obj)));
                ret
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
                    m if STRING_METHODS.contains(&m) => {
                        self.hole(HoleKind::StringMethod, format!("{}.{m}(…)", expr_text(obj)))
                    }
                    _ => {
                        self.unknown_method(&obj_ty, method, STRING_METHODS);
                        Type::Error
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
                Type::Error
            }
            Type::Scalar(scalar) => {
                for a in args {
                    self.infer(a, None);
                }
                let scalar = *scalar;
                if let Some(ty) = scalar_method(scalar, method) {
                    if ty.is_any() {
                        self.hole(HoleKind::Native, format!("{}.{method}()", expr_text(obj)))
                    } else {
                        ty
                    }
                } else {
                    self.error_at_current(
                        "T05",
                        format!("`{}` has no method `{method}`", scalar.name()),
                        &scalar_methods_hint(scalar),
                    );
                    Type::Error
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
                        Type::Error
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
                        Type::Error
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
                        Type::Error
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
                Type::Error
            }
            _ => {
                for a in args {
                    self.infer(a, None);
                }
                self.loose(
                    &obj_ty,
                    HoleKind::ListMethod,
                    format!("{}.{method}(…)", expr_text(obj)),
                )
            }
        }
    }
}

impl Checker<'_, '_> {
    /// A method of an endpoint itself: `update(f)` changes what every read
    /// of it on the page shows and returns what undoes it; `invalidate()`
    /// has them ask again; `prefetch(args)`, `key(args)` and `url(args)`.
    fn endpoint_method(
        &mut self,
        api: &str,
        endpoint: &crate::syntax::parser::ast::Endpoint,
        method: &str,
        args: &[Expr],
    ) -> Type {
        match method {
            "update" => {
                // A paged read shows the items it has gathered, so that is
                // what its change is handed and gives back.
                let answer = match self.paged_items(endpoint, true) {
                    Some(items) => items,
                    None => endpoint.returns.as_ref().map_or(Type::Error, |t| {
                        match self.world.resolve(Type::from_ref(t)) {
                            Type::Result(value, _) => *value,
                            other => other,
                        }
                    }),
                };
                if !matches!(endpoint.method.as_str(), "GET" | "HEAD") {
                    self.error_at_current(
                        "N02",
                        format!(
                            "`{api}.{}` is a `{}`: only what is read can be updated in place",
                            endpoint.name, endpoint.method
                        ),
                        "Update the read the write changes: `Backend.todos.list.update(list => …)`",
                    );
                }
                let change = Type::Func(vec![answer.clone()], Box::new(answer));
                if args.len() != 1 {
                    self.error_at_current(
                        "T10",
                        format!("`{api}.{}.update` takes one function, from what is shown to what is to be shown", endpoint.name),
                        &format!("`{api}.{}.update(list => …)`", endpoint.name),
                    );
                }
                for arg in args {
                    let given = self.infer(arg, Some(&change));
                    if !given.assignable_to(&change) {
                        self.error_at_current(
                            "T01",
                            format!(
                                "`{api}.{}.update` takes a `{change}`, but `{}` is `{given}`",
                                endpoint.name,
                                expr_text(arg)
                            ),
                            "",
                        );
                    }
                }
                // What undoes it.
                Type::Func(Vec::new(), Box::new(Type::Null))
            }
            "invalidate" | "prefetch" => {
                for arg in args {
                    self.infer(arg, None);
                }
                Type::Null
            }
            _ => {
                for arg in args {
                    self.infer(arg, None);
                }
                Type::String
            }
        }
    }
}
