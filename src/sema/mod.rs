//! Resolution: what the new grammar wrote, checked against what the
//! components declare, and lowered onto the shapes the code generators read.
//!
//! The parser cannot know whether `.lg` on a `Button` is a size or a typo:
//! that is the registry's knowledge for a built-in, and the `component`
//! declaration's for one the author wrote. [`check`] asks, and reports
//! every flag, prop, case, event, slot and part that resolves to nothing;
//! [`lower`] rewrites the ones that resolve into the vocabulary the code
//! generators have always read — the legacy modifier words, string cases,
//! attribute arguments — so that a program in either grammar is one program
//! by the time it is compiled.
//!
//! `lower` is idempotent, and a no-op on a program the original grammar
//! produced.

pub mod types;

use crate::error::Diagnostic;
use crate::parser::ast::*;
use crate::registry::{self, ComponentSig, Flag, Legacy, PropType, Sink};
use std::collections::HashMap;

/// What the resolver found: errors stop a build, warnings are printed.
#[derive(Debug, Default)]
pub struct Findings {
    pub errors: Vec<Diagnostic>,
    pub warnings: Vec<Diagnostic>,
}

/// The declared vocabulary of a program: its components, enums and types.
struct Decls<'a> {
    components: HashMap<&'a str, &'a ComponentDecl>,
    enums: HashMap<&'a str, &'a EnumDecl>,
    /// The animations the program declares: cases `animate:` and `exit:`
    /// take beside the built-in ones.
    animations: Vec<&'a str>,
}

impl<'a> Decls<'a> {
    fn of(program: &'a Program) -> Self {
        let mut components = HashMap::new();
        let mut enums = HashMap::new();
        let mut animations = Vec::new();
        for decl in &program.declarations {
            match decl {
                Declaration::Component(c) => {
                    components.insert(c.name.as_str(), c);
                }
                Declaration::Enum(e) => {
                    enums.insert(e.name.as_str(), e);
                }
                Declaration::Animation(a) => animations.push(a.name.as_str()),
                _ => {}
            }
        }
        Self {
            components,
            enums,
            animations,
        }
    }

    /// The cases of the enum a prop of type `ty` accepts, when it is one.
    fn cases_of(&self, ty: &TypeRef) -> Option<Vec<String>> {
        match ty {
            TypeRef::Named(name) => self.enums.get(name.as_str()).map(|e| e.case_names()),
            TypeRef::Optional(inner) => self.cases_of(inner),
            _ => None,
        }
    }
}

// ─── Check ───────────────────────────────────────────────

/// Every element, prop, flag, event, slot and part that does not resolve.
pub fn check(program: &Program, file_of: &dyn Fn(usize) -> String) -> Findings {
    let decls = Decls::of(program);
    let mut findings = Findings::default();
    for (index, decl) in program.declarations.iter().enumerate() {
        let file = file_of(index);
        let (body, component): (&[Statement], Option<&ComponentDecl>) = match decl {
            Declaration::Page(p) => {
                if let Some(layout) = &p.layout {
                    match decls.components.get(layout.name.as_str()) {
                        None => findings.errors.push(
                            Diagnostic::new(
                                format!("`{}` is not a declared component", layout.name),
                                &file,
                                layout.span.line as usize,
                                layout.span.col as usize,
                            )
                            .with_hint("A page's layout is a component with a default slot"),
                        ),
                        Some(c) if !c.slots.iter().any(|s| s.name.is_none()) => {
                            findings.errors.push(
                                Diagnostic::new(
                                    format!("`{}` declares no default slot, so the page has nowhere to go", layout.name),
                                    &file,
                                    layout.span.line as usize,
                                    layout.span.col as usize,
                                )
                                .with_hint("Add `slot` to the component and place `children` where the page belongs"),
                            )
                        }
                        Some(_) => {}
                    }
                }
                (&p.body, None)
            }
            Declaration::Component(c) => (&c.body, Some(c)),
            Declaration::App(a) => (&a.body, None),
            Declaration::Store(s) => (&s.body, None),
            Declaration::Theme(_)
            | Declaration::Type(_)
            | Declaration::Enum(_)
            | Declaration::Const(_)
            | Declaration::Animation(_)
            | Declaration::Test(_)
            | Declaration::Data(_) => continue,
        };
        let mut cx = Checker {
            decls: &decls,
            file: &file,
            component,
            findings: &mut findings,
        };
        cx.statements(body);
    }
    findings
}

struct Checker<'a, 'p> {
    decls: &'a Decls<'p>,
    file: &'a str,
    /// The component whose body is being checked, for `emit`.
    component: Option<&'p ComponentDecl>,
    findings: &'a mut Findings,
}

impl Checker<'_, '_> {
    fn error(&mut self, span: Span, message: String, hint: &str) {
        let d = Diagnostic::new(message, self.file, span.line as usize, span.col as usize);
        self.findings.errors.push(if hint.is_empty() {
            d
        } else {
            d.with_hint(hint)
        });
    }

    fn warning(&mut self, span: Span, message: String, hint: &str) {
        let d = Diagnostic::new(message, self.file, span.line as usize, span.col as usize);
        self.findings.warnings.push(if hint.is_empty() {
            d
        } else {
            d.with_hint(hint)
        });
    }

    fn statements(&mut self, stmts: &[Statement]) {
        for stmt in stmts {
            match &stmt.kind {
                StatementKind::UIElement(el) => self.element(el, stmt.span),
                StatementKind::If(i) => {
                    self.statements(&i.then_body);
                    for (_, b) in &i.else_if_branches {
                        self.statements(b);
                    }
                    if let Some(b) = &i.else_body {
                        self.statements(b);
                    }
                }
                StatementKind::For(f) => self.statements(&f.body),
                StatementKind::Show(s) => self.statements(&s.body),
                StatementKind::Match(m) => {
                    let over_resource = m.arms.iter().any(|a| {
                        matches!(
                            a.pattern,
                            ArmPattern::Loading | ArmPattern::Error | ArmPattern::Ready
                        )
                    });
                    let over_enum = m
                        .arms
                        .iter()
                        .any(|a| matches!(a.pattern, ArmPattern::Case(_)));
                    if over_resource && over_enum {
                        self.error(
                            stmt.span,
                            "A `match` is over a resource (`loading`, `error`, `ready`) or over an enum (`.case`), not both".into(),
                            "",
                        );
                    }
                    for arm in &m.arms {
                        self.statements(&arm.body);
                    }
                }
                StatementKind::Fetch(f) => {
                    for body in [
                        f.loading_block.as_deref(),
                        f.error_block.as_ref().map(|(_, b)| b.as_slice()),
                        f.success_block.as_deref(),
                    ]
                    .into_iter()
                    .flatten()
                    {
                        self.statements(body);
                    }
                }
                StatementKind::Action(a) => self.imperative(&a.body),
                StatementKind::Effect(e) => {
                    self.imperative(&e.body);
                    self.imperative(&e.cleanup);
                }
                StatementKind::Timer(t) => self.imperative(&t.body),
                StatementKind::EventHandler(h) => self.imperative(&h.body),
                _ => {}
            }
        }
    }

    /// `emit` names a declared event of the enclosing component.
    fn imperative(&mut self, stmts: &[Statement]) {
        for stmt in stmts {
            match &stmt.kind {
                StatementKind::Emit(e) => match self.component {
                    Some(c) if c.events.iter().any(|ev| ev.name == e.event) => {}
                    Some(c) => self.error(
                        stmt.span,
                        format!("`{}` declares no event `{}`", c.name, e.event),
                        &format!(
                            "Declare it at the top of the component: `event {}`",
                            e.event
                        ),
                    ),
                    None => self.error(
                        stmt.span,
                        "`emit` fires a component's event; a page has none".into(),
                        "",
                    ),
                },
                StatementKind::If(i) => {
                    self.imperative(&i.then_body);
                    for (_, b) in &i.else_if_branches {
                        self.imperative(b);
                    }
                    if let Some(b) = &i.else_body {
                        self.imperative(b);
                    }
                }
                StatementKind::For(f) => self.imperative(&f.body),
                StatementKind::Try(t) => {
                    self.imperative(&t.body);
                    self.imperative(&t.catch_body);
                }
                _ => {}
            }
        }
    }

