//! Stylesheet rules compiled from `style { }` blocks.
//!
//! A `style { }` block used to compile to inline declarations on its element,
//! one assignment per property per element instance: a list of a hundred
//! cards repeated the same ten declarations a hundred times in the bundle, a
//! hundred more times in the static paint, and a hundred more times in the
//! browser's style resolution. Only a value that reads state has to be inline
//! — a literal and a design token are the same on every instance.
//!
//! Every block now gets a class named by the hash of its rules, so identical
//! blocks share one class and one rule, and the rules are appended to
//! `styles.css` at build time. The class carries:
//!
//! - the block's static base declarations, under a selector of tripled
//!   specificity (`.c.c.c`) so that, like the inline declaration it replaces,
//!   it beats every rule the component library puts on an element;
//! - its pseudo-state rules (`hover { … }`) and media queries, which no inline
//!   style can express; these are `!important`, so they override the base
//!   while their condition holds, as they did when the base was inline.
//!
//! A declaration whose value reads state stays inline and reactive, and beats
//! the class as an inline style always has. Every backend — the bundle, the
//! static paint, the template renderer — uses the same split, so hydration
//! finds the DOM it expects.
use std::collections::{BTreeMap, BTreeSet};

use crate::codegen::style_tokens::{canonical_style_prop, resolve_style_token};
use crate::parser::ast::{
    ComponentRef, Declaration, Expr, Program, Statement, StatementKind, StyleBlock, StyleProperty,
};

/// The class an element carries for its compiled style rules, or `None` when
/// nothing in its style block can be a stylesheet rule.
pub fn scoped_class(block: &StyleBlock) -> Option<String> {
    let text = canonical_text(block)?;
    Some(format!("wf-s{:08x}", fnv1a(&text)))
}

/// Every scoped rule a program needs, deduplicated and in a stable order,
/// ready to append to the stylesheet.
pub fn scoped_rules(program: &Program) -> String {
    let mut rules: BTreeMap<String, String> = BTreeMap::new();
    for decl in &program.declarations {
        let body = match decl {
            Declaration::Page(p) => &p.body,
            Declaration::Component(c) => &c.body,
            Declaration::App(a) => &a.body,
            Declaration::Store(_) | Declaration::Theme(_) => continue,
        };
        let mut uses = Uses::default();
        collect(body, &mut rules, &mut uses);
    }
    let mut out = String::new();
    for css in rules.values() {
        out.push_str(css);
    }
    out
}

/// The scoped rules of a program, split between the sheet every page loads
/// and a sheet per page.
///
/// A site's style blocks mostly belong to one page each — a landing page's
/// hero, a settings page's form — and in one shared sheet every page paid
/// for all of them. A rule goes to a page's own sheet when that page is the
/// only one that can reach it: directly in its body, or through the
/// components it uses, transitively. A rule the `App` body reaches, or a
/// component used from two pages, is shared. A component no page reaches is
/// shared too, so a rule is never lost.
#[derive(Debug, Default, Clone)]
pub struct SplitRules {
    /// Rules every page loads, in `styles.css`.
    pub shared: String,
    /// Rules only one page reaches, by page name; pages with none are absent.
    pub pages: BTreeMap<String, String>,
}

/// What a body reaches directly: its style blocks' classes, and the user
/// components it renders.
#[derive(Default)]
struct Uses {
    classes: BTreeSet<String>,
    components: BTreeSet<String>,
}

pub fn split_rules(program: &Program) -> SplitRules {
    let mut rules: BTreeMap<String, String> = BTreeMap::new();
    let mut app = Uses::default();
    let mut pages: Vec<(String, Uses)> = Vec::new();
    let mut components: BTreeMap<String, Uses> = BTreeMap::new();
    for decl in &program.declarations {
        match decl {
            Declaration::Page(p) => {
                let mut uses = Uses::default();
                collect(&p.body, &mut rules, &mut uses);
                pages.push((p.name.clone(), uses));
            }
            Declaration::Component(c) => {
                let mut uses = Uses::default();
                collect(&c.body, &mut rules, &mut uses);
                components.insert(c.name.clone(), uses);
            }
            Declaration::App(a) => collect(&a.body, &mut rules, &mut app),
            Declaration::Store(_) | Declaration::Theme(_) => {}
        }
    }

    // The classes a body reaches through its components, transitively.
    let reach = |uses: &Uses| -> BTreeSet<String> {
        let mut classes = uses.classes.clone();
        let mut seen: BTreeSet<String> = BTreeSet::new();
        let mut queue: Vec<String> = uses.components.iter().cloned().collect();
        while let Some(name) = queue.pop() {
            if !seen.insert(name.clone()) {
                continue;
            }
            if let Some(c) = components.get(&name) {
                classes.extend(c.classes.iter().cloned());
                queue.extend(c.components.iter().cloned());
            }
        }
        classes
    };

    // Which page, if exactly one, reaches each class.
    let mut owner: BTreeMap<String, Option<String>> = BTreeMap::new();
    for class in reach(&app) {
        owner.insert(class, None);
    }
    for (name, uses) in &pages {
        for class in reach(uses) {
            owner
                .entry(class)
                .and_modify(|o| {
                    if o.as_deref() != Some(name) {
                        *o = None;
                    }
                })
                .or_insert_with(|| Some(name.clone()));
        }
    }

    let mut split = SplitRules::default();
    for (class, css) in &rules {
        match owner.get(class) {
            Some(Some(page)) => split.pages.entry(page.clone()).or_default().push_str(css),
            _ => split.shared.push_str(css),
        }
    }
    split
}

