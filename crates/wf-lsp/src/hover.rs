//! Hover: what the thing under the cursor is, said the way the language
//! reference says it.
//!
//! Resolution reads the syntax tree at the cursor rather than the word
//! alone, so `Button` inside a string gets nothing, `primary` as a modifier
//! gets the modifier and `primary` as a prop gets the prop, and `count` is the
//! `count` of the page the cursor is in — not the first `count` in the file.

use tower_lsp::lsp_types::*;
use webfluent::codegen::builtin::{builtin_to_html, implicit_role, landmark_label};
use webfluent::lexer::TokenType;
use webfluent::parser::ast::*;

use crate::analysis::{self, Binding, BindingKind, ElementPart};
use crate::line_index::word_at;
use crate::project::{Project, SourceFile};
use crate::reference::{self, ComponentDoc};

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
    if let Some(token) = analysis::token_at(tokens, offset)
        && let TokenType::Event(name) = &token.token_type
    {
        return Some(event_doc(name));
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

        if let Some(el) = analysis::element_at(analysis::body_of(decl), offset) {
            let source: &str = &project.files[file_ix].source;
            match analysis::element_part_at(el, source, offset) {
                Some(ElementPart::Name) => return Some(component_doc(project, &el.component)),
                Some(ElementPart::Modifier(ix)) => {
                    let name = &el.modifiers[ix];
                    return Some(modifier_doc(name, &el.component));
                }
                Some(ElementPart::ArgumentName(name)) => {
                    return Some(argument_doc(project, &el.component, name));
                }
                Some(ElementPart::Style) => {
                    if let Some(doc) = style_doc(word) {
                        return Some(doc);
                    }
                }
                None => {}
            }
        }

        // A store member: the word after `Store.`.
        if let Some((store_ix, store)) = analysis::member_owner(project, tokens, offset)
            && let Some(member) = analysis::store_members(store)
                .into_iter()
                .find(|m| m.name == word)
        {
            let store_file = &project.files[project.decl_file[store_ix]];
            return Some(binding_doc(store_file, &member, Some(&store.name)));
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
            return Some(binding_doc(&project.files[file_ix], &binding, None));
        }
    }

    // Names that are global to the project.
    if let Some(doc) = reference::component(word) {
        return Some(builtin_doc(doc));
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
    if let Some((group, doc)) = reference::modifier_doc(word) {
        return Some(format!("**{word}** — {group} modifier\n\n{doc}"));
    }
    None
}

fn find_declaration<'a>(project: &'a Project, name: &str) -> Option<&'a Declaration> {
    project.program.declarations.iter().find(|d| match d {
        Declaration::Page(p) => p.name == name,
        Declaration::Component(c) => c.name == name,
        Declaration::Store(s) => s.name == name,
        Declaration::Theme(t) => t.name == name,
        Declaration::App(_) => false,
    })
}

fn component_doc(project: &Project, reference: &ComponentRef) -> String {
    match reference {
        ComponentRef::BuiltIn(name) => match reference::component(name) {
            Some(doc) => builtin_doc(doc),
            None => format!("**{name}**"),
        },
        ComponentRef::SubComponent(parent, child) => {
            let full = format!("{parent}.{child}");
            match reference::sub_component(&full) {
                Some((doc, _)) => format!(
                    "**{full}** — part of `{}`\n\n{}\n\n```wf\n{}\n```",
                    doc.name, doc.summary, doc.usage
                ),
                None => format!("**{full}**\n\nSub-component of `{parent}`."),
            }
        }
        ComponentRef::UserDefined(name) => match find_declaration(project, name) {
            Some(decl) => declaration_doc(project, decl),
            None => format!("**{name}**\n\nNo `Component {name}` is declared in this project."),
        },
    }
}

