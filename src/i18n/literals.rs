//! Literal keys: a project whose `i18n` sets `"literals": true` is
//! translated by its own words. Every string a reader sees — an element's
//! text, and its `label:`, `title:`, `placeholder:`, `aria-label:` … — is
//! looked up in the locale's table by the text as written, so the source
//! stays in the language it was written in and the table maps that text to
//! each other language: `"Problems": "المشاكل"`. A string that splices
//! plain paths is one key with a placeholder for each,
//! `"{count} missing"`, filled as `t()` fills one. A string the table has
//! not got shows as written.
//!
//! [`translate`] rewrites those strings as the `t()` calls they stand for,
//! just before code generation, so the checks read the source as written;
//! [`uses`] lists them for the translations table, as `t()` calls are.

use crate::parser::ast::{
    Arg, Declaration, Expr, Program, Span, Statement, StatementKind, StringPart, UIElement,
};

/// The named arguments whose value a reader sees or hears.
pub const TEXT_PROPS: &[&str] = &[
    "label",
    "title",
    "placeholder",
    "aria-label",
    "alt",
    "text",
    "caption",
    "summary",
    "description",
    "detail",
    "message",
    "hint",
    "heading",
    "subtitle",
    "tooltip",
    "trigger",
    "emptyText",
    "why",
    // A list of words, one for each of a control's options.
    "labels",
];

/// The endings of a `derived` (or `state`) name that say it is words a
/// reader sees: `caretText`, `emptyLabel`, `word`.
pub const TEXT_NAMES: &[&str] = &[
    "Text", "Label", "Title", "Word", "Words", "Note", "Hint", "Message", "Summary", "Lead",
    "Says", "Phrase", "Caption", "Heading", "Tooltip",
];

/// Whether a declared name says it holds words a reader sees.
pub fn text_name(name: &str) -> bool {
    TEXT_NAMES
        .iter()
        .any(|end| name.ends_with(end) || name == end.to_lowercase())
}

/// The elements whose first argument is not words: an icon's name, code, a
/// key, a source.
pub const NOT_WORDS: &[&str] = &[
    "Icon", "Code", "Kbd", "Image", "Picture", "Video", "Audio", "Svg", "Script", "Style",
    "Children", "Element", "Host", "Embed", "Iframe",
];

/// The name a placeholder is given for a spliced path: its last part.
fn name_of(e: &Expr) -> Option<String> {
    match e {
        Expr::Identifier(n) => Some(n.clone()),
        Expr::PropertyAccess(base, n) | Expr::OptionalProperty(base, n) => {
            name_of(base)?;
            Some(n.clone())
        }
        // A path with what it falls back to: `{item?.name ?? ""}`.
        Expr::BinaryOp(left, crate::parser::ast::BinOp::NullCoalesce, right)
            if matches!(**right, Expr::StringLiteral(_) | Expr::NumberLiteral(_)) =>
        {
            name_of(left)
        }
        _ => None,
    }
}

/// Whether a piece of text is prose a reader reads, not a name or a code
/// word: `Close`, `about `, `used in`, but not `rtl`, `main` or
/// `webfluent.app.json`.
fn prose(t: &str) -> bool {
    let letters = t.chars().filter(|c| c.is_alphabetic()).count();
    let capital = t
        .trim_start()
        .chars()
        .next()
        .is_some_and(|c| c.is_uppercase())
        && t.chars().any(|c| c.is_lowercase());
    letters >= 1 && (t.contains(char::is_whitespace) || capital || !t.is_ascii())
}

