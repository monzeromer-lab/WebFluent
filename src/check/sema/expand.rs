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
        "Toggle" => toggle(el),
        "ToggleGroup" => toggle_group(el),
        "RadioGroup" => radio_group(el),
        "Pagination" => pagination(el),
        "Stepper" => stepper(el),
        "NumberInput" => number_input(el),
        "Listbox" => listbox(el),
        "CommandPalette" => command_palette(el),
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

/// The handlers `src` writes (`on click { … }`), placed nowhere.
fn handlers(src: &str) -> Vec<EventHandler> {
    let mut stmts = parse(&format!("Text(\"\") {{ {src} }}"));
    element_of(&mut stmts)
        .map(|e| std::mem::take(&mut e.events))
        .unwrap_or_default()
}

/// `on click { … }` run before the call's own.
fn first_handlers(el: &mut UIElement, src: &str) {
    let mut own = std::mem::take(&mut el.events);
    el.events = handlers(src);
    el.events.append(&mut own);
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

/// `Toggle("Bold", bind: bold, icon: "type")`: pressed while `bold` holds.
fn toggle(el: &mut UIElement) {
    let Some(bind) = take(el, "bind") else {
        return;
    };
    let b = src(&bind);
    if let Some(icon) = take(el, "icon") {
        el.children = parse(&format!("Icon({}, aria-hidden: \"true\")", src(&icon)));
    }
    give(el, "type", "\"button\"");
    give(
        el,
        "aria-pressed",
        &format!("if {b} {{ \"true\" }} else {{ \"false\" }}"),
    );
    first_handlers(el, &format!("on click {{ {b} = !{b} }}"));
}

/// `ToggleGroup(bind: view, label: "View") { ToggleGroup.Item("Grid", value: "grid") }`.
fn toggle_group(el: &mut UIElement) {
    let Some(bind) = take(el, "bind") else {
        return;
    };
    let b = src(&bind);
    let multiple = take_flag(el, "multiple").is_some();
    if let Some(label) = take(el, "label") {
        give(el, "aria-label", &src(&label));
    }
    give(el, "role", "\"group\"");
    give(el, "roving", ".x");
    toggle_items(&mut el.children, &b, multiple);
}

fn toggle_items(stmts: &mut [Statement], b: &str, multiple: bool) {
    for stmt in stmts.iter_mut() {
        if let StatementKind::UIElement(item) = &mut stmt.kind
            && matches!(&item.component, ComponentRef::SubComponent(p, s) if p == "ToggleGroup" && s == "Item")
        {
            let Some(value) = take(item, "value") else {
                continue;
            };
            let v = src(&value);
            if let Some(icon) = take(item, "icon") {
                item.children = parse(&format!("Icon({}, aria-hidden: \"true\")", src(&icon)));
            }
            give(item, "type", "\"button\"");
            let (pressed, press) = if multiple {
                (
                    format!("{b}.includes({v})"),
                    format!(
                        "{b} = if {b}.includes({v}) {{ {b}.filter(wfChosen => wfChosen != {v}) }} else {{ {b}.concat([{v}]) }}"
                    ),
                )
            } else {
                (format!("{b} == {v}"), format!("{b} = {v}"))
            };
            give(
                item,
                "aria-pressed",
                &format!("if {pressed} {{ \"true\" }} else {{ \"false\" }}"),
            );
            first_handlers(item, &format!("on click {{ {press} }}"));
            continue;
        }
        if !matches!(stmt.kind, StatementKind::UIElement(_)) {
            for body in stmt.kind.bodies_mut() {
                toggle_items(body, b, multiple);
            }
        }
    }
}

/// `RadioGroup(bind: plan, label: "Plan").cards { RadioGroup.Option("Free", value: "free", hint: "…") }`:
/// native radios under a legend, each option's hint read with it.
fn radio_group(el: &mut UIElement) {
    let Some(bind) = take(el, "bind") else {
        return;
    };
    let b = src(&bind);
    if take_flag(el, "cards").is_some() {
        give(el, "data-look", "\"cards\"");
    }
    let legend = take(el, "label")
        .map(|l| format!("RadioGroup.Legend({})", src(&l)))
        .unwrap_or_default();
    radio_options(&mut el.children, &b);
    let mut children = parse(&legend);
    children.append(&mut el.children);
    el.children = children;
}

fn radio_options(stmts: &mut [Statement], b: &str) {
    for stmt in stmts.iter_mut() {
        if let StatementKind::UIElement(option) = &mut stmt.kind
            && matches!(&option.component, ComponentRef::SubComponent(p, s) if p == "RadioGroup" && s == "Option")
        {
            let Some(label) = take_positional(option) else {
                continue;
            };
            let value = take(option, "value").map_or_else(|| src(&label), |v| src(&v));
            let hint = take(option, "hint");
            let described = hint
                .as_ref()
                .map(|h| format!(", aria-description: {}", src(h)))
                .unwrap_or_default();
            let mut out = format!(
                "Radio(bind: {b}, value: {value}, label: {}{described})\n",
                src(&label)
            );
            if let Some(h) = &hint {
                out.push_str(&format!(
                    "RadioGroup.Hint({}, aria-hidden: \"true\")\n",
                    src(h)
                ));
            }
            let mut children = parse(&out);
            children.append(&mut option.children);
            option.children = children;
            continue;
        }
        if !matches!(stmt.kind, StatementKind::UIElement(_)) {
            for body in stmt.kind.bodies_mut() {
                radio_options(body, b);
            }
        }
    }
}

/// `Pagination(bind: page, pages: 12, siblings: 1)`: Previous, the first and
/// last pages and those around the current one, a gap for the rest, Next.
fn pagination(el: &mut UIElement) {
    let (Some(bind), Some(pages)) = (take(el, "bind"), take(el, "pages")) else {
        return;
    };
    let b = src(&bind);
    let p = src(&pages);
    let s = take(el, "siblings").map_or_else(|| "1".to_string(), |e| src(&e));
    let label = take(el, "label").map_or_else(|| lit("Pagination"), |e| src(&e));
    give(el, "aria-label", &label);
    let shown = format!(
        "(1..=({p})).filter(wfN => wfN == 1 || wfN == ({p}) || (wfN >= ({b}) - ({s}) && wfN <= ({b}) + ({s})))"
    );
    el.children = parse(&format!(
        r#"Pagination.Prev(type: "button", aria-label: "Previous page", disabled: ({b}) <= 1) {{
    on click {{ {b} = ({b}) - 1 }}
    Icon("chevron-left", aria-hidden: "true")
}}
for wfPage, wfAt in {shown} by wfPage {{
    if wfAt > 0 && {shown}[wfAt - 1] != wfPage - 1 {{ Pagination.Gap("…", aria-hidden: "true") }}
    Pagination.Page("{{wfPage}}", type: "button", aria-label: "Page {{wfPage}}", aria-current: if wfPage == ({b}) {{ "page" }} else {{ null }}) {{
        on click {{ {b} = wfPage }}
    }}
}}
Pagination.Next(type: "button", aria-label: "Next page", disabled: ({b}) >= ({p})) {{
    on click {{ {b} = ({b}) + 1 }}
    Icon("chevron-right", aria-hidden: "true")
}}"#
    ));
}

