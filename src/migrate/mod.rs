//! `wf migrate`: a project written in the original grammar, rewritten in
//! WebFluent 3.
//!
//! The rewrite is a set of text patches on spans of the original parse —
//! never a reprint of the tree — so comments, blank lines and the author's
//! formatting survive, and only what the new grammar spells differently
//! changes. It runs in two rounds, each a parse of the original grammar:
//!
//! 1. **Order.** A block's clauses are moved to where the new grammar
//!    requires them — `style`, `transition`, handlers, then children — and
//!    a Button's or component's action shorthand is wrapped in a handler.
//!    The result is still the original grammar.
//! 2. **Spelling.** Declarations, modifiers, parts, arguments, handlers,
//!    style values, animate clauses, fetches, routes and stores are
//!    respelled. The result parses as WebFluent 3, and is checked to.
//!
//! What has no mechanical spelling is left as it was and reported as a
//! note, so a migrated project builds or says exactly what is left to do.

use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

use crate::edit::{Patch, apply_patches, modifier_removal_range};
use crate::error::{Result, WebFluentError};
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::parser::ast::*;
use crate::registry;
use crate::syntax::{Dialect, detect_dialect};

/// Something the migration could not rewrite mechanically.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Note {
    pub line: usize,
    pub column: usize,
    pub message: String,
}

/// The outcome for one file.
#[derive(Debug)]
pub struct Migrated {
    /// The rewritten source, or the original when the file was already in
    /// the new grammar.
    pub text: String,
    pub notes: Vec<Note>,
    /// Whether `text` differs from the input.
    pub changed: bool,
}

/// What the migration of one file needs to know about the others.
#[derive(Debug, Default, Clone)]
pub struct ProjectContext {
    /// Components called with a positional argument somewhere in the
    /// project: their first prop becomes the positional one.
    pub positional_components: Vec<String>,
    /// The path each page was routed at by a `Route`, when the app wired
    /// one.
    pub route_paths: HashMap<String, String>,
    /// The design tokens a style value may name with `$`: the project's
    /// themes' and the built-in ones. Any other `var(--x)` is a custom
    /// property of the element's own and stays as written.
    pub tokens: std::collections::HashSet<String>,
    /// The states a `DatePicker` binds. A date picker holds a `Date` now,
    /// not a string that looks like one, so each of these is annotated and
    /// its starting value written as a date.
    pub bound_dates: std::collections::HashSet<String>,
}

/// Read the whole project once, so a file's rewrite can see what the others
/// declare and call.
pub fn project_context(files: &[(PathBuf, String)]) -> ProjectContext {
    let mut ctx = ProjectContext::default();
    ctx.tokens
        .extend(crate::themes::tokens::default_tokens().into_keys());
    for (path, source) in files {
        if detect_dialect(source) != Dialect::V1 {
            continue;
        }
        let Ok(program) = parse_v1(source, &path.to_string_lossy()) else {
            continue;
        };
        for decl in &program.declarations {
            let body = match decl {
                Declaration::Page(p) => &p.body,
                Declaration::Component(c) => &c.body,
                Declaration::App(a) => {
                    for route in routes_in(&a.body) {
                        if let (Some(path), Some(page)) = (route_path(route), route_page(route)) {
                            ctx.route_paths.insert(page.to_string(), path.to_string());
                        }
                    }
                    &a.body
                }
                Declaration::Theme(t) => {
                    ctx.tokens
                        .extend(t.tokens.iter().map(|tok| tok.name.clone()));
                    continue;
                }
                _ => continue,
            };
            collect_positional_calls(body, &mut ctx.positional_components);
            collect_bound_dates(body, &mut ctx.bound_dates);
        }
    }
    ctx.positional_components.sort();
    ctx.positional_components.dedup();
    ctx
}

/// The names a `DatePicker(bind: …)` binds, anywhere in the body.
fn collect_bound_dates(stmts: &[Statement], out: &mut std::collections::HashSet<String>) {
    for stmt in stmts {
        for body in child_bodies(stmt) {
            collect_bound_dates(body, out);
        }
        let StatementKind::UIElement(el) = &stmt.kind else {
            continue;
        };
        if !matches!(&el.component, ComponentRef::BuiltIn(n) if n == "DatePicker") {
            continue;
        }
        for arg in &el.args {
            if let Arg::Named(key, Expr::Identifier(name)) = arg
                && key == "bind"
            {
                out.insert(name.clone());
            }
        }
    }
}

fn collect_positional_calls(stmts: &[Statement], out: &mut Vec<String>) {
    for stmt in stmts {
        for body in child_bodies(stmt) {
            collect_positional_calls(body, out);
        }
        if let StatementKind::UIElement(el) = &stmt.kind
            && let ComponentRef::UserDefined(name) = &el.component
            && el.args.iter().any(|a| matches!(a, Arg::Positional(_)))
        {
            out.push(name.clone());
        }
    }
}

/// Migrate every file of a project. Files already in the new grammar are
/// returned unchanged.
pub fn migrate_project(files: &[(PathBuf, String)]) -> Vec<(PathBuf, Result<Migrated>)> {
    let ctx = project_context(files);
    files
        .iter()
        .map(|(path, source)| {
            (
                path.clone(),
                migrate_file(source, &path.to_string_lossy(), &ctx),
            )
        })
        .collect()
}

/// Migrate one file.
pub fn migrate_file(source: &str, file: &str, ctx: &ProjectContext) -> Result<Migrated> {
    if detect_dialect(source) == Dialect::V2 {
        return Ok(Migrated {
            text: source.to_string(),
            notes: Vec::new(),
            changed: false,
        });
    }
    let mut notes = Vec::new();

    // Round 1: order.
    let program = parse_v1(source, file)?;
    let ordered = {
        let mut r = Rewriter::new(source, file, ctx, &mut notes);
        r.order_program(&program);
        r.finish()?
    };

    // Round 2: spelling.
    let program = parse_v1(&ordered, file).map_err(|e| {
        WebFluentError::EditError(format!(
            "the block reordering produced text the original grammar rejects: {e}"
        ))
    })?;
    let text = {
        let mut r = Rewriter::new(&ordered, file, ctx, &mut notes);
        r.spell_program(&program);
        r.finish()?
    };

    // The result must be WebFluent 3.
    if let Err(e) = crate::parser::v2::parse_v2(&text, file) {
        return Err(WebFluentError::EditError(format!(
            "the migration of {file} produced text WebFluent 3 rejects — a bug in wf migrate, not in the project: {e}"
        )));
    }
    notes.sort_by_key(|n| (n.line, n.column));
    notes.dedup();
    Ok(Migrated {
        changed: text != source,
        text,
        notes,
    })
}

