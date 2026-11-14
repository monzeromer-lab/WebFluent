//! Expressions: what each one is, the property it reads, chains, matches and comparisons.

use super::{
    BinOp, Checker, Expr, Scalar, StringPart, Type, UnaryOp, adds, case_signature, counts,
    disjoint, expr_text, global_type, ill_formed, in_optional_chain, match_arm,
    scalar_methods_hint,
};

impl Checker<'_, '_> {
    /// The type of `expr`; `expected` guides a case or a lambda.
    /// The type of `expr`. An expression nested past what the parser lets
    /// a program write — a tree made some other way — is `Any`, with one
    /// `E120`, rather than a recursion that runs out of stack.
    pub(super) fn infer(&mut self, expr: &Expr, expected: Option<&Type>) -> Type {
        // The parser counts each link of a chain; a desugared form or a
        // lambda may add a few levels of its own.
        const LIMIT: usize = crate::syntax::parser::MAX_NESTING + 32;
        if self.depth >= LIMIT {
            if !self.too_deep {
                self.too_deep = true;
                self.error_at_current(
                    "E120",
                    crate::syntax::parser::NESTING_MESSAGE.to_string(),
                    "Name a part of it with `let` or `derived`",
                );
            }
            return Type::Any;
        }
        self.depth += 1;
        let ty = self.infer_nested(expr, expected);
        self.depth -= 1;
        ty
    }

