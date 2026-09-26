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
            // None of these holds UI.
            Declaration::Store(_)
            | Declaration::Theme(_)
            | Declaration::Type(_)
            | Declaration::Enum(_)
            | Declaration::Api(_)
            | Declaration::External(_)
            | Declaration::Const(_)
            | Declaration::Animation(_)
            | Declaration::Test(_)
            | Declaration::Data(_) => {}
        }
    }

    warnings.extend(lint_seo(program, file_of));
    warnings.extend(lint_persistence(program, file_of));
    warnings.extend(lint_unsafe(program, file_of));
    warnings
}

/// Every place markup a page did not write goes in as markup.
///
/// It is not a mistake — there is no other way to render a CMS's HTML —
/// but it is the one place in the output where a string becomes structure,
/// so every one of them is named in the build, not only greppable.
fn lint_unsafe(program: &Program, file_of: &dyn Fn(usize) -> String) -> Vec<A11yWarning> {
    fn walk(body: &[Statement], file: &str, out: &mut Vec<A11yWarning>) {
        for stmt in body {
            match &stmt.kind {
                StatementKind::UIElement(el) => {
                    // Written `Unsafe.Html`; lowered to the built-in the
                    // code generators know it by.
                    let door = matches!(&el.component, ComponentRef::SubComponent(owner, _) if owner == "Unsafe")
                        || matches!(&el.component, ComponentRef::BuiltIn(n) if n == "UnsafeHtml");
                    if door {
                        let sanitised = el.args.iter().any(|a| {
                            matches!(a, Arg::Positional(Expr::FunctionCall(f, _)) if f == "sanitize")
                        });
                        out.push(A11yWarning::new(
                            "V03",
                            "`Unsafe.Html` puts markup in as markup".to_string(),
                            file,
                            el.span.line as usize,
                            el.span.col as usize,
                            if sanitised {
                                "It goes through `sanitize`, so this is the reviewed case — keep it that way"
                            } else {
                                "Anything the project did not write itself belongs in `sanitize(…)` first"
                            },
                        ));
                    }
                    walk(&el.children, file, out);
                    for fill in &el.slot_fills {
                        walk(&fill.body, file, out);
                    }
                }
                StatementKind::If(i) => {
                    walk(&i.then_body, file, out);
                    for (_, b) in &i.else_if_branches {
                        walk(b, file, out);
                    }
                    if let Some(b) = &i.else_body {
                        walk(b, file, out);
                    }
                }
                StatementKind::For(f) => walk(&f.body, file, out),
                StatementKind::Show(s) => walk(&s.body, file, out),
                StatementKind::Match(m) => {
                    for arm in &m.arms {
                        walk(&arm.body, file, out);
                    }
                }
                _ => {}
            }
        }
    }
    let mut out = Vec::new();
    for (index, decl) in program.declarations.iter().enumerate() {
        let file = file_of(index);
        match decl {
            Declaration::Page(page) => walk(&page.body, &file, &mut out),
            Declaration::Component(c) => walk(&c.body, &file, &mut out),
            Declaration::App(a) => walk(&a.body, &file, &mut out),
            _ => {}
        }
    }
    out
}

