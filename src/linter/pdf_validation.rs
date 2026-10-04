use crate::parser::{ComponentRef, Declaration, Program, Statement, StatementKind, UIElement};

/// The elements a paged output (a PDF document or a slide deck) draws.
/// Every other built-in is refused where it is written — an element that
/// would compile and draw nothing is an error, never a silent gap. The test
/// below holds every registry entry to one side of the line.
pub(crate) const PAGED_DRAWN: &[&str] = &[
    // Layout
    "Container",
    "Row",
    "Column",
    "Grid",
    "Stack",
    "Spacer",
    "Divider",
    "Section",
    // Typography
    "Text",
    "Heading",
    "Paragraph",
    "Code",
    "Blockquote",
    "Markdown",
    "Unsafe",
    // Data display
    "Card",
    "Table",
    "List",
    "Badge",
    "Tag",
    "Avatar",
    "Progress",
    "Alert",
    // The parts a table and unsafe markup lower to.
    "Thead",
    "Tbody",
    "Trow",
    "Tcell",
    "UnsafeHtml",
    // Media and graphics
    "Image",
    "Icon",
    "Chart",
    "QrCode",
    "TableOfContents",
    // A link is a link in a PDF too; a trail of them is a breadcrumb.
    "Link",
    "Breadcrumb",
    // The document's own
    "Document",
    "Header",
    "Footer",
    "PageBreak",
    "Background",
    "Watermark",
    // A deck's (where they may go is the slides check's to say)
    "Presentation",
    "Slide",
    "TitleSlide",
    "SectionSlide",
    "TwoColumn",
    "ImageSlide",
];

/// Why an element a paged output does not draw is refused.
fn refusal(name: &str) -> &'static str {
    match name {
        "Button" | "IconButton" | "ButtonGroup" | "Input" | "Select" | "Option" | "Checkbox"
        | "Radio" | "Switch" | "Slider" | "DatePicker" | "FileUpload" | "Form" | "Dropdown"
        | "Textarea" => "interactive elements are not supported in PDF",
        "Router" | "Route" | "Navbar" | "Sidebar" | "Menu" | "Tabs" | "TabPage" => {
            "navigation components are web-only"
        }
        "Host" | "Element" => "a script draws it in a browser, and a PDF runs no script",
        "Video" | "Audio" | "Carousel" => "a PDF cannot play or animate it",
        _ => "this component is not supported in PDF output",
    }
}

/// Whether a built-in is drawn in a paged output.
pub(crate) fn drawn_in_pdf(name: &str) -> bool {
    PAGED_DRAWN.contains(&name)
}

#[derive(Debug)]
pub struct PdfValidationError {
    pub component: String,
    pub context: String,
    pub reason: String,
    /// The declaration it is in, by index, and where.
    pub decl: usize,
    pub span: crate::parser::ast::Span,
}

impl std::fmt::Display for PdfValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "error[pdf]: '{}' cannot be used in PDF output ({}) — {}",
            self.component, self.context, self.reason
        )
    }
}

pub fn validate_for_pdf(program: &Program) -> Vec<PdfValidationError> {
    let mut errors = Vec::new();

    for (ix, decl) in program.declarations.iter().enumerate() {
        match decl {
            Declaration::Page(page) => {
                let ctx = format!("Page {}", page.name);
                validate_statements(&page.body, &ctx, ix, &mut errors);
            }
            Declaration::Component(comp) => {
                let ctx = format!("Component {}", comp.name);
                validate_statements(&comp.body, &ctx, ix, &mut errors);
            }
            Declaration::App(app) => {
                validate_statements(&app.body, "App", ix, &mut errors);
            }
            _ => {}
        }
    }

    errors
}