fn parse_v1(source: &str, file: &str) -> Result<Program> {
    let tokens = Lexer::new(source, file).tokenize()?;
    Parser::new(tokens, file).parse()
}

// ─── The rewriter ────────────────────────────────────────

struct Rewriter<'a> {
    source: &'a str,
    ctx: &'a ProjectContext,
    notes: &'a mut Vec<Note>,
    patches: Vec<Patch>,
    /// Flags and named arguments an `animate(…)` clause moves onto the
    /// root elements of a branch, keyed by the element's span start.
    lifted: HashMap<u32, (Vec<String>, Vec<String>)>,
}

impl<'a> Rewriter<'a> {
    fn new(
        source: &'a str,
        _file: &str,
        ctx: &'a ProjectContext,
        notes: &'a mut Vec<Note>,
    ) -> Self {
        Self {
            source,
            ctx,
            notes,
            patches: Vec::new(),
            lifted: HashMap::new(),
        }
    }

    fn finish(self) -> Result<String> {
        apply_patches(self.source, self.patches)
    }

    fn patch(&mut self, start: usize, end: usize, text: impl Into<String>) {
        self.patches.push(Patch {
            start,
            end,
            text: text.into(),
        });
    }

    fn replace(&mut self, span: Span, text: impl Into<String>) {
        self.patch(span.start as usize, span.end as usize, text);
    }

