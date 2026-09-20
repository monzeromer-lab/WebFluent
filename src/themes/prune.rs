//! Drop the component rules a project never uses.
//!
//! The built-in stylesheet describes every component the engine has; a site
//! that uses twelve of them shipped the rules for fifty. The compiler knows
//! exactly which built-ins a program calls, so the sheet is cut down to those
//! — plus the sections every page needs: the reset, layout, typography, the
//! shared colour variants, the responsive reflow, focus rings, reduced
//! motion, the skip link.
//!
//! The sheet is written in `/* ─── Section ─── */` blocks, one per component,
//! and this reads it that way. A section is kept when a component it belongs
//! to appears in the program; a component that borrows another's rules (an
//! icon button draws an icon, a sidebar item is a link) keeps that section
//! too. A section this table does not know is kept, so a new one written
//! into the sheet is never silently dropped.

use std::collections::HashSet;

use crate::parser::ast::{Arg, Expr, Statement, StatementKind, UIElement};
use crate::parser::{ComponentRef, Declaration, Program};

/// What a program draws on, read from its elements.
#[derive(Debug, Default, Clone)]
pub struct Usage {
    /// Built-in component names used anywhere, sub-components counted as
    /// their parent.
    pub components: HashSet<String>,
    /// Whether any element carries an animation modifier, an `animate`
    /// clause, a `transition` block or a `Toast`.
    pub animation: bool,
    /// Whether any element takes an `icon:` argument.
    pub icon_arg: bool,
}

impl Usage {
    pub fn of(program: &Program) -> Usage {
        let mut usage = Usage::default();
        for decl in &program.declarations {
            let body = match decl {
                Declaration::Page(p) => &p.body,
                Declaration::Component(c) => &c.body,
                Declaration::App(a) => &a.body,
                Declaration::Store(_)
                | Declaration::Theme(_)
                | Declaration::Type(_)
                | Declaration::Enum(_) => continue,
            };
            walk(body, &mut usage);
        }
        usage
    }

    fn uses(&self, name: &str) -> bool {
        self.components.contains(name)
    }
}

const ANIMATIONS: &[&str] = &[
    "fadeIn",
    "fadeOut",
    "slideUp",
    "slideDown",
    "slideLeft",
    "slideRight",
    "scaleIn",
    "scaleOut",
    "bounce",
    "shake",
    "pulse",
    "spin",
];

fn walk(stmts: &[Statement], usage: &mut Usage) {
    for stmt in stmts {
        match &stmt.kind {
            StatementKind::UIElement(el) => element(el, usage),
            StatementKind::If(i) => {
                if i.animate.is_some() {
                    usage.animation = true;
                }
                walk(&i.then_body, usage);
                for (_, body) in &i.else_if_branches {
                    walk(body, usage);
                }
                if let Some(body) = &i.else_body {
                    walk(body, usage);
                }
            }
            StatementKind::For(f) => {
                if f.animate.is_some() {
                    usage.animation = true;
                }
                walk(&f.body, usage);
            }
            StatementKind::Show(s) => {
                if s.animate.is_some() {
                    usage.animation = true;
                }
                walk(&s.body, usage);
            }
            StatementKind::Fetch(f) => {
                if let Some(b) = &f.loading_block {
                    walk(b, usage);
                }
                if let Some((_, b)) = &f.error_block {
                    walk(b, usage);
                }
                if let Some(b) = &f.success_block {
                    walk(b, usage);
                }
            }
            StatementKind::Match(m) => {
                for arm in &m.arms {
                    walk(&arm.body, usage);
                }
            }
            StatementKind::Animate(_) => usage.animation = true,
            StatementKind::Action(a) => walk(&a.body, usage),
            StatementKind::Effect(e) => walk(&e.body, usage),
            StatementKind::EventHandler(h) => walk(&h.body, usage),
            _ => {}
        }
    }
}

fn element(el: &UIElement, usage: &mut Usage) {
    match &el.component {
        ComponentRef::BuiltIn(name) => {
            usage.components.insert(name.clone());
        }
        ComponentRef::SubComponent(parent, _) => {
            usage.components.insert(parent.clone());
        }
        ComponentRef::UserDefined(_) => {}
    }
    if el
        .modifiers
        .iter()
        .any(|m| ANIMATIONS.contains(&m.as_str()))
        || el.transition_block.is_some()
    {
        usage.animation = true;
    }
    if el
        .args
        .iter()
        .any(|a| matches!(a, Arg::Named(k, v) if k == "icon" && !matches!(v, Expr::Null)))
    {
        usage.icon_arg = true;
    }
    walk(&el.children, usage);
    for fill in &el.slot_fills {
        walk(&fill.body, usage);
    }
    for handler in &el.events {
        walk(&handler.body, usage);
    }
}