fn collect(stmts: &[Statement], rules: &mut BTreeMap<String, String>, uses: &mut Uses) {
    for stmt in stmts {
        match &stmt.kind {
            StatementKind::UIElement(el) => {
                if let Some(block) = &el.style_block {
                    if let Some(class) = scoped_class(block) {
                        rules
                            .entry(class.clone())
                            .or_insert_with(|| rules_for(&class, block));
                        uses.classes.insert(class);
                    }
                }
                if let ComponentRef::UserDefined(name) = &el.component {
                    uses.components.insert(name.clone());
                }
                collect(&el.children, rules, uses);
            }
            StatementKind::If(i) => {
                collect(&i.then_body, rules, uses);
                for (_, body) in &i.else_if_branches {
                    collect(body, rules, uses);
                }
                if let Some(body) = &i.else_body {
                    collect(body, rules, uses);
                }
            }
            StatementKind::For(f) => collect(&f.body, rules, uses),
            StatementKind::Show(s) => collect(&s.body, rules, uses),
            StatementKind::Fetch(f) => {
                if let Some(body) = &f.loading_block {
                    collect(body, rules, uses);
                }
                if let Some((_, body)) = &f.error_block {
                    collect(body, rules, uses);
                }
                if let Some(body) = &f.success_block {
                    collect(body, rules, uses);
                }
            }
            _ => {}
        }
    }
}

/// The CSS selector suffix for a pseudo-state block's name.
fn selector(state: &str) -> &'static str {
    match state {
        "hover" => ":hover",
        "focus" => ":focus-visible",
        "active" => ":active",
        "disabled" => ":disabled",
        "placeholder" => "::placeholder",
        "focus-within" => ":focus-within",
        "current" => "[aria-current=\"page\"]",
        "pressed" => "[aria-pressed=\"true\"]",
        "selected" => "[aria-selected=\"true\"]",
        "checked" => "[aria-checked=\"true\"]",
        "expanded" => "[aria-expanded=\"true\"]",
        "invalid" => "[aria-invalid=\"true\"]",
        _ => "",
    }
}

/// A declaration's CSS property and value when both are known at build time
/// — a literal, or a design-token keyword — and `None` when the value reads
/// state, which only an inline declaration can follow.
///
/// This is the one place that decides what is hoisted; every backend asks it.
pub fn static_declaration(prop: &StyleProperty) -> Option<(String, String)> {
    let name = canonical_style_prop(&prop.name);
    let value = resolve_style_token(&name, &prop.value).or_else(|| match &prop.value {
        Expr::StringLiteral(s) => Some(s.clone()),
        Expr::NumberLiteral(n) => Some(format!("{}", n)),
        _ => None,
    })?;
    Some((name, value))
}

/// The base declarations a stylesheet rule can hold, as CSS text.
fn base_declarations(block: &StyleBlock) -> Vec<String> {
    block
        .properties
        .iter()
        .filter_map(static_declaration)
        .map(|(name, value)| format!("{name}: {value};"))
        .collect()
}

/// A pseudo-state or media declaration as CSS text, or `None` when its value
/// can only be known at run time — a stylesheet rule is static by nature, so
/// such a value is left to the inline path (where it is reactive) and dropped
/// here.
fn declaration(prop: &StyleProperty) -> Option<String> {
    let (name, value) = static_declaration(prop)?;
    // The base rule is written at tripled specificity to stand in for the
    // inline declaration it replaced. A pseudo-state or media rule exists to
    // override the base while its condition holds, so it has to be important;
    // the rules for one element are ordered pseudo-states first, media
    // queries last, so a viewport rule wins a conflict with a state rule.
    Some(format!("{}: {} !important;", name, value))
}

