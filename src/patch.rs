//! An in-place patch from an edit: what a running page must change when a
//! `style { }` block or a literal text changes, with no reload.
//!
//! An editor's preview keeps its scroll and its state only if it does not
//! reload. [`in_place`] compares a program with the program an edit made and
//! answers one of three things:
//!
//! - [`InPlace::Same`]: nothing a page shows changed.
//! - [`InPlace::Patch`]: every difference is a static style or a literal
//!   text, and here is what the page needs. A style's rules live in a class
//!   named by their content (`scoped_css`), so a style edit is one class
//!   swapped and the new rules added. A text is the element's own string.
//! - [`InPlace::Rebuild`]: anything else, with the reason. A patch is only
//!   ever offered when it is certainly enough, so saying no is always safe.
//!
//! [`agrees`] holds a patch to the code a full build generates: the two
//! programs' pages must differ by the patch and by nothing more.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use crate::codegen::js::JsCodegen;
use crate::codegen::node_id::{build_node_map, visit_nodes};
use crate::codegen::scoped_css::{
    breakpoints, rules_for, scoped_class, scoped_rules, static_declaration,
};
use crate::data::each_element_mut;
use crate::parser::ast::{Arg, Declaration, Expr, Program, Span, StyleBlock, UIElement};
use serde::Serialize;

/// The built-ins whose first positional string is their own text, written as
/// the first text in the element they stamp, in both code generators.
const TEXTS: &[&str] = &[
    "Text", "Heading", "Button", "Badge", "Tag", "Link", "Alert", "Label",
];

/// Whether a style block's class goes on the one element this stamps. A
/// control drawn in a box stamps the box and the input with one id and takes
/// the class on one, and a windowed list draws its rows itself. A part of a
/// built-in (`Breadcrumb.Item`) is drawn by its parent, which may not take a
/// style at all, so only the parts shown to take one are let through. Every
/// other answer is a rebuild, which is always safe.
fn takes_a_class(component: &str) -> bool {
    if component.contains('.') {
        return PARTS.contains(&component);
    }
    !BOXED.contains(&component)
}

const BOXED: &[&str] = &["Checkbox", "Radio", "Switch", "Slider", "VirtualList"];
const PARTS: &[&str] = &["Table.Cell", "Table.Row"];

/// What one edit asks of a running page.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum InPlace {
    /// Nothing a page shows changed.
    Same,
    /// The page can follow without reloading.
    Patch(Patch),
    /// The page has to be built again, and why.
    Rebuild { why: String },
}

/// What a page changes to follow an edit.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Patch {
    /// Every changed element, by the id a studio build stamps on it. One id
    /// can be many elements on a page: a list's rows, a component's uses.
    pub nodes: Vec<NodePatch>,
    /// The stylesheet rules the new classes need.
    pub rules: String,
}

/// One element's change.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NodePatch {
    pub node: String,
    /// The class its style block compiled to, when it changes.
    pub remove_class: Option<String>,
    pub add_class: Option<String>,
    /// Its new text, when it changes.
    pub text: Option<String>,
}

/// What `before` and `after`, two whole programs, ask of a running page.
pub fn in_place(before: &Program, after: &Program) -> InPlace {
    in_place_with(before, after, &breakpoints(after))
}