fn validate_statements(
    stmts: &[Statement],
    context: &str,
    decl: usize,
    errors: &mut Vec<PdfValidationError>,
) {
    for stmt in stmts {
        match &stmt.kind {
            StatementKind::UIElement(ui) => validate_ui_element(ui, context, decl, errors),
            StatementKind::If(if_stmt) => {
                validate_statements(&if_stmt.then_body, context, decl, errors);
                if let Some(else_body) = &if_stmt.else_body {
                    validate_statements(else_body, context, decl, errors);
                }
            }
            StatementKind::For(for_stmt) => {
                validate_statements(&for_stmt.body, context, decl, errors);
            }
            StatementKind::Navigate(_) => {
                errors.push(PdfValidationError {
                    component: "navigate".to_string(),
                    context: context.to_string(),
                    reason: "navigation is a web-only feature".to_string(),
                    decl,
                    span: stmt.span,
                });
            }
            StatementKind::Fetch(_) => {
                errors.push(PdfValidationError {
                    component: "fetch".to_string(),
                    context: context.to_string(),
                    reason: "data fetching is a web-only feature".to_string(),
                    decl,
                    span: stmt.span,
                });
            }
            StatementKind::Animate(_) => {
                errors.push(PdfValidationError {
                    component: "animate".to_string(),
                    context: context.to_string(),
                    reason: "animations are a web-only feature".to_string(),
                    decl,
                    span: stmt.span,
                });
            }
            StatementKind::EventHandler(_) => {
                errors.push(PdfValidationError {
                    component: "event handler".to_string(),
                    context: context.to_string(),
                    reason: "event handlers are a web-only feature".to_string(),
                    decl,
                    span: stmt.span,
                });
            }
            _ => {}
        }
    }
}

fn validate_ui_element(
    ui: &UIElement,
    context: &str,
    decl: usize,
    errors: &mut Vec<PdfValidationError>,
) {
    // `children` and a named slot: the caller's block, checked where it is
    // written.
    if ui.slot_name().is_some() {
        return;
    }
    let name = match &ui.component {
        ComponentRef::BuiltIn(n) => n.clone(),
        ComponentRef::SubComponent(parent, _) => parent.clone(),
        ComponentRef::UserDefined(_) => {
            // User components are allowed; their body is validated separately
            validate_statements(&ui.children, context, decl, errors);
            return;
        }
    };

    if !drawn_in_pdf(&name) {
        let reason = refusal(&name).to_string();
        errors.push(PdfValidationError {
            component: name.clone(),
            context: context.to_string(),
            reason,
            decl,
            span: ui.span,
        });
    }

    // Check for event handlers on allowed elements
    if !ui.events.is_empty() {
        errors.push(PdfValidationError {
            component: format!("{} (events)", name),
            context: context.to_string(),
            reason: "event handlers are not supported in PDF".to_string(),
            decl,
            span: ui.span,
        });
    }

    // Recurse into children
    validate_statements(&ui.children, context, decl, errors);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A new built-in is drawn in a paged output or refused there by a
    /// decision, not by forgetting: this lists the ones on neither side.
    #[test]
    fn every_builtin_is_drawn_or_refused_on_purpose() {
        let refused_on_purpose: &[&str] = &[
            "Button",
            "IconButton",
            "ButtonGroup",
            "Input",
            "Select",
            "Checkbox",
            "Radio",
            "Switch",
            "Slider",
            "DatePicker",
            "FileUpload",
            "Form",
            "Dropdown",
            "Textarea",
            "Modal",
            "Dialog",
            "Toast",
            "Spinner",
            "Skeleton",
            "Router",
            "Navbar",
            "Sidebar",
            "Menu",
            "Tabs",
            "Tooltip",
            "Video",
            "Audio",
            "Carousel",
            "Host",
            "Element",
            "TabPage",
            "Option",
            "Route",
        ];
        let mut unclassified = Vec::new();
        for sig in crate::registry::components() {
            if sig.owner.is_some() {
                continue;
            }
            if !drawn_in_pdf(sig.name) && !refused_on_purpose.contains(&sig.name) {
                unclassified.push(sig.name);
            }
        }
        assert!(
            unclassified.is_empty(),
            "decide whether a PDF draws: {unclassified:?}"
        );
    }
}
