//! Validation for slides output: enforces deck structure and rejects
//! interactive/web-only components (same list PDF rejects, plus a few extras
//! that don't fit the slides model like `Header`/`Footer`).

use crate::parser::{Program, Declaration, Statement, UIElement, ComponentRef, Arg};
use super::pdf_validation::REJECTED_COMPONENTS;

const SLIDE_KINDS: &[&str] = &[
    "Slide", "TitleSlide", "SectionSlide", "TwoColumn", "ImageSlide",
];

/// Components that don't make sense inside a slide deck.
/// `Header`/`Footer` are rejected because slides have their own footer chrome via config.
/// `Document`/`Section`/`Paragraph`/`PageBreak` belong to the PDF document model.
const SLIDES_INCOMPATIBLE: &[&str] = &[
    "Header", "Footer", "Document", "Paragraph", "PageBreak",
];

#[derive(Debug)]
pub struct SlidesValidationError {
    pub component: String,
    pub context: String,
    pub reason: String,
}

impl std::fmt::Display for SlidesValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "error[slides]: '{}' in {} — {}",
            self.component, self.context, self.reason)
    }
}

pub fn validate_for_slides(program: &Program) -> Vec<SlidesValidationError> {
    let mut errors = Vec::new();

    for decl in &program.declarations {
        match decl {
            Declaration::Page(page) => {
                let ctx = format!("Page {}", page.name);
                validate_page_body(&page.body, &ctx, &mut errors);
            }
            Declaration::Component(comp) => {
                let ctx = format!("Component {}", comp.name);
                validate_outside_presentation(&comp.body, &ctx, &mut errors);
            }
            Declaration::App(app) => {
                validate_outside_presentation(&app.body, "App", &mut errors);
            }
            _ => {}
        }
    }

    errors
}

fn validate_page_body(stmts: &[Statement], context: &str, errors: &mut Vec<SlidesValidationError>) {
    for stmt in stmts {
        if let Statement::UIElement(ui) = stmt {
            if let ComponentRef::BuiltIn(name) = &ui.component {
                if name == "Presentation" {
                    validate_presentation_children(&ui.children, context, errors);
                    continue;
                }
                if SLIDE_KINDS.contains(&name.as_str()) {
                    errors.push(SlidesValidationError {
                        component: name.clone(),
                        context: context.to_string(),
                        reason: "slide elements must be inside a Presentation { ... } block".to_string(),
                    });
                    continue;
                }
            }
        }
        // Anything else outside a Presentation is allowed but ignored at codegen time.
        // We could warn here but choose not to in v1.
    }
}

fn validate_presentation_children(stmts: &[Statement], context: &str, errors: &mut Vec<SlidesValidationError>) {
    for stmt in stmts {
        match stmt {
            Statement::UIElement(ui) => {
                let name = match &ui.component {
                    ComponentRef::BuiltIn(n) => n.clone(),
                    _ => {
                        errors.push(SlidesValidationError {
                            component: format!("{:?}", ui.component),
                            context: context.to_string(),
                            reason: "Presentation may only contain Slide / TitleSlide / SectionSlide / TwoColumn / ImageSlide".to_string(),
                        });
                        continue;
                    }
                };
                if !SLIDE_KINDS.contains(&name.as_str()) {
                    errors.push(SlidesValidationError {
                        component: name,
                        context: context.to_string(),
                        reason: "Presentation may only contain Slide / TitleSlide / SectionSlide / TwoColumn / ImageSlide".to_string(),
                    });
                    continue;
                }
                validate_slide_kind(&name, ui, context, errors);
            }
            _ => {
                errors.push(SlidesValidationError {
                    component: "non-slide statement".to_string(),
                    context: context.to_string(),
                    reason: "Presentation children must be slide elements (no if/for/state)".to_string(),
                });
            }
        }
    }
}