    fn element(&mut self, el: &UIElement, span: Span) {
        match &el.component {
            ComponentRef::BuiltIn(name) => {
                if let Some(slot) = el.slot_name() {
                    self.slot_use(el, slot, span);
                }
                if let Some(sig) = registry::component(name) {
                    self.builtin(el, sig, span);
                }
            }
            // A part of the project's own component is a component itself.
            ComponentRef::SubComponent(owner, part)
                if self
                    .decls
                    .components
                    .contains_key(format!("{owner}.{part}").as_str()) =>
            {
                let qualified = format!("{owner}.{part}");
                if let Some(decl) = self.decls.components.get(qualified.as_str()) {
                    self.user_component(el, decl, span);
                }
            }
            ComponentRef::SubComponent(owner, part) => match registry::part(owner, part) {
                Some(sig) => self.builtin(el, sig, span),
                None => {
                    let mut known: Vec<String> = registry::parts_of(owner)
                        .map(|p| format!("`{owner}.{}`", p.name))
                        .collect();
                    if let Some(decl) = self.decls.components.get(owner.as_str()) {
                        known.extend(decl.parts.iter().map(|p| format!("`{owner}.{p}`")));
                    }
                    let hint = if known.is_empty() {
                        format!("`{owner}` has no parts")
                    } else {
                        format!("{owner} has {}", known.join(", "))
                    };
                    self.error(
                        span,
                        format!("`{owner}.{part}` is not a part of `{owner}`"),
                        &hint,
                    );
                }
            },
            ComponentRef::UserDefined(name) => {
                if let Some(decl) = self.decls.components.get(name.as_str()) {
                    self.user_component(el, decl, span);
                }
            }
        }
        for handler in &el.events {
            self.imperative(&handler.body);
        }
        self.statements(&el.children);
        for fill in &el.slot_fills {
            self.statements(&fill.body);
        }
    }

    /// A slot used in a component's body: one it declares, handed the
    /// values its declaration names.
    fn slot_use(&mut self, el: &UIElement, slot: &str, span: Span) {
        let Some(component) = self.component else {
            return;
        };
        let declared = component
            .slots
            .iter()
            .find(|s| s.name.as_deref().unwrap_or("children") == slot);
        let Some(decl) = declared else {
            if slot != "children" {
                let names: Vec<String> = component
                    .slots
                    .iter()
                    .map(|s| format!("`{}`", s.name.as_deref().unwrap_or("children")))
                    .collect();
                self.error(
                    span,
                    format!("`{}` declares no slot `{slot}`", component.name),
                    &if names.is_empty() {
                        format!("Declare it first: `slot {slot}`")
                    } else {
                        format!("It declares {}", names.join(", "))
                    },
                );
            }
            return;
        };
        let handed: Vec<&str> = el
            .args
            .iter()
            .filter_map(|a| match a {
                Arg::Named(k, _) if k != "slot" => Some(k.as_str()),
                _ => None,
            })
            .collect();
        for name in &handed {
            if !decl.params.iter().any(|p| p.name == *name) {
                let params: Vec<String> = decl
                    .params
                    .iter()
                    .map(|p| format!("`{}`", p.name))
                    .collect();
                self.error(
                    span,
                    format!("`{slot}` hands no `{name}`"),
                    &if params.is_empty() {
                        format!("Declare what it hands over: `slot {slot}({name}: Type)`")
                    } else {
                        format!("It hands {}", params.join(", "))
                    },
                );
            }
        }
        for p in &decl.params {
            if !handed.contains(&p.name.as_str()) {
                self.error(
                    span,
                    format!("`{slot}` is used without `{}`", p.name),
                    &format!("Write `{slot}({}: value)`", p.name),
                );
            }
        }
    }

    fn builtin(&mut self, el: &UIElement, sig: &'static ComponentSig, span: Span) {
        let name = sig.qualified();
        for (i, word) in el.modifiers.iter().enumerate() {
            let at = el.modifier_spans.get(i).copied().unwrap_or(span);
            match sig.flag(word) {
                Flag::Bool(_) | Flag::Case(..) => {}
                Flag::Ambiguous(props) => {
                    let options: Vec<String> =
                        props.iter().map(|p| format!("`{p}: .{word}`")).collect();
                    self.error(
                        at,
                        format!("`.{word}` is ambiguous on {name}"),
                        &format!("Write {}", options.join(" or ")),
                    );
                }
                Flag::Unknown => {
                    // The universal flags are lowered before this runs; a
                    // word that reaches here resolves to nothing.
                    self.error(
                        at,
                        format!("{name} has no flag or enum case `{word}`"),
                        &self.flag_hint(sig),
                    );
                }
            }
        }
        for (i, arg) in el.args.iter().enumerate() {
            let at = el.arg_spans.get(i).copied().unwrap_or(span);
            // An icon the runtime does not draw shows its name as text.
            let icon = match arg {
                Arg::Positional(Expr::StringLiteral(s)) if name == "Icon" && i == 0 => Some(s),
                Arg::Named(k, Expr::StringLiteral(s)) if k == "icon" => Some(s),
                _ => None,
            };
            if let Some(icon) = icon
                && !registry::ICONS.contains(&icon.as_str())
            {
                self.warning(
                    at,
                    format!("`{icon}` is not an icon the runtime draws; it will show as the word"),
                    &format!("The icons: {}", registry::ICONS.join(", ")),
                );
            }
            match arg {
                Arg::Positional(_) => {
                    if sig.positional.is_none() && i == 0 {
                        // The original grammar passes text positionally to
                        // components that take none; kept as a warning until
                        // every project is migrated.
                        self.warning(
                            at,
                            format!("{name} takes no positional argument"),
                            "Name it: the registry lists the props it takes",
                        );
                    }
                }
                Arg::Named(key, value) => {
                    let Some(prop) = sig.prop(key) else {
                        if !sig.accepts_named(key) {
                            self.warning(
                                at,
                                format!("{name} has no prop `{key}`; it is written to the element as an attribute"),
                                "A typo here does nothing on screen; check the component's props",
                            );
                        }
                        continue;
                    };
                    if let Expr::EnumCase(case) = value {
                        // A declared animation is a case of `animate:` and `exit:`.
                        let own_animation = matches!(key.as_str(), "animate" | "exit")
                            && self.decls.animations.contains(&case.as_str());
                        match prop.ty {
                            _ if own_animation => {}
                            PropType::Enum(cases) if cases.iter().any(|c| c.name == *case) => {}
                            PropType::Enum(cases) => {
                                let mut names: Vec<String> =
                                    cases.iter().map(|c| format!(".{}", c.name)).collect();
                                if matches!(key.as_str(), "animate" | "exit") {
                                    names.extend(
                                        self.decls.animations.iter().map(|a| format!(".{a}")),
                                    );
                                }
                                self.error(
                                    at,
                                    format!("`{key}` on {name} has no case `.{case}`"),
                                    &format!("It takes {}", names.join(", ")),
                                );
                            }
                            _ => self.error(
                                at,
                                format!(
                                    "`{key}` on {name} is not an enum, so `.{case}` means nothing"
                                ),
                                "",
                            ),
                        }
                    }
                }
            }
        }
        for handler in &el.events {
            if !registry::is_dom_event(&handler.event)
                && !sig.events.contains(&handler.event.as_str())
            {
                self.warning(
                    handler.span,
                    format!("`{}` is not an event {name} fires", handler.event),
                    "The DOM events are click, input, change, submit, focus, blur, keydown, keyup, keypress, mouseenter, mouseleave",
                );
            }
        }
        for fill in &el.slot_fills {
            self.error(
                fill.span,
                format!("{name} has no slot `{}`", fill.name),
                "A built-in's block is its children; named slots belong to components you declare",
            );
        }
    }

