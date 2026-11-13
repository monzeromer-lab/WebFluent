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

use crate::syntax::parser::ast::{Arg, Expr, Statement, StatementKind, UIElement};
use crate::syntax::parser::{ComponentRef, Declaration, Program};

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
    /// Whether any element can be dragged or dropped on: `drag:`,
    /// `dropValue:`, `dropInside:`, `dropAxis:`, `dragGroup:`, or a handler
    /// of `receive` or `dragmove`.
    pub drag: bool,
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
                | Declaration::Enum(_)
                | Declaration::Api(_)
                | Declaration::Script(_)
                | Declaration::Const(_)
                | Declaration::Animation(_)
                | Declaration::Test(_)
                | Declaration::Data(_) => continue,
            };
            walk(body, &mut usage);
        }
        usage
    }

    fn uses(&self, name: &str) -> bool {
        self.components.contains(name)
    }
}

/// The animation modifier words.
pub const ANIMATIONS: &[&str] = &[
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
    "expand",
    "collapse",
];

fn walk(stmts: &[Statement], usage: &mut Usage) {
    for stmt in stmts {
        match &stmt.kind {
            StatementKind::UIElement(el) => element(el, usage),
            StatementKind::If(i) if i.animate.is_some() => usage.animation = true,
            StatementKind::For(f) if f.animate.is_some() => usage.animation = true,
            StatementKind::Show(s) if s.animate.is_some() => usage.animation = true,
            StatementKind::Animate(_) => usage.animation = true,
            _ => {}
        }
        // Every body a statement holds — a branch, an arm, a handler, a
        // `try`, a connection's — so nothing drawn inside one is missed.
        for body in stmt.kind.bodies() {
            walk(body, usage);
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
        || el.args.iter().any(|a| {
            matches!(a, Arg::Named(k, v) if k == "transition" && !matches!(v, Expr::Null))
                && matches!(&el.component, ComponentRef::BuiltIn(n) if n == "Router")
        })
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
    if el.args.iter().any(|a| {
        matches!(a, Arg::Named(k, _) if matches!(k.as_str(), "drag" | "dropValue" | "dropInside" | "dropAxis" | "dragGroup"))
    }) || el
        .events
        .iter()
        .any(|h| matches!(h.event.as_str(), "receive" | "dragmove"))
    {
        usage.drag = true;
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
/// The paged elements: a document's and a deck's own, and `Chart` and
/// `QrCode`, which draw on a web page too.
const PAGED: &[&str] = &[
    "Document",
    "Header",
    "Footer",
    "Background",
    "Watermark",
    "PageBreak",
    "Chart",
    "QrCode",
    "TableOfContents",
    "Presentation",
    "Slide",
    "TitleSlide",
    "SectionSlide",
    "TwoColumn",
    "ImageSlide",
    "Section",
    "Paragraph",
];

/// Whether a section named `title` belongs to something the program may not
/// use, and if so whether it does; `None` for a section every page needs.
fn decide(title: &str, usage: &Usage) -> Option<bool> {
    // The name up to a colon or parenthesis: "Switch (mechanism paint)" and
    // "Typography: requested modifiers only" are their first word.
    let name = title.split(['(', ':']).next().unwrap_or(title).trim();
    let any = |names: &[&str]| names.iter().any(|n| usage.uses(n));
    Some(match name {
        "Navbar" => any(&["Navbar"]),
        "Sidebar" => any(&["Sidebar"]),
        // The toggle that opens a narrow screen's menu: a navbar has one too,
        // which without this rule shows at every width.
        "Off-canvas controls" => any(&["Sidebar", "Navbar"]),
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
        "Popover" => any(&["Popover"]),
        "Frame" => any(&["Frame"]),
        "Splitter" => any(&["Splitter"]),
        "Virtual List" => any(&["VirtualList"]),
        "Portal" => any(&["Portal"]),
        // What puts a box in the top layer at its anchor.
        "Layer" => any(&["Popover", "Menu", "Dropdown", "Tooltip"]),
        "Spinner" => any(&["Spinner"]),
        "Progress" => any(&["Progress"]),
        "Skeleton" => any(&["Skeleton"]),
        "Image" => any(&["Image", "Avatar", "Carousel"]),
        "Icon" => any(&["Icon", "IconButton", "Sidebar"]) || usage.icon_arg,
        "Carousel" => any(&["Carousel"]),
        // Layout and typography a project may not use: each with its
        // component.
        "Column" => any(&["Column"]),
        "Grid" => any(&["Grid"]),
        "Spacer" => any(&["Spacer"]),
        "Divider" => any(&["Divider"]),
        "Code" => any(&["Code"]),
        // Highlighted code: a `Code` block, a Markdown fence, or markup a
        // page took from elsewhere that `highlight` coloured.
        "Highlight" => any(&["Code", "Markdown", "Unsafe"]),
        "Blockquote" => any(&["Blockquote"]),
        "Markdown" => any(&["Markdown"]),
        "Drag" => usage.drag,
        // A document's and a deck's own elements; `Chart` and `QrCode`
        // draw on a web page too.
        "Paged documents and slide decks" => any(PAGED),
        "Animation Keyframes" | "Animation Utility Classes" | "Hover Replay" => {
            usage.animation || any(&["Toast", "Spinner", "Skeleton", "Progress"])
        }
        // Everything else — reset, layout, typography, variants, responsive
        // reflow, focus, reduced motion, hit areas, the skip link, and any
        // section this table does not know — stays.
        _ => return None,
    })
}

/// `css` with the sections the program does not draw on removed, and, in
/// the sections every page keeps (the responsive reflow, focus, reduced
/// motion …), each selector that names a component whose own section went.
pub fn prune_css(css: &str, usage: &Usage) -> String {
    let unused = unused_classes(css, usage);
    let mut out = String::with_capacity(css.len());
    let mut section = String::new();
    // `None` for a section every page needs, which is pruned rule by rule.
    let mut decided: Option<bool> = None;
    let flush = |section: &mut String, decided: Option<bool>, out: &mut String| {
        match decided {
            Some(true) => out.push_str(section),
            Some(false) => {}
            None => out.push_str(&prune_rules(section, &unused)),
        }
        section.clear();
    };
    for line in css.split_inclusive('\n') {
        if let Some(title) = section_title(line) {
            flush(&mut section, decided, &mut out);
            decided = decide(title, usage);
        }
        section.push_str(line);
    }
    flush(&mut section, decided, &mut out);
    out
}

/// The classes of the components whose sections a program does not keep —
/// `wf-col` when nothing draws a `Column` — leaving out any class a
/// component it does draw also has (`Paragraph` is drawn as `wf-text`).
fn unused_classes(css: &str, usage: &Usage) -> HashSet<String> {
    let class_of = |name: &str| crate::project::builtin::builtin_to_html(name).1;
    let used: HashSet<&str> = usage.components.iter().map(|c| class_of(c)).collect();
    let mut unused = HashSet::new();
    for line in css.lines() {
        let Some(title) = section_title(line) else {
            continue;
        };
        if decide(title, usage) != Some(false) {
            continue;
        }
        let name = title.split(['(', ':']).next().unwrap_or(title).trim();
        let classes: Vec<&str> = match name {
            "Highlight" => vec!["wf-tok"],
            "Paged documents and slide decks" => PAGED.iter().map(|c| class_of(c)).collect(),
            _ => vec![class_of(&name.replace(' ', ""))],
        };
        for class in classes {
            if !class.is_empty() && !used.contains(class) {
                unused.insert(class.to_string());
            }
        }
    }
    unused
}

/// The family a class belongs to: `wf-col--6` and `wf-navbar__links` are
/// `wf-col`'s and `wf-navbar`'s.
fn class_root(class: &str) -> &str {
    let end = [class.find("__"), class.find("--")]
        .into_iter()
        .flatten()
        .min()
        .unwrap_or(class.len());
    &class[..end]
}

/// Whether a selector names a class of `unused`.
fn names_unused(selector: &str, unused: &HashSet<String>) -> bool {
    let mut rest = selector;
    while let Some(at) = rest.find(".wf-") {
        let class = &rest[at + 1..];
        let end = class
            .find(|c: char| !(c.is_ascii_alphanumeric() || c == '-' || c == '_'))
            .unwrap_or(class.len());
        if unused.contains(class_root(&class[..end])) {
            return true;
        }
        rest = &class[end..];
    }
    false
}

/// `css` with every selector that names an unused class taken out of its
/// list, every rule left with none taken out, and every `@media`,
/// `@supports` or `@container` block left empty taken out. Comments, other
/// at-rules (`@keyframes`, `@font-face`) and declarations are as written.
fn prune_rules(css: &str, unused: &HashSet<String>) -> String {
    if unused.is_empty() {
        return css.to_string();
    }
    let bytes = css.as_bytes();
    let mut out = String::with_capacity(css.len());
    let mut i = 0;
    while i < css.len() {
        // A comment, as written.
        if css[i..].starts_with("/*") {
            let end = css[i + 2..].find("*/").map_or(css.len(), |e| i + 2 + e + 2);
            out.push_str(&css[i..end]);
            i = end;
            continue;
        }
        if bytes[i].is_ascii_whitespace() || bytes[i] == b'}' {
            out.push(char::from(bytes[i]));
            i += 1;
            continue;
        }
        // A rule or an at-rule: its prelude up to `{` (or `;` for a
        // statement such as `@import`), then its block.
        let Some(open) = css[i..].find(['{', ';']).map(|n| i + n) else {
            out.push_str(&css[i..]);
            break;
        };
        if bytes[open] == b';' {
            out.push_str(&css[i..=open]);
            i = open + 1;
            continue;
        }
        let close = matching_brace(css, open);
        let prelude = &css[i..open];
        let block = &css[open + 1..close.min(css.len())];
        let after = (close + 1).min(css.len());
        let head = prelude.trim_start();
        if ["@media", "@supports", "@container"]
            .iter()
            .any(|at| head.starts_with(at))
        {
            let inner = prune_rules(block, unused);
            if inner.contains('{') {
                out.push_str(prelude);
                out.push('{');
                out.push_str(&inner);
                out.push('}');
            }
        } else if head.starts_with('@') {
            out.push_str(&css[i..after]);
        } else {
            let kept: Vec<&str> = split_selectors(prelude)
                .into_iter()
                .filter(|s| !names_unused(s, unused))
                .collect();
            if !kept.is_empty() {
                let lead = &prelude[..prelude.len() - prelude.trim_start().len()];
                out.push_str(lead);
                out.push_str(&kept.iter().map(|s| s.trim()).collect::<Vec<_>>().join(", "));
                out.push(' ');
                out.push('{');
                out.push_str(block);
                out.push('}');
            }
        }
        i = after;
    }
    out
}

/// The byte index of the `}` that closes the `{` at `open`, past comments
/// and strings; the end of `css` when it never closes.
fn matching_brace(css: &str, open: usize) -> usize {
    let bytes = css.as_bytes();
    let mut depth = 0usize;
    let mut i = open;
    while i < bytes.len() {
        match bytes[i] {
            b'/' if bytes.get(i + 1) == Some(&b'*') => {
                i = css[i + 2..]
                    .find("*/")
                    .map_or(bytes.len(), |e| i + 2 + e + 2);
                continue;
            }
            quote @ (b'"' | b'\'') => {
                i += 1;
                while i < bytes.len() && bytes[i] != quote {
                    i += if bytes[i] == b'\\' { 2 } else { 1 };
                }
            }
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return i;
                }
            }
            _ => {}
        }
        i += 1;
    }
    bytes.len()
}

/// A selector list split at its top-level commas: `:is(a, b), c` is two.
fn split_selectors(list: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth = 0i32;
    let mut start = 0;
    for (i, c) in list.char_indices() {
        match c {
            '(' | '[' => depth += 1,
            ')' | ']' => depth -= 1,
            ',' if depth == 0 => {
                parts.push(&list[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    parts.push(&list[start..]);
    parts.into_iter().filter(|s| !s.trim().is_empty()).collect()
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
        let p = program(r#"page P(path: "/") { Container { Text("hi") } }"#);
        let css = prune_css(crate::project::themes::component_css(), &Usage::of(&p));
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
    fn a_page_without_them_ships_no_grid_columns_code_markdown_slides_or_drag() {
        let p = program(r#"page P(path: "/") { Stack { Text("hi") Button("Go") } }"#);
        let css = prune_css(crate::project::themes::component_css(), &Usage::of(&p));
        for gone in [
            ".wf-col--6",
            ".wf-grid[data-cols",
            ".wf-spacer--xl",
            ".wf-divider--label",
            "pre.wf-code",
            ".wf-tok-kw",
            ".wf-blockquote {",
            ".wf-markdown pre",
            ".wf-toc__entry",
            ".wf-slide__title",
            ".wf-drag-ghost",
        ] {
            assert!(
                !css.contains(gone),
                "`{gone}` shipped to a page that never draws it"
            );
        }
        assert!(css.contains(".wf-stack"), "a stack keeps its layout");
        assert!(css.contains(".wf-gap--md"), "and its gaps");
        assert!(
            css.contains(".wf-align--center"),
            "and alignment, which every layout shares"
        );
    }

    #[test]
    fn each_split_section_comes_back_with_its_component() {
        for (src, kept) in [
            (r#"Grid(columns: 3) { Text("a") }"#, ".wf-grid[data-cols"),
            (r#"Row { Column(span: 6) { Text("a") } }"#, ".wf-col--6"),
            ("Spacer.xl", ".wf-spacer--xl"),
            ("Divider", ".wf-divider"),
            (r#"Code("x = 1").block"#, "pre.wf-code"),
            (r#"Code("x = 1").block"#, ".wf-tok-kw"),
            (r#"Markdown("`x`")"#, ".wf-markdown pre"),
            (r#"Markdown("`x`")"#, ".wf-tok-kw"),
            (r#"Blockquote { Text("q") }"#, ".wf-blockquote"),
            (
                r#"Chart(kind: .bar, data: [], x: "a", y: "b")"#,
                ".wf-chart svg",
            ),
            (r#"Text("row", drag: "a")"#, ".wf-drag-ghost"),
            (
                r#"Text("bin") { on receive(d) { log(d) } }"#,
                "[data-drop=\"inside\"]",
            ),
        ] {
            let p = program(&format!("page P(path: \"/\") {{ {src} }}"));
            let css = prune_css(crate::project::themes::component_css(), &Usage::of(&p));
            assert!(css.contains(kept), "`{src}` should keep `{kept}`");
        }
    }

    #[test]
    fn a_component_inside_a_try_or_a_handler_still_counts() {
        let p = program(
            r#"page P(path: "/") { state s = 0
    Button("Go") { on click { try { s = 1 } catch e { s = 2 } } }
    match s { else { Grid(columns: 2) { Text("in an arm") } } } }"#,
        );
        assert!(Usage::of(&p).components.contains("Grid"));
    }

    #[test]
    fn a_used_component_keeps_its_section_and_what_it_borrows() {
        let p = program(
            r#"page P(path: "/") { Sidebar { Sidebar.Item(to: "/", icon: "home") { Text("Home") } } }"#,
        );
        let css = prune_css(crate::project::themes::component_css(), &Usage::of(&p));
        assert!(css.contains(".wf-sidebar"));
        assert!(css.contains(".wf-link"), "a sidebar item is a link");
        assert!(css.contains(".wf-icon"), "and draws an icon");
        assert!(!css.contains(".wf-navbar__brand a"));
    }

    #[test]
    fn an_animation_modifier_keeps_the_keyframes() {
        let p = program(r#"page P(path: "/") { Card.fadeIn { Text("x") } }"#);
        let css = prune_css(crate::project::themes::component_css(), &Usage::of(&p));
        assert!(css.contains("@keyframes"), "{css}");
        assert!(css.contains(".wf-card"));
    }

    #[test]
    fn the_structural_sheet_prunes_the_same_way() {
        let p = program(r#"page P(path: "/") { Button("x") }"#);
        let css = prune_css(crate::project::themes::structural_css(), &Usage::of(&p));
        assert!(css.contains(".wf-btn"));
        assert!(!css.contains(".wf-tabs"), "{css}");
        assert!(
            css.contains("Hit Areas") || css.contains("min-height"),
            "hit areas stay"
        );
    }
}
