use std::collections::HashMap;

use crate::error::A11yWarning;
use crate::parser::ast::*;

/// Run all accessibility lint rules on the parsed program.
/// Returns a list of warnings (non-fatal).
///
/// Warnings name files by the project convention (`src/pages/<Name>.wf`);
/// a caller that merged several files knows the real ones and should use
/// [`lint_accessibility_in`].
pub fn lint_accessibility(program: &Program) -> Vec<A11yWarning> {
    lint_accessibility_in(program, &|index| match &program.declarations[index] {
        Declaration::Page(page) => format!("src/pages/{}.wf", page.name),
        Declaration::Component(comp) => format!("src/components/{}.wf", comp.name),
        _ => "src/App.wf".to_string(),
    })
}

/// [`lint_accessibility`] for a program merged from several files: `file_of`
/// names the file the declaration at that index came from, so a warning
/// points at the source a reader can open.
pub fn lint_accessibility_in(
    program: &Program,
    file_of: &dyn Fn(usize) -> String,
) -> Vec<A11yWarning> {
    let mut warnings = Vec::new();
    // A page's outline includes the headings of the components it calls, so
    // the outline check reads through a call to the component's body.
    let components: std::collections::HashMap<&str, &ComponentDecl> = program
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Component(c) => Some((c.name.as_str(), c)),
            _ => None,
        })
        .collect();

    for (index, decl) in program.declarations.iter().enumerate() {
        match decl {
            Declaration::Page(page) => {
                lint_page(page, &file_of(index), &mut warnings, &components);
            }
            // A component's or the app's body calls components too, and the
            // rules that read through a call — the roles a structure holds,
            // the text a control shows — need them here as on a page.
            Declaration::Component(comp) => {
                lint_statements(
                    &comp.body,
                    &file_of(index),
                    &mut warnings,
                    &mut HeadingTracker::with_components(&components),
                );
            }
            Declaration::App(app) => {
                lint_statements(
                    &app.body,
                    &file_of(index),
                    &mut warnings,
                    &mut HeadingTracker::with_components(&components),
                );
            }
            // Neither holds UI.
            Declaration::Store(_) | Declaration::Theme(_) => {}
        }
    }

    warnings.extend(lint_seo(program, file_of));
    warnings
}

/// Findings a search engine or a link preview would act on.
///
/// These sit with the accessibility rules because they share a root: a page a
/// crawler cannot summarise is usually a page a screen reader cannot either. A
/// missing description is a search result whose snippet is written by whatever
/// crawled it; a duplicate route is two pages competing for one ranking.
fn lint_seo(program: &Program, file_of: &dyn Fn(usize) -> String) -> Vec<A11yWarning> {
    let mut warnings = Vec::new();
    let mut seen_paths: Vec<(&str, &str)> = Vec::new();

    for (index, decl) in program.declarations.iter().enumerate() {
        let Declaration::Page(page) = decl else {
            continue;
        };
        let file = file_of(index);
        // These are findings about the page as a whole, so they point at its
        // header — where the title and description are written.
        let (line, col) = (
            page.header_span.line.max(1) as usize,
            page.header_span.col.max(1) as usize,
        );

        // S01: a page with no title has nothing to show as a search result link.
        if page.title.as_deref().unwrap_or("").trim().is_empty() {
            warnings.push(A11yWarning::new(
                "S01",
                format!("Page {} has no title", page.name),
                &file,
                line,
                col,
                "Add one: Page Name (path: \"/\", title: \"What this page is\")",
            ));
        }

        // S02: no description means the snippet is written for you.
        if page.description.is_none() && !page.noindex {
            warnings.push(A11yWarning::new(
                "S02",
                format!("Page {} has no description", page.name),
                &file,
                line,
                col,
                "Add one: Page Name (path: \"/\", title: \"…\", description: \"A sentence a search result can show\")",
            ));
        }

        // S03: over ~160 characters a description is truncated mid-sentence.
        if let Some(d) = &page.description {
            let len = d.chars().count();
            if len > 160 {
                warnings.push(A11yWarning::new(
                    "S03",
                    format!(
                        "Page {}'s description is {len} characters; a search result shows about 160",
                        page.name
                    ),
                    &file,
                    line,
                    col,
                    "Shorten it, or accept that it will be cut mid-sentence",
                ));
            }
        }

        // S04: two pages on one route is a ranking split and an ambiguous build.
        if let Some((other, _)) = seen_paths.iter().find(|(p, _)| *p == page.path) {
            warnings.push(A11yWarning::new(
                "S04",
                format!(
                    "Pages {} and {} both claim the route {}",
                    other, page.name, page.path
                ),
                &file,
                line,
                col,
                "Give each page its own path",
            ));
        }
        seen_paths.push((&page.name, &page.path));
    }

    warnings
}