    fn slice(&self, span: Span) -> &'a str {
        &self.source[span.start as usize..span.end as usize]
    }

    fn note(&mut self, span: Span, message: impl Into<String>) {
        self.notes.push(Note {
            line: span.line as usize,
            column: span.col as usize,
            message: message.into(),
        });
    }

    /// The indentation of the line `pos` is on.
    fn indent_at(&self, pos: usize) -> String {
        let line_start = self.source[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
        self.source[line_start..pos]
            .chars()
            .take_while(|c| *c == ' ' || *c == '\t')
            .collect()
    }

    // ─── Round 1: order ──────────────────────────────────

    fn order_program(&mut self, program: &Program) {
        for decl in &program.declarations {
            match decl {
                Declaration::Page(p) => self.order_statements(&p.body),
                Declaration::Component(c) => self.order_statements(&c.body),
                Declaration::App(a) => self.order_statements(&a.body),
                _ => {}
            }
        }
    }

    fn order_statements(&mut self, stmts: &[Statement]) {
        for stmt in stmts {
            if let StatementKind::UIElement(el) = &stmt.kind {
                self.order_element(el);
            }
            for body in child_bodies(stmt) {
                self.order_statements(body);
            }
        }
    }

    /// Wrap a Button's, IconButton's or component's action shorthand in an
    /// `on:click` handler, and move a block's clauses ahead of its children
    /// in the order the new grammar requires.
    fn order_element(&mut self, el: &UIElement) {
        let Some(body_span) = el.body_span else {
            return;
        };
        // A Button's block may mix its actions with content; a component's
        // block is a handler only when it holds nothing else (it fills the
        // default slot otherwise) — the rule `sema::lower` applies.
        let mixed = match &el.component {
            ComponentRef::BuiltIn(n) => n == "Button" || n == "IconButton",
            ComponentRef::UserDefined(_) => false,
            ComponentRef::SubComponent(..) => return,
        };
        let shorthand = el.events.is_empty()
            && el.children.iter().any(is_action_statement)
            && (mixed || el.children.iter().all(is_action_statement));

        // The clauses in the order the new grammar wants, each as the text
        // it was written as.
        let mut clauses: Vec<(u8, Span)> = Vec::new();
        if let Some(span) = el.style_span {
            clauses.push((0, span));
        }
        if let Some(t) = &el.transition_block {
            clauses.push((1, t.span));
        }
        for h in &el.events {
            clauses.push((2, h.span));
        }
        let first_child = el
            .children
            .iter()
            .filter(|s| !(shorthand && is_action_statement(s)))
            .map(|s| s.span.start)
            .min();
        let mut in_order = clauses.windows(2).all(|w| w[0].1.start < w[1].1.start);
        if let Some(first) = first_child {
            in_order &= clauses.iter().all(|(_, s)| s.start < first);
        }
        if in_order && !shorthand {
            return;
        }

        // A block written on one line stays on one line.
        if !self.slice(body_span).contains('\n') {
            let mut moved = String::new();
            for (_, span) in &clauses {
                moved.push(' ');
                moved.push_str(self.slice(*span));
                self.remove_line_of(*span);
            }
            if shorthand {
                let mut body = String::new();
                for stmt in el.children.iter().filter(|s| is_action_statement(s)) {
                    if !body.is_empty() {
                        body.push_str("  ");
                    }
                    body.push_str(self.slice(stmt.span));
                    self.remove_line_of(stmt.span);
                }
                moved.push_str(&format!(" on:click {{ {body} }}"));
            }
            self.patch(body_span.start as usize, body_span.start as usize, moved);
            return;
        }

        let indent = format!("{}    ", self.indent_at(el.span.start as usize));
        let mut moved = String::new();
        for (_, span) in &clauses {
            moved.push('\n');
            moved.push_str(&indent);
            moved.push_str(self.slice(*span));
            self.remove_line_of(*span);
        }
        if shorthand {
            let mut body = String::new();
            for stmt in el.children.iter().filter(|s| is_action_statement(s)) {
                body.push('\n');
                body.push_str(&indent);
                body.push_str("    ");
                // The handler sits one level deeper than the statements did.
                body.push_str(&self.slice(stmt.span).replace('\n', "\n    "));
                self.remove_line_of(stmt.span);
            }
            moved.push('\n');
            moved.push_str(&indent);
            moved.push_str(&format!("on:click {{{body}\n{indent}}}"));
        }
        self.patch(body_span.start as usize, body_span.start as usize, moved);
    }

    /// Remove `span` together with the line's indentation before it and
    /// the newline after it, when it holds the line alone.
    fn remove_line_of(&mut self, span: Span) {
        let bytes = self.source.as_bytes();
        let mut start = span.start as usize;
        while start > 0 && matches!(bytes[start - 1], b' ' | b'\t') {
            start -= 1;
        }
        let mut end = span.end as usize;
        while end < bytes.len() && matches!(bytes[end], b' ' | b'\t') {
            end += 1;
        }
        let alone = (start == 0 || bytes[start - 1] == b'\n')
            && (end == bytes.len() || bytes[end] == b'\n');
        if alone {
            if end < bytes.len() {
                end += 1;
            } else {
                start = start.saturating_sub(1);
            }
            self.patch(start, end, "");
        } else {
            // Shares its line: take the text and one separating space.
            let mut e = span.end as usize;
            while e < bytes.len() && bytes[e] == b' ' {
                e += 1;
            }
            self.patch(span.start as usize, e, "");
        }
    }

    // ─── Round 2: spelling ───────────────────────────────

    fn spell_program(&mut self, program: &Program) {
        for decl in &program.declarations {
            match decl {
                Declaration::Page(p) => self.spell_page(p),
                Declaration::Component(c) => self.spell_component(c),
                Declaration::Store(s) => {
                    self.replace_keyword(s.header_span, "Store", "store");
                    self.spell_statements(&s.body, false);
                }
                Declaration::App(a) => {
                    // `App {` — the keyword is the token before the body.
                    if let Some(first) = a.body.first() {
                        let open = self.source[..first.span.start as usize]
                            .rfind('{')
                            .unwrap_or(0);
                        if let Some(at) = self.source[..open].rfind("App") {
                            self.patch(at, at + 3, "app");
                        }
                    } else if let Some(at) = self.source.find("App") {
                        self.patch(at, at + 3, "app");
                    }
                    self.spell_app(&a.body);
                }
                Declaration::Theme(t) => self.spell_theme(t),
                Declaration::Type(_)
                | Declaration::Enum(_)
                | Declaration::Api(_)
                | Declaration::External(_)
                | Declaration::Const(_)
                | Declaration::Animation(_)
                | Declaration::Test(_)
                | Declaration::Data(_) => {}
            }
        }
    }

    /// The declaration keyword at the start of `header`.
    fn replace_keyword(&mut self, header: Span, from: &str, to: &str) {
        let start = header.start as usize;
        if self.source[start..].starts_with(from) {
            self.patch(start, start + from.len(), to);
        }
    }

    fn spell_page(&mut self, page: &PageDecl) {
        self.replace_keyword(page.header_span, "Page", "page");
        let header = self.slice(page.header_span);
        // `Page Name (` → `page Name(`: drop the space before the parenthesis.
        if let Some(paren) = header.find('(') {
            let mut ws = paren;
            while ws > 0 && header.as_bytes()[ws - 1] == b' ' {
                ws -= 1;
            }
            if ws < paren {
                let base = page.header_span.start as usize;
                self.patch(base + ws, base + paren, "");
            }
        }
        // A bare `noindex` takes its value: `noindex: true`.
        if page.noindex {
            let base = page.header_span.start as usize;
            let mut from = 0;
            while let Some(at) = header[from..].find("noindex") {
                let start = from + at;
                let stop = start + "noindex".len();
                let rest = header[stop..].trim_start();
                if !rest.starts_with(':')
                    && !header[..start]
                        .chars()
                        .next_back()
                        .is_some_and(|c| c.is_alphanumeric() || c == '_')
                {
                    self.patch(base + stop, base + stop, ": true");
                    break;
                }
                from = stop;
            }
        }
        // The path a Route wired, when the page declares none or another.
        if let Some(routed) = self.ctx.route_paths.get(&page.name) {
            if page.path.is_empty() {
                let base = page.header_span.start as usize;
                if let Some(paren) = header.find('(') {
                    let after = base + paren + 1;
                    let sep = if header[paren + 1..].trim_start().starts_with(')') {
                        ""
                    } else {
                        ", "
                    };
                    self.patch(after, after, format!("path: \"{routed}\"{sep}"));
                }
            } else if *routed != page.path {
                self.note(
                    page.header_span,
                    format!(
                        "the app routed `{}` at `{routed}` but the page declares `{}`; the page's path is what routes now",
                        page.name, page.path
                    ),
                );
            }
        }
        self.spell_statements(&page.body, false);
    }

    fn spell_component(&mut self, comp: &ComponentDecl) {
        self.replace_keyword(comp.header_span, "Component", "component");
        let header = self.slice(comp.header_span);
        let base = comp.header_span.start as usize;
        if let Some(paren) = header.find('(') {
            let close = header.rfind(')').unwrap_or(header.len());
            let inside = header[paren + 1..close].trim();
            if inside.is_empty() {
                // `Component X ()` → `component X`.
                let mut ws = paren;
                while ws > 0 && header.as_bytes()[ws - 1] == b' ' {
                    ws -= 1;
                }
                self.patch(base + ws, base + close + 1, "");
            } else {
                let mut ws = paren;
                while ws > 0 && header.as_bytes()[ws - 1] == b' ' {
                    ws -= 1;
                }
                if ws < paren {
                    self.patch(base + ws, base + paren, "");
                }
            }
        }
        for (i, prop) in comp.props.iter().enumerate() {
            self.spell_prop(
                prop,
                i == 0 && self.ctx.positional_components.contains(&comp.name),
            );
        }
        self.spell_statements(&comp.body, false);
    }

    /// `List` → `[Any]` in an action's parameter list, which runs from the
    /// first parenthesis to the body.
    fn spell_param_types(&mut self, action: Span) {
        let text = self.slice(action);
        let base = action.start as usize;
        let Some(open) = text.find('(') else {
            return;
        };
        let end = text.find('{').unwrap_or(text.len());
        if end < open {
            return;
        }
        let params = &text[open..end];
        let mut from = 0;
        while let Some(at) = params[from..].find("List") {
            let start = from + at;
            let stop = start + 4;
            let before = params[..start].chars().next_back();
            let after = params[stop..].chars().next();
            let word = |c: Option<char>| c.is_some_and(|c| c.is_alphanumeric() || c == '_');
            if !word(before) && !word(after) {
                self.patch(base + open + start, base + open + stop, "[Any]");
            }
            from = stop;
        }
    }

    /// `a?: String` → `a: String?`, `List` → `[Any]`, and `_ ` before the
    /// prop a caller passes positionally.
    fn spell_prop(&mut self, prop: &PropDecl, positional: bool) {
        let text = self.slice(prop.span);
        let base = prop.span.start as usize;
        if positional {
            self.patch(base, base, "_ ");
        }
        let Some(colon) = text.find(':') else {
            return;
        };
        let optional_mark = text[..colon].trim_end().ends_with('?');
        if optional_mark {
            let q = text[..colon].rfind('?').unwrap();
            self.patch(base + q, base + q + 1, "");
        }
        let after = &text[colon + 1..];
        let type_start = colon + 1 + (after.len() - after.trim_start().len());
        let type_end = type_start
            + after
                .trim_start()
                .find(|c: char| !(c.is_alphanumeric() || c == '_'))
                .unwrap_or(after.trim_start().len());
        let type_text = &text[type_start..type_end];
        let spelled = match type_text {
            "List" => "[Any]".to_string(),
            other => other.to_string(),
        };
        let spelled = if optional_mark {
            format!("{spelled}?")
        } else {
            spelled
        };
        if spelled != type_text {
            self.patch(base + type_start, base + type_end, spelled);
        }
    }

    fn spell_theme(&mut self, theme: &ThemeDecl) {
        self.replace_keyword(theme.span, "Theme", "theme");
        for token in &theme.tokens {
            let text = self.slice(token.span);
            let base = token.span.start as usize;
            if let Some(rest) = text.strip_prefix("token") {
                let ws = rest.len() - rest.trim_start().len();
                self.patch(base, base + 5 + ws, "");
            }
            if let Expr::StringLiteral(value) = &token.value
                && let Some(q) = text.find('"')
            {
                let unquoted = self.tokens_spelled(&unquote_css(value));
                self.patch(base + q, token.span.end as usize, unquoted);
            }
        }
    }

    fn spell_app(&mut self, body: &[Statement]) {
        self.spell_statements(body, false);
    }

    /// Statements of a render block, or of an imperative body when
    /// `imperative` — where `state` becomes `let`.
    fn spell_statements(&mut self, stmts: &[Statement], imperative: bool) {
        for stmt in stmts {
            match &stmt.kind {
                StatementKind::UIElement(el) => self.spell_element(el),
                // `state when = ""` bound to a date picker is a `Date`:
                // the type it always held, now written down.
                StatementKind::State(st)
                    if !imperative
                        && st.ty.is_none()
                        && self.ctx.bound_dates.contains(&st.name) =>
                {
                    if let Expr::StringLiteral(text) = &st.value {
                        let (start, end) = (stmt.span.start as usize, stmt.span.end as usize);
                        let line = &self.source[start..end];
                        if let Some(eq) = line.find('=') {
                            let (ty, value) = if text.is_empty() {
                                // An empty picker holds nothing.
                                ("Date?", "null".to_string())
                            } else {
                                ("Date", format!("@{text}"))
                            };
                            let before = line[..eq].trim_end().len();
                            self.patch(start + eq + 1, end, format!(" {value}"));
                            self.patch(start + before, start + before, format!(": {ty}"));
                            self.note(
                                stmt.span,
                                format!(
                                    "`{}` is bound to a DatePicker, which holds a `{ty}`",
                                    st.name
                                ),
                            );
                        }
                    }
                }
                StatementKind::State(_) if imperative => {
                    let start = stmt.span.start as usize;
                    if self.source[start..].starts_with("state") {
                        self.patch(start, start + 5, "let");
                    }
                }
                StatementKind::If(i) => {
                    if let Some(span) = i.animate_span {
                        self.lift_animate(span, i.animate.as_ref(), &i.then_body);
                        for (_, b) in &i.else_if_branches {
                            self.lift_into(i.animate.as_ref(), b);
                        }
                        if let Some(b) = &i.else_body {
                            self.lift_into(i.animate.as_ref(), b);
                        }
                    }
                    self.spell_statements(&i.then_body, imperative);
                    for (_, b) in &i.else_if_branches {
                        self.spell_statements(b, imperative);
                    }
                    if let Some(b) = &i.else_body {
                        self.spell_statements(b, imperative);
                    }
                }
                StatementKind::For(f) => {
                    if let Some(span) = f.animate_span {
                        self.lift_animate(span, f.animate.as_ref(), &f.body);
                    }
                    self.spell_statements(&f.body, imperative);
                }
                StatementKind::Show(s) => {
                    if let Some(span) = s.animate_span {
                        self.lift_animate(span, s.animate.as_ref(), &s.body);
                    }
                    self.spell_statements(&s.body, imperative);
                }
                StatementKind::Fetch(f) => self.spell_fetch(stmt.span, f, imperative),
                StatementKind::Action(a) => {
                    self.spell_param_types(stmt.span);
                    self.spell_statements(&a.body, true);
                }
                StatementKind::Effect(e) => self.spell_statements(&e.body, true),
                StatementKind::EventHandler(h) => {
                    self.spell_handler(h);
                }
                _ => {}
            }
        }
    }

    /// An `if …, animate(fadeIn, fast, stagger: "50ms")` clause becomes
    /// motion props on the root elements of the branch.
    fn lift_animate(&mut self, clause: Span, config: Option<&AnimateConfig>, body: &[Statement]) {
        self.replace(clause, "");
        self.lift_into(config, body);
    }

    fn lift_into(&mut self, config: Option<&AnimateConfig>, body: &[Statement]) {
        let Some(config) = config else {
            return;
        };
        let mut flags = vec![format!(".{}", config.enter)];
        let mut args = Vec::new();
        if let Some(exit) = &config.exit {
            args.push(format!("exit: .{exit}"));
        }
        match config.duration.as_deref() {
            Some("150ms") => flags.push(".fast".to_string()),
            Some("500ms") => flags.push(".slow".to_string()),
            Some(d) => args.push(format!("duration: \"{d}\"")),
            None => {}
        }
        if let Some(d) = &config.delay {
            args.push(format!("delay: \"{d}\""));
        }
        if let Some(s) = &config.stagger {
            args.push(format!("stagger: \"{s}\""));
        }
        if let Some(e) = &config.easing {
            args.push(format!("easing: \"{e}\""));
        }
        flags.dedup();
        for stmt in body {
            if let StatementKind::UIElement(el) = &stmt.kind {
                let entry = self.lifted.entry(el.span.start).or_default();
                entry.0.extend(flags.iter().cloned());
                entry.1.extend(args.iter().cloned());
            }
        }
    }

    /// `fetch x from url (opts) { loading { } error(e) { } success { } }`
    /// → `resource x = fetch(url, opts)` and `match x { … ready(x) { } }`.
    fn spell_fetch(&mut self, span: Span, f: &FetchDecl, imperative: bool) {
        if imperative {
            self.note(
                span,
                format!(
                    "a `fetch` inside an action or handler has no mechanical rewrite; write `let {} = await fetch(…)` and move the blocks' statements after it",
                    f.variable
                ),
            );
            return;
        }
        let url = self.slice(f.url_span);
        let opts = match f.options_span {
            Some(span) => {
                let inner = self.slice(span);
                let inner = inner.trim_start_matches('(').trim_end_matches(')').trim();
                if inner.is_empty() {
                    String::new()
                } else {
                    format!(", {inner}")
                }
            }
            None => String::new(),
        };
        // The head runs from the statement's start to the `{` of the block.
        let head_end = f.options_span.map(|s| s.end).unwrap_or(f.url_span.end) as usize;
        let brace = self.source[head_end..]
            .find('{')
            .map(|i| head_end + i)
            .unwrap_or(head_end);
        let indent = self.indent_at(span.start as usize);
        self.patch(
            span.start as usize,
            brace,
            format!(
                "resource {name} = fetch({url}{opts})\n{indent}match {name} ",
                name = f.variable
            ),
        );
        if let Some(success) = f.success_span {
            let text = self.slice(success);
            if let Some(brace) = text.find('{') {
                let base = success.start as usize;
                self.patch(base, base + brace, format!("ready({}) ", f.variable));
            }
            if let Some(body) = &f.success_block {
                self.spell_statements(body, false);
            }
        }
        if let Some(body) = &f.loading_block {
            self.spell_statements(body, false);
        }
        if let Some((_, body)) = &f.error_block {
            self.spell_statements(body, false);
        }
    }

    /// `on:click {` → `on click {`, naming the parameter when the body reads
    /// the implicit `event`.
    fn spell_handler(&mut self, h: &EventHandler) {
        let text = self.slice(h.span);
        let base = h.span.start as usize;
        if let Some(head) = text.strip_prefix("on:") {
            let name_end = head
                .find(|c: char| !(c.is_alphanumeric() || c == '_'))
                .unwrap_or(head.len());
            let param = if body_reads(&h.body, &["event"]) {
                "(event)"
            } else {
                ""
            };
            self.patch(
                base,
                base + 3 + name_end,
                format!("on {}{param}", &head[..name_end]),
            );
        }
        self.spell_statements(&h.body, true);
    }

    fn spell_element(&mut self, el: &UIElement) {
        // Pages own their routes now: a `Router` whose block holds nothing
        // but `Route`s is written bare; one with other children loses them.
        if matches!(&el.component, ComponentRef::BuiltIn(n) if n == "Router") {
            let is_route = |s: &Statement| {
                matches!(&s.kind, StatementKind::UIElement(r)
                    if matches!(&r.component, ComponentRef::BuiltIn(n) if n == "Route"))
            };
            if !el.children.is_empty() && el.children.iter().all(is_route) {
                self.replace(el.span, "Router");
                return;
            }
            for child in &el.children {
                if is_route(child) {
                    self.remove_line_of(child.span);
                } else {
                    self.spell_statements(std::slice::from_ref(child), false);
                }
            }
            return;
        }
        let sig = match &el.component {
            ComponentRef::BuiltIn(name) => registry::component(name),
            ComponentRef::SubComponent(owner, part) => registry::part(owner, part),
            ComponentRef::UserDefined(_) => None,
        };
        let name_len = match &el.component {
            ComponentRef::BuiltIn(n) | ComponentRef::UserDefined(n) => n.len(),
            ComponentRef::SubComponent(o, p) => o.len() + 1 + p.len(),
        };
        let name_start = el.span.start as usize;

        // Parts spelled under their owner.
        if let ComponentRef::BuiltIn(n) = &el.component {
            let renamed = match n.as_str() {
                "Thead" => Some("Table.Head"),
                "Tbody" => Some("Table.Body"),
                "Trow" => Some("Table.Row"),
                "Tcell" => Some("Table.Cell"),
                "Option" => Some("Select.Option"),
                "TabPage" => Some("Tabs.Page"),
                _ => None,
            };
            if let Some(to) = renamed {
                self.patch(name_start, name_start + name_len, to);
            }
        }

        let (mut flags, mut new_args) = self.lifted.remove(&el.span.start).unwrap_or_default();
        // A layout used to reflow on a narrow screen whatever the author
        // wrote — every `Row` a column, every `Grid` one column, with
        // `!important`. It does not any more, so a project that relied on
        // it says so: `.stacks` keeps exactly the old behaviour.
        if let ComponentRef::BuiltIn(name) = &el.component
            && matches!(name.as_str(), "Row" | "Grid" | "Column")
            && !el.modifiers.iter().any(|m| m == "stacks")
        {
            flags.push(".stacks".to_string());
        }
        let mut removals: Vec<(usize, usize)> = Vec::new();

        // Modifiers → flags.
        for (i, word) in el.modifiers.iter().enumerate() {
            let span = el.modifier_spans[i];
            let spelling = match sig {
                Some(sig) => sig.spelling_of_legacy(word),
                None => Some(format!(".{word}")),
            };
            match spelling {
                Some(s) if s.starts_with('.') => flags.push(s),
                Some(s) => new_args.push(s),
                None => {
                    if !registry::RETIRED_MODIFIERS.contains(&word.as_str()) {
                        self.note(
                            span,
                            format!(
                                "`{word}` has no spelling on {}; it was dropped",
                                sig.map(|s| s.qualified())
                                    .unwrap_or_else(|| "this element".into())
                            ),
                        );
                    }
                }
            }
            removals.push(modifier_removal_range(self.source, span));
        }

        // Named arguments: enum cases written bare, and `active: "prefix"`.
        for (i, arg) in el.args.iter().enumerate() {
            let Arg::Named(key, value) = arg else {
                continue;
            };
            let span = el.arg_spans[i];
            let Some(sig) = sig else {
                continue;
            };
            let Some(prop) = sig.prop(key) else {
                continue;
            };
            let registry::PropType::Enum(cases) = prop.ty else {
                continue;
            };
            let case = match value {
                Expr::Identifier(w) => Some(w.as_str()),
                Expr::StringLiteral(s) if key == "active" || key == "loading" => Some(s.as_str()),
                _ => None,
            };
            let Some(case) = case else {
                continue;
            };
            if !cases.iter().any(|c| c.name == case) {
                continue;
            }
            if key == "active" {
                flags.push(".prefix".to_string());
                removals.push(modifier_removal_range(self.source, span));
                continue;
            }
            let text = self.slice(span);
            if let Some(colon) = text.find(':') {
                let rest = &text[colon + 1..];
                let ws = rest.len() - rest.trim_start().len();
                let value_start = span.start as usize + colon + 1 + ws;
                self.patch(value_start, span.end as usize, format!(".{case}"));
            }
        }

        // `Option("value", "Label")` → `Select.Option("Label", value: "value")`.
        if matches!(&el.component, ComponentRef::BuiltIn(n) if n == "Option")
            && el.args.len() == 2
            && let (Arg::Positional(_), Arg::Positional(_)) = (&el.args[0], &el.args[1])
        {
            let value = self.slice(el.arg_spans[0]).to_string();
            let label = self.slice(el.arg_spans[1]).to_string();
            let start = el.arg_spans[0].start as usize;
            let end = el.arg_spans[1].end as usize;
            self.patch(start, end, format!("{label}, value: {value}"));
        }

        // `TitleSlide("Title", "Subtitle")` → `TitleSlide("Title", subtitle: "Subtitle")`.
        if matches!(&el.component, ComponentRef::BuiltIn(n) if n == "TitleSlide")
            && el.args.len() >= 2
            && let Arg::Positional(_) = &el.args[1]
        {
            let start = el.arg_spans[1].start as usize;
            self.patch(start, start, "subtitle: ");
        }

        // Apply the removals, merged, and rewrite the parenthesis group.
        removals.sort();
        let mut merged: Vec<(usize, usize)> = Vec::new();
        for (s, e) in removals {
            match merged.last_mut() {
                Some(last) if s <= last.1 => last.1 = last.1.max(e),
                _ => merged.push((s, e)),
            }
        }
        // A run of removed items at the end of the list takes the comma
        // that led into it, or `("Save", )` would remain.
        let bytes = self.source.as_bytes();
        for range in &mut merged {
            let mut after = range.1;
            while after < bytes.len() && bytes[after] == b' ' {
                after += 1;
            }
            if after < bytes.len() && bytes[after] == b')' {
                let mut before = range.0;
                while before > 0 && bytes[before - 1] == b' ' {
                    before -= 1;
                }
                if before > 0 && bytes[before - 1] == b',' {
                    range.0 = before - 1;
                }
            }
        }
        let remaining_args = el.args.len()
            - el.args
                .iter()
                .enumerate()
                .filter(|(i, a)| {
                    matches!(a, Arg::Named(k, _) if k == "active")
                        && merged
                            .iter()
                            .any(|(s, _)| *s <= el.arg_spans[*i].start as usize)
                })
                .count();
        let empty_after = remaining_args == 0 && new_args.is_empty();
        match el.paren_span {
            Some(paren) if empty_after => {
                // Everything inside is going: drop the parentheses too.
                let mut text = String::new();
                for f in &flags {
                    text.push_str(f);
                }
                self.patch(paren.start as usize, paren.end as usize, text);
            }
            Some(paren) => {
                for &(s, e) in &merged {
                    self.patch(s, e, "");
                }
                let close = paren.end as usize - 1;
                if !new_args.is_empty() {
                    // The end of the last argument that stays, when there is one.
                    let last_end = el
                        .arg_spans
                        .iter()
                        .map(|s| (s.start as usize, s.end as usize))
                        .filter(|(start, end)| !merged.iter().any(|(s, m)| s <= start && end <= m))
                        .map(|(_, end)| end)
                        .max();
                    match last_end {
                        // `)` on its own line, as a call laid out one
                        // argument per line has it: the new arguments take
                        // a line of their own, at the arguments' indent.
                        Some(end) if self.source[end..close].contains('\n') => {
                            let line_start = self.source[..end].rfind('\n').map_or(0, |i| i + 1);
                            let indent: String = self.source[line_start..]
                                .chars()
                                .take_while(|c| *c == ' ' || *c == '\t')
                                .collect();
                            self.patch(end, end, format!(",\n{indent}{}", new_args.join(", ")));
                        }
                        Some(_) => self.patch(close, close, format!(", {}", new_args.join(", "))),
                        None => self.patch(close, close, new_args.join(", ")),
                    }
                }
                let mut after = String::new();
                for f in &flags {
                    after.push_str(f);
                }
                self.patch(paren.end as usize, paren.end as usize, after);
            }
            None => {
                let at = name_start + name_len;
                let mut text = String::new();
                if !new_args.is_empty() {
                    text.push_str(&format!("({})", new_args.join(", ")));
                }
                for f in &flags {
                    text.push_str(f);
                }
                self.patch(at, at, text);
            }
        }

        // Clauses and children.
        if let Some(style) = &el.style_block {
            self.spell_style(style);
        }
        if let Some(t) = &el.transition_block {
            self.spell_transition(t);
        }
        for h in &el.events {
            self.spell_handler(h);
        }
        self.spell_statements(&el.children, false);
    }

    /// `transition { opacity 200ms ease }` → `transition { opacity: 200ms ease }`.
    fn spell_transition(&mut self, t: &TransitionBlock) {
        let text = self.slice(t.span);
        let base = t.span.start as usize;
        let Some(open) = text.find('{') else {
            return;
        };
        let close = text.rfind('}').unwrap_or(text.len());
        let inner = &text[open + 1..close];
        let one_line = !inner.contains('\n');
        let mut out = String::new();
        for line in inner.split('\n') {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                out.push_str(line);
                out.push('\n');
                continue;
            }
            let indent = &line[..line.len() - line.trim_start().len()];
            let mut words = trimmed.split_whitespace();
            let prop = words.next().unwrap_or("");
            // A quoted duration or easing loses its quotes; `var(--x)` is `$x`.
            let rest: Vec<String> = words
                .map(|w| self.tokens_spelled(w.trim_matches('"')))
                .collect();
            out.push_str(&format!("{indent}{prop}: {}\n", rest.join(" ")));
        }
        let out = if one_line {
            format!(" {} ", out.trim())
        } else {
            out.trim_end_matches('\n').to_string()
        };
        self.patch(base + open + 1, base + close, out);
    }

    fn spell_style(&mut self, style: &StyleBlock) {
        for prop in &style.properties {
            self.spell_style_value(prop);
        }
        for pseudo in &style.pseudo_blocks {
            let text = self.slice(pseudo.span);
            let base = pseudo.span.start as usize;
            let selector = match pseudo.state.as_str() {
                "hover" => ":hover",
                "focus" => ":focus-visible",
                "active" => ":active",
                "disabled" => ":disabled",
                "placeholder" => "::placeholder",
                "focus-within" => ":focus-within",
                "current" => "[aria-current=\"page\"]",
                "pressed" => "[aria-pressed=\"true\"]",
                "selected" => "[aria-selected=\"true\"]",
                "checked" => "[aria-checked=\"true\"]",
                "expanded" => "[aria-expanded=\"true\"]",
                "invalid" => "[aria-invalid=\"true\"]",
                _ => "",
            };
            let name_len = text
                .find(|c: char| c == '{' || c.is_whitespace())
                .unwrap_or(text.len());
            self.patch(base, base + name_len, format!("&{selector}"));
            for p in &pseudo.properties {
                self.spell_style_value(p);
            }
        }
        for mq in &style.media_queries {
            for p in &mq.properties {
                self.spell_style_value(p);
            }
        }
    }

    /// `var(--name)` → `$name` for every design token in the value.
    fn tokens_spelled(&self, value: &str) -> String {
        let mut out = String::with_capacity(value.len());
        let mut rest = value;
        while let Some(at) = rest.find("var(--") {
            out.push_str(&rest[..at]);
            let after = &rest[at + 6..];
            let name_len = after
                .find(|c: char| !(c.is_alphanumeric() || c == '-' || c == '_'))
                .unwrap_or(after.len());
            let name = &after[..name_len];
            if after[name_len..].starts_with(')') && self.ctx.tokens.contains(name) {
                out.push('$');
                out.push_str(name);
                rest = &after[name_len + 1..];
            } else {
                out.push_str("var(--");
                rest = after;
            }
        }
        out.push_str(rest);
        out
    }

    /// End a value kept as written with `terminator`, when it needs one.
    fn terminate(&mut self, value: Span, terminator: &str) {
        if !terminator.is_empty() {
            let end = value.end as usize;
            self.patch(end, end, terminator);
        }
    }

    /// A style value in the new grammar: raw CSS, a `$token`, or a
    /// `{state}` splice.
    fn spell_style_value(&mut self, prop: &StyleProperty) {
        // A raw value runs to the end of its line; one that shares the line
        // with what follows ends at a `;`.
        let rest = &self.source[prop.value_span.end as usize..];
        let gap = rest.len() - rest.trim_start_matches([' ', '\t']).len();
        let next = rest[gap..].chars().next();
        let terminator = if matches!(next, None | Some('\n') | Some('\r') | Some('}') | Some(';')) {
            ""
        } else {
            // `a: 1;  b: 2` — one space after the semicolon reads best.
            let end = prop.value_span.end as usize;
            self.patch(end, end + gap, " ");
            ";"
        };
        let css_prop = crate::codegen::style_tokens::canonical_style_prop(&prop.name);
        let spelled = match &prop.value {
            Expr::StringLiteral(s) => {
                if prop.name == "content" || prop.name == "quotes" || s.contains('"') {
                    self.terminate(prop.value_span, terminator);
                    return;
                }
                self.tokens_spelled(&unquote_css(s))
            }
            Expr::InterpolatedString(_) => {
                // `"{x}%"` → `{x}%`: the same text without the quotes.
                let raw = self.slice(prop.value_span);
                let inner = raw.trim().trim_matches('"');
                self.tokens_spelled(&inner.replace('\u{FFFE}', "{").replace('\u{FFFF}', "}"))
            }
            Expr::Identifier(name) => {
                match crate::codegen::style_tokens::resolve_style_token(&css_prop, &prop.value) {
                    // `padding: xl` → `padding: $xl`: the short name still
                    // resolves through the property's group — unless the
                    // theme declares the short name itself, when only the
                    // full one means what it did.
                    Some(_) if !self.ctx.tokens.contains(name) => format!("${name}"),
                    Some(var) => {
                        let inner = var.trim_start_matches("var(--").trim_end_matches(')');
                        format!("${inner}")
                    }
                    None => format!("{{{name}}}"),
                }
            }
            Expr::NumberLiteral(_) => {
                self.terminate(prop.value_span, terminator);
                return;
            }
            _ => {
                let raw = self.slice(prop.value_span);
                format!("{{{}}}", raw.trim())
            }
        };
        self.replace(prop.value_span, format!("{spelled}{terminator}"));
    }
}