/// Whether a section named `title` (the text between the dashes) is needed.
fn keep(title: &str, usage: &Usage) -> bool {
    // The name up to a colon or parenthesis: "Switch (mechanism paint)" and
    // "Typography: requested modifiers only" are their first word.
    let name = title.split(['(', ':']).next().unwrap_or(title).trim();
    let any = |names: &[&str]| names.iter().any(|n| usage.uses(n));
    match name {
        "Navbar" => any(&["Navbar"]),
        "Sidebar" | "Off-canvas controls" => any(&["Sidebar"]),
        "Breadcrumb" => any(&["Breadcrumb"]),
        // A navbar's and a sidebar's entries are links.
        "Link" => any(&["Link", "Navbar", "Sidebar", "Breadcrumb"]),
        "Menu" => any(&["Menu"]),
        "Tabs" => any(&["Tabs", "TabPage"]),
        "Card" => any(&["Card"]),
        "Table" => any(&["Table", "Thead", "Tbody", "Trow", "Tcell"]),
        "List" => any(&["List"]),
        "Badge" => any(&["Badge"]),
        "Avatar" => any(&["Avatar"]),
        "Tooltip" => any(&["Tooltip"]),
        "Tag" => any(&["Tag"]),
        // A menu's trigger and a dialog's actions are buttons.
        "Button" => any(&[
            "Button",
            "IconButton",
            "ButtonGroup",
            "Dropdown",
            "Menu",
            "Form",
            "Modal",
            "Dialog",
        ]),
        "Icon Button" => any(&["IconButton"]),
        "Button Group" => any(&["ButtonGroup"]),
        "Dropdown" => any(&["Dropdown"]),
        "Input" => any(&[
            "Input",
            "Form",
            "DatePicker",
            "Slider",
            "FileUpload",
            "Select",
        ]),
        "Select" => any(&["Select"]),
        "Checkbox" => any(&["Checkbox"]),
        "Radio" => any(&["Radio"]),
        "Switch" => any(&["Switch"]),
        "Slider" => any(&["Slider"]),
        "Date Picker" => any(&["DatePicker"]),
        "File Upload" => any(&["FileUpload"]),
        "Form" => any(&["Form"]),
        "Alert" => any(&["Alert"]),
        "Toast" => any(&["Toast"]),
        "Modal" => any(&["Modal"]),
        "Dialog" => any(&["Dialog"]),
        "Spinner" => any(&["Spinner"]),
        "Progress" => any(&["Progress"]),
        "Skeleton" => any(&["Skeleton"]),
        "Image" => any(&["Image", "Avatar", "Carousel"]),
        "Icon" => any(&["Icon", "IconButton", "Sidebar"]) || usage.icon_arg,
        "Carousel" => any(&["Carousel"]),
        "Animation Keyframes" | "Animation Utility Classes" | "Hover Replay" => {
            usage.animation || any(&["Toast", "Spinner", "Skeleton", "Progress"])
        }
        // Everything else — reset, layout, typography, variants, responsive
        // reflow, focus, reduced motion, hit areas, the skip link, and any
        // section this table does not know — stays.
        _ => true,
    }
}

/// `css` with the sections the program does not draw on removed.
pub fn prune_css(css: &str, usage: &Usage) -> String {
    let mut out = String::with_capacity(css.len());
    let mut keeping = true;
    for line in css.split_inclusive('\n') {
        if let Some(title) = section_title(line) {
            keeping = keep(title, usage);
        }
        if keeping {
            out.push_str(line);
        }
    }
    out
}

/// The name in a `/* ─── Name ─── */` marker line.
fn section_title(line: &str) -> Option<&str> {
    let trimmed = line.trim_start();
    let rest = trimmed.strip_prefix("/* ─── ")?;
    let end = rest.find(" ─")?;
    Some(rest[..end].trim())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn program(src: &str) -> Program {
        crate::syntax::parse_source(src, "<t>").expect("parse")
    }

    #[test]
    fn a_page_of_text_keeps_the_frame_and_drops_the_widgets() {
        let p = program(r#"Page P (path: "/") { Container { Text("hi") } }"#);
        let css = prune_css(crate::themes::component_css(), &Usage::of(&p));
        assert!(css.contains(".wf-container"), "layout stays");
        assert!(css.contains(".wf-text"), "typography stays");
        assert!(
            css.contains("prefers-reduced-motion"),
            "reduced motion stays"
        );
        assert!(css.contains(".wf-skip-link"), "the skip link stays");
        // The responsive reflow block keeps a few selectors of its own; the
        // components' own rules are what go.
        assert!(
            !css.contains(".wf-carousel__track {"),
            "an unused component goes"
        );
        assert!(!css.contains(".wf-modal__content {"), "{css}");
        assert!(
            !css.contains("@keyframes wf-fadeIn"),
            "no animation, no keyframes"
        );
    }

    #[test]
    fn a_used_component_keeps_its_section_and_what_it_borrows() {
        let p = program(
            r#"Page P (path: "/") { Sidebar { Sidebar.Item(to: "/", icon: "home") { Text("Home") } } }"#,
        );
        let css = prune_css(crate::themes::component_css(), &Usage::of(&p));
        assert!(css.contains(".wf-sidebar"));
        assert!(css.contains(".wf-link"), "a sidebar item is a link");
        assert!(css.contains(".wf-icon"), "and draws an icon");
        assert!(!css.contains(".wf-navbar__brand a"));
    }

    #[test]
    fn an_animation_modifier_keeps_the_keyframes() {
        let p = program(r#"Page P (path: "/") { Card(fadeIn) { Text("x") } }"#);
        let css = prune_css(crate::themes::component_css(), &Usage::of(&p));
        assert!(css.contains("@keyframes"), "{css}");
        assert!(css.contains(".wf-card"));
    }

    #[test]
    fn the_structural_sheet_prunes_the_same_way() {
        let p = program(r#"Page P (path: "/") { Button("x") }"#);
        let css = prune_css(crate::themes::structural_css(), &Usage::of(&p));
        assert!(css.contains(".wf-btn"));
        assert!(!css.contains(".wf-tabs"), "{css}");
        assert!(
            css.contains("Hit Areas") || css.contains("min-height"),
            "hit areas stay"
        );
    }
}