/// Track heading levels within a page for skip detection.
struct HeadingTracker {
    levels_seen: Vec<u8>,
    h1_count: usize,
    /// Whether the document-outline rules (A11, A12) apply here.
    ///
    /// They are rules about one HTML page: exactly one `h1`, no skipped levels.
    /// A slide deck has an `h1` per slide and a paginated document restarts its
    /// hierarchy per section, so neither rule is meaningful for them — and both
    /// `wf init -t slides` and `wf init -t pdf` produced a scaffold that warned
    /// on its own first build.
    checks_outline: bool,
    /// The program's components by name (see [`Self::component`]).
    components: std::collections::HashMap<String, ComponentDecl>,
    /// Components being expanded, so a component that calls itself stops.
    expanding: Vec<String>,
}

impl HeadingTracker {
    fn new() -> Self {
        Self {
            levels_seen: Vec::new(),
            h1_count: 0,
            checks_outline: true,
            components: std::collections::HashMap::new(),
            expanding: Vec::new(),
        }
    }

    /// A tracker that knows the program's components.
    fn with_components(components: &std::collections::HashMap<&str, &ComponentDecl>) -> Self {
        let mut tracker = Self::new();
        tracker.components = components
            .iter()
            .map(|(k, v)| (k.to_string(), (*v).clone()))
            .collect();
        tracker
    }

    /// A tracker for output that is not one HTML page — a deck or a paginated
    /// document. Every other accessibility rule still runs.
    fn without_outline_checks() -> Self {
        Self {
            checks_outline: false,
            ..Self::new()
        }
    }

    /// The program's components, so a call to one contributes the headings
    /// of its body to the outline being checked. Empty when linting a
    /// component or the app on its own.
    fn component(&self, name: &str) -> Option<ComponentDecl> {
        self.components.get(name).cloned()
    }

    fn record(&mut self, level: u8) {
        if level == 1 {
            self.h1_count += 1;
        }
        self.levels_seen.push(level);
    }

    fn last_level(&self) -> Option<u8> {
        self.levels_seen.last().copied()
    }
}

/// Whether a page's body is a slide deck or a paginated document rather than an
/// HTML page.
fn is_document_or_deck(body: &[Statement]) -> bool {
    body.iter().any(|stmt| {
        let StatementKind::UIElement(ui) = &stmt.kind else {
            return false;
        };
        matches!(&ui.component, ComponentRef::BuiltIn(n)
            if matches!(n.as_str(), "Presentation" | "Document"))
    })
}

fn lint_page(
    page: &PageDecl,
    file: &str,
    warnings: &mut Vec<A11yWarning>,
    components: &std::collections::HashMap<&str, &ComponentDecl>,
) {
    let mut tracker = if is_document_or_deck(&page.body) {
        HeadingTracker::without_outline_checks()
    } else {
        HeadingTracker::new()
    };
    tracker.components = HeadingTracker::with_components(components).components;
    lint_statements(&page.body, file, warnings, &mut tracker);

    if !tracker.checks_outline {
        return;
    }

    // A12: Page should have exactly one h1
    if tracker.h1_count == 0 {
        warnings.push(A11yWarning::new(
            "A12",
            "Page has no h1 heading",
            file,
            1,
            1,
            "Add a main heading: Heading(\"Page Title\", h1)".to_string(),
        ));
    } else if tracker.h1_count > 1 {
        warnings.push(A11yWarning::new(
            "A12",
            format!(
                "Page has {} h1 headings (should be exactly 1)",
                tracker.h1_count
            ),
            file,
            1,
            1,
            "Each page should have a single h1 as the main title".to_string(),
        ));
    }
}