/// The key a string is looked up by, and what fills each placeholder:
/// `"Problems"`, or `"{count} missing"` with `count`. None for a string
/// that is not words, or that splices more than a plain path.
pub fn key_of(e: &Expr) -> Option<(String, Vec<(String, Expr)>)> {
    let words = |s: &str| s.chars().any(|c| c.is_alphabetic());
    match e {
        Expr::StringLiteral(s) if words(s) => Some((s.clone(), Vec::new())),
        Expr::InterpolatedString(parts) => {
            let mut key = String::new();
            let mut said = String::new();
            let mut args: Vec<(String, Expr)> = Vec::new();
            for p in parts {
                match p {
                    StringPart::Literal(t) => {
                        // A brace in the words would read as a placeholder.
                        if t.contains(['{', '}']) {
                            return None;
                        }
                        key.push_str(t);
                        said.push_str(t);
                    }
                    StringPart::Expression(x) => {
                        let base = name_of(x)?;
                        // Two paths with one last part: the second numbered.
                        let mut name = base.clone();
                        let mut n = 2;
                        while args.iter().any(|(a, v)| {
                            *a == name && crate::print::expr(v) != crate::print::expr(x)
                        }) {
                            name = format!("{base}{n}");
                            n += 1;
                        }
                        if !args.iter().any(|(a, _)| *a == name) {
                            args.push((name.clone(), x.clone()));
                        }
                        key.push('{');
                        key.push_str(&name);
                        key.push('}');
                    }
                }
            }
            words(&said).then_some((key, args))
        }
        _ => None,
    }
}

/// The `t()` call a string stands for.
fn call_of(key: String, args: Vec<(String, Expr)>) -> Expr {
    let mut call = vec![Expr::StringLiteral(key)];
    if !args.is_empty() {
        call.push(Expr::MapLiteral(args));
    }
    Expr::FunctionCall("t".into(), call)
}

/// Each string a reader sees in `e`: `e` itself, or the branches of an
/// `if` it chooses between, or what `??` falls back to.
fn each_shown(e: &mut Expr, f: &mut dyn FnMut(&mut Expr)) {
    match e {
        Expr::MethodCall(_, m, args) if m == "__if" => {
            for a in args {
                each_shown(a, f);
            }
        }
        Expr::MethodCall(_, m, args) if m == "__iflet" => {
            for a in args {
                match a {
                    Expr::Lambda(_, body) => each_shown(body, f),
                    other => each_shown(other, f),
                }
            }
        }
        Expr::BinaryOp(_, crate::parser::ast::BinOp::NullCoalesce, right) => each_shown(right, f),
        // `labels: ["Light", "Dark"]`: each a string a reader sees.
        Expr::ListLiteral(items) => {
            for item in items {
                each_shown(item, f);
            }
        }
        other => f(other),
    }
}

fn element_name(ui: &UIElement) -> &str {
    match &ui.component {
        crate::parser::ast::ComponentRef::BuiltIn(n)
        | crate::parser::ast::ComponentRef::UserDefined(n) => n,
        crate::parser::ast::ComponentRef::SubComponent(_, n) => n,
    }
}

/// The arguments of `ui` a reader sees: its first positional one, unless it
/// is an icon's name or code, and the named ones in [`TEXT_PROPS`].
fn each_text_arg(ui: &mut UIElement, f: &mut dyn FnMut(&mut Expr)) {
    let words_first = !NOT_WORDS.contains(&element_name(ui));
    let mut first = true;
    for a in &mut ui.args {
        match a {
            Arg::Positional(e) => {
                if first && words_first {
                    each_shown(e, f);
                }
                first = false;
            }
            Arg::Named(k, e) if TEXT_PROPS.contains(&k.as_str()) => each_shown(e, f),
            _ => {}
        }
    }
}

fn each_element(body: &mut [Statement], f: &mut dyn FnMut(&mut UIElement, Span)) {
    for s in body {
        let span = s.span;
        if let StatementKind::UIElement(ui) = &mut s.kind {
            f(ui, span);
            each_element(&mut ui.children, f);
            for fill in &mut ui.slot_fills {
                each_element(&mut fill.body, f);
            }
        } else {
            for b in s.kind.bodies_mut() {
                each_element(b, f);
            }
        }
    }
}

fn bodies(program: &mut Program) -> Vec<(usize, &mut Vec<Statement>)> {
    program
        .declarations
        .iter_mut()
        .enumerate()
        .filter_map(|(i, d)| match d {
            Declaration::Page(p) => Some((i, &mut p.body)),
            Declaration::Component(c) => Some((i, &mut c.body)),
            Declaration::Store(s) => Some((i, &mut s.body)),
            Declaration::App(a) => Some((i, &mut a.body)),
            _ => None,
        })
        .collect()
}

