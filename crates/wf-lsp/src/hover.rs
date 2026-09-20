//! Hover: what the thing under the cursor is, said the way the language
//! reference and the registry say it.
//!
//! Resolution reads the syntax tree at the cursor rather than the word
//! alone, so `Button` inside a string gets nothing, `.primary` as a flag
//! gets the prop it sets and `primary` as a prop gets the prop, and `count`
//! is the `count` of the page the cursor is in — not the first `count` in
//! the file.

use tower_lsp::lsp_types::*;
use webfluent::codegen::builtin::{builtin_to_html, implicit_role, landmark_label};
use webfluent::lexer::TokenType;
use webfluent::parser::ast::*;
use webfluent::registry::{self, Children, ComponentSig, Flag, PropSig, PropType, Sink};
use webfluent::sema::types::Type;

use crate::analysis::{self, Binding, BindingKind, ElementPart};
use crate::line_index::word_at;
use crate::project::{Project, SourceFile};
use crate::reference;

pub fn provide_hover(project: &Project, file_ix: usize, position: Position) -> Option<Hover> {
    let file = &project.files[file_ix];
    let source: &str = &file.source;
    let offset = file.index.position_to_offset(source, position)?;
    let tokens = analysis::tokens_of(file).unwrap_or_default();
    if analysis::in_string(&tokens, offset) || analysis::in_comment(source, &tokens, offset) {
        return None;
    }
    let (word, word_range) = word_at(source, offset)?;
    let range = Some(Range::new(
        file.index.offset_to_position(source, word_range.start),
        file.index.offset_to_position(source, word_range.end),
    ));
    let content = hover_text(project, file_ix, offset, word, &tokens)?;
    Some(Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: content,
        }),
        range,
    })
}