    fn flag_hint(&self, sig: &'static ComponentSig) -> String {
        let mut words: Vec<String> = Vec::new();
        for p in sig.all_props() {
            if !p.shorthand {
                continue;
            }
            match p.ty {
                PropType::Bool => words.push(format!(".{}", p.name)),
                PropType::Enum(cases) => {
                    for c in cases {
                        if !c.name.is_empty() {
                            words.push(format!(".{}", c.name));
                        }
                    }
                }
                _ => {}
            }
        }
        words.sort();
        words.dedup();
        if words.is_empty() {
            "It takes no flags".to_string()
        } else {
            format!("Its flags are {}", words.join(", "))
        }
    }

    fn user_component(&mut self, el: &UIElement, decl: &ComponentDecl, span: Span) {
        let name = &decl.name;
        for (i, word) in el.modifiers.iter().enumerate() {
            let at = el.modifier_spans.get(i).copied().unwrap_or(span);
            let bool_prop = decl
                .props
                .iter()
                .any(|p| p.name == *word && p.prop_type == TypeRef::Bool);
            let case_props: Vec<&str> = decl
                .props
                .iter()
                .filter(|p| {
                    self.decls
                        .cases_of(&p.prop_type)
                        .is_some_and(|c| c.iter().any(|c| c == word))
                })
                .map(|p| p.name.as_str())
                .collect();
            let universal = matches!(
                registry::component("Container").map(|c| c.flag(word)),
                Some(Flag::Case(..) | Flag::Bool(_))
            );
            match (bool_prop, case_props.len()) {
                (true, _) | (false, 1) => {}
                (false, 0) if universal => {}
                (false, 0) => self.error(
                    at,
                    format!("`{name}` has no flag or enum case `{word}`"),
                    "A flag names a Bool prop, or a case of an enum-typed prop, of the component",
                ),
                (false, _) => {
                    let options: Vec<String> = case_props
                        .iter()
                        .map(|p| format!("`{p}: .{word}`"))
                        .collect();
                    self.error(
                        at,
                        format!("`.{word}` is ambiguous on `{name}`"),
                        &format!("Write {}", options.join(" or ")),
                    );
                }
            }
        }
        for (i, arg) in el.args.iter().enumerate() {
            let at = el.arg_spans.get(i).copied().unwrap_or(span);
            match arg {
                Arg::Positional(_) => {
                    if !decl.props.iter().any(|p| p.positional) && i == 0 && !decl.props.is_empty()
                    {
                        self.warning(
                            at,
                            format!(
                                "`{name}` declares no positional prop; the argument binds to `{}`",
                                decl.props[0].name
                            ),
                            &format!(
                                "Mark the prop: `component {name}(_ {}: …)`",
                                decl.props[0].name
                            ),
                        );
                    }
                }
                Arg::Named(key, value) => {
                    let Some(prop) = decl.props.iter().find(|p| p.name == *key) else {
                        if !key.contains('-')
                            && !registry::UNIVERSAL_PROPS.iter().any(|p| p.name == key)
                        {
                            self.warning(
                                at,
                                format!("`{name}` declares no prop `{key}`"),
                                "It is passed anyway, but nothing in the component reads it",
                            );
                        }
                        continue;
                    };
                    if let Expr::EnumCase(case) = value {
                        match self.decls.cases_of(&prop.prop_type) {
                            Some(cases) if cases.iter().any(|c| c == case) => {}
                            Some(cases) => {
                                let names: Vec<String> =
                                    cases.iter().map(|c| format!(".{c}")).collect();
                                self.error(
                                    at,
                                    format!("`{key}` on `{name}` has no case `.{case}`"),
                                    &format!("It takes {}", names.join(", ")),
                                );
                            }
                            None => self.error(
                                at,
                                format!(
                                    "`{key}` on `{name}` is not an enum, so `.{case}` means nothing"
                                ),
                                "",
                            ),
                        }
                    }
                }
            }
        }
        for handler in &el.events {
            let declared = decl.events.iter().any(|e| e.name == handler.event);
            if !declared && !registry::is_dom_event(&handler.event) {
                let events: Vec<String> = decl
                    .events
                    .iter()
                    .map(|e| format!("`{}`", e.name))
                    .collect();
                self.error(
                    handler.span,
                    format!("`{name}` declares no event `{}`", handler.event),
                    &if events.is_empty() {
                        "It declares no events".to_string()
                    } else {
                        format!("It declares {}", events.join(", "))
                    },
                );
            }
        }
        for fill in &el.slot_fills {
            let declared = decl
                .slots
                .iter()
                .find(|s| s.name.as_deref() == Some(fill.name.as_str()));
            if let Some(slot) = declared {
                if fill.params.len() > slot.params.len() {
                    let hint = if slot.params.is_empty() {
                        format!(
                            "`{}` hands nothing over: write `{} {{ … }}`",
                            fill.name, fill.name
                        )
                    } else {
                        format!(
                            "It hands {}",
                            slot.params
                                .iter()
                                .map(|p| format!("`{}`", p.name))
                                .collect::<Vec<_>>()
                                .join(", ")
                        )
                    };
                    self.error(
                        fill.span,
                        format!(
                            "`{}` names {} value{}, but `{}` hands {} over",
                            fill.name,
                            fill.params.len(),
                            if fill.params.len() == 1 { "" } else { "s" },
                            fill.name,
                            slot.params.len()
                        ),
                        &hint,
                    );
                }
            } else {
                let slots: Vec<String> = decl
                    .slots
                    .iter()
                    .filter_map(|s| s.name.as_ref().map(|n| format!("`{n}`")))
                    .collect();
                self.error(
                    fill.span,
                    format!("`{name}` declares no slot `{}`", fill.name),
                    &if slots.is_empty() {
                        "It declares no named slots".to_string()
                    } else {
                        format!("It declares {}", slots.join(", "))
                    },
                );
            }
        }
    }
}

// ─── Lower ───────────────────────────────────────────────

/// The program with every new-grammar form rewritten into the vocabulary
/// the code generators read.
pub fn lower(mut program: Program) -> Program {
    let owned = Decls::of(&program);
    // The lowering needs the declarations while it mutates bodies; take the
    // pieces it reads out first.
    let user: HashMap<String, UserSig> = owned
        .components
        .values()
        .map(|c| {
            (
                c.name.clone(),
                UserSig {
                    props: c.props.iter().map(|p| p.name.clone()).collect(),
                    bool_props: c
                        .props
                        .iter()
                        .filter(|p| p.prop_type == TypeRef::Bool)
                        .map(|p| p.name.clone())
                        .collect(),
                    enum_props: c
                        .props
                        .iter()
                        .filter_map(|p| {
                            owned
                                .cases_of(&p.prop_type)
                                .map(|cases| (p.name.clone(), cases))
                        })
                        .collect(),
                },
            )
        })
        .collect();
    // Which enums carry a payload on some case: their name on a root is
    // the case's, not the whole value.
    let payload_enums: std::collections::HashSet<String> = owned
        .enums
        .values()
        .filter(|e| e.cases.iter().any(|c| !c.fields.is_empty()))
        .map(|e| e.name.clone())
        .collect();
    let enum_names: std::collections::HashSet<String> =
        owned.enums.keys().map(|k| k.to_string()).collect();
    drop(owned);
    // What a record's construction leaves out: each field's default, and
    // `null` for an optional one, so `Todo(id: "a", title: "b")` carries
    // `done: false` and `note: null` wherever it goes.
    let record_defaults = record_defaults(&program);
    let theme_tokens: std::collections::HashSet<String> = program
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Theme(t) => Some(t.tokens.iter().map(|tok| tok.name.clone())),
            _ => None,
        })
        .flatten()
        .collect();
    for decl in &mut program.declarations {
        if let Declaration::Component(c) = decl {
            mark_enum_props(c, &enum_names, &payload_enums);
        }
        let body = match decl {
            Declaration::Page(p) => &mut p.body,
            Declaration::Component(c) => &mut c.body,
            Declaration::App(a) => &mut a.body,
            Declaration::Store(s) => &mut s.body,
            Declaration::Theme(_)
            | Declaration::Type(_)
            | Declaration::Enum(_)
            | Declaration::Const(_)
            | Declaration::Animation(_)
            | Declaration::Test(_)
            | Declaration::Data(_) => continue,
        };
        lower_statements(body, &user);
        resolve_short_tokens(body, &theme_tokens);
        fill_records(body, &record_defaults);
    }
    for decl in &mut program.declarations {
        let value = match decl {
            Declaration::Const(c) => &mut c.value,
            Declaration::Test(t) => {
                fill_records(&mut t.body, &record_defaults);
                continue;
            }
            _ => continue,
        };
        value.walk_mut(&mut |e| fill_record(e, &record_defaults));
    }
    program
}

