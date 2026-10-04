//! Validation for slides output: enforces deck structure and rejects
//! interactive/web-only components (same list PDF rejects, plus a few extras
//! that don't fit the slides model like `Header`/`Footer`).

use super::pdf_validation::drawn_in_pdf;
use crate::parser::{Arg, ComponentRef, Declaration, Program, Statement, StatementKind, UIElement};

pub(crate) const SLIDE_KINDS: &[&str] = &[
    "Slide",
    "TitleSlide",
    "SectionSlide",
    "TwoColumn",
    "ImageSlide",
];

/// Components that don't make sense inside a slide deck.
/// `Header`/`Footer` are rejected because slides have their own footer chrome via config.
/// `Document`/`Section`/`Paragraph`/`PageBreak` belong to the PDF document model.
pub(crate) const SLIDES_INCOMPATIBLE: &[&str] =
    &["Header", "Footer", "Document", "Paragraph", "PageBreak"];

#[derive(Debug)]
pub struct SlidesValidationError {
    pub component: String,
    pub context: String,
    pub reason: String,
    /// The declaration it is in, by index, and where.
    pub decl: usize,
    pub span: crate::parser::ast::Span,
}

impl std::fmt::Display for SlidesValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "error[slides]: '{}' in {} — {}",
            self.component, self.context, self.reason
        )
    }
}

pub fn validate_for_slides(program: &Program) -> Vec<SlidesValidationError> {
    let mut errors = Vec::new();

    for (ix, decl) in program.declarations.iter().enumerate() {
        match decl {
            Declaration::Page(page) => {
                let ctx = format!("Page {}", page.name);
                validate_page_body(program, &page.body, &ctx, ix, &mut errors);
            }
            // A component whose body is slides is a slide of its own, placed
            // in a Presentation like a `Slide`.
            Declaration::Component(comp) if renders_slides(&comp.body) => {
                let ctx = format!("Component {}", comp.name);
                validate_presentation_children(program, &comp.body, &ctx, ix, &mut errors);
            }
            Declaration::Component(comp) => {
                let ctx = format!("Component {}", comp.name);
                validate_outside_presentation(&comp.body, &ctx, ix, &mut errors);
            }
            Declaration::App(app) => {
                validate_outside_presentation(&app.body, "App", ix, &mut errors);
            }
            _ => {}
        }
    }

    errors
}

/// Whether a body draws slides and nothing else: slide elements, or `for`
/// and `if` over them. Declarations (`state`, `let`) draw nothing.
pub fn renders_slides(stmts: &[Statement]) -> bool {
    let mut any = false;
    for stmt in stmts {
        match &stmt.kind {
            StatementKind::UIElement(ui) => match &ui.component {
                ComponentRef::BuiltIn(n) if SLIDE_KINDS.contains(&n.as_str()) => any = true,
                _ => return false,
            },
            StatementKind::For(f) => {
                if !renders_slides(&f.body) {
                    return false;
                }
                any = true;
            }
            StatementKind::If(i) => {
                let else_ok = i
                    .else_body
                    .as_ref()
                    .is_none_or(|b| b.is_empty() || renders_slides(b));
                if !(renders_slides(&i.then_body) && else_ok) {
                    return false;
                }
                any = true;
            }
            _ => {}
        }
    }
    any
}

/// Whether the program declares a component of that name whose body is slides.
fn slide_component(program: &Program, name: &str) -> bool {
    program.declarations.iter().any(
        |d| matches!(d, Declaration::Component(c) if c.name == name && renders_slides(&c.body)),
    )
}

fn validate_page_body(
    program: &Program,
    stmts: &[Statement],
    context: &str,
    decl: usize,
    errors: &mut Vec<SlidesValidationError>,
) {
    for stmt in stmts {
        if let StatementKind::UIElement(ui) = &stmt.kind
            && let ComponentRef::BuiltIn(name) = &ui.component
        {
            if name == "Presentation" {
                validate_presentation_children(program, &ui.children, context, decl, errors);
                continue;
            }
            if SLIDE_KINDS.contains(&name.as_str()) {
                errors.push(SlidesValidationError {
                    decl,
                    span: ui.span,
                    component: name.clone(),
                    context: context.to_string(),
                    reason: "slide elements must be inside a Presentation { ... } block"
                        .to_string(),
                });
                continue;
            }
        }
        // Anything else outside a Presentation is allowed but ignored at codegen time.
        // We could warn here but choose not to in v1.
    }
}

fn validate_presentation_children(
    program: &Program,
    stmts: &[Statement],
    context: &str,
    decl: usize,
    errors: &mut Vec<SlidesValidationError>,
) {
    for stmt in stmts {
        match &stmt.kind {
            StatementKind::UIElement(ui) => {
                let name = match &ui.component {
                    ComponentRef::BuiltIn(n) => n.clone(),
                    ComponentRef::UserDefined(n) if slide_component(program, n) => continue,
                    _ => {
                        errors.push(SlidesValidationError {
                decl,
                span: ui.span,
                            component: format!("{:?}", ui.component),
                            context: context.to_string(),
                            reason: "Presentation may only contain Slide / TitleSlide / SectionSlide / TwoColumn / ImageSlide, or a component whose body is slides".to_string(),
                        });
                        continue;
                    }
                };
                if !SLIDE_KINDS.contains(&name.as_str()) {
                    errors.push(SlidesValidationError {
                decl,
                span: ui.span,
                        component: name,
                        context: context.to_string(),
                        reason: "Presentation may only contain Slide / TitleSlide / SectionSlide / TwoColumn / ImageSlide".to_string(),
                    });
                    continue;
                }
                validate_slide_kind(&name, ui, context, decl, errors);
            }
            // Slides made from data: a `for` or an `if` whose body is slides.
            StatementKind::For(f) => {
                validate_presentation_children(program, &f.body, context, decl, errors)
            }
            StatementKind::If(i) => {
                validate_presentation_children(program, &i.then_body, context, decl, errors);
                if let Some(else_body) = &i.else_body {
                    validate_presentation_children(program, else_body, context, decl, errors);
                }
            }
            // A declaration draws nothing.
            StatementKind::State(..) | StatementKind::Derived(..) => {}
            _ => {
                errors.push(SlidesValidationError {
                    decl,
                    span: stmt.span,
                    component: "non-slide statement".to_string(),
                    context: context.to_string(),
                    reason: "Presentation children must be slides, or `for`/`if` over them"
                        .to_string(),
                });
            }
        }
    }
}

