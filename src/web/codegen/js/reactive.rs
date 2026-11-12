//! Bindings and liveness: what a bound control writes, a change in place, and whether an expression reads state.

use super::{
    Arg, BROWSER_VALUES, Expr, JsCodegen, Statement, StatementKind, UIElement, bare, sig, top_level,
};

/// The methods that change a list in place.
pub(super) const MUTATORS: &[&str] = &[
    "sort",
    "reverse",
    "splice",
    "fill",
    "unshift",
    "shift",
    "pop",
    "copyWithin",
];

/// One step of a path into a value: a field, or an index.
pub(super) enum Step<'a> {
    Field(&'a str),
    Index(&'a Expr),
}

/// `a.b[i].c` as its root, `a`, and the steps to the place: `b`, `[i]`, `c`.
pub(super) fn path_of(target: &Expr) -> Option<(&Expr, Vec<Step<'_>>)> {
    let mut steps = Vec::new();
    let mut at = target;
    loop {
        match at {
            Expr::PropertyAccess(base, field) => {
                steps.push(Step::Field(field));
                at = base;
            }
            Expr::IndexAccess(base, index) => {
                steps.push(Step::Index(index));
                at = base;
            }
            _ => break,
        }
    }
    steps.reverse();
    Some((at, steps))
}

/// What a `bind:` reads and writes, as JavaScript.
pub(super) struct BindTarget {
    /// An expression that reads the bound value.
    pub(super) read: String,
    /// A statement that writes it, with `$V` where the value goes.
    pub(super) write: String,
    /// For a loop item's field in a loop with no key: what tells the list
    /// it changed, run when the control commits (`change`) rather than on
    /// every keystroke, since an unkeyed list redraws every item.
    pub(super) commit: Option<String>,
}

/// Whether `js` calls a live binding's accessor — `$d()`, `$item()` — that
/// is, `$`, a name, and `()`, the `$` not the end of a longer name.
/// The origin a message from the frame `var` arrives with, as JavaScript:
/// the opaque `"null"` when a sandbox withholds `allow-same-origin`, the
/// page's own for a `srcdoc` document, and otherwise its address's. A
/// sandbox spelled with anything but spaces between its words reads as
/// opaque, which only ever drops a message.
pub(super) fn frame_origin(var: &str) -> String {
    format!(
        "((f) => {{ const s = f.getAttribute(\"sandbox\"); if (s !== null && !s.toLowerCase().split(\" \").includes(\"allow-same-origin\")) return \"null\"; if (f.hasAttribute(\"srcdoc\")) return location.origin; try {{ return new URL(f.getAttribute(\"src\") || \"about:blank\", location.href).origin; }} catch (_) {{ return \"null\"; }} }})({var})"
    )
}

pub(super) fn reads_live_binding(js: &str) -> bool {
    let chars: Vec<char> = js.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '$' && (i == 0 || !(name_char(chars[i - 1]) || chars[i - 1] == '$')) {
            let mut j = i + 1;
            while j < chars.len() && name_char(chars[j]) {
                j += 1;
            }
            if j > i + 1 && chars.get(j) == Some(&'(') && chars.get(j + 1) == Some(&')') {
                return true;
            }
            i = j.max(i + 1);
        } else {
            i += 1;
        }
    }
    false
}

/// Whether `c` continues a name, in any script — as the lexer reads one
/// (`state عدد`, `state été`), so a signal named in Arabic is seen as a
/// signal read.
pub(super) fn name_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || (!c.is_ascii() && unicode_ident::is_xid_continue(c))
}

/// Whether `js` reads through `handle` (`_rows.`), as a name of its own and
/// not the end of a longer one (`max_rows.`).
pub(super) fn reads_handle(js: &str, handle: &str) -> bool {
    js.match_indices(handle).any(|(at, _)| {
        js[..at]
            .chars()
            .next_back()
            .is_none_or(|c| !(c.is_alphanumeric() || c == '_' || c == '$' || c == '.'))
    })
}