/// In `e`, each value a record or a map gives a field a reader sees:
/// `RailPanel(label: "Layers")`, `{ id: "html", label: "HTML" }`.
fn each_field(e: &mut Expr, f: &mut dyn FnMut(&mut Expr)) {
    e.walk_mut(&mut |x| {
        if let Expr::Record(_, pairs) | Expr::MapLiteral(pairs) = x {
            for (k, v) in pairs {
                if TEXT_PROPS.contains(&k.trim_matches('"')) {
                    each_shown(v, f);
                }
            }
        }
    });
}

/// Every string a reader sees in the program, with where it is written:
/// an element's text and its text props; a text field of a record or a
/// map anywhere a value is declared or passed; a value whose name says it
/// is words.
fn each_text(program: &mut Program, f: &mut dyn FnMut(&mut Expr, usize, Span)) {
    fn values(body: &mut [Statement], i: usize, f: &mut dyn FnMut(&mut Expr, usize, Span)) {
        for s in body {
            let span = s.span;
            match &mut s.kind {
                StatementKind::State(d) => {
                    if text_name(&d.name) {
                        each_shown(&mut d.value, &mut |e| f(e, i, span));
                    }
                    each_field(&mut d.value, &mut |e| f(e, i, span));
                }
                StatementKind::Derived(d) => {
                    if text_name(&d.name) {
                        each_shown(&mut d.value, &mut |e| f(e, i, span));
                    }
                    each_field(&mut d.value, &mut |e| f(e, i, span));
                }
                // `wrote = "Wrote {n} changes."` in an action, to a value
                // whose name says it is words.
                StatementKind::Assignment(a) => {
                    if target_name(&a.target).is_some_and(text_name) {
                        each_shown(&mut a.value, &mut |e| f(e, i, span));
                    }
                    each_field(&mut a.value, &mut |e| f(e, i, span));
                }
                StatementKind::Return(Some(e)) => each_field(e, &mut |x| f(x, i, span)),
                StatementKind::UIElement(ui) => {
                    for a in &mut ui.args {
                        match a {
                            Arg::Positional(e) | Arg::Named(_, e) => {
                                each_field(e, &mut |x| f(x, i, span))
                            }
                        }
                    }
                    values(&mut ui.children, i, f);
                    for fill in &mut ui.slot_fills {
                        values(&mut fill.body, i, f);
                    }
                }
                other => {
                    for b in other.bodies_mut() {
                        values(b, i, f);
                    }
                }
            }
        }
    }
    for (i, body) in bodies(program) {
        each_element(body, &mut |ui, span| {
            each_text_arg(ui, &mut |e| f(e, i, span))
        });
        values(body, i, f);
    }
}

/// The name an assignment writes: `wrote`, or `note` of `self.note`.
fn target_name(e: &Expr) -> Option<&str> {
    match e {
        Expr::Identifier(n) => Some(n),
        Expr::PropertyAccess(_, n) | Expr::OptionalProperty(_, n) => Some(n),
        _ => None,
    }
}

/// Whether `e` holds a string with words in it anywhere.
fn holds_words(e: &Expr) -> bool {
    let mut found = false;
    e.clone().walk_mut(&mut |x| match x {
        Expr::StringLiteral(t) if prose(t) => found = true,
        Expr::InterpolatedString(parts) => {
            found |= parts
                .iter()
                .any(|p| matches!(p, StringPart::Literal(t) if prose(t)))
        }
        _ => {}
    });
    found
}

/// Why words a reader sees are not translated.
#[derive(Debug, Clone, PartialEq)]
pub enum Untranslated {
    /// Text that splices more than a plain path: `{plural(n, "file")}`.
    Splices(String),
    /// A value shown by its name, whose name does not say it is words, so
    /// the strings it holds are not looked up: `said`.
    Unnamed(String),
}