fn validate_slide_kind(
    name: &str,
    ui: &UIElement,
    context: &str,
    decl: usize,
    errors: &mut Vec<SlidesValidationError>,
) {
    let slide_ctx = format!("{} > {}", context, name);

    match name {
        "TwoColumn" => {
            let ui_children: Vec<&UIElement> = ui
                .children
                .iter()
                .filter_map(|c| {
                    if let StatementKind::UIElement(u) = &c.kind {
                        Some(u)
                    } else {
                        None
                    }
                })
                .collect();
            if ui_children.len() != 2 {
                errors.push(SlidesValidationError {
                    decl,
                    span: ui.span,
                    component: "TwoColumn".to_string(),
                    context: context.to_string(),
                    reason: format!("requires exactly 2 child blocks, got {}", ui_children.len()),
                });
            }
        }
        "ImageSlide" => {
            let has_src = ui
                .args
                .iter()
                .any(|a| matches!(a, Arg::Named(n, _) if n == "src"));
            if !has_src {
                errors.push(SlidesValidationError {
                    decl,
                    span: ui.span,
                    component: "ImageSlide".to_string(),
                    context: context.to_string(),
                    reason: "missing required `src` argument".to_string(),
                });
            }
        }
        _ => {}
    }

    // Recurse into children to catch interactive components / nested slides.
    validate_inside_slide(&ui.children, &slide_ctx, decl, errors);
}

fn validate_inside_slide(
    stmts: &[Statement],
    context: &str,
    decl: usize,
    errors: &mut Vec<SlidesValidationError>,
) {
    for stmt in stmts {
        match &stmt.kind {
            StatementKind::UIElement(ui) => {
                if let ComponentRef::BuiltIn(name) = &ui.component {
                    if SLIDE_KINDS.contains(&name.as_str()) {
                        errors.push(SlidesValidationError {
                            decl,
                            span: ui.span,
                            component: name.clone(),
                            context: context.to_string(),
                            reason: "slide elements cannot be nested inside other slides"
                                .to_string(),
                        });
                        continue;
                    }
                    if !drawn_in_pdf(name) {
                        errors.push(SlidesValidationError {
                            decl,
                            span: ui.span,
                            component: name.clone(),
                            context: context.to_string(),
                            reason: "interactive / web-only component is not supported in slides"
                                .to_string(),
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
                            decl,
                            span: ui.span,
                            component: name.clone(),
                            context: context.to_string(),
                            reason: why.to_string(),
                        });
                        continue;
                    }
                }
                if !ui.events.is_empty() {
                    errors.push(SlidesValidationError {
                        decl,
                        span: ui.span,
                        component: format!("{} (events)", display_name(&ui.component)),
                        context: context.to_string(),
                        reason: "event handlers are not supported in slides".to_string(),
                    });
                }
                validate_inside_slide(&ui.children, context, decl, errors);
            }
            StatementKind::If(if_stmt) => {
                validate_inside_slide(&if_stmt.then_body, context, decl, errors);
                if let Some(else_body) = &if_stmt.else_body {
                    validate_inside_slide(else_body, context, decl, errors);
                }
            }
            StatementKind::For(for_stmt) => {
                validate_inside_slide(&for_stmt.body, context, decl, errors);
            }
            StatementKind::Navigate(_) => errors.push(SlidesValidationError {
                decl,
                span: stmt.span,
                component: "navigate".to_string(),
                context: context.to_string(),
                reason: "navigation is a web-only feature".to_string(),
            }),
            StatementKind::Fetch(_) => errors.push(SlidesValidationError {
                decl,
                span: stmt.span,
                component: "fetch".to_string(),
                context: context.to_string(),
                reason: "data fetching is a web-only feature".to_string(),
            }),
            StatementKind::Animate(_) => errors.push(SlidesValidationError {
                decl,
                span: stmt.span,
                component: "animate".to_string(),
                context: context.to_string(),
                reason: "animations are a web-only feature".to_string(),
            }),
            StatementKind::EventHandler(_) => errors.push(SlidesValidationError {
                decl,
                span: stmt.span,
                component: "event handler".to_string(),
                context: context.to_string(),
                reason: "event handlers are not supported in slides".to_string(),
            }),
            _ => {}
        }
    }
}

fn validate_outside_presentation(
    stmts: &[Statement],
    context: &str,
    decl: usize,
    errors: &mut Vec<SlidesValidationError>,
) {
    // Slide-kind components are only valid inside a Presentation; flag them anywhere else.
    for stmt in stmts {
        if let StatementKind::UIElement(ui) = &stmt.kind {
            if let ComponentRef::BuiltIn(name) = &ui.component
                && SLIDE_KINDS.contains(&name.as_str())
            {
                errors.push(SlidesValidationError {
                    decl,
                    span: ui.span,
                    component: name.clone(),
                    context: context.to_string(),
                    reason: "slide elements must be inside a Presentation { ... } block"
                        .to_string(),
                });
            }
            validate_outside_presentation(&ui.children, context, decl, errors);
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