fn lint_statements(
    stmts: &[Statement],
    file: &str,
    warnings: &mut Vec<A11yWarning>,
    heading_tracker: &mut HeadingTracker,
) {
    for stmt in stmts {
        match &stmt.kind {
            StatementKind::UIElement(ui) => lint_ui_element(ui, file, warnings, heading_tracker),
            StatementKind::If(if_stmt) => {
                lint_statements(&if_stmt.then_body, file, warnings, heading_tracker);
                for (_, body) in &if_stmt.else_if_branches {
                    lint_statements(body, file, warnings, heading_tracker);
                }
                if let Some(else_body) = &if_stmt.else_body {
                    lint_statements(else_body, file, warnings, heading_tracker);
                }
            }
            StatementKind::For(for_stmt) => {
                lint_statements(&for_stmt.body, file, warnings, heading_tracker);
            }
            StatementKind::Show(show_stmt) => {
                lint_statements(&show_stmt.body, file, warnings, heading_tracker);
            }
            StatementKind::Fetch(fetch) => {
                if let Some(loading) = &fetch.loading_block {
                    lint_statements(loading, file, warnings, heading_tracker);
                }
                if let Some((_, error_body)) = &fetch.error_block {
                    lint_statements(error_body, file, warnings, heading_tracker);
                }
                if let Some(success) = &fetch.success_block {
                    lint_statements(success, file, warnings, heading_tracker);
                }
            }
            _ => {}
        }
    }
}