/// A CSS value out of a string literal: the placeholders the lexer put in
/// for escaped braces become braces again.
fn unquote_css(s: &str) -> String {
    s.replace('\u{FFFE}', "{").replace('\u{FFFF}', "}")
}

/// Whether the statements read any of `names` as a bare identifier.
fn body_reads(stmts: &[Statement], names: &[&str]) -> bool {
    fn expr_reads(e: &Expr, names: &[&str]) -> bool {
        match e {
            Expr::Identifier(n) => names.contains(&n.as_str()),
            _ => e.children().iter().any(|c| expr_reads(c, names)),
        }
    }
    stmts.iter().any(|s| {
        s.kind.exprs().iter().any(|e| expr_reads(e, names))
            || child_bodies(s).iter().any(|b| body_reads(b, names))
    })
}

fn is_action_statement(stmt: &Statement) -> bool {
    match &stmt.kind {
        StatementKind::Assignment(_)
        | StatementKind::MethodCall(_)
        | StatementKind::Navigate(_)
        | StatementKind::Log(_)
        | StatementKind::Emit(_)
        | StatementKind::ExprStatement(_) => true,
        // A branch of nothing but actions is a guard on them, not a
        // conditional render.
        StatementKind::If(i) => {
            let all = |b: &[Statement]| !b.is_empty() && b.iter().all(is_action_statement);
            all(&i.then_body)
                && i.else_if_branches.iter().all(|(_, b)| all(b))
                && i.else_body.as_deref().is_none_or(all)
        }
        _ => false,
    }
}