fn validate_slide_kind(name: &str, ui: &UIElement, context: &str, errors: &mut Vec<SlidesValidationError>) {
    let slide_ctx = format!("{} > {}", context, name);

    match name {
        "TwoColumn" => {
            let ui_children: Vec<&UIElement> = ui.children.iter().filter_map(|c| {
                if let Statement::UIElement(u) = c { Some(u) } else { None }
            }).collect();
            if ui_children.len() != 2 {
                errors.push(SlidesValidationError {
                    component: "TwoColumn".to_string(),
                    context: context.to_string(),
                    reason: format!("requires exactly 2 child blocks, got {}", ui_children.len()),
                });
            }
        }
        "ImageSlide" => {
            let has_src = ui.args.iter().any(|a| matches!(a, Arg::Named(n, _) if n == "src"));
            if !has_src {
                errors.push(SlidesValidationError {
                    component: "ImageSlide".to_string(),
                    context: context.to_string(),
                    reason: "missing required `src` argument".to_string(),
                });
            }
        }
        _ => {}
    }

    // Recurse into children to catch interactive components / nested slides.
    validate_inside_slide(&ui.children, &slide_ctx, errors);
}

fn validate_inside_slide(stmts: &[Statement], context: &str, errors: &mut Vec<SlidesValidationError>) {
    for stmt in stmts {
        match stmt {
            Statement::UIElement(ui) => {
                if let ComponentRef::BuiltIn(name) = &ui.component {
                    if SLIDE_KINDS.contains(&name.as_str()) {
                        errors.push(SlidesValidationError {
                            component: name.clone(),
                            context: context.to_string(),
                            reason: "slide elements cannot be nested inside other slides".to_string(),
                        });
                        continue;
                    }
                    if REJECTED_COMPONENTS.contains(&name.as_str()) {
                        errors.push(SlidesValidationError {
                            component: name.clone(),
                            context: context.to_string(),
                            reason: "interactive / web-only component is not supported in slides".to_string(),
                        });
                        continue;
                    }
                    if SLIDES_INCOMPATIBLE.contains(&name.as_str()) {
                        let why = if matches!(name.as_str(), "Header" | "Footer") {
                            "use slides.footer_text in webfluent.app.json instead"
                        } else {
                            "this component belongs to the PDF document model, not slides"
                        };
                        errors.push(SlidesValidationError {
                            component: name.clone(),
                            context: context.to_string(),
                            reason: why.to_string(),
                        });
                        continue;
                    }
                }
                if !ui.events.is_empty() {
                    errors.push(SlidesValidationError {
                        component: format!("{} (events)", display_name(&ui.component)),
                        context: context.to_string(),
                        reason: "event handlers are not supported in slides".to_string(),
                    });
                }
                validate_inside_slide(&ui.children, context, errors);
            }
            Statement::If(if_stmt) => {
                validate_inside_slide(&if_stmt.then_body, context, errors);
                if let Some(else_body) = &if_stmt.else_body {
                    validate_inside_slide(else_body, context, errors);
                }
            }
            Statement::For(for_stmt) => {
                validate_inside_slide(&for_stmt.body, context, errors);
            }
            Statement::Navigate(_) => errors.push(SlidesValidationError {
                component: "navigate".to_string(),
                context: context.to_string(),
                reason: "navigation is a web-only feature".to_string(),
            }),
            Statement::Fetch(_) => errors.push(SlidesValidationError {
                component: "fetch".to_string(),
                context: context.to_string(),
                reason: "data fetching is a web-only feature".to_string(),
            }),
            Statement::Animate(_) => errors.push(SlidesValidationError {
                component: "animate".to_string(),
                context: context.to_string(),
                reason: "animations are a web-only feature".to_string(),
            }),
            Statement::EventHandler(_) => errors.push(SlidesValidationError {
                component: "event handler".to_string(),
                context: context.to_string(),
                reason: "event handlers are not supported in slides".to_string(),
            }),
            _ => {}
        }
    }
}

fn validate_outside_presentation(stmts: &[Statement], context: &str, errors: &mut Vec<SlidesValidationError>) {
    // Slide-kind components are only valid inside a Presentation; flag them anywhere else.
    for stmt in stmts {
        if let Statement::UIElement(ui) = stmt {
            if let ComponentRef::BuiltIn(name) = &ui.component {
                if SLIDE_KINDS.contains(&name.as_str()) {
                    errors.push(SlidesValidationError {
                        component: name.clone(),
                        context: context.to_string(),
                        reason: "slide elements must be inside a Presentation { ... } block".to_string(),
                    });
                }
            }
            validate_outside_presentation(&ui.children, context, errors);
        }
    }
}

fn display_name(c: &ComponentRef) -> String {
    match c {
        ComponentRef::BuiltIn(n) => n.clone(),
        ComponentRef::SubComponent(p, s) => format!("{}.{}", p, s),
        ComponentRef::UserDefined(n) => n.clone(),
    }
}