pub(super) fn is_reactive_expr(expr_str: &str) -> bool {
    // A live binding's accessor: `$name()` (see `JsCodegen::live_at`).
    if reads_live_binding(expr_str) {
        return true;
    }
    // Check for signal access pattern: _identifier(), the name in any
    // script the lexer takes.
    let chars: Vec<char> = expr_str.chars().collect();
    let letter = |c: char| c != '_' && name_char(c);
    for i in 0..chars.len() {
        if chars[i] == '_' && chars.get(i + 1).is_some_and(|c| letter(*c)) {
            // Found _identifier, check if it's followed by ()
            let mut j = i + 1;
            while j < chars.len() && letter(chars[j]) {
                j += 1;
            }
            // A signal renamed off one of the generated code's own (`_p$`).
            if chars.get(j) == Some(&'$') {
                j += 1;
            }
            if chars.get(j) == Some(&'(') && chars.get(j + 1) == Some(&')') {
                return true;
            }
        }
    }
    // Also check for WF.i18n.t( which is reactive (locale changes)
    if expr_str.contains("WF.i18n.t(")
        || expr_str.contains("WF.i18n.locale()")
        || expr_str.contains("WF.i18n.dir()")
    {
        return true;
    }
    // The browser as values — each a signal the runtime keeps current. Only
    // conditions were drawn live without this, so `Text("{viewport.width}")`,
    // a clock reading `now` or `network.online` in text was computed once
    // and never moved.
    BROWSER_VALUES
        .iter()
        .any(|name| expr_str.contains(&format!("WF.{name}(")))
}