fn hover_text(
    project: &Project,
    file_ix: usize,
    offset: usize,
    word: &str,
    tokens: &[webfluent::lexer::Token],
) -> Option<String> {
    let source: &str = &project.files[file_ix].source;
    let token_ix = analysis::token_index_at(tokens, offset);
    let token = token_ix.map(|ix| &tokens[ix]);
    let before = token_ix
        .and_then(|ix| ix.checked_sub(1))
        .map(|ix| &tokens[ix].token_type);

    // `$token`: a design token, wherever it is — as its own token, or
    // inside a raw style value, where the text says so.
    if let Some(TokenType::DesignToken(name)) = token.map(|t| &t.token_type) {
        return Some(design_token_doc(project, name));
    }
    if let Some(TokenType::RawValue(_)) = token.map(|t| &t.token_type) {
        let (word, range) = word_at(source, offset)?;
        if source[..range.start].ends_with('$') {
            return Some(design_token_doc(project, word));
        }
    }

    // `on click`: the event.
    if let Some(TokenType::Identifier(kw)) = before
        && kw == "on"
        && let Some(TokenType::Identifier(name)) = token.map(|t| &t.token_type)
    {
        return Some(event_doc(project, file_ix, offset, name));
    }

    // Inside a declaration: the tree says exactly what the word is.
    if let Some(decl_ix) = analysis::declaration_at(project, file_ix, offset) {
        let decl = &project.program.declarations[decl_ix];

        if let Declaration::Theme(theme) = decl
            && let Some(token) = theme
                .tokens
                .iter()
                .find(|t| analysis::contains(t.span, offset))
            && (token.name == word || word.starts_with(&token.name))
        {
            return Some(token_doc(theme, token));
        }

        if let Declaration::Page(page) = decl
            && analysis::contains(page.header_span, offset)
        {
            if let Some(layout) = &page.layout
                && analysis::contains(layout.span, offset)
                && layout.name == word
                && let Some(decl) = find_declaration(project, word)
            {
                return Some(declaration_doc(project, decl));
            }
            if let Some(a) = reference::PAGE_ATTRIBUTES.iter().find(|a| a.name == word) {
                return Some(format!("**{word}:** — page attribute\n\n{}", a.doc));
            }
            if let Some(param) = page.params.iter().find(|p| p.name == word) {
                return Some(format!(
                    "**{word}** — route parameter of `{}`\n\nBound from `{}`; type `{}`.",
                    page.name,
                    page.path,
                    type_name(&param.prop_type)
                ));
            }
        }

        if let Some(el) = analysis::element_at(analysis::body_of(decl), offset) {
            match analysis::element_part_at(el, source, offset) {
                Some(ElementPart::Name) => return Some(component_doc(project, &el.component)),
                Some(ElementPart::Modifier(ix)) => {
                    let name = &el.modifiers[ix];
                    return Some(flag_doc(project, name, &el.component));
                }
                Some(ElementPart::ArgumentName(name)) => {
                    return Some(argument_doc(project, &el.component, name));
                }
                Some(ElementPart::Style) => {
                    if let Some(doc) = style_doc(word, token.map(|t| &t.token_type)) {
                        return Some(doc);
                    }
                }
                None => {}
            }
            // `tone: .primary` — the case of the prop it is given to.
            if matches!(before, Some(TokenType::Dot))
                && let Some((key, _)) =
                    el.args
                        .iter()
                        .zip(&el.arg_spans)
                        .find_map(|(arg, span)| match arg {
                            Arg::Named(k, Expr::EnumCase(c))
                                if c == word && analysis::contains(*span, offset) =>
                            {
                                Some((k.clone(), c.clone()))
                            }
                            _ => None,
                        })
            {
                return Some(case_doc(project, &el.component, &key, word));
            }
            // A fill of a named slot.
            if let Some(fill) = el
                .slot_fills
                .iter()
                .find(|f| f.name == word && analysis::contains(f.span, offset))
            {
                let owner = analysis::component_name(&el.component);
                return Some(format!(
                    "**{}** — slot of `{owner}`\n\nThe block renders where `{owner}` places `{}`.",
                    fill.name, fill.name
                ));
            }
        }

        // A store member: the word after `Store.`.
        if let Some((store_ix, store)) = analysis::member_owner(project, tokens, offset)
            && let Some(member) = analysis::store_members(store)
                .into_iter()
                .find(|m| m.name == word)
        {
            let store_file = &project.files[project.decl_file[store_ix]];
            let ty = type_of_binding(project, store_ix, &member);
            return Some(binding_doc(store_file, &member, Some(&store.name), ty));
        }

        if let Some(binding) = analysis::scope_at(decl, offset)
            .into_iter()
            .find(|b| b.name == word)
        {
            if binding.kind == BindingKind::Store
                && let Some(store) = find_declaration(project, &binding.name)
            {
                return Some(declaration_doc(project, store));
            }
            let ty = type_of_binding(project, decl_ix, &binding);
            return Some(binding_doc(&project.files[file_ix], &binding, None, ty));
        }

        // A declared event or slot, at its declaration.
        if let Declaration::Component(c) = decl {
            if let Some(e) = c.events.iter().find(|e| e.name == word) {
                return Some(event_decl_doc(c, e));
            }
            if c.slots.iter().any(|s| s.name.as_deref() == Some(word)) {
                return Some(format!(
                    "**{word}** — slot of `{}`\n\nThe caller fills it with `{word} {{ … }}`; it renders where `{word}` stands in the body.",
                    c.name
                ));
            }
        }
    }

    // Names that are global to the project.
    if let Some(sig) = registry::component(word) {
        return Some(builtin_doc(sig));
    }
    if let Some(decl) = find_declaration(project, word) {
        return Some(declaration_doc(project, decl));
    }
    if let Some(keyword) = reference::keyword(word) {
        return Some(format!(
            "**{}**\n\n{}\n\n```wf\n{}\n```",
            keyword.name, keyword.summary, keyword.example
        ));
    }
    None
}

/// A `type` by name, for the fields an extending record inherits.
pub fn find_type<'a>(project: &'a Project, name: &str) -> Option<&'a TypeDecl> {
    project.program.declarations.iter().find_map(|d| match d {
        Declaration::Type(t) if t.name == name => Some(t),
        _ => None,
    })
}