fn lint_ui_element(
    ui: &UIElement,
    file: &str,
    warnings: &mut Vec<A11yWarning>,
    heading_tracker: &mut HeadingTracker,
) {
    // The element's own position: the parser records a span for every
    // element, so a warning lands on the line that needs the fix rather
    // than on line 1.
    let (line, col) = (ui.span.line.max(1) as usize, ui.span.col.max(1) as usize);

    if let ComponentRef::BuiltIn(name) = &ui.component {
        match name.as_str() {
            // A01: Image missing alt
            "Image" => {
                if !has_named_arg(&ui.args, "alt") {
                    warnings.push(A11yWarning::new(
                        "A01",
                        "Image missing \"alt\" attribute",
                        file,
                        line,
                        col,
                        "Add alt text: Image(src: \"...\", alt: \"Description of image\")",
                    ));
                }
            }

            // A02: IconButton missing accessible label
            "IconButton" => {
                if !has_named_arg(&ui.args, "label") && !has_positional_arg(&ui.args) {
                    warnings.push(A11yWarning::new(
                        "A02",
                        "IconButton missing accessible label",
                        file,
                        line,
                        col,
                        "Add a label: IconButton(icon: \"close\", label: \"Close dialog\")",
                    ));
                }
            }

            // A03: Input missing label
            "Input" => {
                if !has_accessible_name(&ui.args) && !has_named_arg(&ui.args, "placeholder") {
                    warnings.push(A11yWarning::new(
                        "A03",
                        "Input missing \"label\" or \"placeholder\" attribute",
                        file,
                        line,
                        col,
                        "Add a label: Input(text, label: \"Username\")",
                    ));
                }
            }

            // A04: Form control missing label
            "Checkbox" | "Radio" | "Switch" | "Slider" => {
                if !has_accessible_name(&ui.args) {
                    warnings.push(A11yWarning::new(
                        "A04",
                        format!("{} missing \"label\" attribute", name),
                        file,
                        line,
                        col,
                        format!("Add a label: {}(bind: value, label: \"Description\")", name),
                    ));
                }
            }

            // A05: Button has no text content
            "Button" => {
                if !has_positional_arg(&ui.args) && !has_accessible_name(&ui.args) {
                    warnings.push(A11yWarning::new(
                        "A05",
                        "Button has no text content",
                        file,
                        line,
                        col,
                        "Add text: Button(\"Save\", primary)",
                    ));
                }
            }

            // A06: Link has no text content
            "Link" => {
                // `Link("About", to: "/about")` puts the text in a positional
                // argument, exactly as `Button("Save")` does, and renders it as
                // the anchor's text. Ignoring that spelling made this warning
                // fire on the documented form.
                let has_children = ui
                    .children
                    .iter()
                    .any(|s| matches!(&s.kind, StatementKind::UIElement(_)));
                if !has_children
                    && !has_positional_arg(&ui.args)
                    && !has_named_arg(&ui.args, "label")
                {
                    warnings.push(A11yWarning::new(
                        "A06",
                        "Link has no text content",
                        file,
                        line,
                        col,
                        "Add text: Link(\"About\", to: \"/about\")",
                    ));
                }
            }

            // A07: Heading is empty
            "Heading" => {
                // Check heading level for A11
                let level = get_heading_level(&ui.modifiers);
                if level > 0 {
                    heading_tracker.record(level);

                    // A11: Check for skipped levels
                    if heading_tracker.checks_outline && heading_tracker.last_level().is_some() {
                        // Only check against the second-to-last since we just pushed
                        if heading_tracker.levels_seen.len() >= 2 {
                            let prev_level =
                                heading_tracker.levels_seen[heading_tracker.levels_seen.len() - 2];
                            if level > prev_level + 1 {
                                warnings.push(A11yWarning::new(
                                    "A11",
                                    format!(
                                        "Heading level skips from h{} to h{}",
                                        prev_level, level
                                    ),
                                    file,
                                    line,
                                    col,
                                    format!(
                                        "Use h{} instead, or add the missing intermediate headings",
                                        prev_level + 1
                                    ),
                                ));
                            }
                        }
                    }
                }

                // Check for empty text
                if !has_positional_arg(&ui.args) {
                    warnings.push(A11yWarning::new(
                        "A07",
                        "Heading has no text content",
                        file,
                        line,
                        col,
                        "Add text: Heading(\"Section Title\", h2)",
                    ));
                } else if has_empty_string_arg(&ui.args) {
                    warnings.push(A11yWarning::new(
                        "A07",
                        "Heading has empty text content",
                        file,
                        line,
                        col,
                        "Headings should have meaningful text",
                    ));
                }
            }

            // A08: Modal/Dialog missing title
            "Modal" | "Dialog" => {
                if !has_named_arg(&ui.args, "title") {
                    warnings.push(A11yWarning::new(
                        "A08",
                        format!("{} missing \"title\" attribute", name),
                        file,
                        line,
                        col,
                        format!(
                            "Add a title: {}(visible: state, title: \"Dialog Title\")",
                            name
                        ),
                    ));
                }
            }

            // A09: Video missing controls
            "Video" => {
                if !has_named_arg(&ui.args, "controls")
                    && !ui.modifiers.contains(&"controls".to_string())
                {
                    warnings.push(A11yWarning::new(
                        "A09",
                        "Video missing \"controls\" attribute",
                        file,
                        line,
                        col,
                        "Add controls: Video(src: \"...\", controls: true)",
                    ));
                }
            }

            // A10: Table missing header
            "Table" => {
                let has_thead = ui.children.iter().any(|s| {
                    if let StatementKind::UIElement(child) = &s.kind {
                        matches!(&child.component, ComponentRef::BuiltIn(n) if n == "Thead")
                    } else {
                        false
                    }
                });
                if !has_thead {
                    warnings.push(A11yWarning::new(
                        "A10",
                        "Table missing header row (Thead)",
                        file,
                        line,
                        col,
                        "Add a header: Table { Thead { Tcell(\"Column Name\") } ... }",
                    ));
                }
            }

            _ => {}
        }
    }

    lint_aria_structure(ui, file, line, col, warnings, heading_tracker);
    lint_label_in_name(ui, file, line, col, warnings, heading_tracker);

    // A call to a component contributes that component's headings to this
    // page's outline. Its element checks were already reported once, on the
    // component itself, so they are not repeated here.
    if let ComponentRef::UserDefined(name) = &ui.component {
        if !heading_tracker.expanding.contains(name) {
            if let Some(comp) = heading_tracker.component(name) {
                heading_tracker.expanding.push(name.clone());
                let mut quiet = Vec::new();
                lint_statements(&comp.body, file, &mut quiet, heading_tracker);
                // Only the outline findings from inside the expansion matter
                // here — and they are about this page's outline.
                warnings.extend(quiet.into_iter().filter(|w| w.rule_id == "A11"));
                heading_tracker.expanding.pop();
            }
        }
    }

    // Recurse into children
    lint_statements(&ui.children, file, warnings, heading_tracker);
}