/// The rules of a block in one canonical string, which names the class: two
/// blocks that say the same thing hash the same.
fn canonical_text(block: &StyleBlock) -> Option<String> {
    let mut text = String::new();
    let base = base_declarations(block);
    if !base.is_empty() {
        text.push_str(&format!("{{{}}}", base.join(" ")));
    }
    for pseudo in &block.pseudo_blocks {
        let decls: Vec<String> = pseudo.properties.iter().filter_map(declaration).collect();
        if decls.is_empty() {
            continue;
        }
        text.push_str(&format!(
            "{}{{{}}}",
            selector(&pseudo.state),
            decls.join(" ")
        ));
    }
    for mq in &block.media_queries {
        let decls: Vec<String> = mq.properties.iter().filter_map(declaration).collect();
        if decls.is_empty() {
            continue;
        }
        text.push_str(&format!("{}{{{}}}", mq.condition, decls.join(" ")));
    }
    (!text.is_empty()).then_some(text)
}

fn rules_for(class: &str, block: &StyleBlock) -> String {
    let mut css = String::new();
    let base = base_declarations(block);
    if !base.is_empty() {
        css.push_str(&format!(
            ".{c}.{c}.{c} {{ {} }}\n",
            base.join(" "),
            c = class
        ));
    }
    for pseudo in &block.pseudo_blocks {
        let decls: Vec<String> = pseudo.properties.iter().filter_map(declaration).collect();
        if decls.is_empty() {
            continue;
        }
        css.push_str(&format!(
            ".{}{} {{ {} }}\n",
            class,
            selector(&pseudo.state),
            decls.join(" ")
        ));
    }
    for mq in &block.media_queries {
        let decls: Vec<String> = mq.properties.iter().filter_map(declaration).collect();
        if decls.is_empty() {
            continue;
        }
        css.push_str(&format!(
            "{} {{ .{} {{ {} }} }}\n",
            mq.condition,
            class,
            decls.join(" ")
        ));
    }
    css
}