/// `Stepper(bind: step) { Stepper.Step("Account") { … }  Stepper.Step("Billing", done: billed) { … } }`:
/// the steps listed with their states, and each step the panel of its
/// block, hidden while another is the current one. The steps stay where
/// the author wrote them, so each keeps its place (and its id).
fn stepper(el: &mut UIElement) {
    let Some(bind) = take(el, "bind") else {
        return;
    };
    let b = src(&bind);
    let linear = take_flag(el, "linear").is_some();
    let mut list = String::new();
    let mut i = 0;
    for stmt in &mut el.children {
        let StatementKind::UIElement(step) = &mut stmt.kind else {
            continue;
        };
        if !matches!(&step.component, ComponentRef::SubComponent(p, s) if p == "Stepper" && s == "Step")
        {
            continue;
        }
        let title = take_positional(step).map_or_else(|| lit(""), |t| src(&t));
        let done = take(step, "done").map_or_else(|| "false".to_string(), |e| src(&e));
        let error = take(step, "error").map_or_else(|| "false".to_string(), |e| src(&e));
        let disabled = if linear {
            format!(", disabled: {i} > ({b})")
        } else {
            String::new()
        };
        list.push_str(&format!(
            "Stepper.Item {{ Button({title}, data-state: if {error} {{ \"error\" }} else if {b} == {i} {{ \"current\" }} else if {done} {{ \"done\" }} else {{ \"upcoming\" }}, aria-current: if {b} == {i} {{ \"step\" }} else {{ null }}{disabled}) {{ on click {{ {b} = {i} }} }} }}\n"
        ));
        give(step, "hidden", &format!("({b}) != {i}"));
        give(step, "aria-label", &title);
        i += 1;
    }
    let mut children = parse(&format!("Stepper.List {{ {list} }}"));
    children.append(&mut el.children);
    el.children = children;
}