/// What a reader sees that no table translates, with where it is shown.
pub fn untranslated(
    program: &Program,
    file_of: &dyn Fn(usize) -> String,
) -> Vec<(String, Span, Untranslated)> {
    // The values each body declares or writes words into, whose names do
    // not say they are words; a store's by `Store.member` as well.
    fn wordy(body: &[Statement], out: &mut std::collections::BTreeSet<String>) {
        for s in body {
            match &s.kind {
                StatementKind::State(d) if !text_name(&d.name) && holds_words(&d.value) => {
                    out.insert(d.name.clone());
                }
                StatementKind::Derived(d) if !text_name(&d.name) && holds_words(&d.value) => {
                    out.insert(d.name.clone());
                }
                StatementKind::Assignment(a) => {
                    if let Some(n) = target_name(&a.target)
                        && !text_name(n)
                        && holds_words(&a.value)
                    {
                        out.insert(n.to_string());
                    }
                }
                _ => {}
            }
            let mut kind = s.kind.clone();
            for b in kind.bodies_mut() {
                wordy(b, out);
            }
        }
    }
    let mut copy = program.clone();
    let mut by_body: std::collections::HashMap<usize, std::collections::BTreeSet<String>> =
        std::collections::HashMap::new();
    let mut stores = std::collections::BTreeSet::new();
    for (i, body) in bodies(&mut copy) {
        let mut names = std::collections::BTreeSet::new();
        wordy(body, &mut names);
        by_body.insert(i, names);
    }
    for (i, d) in program.declarations.iter().enumerate() {
        if let Declaration::Store(st) = d {
            for n in by_body.get(&i).into_iter().flatten() {
                stores.insert(format!("{}.{n}", st.name));
            }
        }
    }
    let unnamed = |x: &Expr, i: usize| -> Option<String> {
        match x {
            Expr::Identifier(n) if by_body.get(&i).is_some_and(|s| s.contains(n)) => {
                Some(n.clone())
            }
            Expr::PropertyAccess(base, m) | Expr::OptionalProperty(base, m) => match &**base {
                Expr::Identifier(st) if stores.contains(&format!("{st}.{m}")) => {
                    Some(format!("{st}.{m}"))
                }
                _ => None,
            },
            _ => None,
        }
    };
    let mut out = Vec::new();
    each_text(&mut copy, &mut |e, i, span| {
        let why = match e {
            Expr::InterpolatedString(parts) => {
                let words = parts
                    .iter()
                    .any(|p| matches!(p, StringPart::Literal(t) if prose(t)));
                parts.iter().find_map(|p| match p {
                    StringPart::Expression(x) if words && name_of(x).is_none() => {
                        Some(Untranslated::Splices(crate::print::expr(x)))
                    }
                    StringPart::Expression(x) => unnamed(x, i).map(Untranslated::Unnamed),
                    _ => None,
                })
            }
            other => unnamed(other, i).map(Untranslated::Unnamed),
        };
        if let Some(why) = why {
            out.push((file_of(i), span, why));
        }
    });
    out.sort_by(|a, b| (&a.0, a.1.line, a.1.col).cmp(&(&b.0, b.1.line, b.1.col)));
    out.dedup();
    out
}

/// Every string a reader sees, as the `t()` call it stands for.
pub fn translate(program: &mut Program) {
    each_text(program, &mut |e, _, _| {
        if let Some((key, args)) = key_of(e) {
            *e = call_of(key, args);
        }
    });
}

