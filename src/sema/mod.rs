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
}

impl<'a> Decls<'a> {
    fn of(program: &'a Program) -> Self {
        let mut components = HashMap::new();
        let mut enums = HashMap::new();
        for decl in &program.declarations {
            match decl {
                Declaration::Component(c) => {
                    components.insert(c.name.as_str(), c);
                }
                Declaration::Enum(e) => {
                    enums.insert(e.name.as_str(), e);
                }
                _ => {}
            }
        }
        Self { components, enums }
    }

    /// The cases of the enum a prop of type `ty` accepts, when it is one.
    fn cases_of(&self, ty: &TypeRef) -> Option<&'a [String]> {
        match ty {
            TypeRef::Named(name) => self.enums.get(name.as_str()).map(|e| e.cases.as_slice()),
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
            Declaration::Theme(_) | Declaration::Type(_) | Declaration::Enum(_) => continue,
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
                StatementKind::Effect(e) => self.imperative(&e.body),
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
                _ => {}
            }
        }
    }

    fn element(&mut self, el: &UIElement, span: Span) {
        match &el.component {
            ComponentRef::BuiltIn(name) => {
                if let Some(sig) = registry::component(name) {
                    self.builtin(el, sig, span);
                }
            }
            ComponentRef::SubComponent(owner, part) => match registry::part(owner, part) {
                Some(sig) => self.builtin(el, sig, span),
                None => {
                    let known: Vec<String> = registry::parts_of(owner)
                        .map(|p| format!("`{owner}.{}`", p.name))
                        .collect();
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
                    // The original grammar's modifier words are accepted as
                    // they were; a new-grammar flag that resolves to nothing
                    // is an error.
                    if !crate::parser::vocabulary::is_modifier_keyword(word)
                        || registry::RETIRED_MODIFIERS.contains(&word.as_str())
                    {
                        self.error(
                            at,
                            format!("{name} has no flag or enum case `{word}`"),
                            &self.flag_hint(sig),
                        );
                    }
                }
            }
        }
        for (i, arg) in el.args.iter().enumerate() {
            let at = el.arg_spans.get(i).copied().unwrap_or(span);
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
                        match prop.ty {
                            PropType::Enum(cases) if cases.iter().any(|c| c.name == *case) => {}
                            PropType::Enum(cases) => {
                                let names: Vec<String> =
                                    cases.iter().map(|c| format!(".{}", c.name)).collect();
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
            if !decl
                .slots
                .iter()
                .any(|s| s.name.as_deref() == Some(fill.name.as_str()))
            {
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
                                .map(|cases| (p.name.clone(), cases.to_vec()))
                        })
                        .collect(),
                },
            )
        })
        .collect();
    drop(owned);
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
        let body = match decl {
            Declaration::Page(p) => &mut p.body,
            Declaration::Component(c) => &mut c.body,
            Declaration::App(a) => &mut a.body,
            Declaration::Store(s) => &mut s.body,
            Declaration::Theme(_) | Declaration::Type(_) | Declaration::Enum(_) => continue,
        };
        lower_statements(body, &user);
        resolve_short_tokens(body, &theme_tokens);
    }
    program
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
    bool_props: Vec<String>,
    enum_props: Vec<(String, Vec<String>)>,
}

fn lower_statements(stmts: &mut [Statement], user: &HashMap<String, UserSig>) {
    for stmt in stmts.iter_mut() {
        match &mut stmt.kind {
            StatementKind::UIElement(el) => lower_element(el, user),
            StatementKind::If(i) => {
                // One config for the whole `if`, read from the first branch
                // that carries motion props; the others give theirs up too.
                lift_animation(&mut i.animate, &mut i.then_body);
                for (_, b) in &mut i.else_if_branches {
                    lift_animation(&mut i.animate, b);
                }
                if let Some(b) = &mut i.else_body {
                    lift_animation(&mut i.animate, b);
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
                lift_animation(&mut f.animate, &mut f.body);
                lower_statements(&mut f.body, user);
            }
            StatementKind::Show(s) => {
                lift_animation(&mut s.animate, &mut s.body);
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
fn lift_animation(target: &mut Option<AnimateConfig>, body: &mut [Statement]) {
    const MOTION_ARGS: &[&str] = &[
        "animate", "exit", "delay", "stagger", "duration", "speed", "easing",
    ];
    let is_motion_word =
        |m: &str| crate::themes::prune::ANIMATIONS.contains(&m) || matches!(m, "fast" | "slow");
    let mut config: Option<AnimateConfig> = None;
    for stmt in body.iter_mut() {
        let StatementKind::UIElement(el) = &mut stmt.kind else {
            continue;
        };
        let named = |key: &str| {
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
        let easing = named("easing");
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
            .map(|a| !matches!(a, Arg::Named(k, _) if MOTION_ARGS.contains(&k.as_str())))
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

    // Off a branch there is nothing to play an exit or a delay yet; the
    // enter animation is a class already.
    let keep: Vec<bool> = el
        .args
        .iter()
        .map(|a| {
            !matches!(a, Arg::Named(k, _) if matches!(k.as_str(), "exit" | "delay" | "stagger" | "duration" | "easing"))
        })
        .collect();
    retain_by(&mut el.args, &keep);
    retain_by(&mut el.arg_spans, &keep);

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
        // Universal props (`.fadeIn`) stay modifier words; the rest are
        // errors the check reported.
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
    fn a_program_of_the_original_grammar_is_left_alone() {
        let src = "Page P (path: \"/\") { Button(\"x\", primary, large) Row(gap: md) { Tcell(\"h\", header) } }";
        let before = crate::syntax::parse_source(src, "<t>").unwrap();
        let after = lower(before.clone());
        assert_eq!(format!("{before:?}"), format!("{after:?}"));
        let f = check(&before, &|_| "<t>".to_string());
        assert!(f.errors.is_empty(), "{:?}", f.errors);
    }
}