/// [`in_place`] for two versions of one file: the project's breakpoints come
/// from where its theme is, which need not be this file.
pub fn in_place_with(
    before: &Program,
    after: &Program,
    points: &BTreeMap<String, String>,
) -> InPlace {
    let rebuild = |why: &str| InPlace::Rebuild {
        why: why.to_string(),
    };
    if before.declarations.len() != after.declarations.len() {
        return rebuild("a declaration was added or removed");
    }
    let old = elements(before);
    let new = elements(after);
    if old.len() != new.len() {
        return rebuild("an element was added or removed");
    }

    let mut patch = Patch::default();
    let mut rules = BTreeMap::new();
    // A span is unique within a file, and a program is many files: an
    // element is known by its owner and its span.
    let mut changed: HashMap<(&str, Span), &UIElement> = HashMap::new();
    for ((old_id, old_ui), (new_id, new_ui)) in old.iter().zip(&new) {
        if old_id != new_id
            || crate::print::component(&old_ui.component)
                != crate::print::component(&new_ui.component)
        {
            return rebuild("an element moved or became another");
        }
        let mut node = NodePatch {
            node: new_id.clone(),
            ..NodePatch::default()
        };
        let mut differs = false;

        if style_text(old_ui.style_block.as_ref()) != style_text(new_ui.style_block.as_ref()) {
            if live_text(old_ui.style_block.as_ref()) != live_text(new_ui.style_block.as_ref()) {
                return rebuild("a style that reads a value changed");
            }
            if !takes_a_class(&crate::print::component(&new_ui.component)) {
                return rebuild("this element does not take its style as one class");
            }
            let from = old_ui.style_block.as_ref().and_then(scoped_class);
            let to = new_ui.style_block.as_ref().and_then(scoped_class);
            if let (Some(block), Some(class)) = (&new_ui.style_block, &to) {
                rules.insert(class.clone(), rules_for(class, block, points));
            }
            if from != to {
                node.remove_class = from;
                node.add_class = to;
            }
            differs = true;
        }

        let (old_text, new_text) = (own_text(old_ui), own_text(new_ui));
        if old_text != new_text {
            let is_text = TEXTS.contains(&crate::print::component(&new_ui.component).as_str());
            match (is_text, old_text, new_text) {
                (true, Some(_), Some(text)) => node.text = Some(text.to_string()),
                _ => return rebuild("an argument other than an element's text changed"),
            }
            differs = true;
        }

        if differs {
            let owner = new_id.split(':').next().unwrap_or_default();
            changed.insert((owner, new_ui.span), old_ui);
            if node.remove_class.is_some() || node.add_class.is_some() || node.text.is_some() {
                patch.nodes.push(node);
            }
        }
    }

    // With each changed element put back as it was, the two programs must
    // read the same, or something else changed too.
    let mut put_back = after.clone();
    for decl in &mut put_back.declarations {
        let (owner, body) = match decl {
            Declaration::Page(p) => (p.name.clone(), &mut p.body),
            Declaration::Component(c) => (c.name.clone(), &mut c.body),
            Declaration::App(a) => ("App".to_string(), &mut a.body),
            _ => continue,
        };
        each_element_mut(body, &mut |ui| {
            if let Some(old) = changed.get(&(owner.as_str(), ui.span)) {
                ui.style_block = old.style_block.clone();
                // Its text alone, when that's what the patch carries: any
                // other argument changed still shows, a value read included.
                let first =
                    |args: &[Arg]| args.iter().position(|a| matches!(a, Arg::Positional(_)));
                if own_text(ui).is_some()
                    && own_text(old).is_some()
                    && let (Some(to), Some(from)) = (first(&ui.args), first(&old.args))
                {
                    ui.args[to] = old.args[from].clone();
                }
            }
        });
    }
    let reads = |p: &Program| -> Vec<String> {
        p.declarations
            .iter()
            .map(|d| crate::print::declaration(d, p))
            .collect()
    };
    if reads(&put_back) != reads(before) {
        return rebuild("something other than a style or a text changed");
    }

    if patch.nodes.is_empty() {
        return InPlace::Same;
    }
    patch.rules = rules.into_values().collect();
    InPlace::Patch(patch)
}

/// Every element, with the id a studio build stamps on it, in the walk's order.
fn elements(program: &Program) -> Vec<(String, &UIElement)> {
    let mut all = Vec::new();
    visit_nodes(program, &mut |ui, id, _| all.push((id.to_string(), ui)));
    all
}

/// An element's own text: its first positional argument, a plain string.
fn own_text(ui: &UIElement) -> Option<&str> {
    match ui.args.iter().find(|a| matches!(a, Arg::Positional(_))) {
        Some(Arg::Positional(Expr::StringLiteral(s))) => Some(s),
        _ => None,
    }
}

fn style_text(block: Option<&StyleBlock>) -> Option<String> {
    block.map(|b| crate::print::style_block(b, 0))
}