/// What a value kept in the browser's storage says about itself.
///
/// A version is a promise that a value an older build wrote still means
/// something. These two find the places where the promise is not kept.
fn lint_persistence(program: &Program, file_of: &dyn Fn(usize) -> String) -> Vec<A11yWarning> {
    let mut warnings = Vec::new();
    for (index, decl) in program.declarations.iter().enumerate() {
        let file = file_of(index);
        let (body, route_scoped): (&[Statement], bool) = match decl {
            Declaration::Store(store) => (
                &store.body,
                store.scope == crate::parser::ast::StoreScope::Route,
            ),
            Declaration::Page(page) => (&page.body, false),
            Declaration::Component(component) => (&component.body, false),
            _ => continue,
        };
        for stmt in body {
            let StatementKind::State(state) = &stmt.kind else {
                continue;
            };
            let (line, col) = (stmt.span.line as usize, stmt.span.col as usize);
            // P03: the two say opposite things. A `.route` store is
            // dropped when the route changes, and the next read builds it
            // again — which reads the persisted value straight back, so
            // the value outlives the route that was supposed to own it.
            if route_scoped && state.persist {
                warnings.push(A11yWarning::new(
                    "P03",
                    format!(
                        "`{}` is persisted in a store scoped to the route",
                        state.name
                    ),
                    &file,
                    line,
                    col,
                    "The route change drops the store and the next read builds it again from storage, so the value comes straight back. Use `state` for what the route owns, or widen the scope for what outlives it",
                ));
            }
            let Some(policy) = &state.policy else {
                continue;
            };
            let version = policy.version.unwrap_or(0);
            // P02: a step that never runs.
            for step in &policy.migrations {
                if step.to > version {
                    warnings.push(A11yWarning::new(
                        "P02",
                        format!(
                            "`migrate {} -> {}` on `{}` is past the declared version",
                            step.from, step.to, state.name
                        ),
                        &file,
                        line,
                        col,
                        if version == 0 {
                            "Declare the version this build writes: `version: 2`"
                        } else {
                            "A step above `version:` never runs; raise the version or drop the step"
                        },
                    ));
                }
            }
            // P01: a gap in the chain, so a value from that version is lost.
            for at in 1..version {
                if !policy.migrations.iter().any(|m| m.from == at) {
                    warnings.push(A11yWarning::new(
                        "P01",
                        format!(
                            "`{}` is at version {version}, and nothing brings version {at} forward",
                            state.name
                        ),
                        &file,
                        line,
                        col,
                        format!(
                            "A reader who last visited then loses what they had. Add `migrate {at} -> {}`",
                            at + 1
                        ),
                    ));
                }
            }
        }
    }
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
                "Add one: page Name(path: \"/\", title: \"What this page is\")",
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
                "Add one: page Name(path: \"/\", title: \"…\", description: \"A sentence a search result can show\")",
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
        // `seen_paths` holds `(name, path)`: this compared a page's path with
        // the names of the pages before it, so two pages on one route were
        // never reported.
        if let Some((other, _)) = seen_paths.iter().find(|(_, p)| *p == page.path) {
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
    /// While a component is expanded: its `Bool` props, resolved from the
    /// call's flags and arguments over the component's own defaults. An
    /// `if` on one of them takes the branch that call really renders, so a
    /// card that is an `h2` under a page title and an `h3` inside a section
    /// is judged as each caller uses it.
    props: std::collections::HashMap<String, bool>,
    /// While a page's layout is walked: the page's body, placed where the
    /// layout's `children` is, and the warnings it produces there.
    page_body: Option<(Vec<Statement>, String)>,
    page_warnings: Vec<A11yWarning>,
}

impl HeadingTracker {
    fn new() -> Self {
        Self {
            levels_seen: Vec::new(),
            h1_count: 0,
            checks_outline: true,
            components: std::collections::HashMap::new(),
            expanding: Vec::new(),
            props: std::collections::HashMap::new(),
            page_body: None,
            page_warnings: Vec::new(),
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

    /// The one branch of an `if` a call renders, when its condition is a
    /// `Bool` prop this expansion knows. `None` when it cannot be resolved,
    /// and the branches are judged as alternatives.
    fn taken_branch<'a>(&self, if_stmt: &'a IfStmt) -> Option<&'a [Statement]> {
        if self.props.is_empty() || !if_stmt.else_if_branches.is_empty() {
            return None;
        }
        let (name, negated) = match &if_stmt.condition {
            Expr::Identifier(n) => (n, false),
            Expr::UnaryOp(UnaryOp::Not, inner) => match inner.as_ref() {
                Expr::Identifier(n) => (n, true),
                _ => return None,
            },
            _ => return None,
        };
        let value = *self.props.get(name.as_str())? != negated;
        if value {
            Some(&if_stmt.then_body)
        } else {
            Some(if_stmt.else_body.as_deref().unwrap_or(&[]))
        }
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
    // A page framed by a layout is the layout's outline with the page's
    // body at its `children`: the `h1` a layout draws is the page's.
    match page
        .layout
        .as_ref()
        .and_then(|l| tracker.component(&l.name))
    {
        Some(layout) => {
            tracker.page_body = Some((page.body.clone(), file.to_string()));
            let mut quiet = Vec::new();
            lint_statements(&layout.body, file, &mut quiet, &mut tracker);
            warnings.extend(quiet.into_iter().filter(|w| w.rule_id == "A11"));
            match tracker.page_body.take() {
                // The layout placed the page: its findings are the page's.
                None => warnings.append(&mut tracker.page_warnings),
                // A layout without `children`: the page still gets checked.
                Some((body, _)) => lint_statements(&body, file, warnings, &mut tracker),
            }
        }
        None => lint_statements(&page.body, file, warnings, &mut tracker),
    }

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
            "Add a main heading: Heading(\"Page Title\").h1".to_string(),
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
            // Only one branch of an `if`, one arm of a `match` and one state
            // of a resource is ever on the page, so their headings are
            // alternatives rather than a sequence: an `h1` in the `if` and
            // another in the `else` is one `h1`, not two.
            StatementKind::If(if_stmt) => {
                if let Some(taken) = heading_tracker.taken_branch(if_stmt) {
                    lint_statements(taken, file, warnings, heading_tracker);
                } else {
                    let mut bodies: Vec<&[Statement]> = vec![&if_stmt.then_body];
                    bodies.extend(if_stmt.else_if_branches.iter().map(|(_, b)| b.as_slice()));
                    if let Some(else_body) = &if_stmt.else_body {
                        bodies.push(else_body);
                    }
                    lint_alternatives(&bodies, file, warnings, heading_tracker);
                }
            }
            StatementKind::For(for_stmt) => {
                lint_statements(&for_stmt.body, file, warnings, heading_tracker);
            }
            StatementKind::Show(show_stmt) => {
                lint_statements(&show_stmt.body, file, warnings, heading_tracker);
            }
            StatementKind::Fetch(fetch) => {
                let mut bodies: Vec<&[Statement]> = Vec::new();
                if let Some(loading) = &fetch.loading_block {
                    bodies.push(loading);
                }
                if let Some((_, error_body)) = &fetch.error_block {
                    bodies.push(error_body);
                }
                if let Some(success) = &fetch.success_block {
                    bodies.push(success);
                }
                lint_alternatives(&bodies, file, warnings, heading_tracker);
            }
            StatementKind::Match(m) => {
                let bodies: Vec<&[Statement]> = m.arms.iter().map(|a| a.body.as_slice()).collect();
                lint_alternatives(&bodies, file, warnings, heading_tracker);
            }
            _ => {}
        }
    }
}

