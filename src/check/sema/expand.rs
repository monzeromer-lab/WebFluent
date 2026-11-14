//! The built-ins that are drawn as other built-ins: a `Callout` is its
//! title row (an icon and the words) and its text; a `Stat` its label, its
//! figure and how it moved; a `CopyButton` a button a runtime helper copies
//! with. Each is rewritten here, before anything draws it, so the live page,
//! the static paint, the template engine and the PDF all draw it alike from
//! one definition — the parts the registry gives a tag and a class
//! (`Callout.Title` is a `<p class="wf-callout__title">`), and the built-ins
//! that already exist.
//!
//! A rewrite is written as WebFluent, with the caller's expressions printed
//! into it, and parsed; the caller's own block and handlers are moved in as
//! they are. What a runtime helper does (copying, the theme, the headings
//! of the page) is a `mount:` that calls it.

use crate::syntax::parser::ast::{
    Arg, ComponentRef, Declaration, EventHandler, Expr, Span, Statement, StatementKind, UIElement,
};

/// Every built-in under `stmts` that is drawn as others, rewritten.
pub(super) fn expand(stmts: &mut [Statement], seq: &mut usize) {
    for stmt in stmts.iter_mut() {
        if let StatementKind::UIElement(el) = &mut stmt.kind {
            expand_element(el, seq);
        }
        for body in stmt.kind.bodies_mut() {
            expand(body, seq);
        }
    }
}

fn expand_element(el: &mut UIElement, seq: &mut usize) {
    // Inside first: what a block holds is rewritten before its owner moves
    // it.
    expand(&mut el.children, seq);
    for fill in &mut el.slot_fills {
        expand(&mut fill.body, seq);
    }
    for handler in &mut el.events {
        expand(&mut handler.body, seq);
    }
    let ComponentRef::BuiltIn(name) = &el.component else {
        return;
    };
    match name.as_str() {
        "Callout" => callout(el),
        "Kbd" => kbd(el),
        "EmptyState" => empty_state(el),
        "Stat" => stat(el),
        "CodeBlock" => code_block(el),
        "Chip" => chip(el),
        "CopyButton" => copy_button(el),
        "ThemeToggle" => theme_toggle(el),
        "LocaleSwitch" => locale_switch(el),
        "Accordion" => accordion(el, seq),
        "Field" => field(el),
        "Fieldset" => fieldset(el),
        "InputGroup" => input_group(el),
        "OnThisPage" => on_this_page(el),
        _ => {}
    }
}

// ─── Reading and writing a call ─────────────────────────────────────────────

/// `src` as statements: the body of a page that holds it, placed nowhere —
/// no span, so nothing reports a place in the source for what the compiler
/// wrote, and the studio's numbering passes it over.
fn parse(src: &str) -> Vec<Statement> {
    let wrapped = format!("page WfExpanded(path: \"/\") {{\n{src}\n}}\n");
    let mut body = match crate::syntax::parse_source(&wrapped, "<built-in>") {
        Ok(program) => program
            .declarations
            .into_iter()
            .find_map(|d| match d {
                Declaration::Page(p) => Some(p.body),
                _ => None,
            })
            .unwrap_or_default(),
        Err(_) => Vec::new(),
    };
    unplace(&mut body);
    body
}

fn unplace(stmts: &mut [Statement]) {
    for stmt in stmts.iter_mut() {
        stmt.span = Span::default();
        if let StatementKind::UIElement(el) = &mut stmt.kind {
            el.span = Span::default();
            el.paren_span = None;
            el.body_span = None;
            el.style_span = None;
            el.arg_spans = vec![Span::default(); el.args.len()];
            el.modifier_spans = vec![Span::default(); el.modifiers.len()];
            unplace(&mut el.children);
        }
        for body in stmt.kind.bodies_mut() {
            unplace(body);
        }
    }
}

/// The expression `e`, written back as source.
fn src(e: &Expr) -> String {
    crate::syntax::print::expr(e)
}