/// For each declared record, the fields a construction may leave out and
/// what they hold then, in declaration order.
fn record_defaults(program: &Program) -> HashMap<String, Vec<(String, Expr)>> {
    let types: HashMap<&str, &TypeDecl> = program
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Type(t) => Some((t.name.as_str(), t)),
            _ => None,
        })
        .collect();
    types
        .values()
        .map(|t| {
            let fields = t.all_fields(&|name| types.get(name).copied());
            let defaults = fields
                .iter()
                .filter_map(|f| match (&f.default, &f.ty) {
                    (Some(d), _) => Some((f.name.clone(), d.clone())),
                    (None, TypeRef::Optional(_)) => Some((f.name.clone(), Expr::Null)),
                    _ => None,
                })
                .collect();
            (t.name.clone(), defaults)
        })
        .collect()
}

fn fill_records(stmts: &mut [Statement], defaults: &HashMap<String, Vec<(String, Expr)>>) {
    walk_exprs_mut(stmts, &mut |e| fill_record(e, defaults));
}

fn fill_record(expr: &mut Expr, defaults: &HashMap<String, Vec<(String, Expr)>>) {
    let Expr::Record(name, fields) = expr else {
        return;
    };
    let Some(missing) = defaults.get(name) else {
        return;
    };
    for (field, value) in missing {
        if !fields.iter().any(|(k, _)| k == field) {
            fields.push((field.clone(), value.clone()));
        }
    }
}

/// A component's enum props are written to its root element as
/// `data-<prop>="<case>"`, so a stylesheet can select on them
/// (`.wf-card[data-tone="loud"]`), when the root is an element.
fn mark_enum_props(
    c: &mut ComponentDecl,
    enums: &std::collections::HashSet<String>,
    payload_enums: &std::collections::HashSet<String>,
) {
    let marks: Vec<(String, Expr)> = c
        .props
        .iter()
        .filter_map(|p| {
            let named = match &p.prop_type {
                TypeRef::Named(n) => n,
                TypeRef::Optional(inner) => match inner.as_ref() {
                    TypeRef::Named(n) => n,
                    _ => return None,
                },
                _ => return None,
            };
            if !enums.contains(named) {
                return None;
            }
            let value = if payload_enums.contains(named) {
                Expr::MethodCall(
                    Box::new(Expr::Identifier(p.name.clone())),
                    "__case".to_string(),
                    Vec::new(),
                )
            } else {
                Expr::Identifier(p.name.clone())
            };
            Some((format!("data-{}", p.name), value))
        })
        .collect();
    if marks.is_empty() {
        return;
    }
    let Some(first) = c.body.iter_mut().find(|s| {
        !matches!(
            s.kind,
            StatementKind::State(_)
                | StatementKind::Derived(_)
                | StatementKind::Action(_)
                | StatementKind::Effect(_)
                | StatementKind::Use(_)
                | StatementKind::Resource(_)
                | StatementKind::Timer(_)
                | StatementKind::EventHandler(_)
        )
    }) else {
        return;
    };
    if let StatementKind::UIElement(el) = &mut first.kind
        && !matches!(el.component, ComponentRef::UserDefined(_))
        && el.slot_name().is_none()
    {
        for (key, value) in marks {
            if !el
                .args
                .iter()
                .any(|a| matches!(a, Arg::Named(k, _) if *k == key))
            {
                el.args.push(Arg::Named(key, value));
            }
        }
    }
}

/// `$xl` on a padding is the spacing token `$spacing-xl`: a short name in a
/// style value resolves through the property's group, as a bare word did
/// in the original grammar. A name the theme declares stays itself.
fn resolve_short_tokens(stmts: &mut [Statement], theme: &std::collections::HashSet<String>) {
    let declared = |name: &str| theme.contains(name);
    let resolve = |props: &mut Vec<StyleProperty>| {
        for prop in props.iter_mut() {
            let css_prop = crate::codegen::style_tokens::canonical_style_prop(&prop.name);
            let full = |name: &str| {
                crate::codegen::style_tokens::resolve_short_token(&css_prop, name, &declared)
            };
            match &mut prop.value {
                Expr::Token(name) => {
                    if let Some(f) = full(name) {
                        *name = f;
                    }
                }
                Expr::InterpolatedString(parts) => {
                    for part in parts.iter_mut() {
                        if let StringPart::Expression(Expr::Token(name)) = part
                            && let Some(f) = full(name)
                        {
                            *name = f;
                        }
                    }
                }
                _ => {}
            }
        }
    };
    for_each_style_block(stmts, &mut |block| {
        resolve(&mut block.properties);
        for pseudo in &mut block.pseudo_blocks {
            resolve(&mut pseudo.properties);
        }
        for mq in &mut block.media_queries {
            resolve(&mut mq.properties);
        }
    });
}

/// Every style block under `stmts`, at any depth.
fn for_each_style_block(stmts: &mut [Statement], f: &mut dyn FnMut(&mut StyleBlock)) {
    for stmt in stmts.iter_mut() {
        match &mut stmt.kind {
            StatementKind::UIElement(el) => {
                if let Some(block) = &mut el.style_block {
                    f(block);
                }
                for_each_style_block(&mut el.children, f);
                for fill in &mut el.slot_fills {
                    for_each_style_block(&mut fill.body, f);
                }
            }
            StatementKind::If(i) => {
                for_each_style_block(&mut i.then_body, f);
                for (_, b) in &mut i.else_if_branches {
                    for_each_style_block(b, f);
                }
                if let Some(b) = &mut i.else_body {
                    for_each_style_block(b, f);
                }
            }
            StatementKind::For(l) => for_each_style_block(&mut l.body, f),
            StatementKind::Show(s) => for_each_style_block(&mut s.body, f),
            StatementKind::Match(m) => {
                for arm in &mut m.arms {
                    for_each_style_block(&mut arm.body, f);
                }
            }
            StatementKind::Fetch(fe) => {
                if let Some(b) = &mut fe.loading_block {
                    for_each_style_block(b, f);
                }
                if let Some((_, b)) = &mut fe.error_block {
                    for_each_style_block(b, f);
                }
                if let Some(b) = &mut fe.success_block {
                    for_each_style_block(b, f);
                }
            }
            _ => {}
        }
    }
}

/// What lowering needs to know about a user component.
struct UserSig {
    /// Every prop it declares: one named like a universal motion prop
    /// (`duration`, `delay`) is the component's own, not motion.
    props: Vec<String>,
    bool_props: Vec<String>,
    enum_props: Vec<(String, Vec<String>)>,
}