fn find_declaration<'a>(project: &'a Project, name: &str) -> Option<&'a Declaration> {
    project.program.declarations.iter().find(|d| match d {
        Declaration::Page(p) => p.name == name,
        Declaration::Component(c) => c.name == name,
        Declaration::Store(s) => s.name == name,
        Declaration::Theme(t) => t.name == name,
        Declaration::Type(t) => t.name == name,
        Declaration::Enum(e) => e.name == name,
        Declaration::Const(c) => c.name == name,
        Declaration::Animation(a) => a.name == name,
        Declaration::Data(d) => d.name == name,
        Declaration::Test(_) | Declaration::App(_) => false,
    })
}

fn find_component<'a>(project: &'a Project, name: &str) -> Option<&'a ComponentDecl> {
    match find_declaration(project, name) {
        Some(Declaration::Component(c)) => Some(c),
        _ => None,
    }
}

fn find_enum<'a>(project: &'a Project, name: &str) -> Option<&'a EnumDecl> {
    match find_declaration(project, name) {
        Some(Declaration::Enum(e)) => Some(e),
        _ => None,
    }
}

fn signature(reference: &ComponentRef) -> Option<&'static ComponentSig> {
    match reference {
        ComponentRef::BuiltIn(name) => registry::component(name),
        ComponentRef::SubComponent(owner, part) => registry::part(owner, part),
        ComponentRef::UserDefined(_) => None,
    }
}

pub fn type_name(ty: &TypeRef) -> String {
    match ty {
        TypeRef::String => "String".into(),
        TypeRef::Number => "Number".into(),
        TypeRef::Bool => "Bool".into(),
        TypeRef::Map => "Map".into(),
        TypeRef::Any => "Any".into(),
        TypeRef::List(inner) => format!("[{}]", type_name(inner)),
        TypeRef::Optional(inner) => format!("{}?", type_name(inner)),
        TypeRef::Named(name) => name.clone(),
    }
}

fn component_doc(project: &Project, reference: &ComponentRef) -> String {
    match reference {
        ComponentRef::BuiltIn(name) => match registry::component(name) {
            Some(sig) => builtin_doc(sig),
            None => format!("**{name}**"),
        },
        ComponentRef::SubComponent(owner, part) => match registry::part(owner, part) {
            Some(sig) => builtin_doc(sig),
            None => format!("**{owner}.{part}**\n\n`{owner}` has no part called `{part}`."),
        },
        ComponentRef::UserDefined(name) => match find_declaration(project, name) {
            Some(decl) => declaration_doc(project, decl),
            None => format!("**{name}**\n\nNo `component {name}` is declared in this project."),
        },
    }
}

/// How a prop is written, for a usage line.
fn prop_usage(prop: &PropSig) -> String {
    match prop.ty {
        PropType::Bool if prop.shorthand => format!(".{}", prop.name),
        PropType::Enum(cases) => {
            let names: Vec<String> = cases
                .iter()
                .filter(|c| !c.name.is_empty())
                .map(|c| format!(".{}", c.name))
                .collect();
            format!("{}: {}", prop.name, names.join(" | "))
        }
        _ => format!("{}: …", prop.name),
    }
}