/// FNV-1a, 32-bit. Written out rather than taken from `std::hash`, whose
/// default algorithm is not promised to be stable across Rust versions — and
/// a class name that changed between compiler releases would be a diff in
/// every built site for no reason.
fn fnv1a(text: &str) -> u32 {
    let mut hash: u32 = 0x811c9dc5;
    for byte in text.bytes() {
        hash ^= byte as u32;
        hash = hash.wrapping_mul(0x01000193);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    fn program(src: &str) -> Program {
        let tokens = Lexer::new(src, "<t>").tokenize().expect("lex");
        Parser::new(tokens, "<t>").parse().expect("parse")
    }

    fn first_block(src: &str) -> StyleBlock {
        let program = program(src);
        let body = match &program.declarations[0] {
            Declaration::Page(p) => &p.body,
            _ => panic!("page"),
        };
        match &body[0].kind {
            StatementKind::UIElement(el) => el.style_block.clone().expect("style block"),
            _ => panic!("element"),
        }
    }

    #[test]
    fn a_block_with_only_reactive_declarations_needs_no_class() {
        let b = first_block(r#"Page P (path: "/") { Card { style { width: w } } }"#);
        assert_eq!(scoped_class(&b), None);
    }

    #[test]
    fn literal_declarations_are_hoisted_into_a_tripled_class_rule() {
        let src = r#"Page P (path: "/") {
            Card { style { padding: "1rem"  radius: md  --gap: 4 } }
            Card { style { padding: "1rem"  radius: md  --gap: 4 } }
        }"#;
        let p = program(src);
        let css = scoped_rules(&p);
        let class = scoped_class(&first_block(src)).unwrap();
        assert_eq!(
            css.trim(),
            format!(
                ".{c}.{c}.{c} {{ padding: 1rem; border-radius: var(--radius-md); --gap: 4; }}",
                c = class
            )
        );
    }

    #[test]
    fn a_reactive_declaration_stays_out_of_the_rule() {
        let src = r#"Page P (path: "/") {
            state pct = 50
            Card { style { padding: "1rem"  width: "{pct}%" } }
        }"#;
        let p = program(src);
        let css = scoped_rules(&p);
        assert!(css.contains("padding: 1rem;"), "{css}");
        assert!(!css.contains("width"), "{css}");
    }

    #[test]
    fn identical_blocks_share_one_class_and_one_rule() {
        let src = r#"Page P (path: "/") {
            Button("a") { style { color: "red" hover { color: "blue" } } }
            Button("b") { style { color: "red" hover { color: "blue" } } }
            Button("c") { style { hover { color: "green" } } }
        }"#;
        let p = program(src);
        let css = scoped_rules(&p);
        assert_eq!(css.matches(":hover").count(), 2, "{css}");
        let a = first_block(src);
        let class = scoped_class(&a).unwrap();
        assert!(class.starts_with("wf-s"), "{class}");
        assert!(
            css.contains(&format!(".{class}:hover {{ color: blue !important; }}")),
            "{css}"
        );
    }

    #[test]
    fn states_map_to_their_selectors_and_tokens_resolve() {
        let src = r##"Page P (path: "/") {
            Input(text, placeholder: "p") {
                style {
                    focus { border-color: primary }
                    placeholder { color: "#999" }
                    focus-within { outline: "none" }
                    disabled { opacity: 0.5 }
                    active { transform: "translateY(1px)" }
                }
            }
        }"##;
        let css = scoped_rules(&program(src));
        assert!(
            css.contains(":focus-visible { border-color: var(--color-primary) !important; }"),
            "{css}"
        );
        assert!(
            css.contains("::placeholder { color: #999 !important; }"),
            "{css}"
        );
        assert!(
            css.contains(":focus-within { outline: none !important; }"),
            "{css}"
        );
        assert!(
            css.contains(":disabled { opacity: 0.5 !important; }"),
            "{css}"
        );
        assert!(
            css.contains(":active { transform: translateY(1px) !important; }"),
            "{css}"
        );
    }

    #[test]
    fn aria_states_are_attribute_selectors() {
        let src = r##"Page P (path: "/") {
            Link("Home", to: "/") { style { current { background: "#222" } } }
            Button("x", aria-pressed: "true") { style { pressed { color: "red" }  invalid { color: "blue" } } }
        }"##;
        let css = scoped_rules(&program(src));
        assert!(
            css.contains("[aria-current=\"page\"] { background: #222 !important; }"),
            "{css}"
        );
        assert!(
            css.contains("[aria-pressed=\"true\"] { color: red !important; }"),
            "{css}"
        );
        assert!(
            css.contains("[aria-invalid=\"true\"] { color: blue !important; }"),
            "{css}"
        );
    }

    #[test]
    fn media_queries_are_compiled_too_with_tokens_resolved() {
        let src = r#"Page P (path: "/") {
            Card { style { padding: xl  @media (max-width: 768px) { padding: sm } } }
        }"#;
        let css = scoped_rules(&program(src));
        assert!(css.contains("@media (max-width: 768px) { .wf-s"), "{css}");
        assert!(
            css.contains("{ padding: var(--spacing-sm) !important; } }"),
            "{css}"
        );
    }

    #[test]
    fn a_runtime_value_is_left_out_of_the_stylesheet() {
        let src = r##"Page P (path: "/") {
            state c = "red"
            Card { style { hover { color: c  background: "#fff" } } }
        }"##;
        let css = scoped_rules(&program(src));
        assert!(css.contains("background: #fff !important"), "{css}");
        assert!(!css.contains("color:"), "{css}");
    }

    #[test]
    fn rules_are_found_inside_control_flow_and_components() {
        let src = r#"
            Component Chip (label: String) { Badge(label) { style { hover { color: "red" } } } }
            Page P (path: "/") {
                state items = []
                for item in items { if item.on { Chip(label: "x") } }
            }"#;
        let css = scoped_rules(&program(src));
        assert!(css.contains(":hover { color: red !important; }"), "{css}");
    }

    #[test]
    fn rules_are_split_by_the_pages_that_reach_them() {
        let src = r#"
            Component Hero () { Container { style { padding: "9rem" } Text("h") } }
            Component Shell () { Container { style { padding: "7rem" } children } }
            Component Orphan () { Text("o") { style { padding: "5rem" } } }
            Page Home (path: "/") { Hero() Text("a") { style { padding: "1rem" } } }
            Page About (path: "/about") { Text("b") { style { padding: "2rem" } } Text("c") { style { padding: "3rem" } } }
            Page Team (path: "/team") { Text("d") { style { padding: "3rem" } } }
            App { Shell { Router { Route(path: "/", page: Home) } } }
        "#;
        let split = split_rules(&program(src));
        let home = &split.pages["Home"];
        assert!(
            home.contains("padding: 1rem"),
            "the page's own block: {home}"
        );
        assert!(
            home.contains("padding: 9rem"),
            "a component only it uses: {home}"
        );
        let about = &split.pages["About"];
        assert!(about.contains("padding: 2rem"));
        assert!(
            !about.contains("padding: 3rem"),
            "a block two pages share is shared"
        );
        assert!(split.shared.contains("padding: 3rem"));
        assert!(
            split.shared.contains("padding: 7rem"),
            "the app's shell is shared"
        );
        assert!(
            split.shared.contains("padding: 5rem"),
            "a component nothing reaches is kept"
        );
        assert!(!split.shared.contains("padding: 1rem"));
        assert!(!split.shared.contains("padding: 9rem"));
        assert!(
            !split.pages.contains_key("Team"),
            "a page whose every rule is shared has no sheet of its own"
        );
        let all = format!("{}{}{}", split.shared, home, about);
        assert_eq!(
            all.matches("padding: 3rem").count(),
            1,
            "every rule is written exactly once"
        );
        let whole = scoped_rules(&program(src));
        assert_eq!(
            all.len(),
            whole.len(),
            "the split holds exactly the unsplit rules"
        );
    }
}