/// A string literal, as source.
fn lit(text: &str) -> String {
    crate::syntax::print::string(text)
}

/// The positional argument, taken out of the call.
fn take_positional(el: &mut UIElement) -> Option<Expr> {
    let at = el
        .args
        .iter()
        .position(|a| matches!(a, Arg::Positional(_)))?;
    if at < el.arg_spans.len() {
        el.arg_spans.remove(at);
    }
    match el.args.remove(at) {
        Arg::Positional(e) => Some(e),
        Arg::Named(..) => None,
    }
}

/// The named argument `key`, taken out of the call.
fn take(el: &mut UIElement, key: &str) -> Option<Expr> {
    let at = el
        .args
        .iter()
        .position(|a| matches!(a, Arg::Named(k, _) if k == key))?;
    if at < el.arg_spans.len() {
        el.arg_spans.remove(at);
    }
    match el.args.remove(at) {
        Arg::Named(_, e) => Some(e),
        Arg::Positional(_) => None,
    }
}

/// A `Bool` prop given as a flag (`.copy`) or by name (`copy: true`), taken
/// out of the call: the condition, or `None` where it is not given.
fn take_flag(el: &mut UIElement, key: &str) -> Option<Expr> {
    if let Some(at) = el.modifiers.iter().position(|m| m == key) {
        el.modifiers.remove(at);
        if at < el.modifier_spans.len() {
            el.modifier_spans.remove(at);
        }
        return Some(Expr::BoolLiteral(true));
    }
    take(el, key)
}

