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
                lint_statements(&app.body, "src/App.wf", &mut warnings, &mut HeadingTracker::new());
            }
            Declaration::Store(_) => {} // No UI in stores
        }
    }

    warnings
}

/// Track heading levels within a page for skip detection.
struct HeadingTracker {
    levels_seen: Vec<u8>,
    h1_count: usize,
}

impl HeadingTracker {
    fn new() -> Self {
        Self {
            levels_seen: Vec::new(),
            h1_count: 0,
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

fn lint_page(page: &PageDecl, file: &str, warnings: &mut Vec<A11yWarning>) {
    let mut tracker = HeadingTracker::new();
    lint_statements(&page.body, file, warnings, &mut tracker);

    // A12: Page should have exactly one h1
    if tracker.h1_count == 0 {
        warnings.push(A11yWarning::new(
            "A12", "Page has no h1 heading", file, 1, 1,
            format!("Add a main heading: Heading(\"Page Title\", h1)"),
        ));
    } else if tracker.h1_count > 1 {
        warnings.push(A11yWarning::new(
            "A12",
            format!("Page has {} h1 headings (should be exactly 1)", tracker.h1_count),
            file, 1, 1,
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
        match stmt {
            Statement::UIElement(ui) => lint_ui_element(ui, file, warnings, heading_tracker),
            Statement::If(if_stmt) => {
                lint_statements(&if_stmt.then_body, file, warnings, heading_tracker);
                for (_, body) in &if_stmt.else_if_branches {
                    lint_statements(body, file, warnings, heading_tracker);
                }
                if let Some(else_body) = &if_stmt.else_body {
                    lint_statements(else_body, file, warnings, heading_tracker);
                }
            }
            Statement::For(for_stmt) => {
                lint_statements(&for_stmt.body, file, warnings, heading_tracker);
            }
            Statement::Show(show_stmt) => {
                lint_statements(&show_stmt.body, file, warnings, heading_tracker);
            }
            Statement::Fetch(fetch) => {
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
                        "A01", "Image missing \"alt\" attribute", file, line, col,
                        "Add alt text: Image(src: \"...\", alt: \"Description of image\")",
                    ));
                }
            }

            // A02: IconButton missing accessible label
            "IconButton" => {
                if !has_named_arg(&ui.args, "label") && !has_positional_arg(&ui.args) {
                    warnings.push(A11yWarning::new(
                        "A02", "IconButton missing accessible label", file, line, col,
                        "Add a label: IconButton(icon: \"close\", label: \"Close dialog\")",
                    ));
                }
            }

            // A03: Input missing label
            "Input" => {
                if !has_named_arg(&ui.args, "label") && !has_named_arg(&ui.args, "placeholder") {
                    warnings.push(A11yWarning::new(
                        "A03", "Input missing \"label\" or \"placeholder\" attribute", file, line, col,
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
                        file, line, col,
                        format!("Add a label: {}(bind: value, label: \"Description\")", name),
                    ));
                }
            }

            // A05: Button has no text content
            "Button" => {
                if !has_positional_arg(&ui.args) && !has_named_arg(&ui.args, "label") {
                    warnings.push(A11yWarning::new(
                        "A05", "Button has no text content", file, line, col,
                        "Add text: Button(\"Save\", primary)",
                    ));
                }
            }

            // A06: Link has no text content
            "Link" => {
                let has_children = ui.children.iter().any(|s| matches!(s, Statement::UIElement(_)));
                if !has_children && !has_named_arg(&ui.args, "label") {
                    warnings.push(A11yWarning::new(
                        "A06", "Link has no text content", file, line, col,
                        "Add text: Link(to: \"/about\") { Text(\"About\") }",
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
                    if let Some(_prev) = heading_tracker.last_level() {
                        // Only check against the second-to-last since we just pushed
                        if heading_tracker.levels_seen.len() >= 2 {
                            let prev_level = heading_tracker.levels_seen[heading_tracker.levels_seen.len() - 2];
                            if level > prev_level + 1 {
                                warnings.push(A11yWarning::new(
                                    "A11",
                                    format!("Heading level skips from h{} to h{}", prev_level, level),
                                    file, line, col,
                                    format!("Use h{} instead, or add the missing intermediate headings", prev_level + 1),
                                ));
                            }
                        }
                    }
                }

                // Check for empty text
                if !has_positional_arg(&ui.args) {
                    warnings.push(A11yWarning::new(
                        "A07", "Heading has no text content", file, line, col,
                        "Add text: Heading(\"Section Title\", h2)",
                    ));
                } else if has_empty_string_arg(&ui.args) {
                    warnings.push(A11yWarning::new(
                        "A07", "Heading has empty text content", file, line, col,
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
                        file, line, col,
                        format!("Add a title: {}(visible: state, title: \"Dialog Title\")", name),
                    ));
                }
            }

            // A09: Video missing controls
            "Video" => {
                if !has_named_arg(&ui.args, "controls") && !ui.modifiers.contains(&"controls".to_string()) {
                    warnings.push(A11yWarning::new(
                        "A09", "Video missing \"controls\" attribute", file, line, col,
                        "Add controls: Video(src: \"...\", controls: true)",
                    ));
                }
            }

            // A10: Table missing header
            "Table" => {
                let has_thead = ui.children.iter().any(|s| {
                    if let Statement::UIElement(child) = s {
                        matches!(&child.component, ComponentRef::BuiltIn(n) if n == "Thead")
                    } else {
                        false
                    }
                });
                if !has_thead {
                    warnings.push(A11yWarning::new(
                        "A10", "Table missing header row (Thead)", file, line, col,
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
    args.iter().any(|a| matches!(a, Arg::Named(n, _) if n == name))
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