/// `on key("ctrl+k") { body }` → `on keydown(e) { if WF.keyIs(e, "ctrl+k") { body } }`:
/// one spelling of a key, matched by the runtime.
fn lower_key_handler(handler: &mut EventHandler) {
    let Some(key) = handler.key.take() else {
        return;
    };
    if handler.event != "key" {
        return;
    }
    let param = handler.param.clone().unwrap_or_else(|| "_ke".to_string());
    let body = std::mem::take(&mut handler.body);
    handler.event = "keydown".to_string();
    handler.param = Some(param.clone());
    handler.body = vec![Statement {
        kind: StatementKind::If(IfStmt {
            condition: Expr::FunctionCall(
                "WF.keyIs".to_string(),
                vec![Expr::Identifier(param), Expr::StringLiteral(key)],
            ),
            binding: None,
            animate: None,
            animate_span: None,
            then_body: body,
            else_if_branches: Vec::new(),
            else_body: None,
        }),
        span: handler.span,
    }];
}

/// The names of the props `el`'s component declares, when it is the
/// project's own: what a universal prop of the same name gives way to.
fn own_props<'u>(el: &UIElement, user: &'u HashMap<String, UserSig>) -> &'u [String] {
    match &el.component {
        ComponentRef::UserDefined(name) => user.get(name).map_or(&[], |us| us.props.as_slice()),
        _ => &[],
    }
}

fn lower_statements(stmts: &mut [Statement], user: &HashMap<String, UserSig>) {
    for stmt in stmts.iter_mut() {
        match &mut stmt.kind {
            StatementKind::UIElement(el) => lower_element(el, user),
            StatementKind::If(i) => {
                // One config for the whole `if`, read from the first branch
                // that carries motion props; the others give theirs up too.
                lift_animation(&mut i.animate, &mut i.then_body, user);
                for (_, b) in &mut i.else_if_branches {
                    lift_animation(&mut i.animate, b, user);
                }
                if let Some(b) = &mut i.else_body {
                    lift_animation(&mut i.animate, b, user);
                }
                lower_statements(&mut i.then_body, user);
                for (_, b) in &mut i.else_if_branches {
                    lower_statements(b, user);
                }
                if let Some(b) = &mut i.else_body {
                    lower_statements(b, user);
                }
            }
            StatementKind::For(f) => {
                lift_animation(&mut f.animate, &mut f.body, user);
                lower_statements(&mut f.body, user);
            }
            StatementKind::Show(s) => {
                lift_animation(&mut s.animate, &mut s.body, user);
                lower_statements(&mut s.body, user);
            }
            StatementKind::Match(m) => {
                for arm in &mut m.arms {
                    lower_statements(&mut arm.body, user);
                }
            }
            StatementKind::Fetch(f) => {
                if let Some(b) = &mut f.loading_block {
                    lower_statements(b, user);
                }
                if let Some((_, b)) = &mut f.error_block {
                    lower_statements(b, user);
                }
                if let Some(b) = &mut f.success_block {
                    lower_statements(b, user);
                }
            }
            _ => {}
        }
    }
}

/// The universal motion props on a branch's root elements are the branch's
/// animation config — what the runtime's conditional and list rendering
/// play on enter and exit — and come off the elements, as the original
/// grammar's `if …, animate(…)` clause never touched them. An element
/// outside a branch keeps its enter animation as a class, which plays when
/// it is first painted.
fn lift_animation(
    target: &mut Option<AnimateConfig>,
    body: &mut [Statement],
    user: &HashMap<String, UserSig>,
) {
    const MOTION_ARGS: &[&str] = &[
        "animate", "exit", "delay", "stagger", "duration", "speed", "easing",
    ];
    let mut config: Option<AnimateConfig> = None;
    for stmt in body.iter_mut() {
        let StatementKind::UIElement(el) = &mut stmt.kind else {
            continue;
        };
        let own = own_props(el, user).to_vec();
        let is_motion_word = |m: &str| {
            !own.iter().any(|p| p == m)
                && (crate::themes::prune::ANIMATIONS.contains(&m) || matches!(m, "fast" | "slow"))
        };
        let named = |key: &str| {
            if own.iter().any(|p| p == key) {
                return None;
            }
            el.args.iter().find_map(|a| match a {
                Arg::Named(k, Expr::StringLiteral(v)) if k == key => Some(v.clone()),
                Arg::Named(k, Expr::EnumCase(v)) if k == key => Some(v.clone()),
                _ => None,
            })
        };
        let enter = el
            .modifiers
            .iter()
            .find(|m| crate::themes::prune::ANIMATIONS.contains(&m.as_str()))
            .cloned()
            .or_else(|| named("animate"));
        let speed = el
            .modifiers
            .iter()
            .find(|m| matches!(m.as_str(), "fast" | "slow"))
            .cloned()
            .or_else(|| named("speed"));
        let exit = named("exit");
        let delay = named("delay");
        let stagger = named("stagger");
        let easing = named("easing").map(|e| crate::codegen::builtin::easing_css(&e).to_string());
        let duration = named("duration").or_else(|| match speed.as_deref() {
            Some("fast") => Some("150ms".to_string()),
            Some("slow") => Some("500ms".to_string()),
            _ => None,
        });
        let has_motion = enter.is_some()
            || exit.is_some()
            || delay.is_some()
            || stagger.is_some()
            || duration.is_some();
        if !has_motion {
            continue;
        }
        if config.is_none() {
            config = Some(AnimateConfig {
                enter: enter.unwrap_or_else(|| "fadeIn".to_string()),
                exit,
                duration,
                delay,
                stagger,
                easing,
            });
        }
        // Off the element: the branch plays them.
        let keep: Vec<bool> = el.modifiers.iter().map(|m| !is_motion_word(m)).collect();
        retain_by(&mut el.modifiers, &keep);
        retain_by(&mut el.modifier_spans, &keep);
        let keep: Vec<bool> = el
            .args
            .iter()
            .map(|a| {
                !matches!(a, Arg::Named(k, _)
                    if MOTION_ARGS.contains(&k.as_str()) && !own.iter().any(|p| p == k))
            })
            .collect();
        retain_by(&mut el.args, &keep);
        retain_by(&mut el.arg_spans, &keep);
    }
    if target.is_none() {
        *target = config;
    }
}

/// Keep the items of `items` whose flag in `keep` is set; a list of spans
/// shorter than its items (a node built by hand) is left alone.
fn retain_by<T>(items: &mut Vec<T>, keep: &[bool]) {
    if items.len() != keep.len() {
        return;
    }
    let mut i = 0;
    items.retain(|_| {
        let k = keep[i];
        i += 1;
        k
    });
}