fn builtin_doc(doc: &ComponentDoc) -> String {
    let (tag, class) = builtin_to_html(doc.name);
    let mut out = format!(
        "**{}** — {} component\n\n{}\n\n```wf\n{}\n```\n",
        doc.name, doc.group, doc.summary, doc.usage
    );
    if !doc.positional.is_empty() {
        out.push_str(&format!("\nPositional: {}  \n", doc.positional));
    }
    if !doc.args.is_empty() {
        out.push_str("\n**Arguments**\n");
        for a in doc.args {
            out.push_str(&format!("- `{}:` — {}\n", a.name, a.doc));
        }
        out.push_str("\nAny other `name: value` becomes an attribute on the element.\n");
    }
    if !doc.modifiers.is_empty() {
        out.push_str(&format!(
            "\n**Modifiers**: `{}`\n",
            doc.modifiers.join("`, `")
        ));
    }
    if !doc.children.is_empty() {
        out.push_str(&format!(
            "\n**Sub-components**: `{}`\n",
            doc.children.join("`, `")
        ));
    }
    let mut html = format!("\nRenders `<{tag}>`");
    if !class.is_empty() {
        html.push_str(&format!(" with class `{class}`"));
    }
    if let Some(role) = implicit_role(doc.name, &[]) {
        html.push_str(&format!(", role `{role}`"));
    }
    if let Some(landmark) = landmark_label(doc.name) {
        html.push_str(&format!(" (landmark: {landmark})"));
    }
    out.push_str(&html);
    out.push('.');
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
                        "{}{}: {:?}",
                        p.name,
                        if p.optional { "?" } else { "" },
                        p.prop_type
                    );
                    if p.default.is_some() {
                        s.push_str(" = …");
                    }
                    s
                })
                .collect::<Vec<_>>()
                .join(", ");
            let usage = if c.props.is_empty() {
                format!("{} {{ … }}", c.name)
            } else {
                let call = c
                    .props
                    .iter()
                    .map(|p| format!("{}: …", p.name))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}({call})", c.name)
            };
            format!(
                "**{}** — component\n\n```wf\nComponent {} ({props})\n```\n\nCall it with named arguments: `{usage}`.{declared}",
                c.name, c.name
            )
        }
        Declaration::Page(p) => {
            let mut out = format!("**{}** — page at `{}`", p.name, p.path);
            if let Some(title) = &p.title {
                out.push_str(&format!("\n\nTitle: {title}"));
            }
            if let Some(description) = &p.description {
                out.push_str(&format!("  \nDescription: {description}"));
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
            "**{}** — theme\n\n{} token{} over the baseline. Each is `var(--name)` in any style.{declared}",
            t.name,
            t.tokens.len(),
            if t.tokens.len() == 1 { "" } else { "s" }
        ),
        Declaration::App(_) => "**App** — the root of the site".to_string(),
    }
}

