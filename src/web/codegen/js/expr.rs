//! Expressions, and the list, string and scalar methods they call.

use super::{
    BROWSER_CLASSES, BROWSER_GLOBALS, BROWSER_VALUES, BinOp, CONNECTION_SIGNALS, Expr, JsCodegen,
    RESOURCE_SIGNALS, SCALAR_METHODS, StringPart, UnaryOp, bare, bare_list, js_str, sig,
    template_text, top_level,
};

/// The runtime function a method of one of the language's own types
/// compiles to: `due.plus(days: 3)` is `WF.plus(due, { days: 3 })`.
///
/// Routed by name, because the value carries its own kind at run time —
/// and each function leaves a receiver that has a method of that name to
/// answer for itself, so a record with its own `plus` is never taken over.
/// The helpers the runtime adds to lists and strings, each a runtime function
/// of the same name: `items.sortBy(f)` is `WF.sortBy(items, f)`.
pub const LIST_AND_STRING_HELPERS: &[&str] = &[
    "sortBy",
    "groupBy",
    "unique",
    "take",
    "first",
    "last",
    "capitalize",
    "truncate",
    "dedent",
    "lines",
    "words",
];

/// How the receiver of a method call is held, which decides whether a
/// mutating method assigns back through a signal, through a store's setter,
/// or mutates the value in place.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Holder {
    /// A local, a parameter, a literal: nothing reactive reads it.
    Plain,
    /// A page or component `state`, read as `_name()`.
    Signal,
    /// A store's state: `Cart.items` from outside, `store.items` within.
    StoreMember,
}

/// The one table mapping a WebFluent method onto JavaScript.
///
/// Both the page emitter (`emit_expr`) and the store emitter
/// (`emit_store_expr`) read it. They used to keep a copy each, and the
/// copies drifted: `remove`, `contains`, `toUpper`, `toLower` and all 41 of
/// the scalar methods were in one and not the other, so inside a store
/// action they compiled to `store.items.remove(i)` and `store.due.plus(…)`
/// — methods no JavaScript array, string or JSON value has. Every one threw
/// the first time its action ran, and nothing before this point could see
/// it: parsing, the types and the linters all pass on the source that
/// produces them.
///
/// # Panics
///
/// Never: a scalar method is looked up only once it is known to be one.
pub fn method_to_js(method: &str, obj: &str, args: &[String], holder: Holder) -> String {
    let joined = args.join(", ");
    let with_obj = |name: &str| {
        if args.is_empty() {
            format!("WF.{name}({obj})")
        } else {
            format!("WF.{name}({obj}, {joined})")
        }
    };

    match method {
        // `items.push(x)` and `items.remove(i)` mutate. On a signal or a
        // store's state the new list is assigned back, so what reads it
        // repaints and a persisted one is written; an in-place change would
        // leave the holder with the array it already had.
        "push" => match holder {
            Holder::Signal => {
                format!("{}.set([...{obj}, {joined}])", obj.trim_end_matches("()"))
            }
            Holder::StoreMember => format!("({obj} = [...{obj}, {joined}])"),
            Holder::Plain => format!("{obj}.push({joined})"),
        },
        "remove" => match holder {
            // The comma leaves the new list as the value of the expression.
            // `set` itself returns nothing, so `items = items.remove(i)`
            // used to set the list to the nothing the inner call handed
            // back — the removal happened, then the list became undefined.
            // Reading the signal again costs no new subscription: the
            // argument above already read it.
            Holder::Signal => {
                let sig = obj.trim_end_matches("()");
                format!("({sig}.set(WF.removeAt({obj}, {joined})), {obj})")
            }
            Holder::StoreMember => format!("({obj} = WF.removeAt({obj}, {joined}))"),
            Holder::Plain => format!("{obj}.splice({joined}, 1)"),
        },
        // On a state, `sort` and `reverse` in an expression work on a copy:
        // reading a sorted list must not change the list.
        "sort" | "reverse" if !matches!(holder, Holder::Plain) => {
            format!("[...{obj}].{method}({joined})")
        }
        "filter" => format!("{obj}.filter({joined})"),
        "map" => format!("{obj}.map({joined})"),
        "sum" => format!("{obj}.reduce((a,b) => a+b, 0)"),
        "length" => format!("{obj}.length"),
        "toUpper" => format!("{obj}.toUpperCase()"),
        "toLower" => format!("{obj}.toLowerCase()"),
        "contains" => format!("{obj}.includes({joined})"),
        "trim" => format!("{obj}.trim()"),
        "split" => format!("{obj}.split({joined})"),
        // What the language's own types can do: a date's arithmetic,
        // money's, a URL's parts, a colour's mixing. Each is a plain JSON
        // value at run time, so the work is the runtime's, never a method
        // on the value.
        m if scalar_method(m).is_some() => with_obj(scalar_method(m).expect("just checked")),
        // The helpers the runtime adds to lists and strings.
        m if LIST_AND_STRING_HELPERS.contains(&m) => with_obj(m),
        _ => format!("{obj}.{method}({joined})"),
    }
}