fn builtin_doc(sig: &'static ComponentSig) -> String {
    let full = sig.qualified();
    let what = match sig.owner {
        Some(owner) => format!("part of `{owner}`"),
        None => format!("{} component", sig.group),
    };
    let mut out = format!("**{full}** — {what}\n\n{}\n", sig.summary);

    // The usage line: the positional, the block.
    let mut usage = full.clone();
    if let Some(p) = &sig.positional {
        usage.push_str(&format!("({})", p.name));
    }
    if sig.children == Children::Elements {
        usage.push_str(" { … }");
    }
    out.push_str(&format!("\n```wf\n{usage}\n```\n"));

    if let Some(p) = &sig.positional {
        out.push_str(&format!("\nPositional: `{}` — {}  \n", p.name, p.summary));
    }
    let own: Vec<&PropSig> = sig.props.iter().collect();
    if !own.is_empty() {
        out.push_str("\n**Props**\n");
        for p in own {
            out.push_str(&format!("- `{}` — {}\n", prop_usage(p), p.summary));
        }
    }
    let flags: Vec<String> = sig
        .props
        .iter()
        .filter(|p| p.shorthand)
        .flat_map(|p| match p.ty {
            PropType::Bool => vec![format!(".{}", p.name)],
            PropType::Enum(cases) => cases
                .iter()
                .filter(|c| !c.name.is_empty())
                .map(|c| format!(".{}", c.name))
                .collect(),
            _ => Vec::new(),
        })
        .collect();
    if !flags.is_empty() {
        out.push_str(&format!("\n**Flags**: `{}`\n", flags.join("`, `")));
    }
    if !sig.parts.is_empty() {
        let parts: Vec<String> = sig.parts.iter().map(|p| format!("{full}.{p}")).collect();
        out.push_str(&format!("\n**Parts**: `{}`\n", parts.join("`, `")));
    }
    if !sig.events.is_empty() {
        out.push_str(&format!("\n**Events**: `{}`\n", sig.events.join("`, `")));
    }
    out.push_str("\nEvery element also takes `class:`, the motion props (`animate:`, `exit:`, `delay:`, `stagger:`), `aria-*` and `data-*`.\n");

    if let registry::Ir::BuiltIn(ir) = sig.ir {
        let (tag, class) = builtin_to_html(ir);
        let mut html = format!("\nRenders `<{tag}>`");
        if !class.is_empty() {
            html.push_str(&format!(" with class `{class}`"));
        }
        if let Some(role) = implicit_role(ir, &[]) {
            html.push_str(&format!(", role `{role}`"));
        }
        if let Some(landmark) = landmark_label(ir) {
            html.push_str(&format!(" (landmark: {landmark})"));
        }
        out.push_str(&html);
        out.push('.');
    }
    out
}