/// One of an enum prop's `cases`, given as a flag or by name; the flag
/// stays (it is the class the stylesheet draws by).
fn case(el: &UIElement, key: &str, cases: &[&'static str]) -> Option<&'static str> {
    el.case_of(key, cases)
}

/// Whether the call already says `key`.
fn gives(el: &UIElement, key: &str) -> bool {
    el.args
        .iter()
        .any(|a| matches!(a, Arg::Named(k, _) if k == key))
}

/// `key: value` on the call, unless it already says `key`.
fn give(el: &mut UIElement, key: &str, value: &str) {
    if gives(el, key) {
        return;
    }
    if let Some(e) = parse_expr(value) {
        el.args.push(Arg::Named(key.to_string(), e));
        el.arg_spans.push(Span::default());
    }
}

/// `value` as an expression.
fn parse_expr(value: &str) -> Option<Expr> {
    let wrapped = format!("const WfExpandedValue = {value}\n");
    let program = crate::syntax::parse_source(&wrapped, "<built-in>").ok()?;
    program.declarations.into_iter().find_map(|d| match d {
        Declaration::Const(c) => Some(c.value),
        _ => None,
    })
}

/// The handlers of `event` on the call, taken out of it.
fn take_handlers(el: &mut UIElement, event: &str) -> Vec<EventHandler> {
    let (taken, kept) = std::mem::take(&mut el.events)
        .into_iter()
        .partition(|h| h.event == event);
    el.events = kept;
    taken
}

/// The first element of `stmts`, for a snippet of one.
fn element_of(stmts: &mut [Statement]) -> Option<&mut UIElement> {
    stmts.iter_mut().find_map(|s| match &mut s.kind {
        StatementKind::UIElement(e) => Some(e),
        _ => None,
    })
}

// ─── Each built-in ──────────────────────────────────────────────────────────

/// `Callout("Run wf check first.", title: "Tip").tip`: a note, its title
/// row an icon and the kind's word (or `title:`), then its text and block.
fn callout(el: &mut UIElement) {
    let kind = case(
        el,
        "kind",
        &["note", "tip", "important", "warning", "caution"],
    )
    .unwrap_or("note");
    let (icon, word) = match kind {
        "tip" => ("sparkle", "Tip"),
        "important" => ("star", "Important"),
        "warning" => ("warning", "Warning"),
        "caution" => ("alert-circle", "Caution"),
        _ => ("info", "Note"),
    };
    let title = take(el, "title").map_or_else(|| lit(word), |e| src(&e));
    let mut head = format!("Callout.Title({title}) {{ Icon({}) }}\n", lit(icon));
    if let Some(text) = take_positional(el) {
        head.push_str(&format!("Text({})\n", src(&text)));
    }
    let mut children = parse(&head);
    children.append(&mut el.children);
    el.children = children;
    give(el, "role", "\"note\"");
    // The stylesheet colours it by its kind.
    give(el, "data-kind", &lit(kind));
}

/// `Kbd("mod+k")`: the combination as this system writes it.
fn kbd(el: &mut UIElement) {
    if let Some(at) = el.args.iter().position(|a| matches!(a, Arg::Positional(_)))
        && let Arg::Positional(keys) = &el.args[at]
    {
        let label = Expr::FunctionCall("keyLabel".to_string(), vec![keys.clone()]);
        el.args[at] = Arg::Positional(label);
    }
}

/// `EmptyState(icon:, title:, text:) { actions }`.
fn empty_state(el: &mut UIElement) {
    let mut out = String::new();
    if let Some(icon) = take(el, "icon") {
        out.push_str(&format!("Icon({}, aria-hidden: \"true\").lg\n", src(&icon)));
    }
    if let Some(title) = take(el, "title") {
        out.push_str(&format!("EmptyState.Title({})\n", src(&title)));
    }
    if let Some(text) = take(el, "text") {
        out.push_str(&format!("EmptyState.Text({})\n", src(&text)));
    }
    let actions = std::mem::take(&mut el.children);
    let mut children = parse(&out);
    if !actions.is_empty() {
        let mut wrap = parse("EmptyState.Actions { }");
        if let Some(a) = element_of(&mut wrap) {
            a.children = actions;
        }
        children.append(&mut wrap);
    }
    el.children = children;
}

/// `Stat("Revenue", value: total, format: .currency, change: 0.124)`.
fn stat(el: &mut UIElement) {
    let label = take_positional(el).map(|e| src(&e)).unwrap_or_default();
    let style = case(
        el,
        "format",
        &["number", "integer", "currency", "percent", "compact"],
    );
    take(el, "format");
    let count = take(el, "count").map(|e| format!(", count: {}", src(&e)));
    let mut out = format!("Stat.Label({label})\n");
    if let Some(value) = take(el, "value") {
        let shown = match style {
            Some(s) => format!("format({}, .{s})", src(&value)),
            None => src(&value),
        };
        out.push_str(&format!(
            "Stat.Value({shown}{})\n",
            count.unwrap_or_default()
        ));
    }
    if let Some(change) = take(el, "change") {
        let c = src(&change);
        out.push_str(&format!(
            "Stat.Change(format(if ({c}) < 0 {{ -({c}) }} else {{ {c} }}, .percent, 1), data-trend: if ({c}) < 0 {{ \"down\" }} else {{ \"up\" }}) {{ VisuallyHidden(if ({c}) < 0 {{ \"Down \" }} else {{ \"Up \" }}) }}\n"
        ));
    }
    let mut children = parse(&out);
    children.append(&mut el.children);
    el.children = children;
}

/// `CodeBlock(code, language: "wf", title: "App.wf").copy.lines`.
fn code_block(el: &mut UIElement) {
    let Some(code) = take_positional(el) else {
        return;
    };
    let code = src(&code);
    let language = take(el, "language").map(|e| format!(", language: {}", src(&e)));
    let title = take(el, "title");
    let copy = take_flag(el, "copy");
    let lines = take_flag(el, "lines");
    let mut out = String::new();
    if title.is_some() || copy.is_some() {
        let words = title.map(|t| format!("({})", src(&t))).unwrap_or_default();
        let button = if copy.is_some() {
            format!("CopyButton(value: {code})")
        } else {
            String::new()
        };
        out.push_str(&format!("CodeBlock.Bar{words} {{ {button} }}\n"));
    }
    if lines.is_some() {
        out.push_str(&format!(
            "CodeBlock.Lines({code}.lines().map((line, i) => \"{{i + 1}}\").join(\"\\n\"), aria-hidden: \"true\")\n"
        ));
    }
    out.push_str(&format!(
        "Code({code}{}, tabindex: \"0\").block\n",
        language.unwrap_or_default()
    ));
    // Its copy button is one of these built-ins too.
    let mut children = parse(&out);
    let mut seq = 0;
    expand(&mut children, &mut seq);
    el.children = children;
}

/// `Chip("Rust", icon: "code", selected: on).removable { on click { … }  on remove { … } }`.
fn chip(el: &mut UIElement) {
    let label = take_positional(el).map_or_else(|| lit(""), |e| src(&e));
    let icon = take(el, "icon")
        .map(|i| format!("Icon({}, aria-hidden: \"true\")", src(&i)))
        .unwrap_or_default();
    let selected = take(el, "selected");
    let removable = take_flag(el, "removable");
    let clicks = take_handlers(el, "click");
    let removes = take_handlers(el, "remove");
    let interactive = selected.is_some() || !clicks.is_empty();
    let mut out = if interactive {
        let pressed = selected
            .map(|s| {
                format!(
                    ", aria-pressed: if {} {{ \"true\" }} else {{ \"false\" }}",
                    src(&s)
                )
            })
            .unwrap_or_default();
        format!("Chip.Action({label}, type: \"button\"{pressed}) {{ {icon} }}\n")
    } else {
        format!("Chip.Label({label}) {{ {icon} }}\n")
    };
    if removable.is_some() {
        out.push_str(&format!(
            "Chip.Remove(type: \"button\", aria-label: \"Remove \" + {label}) {{ Icon(\"close\", aria-hidden: \"true\") }}\n"
        ));
    }
    let mut children = parse(&out);
    let mut parts = children.iter_mut().filter_map(|s| match &mut s.kind {
        StatementKind::UIElement(e) => Some(e),
        _ => None,
    });
    if let Some(first) = parts.next() {
        first.events.extend(clicks);
    }
    if let Some(remove) = parts.next() {
        for mut h in removes {
            h.event = "click".to_string();
            remove.events.push(h);
        }
    }
    el.children = children;
}

/// `CopyButton(value: url)`: a button the runtime copies with.
fn copy_button(el: &mut UIElement) {
    let value = take(el, "value").map_or_else(|| lit(""), |e| src(&e));
    let copied = take(el, "copied").map_or_else(|| lit("Copied"), |e| src(&e));
    let label = take_positional(el).map_or_else(|| lit("Copy"), |e| src(&e));
    if let Some(e) = parse_expr(&label) {
        el.args.insert(0, Arg::Positional(e));
        el.arg_spans.insert(0, Span::default());
    }
    give(el, "type", "\"button\"");
    give(
        el,
        "mount",
        &format!("(n) => __wf.copyButton(n, () => {value}, {label}, {copied})"),
    );
}

/// `ThemeToggle`: a moon in a light page, a sun in a dark one.
fn theme_toggle(el: &mut UIElement) {
    give(el, "type", "\"button\"");
    give(el, "aria-label", "\"Switch to the dark theme\"");
    give(el, "mount", "(n) => __wf.themeToggle(n)");
    el.children = parse(
        "Icon(\"moon\", data-shows: \"light\", aria-hidden: \"true\")\nIcon(\"sun\", data-shows: \"dark\", aria-hidden: \"true\")",
    );
}

/// `LocaleSwitch`: a button per locale, which the runtime draws.
fn locale_switch(el: &mut UIElement) {
    let label = take(el, "label").map_or_else(|| lit("Language"), |e| src(&e));
    give(el, "role", "\"group\"");
    give(el, "aria-label", &label);
    give(el, "mount", "(n) => __wf.localeSwitch(n)");
}

/// `Accordion.single { Accordion.Item("Shipping") { … } }`: each item a
/// `<details>`, its title the `<summary>`; one name for them all when one
/// is to be open at a time.
fn accordion(el: &mut UIElement, seq: &mut usize) {
    let single = take_flag(el, "single").is_some();
    let name = if single {
        *seq += 1;
        Some(format!("wf-accordion-{seq}"))
    } else {
        None
    };
    items(&mut el.children, name.as_deref());
}

fn items(stmts: &mut [Statement], name: Option<&str>) {
    for stmt in stmts.iter_mut() {
        if let StatementKind::UIElement(item) = &mut stmt.kind
            && matches!(&item.component, ComponentRef::SubComponent(p, s) if p == "Accordion" && s == "Item")
            && item.args.iter().any(|a| matches!(a, Arg::Positional(_)))
        {
            let title = take_positional(item).map(|e| src(&e)).unwrap_or_default();
            if let Some(n) = name {
                give(item, "name", &lit(n));
            }
            let body = std::mem::take(&mut item.children);
            let mut parts = parse(&format!(
                "Accordion.Summary({title})\nAccordion.Panel {{ }}"
            ));
            if let Some(StatementKind::UIElement(panel)) = parts.get_mut(1).map(|s| &mut s.kind) {
                panel.children = body;
            }
            item.children = parts;
            continue;
        }
        // An item a `for` or an `if` draws.
        if !matches!(stmt.kind, StatementKind::UIElement(_)) {
            for body in stmt.kind.bodies_mut() {
                items(body, name);
            }
        }
    }
}

/// `Field(label:, hint:, error:) { control }`.
fn field(el: &mut UIElement) {
    let label = take(el, "label");
    let hint = take(el, "hint");
    let error = take(el, "error");
    let mut before = String::new();
    if let Some(l) = &label {
        before.push_str(&format!("Field.Label({})\n", src(l)));
    }
    let mut after = String::new();
    if let Some(h) = &hint {
        after.push_str(&format!("Field.Hint({})\n", src(h)));
    }
    let read = match &error {
        Some(e) => {
            let e = src(e);
            after.push_str(&format!(
                "Field.Error({e} ?? \"\", hidden: ({e} ?? \"\") == \"\")\n"
            ));
            format!("() => {e}")
        }
        None => "null".to_string(),
    };
    let mut children = parse(&before);
    children.append(&mut el.children);
    children.append(&mut parse(&after));
    el.children = children;
    give(el, "mount", &format!("(n) => __wf.fieldFor(n, {read})"));
}

/// `Fieldset(legend: "Address") { … }`.
fn fieldset(el: &mut UIElement) {
    let Some(legend) = take(el, "legend") else {
        return;
    };
    let mut children = parse(&format!("Fieldset.Legend({})", src(&legend)));
    children.append(&mut el.children);
    el.children = children;
}

/// `InputGroup { Icon("search")  Input(bind: q)  InputGroup.Clear }`.
fn input_group(el: &mut UIElement) {
    for stmt in &mut el.children {
        if let StatementKind::UIElement(part) = &mut stmt.kind
            && matches!(&part.component, ComponentRef::SubComponent(p, s) if p == "InputGroup" && s == "Clear")
        {
            give(part, "type", "\"button\"");
            give(part, "aria-label", "\"Clear\"");
            give(part, "mount", "(n) => __wf.inputClear(n)");
            if part.children.is_empty() {
                part.children = parse("Icon(\"close\", aria-hidden: \"true\")");
            }
        }
    }
}

/// `OnThisPage(levels: 2)`: its title, and the links the runtime draws.
fn on_this_page(el: &mut UIElement) {
    let label = take(el, "label").map_or_else(|| lit("On this page"), |e| src(&e));
    let levels = take(el, "levels").map_or_else(|| "1".to_string(), |e| src(&e));
    give(el, "aria-label", &label);
    give(el, "mount", &format!("(n) => __wf.onThisPage(n, {levels})"));
    el.children = parse(&format!("OnThisPage.Title({label}, aria-hidden: \"true\")"));
}