impl JsCodegen {
    /// A path into a value, `["done", (i)]`, for `WF.setIn`.
    pub(super) fn path_js(&self, steps: &[Step<'_>], emit: &dyn Fn(&Expr) -> String) -> String {
        let parts: Vec<String> = steps
            .iter()
            .map(|s| match s {
                Step::Field(f) => format!("\"{f}\""),
                Step::Index(i) => format!("({})", emit(i)),
            })
            .collect();
        format!("[{}]", parts.join(", "))
    }

    /// `form.name = v`, `items[i].done = v`, `Cart.items[0].done = v`,
    /// `t.done = v` for a loop's item: an assignment into a state's value,
    /// compiled as the state given a copy with that place changed. In
    /// place, the state kept the object it already had and nothing that
    /// read it repainted.
    pub(super) fn immutable_update(&self, target: &Expr, value: &str) -> Option<String> {
        let (root, steps) = path_of(target)?;
        if steps.is_empty() {
            return None;
        }
        let Expr::Identifier(name) = root else {
            return None;
        };
        let emit = |e: &Expr| self.emit_expr(e);
        if self.is_state_signal(root) {
            let path = self.path_js(&steps, &emit);
            return Some(format!(
                "{s}.set(WF.setIn({s}(), {path}, {value}));",
                s = sig(name)
            ));
        }
        if self.stores.contains(name)
            && !self.loop_bindings.contains(name)
            && let [Step::Field(member), rest @ ..] = steps.as_slice()
            && !rest.is_empty()
        {
            let path = self.path_js(rest, &emit);
            let name = top_level(name);
            return Some(format!(
                "{name}.{member} = WF.setIn({name}.{member}, {path}, {value});"
            ));
        }
        // A loop's item, changed in a handler: the item replaced in its
        // list, which redraws it.
        let (_, list, _) = self
            .loop_sources
            .iter()
            .rev()
            .find(|(item, _, _)| item == name)?;
        let path = self.path_js(&steps, &emit);
        let item = self.binding_js(name);
        let replace = format!("(__x) => __x === {item} ? WF.setIn(__x, {path}, {value}) : __x");
        match list {
            Expr::Identifier(l) if self.is_state_signal(list) => {
                Some(format!("{s}.set({s}().map({replace}));", s = sig(l)))
            }
            Expr::PropertyAccess(b, m) if matches!(b.as_ref(), Expr::Identifier(s) if self.stores.contains(s)) =>
            {
                let store = self.emit_expr(b);
                Some(format!("{store}.{m} = {store}.{m}.map({replace});"))
            }
            _ => None,
        }
    }

    /// `items.sort(f)`, `items.reverse()`, `items.splice(i, 1)` written as a
    /// statement on a state: run on a copy, and the copy assigned back.
    pub(super) fn mutating_call(&self, expr: &Expr) -> Option<String> {
        let Expr::MethodCall(obj, method, args) = expr else {
            return None;
        };
        if !MUTATORS.contains(&method.as_str()) {
            return None;
        }
        let args: Vec<String> = args.iter().map(|a| self.emit_expr(a)).collect();
        let args = args.join(", ");
        match obj.as_ref() {
            Expr::Identifier(name) if self.is_state_signal(obj) => Some(format!(
                "{s}.set(WF.mutated({s}(), \"{method}\", [{args}]));",
                s = sig(name)
            )),
            Expr::PropertyAccess(base, member) if matches!(base.as_ref(), Expr::Identifier(s) if self.stores.contains(s)) =>
            {
                let store = self.emit_expr(base);
                Some(format!(
                    "{store}.{member} = WF.mutated({store}.{member}, \"{method}\", [{args}]);"
                ))
            }
            _ => None,
        }
    }

    /// The element's `bind:` argument, as a target it can write.
    pub(super) fn bind_arg(&self, ui: &UIElement) -> Option<BindTarget> {
        ui.args.iter().find_map(|a| match a {
            Arg::Named(k, v) if k == "bind" => self.bind_target(v),
            _ => None,
        })
    }

    /// A write followed by its commit, for a control whose every write is a
    /// commit — a checkbox's click, a date chosen.
    pub(super) fn with_commit(target: &BindTarget, write: &str) -> String {
        match &target.commit {
            Some(commit) => format!("{write}; {commit}"),
            None => write.to_string(),
        }
    }

    /// A radio group's `name`, from what its radios bind — none for a loop
    /// item's field, whose rows are each a group of their own.
    pub(super) fn group_name(&self, read: &str) -> Option<String> {
        let owner = read.split('.').next().unwrap_or(read);
        if self.loop_bindings.iter().any(|b| b == owner) {
            return None;
        }
        // A live item is read through its accessor, `$t().choice`.
        if let Some(name) = owner.strip_prefix('$').and_then(|o| o.strip_suffix("()"))
            && self.is_live(name)
        {
            return None;
        }
        Some(
            read.trim_start_matches('_')
                .trim_end_matches("()")
                .chars()
                .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
                .collect(),
        )
    }

    /// What `bind:` names, when it names something that can be written: a
    /// state (`bind: draft`), a store's member (`bind: Cart.note`), or a
    /// field of a loop's item (`bind: t.title`) — which writes the item and
    /// hands the list a new array, so whatever reads the list sees it.
    pub(super) fn bind_target(&self, expr: &Expr) -> Option<BindTarget> {
        match expr {
            Expr::Identifier(name) if self.is_state_signal(expr) => Some(BindTarget {
                read: format!("{}()", sig(name)),
                write: format!("{}.set($V)", sig(name)),
                commit: None,
            }),
            Expr::PropertyAccess(base, field) => {
                let Expr::Identifier(owner) = base.as_ref() else {
                    return None;
                };
                if self.stores.contains(owner) && !self.loop_bindings.contains(owner) {
                    let owner = top_level(owner);
                    return Some(BindTarget {
                        read: format!("{owner}.{field}"),
                        write: format!("{owner}.{field} = $V"),
                        commit: None,
                    });
                }
                let (_, list, keyed) = self
                    .loop_sources
                    .iter()
                    .rev()
                    .find(|(item, _, _)| item == owner)?;
                let touch = match list {
                    Expr::Identifier(l) if self.is_state_signal(list) => {
                        Some(format!("{s}.set({s}().slice())", s = sig(l)))
                    }
                    Expr::PropertyAccess(b, m) if matches!(b.as_ref(), Expr::Identifier(s) if self.stores.contains(s)) =>
                    {
                        let store = self.emit_expr(b);
                        Some(format!("{store}.{m} = {store}.{m}.slice()"))
                    }
                    _ => None,
                };
                let item = self.binding_js(owner);
                let read = format!("{item}.{field}");
                Some(match (touch, keyed) {
                    (Some(t), true) => BindTarget {
                        read,
                        write: format!("{item}.{field} = $V; {t}"),
                        commit: None,
                    },
                    (touch, _) => BindTarget {
                        read,
                        write: format!("{item}.{field} = $V"),
                        commit: touch,
                    },
                })
            }
            _ => None,
        }
    }

    /// Whether a compiled expression reads reactive state.
    ///
    /// [`is_reactive_expr`] recognises page signals (`_x()`) and i18n; a store
    /// field compiles to `Store.field`, a getter over a signal, which that
    /// textual check cannot see. Anything deciding between a one-shot value
    /// and an effect asks here.
    /// A `pending` signal for each async action of the body, declared
    /// before anything reads it.
    pub(super) fn emit_pending_signals(&mut self, body: &[Statement]) {
        for stmt in body {
            if let StatementKind::Action(a) = &stmt.kind
                && crate::syntax::parser::ast::awaits(&a.body)
            {
                self.emit_line(&format!(
                    "const {}_pending = WF.signal(false);",
                    sig(&a.name)
                ));
            }
        }
    }

    /// Whether `expr` is a bare name that resolves to one of the body's own
    /// signals — a `state`, `persist` or `let` — rather than a prop, a loop
    /// binding, a constant or a global.
    pub(super) fn is_state_signal(&self, expr: &Expr) -> bool {
        let Expr::Identifier(name) = expr else {
            return false;
        };
        // A live binding reads the same way, and is not one to write.
        if self.is_live(name) {
            return false;
        }
        let emitted = self.emit_expr(expr);
        emitted == format!("{}()", sig(name))
    }

    /// Bind `name` live for the body about to be emitted: it is read as
    /// `$name()`, the accessor the body is handed. Undone by
    /// [`Self::unbind`] with the length `loop_bindings` had before.
    pub(super) fn bind_live(&mut self, name: &str) {
        self.live_at.push(self.loop_bindings.len());
        self.loop_bindings.push(name.to_string());
    }

    /// Back to the bindings there were at `bound`, live ones included.
    pub(super) fn unbind(&mut self, bound: usize) {
        self.loop_bindings.truncate(bound);
        self.live_at.retain(|&at| at < bound);
    }

    /// Whether `name`, where it is read, is the innermost binding of that
    /// name and a live one. A lambda's parameter of the same name is nearer.
    pub(super) fn is_live(&self, name: &str) -> bool {
        !self.lambda_params.borrow().iter().any(|p| p == name)
            && self
                .loop_bindings
                .iter()
                .rposition(|b| b == name)
                .is_some_and(|at| self.live_at.contains(&at))
    }

    /// A binding as the code that reads it: `$t()` for a live one, `t`
    /// for a plain one.
    pub(super) fn binding_js(&self, name: &str) -> String {
        if self.is_live(name) {
            format!("${name}()")
        } else {
            bare(name)
        }
    }

    /// Whether `expr` reaches a store's state: `Cart.items` from a page, or
    /// bare `items` inside the store's own body, emitted as `store.items`.
    pub(super) fn is_store_member(&self, expr: &Expr, emitted: &str) -> bool {
        match expr {
            Expr::PropertyAccess(base, _) => {
                matches!(base.as_ref(), Expr::Identifier(s) if self.stores.contains(s))
            }
            Expr::Identifier(name) => emitted == format!("store.{name}"),
            _ => false,
        }
    }

    /// Whether `expr` names an action: one of the page's own, or a store's.
    pub(super) fn is_action_ref(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Identifier(name) => self.body.own_actions.contains(name),
            Expr::PropertyAccess(base, _) => {
                matches!(base.as_ref(), Expr::Identifier(store) if self.stores.contains(store))
            }
            _ => false,
        }
    }

    /// An emitted value as `WF.el` takes it: a thunk when it reads state, so
    /// the text or attribute follows it; as it is otherwise.
    pub(super) fn live(&self, emitted: &str) -> String {
        if self.is_reactive(emitted) {
            format!("() => {emitted}")
        } else {
            emitted.to_string()
        }
    }

    pub(super) fn is_reactive(&self, expr_str: &str) -> bool {
        is_reactive_expr(expr_str)
            // What a resource or a connection holds is a signal on its
            // handle — `_rows.data()`, `_chat.messages()` — which the
            // `_name()` pattern does not see. A label that read only a
            // resource was worked out once, while it was loading.
            || self
                .body.resources
                .iter()
                .any(|r| reads_handle(expr_str, &format!("{}.", sig(r))))
            || expr_str.contains("_p.")
            || self
                .body.form_handles
                .iter()
                .any(|f| expr_str.contains(&format!("{f}.")))
            // A store read in the generated code by the name it has there:
            // one called like a browser global is `History$`, whose props
            // were copied once instead of followed.
            || self
                .stores
                .iter()
                .any(|s| expr_str.contains(&format!("{}.", top_level(s))))
    }
}
