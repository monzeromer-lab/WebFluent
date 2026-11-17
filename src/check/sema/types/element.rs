//! Elements and component calls: their props, required arguments, bindings and classes.

use super::{
    Arg, Body, Checker, ComponentDecl, ComponentRef, DOM_SENDS, Expr, HoleKind, LayoutRef,
    PropDecl, PropType, Scalar, Span, Type, TypeRef, UIElement, arg_value, bound_type, expr_text,
    registry, takes_length,
};

impl Checker<'_, '_> {
    /// A `class:` value: a string, a map whose values are conditions, a list
    /// of strings and maps, or a value holding one of those.
    pub(super) fn check_class_value(&mut self, value: &Expr, at: Span, element: &str) {
        match value {
            Expr::MapLiteral(pairs) => {
                for (key, on) in pairs {
                    if key == "..." {
                        self.infer(on, None);
                        continue;
                    }
                    let given = self.infer(on, Some(&Type::Bool));
                    if !given.unwrapped().assignable_to(&Type::Bool) {
                        self.error(
                            at,
                            "T01",
                            format!(
                                "`{}` in `class:` on `{element}` is `{given}`, but `Bool` is wanted",
                                key.trim_matches('"')
                            ),
                            "Each key of a `class:` map is a class, on while its value is true: `{ \"is-done\": todo.done }`",
                        );
                    }
                }
            }
            Expr::ListLiteral(items) => {
                for item in items {
                    self.check_class_value(item, at, element);
                }
            }
            _ => {
                let given = self.infer(value, Some(&Type::String));
                let fits = |t: &Type| {
                    matches!(
                        t,
                        Type::Any
                            | Type::Error
                            | Type::String
                            | Type::Null
                            | Type::Map
                            | Type::Shape(_)
                            | Type::List(_)
                    )
                };
                // A bare `Bool` would paint the class `true`: a condition
                // belongs in a map, `{ "is-on": on }`.
                let ok = match &given {
                    Type::Optional(inner) => fits(inner),
                    other => fits(other),
                };
                if !ok {
                    self.error(
                        at,
                        "T01",
                        format!("`class:` on `{element}` is `{given}`, but a class name is wanted"),
                        "Give a string, a map of class to condition (`{ \"is-on\": on }`) or a list of those",
                    );
                }
            }
        }
    }

    pub(super) fn layout(&mut self, layout: &LayoutRef) {
        let Some(component) = self.world.components.get(layout.name.as_str()) else {
            return;
        };
        self.component_args(&layout.args, &[], &[], component, layout.span);
    }

