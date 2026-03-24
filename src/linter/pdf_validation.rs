use crate::parser::{Program, Declaration, Statement, UIElement, ComponentRef};

/// Interactive or web-only components that are not allowed in PDF output
const REJECTED_COMPONENTS: &[&str] = &[
    // Data input (interactive)
    "Button", "IconButton", "ButtonGroup", "Input", "Select", "Option",
    "Checkbox", "Radio", "Switch", "Slider", "DatePicker", "FileUpload", "Form",
    "Dropdown",
    // Feedback (interactive)
    "Modal", "Dialog", "Toast", "Spinner", "Skeleton",
    // Navigation (web-only)
    "Router", "Route", "Navbar", "Sidebar", "Menu", "Tabs", "TabPage",
    "Breadcrumb", "Link",
    // Media (interactive)
    "Video", "Carousel",
    // Layout (web-only)
    "Tooltip",
];

#[derive(Debug)]
pub struct PdfValidationError {
    pub component: String,
    pub context: String,
    pub reason: String,
}

impl std::fmt::Display for PdfValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "error[pdf]: '{}' cannot be used in PDF output ({}) — {}",
            self.component, self.context, self.reason)
    }
}

pub fn validate_for_pdf(program: &Program) -> Vec<PdfValidationError> {
    let mut errors = Vec::new();

    for decl in &program.declarations {
        match decl {
            Declaration::Page(page) => {
                let ctx = format!("Page {}", page.name);
                validate_statements(&page.body, &ctx, &mut errors);
            }
            Declaration::Component(comp) => {
                let ctx = format!("Component {}", comp.name);
                validate_statements(&comp.body, &ctx, &mut errors);
            }
            Declaration::App(app) => {
                validate_statements(&app.body, "App", &mut errors);
            }
            _ => {}
        }
    }

    errors
}

fn validate_statements(stmts: &[Statement], context: &str, errors: &mut Vec<PdfValidationError>) {
    for stmt in stmts {
        match stmt {
            Statement::UIElement(ui) => validate_ui_element(ui, context, errors),
            Statement::If(if_stmt) => {
                validate_statements(&if_stmt.then_body, context, errors);
                if let Some(else_body) = &if_stmt.else_body {
                    validate_statements(else_body, context, errors);
                }
            }
            Statement::For(for_stmt) => {
                validate_statements(&for_stmt.body, context, errors);
            }
            Statement::Navigate(_) => {
                errors.push(PdfValidationError {
                    component: "navigate".to_string(),
                    context: context.to_string(),
                    reason: "navigation is a web-only feature".to_string(),
                });
            }
            Statement::Fetch(_) => {
                errors.push(PdfValidationError {
                    component: "fetch".to_string(),
                    context: context.to_string(),
                    reason: "data fetching is a web-only feature".to_string(),
                });
            }
            Statement::Animate(_) => {
                errors.push(PdfValidationError {
                    component: "animate".to_string(),
                    context: context.to_string(),
                    reason: "animations are a web-only feature".to_string(),
                });
            }
            Statement::EventHandler(_) => {
                errors.push(PdfValidationError {
                    component: "event handler".to_string(),
                    context: context.to_string(),
                    reason: "event handlers are a web-only feature".to_string(),
                });
            }
            _ => {}
        }
    }
}

fn validate_ui_element(ui: &UIElement, context: &str, errors: &mut Vec<PdfValidationError>) {
    let name = match &ui.component {
        ComponentRef::BuiltIn(n) => n.clone(),
        ComponentRef::SubComponent(parent, _) => parent.clone(),
        ComponentRef::UserDefined(_) => {
            // User components are allowed; their body is validated separately
            validate_statements(&ui.children, context, errors);
            return;
        }
    };

    if REJECTED_COMPONENTS.contains(&name.as_str()) {
        let reason = if matches!(name.as_str(),
            "Button" | "Input" | "Select" | "Checkbox" | "Radio" | "Switch" |
            "Slider" | "DatePicker" | "FileUpload" | "Form" | "Dropdown" |
            "IconButton" | "ButtonGroup" | "Option"
        ) {
            "interactive elements are not supported in PDF".to_string()
        } else if matches!(name.as_str(),
            "Router" | "Route" | "Navbar" | "Sidebar" | "Menu" | "Tabs" |
            "TabPage" | "Breadcrumb" | "Link"
        ) {
            "navigation components are web-only".to_string()
        } else {
            "this component is not supported in PDF output".to_string()
        };

        errors.push(PdfValidationError {
            component: name.clone(),
            context: context.to_string(),
            reason,
        });
    }

    // Check for event handlers on allowed elements
    if !ui.events.is_empty() {
        errors.push(PdfValidationError {
            component: format!("{} (events)", name),
            context: context.to_string(),
            reason: "event handlers are not supported in PDF".to_string(),
        });
    }

    // Recurse into children
    validate_statements(&ui.children, context, errors);
}
