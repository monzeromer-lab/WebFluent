use crate::error::A11yWarning;
use crate::parser::ast::*;

/// Run all accessibility lint rules on the parsed program.
/// Returns a list of warnings (non-fatal).
pub fn lint_accessibility(program: &Program) -> Vec<A11yWarning> {
    let mut warnings = Vec::new();

    for decl in &program.declarations {
        match decl {
            Declaration::Page(page) => {
                let file = format!("src/pages/{}.wf", page.name);
                lint_page(page, &file, &mut warnings);
            }
            Declaration::Component(comp) => {
                let file = format!("src/components/{}.wf", comp.name);
                lint_statements(&comp.body, &file, &mut warnings, &mut HeadingTracker::new());
            }
            Declaration::App(app) => {
                lint_statements(
                    &app.body,
                    "src/App.wf",
                    &mut warnings,
                    &mut HeadingTracker::new(),
                );
            }
            // Neither holds UI.
            Declaration::Store(_) | Declaration::Theme(_) => {}
        }
    }

    warnings.extend(lint_seo(program));
    warnings
}

/// Findings a search engine or a link preview would act on.
///
/// These sit with the accessibility rules because they share a root: a page a
/// crawler cannot summarise is usually a page a screen reader cannot either. A
/// missing description is a search result whose snippet is written by whatever
/// crawled it; a duplicate route is two pages competing for one ranking.
fn lint_seo(program: &Program) -> Vec<A11yWarning> {
    let mut warnings = Vec::new();
    let mut seen_paths: Vec<(&str, &str)> = Vec::new();

    for decl in &program.declarations {
        let Declaration::Page(page) = decl else {
            continue;
        };
        let file = format!("src/pages/{}.wf", page.name);

        // S01: a page with no title has nothing to show as a search result link.
        if page.title.as_deref().unwrap_or("").trim().is_empty() {
            warnings.push(A11yWarning::new(
                "S01",
                format!("Page {} has no title", page.name),
                &file,
                1,
                1,
                "Add one: Page Name (path: \"/\", title: \"What this page is\")",
            ));
        }

        // S02: no description means the snippet is written for you.
        if page.description.is_none() && !page.noindex {
            warnings.push(A11yWarning::new(
                "S02",
                format!("Page {} has no description", page.name),
                &file,
                1,
                1,
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
                    1,
                    1,
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
                1,
                1,
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
}

impl HeadingTracker {
    fn new() -> Self {
        Self {
            levels_seen: Vec::new(),
            h1_count: 0,
            checks_outline: true,
        }
    }

    /// A tracker for output that is not one HTML page — a deck or a paginated
    /// document. Every other accessibility rule still runs.
    fn without_outline_checks() -> Self {
        Self {
            checks_outline: false,
            ..Self::new()
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

fn lint_page(page: &PageDecl, file: &str, warnings: &mut Vec<A11yWarning>) {
    let mut tracker = if is_document_or_deck(&page.body) {
        HeadingTracker::without_outline_checks()
    } else {
        HeadingTracker::new()
    };
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
    let (line, col) = (0, 0); // We don't have line info on AST nodes directly — use 0:0

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
                if !has_named_arg(&ui.args, "label") && !has_named_arg(&ui.args, "placeholder") {
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
                if !has_named_arg(&ui.args, "label") {
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
                if !has_positional_arg(&ui.args) && !has_named_arg(&ui.args, "label") {
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

    // Recurse into children
    lint_statements(&ui.children, file, warnings, heading_tracker);
}

// ─── Helper functions ────────────────────────────────

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