/// Bodies of which only one is on the page at a time. Each is checked from
/// the outline as it stood before them; afterwards the outline is the one the
/// body with the most headings left, so a later heading is measured against
/// what a reader could actually have seen.
fn lint_alternatives(
    bodies: &[&[Statement]],
    file: &str,
    warnings: &mut Vec<A11yWarning>,
    heading_tracker: &mut HeadingTracker,
) {
    let (levels, h1s) = (
        heading_tracker.levels_seen.clone(),
        heading_tracker.h1_count,
    );
    let mut widest: Option<(Vec<u8>, usize)> = None;
    for body in bodies {
        heading_tracker.levels_seen = levels.clone();
        heading_tracker.h1_count = h1s;
        lint_statements(body, file, warnings, heading_tracker);
        let after = (
            heading_tracker.levels_seen.clone(),
            heading_tracker.h1_count,
        );
        if widest
            .as_ref()
            .is_none_or(|w| after.1 > w.1 || (after.1 == w.1 && after.0.len() > w.0.len()))
        {
            widest = Some(after);
        }
    }
    if let Some((levels, h1s)) = widest {
        heading_tracker.levels_seen = levels;
        heading_tracker.h1_count = h1s;
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

    // The layout's `children`: the page's own body goes here.
    if matches!(&ui.component, ComponentRef::BuiltIn(n) if n == "Children")
        && let Some((body, page_file)) = heading_tracker.page_body.take()
    {
        let mut found = Vec::new();
        lint_statements(&body, &page_file, &mut found, heading_tracker);
        heading_tracker.page_warnings.append(&mut found);
        return;
    }

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
                        "Add a label: Input(label: \"Username\").text",
                    ));
                }
            }

            // A04: Form control missing label
            "Checkbox" | "Radio" | "Switch" | "Slider" | "Textarea" => {
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
                        "Add text: Button(\"Save\").primary",
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

            // Markdown known at build time contributes its headings to the
            // outline, as the rendered page will have them.
            "Markdown" => {
                if let Some(Arg::Positional(Expr::StringLiteral(text))) = ui.args.first() {
                    for line in text.lines() {
                        let level = line.chars().take_while(|c| *c == '#').count();
                        if (1..=6).contains(&level) && line[level..].starts_with(' ') {
                            heading_tracker.record(level as u8);
                        }
                    }
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
                        "Add text: Heading(\"Section Title\").h2",
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

            // A09: Video missing controls, or captions
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
                // A video nobody can hear is a video nobody can follow.
                if !has_named_arg(&ui.args, "captions") {
                    warnings.push(A11yWarning::new(
                        "A09",
                        "Video missing \"captions\"",
                        file,
                        line,
                        col,
                        "Add captions: Video(src: \"...\", captions: \"/captions.en.vtt\")",
                    ));
                }
            }
            // The same for sound: what was said, in writing.
            "Audio" => {
                if !has_named_arg(&ui.args, "controls")
                    && !ui.modifiers.contains(&"controls".to_string())
                {
                    warnings.push(A11yWarning::new(
                        "A09",
                        "Audio missing \"controls\" attribute",
                        file,
                        line,
                        col,
                        "Add controls: Audio(src: \"...\", controls: true)",
                    ));
                }
                if !has_named_arg(&ui.args, "transcript") {
                    warnings.push(A11yWarning::new(
                        "A09",
                        "Audio missing \"transcript\"",
                        file,
                        line,
                        col,
                        "Add a transcript: Audio(src: \"...\", transcript: \"/talk.txt\")",
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
                let outer = std::mem::replace(&mut heading_tracker.props, bool_props(&comp, ui));
                let mut quiet = Vec::new();
                lint_statements(&comp.body, file, &mut quiet, heading_tracker);
                heading_tracker.props = outer;
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

/// The `Bool` props of `comp` as this call sets them: its declared
/// defaults, then a flag written on the call, then a literal argument.
/// A value that is not known at build time leaves the prop out, and an
/// `if` on it is judged as alternatives.
fn bool_props(comp: &ComponentDecl, ui: &UIElement) -> std::collections::HashMap<String, bool> {
    let mut props = std::collections::HashMap::new();
    for prop in &comp.props {
        if let Some(Expr::BoolLiteral(b)) = &prop.default {
            props.insert(prop.name.clone(), *b);
        }
    }
    for flag in &ui.modifiers {
        if comp.props.iter().any(|p| &p.name == flag) {
            props.insert(flag.clone(), true);
        }
    }
    for arg in &ui.args {
        if let Arg::Named(name, value) = arg {
            match value {
                Expr::BoolLiteral(b) => {
                    props.insert(name.clone(), *b);
                }
                _ => {
                    props.remove(name);
                }
            }
        }
    }
    props
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
        let src = r#"page P(path: "/", title: "t", description: "d") {
            Heading("h").h1
            Input(id: "n", aria-labelledby: "n-label").text
            Switch(bind: on, aria-label: "Reuse the build cache")
        }"#;
        let r = rules(src);
        assert!(!r.contains(&"A03".to_string()), "{r:?}");
        assert!(!r.contains(&"A04".to_string()), "{r:?}");
    }

    #[test]
    fn a_pages_outline_reads_through_the_components_it_calls() {
        let src = r#"
            component Opener(title: String) { Heading(title).h2 }
            page P(path: "/", title: "t", description: "d") {
                Heading("Page").h1
                Opener(title: "Section")
                Heading("Card").h3
            }"#;
        let r = rules(src);
        assert!(
            !r.contains(&"A11".to_string()),
            "the h2 inside Opener bridges h1 and h3: {r:?}"
        );
    }

    /// A card that titles itself `h2` under a page title and `h3` inside a
    /// section renders one of the two, chosen by the caller. Walking both
    /// branches reported the `h3` one against a page whose call says `h2`.
    #[test]
    fn a_branch_on_a_bool_prop_is_resolved_from_the_call() {
        let src = r#"
            component Plan(name: String, top: Bool = false) {
                if top { Heading(name).h2 } else { Heading(name).h3 }
            }
            page P(path: "/", title: "t", description: "d") {
                Heading("Page").h1
                Plan(name: "Hobby", top: true)
            }"#;
        let r = rules(src);
        assert!(
            !r.contains(&"A11".to_string()),
            "top: true renders the h2: {r:?}"
        );
    }

    /// The same card called the other way really does skip.
    #[test]
    fn a_branch_on_a_bool_prop_still_warns_when_the_call_takes_it() {
        let src = r#"
            component Plan(name: String, top: Bool = false) {
                if top { Heading(name).h2 } else { Heading(name).h3 }
            }
            page P(path: "/", title: "t", description: "d") {
                Heading("Page").h1
                Plan(name: "Hobby")
            }"#;
        assert!(rules(src).contains(&"A11".to_string()));
    }

    #[test]
    fn a_skipped_level_inside_a_called_component_is_the_pages_problem() {
        let src = r#"
            component Tile(title: String) { Heading(title).h4 }
            page P(path: "/", title: "t", description: "d") {
                Heading("Page").h1
                Tile(title: "x")
            }"#;
        let r = rules(src);
        assert!(r.contains(&"A11".to_string()), "{r:?}");
    }

    #[test]
    fn an_unnamed_input_still_warns() {
        let src = r#"page P(path: "/", title: "t", description: "d") { Heading("h").h1 Input(id: "n").text }"#;
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
    fn two_pages_on_one_route_are_reported() {
        let src = "page A(path: \"/about\", title: \"A\", description: \"d\") { Heading(\"A\").h1 }\npage B(path: \"/about\", title: \"B\", description: \"d\") { Heading(\"B\").h1 }\n";
        let found = warnings(src);
        assert!(
            found.iter().any(|w| w.rule_id == "S04"
                && w.message
                    .contains("Pages A and B both claim the route /about")),
            "{found:?}"
        );
        let apart = "page A(path: \"/a\", title: \"A\", description: \"d\") { Heading(\"A\").h1 }\npage B(path: \"/b\", title: \"B\", description: \"d\") { Heading(\"B\").h1 }\n";
        assert!(warnings(apart).iter().all(|w| w.rule_id != "S04"));
    }

    #[test]
    fn an_h1_in_each_branch_is_one_h1() {
        // Only one branch shows, so a page that heads each with an `h1` has
        // exactly one — which is what a detail page with a "not found" branch
        // always looked like, and what A12 used to count as two.
        let src = "page P(path: \"/\", title: \"t\", description: \"d\") {\n    state found = true\n    if found {\n        Heading(\"A\").h1\n    } else {\n        Heading(\"B\").h1\n    }\n}\n";
        let found = warnings(src);
        assert!(found.iter().all(|w| w.rule_id != "A12"), "{found:?}");
        // Two in one branch are still two.
        let twice = "page P(path: \"/\", title: \"t\", description: \"d\") {\n    state found = true\n    if found {\n        Heading(\"A\").h1\n        Heading(\"B\").h1\n    }\n}\n";
        assert!(warnings(twice).iter().any(|w| w.rule_id == "A12"));
        // And a page whose only h1 is in one branch of several has one.
        let one = "page P(path: \"/\", title: \"t\", description: \"d\") {\n    state found = true\n    if found {\n        Heading(\"A\").h1\n    } else {\n        Text(\"none\")\n    }\n}\n";
        assert!(warnings(one).iter().all(|w| w.rule_id != "A12"));
    }

    #[test]
    fn a_page_framed_by_a_layout_is_judged_with_the_layouts_outline() {
        // The layout draws the h1; the page's body sits at its `children`.
        let src = "component Shell(_ heading: String) {\n    slot\n    Heading(heading).h1\n    children\n}\npage P(path: \"/\", title: \"t\", description: \"d\", layout: Shell(\"Hi\")) {\n    Heading(\"Section\").h2\n    Image(src: \"x.png\")\n}\n";
        let found = warnings(src);
        assert!(found.iter().all(|w| w.rule_id != "A12"), "{found:?}");
        assert!(found.iter().all(|w| w.rule_id != "A11"), "{found:?}");
        // The page's own findings still surface, once.
        assert_eq!(
            found.iter().filter(|w| w.rule_id == "A01").count(),
            1,
            "{found:?}"
        );
        // A skip across the boundary is seen: h1 in the layout, h3 in the page.
        let skip = "component Shell {\n    slot\n    Heading(\"Hi\").h1\n    children\n}\npage P(path: \"/\", title: \"t\", description: \"d\", layout: Shell) {\n    Heading(\"Deep\").h3\n}\n";
        assert!(warnings(skip).iter().any(|w| w.rule_id == "A11"));
    }

    #[test]
    fn a_tablist_of_plain_buttons_is_reported_once_on_its_line() {
        let src = "page P(path: \"/\", title: \"t\", description: \"d\") {\n    Heading(\"H\").h1\n    Row(role: \"tablist\") {\n        Button(\"One\")\n        Button(\"Two\")\n    }\n}\n";
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
        let src = "component Tab(label: String) {\n    Button(label, role: \"tab\")\n}\npage P(path: \"/\", title: \"t\", description: \"d\") {\n    Heading(\"H\").h1\n    Row(role: \"tablist\") {\n        Button(\"One\", role: \"tab\")\n        Tab(label: \"Two\")\n    }\n}\n";
        assert!(warnings(src).iter().all(|w| w.rule_id != "A14"));
        let bare = "component Chip(label: String) {\n    Button(label)\n}\npage P(path: \"/\", title: \"t\", description: \"d\") {\n    Heading(\"H\").h1\n    Row(role: \"tablist\") {\n        Chip(label: \"Two\")\n    }\n}\n";
        assert!(
            warnings(bare).iter().any(|w| w.rule_id == "A14"),
            "a component whose root is a plain button is a plain button"
        );
    }

    #[test]
    fn a_component_root_whose_role_is_a_prop_has_the_role_the_call_gives_it() {
        let src = "component Entry(label: String, role: String = \"menuitem\") {\n    Button(label, role: role)\n}\npage P(path: \"/\", title: \"t\", description: \"d\") {\n    Heading(\"H\").h1\n    Stack(role: \"menu\") {\n        Entry(label: \"Open\")\n        Entry(label: \"Pick\", role: \"menuitemradio\")\n    }\n    Stack(role: \"menu\") {\n        Entry(label: \"Wrong\", role: \"tab\")\n    }\n    Stack(role: \"menu\") {\n        Entry(label: \"Unknown\", role: someRole)\n    }\n}\n";
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
        let src = "page P(path: \"/\", title: \"t\", description: \"d\") {\n    Heading(\"H\").h1\n    Button(\"Save\", aria-label: \"Submit the form\")\n    Button(\"Delete\", aria-label: \"Delete the draft\")\n    Link(to: \"/\", aria-label: \"Go home\") { Text(\"Home\") }\n}\n";
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
        let src = "component Kbd(text: String) {\n    Text(text)\n}\npage P(path: \"/\", title: \"t\", description: \"d\") {\n    Heading(\"H\").h1\n    Button(\"\", aria-label: \"Search, command palette\") {\n        Icon(\"search\")\n        Kbd(text: \"⌘K\")\n    }\n    Button(\"\", aria-label: \"Search, command palette, ⌘K\") {\n        Icon(\"search\")\n        Kbd(text: \"⌘K\")\n    }\n    Button(\"\", aria-label: \"Notifications\") {\n        Icon(\"bell\")\n        Kbd(text: count)\n    }\n}\n";
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
        let src = "page P(path: \"/\", title: \"t\", description: \"d\") {\n    Heading(\"H\").h1\n    Container {\n        Image(src: \"/a.png\")\n    }\n}\n";
        let a01 = warnings(src)
            .into_iter()
            .find(|w| w.rule_id == "A01")
            .expect("A01");
        assert_eq!((a01.line, a01.column), (4, 9));
    }
}