/// `NumberInput(bind: qty, min: 1, max: 99, label: "Quantity")`: the input
/// between − and +, stepped by the arrow keys and, ten at a time, `PageUp` and
/// `PageDown`, and held between its ends.
fn number_input(el: &mut UIElement) {
    let Some(bind) = take(el, "bind") else {
        return;
    };
    let b = src(&bind);
    let min = take(el, "min").map(|e| src(&e));
    let max = take(el, "max").map(|e| src(&e));
    let step = take(el, "step").map_or_else(|| "1".to_string(), |e| src(&e));
    let label = take(el, "label").map(|e| src(&e));
    let hint = take(el, "hint").map(|e| src(&e));
    // Its name where the field's label cannot reach it yet (the static
    // paint); the label names it once the page runs.
    let named = format!(
        ", aria-label: {}",
        label.clone().unwrap_or_else(|| lit("Number"))
    );
    let lo = min.clone().unwrap_or_else(|| "-Infinity".to_string());
    let hi = max.clone().unwrap_or_else(|| "Infinity".to_string());
    let clamp = |v: &str| format!("Math.min({hi}, Math.max({lo}, {v}))");
    let attrs = [
        min.map(|m| format!(", min: {m}")).unwrap_or_default(),
        max.map(|m| format!(", max: {m}")).unwrap_or_default(),
        format!(", step: {step}"),
    ]
    .concat();
    let field = [
        label.map(|l| format!("label: {l}")),
        hint.map(|h| format!("hint: {h}")),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join(", ");
    let mut children = parse(&format!(
        r#"Field({field}) {{
    InputGroup {{
        Input(bind: {b}{attrs}{named}).number {{
            on keydown(wfKey) {{
                if wfKey.key == "PageUp" {{ wfKey.preventDefault()  {b} = {up10} }}
                if wfKey.key == "PageDown" {{ wfKey.preventDefault()  {b} = {down10} }}
            }}
        }}
        NumberInput.Decrease(type: "button", tabindex: "-1", aria-label: "Decrease", disabled: ({b}) <= {lo}) {{
            on click {{ {b} = {down} }}
            Icon("minus", aria-hidden: "true")
        }}
        NumberInput.Increase(type: "button", tabindex: "-1", aria-label: "Increase", disabled: ({b}) >= {hi}) {{
            on click {{ {b} = {up} }}
            Icon("plus", aria-hidden: "true")
        }}
    }}
}}"#,
        up10 = clamp(&format!("({b}) + ({step}) * 10")),
        down10 = clamp(&format!("({b}) - ({step}) * 10")),
        up = clamp(&format!("({b}) + ({step})")),
        down = clamp(&format!("({b}) - ({step})")),
    ));
    let mut seq = 0;
    expand(&mut children, &mut seq);
    el.children = children;
}

/// `Listbox(bind: chosen, label: "Fruit") { Listbox.Option("Apple", value: "a") }`:
/// options a press chooses, the keyboard the runtime's.
fn listbox(el: &mut UIElement) {
    let Some(bind) = take(el, "bind") else {
        return;
    };
    let b = src(&bind);
    let multiple = take_flag(el, "multiple").is_some();
    if let Some(label) = take(el, "label") {
        give(el, "aria-label", &src(&label));
    }
    give(el, "role", "\"listbox\"");
    give(el, "mount", &format!("(n) => __wf.listbox(n, {multiple})"));
    listbox_options(&mut el.children, &b, multiple);
}