fn binding_doc(file: &SourceFile, binding: &Binding, store: Option<&str>) -> String {
    let source = declaring_line(file, binding);
    let owner = store
        .map(|s| format!("\n\nMember of store `{s}`."))
        .unwrap_or_default();
    let what = match binding.kind {
        BindingKind::State => {
            "A reactive variable: every element that reads it updates when it changes."
        }
        BindingKind::Derived => "A computed value, recomputed when the state it reads changes.",
        BindingKind::Action => "An action; call it from a handler or another action.",
        BindingKind::Prop => "A prop of this component; the caller passes it by name.",
        BindingKind::Param => "A parameter of this action.",
        BindingKind::LoopItem => "The current item of the enclosing `for`.",
        BindingKind::LoopIndex => "The index of the current item of the enclosing `for`.",
        BindingKind::FetchResult => "The data the enclosing `fetch` loaded.",
        BindingKind::FetchError => "The error the enclosing `fetch` failed with.",
        BindingKind::Store => "A store brought into scope with `use`.",
    };
    match source {
        Some(line) => format!(
            "**{}** — {}\n\n```wf\n{line}\n```\n\n{what}{owner}",
            binding.name,
            binding.kind.label()
        ),
        None => format!(
            "**{}** — {}\n\n{what}{owner}",
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

fn modifier_doc(name: &str, component: &ComponentRef) -> String {
    let component_name = analysis::component_name(component);
    match reference::modifier_doc(name) {
        Some((group, doc)) => {
            let accepted = reference::component(&component_name)
                .filter(|c| !c.modifiers.is_empty())
                .map(|c| {
                    if c.modifiers.contains(&name) {
                        format!("\n\n`{}` lists it.", c.name)
                    } else {
                        format!(
                            "\n\n`{}` does not list it; it is still applied as a class.",
                            c.name
                        )
                    }
                })
                .unwrap_or_default();
            format!("**{name}** — {group} modifier on `{component_name}`\n\n{doc}{accepted}")
        }
        None => format!(
            "**{name}**\n\nA bare word `{component_name}` does not recognise; the compiler warns (V01)."
        ),
    }
}

fn argument_doc(project: &Project, component: &ComponentRef, name: &str) -> String {
    let component_name = analysis::component_name(component);
    if let ComponentRef::UserDefined(user) = component
        && let Some(Declaration::Component(c)) = find_declaration(project, user)
    {
        if let Some(prop) = c.props.iter().find(|p| p.name == name) {
            return format!(
                "**{name}:** — prop of `{}`\n\nType `{:?}`{}{}.",
                c.name,
                prop.prop_type,
                if prop.optional { ", optional" } else { "" },
                if prop.default.is_some() {
                    ", with a default"
                } else {
                    ""
                }
            );
        }
        return format!(
            "**{name}:**\n\n`{}` declares no prop called `{name}`; it becomes an attribute on its root element.",
            c.name
        );
    }
    if (component_name == "Route" || component_name == "Page")
        && let Some(a) = reference::PAGE_ATTRIBUTES.iter().find(|a| a.name == name)
    {
        return format!("**{name}:** — page attribute\n\n{}", a.doc);
    }
    if let Some(doc) = reference::component(&component_name)
        && let Some(a) = doc.args.iter().find(|a| a.name == name)
    {
        return format!("**{name}:** — argument of `{}`\n\n{}", doc.name, a.doc);
    }
    if name.starts_with("aria-") || name.starts_with("data-") {
        return format!(
            "**{name}:** — attribute on `{component_name}`\n\nWritten to the root element. A value that reads state follows it; `aria-*` keeps a `false` as the string `\"false\"`."
        );
    }
    if name == "bind" {
        return format!(
            "**bind:** — argument of `{component_name}`\n\nState variable the control reads and writes."
        );
    }
    format!(
        "**{name}:** — attribute on `{component_name}`\n\nNot one `{component_name}` reads itself, so it is written to the root element as an HTML attribute."
    )
}

fn style_doc(word: &str) -> Option<String> {
    if reference::pseudo_states().contains(&word) {
        let extra = match word {
            "focus" => {
                " Compiled to `:focus-visible`: the keyboard focus ring, not a ring on every click."
            }
            "current" => {
                " Matches `aria-current=\"page\"`, which the router sets on the active link."
            }
            "pressed" | "selected" | "checked" | "expanded" | "invalid" => {
                " Matches the `aria-` attribute of the same name being `\"true\"`."
            }
            _ => "",
        };
        return Some(format!(
            "**{word}** — pseudo-state\n\nA stylesheet rule that applies while the element is `{word}`; values must be literals or token keywords, not state.{extra}"
        ));
    }
    if let Some(name) = word.strip_prefix("--") {
        return Some(format!(
            "**--{name}** — custom property\n\nA CSS variable on this element; it may read state, and a pseudo-state rule can read it with `var(--{name})`."
        ));
    }
    None
}

fn token_doc(theme: &ThemeDecl, token: &ThemeToken) -> String {
    format!(
        "**{}** — design token of theme `{}`\n\nAvailable in any style as `var(--{})`.",
        token.name, theme.name, token.name
    )
}

fn event_doc(name: &str) -> String {
    let what = reference::EVENTS
        .iter()
        .find(|(e, _)| *e == name)
        .map(|(_, d)| *d)
        .unwrap_or("A DOM event of this name");
    format!(
        "**on:{name}** — event handler\n\n{what}. Inside the block, `event` is the DOM event (`event.target`, `event.currentTarget`)."
    )
}
