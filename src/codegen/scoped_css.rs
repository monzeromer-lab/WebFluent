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
use std::collections::BTreeMap;

use crate::codegen::style_tokens::{canonical_style_prop, resolve_style_token};
use crate::parser::ast::{
    Declaration, Expr, Program, Statement, StatementKind, StyleBlock, StyleProperty,
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
        collect(body, &mut rules);
    }
    let mut out = String::new();
    for css in rules.values() {
        out.push_str(css);
    }
    out
}

fn collect(stmts: &[Statement], rules: &mut BTreeMap<String, String>) {
    for stmt in stmts {
        match &stmt.kind {
            StatementKind::UIElement(el) => {
                if let Some(block) = &el.style_block {
                    if let Some(class) = scoped_class(block) {
                        rules
                            .entry(class.clone())
                            .or_insert_with(|| rules_for(&class, block));
                    }
                }
                collect(&el.children, rules);
            }
            StatementKind::If(i) => {
                collect(&i.then_body, rules);
                for (_, body) in &i.else_if_branches {
                    collect(body, rules);
                }
                if let Some(body) = &i.else_body {
                    collect(body, rules);
                }
            }
            StatementKind::For(f) => collect(&f.body, rules),
            StatementKind::Show(s) => collect(&s.body, rules),
            StatementKind::Fetch(f) => {
                if let Some(body) = &f.loading_block {
                    collect(body, rules);
                }
                if let Some((_, body)) = &f.error_block {
                    collect(body, rules);
                }
                if let Some(body) = &f.success_block {
                    collect(body, rules);
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
}