fn declaration_doc(project: &Project, decl: &Declaration) -> String {
    let file = project
        .program
        .declarations
        .iter()
        .position(|d| std::ptr::eq(d, decl))
        .map(|ix| project.label_of(project.decl_file[ix]))
        .unwrap_or_default();
    let declared = if file.is_empty() {
        String::new()
    } else {
        format!("\n\nDeclared in `{file}`.")
    };
    match decl {
        Declaration::Component(c) => {
            let props = c
                .props
                .iter()
                .map(|p| {
                    let mut s = format!(
                        "{}{}: {}",
                        if p.positional { "_ " } else { "" },
                        p.name,
                        type_name(&p.prop_type)
                    );
                    if p.default.is_some() {
                        s.push_str(" = …");
                    }
                    s
                })
                .collect::<Vec<_>>()
                .join(", ");
            let call = c
                .props
                .iter()
                .map(|p| {
                    if p.positional {
                        "…".to_string()
                    } else {
                        format!("{}: …", p.name)
                    }
                })
                .collect::<Vec<_>>()
                .join(", ");
            let mut usage = if call.is_empty() {
                c.name.clone()
            } else {
                format!("{}({call})", c.name)
            };
            if c.slots.iter().any(|s| s.name.is_none()) {
                usage.push_str(" { … }");
            }
            let header = if props.is_empty() {
                format!("component {}", c.name)
            } else {
                format!("component {}({props})", c.name)
            };
            let mut out = format!("**{}** — component\n\n", c.name);
            if let Some(doc) = &c.doc {
                out.push_str(doc);
                out.push_str("\n\n");
            }
            out.push_str(&format!("```wf\n{header}\n```\n\nCall it as `{usage}`."));
            if !c.events.is_empty() {
                let events: Vec<String> =
                    c.events.iter().map(|e| format!("`{}`", e.name)).collect();
                out.push_str(&format!(
                    "\n\nEvents: {} — handle one with `on name(args) {{ }}`.",
                    events.join(", ")
                ));
            }
            let named: Vec<String> = c
                .slots
                .iter()
                .filter_map(|s| {
                    let name = s.name.clone()?;
                    if s.params.is_empty() {
                        Some(name)
                    } else {
                        let params: Vec<String> = s
                            .params
                            .iter()
                            .map(|p| format!("{}: {}", p.name, type_name(&p.param_type)))
                            .collect();
                        Some(format!("{name}({})", params.join(", ")))
                    }
                })
                .collect();
            if !named.is_empty() {
                out.push_str(&format!(
                    "\n\nSlots: `{}` — fill one with `name {{ … }}`, a scoped one with `name(value) {{ … }}`.",
                    named.join("`, `")
                ));
            }
            if !c.parts.is_empty() {
                let parts: Vec<String> = c
                    .parts
                    .iter()
                    .map(|p| format!("`{}.{p}`", c.name))
                    .collect();
                out.push_str(&format!("\n\nParts: {}.", parts.join(", ")));
            }
            out.push_str(&declared);
            out
        }
        Declaration::Page(p) => {
            let mut out = format!("**{}** — page at `{}`", p.name, p.path);
            if let Some(title) = &p.title {
                out.push_str(&format!("\n\nTitle: {title}"));
            }
            if let Some(description) = &p.description {
                out.push_str(&format!("  \nDescription: {description}"));
            }
            if let Some(layout) = &p.layout {
                out.push_str(&format!("  \nLayout: `{}`", layout.name));
            }
            out.push_str(&declared);
            out
        }
        Declaration::Store(s) => {
            let members = analysis::store_members(s);
            let mut out = format!(
                "**{}** — store\n\n`use {}` brings it into scope; read `{}.member`.",
                s.name, s.name, s.name
            );
            if !members.is_empty() {
                out.push_str("\n\n**Members**\n");
                for m in members {
                    out.push_str(&format!("- `{}` — {}\n", m.name, m.kind.label()));
                }
            }
            out.push_str(&declared);
            out
        }
        Declaration::Theme(t) => format!(
            "**{}** — theme\n\n{} token{} over the baseline. Each is `$name` in a style block and `var(--name)` in a stylesheet.{declared}",
            t.name,
            t.tokens.len(),
            if t.tokens.len() == 1 { "" } else { "s" }
        ),
        Declaration::App(_) => "**app** — the root of the site".to_string(),
        Declaration::Type(t) => format!(
            "**{}** — type{}\n\n{}{declared}",
            t.name,
            t.extends
                .as_ref()
                .map(|b| format!(" = {b} …"))
                .unwrap_or_default(),
            t.all_fields(&|name| find_type(project, name))
                .iter()
                .map(|f| format!("- `{}: {}`", f.name, type_name(&f.ty)))
                .collect::<Vec<_>>()
                .join("\n")
        ),
        Declaration::Enum(e) => format!(
            "**{}** — enum\n\nCases: {}{declared}",
            e.name,
            e.cases
                .iter()
                .map(|c| {
                    if c.fields.is_empty() {
                        format!("`.{}`", c.name)
                    } else {
                        let fields: Vec<String> = c
                            .fields
                            .iter()
                            .map(|f| format!("{}: {}", f.name, type_name(&f.ty)))
                            .collect();
                        format!("`.{}({})`", c.name, fields.join(", "))
                    }
                })
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Declaration::Const(c) => format!(
            "**{}** — const{}{}{declared}",
            c.name,
            c.ty.as_ref()
                .map(|t| format!(": {}", type_name(t)))
                .unwrap_or_default(),
            c.doc
                .as_ref()
                .map(|d| format!("\n\n{d}"))
                .unwrap_or_default()
        ),
        Declaration::Animation(a) => format!(
            "**{}** — animation\n\nKeyframes: {}{}{declared}\n\nPlay it with `animate: .{}`, or in a style as `animation: {} 1s`.",
            a.name,
            a.frames
                .iter()
                .map(|f| format!("`{}`", f.selector))
                .collect::<Vec<_>>()
                .join(", "),
            a.doc
                .as_ref()
                .map(|d| format!("\n\n{d}"))
                .unwrap_or_default(),
            a.name,
            a.name
        ),
        Declaration::Data(d) => format!(
            "**{}** — data from `{}`{}{}{declared}\n\nRead at build time; a constant everywhere.",
            d.name,
            d.file,
            d.ty.as_ref()
                .map(|t| format!(": {}", type_name(t)))
                .unwrap_or_default(),
            d.doc
                .as_ref()
                .map(|doc| format!("\n\n{doc}"))
                .unwrap_or_default()
        ),
        Declaration::Test(t) => format!(
            "**test \"{}\"** — rendered by `wf test`{}{declared}",
            t.name,
            if t.expects.is_empty() {
                String::new()
            } else {
                format!(", held to {} expectation(s)", t.expects.len())
            }
        ),
    }
}

/// The type the checker infers for a binding, when it is more than `Any`.
pub fn type_of_binding(project: &Project, decl_ix: usize, binding: &Binding) -> Option<Type> {
    let info = webfluent::sema::types::check(&project.program, &|_| String::new());
    let ty = info
        .type_at(decl_ix, &binding.name, binding.span)
        .cloned()
        .or_else(|| {
            info.bindings
                .iter()
                .rev()
                .find(|t| t.decl == decl_ix && t.name == binding.name)
                .map(|t| t.ty.clone())
        })?;
    (!ty.is_any()).then_some(ty)
}

fn binding_doc(
    file: &SourceFile,
    binding: &Binding,
    store: Option<&str>,
    ty: Option<Type>,
) -> String {
    let source = declaring_line(file, binding);
    let owner = store
        .map(|s| format!("\n\nMember of store `{s}`."))
        .unwrap_or_default();
    let typed = ty.map(|t| format!("\n\nType `{t}`.")).unwrap_or_default();
    let what = match binding.kind {
        BindingKind::State => {
            "A reactive variable: every element that reads it updates when it changes."
        }
        BindingKind::Derived => "A computed value, recomputed when the state it reads changes.",
        BindingKind::Action => "An action; call it from a handler or another action.",
        BindingKind::Prop => "A prop of this component; the caller passes it by name.",
        BindingKind::Param => "A parameter of this action or handler.",
        BindingKind::LoopItem => "The current item of the enclosing `for`.",
        BindingKind::LoopIndex => "The index of the current item of the enclosing `for`.",
        BindingKind::FetchResult => "The data the enclosing `fetch` loaded.",
        BindingKind::FetchError => "The error the enclosing `fetch` failed with.",
        BindingKind::Store => "A store brought into scope with `use`.",
        BindingKind::Resource => "An async value; render its states with `match`.",
        BindingKind::ArmBinding => "The value the enclosing arm binds.",
    };
    match source {
        Some(line) => format!(
            "**{}** — {}\n\n```wf\n{line}\n```\n\n{what}{typed}{owner}",
            binding.name,
            binding.kind.label()
        ),
        None => format!(
            "**{}** — {}\n\n{what}{typed}{owner}",
            binding.name,
            binding.kind.label()
        ),
    }
}

/// The first line of the statement that declares a binding.
fn declaring_line(file: &SourceFile, binding: &Binding) -> Option<String> {
    let span = binding.span;
    let text = file.source.get(span.start as usize..span.end as usize)?;
    let first = text.lines().next()?.trim();
    (!first.is_empty()).then(|| first.to_string())
}

/// What a `.flag` on an element sets.
fn flag_doc(project: &Project, name: &str, component: &ComponentRef) -> String {
    let component_name = analysis::component_name(component);
    if let Some(sig) = signature(component) {
        return match sig.flag(name) {
            Flag::Bool(prop) => format!(
                "**.{name}** — `{}: true` on `{component_name}`\n\n{}",
                prop.name, prop.summary
            ),
            Flag::Case(prop, case) => {
                let others: Vec<String> = match prop.ty {
                    PropType::Enum(cases) => cases
                        .iter()
                        .filter(|c| !c.name.is_empty() && c.name != case.name)
                        .map(|c| format!(".{}", c.name))
                        .collect(),
                    _ => Vec::new(),
                };
                let mut out = format!(
                    "**.{name}** — `{}: .{name}` on `{component_name}`\n\n{}",
                    prop.name, case.summary
                );
                if !others.is_empty() {
                    out.push_str(&format!(
                        "\n\nThe other cases of `{}`: `{}`.",
                        prop.name,
                        others.join("`, `")
                    ));
                }
                out
            }
            Flag::Ambiguous(props) => format!(
                "**.{name}** — a case of more than one prop of `{component_name}`: `{}`\n\nName the prop: `{}: .{name}`.",
                props.join("`, `"),
                props[0]
            ),
            Flag::Unknown => format!(
                "**.{name}**\n\n`{component_name}` has no flag or case called `{name}`; the compiler rejects it."
            ),
        };
    }
    if let ComponentRef::UserDefined(user) = component
        && let Some(c) = find_component(project, user)
    {
        if c.props.iter().any(|p| p.name == name) {
            return format!(
                "**.{name}** — `{name}: true` on `{}`\n\nA `Bool` prop, set by the flag.",
                c.name
            );
        }
        for prop in &c.props {
            if let TypeRef::Named(enum_name) = &prop.prop_type
                && let Some(e) = find_enum(project, enum_name)
                && e.case(name).is_some()
            {
                return format!(
                    "**.{name}** — `{}: .{name}` on `{}`\n\nA case of `{enum_name}`.",
                    prop.name, c.name
                );
            }
        }
        return format!(
            "**.{name}**\n\n`{}` declares no `Bool` prop or enum case called `{name}`.",
            c.name
        );
    }
    format!("**.{name}** — flag on `{component_name}`")
}

/// The case a named enum prop is given.
fn case_doc(project: &Project, component: &ComponentRef, key: &str, case: &str) -> String {
    let component_name = analysis::component_name(component);
    if let Some(sig) = signature(component)
        && let Some(prop) = sig.prop(key)
        && let PropType::Enum(cases) = prop.ty
    {
        let doc = cases
            .iter()
            .find(|c| c.name == case)
            .map(|c| c.summary)
            .unwrap_or("Not one of its cases.");
        return format!("**.{case}** — `{key}` of `{component_name}`\n\n{doc}");
    }
    if let ComponentRef::UserDefined(user) = component
        && let Some(c) = find_component(project, user)
        && let Some(prop) = c.props.iter().find(|p| p.name == key)
    {
        return format!(
            "**.{case}** — `{key}` of `{}`\n\nA case of `{}`.",
            c.name,
            type_name(&prop.prop_type)
        );
    }
    format!("**.{case}** — `{key}` of `{component_name}`")
}

fn argument_doc(project: &Project, component: &ComponentRef, name: &str) -> String {
    let component_name = analysis::component_name(component);
    if let ComponentRef::UserDefined(user) = component
        && let Some(c) = find_component(project, user)
    {
        if let Some(prop) = c.props.iter().find(|p| p.name == name) {
            let mut out = format!(
                "**{name}:** — prop of `{}`\n\nType `{}`{}.",
                c.name,
                type_name(&prop.prop_type),
                if prop.default.is_some() {
                    ", with a default"
                } else {
                    ""
                }
            );
            if let Some(doc) = &prop.doc {
                out.push_str(&format!("\n\n{doc}"));
            }
            return out;
        }
        if let Some(u) = registry::UNIVERSAL_PROPS.iter().find(|p| p.name == name) {
            return format!("**{name}:** — on every element\n\n{}", u.summary);
        }
        return format!(
            "**{name}:**\n\n`{}` declares no prop called `{name}`; it is passed anyway, but nothing reads it.",
            c.name
        );
    }
    if let Some(sig) = signature(component) {
        if let Some(prop) = sig.prop(name) {
            let how = match (prop.sink, prop.ty) {
                (Sink::Attr, _) => "Written to the element as an attribute.",
                (Sink::Class, _) => "Selects a class of the element.",
                (Sink::Special, PropType::State) => {
                    "A state variable the element reads and writes."
                }
                (Sink::Special, _) => "Read by the element itself.",
            };
            return format!(
                "**{name}:** — `{}` of `{component_name}`\n\n{}\n\n{how}",
                prop_usage(prop),
                prop.summary
            );
        }
        if name.starts_with("aria-") || name.starts_with("data-") {
            return format!(
                "**{name}:** — attribute on `{component_name}`\n\nWritten to the root element. A value that reads state follows it; `aria-*` keeps a `false` as the string `\"false\"`."
            );
        }
        if sig.accepts_named(name) {
            return format!(
                "**{name}:** — HTML attribute on `{component_name}`\n\nWritten to the root element; a value that reads state follows it."
            );
        }
        return format!(
            "**{name}:**\n\n`{component_name}` has no prop called `{name}`; it is written to the element as an attribute, and the compiler warns."
        );
    }
    format!("**{name}:** — attribute on `{component_name}`")
}

fn style_doc(word: &str, token: Option<&TokenType>) -> Option<String> {
    if let Some(TokenType::RawSelector(selector)) = token {
        let doc = reference::SELECTORS
            .iter()
            .find(|(s, _)| *s == selector)
            .map(|(_, d)| *d)
            .unwrap_or("A nested rule, in CSS nesting syntax: `&` is the element");
        return Some(format!(
            "**{selector}** — nested rule\n\n{doc}. Its values must be literals or tokens, not state."
        ));
    }
    if let Some(TokenType::StyleProp(name)) = token {
        if let Some(custom) = name.strip_prefix("--") {
            return Some(format!(
                "**--{custom}** — custom property\n\nA CSS variable on this element; it may read state, and a nested rule can read it with `var(--{custom})`."
            ));
        }
        return Some(format!(
            "**{name}** — CSS property\n\nA raw CSS value to the end of the line, or a `;`. `$name` is a design token; `{{expr}}` reads state and stays live."
        ));
    }
    if let Some(name) = word.strip_prefix("--") {
        return Some(format!(
            "**--{name}** — custom property\n\nA CSS variable on this element; it may read state, and a nested rule can read it with `var(--{name})`."
        ));
    }
    None
}

fn token_doc(theme: &ThemeDecl, token: &ThemeToken) -> String {
    format!(
        "**{}** — design token of theme `{}`\n\n`${}` in a style block; `var(--{})` in a stylesheet.",
        token.name, theme.name, token.name, token.name
    )
}

fn design_token_doc(project: &Project, name: &str) -> String {
    for decl in &project.program.declarations {
        if let Declaration::Theme(t) = decl
            && let Some(token) = t.tokens.iter().find(|tok| tok.name == name)
        {
            return token_doc(t, token);
        }
    }
    if let Some((_, doc)) = reference::TOKENS.iter().find(|(n, _)| *n == name) {
        return format!(
            "**${name}** — baseline design token\n\n{doc}. Compiles to `var(--{name})`; a theme may override it."
        );
    }
    let group_hint = reference::TOKENS
        .iter()
        .filter(|(n, _)| n.ends_with(&format!("-{name}")))
        .map(|(n, _)| format!("`${n}`"))
        .collect::<Vec<_>>();
    if !group_hint.is_empty() {
        return format!(
            "**${name}** — a short token name\n\nResolves through the property's group: {}. Compiles to `var(--{name})` where no group applies.",
            group_hint.join(", ")
        );
    }
    format!(
        "**${name}**\n\nCompiles to `var(--{name})`. No theme in this project declares it; if it is a custom property set on an element, write `var(--{name})`."
    )
}

fn event_doc(project: &Project, file_ix: usize, offset: usize, name: &str) -> String {
    // A component's own event, when the handler sits on a component.
    if let Some(decl_ix) = analysis::declaration_at(project, file_ix, offset) {
        let decl = &project.program.declarations[decl_ix];
        if let Some(el) = analysis::element_at(analysis::body_of(decl), offset)
            && let ComponentRef::UserDefined(user) = &el.component
            && let Some(c) = find_component(project, user)
            && let Some(e) = c.events.iter().find(|e| e.name == name)
        {
            return event_decl_doc(c, e);
        }
    }
    let what = reference::events()
        .iter()
        .find(|(e, _)| *e == name)
        .map(|(_, d)| *d)
        .unwrap_or("A DOM event of this name");
    format!(
        "**on {name}** — event handler\n\n{what}. `on {name}(event) {{ }}` names the DOM event (`event.target`, `event.key`)."
    )
}

fn event_decl_doc(c: &ComponentDecl, e: &EventDecl) -> String {
    let params: Vec<String> = e
        .params
        .iter()
        .map(|p| format!("{}: {}", p.name, type_name(&p.param_type)))
        .collect();
    let mut out = format!(
        "**{}** — event of `{}`\n\n```wf\nevent {}({})\n```\n\nFired with `emit {}(…)`; handled at the call with `on {}({}) {{ }}`.",
        e.name,
        c.name,
        e.name,
        params.join(", "),
        e.name,
        e.name,
        e.params
            .iter()
            .map(|p| p.name.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    );
    if let Some(doc) = &e.doc {
        out.push_str(&format!("\n\n{doc}"));
    }
    out
}