/// The runtime function a scalar method compiles to.
pub fn scalar_method(name: &str) -> Option<&'static str> {
    SCALAR_METHODS
        .iter()
        .find(|(method, _)| *method == name)
        .map(|(_, runtime)| *runtime)
}

/// Whether either side of a comparison is the literal `null`.
pub(super) fn null_side(left: &Expr, right: &Expr) -> bool {
    matches!(left, Expr::Null) || matches!(right, Expr::Null)
}

/// Where an expression is written: on a page, or in a store whose own
/// members are `store_states` and are read through `store.`.
#[derive(Clone, Copy)]
pub(super) enum Cx<'s> {
    Page,
    Store(&'s [String]),
}

impl JsCodegen {
    /// An expression on a page.
    pub(super) fn emit_expr(&self, expr: &Expr) -> String {
        self.emit_expr_in(expr, Cx::Page)
    }

    /// An expression where `cx` says it is written: what is the same on a
    /// page and in a store is written once, and a name, a property read, a
    /// method's holder and a call each resolve as their place reads them.
    pub(super) fn emit_expr_in(&self, expr: &Expr, cx: Cx<'_>) -> String {
        match (expr, cx) {
            (Expr::Typed(_, carrier), _) => self.emit_expr_in(carrier, cx),
            (Expr::StringLiteral(s), _) => js_str(s),
            (Expr::InterpolatedString(parts), _) => {
                let mut out = String::from("`");
                for part in parts {
                    match part {
                        StringPart::Literal(s) => out.push_str(&template_text(s)),
                        StringPart::Expression(e) => {
                            out.push_str("${");
                            out.push_str(&self.emit_expr_in(e, cx));
                            out.push('}');
                        }
                    }
                }
                out.push('`');
                out
            }
            (Expr::Regex(pattern, flags), _) => format!("/{pattern}/{flags}"),
            (Expr::Spread(inner), _) => format!("...{}", self.emit_expr_in(inner, cx)),
            (Expr::Range(a, b, inclusive), _) => format!(
                "WF.range({}, {}, {})",
                self.emit_expr_in(a, cx),
                self.emit_expr_in(b, cx),
                inclusive
            ),
            (Expr::NumberLiteral(n), _) => crate::base::text::number(*n),
            (Expr::BoolLiteral(b), _) => format!("{b}"),
            (Expr::Null, _) => "null".to_string(),
            (Expr::Identifier(name), Cx::Page) => {
                // The names the generated code uses for a route's
                // parameters, a handler's event and a keyed loop's
                // bindings. They are read as themselves — unless the page
                // declared one of them, in which case the declaration
                // wins: a `state key` used to compile to a bare global,
                // so it rendered as nothing and never updated.
                const IMPLICIT: &[&str] = &["params", "value", "key", "event", "e"];

                // A live binding — an arm's, an `if let`'s, a keyed loop's —
                // is read through its accessor, wherever it is read.
                if self.is_live(name) {
                    return format!("${name}()");
                }
                // i18n: locale and dir are reactive i18n signals
                if self.has_i18n() && (name == "locale" || name == "dir") {
                    return format!("WF.i18n.{name}()");
                }
                // Browser globals should NOT be prefixed
                // `BROWSER_GLOBALS`, below: shared with the type checker.
                // A global, unless the program declared the name itself: a
                // `state history` is the page's, not the browser's.
                if BROWSER_GLOBALS.contains(&name.as_str())
                    && !self.body.own_names.contains(name)
                    && !self.body.current_props.contains(name)
                    && !self.body.page_params.contains(name)
                    && !self.loop_bindings.contains(name)
                    && !self.lambda_params.borrow().iter().any(|n| n == name)
                    && !self.consts.contains(name)
                    && !self.body.refs.contains(name)
                    && !self.stores.contains(name)
                {
                    return name.clone();
                }
                // The browser as values, kept current by the runtime, unless
                // the name is the writer's own.
                if BROWSER_VALUES.contains(&name.as_str())
                    && !self.body.own_names.contains(name)
                    && !self.body.current_props.contains(name)
                    && !self.body.page_params.contains(name)
                    && !self.loop_bindings.contains(name)
                    && !self.lambda_params.borrow().iter().any(|n| n == name)
                {
                    return format!("WF.{name}()");
                }
                // What a block binds — a loop's item, a lambda's parameter, a
                // caught error, an `if let` — and what it declares with
                // `let` shadow a prop, a route parameter or a resource of
                // the same name.
                if self.loop_bindings.contains(name)
                    || self.lambda_params.borrow().iter().any(|n| n == name)
                {
                    return bare(name);
                }
                if self.block_lets.contains(name) {
                    return format!("{}()", sig(name));
                }
                // Store references, component props, and built-in names stay as-is
                if self.body.current_props.contains(name) {
                    return format!("_p.{name}");
                }
                if self.body.resources.contains(name) {
                    return sig(name);
                }
                if self.body.page_params.contains(name) {
                    return format!("params.{name}");
                }
                // An action named as a value — `addEventListener("scroll",
                // track)`, a callback handed to a library — is the function
                // itself. It used to be read as a signal, `_track()`, which
                // does not exist.
                if self.body.own_actions.contains(name)
                    && !self.lambda_params.borrow().contains(name)
                {
                    return bare(name);
                }
                // A name a project script declares — `const VERSION = "1"`,
                // `window.ready = …`, a function handed on as a value — is
                // the script's global, unless the body declares it too. It
                // was read as a signal, `_VERSION()`, which nothing
                // declares, so the page threw as it drew.
                if self.body.refs.contains(name) {
                    bare(name)
                } else if self.stores.contains(name)
                    // A `const`, unless the body declares a signal of its
                    // name: a component's `state label` beside a project's
                    // `const label` read the constant, and never changed.
                    || (self.consts.contains(name) && !self.body.body_signals.contains(name))
                {
                    top_level(name)
                } else if
                // The project's `env`, unless the page declares its
                // own: a `state env` read the config's map, and
                // `Text("Live in {env}")` showed `[object Object]`.
                (name == "env" && !self.body.own_names.contains(name))
                    || (IMPLICIT.contains(&name.as_str()) && !self.body.own_names.contains(name))
                    || (self.script_names.contains(name) && !self.body.own_names.contains(name))
                    || name.starts_with('_')
                {
                    name.clone()
                } else {
                    // State variable (signal) — access via _name()
                    format!("{}()", sig(name))
                }
            }
            (Expr::Identifier(name), Cx::Store(store_states)) => {
                if self.is_own_member(name, store_states) {
                    format!("store.{name}")
                } else if self.store_locals.borrow().contains(name) {
                    bare(name)
                } else if self.consts.contains(name) || self.stores.contains(name) {
                    top_level(name)
                } else if BROWSER_VALUES.contains(&name.as_str())
                    && !self.body.own_names.contains(name)
                {
                    format!("WF.{name}()")
                } else {
                    name.clone()
                }
            }

            // `save.pending` and `Store.save.pending`: whether a call of an
            // async action is under way, a signal.
            (Expr::PropertyAccess(base, prop), Cx::Page)
                if prop == "pending" && self.is_action_ref(base) =>
            {
                match base.as_ref() {
                    Expr::Identifier(name) => format!("{}_pending()", sig(name)),
                    other => format!("{}.pending()", self.emit_expr(other)),
                }
            }
            // What a resource or a connection holds — `rows.state`,
            // `rows.data`, `rows.error`, `rows.refreshing`, `rows.pending`, a
            // paged resource's `rows.items` and `rows.hasMore`, a socket's
            // `chat.messages` and `chat.closure` —
            // is a signal on the handle. It is read where it is written, so
            // `Text("{rows.state}")` shows the state and follows it; it used to
            // show the signal's own source, once.
            // A store's connection, `Lang.css.pending`: the same.
            (Expr::PropertyAccess(base, prop), Cx::Page)
                if CONNECTION_SIGNALS.contains(&prop.as_str())
                    && matches!(base.as_ref(), Expr::PropertyAccess(owner, name)
                        if matches!(owner.as_ref(), Expr::Identifier(store)
                            if self.store_connections.iter().any(|(s, c)| s == store && c == name))) =>
            {
                format!("{}.{}()", self.emit_expr(base), prop)
            }
            // A store's resource, `Feed.rows.state`: the same.
            (Expr::PropertyAccess(base, prop), Cx::Page)
                if RESOURCE_SIGNALS.contains(&prop.as_str())
                    && matches!(base.as_ref(), Expr::PropertyAccess(owner, name)
                        if matches!(owner.as_ref(), Expr::Identifier(store)
                            if self.store_resources.iter().any(|(s, r)| s == store && r == name))) =>
            {
                format!("{}.{}()", self.emit_expr(base), prop)
            }
            (Expr::PropertyAccess(base, prop), Cx::Page)
                if matches!(
                    prop.as_str(),
                    "state"
                        | "data"
                        | "error"
                        | "refreshing"
                        | "pending"
                        | "items"
                        | "hasMore"
                        | "messages"
                        | "closure"
                ) && matches!(base.as_ref(), Expr::Identifier(n)
                    if self.body.resources.contains(n)
                        && !self.lambda_params.borrow().contains(n)
                        && !self.loop_bindings.contains(n)) =>
            {
                format!("{}.{}()", self.emit_expr(base), prop)
            }
            (Expr::PropertyAccess(base, prop), Cx::Page) => {
                let base_str = self.emit_expr(base);
                format!("{base_str}.{prop}")
            }
            // A connection of this store's: what it holds is a signal.
            (Expr::PropertyAccess(base, prop), Cx::Store(store_states))
                if CONNECTION_SIGNALS.contains(&prop.as_str())
                    && matches!(base.as_ref(), Expr::Identifier(n)
                        if self.own_connections.contains(n) && self.is_own_member(n, store_states)) =>
            {
                format!("{}.{prop}()", self.emit_store_expr(base, store_states))
            }
            // A resource of this store's: what it holds is a signal.
            (Expr::PropertyAccess(base, prop), Cx::Store(store_states))
                if RESOURCE_SIGNALS.contains(&prop.as_str())
                    && matches!(base.as_ref(), Expr::Identifier(n)
                        if self.own_resources.contains(n) && self.is_own_member(n, store_states)) =>
            {
                format!("{}.{prop}()", self.emit_store_expr(base, store_states))
            }
            (Expr::PropertyAccess(base, prop), Cx::Store(store_states))
                if prop == "pending" && self.is_action_ref(base) =>
            {
                format!("{}.pending()", self.emit_store_expr(base, store_states))
            }
            (Expr::PropertyAccess(base, prop), Cx::Store(store_states)) => {
                let base_str = self.emit_store_expr(base, store_states);
                format!("{base_str}.{prop}")
            }

            (Expr::IndexAccess(base, index), _) => {
                let base_str = self.emit_expr_in(base, cx);
                let idx_str = self.emit_expr_in(index, cx);
                format!("{base_str}[{idx_str}]")
            }
            // `?.` is JavaScript's own.
            (Expr::OptionalProperty(base, prop), _) => {
                format!("{}?.{}", self.emit_expr_in(base, cx), prop)
            }
            (Expr::OptionalIndex(base, index), _) => {
                format!(
                    "{}?.[{}]",
                    self.emit_expr_in(base, cx),
                    self.emit_expr_in(index, cx)
                )
            }
            (Expr::OptionalMethod(obj, method, args), _) => {
                let args: Vec<String> = args.iter().map(|a| self.emit_expr_in(a, cx)).collect();
                format!(
                    "{}?.{}({})",
                    self.emit_expr_in(obj, cx),
                    method,
                    args.join(", ")
                )
            }
            (Expr::BinaryOp(left, op, right), _) => {
                let l = self.emit_expr_in(left, cx);
                let r = self.emit_expr_in(right, cx);
                let op_str = match op {
                    BinOp::Add => "+",
                    BinOp::Sub => "-",
                    BinOp::Mul => "*",
                    BinOp::Div => "/",
                    BinOp::Mod => "%",
                    // Against `null`, a value that is absent — a query key
                    // the address lacks, a field a record leaves out — is
                    // `undefined` in the browser, and counts as null.
                    BinOp::Eq if null_side(left, right) => "==",
                    BinOp::Neq if null_side(left, right) => "!=",
                    BinOp::Eq => "===",
                    BinOp::Neq => "!==",
                    BinOp::Lt => "<",
                    BinOp::Gt => ">",
                    BinOp::Lte => "<=",
                    BinOp::Gte => ">=",
                    BinOp::And => "&&",
                    BinOp::Or => "||",
                    BinOp::NullCoalesce => "??",
                };
                format!("({l} {op_str} {r})")
            }
            (Expr::UnaryOp(op, expr), _) => {
                let e = self.emit_expr_in(expr, cx);
                match op {
                    UnaryOp::Not => format!("!{e}"),
                    UnaryOp::Neg => format!("-{e}"),
                }
            }
            (Expr::MethodCall(obj, method, args), _) => {
                // The arm a `match` with no `else` cannot reach: the checker
                // proved every case has one.
                if method == "__exhaustive" {
                    return "null".to_string();
                }
                // The case of an enum value, its payload aside.
                if method == "__case" && args.is_empty() {
                    return format!("WF.caseOf({})", self.emit_expr_in(obj, cx));
                }
                // `match` over an enum as a value: the case, and the payload
                // where the case is the one asked for.
                if (method == "__is" || method == "__payload") && args.len() == 1 {
                    let subject = self.emit_expr_in(obj, cx);
                    let case = self.emit_expr_in(&args[0], cx);
                    return if method == "__is" {
                        format!("(WF.caseOf({subject}) === {case})")
                    } else {
                        format!("WF.payload({subject}, {case})")
                    };
                }
                if method == "__if" && args.len() == 2 {
                    // Conditional expression
                    let cond = self.emit_expr_in(obj, cx);
                    let then_val = self.emit_expr_in(&args[0], cx);
                    let else_val = self.emit_expr_in(&args[1], cx);
                    return format!("({cond} ? {then_val} : {else_val})");
                }
                // `if let x = e { a } else { b }`: the value is bound once,
                // and read as a plain name in `a`.
                if method == "__iflet"
                    && args.len() == 2
                    && let Expr::Lambda(name, then_expr) = &args[0]
                {
                    let value = self.emit_expr_in(obj, cx);
                    let then_val =
                        self.with_locals(cx, [name.clone()], || self.emit_expr_in(then_expr, cx));
                    let else_val = self.emit_expr_in(&args[1], cx);
                    let name = bare(name);
                    // A branch that awaits needs its arrow to be async, and
                    // the arrow awaited, or the page's code doesn't parse.
                    if then_val.contains("await ") || else_val.contains("await ") {
                        return format!(
                            "(await (async ({name}) => {name} != null ? {then_val} : {else_val})({value}))"
                        );
                    }
                    return format!(
                        "(({name}) => {name} != null ? {then_val} : {else_val})({value})"
                    );
                }

                match cx {
                    Cx::Page => {
                        let obj_str = self.emit_expr(obj);
                        let args_str: Vec<String> =
                            args.iter().map(|a| self.emit_expr(a)).collect();

                        // A store's own actions come first. `Todos.remove(id)` calls
                        // the action the store declares, not the list method of the
                        // name; a store is an object, and mapping the call onto a
                        // list's `splice` leaves it calling something that is not
                        // there. The list methods stay for `Todos.items.remove(0)`,
                        // which reaches the list rather than the store.
                        if matches!(obj.as_ref(), Expr::Identifier(s) if self.stores.contains(s)) {
                            return format!("{}.{}({})", obj_str, method, args_str.join(", "));
                        }

                        // Map WebFluent methods to JS, through the one table the
                        // store emitter reads too.
                        let holder = if self.is_state_signal(obj) {
                            Holder::Signal
                        } else if self.is_store_member(obj, &obj_str) {
                            Holder::StoreMember
                        } else {
                            Holder::Plain
                        };
                        method_to_js(method, &obj_str, &args_str, holder)
                    }
                    Cx::Store(store_states) => {
                        let obj_str = self.emit_store_expr(obj, store_states);
                        let args_str: Vec<String> = args
                            .iter()
                            .map(|a| self.emit_store_expr(a, store_states))
                            .collect();
                        // Another store's own actions come first, as they do on a
                        // page: `OpenFiles.preview(path)` calls the action the
                        // store declares. Through the table it was the runtime's
                        // `WF.preview(OpenFiles, path)`, and the action never ran
                        // with its arguments.
                        if let Expr::Identifier(s) = obj.as_ref()
                            && self.stores.contains(s)
                            && !store_states.contains(s)
                            && !self.store_locals.borrow().contains(s)
                        {
                            return format!("{}.{}({})", obj_str, method, args_str.join(", "));
                        }
                        // The same table the page emitter reads. A bare name that
                        // is one of the store's own states is held by the store, so
                        // a mutating method assigns back through it.
                        let holder = match obj.as_ref() {
                            Expr::Identifier(n) if self.is_own_member(n, store_states) => {
                                Holder::StoreMember
                            }
                            Expr::PropertyAccess(base, _) => match base.as_ref() {
                                Expr::Identifier(s) if self.stores.contains(s) => {
                                    Holder::StoreMember
                                }
                                _ => Holder::Plain,
                            },
                            _ => Holder::Plain,
                        };
                        method_to_js(method, &obj_str, &args_str, holder)
                    }
                }
            }
            (Expr::FunctionCall(name, args), Cx::Page) => {
                // i18n: t("key") or t("key", name: value, ...)
                if name == "t" && self.has_i18n() {
                    if args.is_empty() {
                        return "\"\"".to_string();
                    }
                    let key = self.emit_expr(&args[0]);
                    if args.len() == 1 {
                        return format!("WF.i18n.t({key})");
                    }
                    // Remaining args are named params for interpolation
                    // They come as FunctionCall args — could be positional expressions
                    // In practice the parser sees t("key", name: value) where name: value
                    // is parsed as separate expressions. We need to handle both positional
                    // and the case where the parser gave us the values.
                    let params: Vec<String> = args[1..].iter().map(|a| self.emit_expr(a)).collect();
                    return format!("WF.i18n.t({}, {})", key, params.join(", "));
                }
                // i18n: setLocale("ar")
                if name == "setLocale" && self.has_i18n() {
                    let args_str: Vec<String> = args.iter().map(|a| self.emit_expr(a)).collect();
                    return format!("WF.i18n.setLocale({})", args_str.join(", "));
                }

                // `now(every: 1.seconds)`: the browser's clock, ticking as often
                // as the page asks. It compiled to a bare `now(…)`, which
                // nothing declares, and threw where the page read it.
                if name == "now" && !self.body.own_names.contains(name) {
                    let args_str: Vec<String> = args.iter().map(|a| self.emit_expr(a)).collect();
                    return format!("WF.now({})", args_str.join(", "));
                }
                // WF runtime functions
                if name == "replayAnimation" {
                    let args_str: Vec<String> = args.iter().map(|a| self.emit_expr(a)).collect();
                    return format!("WF.replay({})", args_str.join(", "));
                }
                // `format(n, .currency)`, `ago(date)`: the runtime's, unless
                // the page declares an action of the name.
                if matches!(
                    name.as_str(),
                    // `beacon(url, data)` outlives the page that sends it,
                    // which is the whole of why it exists.
                    // `uuid()` is where a `Uuid` comes from: the platform's
                    // generator, or a version-4 layout where there is none.
                    "format"
                        | "ago"
                        | "setTheme"
                        | "beacon"
                        | "optimistic"
                        | "sanitize"
                        | "uuid"
                        | "keyIs"
                        | "keyLabel"
                ) && !self.body.own_actions.contains(name)
                {
                    let args_str: Vec<String> = args.iter().map(|a| self.emit_expr(a)).collect();
                    return format!("WF.{name}({})", args_str.join(", "));
                }

                let args_str: Vec<String> = args.iter().map(|a| self.emit_expr(a)).collect();
                // `await fetch(url, opts)`: the parsed body, as a `resource`
                // reads it; a failed response throws.
                if name == "fetch" && !self.body.own_actions.contains(name) {
                    let mut args_str = args_str;
                    if let Some(first) = args.first() {
                        args_str[0] = self.encoded_address(first);
                    }
                    return format!("WF.request({})", args_str.join(", "));
                }
                // Check if it's a store function
                if self.stores.contains(name) {
                    format!(
                        "{}.{}({})",
                        top_level(name),
                        args_str.first().unwrap_or(&String::new()),
                        args_str.get(1..).unwrap_or(&[]).join(", ")
                    )
                } else if (self.script_classes.contains(name)
                    || BROWSER_CLASSES.contains(&name.as_str()))
                    && !self.body.own_names.contains(name)
                    && !self.consts.contains(name)
                {
                    // A class a project script or the browser declares:
                    // constructed, since calling one without `new` throws.
                    format!("new {}({})", name, args_str.join(", "))
                } else if self.body.current_props.contains(name)
                    || (self.body.own_names.contains(name) && !self.body.own_actions.contains(name))
                {
                    // A state, derived or prop that holds a function —
                    // `derived scale = (x) => x * factor`, then
                    // `scale(3)` — calls what the name holds, read as any
                    // other read of it is. It compiled to a bare
                    // `scale(3)`, which nothing declares.
                    let callee = self.emit_expr(&Expr::Identifier(name.clone()));
                    format!("{}({})", callee, args_str.join(", "))
                } else if self.body.own_actions.contains(name)
                    || self.loop_bindings.contains(name)
                    || self.lambda_params.borrow().contains(name)
                {
                    // The program's own, written as its declaration is.
                    format!("{}({})", bare(name), args_str.join(", "))
                } else if self.consts.contains(name) {
                    format!("{}({})", top_level(name), args_str.join(", "))
                } else {
                    format!("{}({})", name, args_str.join(", "))
                }
            }
            (Expr::FunctionCall(name, args), Cx::Store(store_states)) => {
                let args_str: Vec<String> = args
                    .iter()
                    .map(|a| self.emit_store_expr(a, store_states))
                    .collect();
                // A call to one of the store's own actions goes through the
                // store; anything else is a global (`Math.round` is a method
                // call, but `parseInt` is a plain function).
                if self.is_own_member(name, store_states) {
                    format!("store.{}({})", name, args_str.join(", "))
                } else if (name == "t" || name == "setLocale") && self.has_i18n() {
                    // A store's words, translated as a page's are.
                    format!("WF.i18n.{}({})", name, args_str.join(", "))
                } else if matches!(
                    name.as_str(),
                    // The same set a page reaches: a store's action is where
                    // `optimistic` belongs, and `uuid` where an id is made.
                    "format"
                        | "ago"
                        | "setTheme"
                        | "beacon"
                        | "optimistic"
                        | "sanitize"
                        | "uuid"
                        | "keyIs"
                        | "keyLabel"
                ) {
                    format!("WF.{}({})", name, args_str.join(", "))
                } else if name == "fetch" {
                    format!("WF.request({})", args_str.join(", "))
                } else if self.store_locals.borrow().contains(name) {
                    format!("{}({})", bare(name), args_str.join(", "))
                } else if self.consts.contains(name) {
                    format!("{}({})", top_level(name), args_str.join(", "))
                } else if self.script_classes.contains(name)
                    || BROWSER_CLASSES.contains(&name.as_str())
                {
                    format!("new {}({})", name, args_str.join(", "))
                } else {
                    format!("{}({})", name, args_str.join(", "))
                }
            }

            (Expr::ListLiteral(items), _) => {
                let items_str: Vec<String> =
                    items.iter().map(|i| self.emit_expr_in(i, cx)).collect();
                format!("[{}]", items_str.join(", "))
            }
            (Expr::MapLiteral(entries) | Expr::Record(_, entries), _) => {
                let entries_str: Vec<String> = entries
                    .iter()
                    .map(|(k, v)| match k.as_str() {
                        "..." => format!("...{}", self.emit_expr_in(v, cx)),
                        _ => format!("{}: {}", k, self.emit_expr_in(v, cx)),
                    })
                    .collect();
                // Parenthesised, so a map literal is an object wherever it
                // lands — as an arrow function's body a bare `{` is a block.
                format!("({{ {} }})", entries_str.join(", "))
            }
            // A lambda's parameters shadow what a program or a store declares
            // of their names, in its body.
            (Expr::Lambda(param, body), _) => {
                let names = param
                    .split(',')
                    .map(str::trim)
                    .filter(|p| !p.is_empty())
                    .map(str::to_string);
                let body_str = self.with_locals(cx, names, || self.emit_expr_in(body, cx));
                format!("(({}) => {})", bare_list(param), body_str)
            }
            (Expr::EnumCase(case), _) => format!("\"{case}\""),
            (Expr::CaseValue(case, args), _) => {
                let mut items = vec![format!("\"{case}\"")];
                items.extend(args.iter().map(|a| self.emit_expr_in(a, cx)));
                format!("[{}]", items.join(", "))
            }
            // A design token is its custom property.
            (Expr::Token(name), _) => format!("\"var(--{name})\""),
            (Expr::Await(inner), _) => format!("(await {})", self.emit_expr_in(inner, cx)),
        }
    }

    /// Run `f` with `names` shadowing what a program or a store declares,
    /// in the stack `cx` reads: a lambda's parameters, an `if let`'s
    /// binding. They are unbound when `f` returns.
    pub(super) fn with_locals<R>(
        &self,
        cx: Cx<'_>,
        names: impl IntoIterator<Item = String>,
        f: impl FnOnce() -> R,
    ) -> R {
        let stack = match cx {
            Cx::Page => &self.lambda_params,
            Cx::Store(_) => &self.store_locals,
        };
        let bound = stack.borrow().len();
        stack.borrow_mut().extend(names);
        let out = f();
        stack.borrow_mut().truncate(bound);
        out
    }
}