fn lower_element(el: &mut UIElement, user: &HashMap<String, UserSig>) {
    // `on key("ctrl+k") { … }` is a keydown that runs when the key matches.
    for handler in &mut el.events {
        lower_key_handler(handler);
    }
    // A part of the project's own component is the component `Owner.Part`.
    if let ComponentRef::SubComponent(owner, part) = &el.component
        && user.contains_key(&format!("{owner}.{part}"))
    {
        el.component = ComponentRef::UserDefined(format!("{owner}.{part}"));
    }
    // Parts spelled the new way resolve to the names the generators know.
    if let ComponentRef::SubComponent(owner, part) = &el.component
        && let Some(sig) = registry::part(owner, part)
    {
        el.component = match sig.ir {
            registry::Ir::BuiltIn(name) => ComponentRef::BuiltIn(name.to_string()),
            registry::Ir::Sub(o, p) => ComponentRef::SubComponent(o.to_string(), p.to_string()),
        };
    }

    let sig = match &el.component {
        ComponentRef::BuiltIn(name) => registry::component(name),
        ComponentRef::SubComponent(owner, part) => registry::part(owner, part),
        ComponentRef::UserDefined(_) => None,
    };

    // Off a branch, an exit, a delay or a duration is the element's own:
    // written as `data-wf-*` markers the runtime reads when the element
    // leaves or first animates in. A stagger belongs to a list and means
    // nothing here.
    let user_component = matches!(el.component, ComponentRef::UserDefined(_));
    let own = own_props(el, user).to_vec();
    // The Router's `duration` times its route transition, not a marker.
    let router = matches!(&el.component, ComponentRef::BuiltIn(n) if n == "Router");
    for arg in el.args.iter_mut() {
        if !router
            && let Arg::Named(k, v) = arg
            && !own.iter().any(|p| p == k)
            && (matches!(k.as_str(), "exit" | "delay" | "duration" | "easing")
                || (user_component && matches!(k.as_str(), "animate" | "speed")))
        {
            let value = match v {
                Expr::EnumCase(c) | Expr::StringLiteral(c) => Expr::StringLiteral(c.clone()),
                other => other.clone(),
            };
            // A speed is a duration; an easing name is its timing function.
            let (key, value) = match (k.as_str(), &value) {
                ("speed", Expr::StringLiteral(s)) => (
                    "duration".to_string(),
                    Expr::StringLiteral(
                        match s.as_str() {
                            "fast" => "150ms",
                            "slow" => "500ms",
                            _ => "300ms",
                        }
                        .to_string(),
                    ),
                ),
                ("easing", Expr::StringLiteral(e)) => (
                    k.clone(),
                    Expr::StringLiteral(crate::codegen::builtin::easing_css(e).to_string()),
                ),
                _ => (k.clone(), value),
            };
            *arg = Arg::Named(format!("data-wf-{key}"), value);
        }
    }
    let keep: Vec<bool> = el
        .args
        .iter()
        .map(|a| !matches!(a, Arg::Named(k, _) if k == "stagger" && !own.iter().any(|p| p == k)))
        .collect();
    retain_by(&mut el.args, &keep);
    retain_by(&mut el.arg_spans, &keep);

    match (&el.component, sig) {
        (ComponentRef::UserDefined(name), _) => {
            if let Some(us) = user.get(name) {
                lower_user_flags(el, us);
            }
        }
        (_, Some(sig)) => lower_builtin(el, sig),
        _ => {}
    }
    lower_action_shorthand(el);

    lower_statements(&mut el.children, user);
    for fill in &mut el.slot_fills {
        lower_statements(&mut fill.body, user);
    }
}

/// The original grammar's click shorthand — a Button's, an IconButton's or
/// a user component's block of action statements — spelled out as the
/// `on click { … }` handler it always was, so one emitter serves both
/// grammars. A Button's block may mix content with its actions; a user
/// component's block is a handler only when it holds nothing else,
/// because otherwise it fills the default slot.
fn lower_action_shorthand(el: &mut UIElement) {
    if !el.events.is_empty() || el.children.is_empty() {
        return;
    }
    let mixed = match &el.component {
        ComponentRef::BuiltIn(name) => matches!(name.as_str(), "Button" | "IconButton"),
        ComponentRef::UserDefined(_) => false,
        ComponentRef::SubComponent(..) => return,
    };
    if !mixed && !el.children.iter().all(is_action_statement) {
        return;
    }
    let (actions, content): (Vec<Statement>, Vec<Statement>) = std::mem::take(&mut el.children)
        .into_iter()
        .partition(is_action_statement);
    el.children = content;
    if actions.is_empty() {
        return;
    }
    let span = Span {
        start: actions[0].span.start,
        end: actions[actions.len() - 1].span.end,
        ..actions[0].span
    };
    el.events.push(EventHandler {
        event: "click".to_string(),
        param: None,
        key: None,
        body: actions,
        span,
    });
}

/// Whether a statement does something rather than shows something.
fn is_action_statement(stmt: &Statement) -> bool {
    match &stmt.kind {
        StatementKind::Assignment(_)
        | StatementKind::MethodCall(_)
        | StatementKind::Navigate(_)
        | StatementKind::Log(_)
        | StatementKind::Emit(_)
        | StatementKind::ExprStatement(_) => true,
        // A branch of nothing but actions is a guard on them, not a
        // conditional render.
        StatementKind::If(i) => {
            let all = |b: &[Statement]| !b.is_empty() && b.iter().all(is_action_statement);
            all(&i.then_body)
                && i.else_if_branches.iter().all(|(_, b)| all(b))
                && i.else_body.as_deref().is_none_or(all)
        }
        _ => false,
    }
}

fn lower_builtin(el: &mut UIElement, sig: &'static ComponentSig) {
    // Named enum cases first — arguments precede flags in the source, and
    // the class order follows the source as it always did — then flags.
    let mut modifiers = Vec::new();
    let mut modifier_spans = Vec::new();
    let mut kept_args = Vec::new();
    let mut kept_spans = Vec::new();
    let arg_spans: Vec<Span> = (0..el.args.len())
        .map(|i| el.arg_spans.get(i).copied().unwrap_or_default())
        .collect();
    for (arg, span) in std::mem::take(&mut el.args).into_iter().zip(arg_spans) {
        match &arg {
            Arg::Named(key, Expr::EnumCase(case)) => {
                if let Some(prop) = sig.prop(key)
                    && let PropType::Enum(cases) = prop.ty
                    && let Some(c) = cases.iter().find(|c| c.name == *case)
                {
                    if !c.legacy.is_empty() {
                        modifiers.push(c.legacy.to_string());
                        modifier_spans.push(span);
                        continue;
                    }
                    if prop.sink == Sink::Class {
                        continue; // a default
                    }
                    kept_args.push(Arg::Named(
                        key.clone(),
                        Expr::StringLiteral(c.name.to_string()),
                    ));
                    kept_spans.push(span);
                    continue;
                }
                kept_args.push(Arg::Named(key.clone(), Expr::StringLiteral(case.clone())));
                kept_spans.push(span);
            }
            _ => {
                kept_args.push(arg);
                kept_spans.push(span);
            }
        }
    }

    // Flags → legacy modifier words, or attribute arguments.
    for (i, word) in el.modifiers.iter().enumerate() {
        let span = el.modifier_spans.get(i).copied().unwrap_or_default();
        match sig.flag(word) {
            Flag::Bool(p) => match p.legacy {
                Legacy::Modifier(w) => {
                    modifiers.push(w.to_string());
                    modifier_spans.push(span);
                }
                Legacy::Attr => {
                    kept_args.push(Arg::Named(p.name.to_string(), Expr::BoolLiteral(true)));
                    kept_spans.push(span);
                }
            },
            Flag::Case(p, c) => {
                if !c.legacy.is_empty() {
                    modifiers.push(c.legacy.to_string());
                    modifier_spans.push(span);
                } else if p.sink != Sink::Class {
                    kept_args.push(Arg::Named(
                        p.name.to_string(),
                        Expr::StringLiteral(c.name.to_string()),
                    ));
                    kept_spans.push(span);
                }
                // A class-sink case with no legacy word is a default: nothing.
            }
            Flag::Ambiguous(_) | Flag::Unknown => {
                modifiers.push(word.clone());
                modifier_spans.push(span);
            }
        }
    }
    el.modifiers = modifiers;
    el.modifier_spans = modifier_spans;
    el.args = kept_args;
    el.arg_spans = kept_spans;
}