/// The declarations of a block that read a value, which stay on the element
/// and which no class can carry.
fn live_text(block: Option<&StyleBlock>) -> Vec<String> {
    block.map_or_else(Vec::new, |b| {
        b.properties
            .iter()
            .filter(|p| static_declaration(p).is_none())
            .map(crate::print::style_property)
            .collect()
    })
}

/// Whether a patch and a full build agree: `after`'s generated pages are
/// `before`'s with the patch applied, and its stylesheet rules are what the
/// patch adds. `Err` says how they differ.
pub fn agrees(before: &Program, after: &Program, patch: &Patch) -> Result<(), String> {
    for ssg in [false, true] {
        let old = generated(before, ssg);
        let new = generated(after, ssg);
        pages_agree(&old, &new, patch).map_err(|e| format!("ssg {ssg}: {e}"))?;
    }
    let old: BTreeSet<String> = scoped_rules(before).lines().map(str::to_string).collect();
    let new: BTreeSet<String> = scoped_rules(after).lines().map(str::to_string).collect();
    let mut needed: BTreeSet<&str> = new.iter().map(String::as_str).collect();
    for line in old.iter().map(String::as_str).chain(patch.rules.lines()) {
        needed.remove(line);
    }
    if let Some(line) = needed.iter().next() {
        return Err(format!(
            "a rule the build wrote and the patch did not: {line}"
        ));
    }
    for line in patch.rules.lines() {
        if !new.contains(line) {
            return Err(format!(
                "a rule the patch added and the build did not: {line}"
            ));
        }
    }
    Ok(())
}

fn generated(program: &Program, ssg: bool) -> String {
    let mut js = JsCodegen::new();
    js.set_ssg(ssg);
    js.set_studio(build_node_map(program));
    js.generate(program)
}

/// A line that gives an element its compiled style class.
fn is_class_line(line: &str) -> bool {
    line.contains(".classList.add(\"wf-s")
}

fn pages_agree(old: &str, new: &str, patch: &Patch) -> Result<(), String> {
    let classes = |js: &str| -> BTreeMap<String, usize> {
        let mut seen = BTreeMap::new();
        for line in js.lines().filter(|l| is_class_line(l)) {
            if let Some(class) = line.split('"').nth(1) {
                *seen.entry(class.to_string()).or_insert(0) += 1;
            }
        }
        seen
    };
    // What the patch takes off and puts on, to what the old page gives out.
    let mut expected = classes(old);
    for node in &patch.nodes {
        let uses = new_uses(old, node);
        if let Some(class) = &node.remove_class {
            match expected.get_mut(class) {
                Some(n) if *n >= uses => *n -= uses,
                _ => return Err(format!("{}: nothing to remove {class} from", node.node)),
            }
        }
        if let Some(class) = &node.add_class {
            *expected.entry(class.clone()).or_insert(0) += uses;
        }
    }
    expected.retain(|_, n| *n > 0);
    if expected != classes(new) {
        return Err(format!(
            "the classes differ: expected {expected:?}, built {:?}",
            classes(new)
        ));
    }

    let rest = |js: &str| -> Vec<String> {
        js.lines()
            .filter(|l| !is_class_line(l))
            .map(str::to_string)
            .collect()
    };
    let (old_lines, new_lines) = (rest(old), rest(new));
    if old_lines.len() != new_lines.len() {
        return Err(format!(
            "the build has {} lines of its own and the old page {}",
            new_lines.len(),
            old_lines.len()
        ));
    }
    for (a, b) in old_lines.iter().zip(&new_lines) {
        if a == b {
            continue;
        }
        let explained = patch.nodes.iter().any(|node| {
            let Some(text) = &node.text else { return false };
            let stamp = format!("\"data-wf-node\": {}", js_string(&node.node));
            b.contains(&stamp) && b.contains(&js_string(text)) && same_but_strings(a, b)
        });
        if !explained {
            return Err(format!("a line the patch does not explain:\n  {a}\n  {b}"));
        }
    }
    Ok(())
}