/// Every statement list nested in a statement.
fn child_bodies(stmt: &Statement) -> Vec<&[Statement]> {
    match &stmt.kind {
        StatementKind::UIElement(el) => {
            let mut bodies: Vec<&[Statement]> = vec![&el.children];
            bodies.extend(el.events.iter().map(|e| e.body.as_slice()));
            bodies.extend(el.slot_fills.iter().map(|f| f.body.as_slice()));
            bodies
        }
        StatementKind::If(i) => {
            let mut bodies: Vec<&[Statement]> = vec![&i.then_body];
            bodies.extend(i.else_if_branches.iter().map(|(_, b)| b.as_slice()));
            bodies.extend(i.else_body.as_deref());
            bodies
        }
        StatementKind::For(f) => vec![&f.body],
        StatementKind::Show(s) => vec![&s.body],
        StatementKind::Fetch(f) => {
            let mut bodies: Vec<&[Statement]> = Vec::new();
            bodies.extend(f.loading_block.as_deref());
            bodies.extend(f.error_block.as_ref().map(|(_, b)| b.as_slice()));
            bodies.extend(f.success_block.as_deref());
            bodies
        }
        StatementKind::Match(m) => m.arms.iter().map(|a| a.body.as_slice()).collect(),
        StatementKind::Effect(e) => vec![&e.body],
        StatementKind::Action(a) => vec![&a.body],
        StatementKind::EventHandler(h) => vec![&h.body],
        _ => Vec::new(),
    }
}