/// A14: a `role:` that requires particular children, given children that
/// are not them.
///
/// `role="tablist"` promises assistive technology a set of tabs; a button
/// inside it is not one, and a screen reader will announce a tab list with
/// no tabs. Only built-in children are judged — a user component may carry
/// the role on its own root — and only when their role is known.
fn lint_aria_structure(
    ui: &UIElement,
    file: &str,
    line: usize,
    col: usize,
    warnings: &mut Vec<A11yWarning>,
    tracker: &HeadingTracker,
) {
    let Some(role) = named_arg_literal(&ui.args, "role") else {
        return;
    };
    let required: &[&str] = match role.as_str() {
        "tablist" => &["tab"],
        "list" => &["listitem"],
        "listbox" => &["option", "group"],
        "menu" | "menubar" => &[
            "menuitem",
            "menuitemcheckbox",
            "menuitemradio",
            "group",
            "separator",
        ],
        "radiogroup" => &["radio"],
        "tree" => &["treeitem", "group"],
        "grid" | "table" | "treegrid" => &["row", "rowgroup"],
        "row" => &["cell", "gridcell", "columnheader", "rowheader"],
        _ => return,
    };
    for child in &ui.children {
        let StatementKind::UIElement(inner) = &child.kind else {
            continue;
        };
        // A call to a component is judged by the component's root element,
        // which is what it renders in this position. A root whose role is a
        // prop — `Button(role: role)` with `role: String = "menuitem"` — has
        // the role the call passed, or the prop's default.
        let expanded;
        let mut prop_role: Option<Option<String>> = None;
        let inner = match &inner.component {
            ComponentRef::UserDefined(component) => {
                let Some(decl) = tracker.component(component) else {
                    continue;
                };
                let Some(root) = root_element(&decl.body) else {
                    continue;
                };
                if let Some(Arg::Named(_, Expr::Identifier(prop))) = root
                    .args
                    .iter()
                    .find(|a| matches!(a, Arg::Named(k, _) if k == "role"))
                {
                    let passed = inner.args.iter().find_map(|a| match a {
                        Arg::Named(k, v) if k == prop => Some(v),
                        _ => None,
                    });
                    let default = decl
                        .props
                        .iter()
                        .find(|p| &p.name == prop)
                        .and_then(|p| p.default.as_ref());
                    prop_role = Some(match passed.or(default) {
                        Some(Expr::StringLiteral(s)) => Some(s.clone()),
                        _ => None,
                    });
                }
                expanded = root.clone();
                &expanded
            }
            _ => inner,
        };
        let ComponentRef::BuiltIn(name) = &inner.component else {
            continue;
        };
        let child_role = match prop_role {
            // A role the build cannot read is not one it can judge.
            Some(None) => continue,
            Some(Some(role)) => Some(role),
            None => named_arg_literal(&inner.args, "role").or_else(|| {
                crate::codegen::builtin::implicit_role(name, &inner.modifiers).map(str::to_string)
            }),
        };
        let Some(child_role) = child_role else {
            // A plain container with no role of its own breaks the required
            // ownership just the same, but it may be an author's wrapper
            // around the right children; only a control is certain.
            if matches!(
                name.as_str(),
                "Button" | "IconButton" | "Link" | "Input" | "Text"
            ) {
                warnings.push(A11yWarning::new(
                    "A14",
                    format!("role \"{role}\" requires children with role \"{}\", but holds a {name}", required[0]),
                    file,
                    line,
                    col,
                    format!("Give each child role: \"{}\", or use the built-in that owns this structure", required[0]),
                ));
                return;
            }
            continue;
        };
        if !required.contains(&child_role.as_str()) {
            warnings.push(A11yWarning::new(
                "A14",
                format!(
                    "role \"{role}\" requires children with role \"{}\", but holds a {name} with role \"{child_role}\"",
                    required[0]
                ),
                file,
                line,
                col,
                format!("Give each child role: \"{}\", or use the built-in that owns this structure", required[0]),
            ));
            return;
        }
    }
}