/// How many times the old page creates this element.
fn new_uses(old: &str, node: &NodePatch) -> usize {
    let stamp = format!("\"data-wf-node\": {}", js_string(&node.node));
    old.matches(&stamp).count()
}

fn js_string(text: &str) -> String {
    serde_json::to_string(text).unwrap_or_default()
}

/// Two lines that differ only inside one string.
fn same_but_strings(a: &str, b: &str) -> bool {
    let start = a.bytes().zip(b.bytes()).take_while(|(x, y)| x == y).count();
    let end = a
        .bytes()
        .rev()
        .zip(b.bytes().rev())
        .take_while(|(x, y)| x == y)
        .count();
    if start + end > a.len().min(b.len()) {
        return true;
    }
    let (mid_a, mid_b) = (&a[start..a.len() - end], &b[start..b.len() - end]);
    !mid_a.contains('"') && !mid_b.contains('"') || a[..start].matches('"').count() % 2 == 1
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_source;

    fn program(source: &str) -> Program {
        parse_source(source, "t.wf").unwrap()
    }

    const PAGE: &str = r#"page Home(path: "/", title: "Home") {
    state n = 3
    Stack {
        Heading("Hello").h1 {
            style {
                padding: 8px
            }
        }
        Text("Count {n}")
        Text("Plain")
        Button("Press") {
            on click {
                n = n + 1
            }
        }
    }
}
"#;

    #[test]
    fn braces_with_no_name_in_them_are_text_and_agree() {
        // The language reads `{1 + 1}` as words: only a name makes it a value.
        let a = program(PAGE);
        let b = program(&PAGE.replace("\"Plain\"", "\"Again {1 + 1}\""));
        let InPlace::Patch(p) = in_place(&a, &b) else {
            panic!("not a patch");
        };
        assert_eq!(p.nodes[0].text.as_deref(), Some("Again {1 + 1}"));
        agrees(&a, &b, &p).unwrap();
    }

    #[test]
    fn a_link_whose_text_and_address_both_change_is_a_rebuild() {
        let source = "page Home(path: \"/\", title: \"Home\") {\n    Link(\"A\", to: \"/\")\n}\n";
        let moved = source.replace("\"A\", to: \"/\"", "\"B\", to: \"/b\"");
        assert!(matches!(
            in_place(&program(source), &program(&moved)),
            InPlace::Rebuild { .. }
        ));
        let words = source.replace("\"A\"", "\"B\"");
        assert!(matches!(
            in_place(&program(source), &program(&words)),
            InPlace::Patch(_)
        ));
    }

    #[test]
    fn a_value_read_that_changes_beside_a_style_is_a_rebuild() {
        let source = "page Home(path: \"/\", title: \"Home\") {\n    state a = \"x\"\n    state b = \"y\"\n    Text(a) {\n        style {\n            margin-top: 1px\n        }\n    }\n}\n";
        let style = source.replace("margin-top: 1px", "margin-top: 2px");
        assert!(matches!(
            in_place(&program(source), &program(&style)),
            InPlace::Patch(_)
        ));
        let both = style.replace("Text(a)", "Text(b)");
        assert!(matches!(
            in_place(&program(source), &program(&both)),
            InPlace::Rebuild { .. }
        ));
    }

    #[test]
    fn nothing_changed_is_the_same() {
        let a = program(PAGE);
        assert_eq!(in_place(&a, &program(PAGE)), InPlace::Same);
    }

    #[test]
    fn a_comment_or_a_space_is_the_same() {
        let moved = PAGE.replace("    state n = 3\n", "    // A count.\n\n    state n = 3\n");
        assert_eq!(in_place(&program(PAGE), &program(&moved)), InPlace::Same);
    }

    #[test]
    fn a_style_is_one_class_and_its_rules() {
        let (a, b) = (program(PAGE), program(&PAGE.replace("8px", "12px")));
        let InPlace::Patch(p) = in_place(&a, &b) else {
            panic!("not a patch");
        };
        assert_eq!(p.nodes.len(), 1);
        assert_eq!(p.nodes[0].node, "Home:1.0");
        assert!(
            p.nodes[0]
                .remove_class
                .as_ref()
                .unwrap()
                .starts_with("wf-s")
        );
        let class = p.nodes[0].add_class.as_ref().unwrap();
        assert!(
            p.rules.contains(class) && p.rules.contains("12px"),
            "{}",
            p.rules
        );
        agrees(&a, &b, &p).unwrap();
    }

    #[test]
    fn a_style_added_and_a_style_taken_off() {
        let none = PAGE.replace(
            "            style {\n                padding: 8px\n            }\n",
            "",
        );
        let (a, b) = (program(&none), program(PAGE));
        let InPlace::Patch(p) = in_place(&a, &b) else {
            panic!("not a patch");
        };
        assert_eq!(p.nodes[0].remove_class, None);
        assert!(p.nodes[0].add_class.is_some());
        agrees(&a, &b, &p).unwrap();
        let InPlace::Patch(p) = in_place(&b, &a) else {
            panic!("not a patch");
        };
        assert_eq!(p.nodes[0].add_class, None);
        assert!(p.nodes[0].remove_class.is_some());
        agrees(&b, &a, &p).unwrap();
    }

    #[test]
    fn a_literal_text_is_its_string() {
        let (a, b) = (
            program(PAGE),
            program(&PAGE.replace("Plain", "Said \\\"plainly\\\"")),
        );
        let InPlace::Patch(p) = in_place(&a, &b) else {
            panic!("not a patch");
        };
        assert_eq!(p.nodes[0].text.as_deref(), Some("Said \"plainly\""));
        agrees(&a, &b, &p).unwrap();
    }

    #[test]
    fn what_a_patch_cannot_carry_is_a_rebuild() {
        let a = program(PAGE);
        for (what, edited) in [
            (
                "element",
                PAGE.replace("Text(\"Plain\")", "Text(\"Plain\")\n        Text(\"More\")"),
            ),
            ("reads", PAGE.replace("Count {n}", "Total {n}")),
            ("to reads", PAGE.replace("\"Plain\"", "\"Plain {n}\"")),
            ("flag", PAGE.replace(".h1", ".h2")),
            ("state", PAGE.replace("n = 3", "n = 4")),
            ("handler", PAGE.replace("n + 1", "n + 2")),
            ("kind", PAGE.replace("Text(\"Plain\")", "Badge(\"Plain\")")),
            ("live", PAGE.replace("padding: 8px", "padding: {n}px")),
        ] {
            assert!(
                matches!(in_place(&a, &program(&edited)), InPlace::Rebuild { .. }),
                "{what}"
            );
        }
    }

    #[test]
    fn a_style_beside_a_live_value_can_change_and_the_value_cannot() {
        let live = PAGE.replace("padding: 8px", "padding: 8px\n                width: {n}px");
        let (a, b) = (program(&live), program(&live.replace("8px", "9px")));
        let InPlace::Patch(p) = in_place(&a, &b) else {
            panic!("not a patch");
        };
        agrees(&a, &b, &p).unwrap();
        let moved = live.replace("{n}px", "{n + 1}px");
        assert!(matches!(
            in_place(&a, &program(&moved)),
            InPlace::Rebuild { .. }
        ));
    }

    #[test]
    fn an_element_in_a_component_is_every_use_of_it() {
        let source = r#"component Note() {
    Text("One")
}

page Home(path: "/", title: "Home") {
    Note()
    Note()
}
"#;
        let (a, b) = (program(source), program(&source.replace("One", "Two")));
        let InPlace::Patch(p) = in_place(&a, &b) else {
            panic!("not a patch");
        };
        assert_eq!(p.nodes[0].node, "Note:0");
        agrees(&a, &b, &p).unwrap();
    }

    #[test]
    fn agreeing_notices_a_patch_that_is_not_enough() {
        let (a, b) = (program(PAGE), program(&PAGE.replace("Plain", "Other")));
        let wrong = Patch::default();
        assert!(agrees(&a, &b, &wrong).is_err());
        let (c, d) = (program(PAGE), program(&PAGE.replace("8px", "12px")));
        assert!(agrees(&c, &d, &Patch::default()).is_err());
    }
}