fn lower_user_flags(el: &mut UIElement, us: &UserSig) {
    let mut modifiers = Vec::new();
    let mut modifier_spans = Vec::new();
    for (i, word) in el.modifiers.iter().enumerate() {
        let span = el.modifier_spans.get(i).copied().unwrap_or_default();
        if us.bool_props.contains(word) {
            el.args
                .push(Arg::Named(word.clone(), Expr::BoolLiteral(true)));
            el.arg_spans.push(span);
            continue;
        }
        let hits: Vec<&String> = us
            .enum_props
            .iter()
            .filter(|(_, cases)| cases.contains(word))
            .map(|(p, _)| p)
            .collect();
        if hits.len() == 1 {
            el.args
                .push(Arg::Named(hits[0].clone(), Expr::EnumCase(word.clone())));
            el.arg_spans.push(span);
            continue;
        }
        // A universal flag — an enter animation, a speed — is a motion
        // marker on the component's root; the rest are errors the check
        // reported, kept as words.
        if crate::themes::prune::ANIMATIONS.contains(&word.as_str()) {
            el.args.push(Arg::Named(
                "data-wf-animate".to_string(),
                Expr::StringLiteral(word.clone()),
            ));
            el.arg_spans.push(span);
            continue;
        }
        if let Some(ms) = match word.as_str() {
            "fast" => Some("150ms"),
            "slow" => Some("500ms"),
            _ => None,
        } {
            el.args.push(Arg::Named(
                "data-wf-duration".to_string(),
                Expr::StringLiteral(ms.to_string()),
            ));
            el.arg_spans.push(span);
            continue;
        }
        modifiers.push(word.clone());
        modifier_spans.push(span);
    }
    el.modifiers = modifiers;
    el.modifier_spans = modifier_spans;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::v2::parse_v2;

    fn checked(src: &str) -> Findings {
        let program = parse_v2(src, "<t>").expect("parse");
        check(&program, &|_| "<t>".to_string())
    }

    fn lowered(src: &str) -> Program {
        lower(parse_v2(src, "<t>").expect("parse"))
    }

    fn first_element(program: &Program) -> &UIElement {
        for decl in &program.declarations {
            if let Declaration::Page(p) = decl
                && let Some(Statement {
                    kind: StatementKind::UIElement(el),
                    ..
                }) = p.body.first()
            {
                return el;
            }
        }
        panic!("no element")
    }

    #[test]
    fn flags_lower_to_the_words_the_generators_read() {
        let p = lowered("page P(path: \"/\") { Button(\"x\").primary.lg.outlined.disabled }");
        let el = first_element(&p);
        assert_eq!(el.modifiers, vec!["primary", "large", "outlined"]);
        assert!(matches!(&el.args[1], Arg::Named(k, Expr::BoolLiteral(true)) if k == "disabled"));
        let p = lowered("page P(path: \"/\") { Heading(\"x\", level: .h1, align: .center) }");
        let el = first_element(&p);
        assert_eq!(el.modifiers, vec!["h1", "center"]);
        assert_eq!(el.args.len(), 1, "{:?}", el.args);
        let p = lowered("page P(path: \"/\") { Row(gap: .md, align: .center) { } }");
        let el = first_element(&p);
        assert!(
            matches!(&el.args[0], Arg::Named(k, Expr::StringLiteral(v)) if k == "gap" && v == "md")
        );
        assert!(el.modifiers.is_empty());
        let p = lowered("page P(path: \"/\") { Spacer.lg }");
        assert_eq!(first_element(&p).modifiers, vec!["lg"]);
        let p = lowered("page P(path: \"/\") { Input(bind: x).email }");
        assert_eq!(first_element(&p).modifiers, vec!["email"]);
    }

    #[test]
    fn parts_lower_to_their_ir_and_a_link_flag_to_its_argument() {
        let p = lowered("page P(path: \"/\") { Table.Row { Table.Cell(\"x\").header } }");
        let el = first_element(&p);
        assert!(matches!(&el.component, ComponentRef::BuiltIn(n) if n == "Trow"));
        let StatementKind::UIElement(cell) = &el.children[0].kind else {
            panic!()
        };
        assert!(matches!(&cell.component, ComponentRef::BuiltIn(n) if n == "Tcell"));
        assert_eq!(cell.modifiers, vec!["header"]);
        let p = lowered("page P(path: \"/\") { Link(\"Docs\", to: \"/docs\").prefix }");
        let el = first_element(&p);
        assert!(
            matches!(&el.args[2], Arg::Named(k, Expr::StringLiteral(v)) if k == "active" && v == "prefix")
        );
    }

    #[test]
    fn user_component_flags_become_props_and_motion_lifts_to_the_branch() {
        let src = "enum Tone { calm, loud }\ncomponent Chip(_ label: String, tone: Tone = .calm, removable: Bool = false) { Text(label) }\npage P(path: \"/\") { if open { Chip(\"x\", exit: .fadeOut, stagger: \"50ms\").loud.removable.fadeIn } }";
        let p = lowered(src);
        let Declaration::Page(page) = &p.declarations[2] else {
            panic!()
        };
        let StatementKind::If(i) = &page.body[0].kind else {
            panic!()
        };
        let anim = i.animate.as_ref().expect("lifted");
        assert_eq!(anim.enter, "fadeIn");
        assert_eq!(anim.exit.as_deref(), Some("fadeOut"));
        assert_eq!(anim.stagger.as_deref(), Some("50ms"));
        let StatementKind::UIElement(chip) = &i.then_body[0].kind else {
            panic!()
        };
        // The branch plays the enter animation; the element keeps no class.
        assert!(chip.modifiers.is_empty(), "{:?}", chip.modifiers);
        assert!(
            chip.args.iter().any(
                |a| matches!(a, Arg::Named(k, Expr::EnumCase(c)) if k == "tone" && c == "loud")
            )
        );
        assert!(
            chip.args
                .iter()
                .any(|a| matches!(a, Arg::Named(k, Expr::BoolLiteral(true)) if k == "removable"))
        );
        assert!(
            !chip
                .args
                .iter()
                .any(|a| matches!(a, Arg::Named(k, _) if k == "exit"))
        );
    }

    #[test]
    fn the_check_names_what_does_not_resolve() {
        let f = checked(
            "page P(path: \"/\") { Button(\"x\").huge  Row.center { }  Card.Nope { }  Button(\"y\", tone: .purple) }",
        );
        let messages: Vec<String> = f.errors.iter().map(|d| d.to_string()).collect();
        assert!(
            messages[0].contains("no flag or enum case `huge`") && messages[0].contains(".primary"),
            "{messages:?}"
        );
        assert!(
            messages[1].contains("`.center` is ambiguous")
                && messages[1].contains("`align: .center`"),
            "{messages:?}"
        );
        assert!(
            messages[2].contains("`Card.Nope` is not a part"),
            "{messages:?}"
        );
        assert!(
            messages[3].contains("has no case `.purple`"),
            "{messages:?}"
        );
        assert_eq!(f.errors.len(), 4, "{messages:?}");
    }

    #[test]
    fn a_declared_animation_is_a_case_of_animate_and_exit() {
        let src = "animation Pulse { from { opacity: 1 } to { opacity: 0 } }\npage P(path: \"/\") { Card(animate: .Pulse, exit: .Pulse) { Text(\"x\") }  Card(animate: .Wobble) { Text(\"y\") } }";
        let f = checked(src);
        let messages: Vec<String> = f.errors.iter().map(|d| d.to_string()).collect();
        assert_eq!(messages.len(), 1, "{messages:?}");
        assert!(
            messages[0].contains("`animate` on Card has no case `.Wobble`")
                && messages[0].contains(".Pulse"),
            "{messages:?}"
        );
    }

    #[test]
    fn a_part_is_a_component_of_its_owner() {
        let src = "component Panel(_ title: String) { part Header(_ text: String) { Text(text) }  Text(title)  children }\npage P(path: \"/\") { Panel(\"p\") { Panel.Header(\"h\")  Panel.Header(\"h\", extra: 1)  Panel.Nope } }";
        let f = checked(src);
        let messages: Vec<String> = f.errors.iter().map(|d| d.to_string()).collect();
        assert!(
            messages
                .iter()
                .any(|m| m.contains("`Panel.Nope` is not a part of `Panel`")
                    && m.contains("`Panel.Header`")),
            "{messages:?}"
        );
        assert_eq!(f.errors.len(), 1, "{messages:?}");
        let warnings: Vec<String> = f.warnings.iter().map(|d| d.to_string()).collect();
        assert!(
            warnings
                .iter()
                .any(|m| m.contains("`Panel.Header` declares no prop `extra`")),
            "{warnings:?}"
        );
        let src = "component Panel { part Header { part Deep { Text(\"x\") } } }";
        let err = parse_v2(src, "<t>")
            .err()
            .map(|e| e.to_string())
            .unwrap_or_default();
        assert!(err.contains("a part declares no parts of its own"), "{err}");
    }

    #[test]
    fn a_scoped_slot_hands_over_what_it_declares() {
        let src = "component Rows(items: [Any]) { slot row(item: Any, index: Number)  slot empty  for it, i in items { row(item: it, index: i) }  row(item: 1)  row(item: 1, index: 2, extra: 3)  gone(item: 1) }\npage P(path: \"/\") {  Rows(items: [1]) { row(n, i) { Text(\"{i}: {n}\") } }   Rows(items: [1]) { row(a, b, c) { Text(a) }  empty(x) { Text(x) } } }";
        let f = checked(src);
        let messages: Vec<String> = f.errors.iter().map(|d| d.to_string()).collect();
        for expected in [
            "`row` is used without `index`",
            "`row` hands no `extra`",
            "`Rows` declares no slot `gone`",
            "`row` names 3 values, but `row` hands 2 over",
            "`empty` names 1 value, but `empty` hands 0 over",
        ] {
            assert!(
                messages.iter().any(|m| m.contains(expected)),
                "{expected} in {messages:?}"
            );
        }
        assert_eq!(f.errors.len(), 5, "{messages:?}");
    }

    #[test]
    fn events_slots_and_layouts_are_checked_against_the_declaration() {
        let src = "component Entry(_ label: String) { event pick  slot trailing  Button(label) { on click { emit pick } }  Button(\"y\") { on click { emit nope } } }\npage P(path: \"/\", layout: Shell) { Entry(\"x\") { on chosen { go() }  on pick { go() }  side { Text(\"s\") } } }";
        let f = checked(src);
        let messages: Vec<String> = f.errors.iter().map(|d| d.to_string()).collect();
        assert!(
            messages
                .iter()
                .any(|m| m.contains("declares no event `nope`")),
            "{messages:?}"
        );
        assert!(
            messages
                .iter()
                .any(|m| m.contains("`Shell` is not a declared component")),
            "{messages:?}"
        );
        assert!(
            messages
                .iter()
                .any(|m| m.contains("declares no event `chosen`")),
            "{messages:?}"
        );
        assert!(
            messages
                .iter()
                .any(|m| m.contains("declares no slot `side`")),
            "{messages:?}"
        );
        assert_eq!(f.errors.len(), 4, "{messages:?}");
    }

    #[test]
    fn an_icon_the_runtime_does_not_draw_is_a_warning() {
        let f = checked(
            "page P(path: \"/\") { Icon(\"rocket\")  Icon(\"home\")  IconButton(icon: \"cog\", label: \"x\")  Sidebar { Sidebar.Item(to: \"/\", icon: \"settings\") { Text(\"s\") } } }",
        );
        let messages: Vec<String> = f.warnings.iter().map(|w| w.message.clone()).collect();
        assert_eq!(messages.len(), 2, "{messages:?}");
        assert!(
            messages[0].contains("`rocket` is not an icon"),
            "{messages:?}"
        );
        assert!(messages[1].contains("`cog` is not an icon"), "{messages:?}");
    }

    #[test]
    fn a_record_construction_carries_the_fields_it_left_out() {
        let p = lowered(
            "type Base { id: String, note: String? }\ntype Todo = Base { title: String, done: Bool = false, tone: String = \"calm\" }\nconst T = Todo(id: \"a\", title: \"x\", tone: \"loud\")\npage P(path: \"/\") { state t = Todo(id: \"b\", title: \"y\")\n Text(t.title) { on click { log(Todo(id: \"c\", title: \"z\", done: true)) } } }",
        );
        let fields = |e: &Expr| -> Vec<String> {
            let Expr::Record(_, f) = e else {
                panic!("{e:?}")
            };
            f.iter().map(|(k, v)| format!("{k}={v:?}")).collect()
        };
        let Declaration::Const(c) = &p.declarations[2] else {
            panic!()
        };
        assert_eq!(
            fields(&c.value),
            [
                "id=StringLiteral(\"a\")",
                "title=StringLiteral(\"x\")",
                "tone=StringLiteral(\"loud\")",
                "note=Null",
                "done=BoolLiteral(false)"
            ]
        );
        let Declaration::Page(page) = &p.declarations[3] else {
            panic!()
        };
        let StatementKind::State(st) = &page.body[0].kind else {
            panic!()
        };
        assert!(fields(&st.value).contains(&"done=BoolLiteral(false)".to_string()));
        let StatementKind::UIElement(el) = &page.body[1].kind else {
            panic!()
        };
        let StatementKind::Log(e) = &el.events[0].body[0].kind else {
            panic!("{:?}", el.events[0].body[0])
        };
        let f = fields(e);
        assert!(
            f.contains(&"done=BoolLiteral(true)".to_string())
                && f.contains(&"tone=StringLiteral(\"calm\")".to_string()),
            "{f:?}"
        );
    }

    #[test]
    fn lowering_is_idempotent() {
        let src = "page P(path: \"/\") { Button(\"x\").primary.lg Row(gap: .md) { Table.Cell(\"h\").header } }";
        let parsed = crate::syntax::parse_source(src, "<t>").unwrap();
        let once = lower(parsed.clone());
        let twice = lower(once.clone());
        assert_eq!(format!("{once:?}"), format!("{twice:?}"));
        let f = check(&parsed, &|_| "<t>".to_string());
        assert!(f.errors.is_empty(), "{:?}", f.errors);
    }

    #[test]
    fn an_easing_name_becomes_its_timing_function() {
        let src = "page P(path: \"/\") { state open = true\n if open { Text(\"x\", exit: .fadeOut, easing: \"spring\").fadeIn }\n Card { Text(\"y\", exit: .fadeOut, easing: \"easeOut\") } }";
        let program = lowered(src);
        let Declaration::Page(page) = &program.declarations[0] else {
            panic!()
        };
        let StatementKind::If(i) = &page.body[1].kind else {
            panic!("{:?}", page.body[1].kind)
        };
        let anim = i.animate.as_ref().unwrap();
        assert_eq!(
            anim.easing.as_deref(),
            Some("cubic-bezier(0.175, 0.885, 0.32, 1.275)")
        );
        let StatementKind::UIElement(card) = &page.body[2].kind else {
            panic!()
        };
        let StatementKind::UIElement(text) = &card.children[0].kind else {
            panic!()
        };
        assert!(text.args.iter().any(|a| matches!(a, Arg::Named(k, Expr::StringLiteral(v)) if k == "data-wf-easing" && v == "ease-out")), "{:?}", text.args);
    }

    #[test]
    fn a_components_own_prop_shadows_a_universal_motion_prop() {
        let src = "component DeployRow(duration: String, delay: String) { Text(duration)  Text(delay) }\npage P(path: \"/\") { state open = true\n DeployRow(duration: \"22.4 s\", delay: \"none\", exit: .fadeOut)\n if open { DeployRow(duration: \"1 s\", delay: \"x\").fadeIn } }";
        let program = lowered(src);
        let Declaration::Page(page) = &program.declarations[1] else {
            panic!()
        };
        let StatementKind::UIElement(row) = &page.body[1].kind else {
            panic!()
        };
        let named = |el: &UIElement, k: &str| {
            el.args
                .iter()
                .any(|a| matches!(a, Arg::Named(key, _) if key == k))
        };
        assert!(
            named(row, "duration") && named(row, "delay"),
            "{:?}",
            row.args
        );
        assert!(
            named(row, "data-wf-exit") && !named(row, "data-wf-duration"),
            "{:?}",
            row.args
        );
        let StatementKind::If(i) = &page.body[2].kind else {
            panic!()
        };
        let anim = i.animate.as_ref().unwrap();
        assert_eq!(anim.enter, "fadeIn");
        assert_eq!(anim.duration, None);
        assert_eq!(anim.delay, None);
        let StatementKind::UIElement(inner) = &i.then_body[0].kind else {
            panic!()
        };
        assert!(
            named(inner, "duration") && named(inner, "delay"),
            "{:?}",
            inner.args
        );
    }
}