/// A15: a control whose `aria-label` does not contain its visible text.
///
/// Someone using voice control says what they see — "click Save" — and the
/// software matches that against the accessible name. When the label says
/// something else, the button they can see cannot be spoken to (WCAG 2.5.3,
/// Label in Name). The visible text is what the control's literals paint:
/// its positional label, the `Text` and other literals in its block, and
/// what a user component in the block renders from its own literals and
/// the literal props it was given — a `Kbd(text: "⌘K")` inside a search
/// button is text the reader sees. Each piece the build can read must be in
/// the label; a value that reads state cannot be checked and is not.
fn lint_label_in_name(
    ui: &UIElement,
    file: &str,
    line: usize,
    col: usize,
    warnings: &mut Vec<A11yWarning>,
    tracker: &HeadingTracker,
) {
    let ComponentRef::BuiltIn(name) = &ui.component else {
        return;
    };
    if !matches!(name.as_str(), "Button" | "Link" | "Tag" | "Badge") {
        return;
    }
    let Some(label) = named_arg_literal(&ui.args, "aria-label") else {
        return;
    };
    let mut visible = Vec::new();
    if name != "Link" {
        visible.extend(positional_literal(&ui.args));
    }
    visible_text(&ui.children, &HashMap::new(), tracker, 0, &mut visible);
    let label_lower = label.to_lowercase();
    let Some(missing) = visible
        .iter()
        .map(|t| t.trim().to_lowercase())
        .find(|t| !t.is_empty() && !label_lower.contains(t.as_str()))
    else {
        return;
    };
    warnings.push(A11yWarning::new(
        "A15",
        format!("{name} shows \"{missing}\" but its aria-label says \"{label}\""),
        file,
        line,
        col,
        "Start the aria-label with the visible text, so what a user says matches what they see",
    ));
}

/// The literal text `body` paints, in order, into `out`. `props` are the
/// literal props of the component being expanded, so `Text(label)` inside it
/// reads as the text the call passed.
fn visible_text(
    body: &[Statement],
    props: &HashMap<String, String>,
    tracker: &HeadingTracker,
    depth: usize,
    out: &mut Vec<String>,
) {
    if depth > 8 {
        return;
    }
    let literal = |expr: &Expr| -> Option<String> {
        match expr {
            Expr::StringLiteral(s) => Some(s.clone()),
            Expr::Identifier(name) => props.get(name).cloned(),
            _ => None,
        }
    };
    for stmt in body {
        let StatementKind::UIElement(el) = &stmt.kind else {
            continue;
        };
        match &el.component {
            // An icon's positional argument names a glyph, not text.
            ComponentRef::BuiltIn(n) if n == "Icon" || n == "Image" => {}
            ComponentRef::BuiltIn(_) | ComponentRef::SubComponent(_, _) => {
                if let Some(text) = el.args.iter().find_map(|a| match a {
                    Arg::Positional(e) => literal(e),
                    _ => None,
                }) {
                    out.push(text);
                }
                visible_text(&el.children, props, tracker, depth + 1, out);
            }
            ComponentRef::UserDefined(name) => {
                let Some(component) = tracker.component(name) else {
                    continue;
                };
                // The call's literal props, by the component's prop names.
                let mut inner: HashMap<String, String> = HashMap::new();
                let mut positional = el.args.iter().filter_map(|a| match a {
                    Arg::Positional(e) => Some(e),
                    _ => None,
                });
                for prop in &component.props {
                    let value = el
                        .args
                        .iter()
                        .find_map(|a| match a {
                            Arg::Named(k, e) if k == &prop.name => Some(e),
                            _ => None,
                        })
                        .or_else(|| positional.next());
                    if let Some(text) = value.and_then(literal) {
                        inner.insert(prop.name.clone(), text);
                    }
                }
                visible_text(&component.body, &inner, tracker, depth + 1, out);
                visible_text(&el.children, props, tracker, depth + 1, out);
            }
        }
    }
}

/// The first element a body renders — a component's root, when the body
/// begins with an element.
fn root_element(body: &[Statement]) -> Option<&UIElement> {
    body.iter().find_map(|s| match &s.kind {
        StatementKind::UIElement(el) => Some(el),
        _ => None,
    })
}

/// The literal string value of a named argument, if it is one.
fn named_arg_literal(args: &[Arg], name: &str) -> Option<String> {
    args.iter().find_map(|a| match a {
        Arg::Named(n, Expr::StringLiteral(s)) if n == name => Some(s.clone()),
        _ => None,
    })
}

/// The first positional argument, when it is a string literal.
fn positional_literal(args: &[Arg]) -> Option<String> {
    args.iter().find_map(|a| match a {
        Arg::Positional(Expr::StringLiteral(s)) => Some(s.clone()),
        _ => None,
    })
}

// ─── Helper functions ────────────────────────────────