fn listbox_options(stmts: &mut [Statement], b: &str, multiple: bool) {
    for stmt in stmts.iter_mut() {
        if let StatementKind::UIElement(option) = &mut stmt.kind
            && matches!(&option.component, ComponentRef::SubComponent(p, s) if p == "Listbox" && s == "Option")
        {
            let Some(value) = take(option, "value") else {
                continue;
            };
            let v = src(&value);
            if let Some(icon) = take(option, "icon") {
                let mut icon = parse(&format!("Icon({}, aria-hidden: \"true\")", src(&icon)));
                icon.append(&mut option.children);
                option.children = icon;
            }
            give(option, "role", "\"option\"");
            let (chosen, choose) = if multiple {
                (
                    format!("{b}.includes({v})"),
                    format!(
                        "{b} = if {b}.includes({v}) {{ {b}.filter(wfChosen => wfChosen != {v}) }} else {{ {b}.concat([{v}]) }}"
                    ),
                )
            } else {
                (format!("{b} == {v}"), format!("{b} = {v}"))
            };
            give(
                option,
                "aria-selected",
                &format!("if {chosen} {{ \"true\" }} else {{ \"false\" }}"),
            );
            // A disabled option's press is stopped by the runtime first.
            first_handlers(option, &format!("on click {{ {choose} }}"));
            continue;
        }
        if !matches!(stmt.kind, StatementKind::UIElement(_)) {
            for body in stmt.kind.bodies_mut() {
                listbox_options(body, b, multiple);
            }
        }
    }
}

/// `CommandPalette(open: palette) { CommandPalette.Group("Pages") { CommandPalette.Item("Home") { on select { … } } } }`:
/// a modal with a field and what matches it.
fn command_palette(el: &mut UIElement) {
    let Some(open) = take(el, "open") else {
        return;
    };
    let label = take(el, "label").map_or_else(|| lit("Commands"), |e| src(&e));
    let placeholder =
        take(el, "placeholder").map_or_else(|| lit("Type a command or search…"), |e| src(&e));
    palette_parts(&mut el.children);
    let body = std::mem::take(&mut el.children);
    let mut modal = parse(&format!(
        "Modal(visible: {o}, title: {label}) {{\n    Input(aria-label: {label}, placeholder: {placeholder}).search\n    CommandPalette.List(role: \"listbox\", aria-label: {label}) {{ }}\n}}",
        o = src(&open)
    ));
    if let Some(StatementKind::UIElement(m)) = modal.first_mut().map(|s| &mut s.kind)
        && let Some(StatementKind::UIElement(list)) = m.children.get_mut(1).map(|s| &mut s.kind)
    {
        list.children = body;
    }
    el.children = modal;
    give(el, "mount", "(n) => __wf.commandPalette(n)");
}

fn palette_parts(stmts: &mut [Statement]) {
    for stmt in stmts.iter_mut() {
        if let StatementKind::UIElement(part) = &mut stmt.kind {
            match &part.component {
                ComponentRef::SubComponent(p, s) if p == "CommandPalette" && s == "Group" => {
                    if let Some(title) = take_positional(part) {
                        let t = src(&title);
                        give(part, "role", "\"group\"");
                        give(part, "aria-label", &t);
                        let mut head = parse(&format!("Text({t}, aria-hidden: \"true\").sm.muted"));
                        palette_parts(&mut part.children);
                        head.append(&mut part.children);
                        part.children = head;
                    }
                    continue;
                }
                ComponentRef::SubComponent(p, s) if p == "CommandPalette" && s == "Item" => {
                    let mut out = String::new();
                    if let Some(icon) = take(part, "icon") {
                        out.push_str(&format!("Icon({}, aria-hidden: \"true\")\n", src(&icon)));
                    }
                    if let Some(keys) = take(part, "shortcut") {
                        out.push_str(&format!("Kbd({}, aria-hidden: \"true\")\n", src(&keys)));
                    }
                    let mut children = parse(&out);
                    expand(&mut children, &mut 0);
                    children.append(&mut part.children);
                    part.children = children;
                    give(part, "role", "\"option\"");
                    continue;
                }
                _ => {}
            }
        }
        for body in stmt.kind.bodies_mut() {
            palette_parts(body);
        }
    }
}