    pub(super) fn element(&mut self, el: &UIElement, span: Span) {
        // Whether this element shows a list, whose item its parts and slots
        // speak of until it ends.
        let mut pushed = false;
        match &el.component {
            ComponentRef::UserDefined(name) => {
                if let Some(component) = self.world.components.get(name.as_str()) {
                    self.component_args(&el.args, &el.arg_spans, &el.modifiers, component, span);
                } else {
                    for arg in &el.args {
                        self.infer(arg_value(arg), None);
                    }
                }
            }
            // A scoped slot's values, against what its declaration names.
            ComponentRef::BuiltIn(_) if el.slot_name().is_some() => {
                let slot = el.slot_name().unwrap_or("children").to_string();
                let params: Vec<(String, Type)> = self
                    .component
                    .and_then(|c| {
                        c.slots
                            .iter()
                            .find(|s| s.name.as_deref().unwrap_or("children") == slot)
                    })
                    .map(|s| {
                        s.params
                            .iter()
                            .map(|p| {
                                (
                                    p.name.clone(),
                                    self.world.resolve(Type::from_ref(&p.param_type)),
                                )
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                for arg in &el.args {
                    let Arg::Named(key, value) = arg else {
                        continue;
                    };
                    if key == "slot" {
                        continue;
                    }
                    let wanted = params
                        .iter()
                        .find(|(n, _)| n == key)
                        .map(|(_, t)| t.clone());
                    let given = self.infer(value, wanted.as_ref());
                    if let Some(wanted) = wanted
                        && !given.assignable_to(&wanted)
                    {
                        self.error_at_current(
                            "T01",
                            format!("`{key}` of `{slot}` is `{given}`, but `{wanted}` is wanted"),
                            "",
                        );
                    }
                }
            }
            ComponentRef::BuiltIn(name) => {
                let sig = registry::component(name);
                if let Some(item) = sig.and_then(|sig| self.item_of(el, sig)) {
                    self.items.push(item);
                    pushed = true;
                }
                self.builtin_args(el, sig, span);
            }
            ComponentRef::SubComponent(owner, part) => {
                let qualified = format!("{owner}.{part}");
                if let Some(component) = self.world.components.get(qualified.as_str()) {
                    self.component_args(&el.args, &el.arg_spans, &el.modifiers, component, span);
                } else {
                    let sig = registry::part(owner, part);
                    self.builtin_args(el, sig, span);
                }
            }
        }
        if let Some(style) = &el.style_block {
            for prop in &style.properties {
                self.current_span = prop.span;
                let ty = self.infer(&prop.value, None);
                // V07: a number spliced alone where CSS wants a length is
                // a length with no unit, which the browser drops.
                let bare = !matches!(
                    prop.value,
                    Expr::StringLiteral(_)
                        | Expr::InterpolatedString(_)
                        | Expr::NumberLiteral(_)
                        | Expr::Token(_)
                );
                if bare && ty.unwrapped() == Type::Number && takes_length(&prop.name) {
                    self.warn(
                        "V07",
                        format!(
                            "`{}: {{{}}}` is a number with no unit, which the browser drops",
                            prop.name,
                            expr_text(&prop.value)
                        ),
                        &format!(
                            "Give it one: `{}: {{{}}}px` (or `%`, `rem`)",
                            prop.name,
                            expr_text(&prop.value)
                        ),
                    );
                    let spliced = format!("{{{}}}", expr_text(&prop.value));
                    self.plan_last_warning(
                        "Add `px`".to_string(),
                        crate::diagnostics::fixes::Plan::Rename {
                            from: spliced.clone(),
                            to: format!("{spliced}px"),
                        },
                    );
                }
            }
        }
        self.current_span = span;
        for handler in &el.events {
            self.push_scope();
            // A declared event's handler receives the values `emit` sends,
            // each name typed as the event declares it; a DOM handler
            // receives the event.
            let declared = match &el.component {
                ComponentRef::UserDefined(name) => self
                    .world
                    .components
                    .get(name.as_str())
                    .and_then(|c| c.events.iter().find(|e| e.name == handler.event)),
                _ => None,
            };
            if let Some(event) = declared {
                let sends: Vec<Type> = event
                    .params
                    .iter()
                    .map(|p| self.world.resolve(Type::from_ref(&p.param_type)))
                    .collect();
                let what = format!(
                    "`event {}({})` sends {}",
                    event.name,
                    event
                        .params
                        .iter()
                        .map(|p| format!("{}: {}", p.name, Type::from_ref(&p.param_type)))
                        .collect::<Vec<_>>()
                        .join(", "),
                    match sends.len() {
                        0 => "no value".to_string(),
                        1 => "one value".to_string(),
                        n => format!("{n} values"),
                    }
                );
                let names: Vec<&str> = event.params.iter().map(|p| p.name.as_str()).collect();
                self.handler_names_hinted(
                    handler,
                    &sends,
                    &what,
                    &if names.is_empty() {
                        format!("Write `on {} {{ … }}`", event.name)
                    } else {
                        format!(
                            "Name at most {}, in order: `on {}({})`",
                            names.len(),
                            event.name,
                            names.join(", ")
                        )
                    },
                );
            } else {
                // An event of the built-in's own, beside the DOM's.
                let own = match &el.component {
                    ComponentRef::BuiltIn(n) => registry::component(n),
                    ComponentRef::SubComponent(o, n) => registry::part(o, n),
                    ComponentRef::UserDefined(_) => None,
                }
                .is_some_and(|c| c.events.contains(&handler.event.as_str()));
                let event = self.dom_event(handler, own);
                self.handler_names(handler, &[event], DOM_SENDS);
            }
            let was = std::mem::replace(&mut self.async_ok, true);
            self.in_handler += 1;
            self.statements(&handler.body, Body::Imperative);
            self.in_handler -= 1;
            self.async_ok = was;
            self.pop_scope();
        }
        for fill in &el.slot_fills {
            // A fill's names take the types the slot's declaration gives
            // its values, in order.
            let built_in = !matches!(&el.component, ComponentRef::UserDefined(_));
            let params: Vec<Type> = match &el.component {
                ComponentRef::UserDefined(name) => self
                    .world
                    .components
                    .get(name.as_str())
                    .and_then(|c| {
                        c.slots
                            .iter()
                            .find(|s| s.name.as_deref() == Some(fill.name.as_str()))
                    })
                    .map(|s| {
                        s.params
                            .iter()
                            .map(|p| self.world.resolve(Type::from_ref(&p.param_type)))
                            .collect()
                    })
                    .unwrap_or_default(),
                _ => Vec::new(),
            };
            self.push_scope();
            for (i, name) in fill.params.iter().enumerate() {
                let ty = if let Some(ty) = params.get(i) {
                    ty.clone()
                } else if built_in
                    && i == 0
                    && let Some(item) = self.items.last()
                {
                    // A built-in's slot hands over one of its items: a
                    // `DataTable` column's `cell(row)`, a `Tree`'s `row(item)`.
                    item.clone()
                } else {
                    self.current_span = fill.span;
                    let kind = if built_in {
                        HoleKind::BuiltinProp
                    } else {
                        HoleKind::ExtraEventValue
                    };
                    self.hole(kind, format!("{}({name})", fill.name))
                };
                self.bind_fixed(name, ty, fill.span, "a value a slot hands over");
            }
            self.statements(&fill.body, Body::Page);
            self.pop_scope();
        }
        // Inside a paged document's running element, `page` and `pages`
        // are the number of the page it is drawn on and the count.
        let running =
            matches!(&el.component, ComponentRef::BuiltIn(n) if n == "Header" || n == "Footer");
        self.in_running += usize::from(running);
        self.statements(&el.children, Body::Page);
        self.in_running -= usize::from(running);
        if pushed {
            self.items.pop();
        }
    }

    /// What one of the items a built-in shows is: an item of the list its
    /// `rows:`, `items:` or `options:` is given, or of what its `search:`
    /// answers.
    fn item_of(&mut self, el: &UIElement, sig: &'static registry::ComponentSig) -> Option<Type> {
        let list = sig.props.iter().find(|p| p.ty == PropType::Items)?;
        let named = |key: &str| {
            el.args.iter().find_map(|a| match a {
                Arg::Named(k, v) if k == key => Some(v),
                _ => None,
            })
        };
        let given = if let Some(value) = named(list.name) {
            self.infer_quiet(value)
        } else {
            let search = named("search")?;
            let asks = Type::Func(vec![Type::String], Box::new(Type::Null));
            match self.infer_quiet_as(search, Some(&asks)) {
                Type::Func(_, ret) => match *ret {
                    Type::Promise(inner) => *inner,
                    other => other,
                },
                _ => return None,
            }
        };
        match given.unwrapped() {
            Type::List(item) => Some(*item),
            Type::Result(value, _) => match value.unwrapped() {
                Type::List(item) => Some(*item),
                _ => None,
            },
            _ => None,
        }
    }

    /// The arguments of a call to a user component, against its props.
    pub(super) fn component_args(
        &mut self,
        args: &[Arg],
        spans: &[Span],
        modifiers: &[String],
        component: &ComponentDecl,
        span: Span,
    ) {
        self.required_props(args, modifiers, component, span);
        self.in_args = true;
        for (i, arg) in args.iter().enumerate() {
            let at = spans.get(i).copied().unwrap_or(span);
            self.current_span = at;
            let prop = match arg {
                Arg::Positional(_) => component
                    .props
                    .iter()
                    .find(|p| p.positional)
                    .or(component.props.first()),
                Arg::Named(key, _) => component.props.iter().find(|p| &p.name == key),
            };
            let value = arg_value(arg);
            let Some(prop) = prop else {
                self.infer(value, None);
                continue;
            };
            let mut wanted = self.world.resolve(Type::from_ref(&prop.prop_type));
            if prop.optional {
                wanted = Type::optional(wanted);
            }
            let given = self.infer(value, Some(&wanted));
            self.expect(
                &given,
                &wanted,
                at,
                &format!("`{}` of `{}`", prop.name, component.name),
            );
        }
        self.in_args = false;
    }

    /// F01: `bind:` writes what the control holds back to what it names,
    /// so it names something that can change: a state, a store's state, a
    /// field of a loop's item — not a constant, a derived value, a prop or
    /// a value worked out on the spot.
    pub(super) fn check_bindable(&mut self, value: &Expr, at: Span, control: &str) -> bool {
        let refused = match value {
            Expr::Identifier(n) => self.fixed_kind(n).map(|what| format!("`{n}` is {what}")),
            Expr::PropertyAccess(base, member) => match base.as_ref() {
                Expr::Identifier(store) => self
                    .world
                    .store_fixed
                    .get(store.as_str())
                    .and_then(|m| m.get(member))
                    .map(|what| format!("`{store}.{member}` is {what}")),
                _ => None,
            },
            Expr::OptionalProperty(..) | Expr::IndexAccess(..) | Expr::OptionalIndex(..) => None,
            other => Some(format!(
                "`{}` is a value, not something to write to",
                expr_text(other)
            )),
        };
        let Some(why) = refused else {
            return true;
        };
        self.error(
            at,
            "F01",
            format!("`bind:` on `{control}` writes back to what it names, and {why}"),
            "Bind a `state` (or a store's state, or a field of a loop's item); show a value with `value:`",
        );
        false
    }

    /// C01: a prop with no default that a call does not give — by name,
    /// positionally, or as a flag (`.active` for a `Bool`, `.loud` for a
    /// case of an enum-typed prop). In the component it would be
    /// `undefined` where its declaration promises a value.
    pub(super) fn required_props(
        &mut self,
        args: &[Arg],
        modifiers: &[String],
        component: &ComponentDecl,
        span: Span,
    ) {
        let positional = component
            .props
            .iter()
            .find(|p| p.positional)
            .or(component.props.first())
            .map(|p| p.name.as_str());
        let given: Vec<&str> = args
            .iter()
            .filter_map(|a| match a {
                Arg::Named(k, _) => Some(k.as_str()),
                Arg::Positional(_) => positional,
            })
            .collect();
        let by_flag = |prop: &PropDecl| {
            modifiers.iter().any(|m| {
                m == &prop.name
                    || matches!(&prop.prop_type, TypeRef::Named(e)
                        if self.world.enums.get(e.as_str()).is_some_and(|d| d.case_names().contains(m)))
            })
        };
        let missing: Vec<String> = component
            .props
            .iter()
            .filter(|p| {
                p.default.is_none()
                    && !p.optional
                    && !matches!(
                        p.prop_type,
                        TypeRef::Bool | TypeRef::Optional(_) | TypeRef::Any
                    )
                    && !given.contains(&p.name.as_str())
                    && !by_flag(p)
            })
            .map(|p| format!("`{}`", p.name))
            .collect();
        if missing.is_empty() {
            return;
        }
        // The fix: each missing prop, with a value of its type to replace.
        let filled: Vec<String> = component
            .props
            .iter()
            .filter(|p| missing.contains(&format!("`{}`", p.name)))
            .map(|p| format!("{}: {}", p.name, self.placeholder(&p.prop_type)))
            .collect();
        self.error(
            span,
            "C01",
            format!(
                "`{}` is placed without {}, which {} no default",
                component.name,
                missing.join(", "),
                if missing.len() == 1 { "has" } else { "have" }
            ),
            &format!(
                "Pass {}, or give the prop a default in `component {}(…)`",
                missing.join(", "),
                component.name
            ),
        );
        self.plan_last(
            format!("Pass {}", missing.join(", ")),
            crate::diagnostics::fixes::Plan::AddArgument {
                text: filled.join(", "),
            },
        );
    }

    /// A value of `ty` for a fix to write where one is missing: the empty
    /// one where there is one, an enum's first case.
    pub(super) fn placeholder(&self, ty: &TypeRef) -> String {
        match ty {
            TypeRef::String => "\"\"".to_string(),
            TypeRef::Number => "0".to_string(),
            TypeRef::Bool => "false".to_string(),
            TypeRef::Map => "{}".to_string(),
            TypeRef::List(_) => "[]".to_string(),
            TypeRef::Optional(_) | TypeRef::Any | TypeRef::Unit => "null".to_string(),
            TypeRef::Dict(..) => "{}".to_string(),
            TypeRef::Func(params, _) => format!(
                "({}) => null",
                (0..params.len())
                    .map(|i| format!("a{i}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            TypeRef::Result(..) => "null".to_string(),
            TypeRef::Refined(inner, _) => self.placeholder(inner),
            TypeRef::Named(name) => match self.world.enums.get(name.as_str()) {
                Some(e) => e
                    .case_names()
                    .first()
                    .map_or_else(|| "null".to_string(), |c| format!(".{c}")),
                None => format!("{name}()"),
            },
        }
    }

    /// The arguments of a built-in, against the registry's prop types.
    pub(super) fn builtin_args(
        &mut self,
        el: &UIElement,
        sig: Option<&'static registry::ComponentSig>,
        span: Span,
    ) {
        let name = match &el.component {
            ComponentRef::BuiltIn(n) => n.clone(),
            ComponentRef::SubComponent(o, p) => format!("{o}.{p}"),
            ComponentRef::UserDefined(n) => n.clone(),
        };
        self.in_args = true;
        for (i, arg) in el.args.iter().enumerate() {
            let at = el.arg_spans.get(i).copied().unwrap_or(span);
            let at_span = at;
            self.current_span = at;
            let prop = match (arg, sig) {
                (Arg::Positional(_), Some(sig)) => sig.positional.as_ref(),
                (Arg::Named(key, _), Some(sig)) => sig.prop(key),
                _ => None,
            };
            let value = arg_value(arg);
            // `columns: { base: 1, md: 2 }` is one value per breakpoint,
            // each checked as the prop itself; the widths are the
            // stylesheet's, not a type's.
            if crate::project::scoped_css::is_responsive(value)
                && !matches!(arg, Arg::Named(k, _) if k == "class")
                && let Some(prop) = prop
            {
                let wanted = match prop.ty {
                    PropType::Num => Some(Type::Number),
                    PropType::Str | PropType::File(_) => Some(Type::String),
                    PropType::Bool => Some(Type::Bool),
                    PropType::Ty(spelling) => {
                        match self.prop_alternatives(spelling, None).as_slice() {
                            [one] => Some(one.clone()),
                            _ => None,
                        }
                    }
                    _ => None,
                };
                if let Expr::MapLiteral(pairs) = value {
                    for (step, at) in pairs {
                        let given = self.infer(at, wanted.as_ref());
                        if let Some(wanted) = &wanted
                            && !given.assignable_to(wanted)
                        {
                            self.error(
                                at_span,
                                "T01",
                                format!(
                                    "`{}:` at `{}` on `{name}` is `{given}`, but `{wanted}` is wanted",
                                    prop.name,
                                    step.trim_matches('"')
                                ),
                                "",
                            );
                        }
                    }
                }
                continue;
            }
            // Nothing the engine draws takes a secret: it would be shown,
            // or written into an attribute, which is the same thing.
            if let Expr::Identifier(_) | Expr::PropertyAccess(..) = value
                && self.infer_quiet(value).unwrapped() == Type::Scalar(Scalar::Secret)
            {
                self.error(
                    at,
                    "T12",
                    format!(
                        "`{}` is a `Secret`, and `{name}` would show it",
                        expr_text(value)
                    ),
                    "A secret must not reach the page; send it as a value, or show one field of what it unlocks",
                );
            }
            let Some(prop) = prop else {
                self.infer(value, None);
                continue;
            };
            match (prop.ty, prop.name) {
                // `class:` — a string, `{ "is-on": cond }`, or a list of either.
                (_, "class") => self.check_class_value(value, at, &name),
                // `bind:` — the control's value type.
                (PropType::State, "ref") => {
                    if !matches!(value, Expr::Identifier(_)) {
                        self.error(
                            at,
                            "T01",
                            format!(
                                "`ref:` on `{name}` names the handle, but `{}` is given",
                                expr_text(value)
                            ),
                            "Write `ref: nameInput`, then read `nameInput` as the element",
                        );
                    }
                }
                (PropType::State, "bind") => {
                    let refused = name != "Form" && !self.check_bindable(value, at, &name);
                    let wanted = if refused {
                        None
                    } else {
                        bound_type(&name, &el.modifiers)
                    };
                    let given = self.infer(value, wanted.as_ref());
                    if let Some(wanted) = wanted {
                        // A state that starts as `null` may hold the value later.
                        let given = match given {
                            Type::Optional(inner) => *inner,
                            other => other,
                        };
                        if !given.assignable_to(&wanted) {
                            let hint = match (&wanted, &given) {
                                (Type::String, Type::Number) if name == "Input" => {
                                    "A number field is `Input(…).number`, which holds a number"
                                        .to_string()
                                }
                                (Type::Number, Type::String) => {
                                    "Hold text in a `String` state, or drop the `.number`"
                                        .to_string()
                                }
                                _ => format!("`{name}` holds a `{wanted}`"),
                            };
                            self.error(
                                at,
                                "F02",
                                format!(
                                    "`bind:` on `{name}` holds a `{wanted}`, but `{}` is `{given}`",
                                    expr_text(value)
                                ),
                                &hint,
                            );
                        }
                    }
                }
                (PropType::Str | PropType::File(_) | PropType::Path, _) => {
                    let given = self.infer(value, Some(&Type::String));
                    // Text-like props take a number or a bool as text.
                    if !matches!(given, Type::Number | Type::Bool) {
                        self.expect(
                            &given,
                            &Type::String,
                            at,
                            &format!("`{}:` on `{name}`", prop.name),
                        );
                    }
                }
                (PropType::Num, _) => {
                    let given = self.infer(value, Some(&Type::Number));
                    self.expect(
                        &given,
                        &Type::Number,
                        at,
                        &format!("`{}:` on `{name}`", prop.name),
                    );
                }
                (PropType::Bool, _) => {
                    let given = self.infer(value, Some(&Type::Bool));
                    self.expect(
                        &given,
                        &Type::Bool,
                        at,
                        &format!("`{}:` on `{name}`", prop.name),
                    );
                }
                (PropType::Items, prop_name) => {
                    let given = self.infer(value, None);
                    if !matches!(given.unwrapped(), Type::List(_)) && !given.is_any() {
                        self.error(
                            at,
                            "T01",
                            format!(
                                "`{prop_name}:` on `{name}` is a list, but `{}` is `{given}`",
                                expr_text(value)
                            ),
                            "Give it the list it shows",
                        );
                    }
                }
                (PropType::Mount, prop_name) => {
                    let wanted = Type::Func(
                        vec![Type::Host("Element".to_string())],
                        Box::new(Type::Null),
                    );
                    let given = self.infer(value, Some(&wanted));
                    self.expect(&given, &wanted, at, &format!("`{prop_name}:` on `{name}`"));
                }
                (PropType::Ty(spelling), prop_name) => {
                    let handle = self.mount_handle(el);
                    let alternatives = self.prop_alternatives(spelling, handle.as_ref());
                    self.check_alternatives(
                        value,
                        &alternatives,
                        at,
                        &format!("`{prop_name}:` on `{name}`"),
                    );
                }
                (_, _) => {
                    self.infer(value, None);
                }
            }
        }
        self.in_args = false;
    }

    /// What `mount:` on `el` hands back: the handle `update:` and
    /// `cleanup:` are given.
    fn mount_handle(&mut self, el: &UIElement) -> Option<Type> {
        let value = el.args.iter().find_map(|a| match a {
            Arg::Named(k, v) if k == "mount" => Some(v),
            _ => None,
        })?;
        let wanted = Type::Func(
            vec![Type::Host("Element".to_string())],
            Box::new(Type::Null),
        );
        match self.infer_quiet_as(value, Some(&wanted)) {
            Type::Func(_, ret) => Some(*ret),
            _ => None,
        }
    }

    /// The types a registry prop's spelling allows, `Number or Money`, each
    /// read as the checker reads it: `Item` one of the items the enclosing
    /// built-in shows, `Handle` what its `mount:` hands back, `Picture` an
    /// `image`, and an object of the browser's as that object.
    pub(super) fn prop_alternatives(&self, spelling: &str, handle: Option<&Type>) -> Vec<Type> {
        let item = self.items.last().cloned();
        let mut out = Vec::new();
        for part in split_or(spelling) {
            let Some(t) = crate::syntax::parser::v2::parse_type_text(part) else {
                continue;
            };
            let placed = place(Type::from_ref(&t), item.as_ref(), handle);
            out.push(self.host_type_of(placed));
        }
        out
    }

    /// `value` checked against what a prop allows: the first of
    /// `alternatives` it fits, or `T01` naming them all.
    pub(super) fn check_alternatives(
        &mut self,
        value: &Expr,
        alternatives: &[Type],
        at: Span,
        what: &str,
    ) {
        // A value the checker cannot say — an error already reported about
        // it, or what an untyped program leaves `Any` — is not refused again.
        let fits = |given: &Type, wanted: &Type| {
            given.unwrapped().is_any()
                || given.assignable_to(wanted)
                || settles_to(given).is_some_and(|g| g.assignable_to(wanted))
        };
        let chosen = if alternatives.len() == 1 {
            alternatives.first()
        } else {
            // It fits when it is checked against it and nothing is wrong:
            // an ISO string is not a `Date`, but it is a `String`. Failing
            // that, the first it fits, when what is wrong is inside it and
            // reported there.
            let probes: Vec<(Type, bool)> = alternatives
                .iter()
                .map(|alt| self.probe(value, Some(alt)))
                .collect();
            alternatives
                .iter()
                .zip(&probes)
                .find(|(alt, (given, clean))| *clean && fits(given, alt))
                .or_else(|| {
                    alternatives
                        .iter()
                        .zip(&probes)
                        .find(|(alt, (given, _))| fits(given, alt))
                })
                .map(|(alt, _)| alt)
        };
        if let Some(wanted) = chosen {
            let wanted = wanted.clone();
            let given = self.infer(value, Some(&wanted));
            if !fits(&given, &wanted) {
                self.expect(&given, &wanted, at, what);
            }
        } else {
            {
                let given = self.infer(value, alternatives.first());
                if !given.unwrapped().is_any() {
                    if let Some((message, hint)) = alternatives
                        .iter()
                        .find_map(|alt| super::ty::conversion(&given, alt))
                    {
                        self.error(at, "T34", format!("{what} is `{given}`, {message}"), hint);
                        if let Some(alt) = alternatives.first() {
                            self.plan_conversion(value, &given, alt);
                        }
                        return;
                    }
                    let mut wanted: Vec<String> =
                        alternatives.iter().map(|t| format!("`{t}`")).collect();
                    let last = wanted.pop().unwrap_or_default();
                    let takes = if wanted.is_empty() {
                        last
                    } else {
                        format!("{} or {last}", wanted.join(", "))
                    };
                    self.error(
                        at,
                        "T01",
                        format!(
                            "{what} takes {takes}, but `{}` is `{given}`",
                            expr_text(value)
                        ),
                        "",
                    );
                }
            }
        }
    }
}

/// A function whose result is a promise of `R` — an async lambda, a call
/// to a service — as the function of `R` it settles to: `search:` may ask
/// the server.
fn settles_to(t: &Type) -> Option<Type> {
    match t {
        Type::Func(params, ret) => match &**ret {
            Type::Promise(inner) => Some(Type::Func(params.clone(), inner.clone())),
            _ => None,
        },
        _ => None,
    }
}

/// `A or B or C`, split where ` or ` stands outside brackets.
fn split_or(spelling: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut depth = 0i32;
    let mut start = 0;
    let bytes = spelling.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => depth -= 1,
            b' ' if depth == 0 && spelling[i..].starts_with(" or ") => {
                out.push(spelling[start..i].trim());
                start = i + 4;
                i += 4;
                continue;
            }
            _ => {}
        }
        i += 1;
    }
    out.push(spelling[start..].trim());
    out
}

/// A registry spelling's placeholders put in: `Item`, `Handle`, `Picture`.
fn place(t: Type, item: Option<&Type>, handle: Option<&Type>) -> Type {
    let go = |t: Type| place(t, item, handle);
    match t {
        Type::Record(n) if n == "Item" => item.cloned().unwrap_or(Type::Error),
        Type::Record(n) if n == "Handle" => handle.cloned().unwrap_or(Type::Error),
        Type::Record(n) if n == "Picture" => super::image_type(),
        Type::List(i) => Type::list(go(*i)),
        Type::Optional(i) => Type::optional(go(*i)),
        Type::Dict(k, v) => Type::Dict(Box::new(go(*k)), Box::new(go(*v))),
        Type::Func(ps, r) => Type::Func(ps.into_iter().map(go).collect(), Box::new(go(*r))),
        Type::Result(v, e) => Type::Result(Box::new(go(*v)), Box::new(go(*e))),
        other => other,
    }
}