    pub(super) fn infer_nested(&mut self, expr: &Expr, expected: Option<&Type>) -> Type {
        match expr {
            // A literal written where one of the language's own types is
            // wanted is read as one, and held to its shape — so a `Url`
            // that is not a URL is caught where it is written, not where
            // it is used.
            Expr::StringLiteral(text) => match expected.map(super::ty::Type::unwrapped) {
                Some(Type::Scalar(s)) if s.is_text() => {
                    if let Some(shape) = ill_formed(s, text) {
                        let name = s.name();
                        // "an Email", but "a Url" and "a Uuid": the letter is not the sound.
                        let article = if name.starts_with('E') { "an" } else { "a" };
                        self.error_at_current(
                            "T01",
                            format!("`\"{text}\"` is not {article} `{name}`"),
                            &shape,
                        );
                    }
                    Type::Scalar(s)
                }
                _ => Type::String,
            },
            Expr::InterpolatedString(parts) => {
                for part in parts {
                    if let StringPart::Expression(e) = part {
                        let ty = self.infer(e, None);
                        // A list, a map or a record in text shows as
                        // `[object Object]` or its items run together.
                        if matches!(
                            ty.unwrapped(),
                            Type::List(_)
                                | Type::Map
                                | Type::Shape(_)
                                | Type::Record(_)
                                | Type::Func(..)
                        ) {
                            let shown = if matches!(ty.unwrapped(), Type::List(_)) {
                                "its items run together with commas"
                            } else {
                                "`[object Object]`"
                            };
                            self.warn(
                                "T20",
                                format!(
                                    "`{}` is `{}`, and in text it shows as {shown}",
                                    expr_text(e),
                                    ty.unwrapped()
                                ),
                                "Show a field of it, or `.join(\", \")` a list of text",
                            );
                        }
                        // A value that may be null shows as "null" (or
                        // "undefined") in the text.
                        if let Type::Optional(inner) = &ty
                            && !inner.is_any()
                        {
                            self.warn(
                                "T19",
                                format!(
                                    "`{}` may be null, and in text it would show as `null`",
                                    expr_text(e)
                                ),
                                &format!(
                                    "Say what to show instead: `{{{} ?? \"\"}}`",
                                    expr_text(e)
                                ),
                            );
                        }
                        // A `Secret` in a string is a secret in a URL, in a
                        // log line, in the page — wherever that string goes.
                        if ty.unwrapped() == Type::Scalar(Scalar::Secret) {
                            self.error_at_current(
                                "T12",
                                format!("`{}` is a `Secret`, and this puts it in text", expr_text(e)),
                                "A secret must not be shown, logged or put in a URL; send it as a value, or read one field of what it unlocks",
                            );
                        }
                    }
                }
                Type::String
            }
            Expr::NumberLiteral(_) => Type::Number,
            // A literal of one of the language's own types. The carrier is
            // checked as itself, and the value is what the name says.
            Expr::Typed(name, carrier) => {
                if let (Some(scalar), Expr::StringLiteral(text)) =
                    (Scalar::of_name(name), carrier.as_ref())
                    && let Some(shape) = ill_formed(scalar, text)
                {
                    self.error_at_current("T01", format!("`@{text}` is not a `{name}`"), &shape);
                }
                self.infer(carrier, None);
                self.world.resolve(Type::Record(name.clone()))
            }
            Expr::BoolLiteral(_) => Type::Bool,
            Expr::Null => Type::Null,
            Expr::Token(_) => Type::Token,
            Expr::Regex(..) => Type::Regex,
            Expr::EnumCase(case) => match expected.map(super::ty::Type::unwrapped) {
                Some(Type::Enum(name)) => {
                    let known = self.world.enums.get(name.as_str()).map(|e| e.case_names());
                    if let Some(cases) = known
                        && !cases.contains(case)
                        && !self.in_args
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
                    } else if let Some(fields) = self.payload_of(&name, case)
                        && !fields.is_empty()
                    {
                        self.error_at_current(
                            "T02",
                            format!(
                                "`.{case}` carries a payload; `.{case}` alone is not a `{name}`"
                            ),
                            &format!("Give it: `{}`", case_signature(case, &fields)),
                        );
                    }
                    Type::Enum(name)
                }
                _ => Type::Case(case.clone()),
            },
            // `.failed("x")`: the case with its payload, checked against the
            // enum's declaration when the position names one.
            Expr::CaseValue(case, args) => {
                if let Some(Type::Enum(name)) = expected.map(super::ty::Type::unwrapped) {
                    let known = self.world.enums.get(name.as_str()).map(|e| e.case_names());
                    match self.payload_of(&name, case) {
                        Some(fields) if !fields.is_empty() => {
                            let params: Vec<Type> = fields.iter().map(|(_, t)| t.clone()).collect();
                            self.call_args(&params, args, &format!("`.{case}`"));
                        }
                        Some(_) => {
                            for a in args {
                                self.infer(a, None);
                            }
                            self.error_at_current(
                                "T02",
                                format!(
                                    "`.{case}` carries nothing; `.{case}(…)` is not a `{name}`"
                                ),
                                &format!("Write `.{case}`"),
                            );
                        }
                        None => {
                            for a in args {
                                self.infer(a, None);
                            }
                            if let Some(cases) = known
                                && !self.in_args
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
                        }
                    }
                    Type::Enum(name)
                } else {
                    for a in args {
                        self.infer(a, None);
                    }
                    Type::Case(case.clone())
                }
            }
            Expr::Identifier(name) => self
                .lookup(name)
                .or_else(|| self.world.consts.get(name).cloned())
                // A service, and a store, are reachable by name from
                // anywhere: the bundle declares each once, at the top. `use`
                // says a body depends on a store, it does not make the name
                // visible — so `Store.member` is checked either way.
                .or_else(|| {
                    self.world
                        .apis
                        .contains_key(name.as_str())
                        .then(|| Type::Api(name.clone()))
                })
                .or_else(|| {
                    self.world
                        .stores
                        .contains_key(name.as_str())
                        .then(|| Type::Store(name.clone()))
                })
                .unwrap_or_else(|| {
                    if !self.is_known_name(name) {
                        self.unresolved(name, false);
                    }
                    global_type(name)
                }),
            // A plain access after a `?.` in the same chain is short-circuited
            // with it: `a?.b.c` is null when `a` is, never a fault.
            Expr::PropertyAccess(base, field) if in_optional_chain(base) => {
                // `a?.b.c`: the `?.` covers `a` being null, not `b`. When
                // `b` may itself be null, `.c` may fail.
                let (base_ty, _) = self.chain(base);
                if let Type::Optional(_) = base_ty {
                    self.error_at_current(
                        "T04",
                        format!(
                            "`{}` may be null even when the `?.` before it is not, so `.{field}` may fail",
                            expr_text(base)
                        ),
                        &format!("Read it through null too: `{}?.{field}`", expr_text(base)),
                    );
                }
                Type::optional(self.property(&base_ty.unwrapped(), base, field))
            }
            Expr::PropertyAccess(base, field) => {
                let base_ty = self.infer(base, None);
                self.property(&base_ty, base, field)
            }
            Expr::IndexAccess(base, index) => {
                let optional = in_optional_chain(base);
                let base_ty = self.infer(base, None);
                let base_ty = if optional {
                    base_ty.unwrapped()
                } else {
                    base_ty
                };
                self.infer(index, None);
                // `todos[0]` is nothing when the list is empty: the item
                // may be null, unless a condition around it says the list
                // has items.
                let fixed_index = matches!(**index, Expr::NumberLiteral(_));
                let ty = match base_ty.unwrapped() {
                    Type::List(inner) if fixed_index && !self.non_empty.contains(base) => {
                        Type::optional(*inner)
                    }
                    Type::List(inner) => *inner,
                    Type::String => Type::String,
                    _ => Type::Any,
                };
                if optional { Type::optional(ty) } else { ty }
            }
            // `?.`: the base may be null, and the result may be too.
            Expr::OptionalProperty(base, field) => {
                let base_ty = self.infer(base, None).unwrapped();
                Type::optional(self.property(&base_ty, base, field))
            }
            Expr::OptionalIndex(base, index) => {
                let base_ty = self.infer(base, None).unwrapped();
                self.infer(index, None);
                Type::optional(match base_ty {
                    Type::List(inner) => *inner,
                    Type::String => Type::String,
                    _ => Type::Any,
                })
            }
            Expr::OptionalMethod(obj, method, args) => {
                // Checked as the plain call on the unwrapped base: the same
                // methods, the same arguments, a result that may be null.
                let ty = self.method_call_on(obj, method, args, true);
                Type::optional(ty)
            }
            Expr::BinaryOp(l, op, r) => {
                // A case on one side is read against the other side's enum:
                // `tone == .quiet` asks the `Tone` enum about `.quiet`.
                let lt = self.infer(l, None);
                let rt = match (op, lt.unwrapped()) {
                    (BinOp::Eq | BinOp::Neq, Type::Enum(_)) if matches!(**r, Expr::EnumCase(_)) => {
                        Type::Case(match &**r {
                            Expr::EnumCase(c) => c.clone(),
                            _ => unreachable!(),
                        })
                    }
                    _ => self.infer(r, None),
                };
                match op {
                    BinOp::Add => {
                        for (side, ty) in [(l, &lt), (r, &rt)] {
                            if ty.unwrapped() == Type::Scalar(Scalar::Secret) {
                                self.error_at_current(
                                    "T12",
                                    format!("`{}` is a `Secret`, and `+` would put it in text", expr_text(side)),
                                    "A secret must not be shown, logged or put in a URL; send it as a value",
                                );
                            }
                        }
                        if lt.unwrapped() == Type::String || rt.unwrapped() == Type::String {
                            Type::String
                        } else if lt == Type::Number && rt == Type::Number {
                            Type::Number
                        } else {
                            for (side, ty) in [(l, &lt), (r, &rt)] {
                                if !adds(ty) {
                                    self.arithmetic_fault("+", side, ty);
                                }
                            }
                            Type::Any
                        }
                    }
                    BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod => {
                        let sign = match op {
                            BinOp::Sub => "-",
                            BinOp::Mul => "*",
                            BinOp::Div => "/",
                            _ => "%",
                        };
                        for (side, ty) in [(l, &lt), (r, &rt)] {
                            if !counts(ty) {
                                self.arithmetic_fault(sign, side, ty);
                            }
                        }
                        Type::Number
                    }
                    BinOp::Eq | BinOp::Neq => {
                        self.comparison(l, &lt, r, &rt, matches!(op, BinOp::Eq));
                        Type::Bool
                    }
                    BinOp::Lt | BinOp::Gt | BinOp::Lte | BinOp::Gte => Type::Bool,
                    BinOp::And | BinOp::Or => Type::Bool,
                    BinOp::NullCoalesce => Type::join(lt.unwrapped(), rt),
                }
            }
            Expr::UnaryOp(op, e) => {
                self.infer(e, None);
                match op {
                    UnaryOp::Not => Type::Bool,
                    UnaryOp::Neg => Type::Number,
                }
            }
            Expr::MethodCall(obj, method, args) if in_optional_chain(obj) => {
                Type::optional(self.method_call_on(obj, method, args, true))
            }
            Expr::MethodCall(obj, method, args) => self.method_call(obj, method, args),
            // `Offers()`: a record whose fields all have defaults, built
            // with none given — a record, as `Offers(definition: true)` is,
            // unless something in reach is a function of that name.
            Expr::FunctionCall(name, args)
                if args.is_empty()
                    && self.lookup(name).is_none()
                    && !self.world.scripts.contains_key(name.as_str())
                    && self.world.record_fields(name).is_some() =>
            {
                self.infer(&Expr::Record(name.clone(), Vec::new()), expected)
            }
            Expr::FunctionCall(name, args) => self.function_call(name, args),
            Expr::ListLiteral(items) => {
                let expected_item = match expected {
                    Some(Type::List(inner)) => Some((**inner).clone()),
                    _ => None,
                };
                let mut item = None;
                for e in items {
                    let ty = self.infer(e, expected_item.as_ref());
                    item = Some(match item {
                        None => ty,
                        Some(prev) => Type::join(prev, ty),
                    });
                }
                Type::list(item.unwrap_or(Type::Any))
            }
            // A map literal has the shape it was written with; a spread inside
            // it makes the keys unknown.
            Expr::MapLiteral(pairs) => {
                let wanted = match expected {
                    Some(Type::Record(name)) => self.world.record_fields(name).map(|fields| {
                        fields
                            .into_iter()
                            .map(|f| (f.name.clone(), self.world.resolve(Type::from_ref(&f.ty))))
                            .collect::<Vec<_>>()
                    }),
                    Some(Type::Shape(fields)) => Some(fields.clone()),
                    _ => None,
                };
                let mut shape: Vec<(String, Type)> = Vec::new();
                let mut spread = false;
                for (k, v) in pairs {
                    if k == "..." {
                        self.infer(v, None);
                        spread = true;
                        continue;
                    }
                    let key = k.trim_matches('"').to_string();
                    let want = wanted
                        .as_ref()
                        .and_then(|w| w.iter().find(|(n, _)| *n == key).map(|(_, t)| t.clone()));
                    let ty = self.infer(v, want.as_ref());
                    if let Some(want) = &want
                        && !ty.assignable_to(want)
                    {
                        self.error_at_current(
                            "T01",
                            format!("`{key}` is `{ty}`, but `{want}` is wanted"),
                            "",
                        );
                    }
                    match shape.iter().position(|(n, _)| *n == key) {
                        Some(at) => shape[at].1 = ty,
                        None => shape.push((key, ty)),
                    }
                }
                // A map written where a record is wanted is that record: a
                // key it does not have, or a field it needs and lacks, is
                // a fault here, where it is written.
                if !spread
                    && let Some(Type::Record(record)) = expected.map(super::ty::Type::unwrapped)
                    && self.world.record_fields(&record).is_some()
                {
                    let keys: Vec<String> = shape.iter().map(|(k, _)| k.clone()).collect();
                    self.record_keys_given(&record, &keys, "the map");
                }
                // An empty literal says nothing about its keys.
                if spread || shape.is_empty() {
                    Type::Map
                } else {
                    Type::Shape(shape)
                }
            }
            // `...items` in a list: its items; the list's own type is theirs.
            Expr::Spread(inner) => match self.infer(inner, None).unwrapped() {
                Type::List(item) => *item,
                Type::Any => Type::Any,
                other => {
                    self.error_at_current(
                        "T08",
                        format!(
                            "`...` spreads a list, but `{}` is `{other}`",
                            expr_text(inner)
                        ),
                        "",
                    );
                    Type::Any
                }
            },
            Expr::Range(a, b, _) => {
                for e in [a, b] {
                    let ty = self.infer(e, Some(&Type::Number));
                    if !ty.assignable_to(&Type::Number) {
                        self.error_at_current(
                            "T01",
                            format!(
                                "a range's end `{}` is `{ty}`, but `Number` is wanted",
                                expr_text(e)
                            ),
                            "",
                        );
                    }
                }
                Type::list(Type::Number)
            }
            Expr::Record(name, fields) => {
                if self.world.record_fields(name).is_some() {
                    let given: Vec<(&str, &Expr)> =
                        fields.iter().map(|(k, v)| (k.as_str(), v)).collect();
                    self.record_fields_given(name, &given, &format!("`{name}(…)`"));
                    Type::Record(name.clone())
                } else if self.world.enums.contains_key(name.as_str()) {
                    Type::Enum(name.clone())
                } else {
                    for (_, v) in fields {
                        self.infer(v, None);
                    }
                    Type::Any
                }
            }
            Expr::Lambda(params, body) => {
                // `() => …` takes nothing: its parameters are the empty string.
                let names: Vec<&str> = params
                    .split(',')
                    .map(str::trim)
                    .filter(|n| !n.is_empty())
                    .collect();
                let param_types: Vec<Type> = match expected {
                    Some(Type::Func(types, _)) => names
                        .iter()
                        .enumerate()
                        .map(|(i, _)| types.get(i).cloned().unwrap_or(Type::Any))
                        .collect(),
                    _ => names.iter().map(|_| Type::Any).collect(),
                };
                self.push_scope();
                for (name, ty) in names.iter().zip(&param_types) {
                    self.refuse_language_name(name, self.current_span);
                    self.narrow(name, ty.clone());
                }
                let ret = self.infer(body, None);
                self.pop_scope();
                Type::Func(param_types, Box::new(ret))
            }
            Expr::Await(e) => {
                if !self.async_ok {
                    let (message, hint) = if self.in_derived {
                        (
                            "a `derived` value cannot `await`: it is worked out at once, whenever what it reads changes",
                            "Fetch it with a `resource`, which is loading until the answer arrives and follows what its address reads",
                        )
                    } else {
                        (
                            "`await` is written in an action, a handler, a timer or a service's hook, and this is none of them",
                            "Move the call into an `action` and call that; to show data as it arrives, use a `resource`",
                        )
                    };
                    self.error_at_current("T17", message.to_string(), hint);
                }
                let ty = self.infer(e, None);
                match ty {
                    Type::Resource(inner) | Type::Promise(inner) => *inner,
                    other => other,
                }
            }
        }
    }

    pub(super) fn property(&mut self, base_ty: &Type, base: &Expr, field: &str) -> Type {
        if let Type::Promise(inner) = base_ty {
            self.error_at_current(
                "T17",
                format!(
                    "`{}` is an async action's result, a promise of `{inner}`; it has no `{field}` until it is awaited",
                    expr_text(base)
                ),
                "`await` it inside an action or a handler: `(await load()).field`",
            );
            self.plan_await(base);
            return Type::Any;
        }
        match base_ty {
            // What a failed request says about itself.
            Type::NetError => match field {
                "message" => Type::String,
                "status" => Type::Number,
                "kind" => Type::String,
                "body" => Type::Any,
                "headers" => Type::Map,
                _ => {
                    self.error_at_current(
                        "T05",
                        format!("a request's error has no field `{field}`"),
                        "It has `.message`, `.status`, `.kind`, `.body` and `.headers`, and matches `.offline`, `.timeout`, `.aborted`, `.parse`, `.network` and `.status(code, body)`",
                    );
                    Type::Any
                }
            },
            // An endpoint of a service.
            Type::Api(name) => {
                if let Some(endpoint) = self.world.endpoint(name, field) {
                    Type::Func(
                        endpoint
                            .params
                            .iter()
                            .map(|p| self.world.resolve(Type::from_ref(&p.prop_type)))
                            .collect(),
                        Box::new(match &endpoint.returns {
                            Some(ty) => self.world.resolve(Type::from_ref(ty)),
                            None => Type::Null,
                        }),
                    )
                } else {
                    let names: Vec<String> = self
                        .world
                        .apis
                        .get(name.as_str())
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
                        .get(name.as_str())
                        .map(|a| a.endpoints.iter().map(|e| e.name.clone()).collect())
                        .unwrap_or_default();
                    let known: Vec<&str> = known.iter().map(String::as_str).collect();
                    self.error_naming(
                        "T06",
                        format!("`{name}` has no endpoint `{field}`"),
                        &format!("Its endpoints are {}", names.join(", ")),
                        field,
                        &known,
                        true,
                    );
                    Type::Any
                }
            }
            // A scalar's fields are its carrier's: money is an amount and a
            // currency, a file a name, a size and a type. Everything the
            // language computes is a call, so nothing here is invented.
            Type::Scalar(scalar) => match (*scalar, field) {
                (Scalar::Money, "amount") => Type::Number,
                (Scalar::Money, "currency") => Type::String,
                (Scalar::File, "name" | "type") => Type::String,
                (Scalar::File, "size") => Type::Number,
                // A secret is not a string: reading it apart is how it
                // leaks a character at a time.
                (Scalar::Secret, _) => {
                    self.error_at_current(
                        "T12",
                        format!("`{}` is a `Secret`, which has no `{field}`: it is not read apart", expr_text(base)),
                        "Hand it on whole — to `api` headers, or a request body — and read what it unlocks",
                    );
                    Type::Any
                }
                (s, "length") if s.is_text() => Type::Number,
                (s, _) => {
                    self.error_at_current(
                        "T05",
                        format!("`{}` has no field `{field}`", s.name()),
                        &scalar_methods_hint(s),
                    );
                    Type::Any
                }
            },
            Type::Shape(fields) => {
                if let Some((_, ty)) = fields.iter().find(|(n, _)| n == field) {
                    ty.clone()
                } else {
                    let names: Vec<String> = fields.iter().map(|(n, _)| format!("`{n}`")).collect();
                    let known: Vec<&str> = fields.iter().map(|(n, _)| n.as_str()).collect();
                    self.error_naming(
                        "T05",
                        format!("`{}` has no field `{field}`", expr_text(base)),
                        &format!("It was written with {}", names.join(", ")),
                        field,
                        &known,
                        true,
                    );
                    Type::Any
                }
            }
            Type::Record(name) => {
                if let Some(ty) = self.world.record_field(name, field) {
                    self.world.resolve(ty)
                } else {
                    let known: Vec<String> = self
                        .world
                        .record_fields(name)
                        .map(|all| all.iter().map(|f| f.name.clone()).collect())
                        .unwrap_or_default();
                    let fields: Vec<String> = known.iter().map(|f| format!("`{f}`")).collect();
                    let known: Vec<&str> = known.iter().map(String::as_str).collect();
                    self.error_naming(
                        "T05",
                        format!("`{name}` has no field `{field}`"),
                        &format!("Its fields are {}", fields.join(", ")),
                        field,
                        &known,
                        true,
                    );
                    Type::Any
                }
            }
            Type::Store(name) => match self.world.stores.get(name.as_str()) {
                Some(members) => {
                    if let Some(ty) = members.get(field) {
                        ty.clone()
                    } else {
                        let mut names: Vec<String> = members.keys().cloned().collect();
                        names.sort();
                        let known: Vec<&str> = names.iter().map(String::as_str).collect();
                        let listed: Vec<String> = names.iter().map(|n| format!("`{n}`")).collect();
                        self.error_naming(
                            "T06",
                            format!("`{name}` has no member `{field}`"),
                            &format!("Its members are {}", listed.join(", ")),
                            field,
                            &known,
                            true,
                        );
                        Type::Any
                    }
                }
                None => Type::Any,
            },
            Type::Optional(inner) => {
                self.error_at_current(
                    "T04",
                    format!(
                        "`{}` may be null, so `.{field}` may fail",
                        expr_text(base)
                    ),
                    "Unwrap it first: `if let x = value { … }`, `value ?? fallback`, `value?.field`, or a check for `!= null`",
                );
                self.plan_last(
                    format!("Read it through null: `?.{field}`"),
                    crate::diagnostics::fixes::Plan::Rename {
                        from: format!(".{field}"),
                        to: format!("?.{field}"),
                    },
                );
                self.property(inner, base, field)
            }
            Type::List(_) => match field {
                "length" => Type::Number,
                _ => Type::Any,
            },
            Type::String => match field {
                "length" => Type::Number,
                _ => Type::Any,
            },
            Type::Resource(inner) => match field {
                "data" => Type::optional((**inner).clone()),
                "state" => Type::String,
                "error" => Type::optional(Type::Any),
                "refreshing" | "pending" | "hasMore" => Type::Bool,
                // `paginate:`'s pages, gathered: each answer if it is a list,
                // or its `items` if it carries them.
                "items" => match &**inner {
                    Type::List(_) => (**inner).clone(),
                    Type::Shape(fields) => match fields.iter().find(|(n, _)| n == "items") {
                        Some((_, ty @ Type::List(_))) => ty.clone(),
                        _ => Type::Any,
                    },
                    Type::Record(name) => match self.world.record_field(name, "items") {
                        Some(ty) => match self.world.resolve(ty) {
                            list @ Type::List(_) => list,
                            _ => Type::Any,
                        },
                        None => Type::Any,
                    },
                    _ => Type::Any,
                },
                _ => Type::Any,
            },
            // `save.pending`: whether a call of the action is under way —
            // which is only ever for an action that awaits something.
            Type::Func(_, ret) => match field {
                "pending" => {
                    let is_action = match base {
                        Expr::Identifier(n) => self.fixed_kind(n) == Some("an action"),
                        Expr::PropertyAccess(store, member) => matches!(store.as_ref(),
                            Expr::Identifier(s) if self.world.store_fixed.get(s.as_str())
                                .and_then(|m| m.get(member)).copied() == Some("an action")),
                        _ => false,
                    };
                    if is_action && !matches!(**ret, Type::Promise(_)) {
                        self.error_at_current(
                            "T17",
                            format!(
                                "`{}` awaits nothing, so `.pending` is always false",
                                expr_text(base)
                            ),
                            "`.pending` is true while a call of an async action runs; an action that awaits nothing finishes before the page repaints",
                        );
                    }
                    Type::Bool
                }
                // `Backend.avatar.progress`: how far an upload has got, 0 to 1
                // — a service's endpoint's, which an action does not have.
                "progress"
                    if matches!(base, Expr::PropertyAccess(owner, _)
                        if matches!(owner.as_ref(), Expr::Identifier(n) if self.world.apis.contains_key(n.as_str()))) =>
                {
                    Type::Number
                }
                _ => {
                    self.error_at_current(
                        "T05",
                        format!(
                            "`{}` is an action; it has no field `{field}`",
                            expr_text(base)
                        ),
                        "An action has `pending`, true while a call of it runs; a service's endpoint also has `progress`",
                    );
                    Type::Any
                }
            },
            _ => Type::Any,
        }
    }

    /// The type of a step of a `?.` chain as if the chain had not been
    /// cut short — what the step itself may be — and whether a `?.` in it
    /// may cut it short.
    pub(super) fn chain(&mut self, expr: &Expr) -> (Type, bool) {
        match expr {
            Expr::OptionalProperty(base, field) => {
                let (base_ty, _) = self.chain(base);
                (self.property(&base_ty.unwrapped(), base, field), true)
            }
            Expr::PropertyAccess(base, field) if in_optional_chain(base) => {
                let (base_ty, short) = self.chain(base);
                if let Type::Optional(_) = base_ty {
                    self.error_at_current(
                        "T04",
                        format!(
                            "`{}` may be null even when the `?.` before it is not, so `.{field}` may fail",
                            expr_text(base)
                        ),
                        &format!("Read it through null too: `{}?.{field}`", expr_text(base)),
                    );
                }
                (self.property(&base_ty.unwrapped(), base, field), short)
            }
            other => {
                let ty = self.infer(other, None);
                let short = in_optional_chain(other);
                if short {
                    (ty.unwrapped(), true)
                } else {
                    (ty, false)
                }
            }
        }
    }

    /// A `match` expression's arms, read back from what it lowers to: every
    /// case of the enum covered when there is no `else`, no case twice, and
    /// no `else` that no value reaches.
    pub(super) fn check_match_chain(&mut self, obj: &Expr, method: &str, args: &[Expr]) {
        let head = Expr::MethodCall(Box::new(obj.clone()), method.to_string(), args.to_vec());
        let Some((subject, _, _)) = match_arm(&head) else {
            return;
        };
        let subject_text = expr_text(subject);
        let mut cases: Vec<String> = Vec::new();
        let mut dupes: Vec<String> = Vec::new();
        let mut at = &head;
        let fallback = loop {
            match match_arm(at) {
                Some((s, case, rest)) if expr_text(s) == subject_text => {
                    if cases.contains(&case) {
                        dupes.push(case);
                    } else {
                        cases.push(case);
                    }
                    at = rest;
                }
                _ => break at,
            }
        };
        let has_else = !matches!(fallback, Expr::MethodCall(_, m, _) if m == "__exhaustive");
        let subject_ty = self.infer_quiet(subject).unwrapped();
        self.match_coverage(&subject_text, &subject_ty, &cases, &dupes, has_else, false);
    }

    /// What a `match` — statement or expression — covers of an enum.
    pub(super) fn match_coverage(
        &mut self,
        subject: &str,
        ty: &Type,
        cases: &[String],
        dupes: &[String],
        has_else: bool,
        statement: bool,
    ) {
        for case in dupes {
            self.error_at_current(
                "T15",
                format!("`.{case}` has two arms in this `match`; the second is never reached"),
                "Keep one of them",
            );
        }
        let Type::Enum(name) = ty else {
            if !has_else {
                self.error_at_current(
                    "T15",
                    format!("this `match` has no `else`, and nothing says `{subject}` is an enum whose every case it covers"),
                    "Add `else { … }`, or declare its type so the cases can be counted",
                );
            }
            return;
        };
        let all = self
            .world
            .enums
            .get(name.as_str())
            .map(|e| e.case_names())
            .unwrap_or_default();
        let missing: Vec<String> = all
            .iter()
            .filter(|c| !cases.contains(c))
            .map(|c| format!("`.{c}`"))
            .collect();
        if !has_else && !missing.is_empty() {
            self.error_at_current(
                "T15",
                format!(
                    "this `match` on `{subject}` has no arm for {}, and no `else`",
                    missing.join(", ")
                ),
                "Add an arm for each, or `else { … }` for the rest",
            );
            // An arm for each, empty: a statement's draws nothing, an
            // expression's is `null`; a payload is named by its fields.
            let arms: Vec<String> = all
                .iter()
                .filter(|c| !cases.contains(c))
                .map(|c| {
                    let names = self
                        .payload_of(name, c)
                        .filter(|fields| !fields.is_empty())
                        .map(|fields| {
                            let names: Vec<&str> = fields.iter().map(|(n, _)| n.as_str()).collect();
                            format!("({})", names.join(", "))
                        })
                        .unwrap_or_default();
                    let body = if statement { "{ }" } else { "{ null }" };
                    format!(".{c}{names} {body}")
                })
                .collect();
            self.plan_last(
                format!("Add an arm for {}", missing.join(", ")),
                crate::diagnostics::fixes::Plan::AddArms { arms },
            );
        } else if has_else && missing.is_empty() && !all.is_empty() {
            self.warn(
                "U10",
                format!("every case of `{name}` has its own arm, so this `match`'s `else` is never reached"),
                "Remove the `else`",
            );
        }
    }

    /// U08: a condition that is always the same — a literal, or a value
    /// compared with itself — so one branch never runs.
    pub(super) fn constant_condition(&mut self, cond: &Expr) {
        let always = match cond {
            Expr::BoolLiteral(b) => Some(if *b { "true" } else { "false" }),
            Expr::NumberLiteral(n) => Some(if *n == 0.0 { "false" } else { "true" }),
            Expr::StringLiteral(t) => Some(if t.is_empty() { "false" } else { "true" }),
            Expr::Null => Some("false"),
            Expr::BinaryOp(l, op, r)
                if matches!(
                    op,
                    BinOp::Eq | BinOp::Neq | BinOp::Lte | BinOp::Gte | BinOp::Lt | BinOp::Gt
                ) && expr_text(l) == expr_text(r)
                    && !matches!(**l, Expr::FunctionCall(..) | Expr::MethodCall(..)) =>
            {
                Some(if matches!(op, BinOp::Eq | BinOp::Lte | BinOp::Gte) {
                    "true"
                } else {
                    "false"
                })
            }
            _ => None,
        };
        if let Some(always) = always {
            self.warn(
                "U08",
                format!(
                    "`{}` is always {always}, so one branch never runs",
                    expr_text(cond)
                ),
                "Use the value that decides, or remove the branch that cannot run",
            );
        }
    }

    /// `==` or `!=` between values that can never be equal: a string and a
    /// number (`"1" == 1` is `false` in the compiled code), a record and a
    /// string, or an enum and a case it does not have. The comparison is
    /// always the same, so it is almost always a mistake.
    pub(super) fn comparison(&mut self, l: &Expr, lt: &Type, r: &Expr, rt: &Type, eq: bool) {
        let (a, b) = (lt.unwrapped(), rt.unwrapped());
        // An enum against a case it does not have.
        for (e, c) in [(&a, &b), (&b, &a)] {
            if let (Type::Enum(name), Type::Case(case)) = (e, c)
                && let Some(decl) = self.world.enums.get(name.as_str())
                && !decl.case_names().contains(case)
            {
                self.error_at_current(
                    "T14",
                    format!(
                        "`{name}` has no case `.{case}`, so this is always {}",
                        if eq { "false" } else { "true" }
                    ),
                    &format!(
                        "`{name}` takes {}",
                        decl.case_names()
                            .iter()
                            .map(|c| format!(".{c}"))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                );
                return;
            }
        }
        if disjoint(&a, &b) {
            let hint = match (&a, &b) {
                (Type::String, Type::Number) | (Type::Number, Type::String) => {
                    "Convert one side: `Number(text) == n`, or `\"{n}\" == text`"
                }
                _ => "Compare values of one type",
            };
            self.error_at_current(
                "T14",
                format!(
                    "`{}` is `{a}` and `{}` is `{b}`, which are never equal, so this is always {}",
                    expr_text(l),
                    expr_text(r),
                    if eq { "false" } else { "true" }
                ),
                hint,
            );
        }
    }

    /// Arithmetic on something that is not a number: `"a" - 1` is `NaN`,
    /// `items * 2` is `NaN`, and `list + 1` is a string nobody wanted.
    pub(super) fn arithmetic_fault(&mut self, sign: &str, side: &Expr, ty: &Type) {
        self.error_at_current(
            "T18",
            format!(
                "`{sign}` takes numbers, but `{}` is `{ty}`",
                expr_text(side)
            ),
            match ty.unwrapped() {
                Type::String => {
                    "Convert it: `Number(value)`; `+` joins text when one side is a string"
                }
                Type::List(_) => "Use its length, or `sum()` its numbers",
                _ => "Use a number field of it",
            },
        );
    }

    /// The payload of `case` on the enum `name`: its fields and their types,
    /// empty for a bare case, `None` for a case the enum lacks.
    pub(super) fn payload_of(&self, name: &str, case: &str) -> Option<Vec<(String, Type)>> {
        let decl = self.world.enums.get(name)?;
        let case = decl.case(case)?;
        Some(
            case.fields
                .iter()
                .map(|f| (f.name.clone(), self.world.resolve(Type::from_ref(&f.ty))))
                .collect(),
        )
    }

    /// An error at the statement or argument being checked. Expressions
    /// carry no span of their own; the nearest enclosing one is used.
    /// Whether `name` means something without the program declaring it:
    /// the browser's globals and values, the names a handler, a loop or a
    /// route brings with it, a service, a store or an imported module.
    pub(super) fn is_known_name(&self, name: &str) -> bool {
        crate::registry::browser::BROWSER_GLOBALS.contains(&name)
            || crate::registry::browser::BROWSER_VALUES.contains(&name)
            || (self.in_handler > 0 && matches!(name, "event" | "e" | "value" | "key"))
            || (self.in_running > 0 && matches!(name, "page" | "pages"))
            || matches!(
                name,
                "params"
                    | "env"
                    | "locale"
                    | "dir"
                    | "undefined"
                    | "NaN"
                    | "Infinity"
                    | "globalThis"
                    | "this"
            )
            || super::names::OVERLAY_FUNCTIONS.contains(&name)
            || self.world.scripts.contains_key(name)
            || self.world.apis.contains_key(name)
            || self.world.stores.contains_key(name)
            || self.world.consts.contains_key(name)
            || self.world.components.contains_key(name)
    }

    /// The type of `e`, reporting nothing: a probe of a value the caller
    /// goes on to check properly, which used to report each finding in it
    /// once per probe.
    pub(super) fn infer_quiet(&mut self, e: &Expr) -> Type {
        let errors = self.info.findings.errors.len();
        let warnings = self.info.findings.warnings.len();
        let unresolved = self.info.unresolved.len();
        let too_deep = self.too_deep;
        let ty = self.infer(e, None);
        // What was found here is dropped, so it may be found again.
        self.too_deep = too_deep;
        self.info.findings.errors.truncate(errors);
        self.info.findings.warnings.truncate(warnings);
        self.info.unresolved.truncate(unresolved);
        ty
    }
}