/// Whether the arguments give the control an accessible name: a `label`, or
/// the ARIA attributes that name an element from elsewhere (`aria-label`,
/// `aria-labelledby`), which is how a field labelled by a separate element
/// says so.
fn has_accessible_name(args: &[Arg]) -> bool {
    has_named_arg(args, "label")
        || has_named_arg(args, "aria-label")
        || has_named_arg(args, "aria-labelledby")
}

fn has_named_arg(args: &[Arg], name: &str) -> bool {
    args.iter()
        .any(|a| matches!(a, Arg::Named(n, _) if n == name))
}

fn has_positional_arg(args: &[Arg]) -> bool {
    args.iter().any(|a| matches!(a, Arg::Positional(_)))
}

fn has_empty_string_arg(args: &[Arg]) -> bool {
    args.iter().any(|a| {
        if let Arg::Positional(Expr::StringLiteral(s)) = a {
            s.is_empty()
        } else {
            false
        }
    })
}

fn get_heading_level(modifiers: &[String]) -> u8 {
    for m in modifiers {
        match m.as_str() {
            "h1" => return 1,
            "h2" => return 2,
            "h3" => return 3,
            "h4" => return 4,
            "h5" => return 5,
            "h6" => return 6,
            _ => {}
        }
    }
    0 // No heading level specified
}

#[cfg(test)]
mod naming_tests {
    //! A control named by ARIA is named. A field whose visible label is a
    //! separate element says so with `aria-labelledby`, and used to draw A03
    //! anyway, which taught authors to add a bogus `label` attribute.
    use super::*;

    fn rules(src: &str) -> Vec<String> {
        let program = crate::syntax::parse_source(src, "<t>").expect("parse");
        lint_accessibility(&program)
            .into_iter()
            .map(|w| w.rule_id)
            .collect()
    }

    #[test]
    fn aria_labelledby_names_an_input_and_a_switch() {
        let src = r#"Page P (path: "/", title: "t", description: "d") {
            Heading("h", h1)
            Input(text, id: "n", aria-labelledby: "n-label")
            Switch(bind: on, aria-label: "Reuse the build cache")
        }"#;
        let r = rules(src);
        assert!(!r.contains(&"A03".to_string()), "{r:?}");
        assert!(!r.contains(&"A04".to_string()), "{r:?}");
    }

    #[test]
    fn a_pages_outline_reads_through_the_components_it_calls() {
        let src = r#"
            Component Opener (title: String) { Heading(title, h2) }
            Page P (path: "/", title: "t", description: "d") {
                Heading("Page", h1)
                Opener(title: "Section")
                Heading("Card", h3)
            }"#;
        let r = rules(src);
        assert!(
            !r.contains(&"A11".to_string()),
            "the h2 inside Opener bridges h1 and h3: {r:?}"
        );
    }

    #[test]
    fn a_skipped_level_inside_a_called_component_is_the_pages_problem() {
        let src = r#"
            Component Tile (title: String) { Heading(title, h4) }
            Page P (path: "/", title: "t", description: "d") {
                Heading("Page", h1)
                Tile(title: "x")
            }"#;
        let r = rules(src);
        assert!(r.contains(&"A11".to_string()), "{r:?}");
    }

    #[test]
    fn an_unnamed_input_still_warns() {
        let src = r#"Page P (path: "/", title: "t", description: "d") { Heading("h", h1) Input(text, id: "n") }"#;
        assert!(rules(src).contains(&"A03".to_string()));
    }
}

#[cfg(test)]
mod structure_tests {
    //! A14 and A15, and the positions every element rule now carries.
    use super::*;

    fn warnings(src: &str) -> Vec<A11yWarning> {
        let program = crate::syntax::parse_source(src, "<t>").expect("parse");
        lint_accessibility(&program)
    }

    #[test]
    fn a_tablist_of_plain_buttons_is_reported_once_on_its_line() {
        let src = "Page P (path: \"/\", title: \"t\", description: \"d\") {\n    Heading(\"H\", h1)\n    Row(role: \"tablist\") {\n        Button(\"One\")\n        Button(\"Two\")\n    }\n}\n";
        let found: Vec<_> = warnings(src)
            .into_iter()
            .filter(|w| w.rule_id == "A14")
            .collect();
        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!(found[0].line, 3, "points at the Row");
        assert!(
            found[0]
                .message
                .contains("requires children with role \"tab\""),
            "{}",
            found[0].message
        );
    }