/// Every string a reader sees, as a use of its key.
pub fn uses(program: &Program, file_of: &dyn Fn(usize) -> String) -> Vec<super::table::Use> {
    let mut copy = program.clone();
    let mut out = Vec::new();
    each_text(&mut copy, &mut |e, i, span| {
        if let Some((key, args)) = key_of(e) {
            out.push(super::table::Use {
                key,
                dynamic: false,
                passes: Some(args.into_iter().map(|(n, _)| n).collect()),
                file: file_of(i),
                line: span.line,
                col: span.col,
            });
        }
    });
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn page(src: &str) -> Program {
        crate::syntax::parse_source(src, "src/pages/p.wf").unwrap()
    }

    fn printed(p: &Program) -> String {
        let Declaration::Page(page) = &p.declarations[0] else {
            panic!()
        };
        page.body
            .iter()
            .filter_map(|s| match &s.kind {
                StatementKind::UIElement(ui) => Some(crate::print::element(ui, 0)),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn expr(src: &str) -> Expr {
        let p = page(&format!(
            "page P(path: \"/\") {{\n    state n = 1\n    Text({src})\n}}\n"
        ));
        let Declaration::Page(page) = &p.declarations[0] else {
            panic!()
        };
        let StatementKind::UIElement(ui) = &page.body[1].kind else {
            panic!()
        };
        let Arg::Positional(e) = &ui.args[0] else {
            panic!()
        };
        e.clone()
    }

    #[test]
    fn a_text_and_its_label_are_looked_up_by_their_words() {
        let mut p = page(
            "page P(path: \"/\") {\n    Heading(\"Problems\").h2\n    Button(\"Save\", aria-label: \"Save the file\", class: \"primary\")\n    Icon(\"warning\")\n    Code(\"src/app.wf\")\n}\n",
        );
        translate(&mut p);
        let out = printed(&p);
        assert!(out.contains("Heading(t(\"Problems\")).h2"), "{out}");
        assert!(
            out.contains(
                "Button(t(\"Save\"), aria-label: t(\"Save the file\"), class: \"primary\")"
            ),
            "{out}"
        );
        assert!(
            out.contains("Icon(\"warning\")") && out.contains("Code(\"src/app.wf\")"),
            "{out}"
        );
    }

    #[test]
    fn a_component_s_detail_is_words_too() {
        let mut p = page(
            "page P(path: \"/\") {\n    Setting(label: \"Wrap\", detail: \"Long lines wrap.\")\n}\n",
        );
        translate(&mut p);
        let out = printed(&p);
        assert!(out.contains("detail: t(\"Long lines wrap.\")"), "{out}");
    }

    #[test]
    fn a_string_that_splices_paths_is_one_key_with_placeholders() {
        let (key, args) = key_of(&expr("\"{l.missing} missing in {l.code}\"")).unwrap();
        assert_eq!(key, "{missing} missing in {code}");
        assert_eq!(
            args.iter()
                .map(|(n, e)| (n.as_str(), crate::print::expr(e)))
                .collect::<Vec<_>>(),
            [
                ("missing", "l.missing".to_string()),
                ("code", "l.code".to_string())
            ]
        );
        assert!(
            key_of(&expr("\"{n + 1} more\"")).is_none(),
            "more than a path stays as written"
        );
        assert!(key_of(&expr("\"{a}:{b}\"")).is_none(), "no words");
        assert!(key_of(&expr("\"42\"")).is_none());
        let (k, _) = key_of(&expr("\"{a.name} and {b.name}\"")).unwrap();
        assert_eq!(k, "{name} and {name2}");
    }

    #[test]
    fn a_path_with_its_fallback_is_named_by_the_path() {
        let (key, args) = key_of(&expr("\"Delete {item?.name ?? \\\"\\\"}?\"")).unwrap();
        assert_eq!(key, "Delete {name}?");
        assert_eq!(crate::print::expr(&args[0].1), "item?.name ?? \"\"");
        assert!(
            key_of(&expr("\"{a ?? b} left\"")).is_none(),
            "a fallback that is not a value"
        );
    }

    #[test]
    fn words_written_in_an_action_to_a_value_named_as_words_are_looked_up() {
        let mut p = page(
            "store S {\n    state wroteNote = \"\"\n    state kind = \"\"\n    action save(n: Number) {\n        wroteNote = \"Wrote {n} changes.\"\n        kind = \"Kept as it is\"\n    }\n}\n",
        );
        translate(&mut p);
        let Declaration::Store(st) = &p.declarations[0] else {
            panic!()
        };
        let out = format!("{:?}", st.body);
        assert!(out.contains("Wrote {n} changes."), "{out}");
        assert!(
            out.contains("FunctionCall(\"t\", [StringLiteral(\"Wrote {n} changes.\")"),
            "{out}"
        );
        assert!(
            !out.contains("FunctionCall(\"t\", [StringLiteral(\"Kept as it is\")"),
            "{out}"
        );
    }

    #[test]
    fn words_no_table_can_translate_are_found_where_they_are_shown() {
        let p = page(
            "page P(path: \"/\") {\n    state n = 2\n    derived said = \"{n} errors\"\n    derived saidText = \"{n} errors\"\n    derived dir = if n > 1 { \"rtl\" } else { \"ltr\" }\n    Text(said)\n    Text(saidText)\n    Text(dir)\n    Text(\"Found {n + 1} more\")\n    Text(\"{n + 1}:{n}\")\n}\n",
        );
        let found: Vec<(u32, Untranslated)> = untranslated(&p, &|_| "src/pages/p.wf".into())
            .into_iter()
            .map(|(_, span, why)| (span.line, why))
            .collect();
        assert_eq!(
            found,
            [
                (6, Untranslated::Unnamed("said".into())),
                (9, Untranslated::Splices("n + 1".into())),
            ]
        );
    }

    #[test]
    fn a_list_of_labels_is_a_list_of_words() {
        let mut p = page(
            "page P(path: \"/\") {\n    Segmented(chosen: \"a\", options: [\"a\", \"b\"], labels: [\"Light\", \"Dark\"])\n}\n",
        );
        translate(&mut p);
        let out = printed(&p);
        assert!(
            out.contains("t(\"Light\")") && out.contains("t(\"Dark\")"),
            "{out}"
        );
        assert!(!out.contains("t(\"a\")"), "options are values: {out}");
    }

    #[test]
    fn markdown_written_in_place_is_words_too() {
        let mut p = page("page P(path: \"/\") {\n    Markdown(\"Read **this** first.\")\n}\n");
        translate(&mut p);
        let out = printed(&p);
        assert!(out.contains("t(\"Read **this** first.\")"), "{out}");
    }

    #[test]
    fn the_branches_a_text_chooses_between_are_each_looked_up() {
        let mut p = page(
            "page P(path: \"/\") {\n    state on = true\n    Text(if on { \"Open\" } else { \"Closed\" })\n}\n",
        );
        translate(&mut p);
        let out = printed(&p);
        assert!(
            out.contains("t(\"Open\")") && out.contains("t(\"Closed\")"),
            "{out}"
        );
    }

    #[test]
    fn its_uses_are_listed_with_their_places() {
        let p = page(
            "page P(path: \"/\") {\n    state n = 1\n    Text(\"{n} missing\")\n    Text(\"Ready\")\n}\n",
        );
        let u = uses(&p, &|_| "src/pages/p.wf".into());
        assert_eq!(
            u.iter()
                .map(|u| (u.key.as_str(), u.line))
                .collect::<Vec<_>>(),
            [("{n} missing", 3), ("Ready", 4)]
        );
        assert_eq!(u[0].passes, Some(vec!["n".to_string()]));
    }

    #[test]
    fn a_text_field_of_a_record_and_a_value_named_as_words() {
        let mut p = crate::syntax::parse_source("type Tab { id: String, label: String }\nstore Ed {\n    state line = 1\n    derived caretText = \"Ln {line}\"\n    derived mode = \"code\"\n    derived tabs: [Tab] = [Tab(id: \"problems\", label: \"Problems\")]\n}\n", "src/stores/ed.wf").unwrap();
        translate(&mut p);
        let Declaration::Store(st) = &p.declarations[1] else {
            panic!()
        };
        let shown: Vec<String> = st
            .body
            .iter()
            .filter_map(|s| match &s.kind {
                StatementKind::Derived(d) => Some(crate::print::expr(&d.value)),
                _ => None,
            })
            .collect();
        assert_eq!(shown[0], "t(\"Ln \\{line\\}\", line: line)");
        assert_eq!(shown[1], "\"code\"", "a value not named as words is left");
        assert_eq!(shown[2], "[Tab(id: \"problems\", label: t(\"Problems\"))]");
    }
}