/// The `Route` elements under the first `Router` in `body`, at any depth.
fn routes_in(body: &[Statement]) -> Vec<&UIElement> {
    for stmt in body {
        if let StatementKind::UIElement(ui) = &stmt.kind {
            if matches!(&ui.component, ComponentRef::BuiltIn(n) if n == "Router") {
                return ui
                    .children
                    .iter()
                    .filter_map(|s| match &s.kind {
                        StatementKind::UIElement(r)
                            if matches!(&r.component, ComponentRef::BuiltIn(n) if n == "Route") =>
                        {
                            Some(r)
                        }
                        _ => None,
                    })
                    .collect();
            }
            let nested = routes_in(&ui.children);
            if !nested.is_empty() {
                return nested;
            }
        }
    }
    Vec::new()
}

fn route_path(route: &UIElement) -> Option<&str> {
    route.args.iter().find_map(|a| match a {
        Arg::Named(k, Expr::StringLiteral(p)) if k == "path" => Some(p.as_str()),
        _ => None,
    })
}

fn route_page(route: &UIElement) -> Option<&str> {
    route.args.iter().find_map(|a| match a {
        Arg::Named(k, Expr::Identifier(p)) if k == "page" => Some(p.as_str()),
        _ => None,
    })
}

/// The `.wf` files under `dir`, `App.wf` first, as `wf build` reads them.
pub fn source_files(dir: &Path) -> std::io::Result<Vec<(PathBuf, String)>> {
    let mut paths = Vec::new();
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
        let mut entries: Vec<PathBuf> = std::fs::read_dir(dir)?
            .flatten()
            .map(|e| e.path())
            .collect();
        entries.sort();
        for path in entries {
            if path.is_dir() {
                walk(&path, out)?;
            } else if crate::syntax::is_source_file(&path) {
                out.push(path);
            }
        }
        Ok(())
    }
    walk(dir, &mut paths)?;
    let mut files = Vec::new();
    for path in paths {
        let text = std::fs::read_to_string(&path)?;
        files.push((path, text));
    }
    Ok(files)
}

/// A one-line report of a file's notes, for the console.
pub fn report(path: &Path, migrated: &Migrated) -> String {
    let mut out = String::new();
    for note in &migrated.notes {
        out.push_str(&format!(
            "  {}:{}:{}: {}\n",
            path.display(),
            note.line,
            note.column,
            note.message
        ));
    }
    out
}

/// Migrations keyed by path, in order.
pub type Outcome = BTreeMap<PathBuf, Result<Migrated>>;