    #[test]
    fn a_tablist_of_tabs_is_fine_and_a_component_is_judged_by_its_root() {
        let src = "Component Tab (label: String) {\n    Button(label, role: \"tab\")\n}\nPage P (path: \"/\", title: \"t\", description: \"d\") {\n    Heading(\"H\", h1)\n    Row(role: \"tablist\") {\n        Button(\"One\", role: \"tab\")\n        Tab(label: \"Two\")\n    }\n}\n";
        assert!(warnings(src).iter().all(|w| w.rule_id != "A14"));
        let bare = "Component Chip (label: String) {\n    Button(label)\n}\nPage P (path: \"/\", title: \"t\", description: \"d\") {\n    Heading(\"H\", h1)\n    Row(role: \"tablist\") {\n        Chip(label: \"Two\")\n    }\n}\n";
        assert!(
            warnings(bare).iter().any(|w| w.rule_id == "A14"),
            "a component whose root is a plain button is a plain button"
        );
    }

    #[test]
    fn a_component_root_whose_role_is_a_prop_has_the_role_the_call_gives_it() {
        let src = "Component Entry (label: String, role: String = \"menuitem\") {\n    Button(label, role: role)\n}\nPage P (path: \"/\", title: \"t\", description: \"d\") {\n    Heading(\"H\", h1)\n    Stack(role: \"menu\") {\n        Entry(label: \"Open\")\n        Entry(label: \"Pick\", role: \"menuitemradio\")\n    }\n    Stack(role: \"menu\") {\n        Entry(label: \"Wrong\", role: \"tab\")\n    }\n    Stack(role: \"menu\") {\n        Entry(label: \"Unknown\", role: someRole)\n    }\n}\n";
        let found: Vec<_> = warnings(src)
            .into_iter()
            .filter(|w| w.rule_id == "A14")
            .collect();
        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!(found[0].line, 10, "only the tab in a menu is wrong");
        assert!(
            found[0].message.contains("with role \"tab\""),
            "{}",
            found[0].message
        );
    }

    #[test]
    fn an_aria_label_that_hides_the_visible_text_is_reported() {
        let src = "Page P (path: \"/\", title: \"t\", description: \"d\") {\n    Heading(\"H\", h1)\n    Button(\"Save\", aria-label: \"Submit the form\")\n    Button(\"Delete\", aria-label: \"Delete the draft\")\n    Link(to: \"/\", aria-label: \"Go home\") { Text(\"Home\") }\n}\n";
        let found: Vec<_> = warnings(src)
            .into_iter()
            .filter(|w| w.rule_id == "A15")
            .collect();
        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!(found[0].line, 3);
        assert!(
            found[0].message.contains("\"save\""),
            "{}",
            found[0].message
        );
    }

    #[test]
    fn text_a_child_or_a_component_paints_counts_as_visible() {
        // The keyboard hint a Kbd component renders from its prop is text the
        // reader sees, and an icon's name is not.
        let src = "Component Kbd (text: String) {\n    Text(text)\n}\nPage P (path: \"/\", title: \"t\", description: \"d\") {\n    Heading(\"H\", h1)\n    Button(\"\", aria-label: \"Search, command palette\") {\n        Icon(\"search\")\n        Kbd(text: \"⌘K\")\n    }\n    Button(\"\", aria-label: \"Search, command palette, ⌘K\") {\n        Icon(\"search\")\n        Kbd(text: \"⌘K\")\n    }\n    Button(\"\", aria-label: \"Notifications\") {\n        Icon(\"bell\")\n        Kbd(text: count)\n    }\n}\n";
        let found: Vec<_> = warnings(src)
            .into_iter()
            .filter(|w| w.rule_id == "A15")
            .collect();
        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!(found[0].line, 6);
        assert!(found[0].message.contains("\"⌘k\""), "{}", found[0].message);
    }

    #[test]
    fn element_rules_carry_the_element_position() {
        let src = "Page P (path: \"/\", title: \"t\", description: \"d\") {\n    Heading(\"H\", h1)\n    Container {\n        Image(src: \"/a.png\")\n    }\n}\n";
        let a01 = warnings(src)
            .into_iter()
            .find(|w| w.rule_id == "A01")
            .expect("A01");
        assert_eq!((a01.line, a01.column), (4, 9));
    }
}
