//! The parser for WebFluent 3 (`spec/SYNTAX_V2.md`).
//!
//! It produces the same [`Program`] the original parser does, so every code
//! generator and lint reads both grammars; what the new grammar can say that
//! the old one could not lands in the fields the AST gained for it (enum
//! cases, tokens, events, slots, resources, `match`, keys, bindings).
//!
//! The grammar is LL(1) over the tokens of [`crate::lexer::LexerV2`]: no
//! word is a keyword to the lexer, so this parser reserves words by
//! position. The rules that matter:
//!
//! - a declaration starts with `page`, `component`, `store`, `theme`,
//!   `app`, `type` or `enum`;
//! - in a render block, a Capitalised word starts an element and a
//!   lowercase word is a statement keyword, a slot use, or an error;
//! - an element is `Path [ ( args ) ] ( .flag )* [ { block } ]`, and its
//!   block holds, in this order, `style`, `transition`, `on` handlers,
//!   slot fills and children;
//! - an imperative block (`action`, `effect`, `on`) holds statements, where a
//!   Capitalised word begins an expression, never an element.
//!
//! A `.word` after an element is a flag: the sema pass resolves it against
//! the component's props. A `.word` in expression position is an enum case.

use crate::error::{Diagnostic, Result, WebFluentError};
use crate::lexer::v2::LexerV2;
use crate::lexer::{Token, TokenType};
use crate::parser::ast::*;

/// Parse a WebFluent 3 source file.
pub fn parse_v2(source: &str, file: &str) -> Result<Program> {
    let tokens = LexerV2::new(source, file).tokenize()?;
    ParserV2::new(tokens, file).parse()
}

/// The clause a render block is at, so the order is enforced.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Stage {
    Style,
    Transition,
    Handlers,
    Fills,
    Children,
}

impl Stage {
    fn name(self) -> &'static str {
        match self {
            Stage::Style => "style",
            Stage::Transition => "transition",
            Stage::Handlers => "on handlers",
            Stage::Fills => "slot fills",
            Stage::Children => "children",
        }
    }
}

/// What kind of body is being parsed, which decides what a word may mean.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Body {
    Page,
    Component,
    Store,
    App,
    /// The children of an element, a branch, a loop, an arm.
    Nested,
}

/// A parsed component body: its statements, the interior span, and the
/// `event` and `slot` declarations lifted out of it.
struct ComponentBody {
    statements: Vec<Statement>,
    span: Span,
    events: Vec<EventDecl>,
    slots: Vec<SlotDecl>,
}

pub struct ParserV2 {
    tokens: Vec<Token>,
    pos: usize,
    file: String,
}

const CLAUSE_WORDS: &[&str] = &["style", "transition", "on"];
const STATEMENT_WORDS: &[&str] = &[
    "state", "derived", "effect", "action", "use", "resource", "event", "slot", "if", "for",
    "show", "match", "children", "let", "return", "navigate", "log", "emit", "else",
];

impl ParserV2 {
    pub fn new(tokens: Vec<Token>, file: &str) -> Self {
        Self {
            tokens,
            pos: 0,
            file: file.to_string(),
        }
    }

    // ─── Token helpers ───────────────────────────────────

    fn current(&self) -> &Token {
        &self.tokens[self.pos.min(self.tokens.len() - 1)]
    }

    fn kind(&self) -> &TokenType {
        &self.current().token_type
    }

    fn kind_at(&self, ahead: usize) -> &TokenType {
        &self.tokens[(self.pos + ahead).min(self.tokens.len() - 1)].token_type
    }

    fn at_end(&self) -> bool {
        matches!(self.kind(), TokenType::EOF)
    }

    fn advance(&mut self) -> Token {
        let token = self.current().clone();
        if !self.at_end() {
            self.pos += 1;
        }
        token
    }

    fn check(&self, kind: &TokenType) -> bool {
        std::mem::discriminant(self.kind()) == std::mem::discriminant(kind)
    }

    fn eat(&mut self, kind: &TokenType) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, kind: &TokenType, what: &str) -> Result<Token> {
        if self.check(kind) {
            Ok(self.advance())
        } else {
            Err(self.error(format!("Expected {what}, got {}", self.describe())))
        }
    }

    /// Whether the current token is the identifier `word`.
    fn is_word(&self, word: &str) -> bool {
        matches!(self.kind(), TokenType::Identifier(w) if w == word)
    }

    fn is_word_at(&self, ahead: usize, word: &str) -> bool {
        matches!(self.kind_at(ahead), TokenType::Identifier(w) if w == word)
    }

    fn eat_word(&mut self, word: &str) -> bool {
        if self.is_word(word) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect_word(&mut self, word: &str) -> Result<()> {
        if self.eat_word(word) {
            Ok(())
        } else {
            Err(self.error(format!("Expected `{word}`, got {}", self.describe())))
        }
    }

    fn ident(&self) -> Option<&str> {
        match self.kind() {
            TokenType::Identifier(w) => Some(w),
            _ => None,
        }
    }

    fn expect_ident(&mut self, what: &str) -> Result<String> {
        match self.kind() {
            TokenType::Identifier(w) => {
                let w = w.clone();
                self.advance();
                Ok(w)
            }
            _ => Err(self.error(format!("Expected {what}, got {}", self.describe()))),
        }
    }

    fn is_capitalized(&self) -> bool {
        self.ident()
            .and_then(|w| w.chars().next())
            .is_some_and(|c| c.is_uppercase())
    }

    /// The current token, for a message.
    fn describe(&self) -> String {
        match self.kind() {
            TokenType::Identifier(w) => format!("`{w}`"),
            TokenType::StringLiteral(s) => format!("the string \"{s}\""),
            TokenType::NumberLiteral(n) => format!("the number {n}"),
            TokenType::EOF => "the end of the file".to_string(),
            TokenType::OpenBrace => "`{`".to_string(),
            TokenType::CloseBrace => "`}`".to_string(),
            TokenType::OpenParen => "`(`".to_string(),
            TokenType::CloseParen => "`)`".to_string(),
            TokenType::Colon => "`:`".to_string(),
            TokenType::Comma => "`,`".to_string(),
            TokenType::Dot => "`.`".to_string(),
            TokenType::Equals => "`=`".to_string(),
            TokenType::DesignToken(t) => format!("`${t}`"),
            TokenType::RawValue(v) => format!("the value `{v}`"),
            TokenType::RawSelector(v) => format!("the selector `{v}`"),
            TokenType::StyleProp(v) => format!("the property `{v}`"),
            other => format!("`{other}`"),
        }
    }

    fn error(&self, message: String) -> WebFluentError {
        let t = self.current();
        WebFluentError::ParseError(Diagnostic::new(message, &self.file, t.line, t.column))
    }

    fn error_with_hint(&self, message: String, hint: &str) -> WebFluentError {
        let t = self.current();
        WebFluentError::ParseError(
            Diagnostic::new(message, &self.file, t.line, t.column).with_hint(hint),
        )
    }

    // ─── Spans ───────────────────────────────────────────

    fn mark(&self) -> (u32, u32, u32) {
        let t = self.current();
        (t.offset as u32, t.line as u32, t.column as u32)
    }

    fn prev_end(&self) -> u32 {
        if self.pos == 0 {
            0
        } else {
            self.tokens[self.pos - 1].end as u32
        }
    }

    fn span_since(&self, mark: (u32, u32, u32)) -> Span {
        Span::new(mark.0, self.prev_end(), mark.1, mark.2)
    }

    /// The `///` lines before the current token, joined.
    fn take_docs(&mut self) -> Option<String> {
        let mut lines = Vec::new();
        while let TokenType::DocComment(text) = self.kind() {
            lines.push(text.clone());
            self.advance();
        }
        (!lines.is_empty()).then(|| lines.join("\n"))
    }

    // ─── Declarations ────────────────────────────────────

    pub fn parse(&mut self) -> Result<Program> {
        let mut declarations = Vec::new();
        while !self.at_end() {
            let doc = self.take_docs();
            if self.at_end() {
                break;
            }
            let word = self.ident().map(str::to_string).unwrap_or_default();
            let decl = match word.as_str() {
                "page" => self.parse_page(doc)?,
                "component" => self.parse_component(doc)?,
                "store" => self.parse_store()?,
                "theme" => self.parse_theme()?,
                "app" => self.parse_app()?,
                "type" => self.parse_type_decl(doc)?,
                "enum" => self.parse_enum_decl(doc)?,
                "Page" | "Component" | "Store" | "App" | "Theme" => {
                    return Err(self.error_with_hint(
                        format!("`{word}` is a WebFluent 2 declaration"),
                        &format!(
                            "WebFluent 3 spells it `{}`; run `wf migrate` to convert the project",
                            word.to_lowercase()
                        ),
                    ));
                }
                _ => {
                    return Err(self.error_with_hint(
                        format!(
                            "Expected a declaration — page, component, store, theme, app, type or enum — got {}",
                            self.describe()
                        ),
                        "Every file is a list of declarations; elements live inside a page or component",
                    ));
                }
            };
            declarations.push(decl);
        }
        Ok(Program { declarations })
    }

    fn parse_page(&mut self, _doc: Option<String>) -> Result<Declaration> {
        let mark = self.mark();
        self.expect_word("page")?;
        let name = self.expect_ident("the page's name")?;
        let mut page = PageDecl {
            name,
            path: String::new(),
            title: None,
            guard: None,
            redirect: None,
            description: None,
            image: None,
            page_type: None,
            noindex: false,
            layout: None,
            params: Vec::new(),
            body: Vec::new(),
            span: Span::dummy(),
            header_span: Span::dummy(),
            body_span: Span::dummy(),
        };
        if self.eat(&TokenType::OpenParen) {
            while !self.check(&TokenType::CloseParen) && !self.at_end() {
                let arg_mark = self.mark();
                let key = self.expect_ident("a page attribute")?;
                self.expect(&TokenType::Colon, "`:` after the attribute name")?;
                match key.as_str() {
                    "path" => page.path = self.expect_string("the page's path")?,
                    "title" => page.title = Some(self.expect_string("the page's title")?),
                    "description" => {
                        page.description = Some(self.expect_string("the description")?)
                    }
                    "image" => page.image = Some(self.expect_string("the image")?),
                    "type" => page.page_type = Some(self.expect_string("the page type")?),
                    "redirect" => page.redirect = Some(self.expect_string("the redirect")?),
                    "noindex" => {
                        page.noindex = match self.kind() {
                            TokenType::BoolLiteral(b) => {
                                let b = *b;
                                self.advance();
                                b
                            }
                            _ => return Err(self.error("`noindex` takes `true` or `false`".into())),
                        }
                    }
                    "guard" => page.guard = Some(self.parse_expression()?),
                    "layout" => {
                        let layout_mark = self.mark();
                        let layout = self.expect_ident("the layout component's name")?;
                        if !layout.chars().next().is_some_and(char::is_uppercase) {
                            return Err(self.error(format!(
                                "`{layout}` is not a component name; a layout is a component"
                            )));
                        }
                        let args = if self.check(&TokenType::OpenParen) {
                            self.parse_call_args()?.0
                        } else {
                            Vec::new()
                        };
                        page.layout = Some(LayoutRef {
                            name: layout,
                            args,
                            span: self.span_since(layout_mark),
                        });
                    }
                    // Anything else is a route parameter: `id: String`.
                    _ => {
                        let ty = self.parse_type_ref()?;
                        page.params.push(PropDecl {
                            name: key,
                            prop_type: ty,
                            optional: false,
                            default: None,
                            positional: false,
                            doc: None,
                            span: self.span_since(arg_mark),
                        });
                    }
                }
                if !self.check(&TokenType::CloseParen) {
                    self.expect(&TokenType::Comma, "`,` between page attributes")?;
                }
            }
            self.expect(&TokenType::CloseParen, "`)`")?;
        }
        page.header_span = self.span_since(mark);
        let (body, body_span) = self.parse_render_block(Body::Page)?;
        page.body = body;
        page.body_span = body_span;
        page.span = self.span_since(mark);
        Ok(Declaration::Page(page))
    }

    fn expect_string(&mut self, what: &str) -> Result<String> {
        match self.kind() {
            TokenType::StringLiteral(s) => {
                let s = s.clone();
                self.advance();
                Ok(s)
            }
            _ => Err(self.error(format!(
                "Expected {what} as a string, got {}",
                self.describe()
            ))),
        }
    }

    fn parse_component(&mut self, doc: Option<String>) -> Result<Declaration> {
        let mark = self.mark();
        self.expect_word("component")?;
        let name = self.expect_ident("the component's name")?;
        let mut props = Vec::new();
        if self.eat(&TokenType::OpenParen) {
            while !self.check(&TokenType::CloseParen) && !self.at_end() {
                props.push(self.parse_prop_decl(props.is_empty())?);
                if !self.check(&TokenType::CloseParen) {
                    self.expect(&TokenType::Comma, "`,` between props")?;
                }
            }
            self.expect(&TokenType::CloseParen, "`)`")?;
        }
        let header_span = self.span_since(mark);
        // Events and slots are declared at the top of the body; the parser
        // collects them wherever they appear so the error is about order
        // only when it matters.
        let ComponentBody {
            statements: body,
            span: body_span,
            events,
            slots,
        } = self.parse_component_body()?;
        Ok(Declaration::Component(ComponentDecl {
            name,
            props,
            events,
            slots,
            doc,
            body,
            span: self.span_since(mark),
            header_span,
            body_span,
        }))
    }

    /// `[_] name: Type [= default]`, with `///` docs before it.
    fn parse_prop_decl(&mut self, first: bool) -> Result<PropDecl> {
        let doc = self.take_docs();
        let mark = self.mark();
        let positional = self.eat_word("_");
        if positional && !first {
            return Err(self.error_with_hint(
                "The positional prop must be declared first".into(),
                "Move the `_ name: Type` prop to the front of the list",
            ));
        }
        let name = self.expect_ident("a prop name")?;
        self.expect(&TokenType::Colon, "`:` after the prop name")?;
        let prop_type = self.parse_type_ref()?;
        let optional = matches!(prop_type, TypeRef::Optional(_));
        let default = if self.eat(&TokenType::Equals) {
            Some(self.parse_expression()?)
        } else {
            None
        };
        Ok(PropDecl {
            name,
            prop_type,
            optional,
            default,
            positional,
            doc,
            span: self.span_since(mark),
        })
    }

    /// `String`, `Number`, `Bool`, `Map`, `Any`, `[T]`, `T?`, or a declared
    /// name.
    fn parse_type_ref(&mut self) -> Result<TypeRef> {
        let base = if self.eat(&TokenType::OpenBracket) {
            let inner = self.parse_type_ref()?;
            self.expect(&TokenType::CloseBracket, "`]` to close the list type")?;
            TypeRef::List(Box::new(inner))
        } else {
            let name = self.expect_ident("a type")?;
            match name.as_str() {
                "String" => TypeRef::String,
                "Number" | "Int" | "Float" => TypeRef::Number,
                "Bool" => TypeRef::Bool,
                "Map" => TypeRef::Map,
                "Any" => TypeRef::Any,
                "List" => {
                    return Err(self.error_with_hint(
                        "`List` is not a type".into(),
                        "Write the element type in brackets: `[Todo]`, or `[Any]`",
                    ));
                }
                _ => TypeRef::Named(name),
            }
        };
        Ok(if self.eat(&TokenType::QuestionMark) {
            TypeRef::Optional(Box::new(base))
        } else {
            base
        })
    }

    fn parse_store(&mut self) -> Result<Declaration> {
        let mark = self.mark();
        self.expect_word("store")?;
        let name = self.expect_ident("the store's name")?;
        let header_span = self.span_since(mark);
        let (body, body_span) = self.parse_render_block(Body::Store)?;
        Ok(Declaration::Store(StoreDecl {
            name,
            body,
            span: self.span_since(mark),
            header_span,
            body_span,
        }))
    }

    fn parse_app(&mut self) -> Result<Declaration> {
        self.expect_word("app")?;
        let (body, _) = self.parse_render_block(Body::App)?;
        Ok(Declaration::App(AppDecl { body }))
    }

    /// `theme Name { token-name: value … }` — the lexer hands the values
    /// over as CSS text.
    fn parse_theme(&mut self) -> Result<Declaration> {
        let mark = self.mark();
        self.expect_word("theme")?;
        let name = self.expect_ident("the theme's name")?;
        self.expect(&TokenType::OpenBrace, "`{`")?;
        let mut tokens = Vec::new();
        while !self.check(&TokenType::CloseBrace) && !self.at_end() {
            let token_mark = self.mark();
            let (prop, value) = self.parse_style_declaration()?;
            tokens.push(ThemeToken {
                name: prop,
                value: self.style_value_expr(&value)?,
                span: self.span_since(token_mark),
            });
        }
        self.expect(&TokenType::CloseBrace, "`}`")?;
        Ok(Declaration::Theme(ThemeDecl {
            name,
            tokens,
            span: self.span_since(mark),
        }))
    }

    fn parse_type_decl(&mut self, doc: Option<String>) -> Result<Declaration> {
        let mark = self.mark();
        self.expect_word("type")?;
        let name = self.expect_ident("the type's name")?;
        let header_span = self.span_since(mark);
        self.expect(&TokenType::OpenBrace, "`{`")?;
        let mut fields = Vec::new();
        while !self.check(&TokenType::CloseBrace) && !self.at_end() {
            let field_doc = self.take_docs();
            let field_mark = self.mark();
            let field_name = self.expect_ident("a field name")?;
            self.expect(&TokenType::Colon, "`:` after the field name")?;
            let ty = self.parse_type_ref()?;
            let default = if self.eat(&TokenType::Equals) {
                Some(self.parse_expression()?)
            } else {
                None
            };
            fields.push(FieldDecl {
                name: field_name,
                ty,
                default,
                doc: field_doc,
                span: self.span_since(field_mark),
            });
            if !self.check(&TokenType::CloseBrace) {
                self.eat(&TokenType::Comma);
            }
        }
        self.expect(&TokenType::CloseBrace, "`}`")?;
        Ok(Declaration::Type(TypeDecl {
            name,
            fields,
            doc,
            span: self.span_since(mark),
            header_span,
        }))
    }

    fn parse_enum_decl(&mut self, doc: Option<String>) -> Result<Declaration> {
        let mark = self.mark();
        self.expect_word("enum")?;
        let name = self.expect_ident("the enum's name")?;
        let header_span = self.span_since(mark);
        self.expect(&TokenType::OpenBrace, "`{`")?;
        let mut cases = Vec::new();
        while !self.check(&TokenType::CloseBrace) && !self.at_end() {
            self.eat(&TokenType::Dot);
            cases.push(self.expect_ident("a case name")?);
            if !self.check(&TokenType::CloseBrace) {
                self.eat(&TokenType::Comma);
            }
        }
        self.expect(&TokenType::CloseBrace, "`}`")?;
        if cases.is_empty() {
            return Err(self.error(format!("`enum {name}` has no cases")));
        }
        Ok(Declaration::Enum(EnumDecl {
            name,
            cases,
            doc,
            span: self.span_since(mark),
            header_span,
        }))
    }

    // ─── Render blocks ───────────────────────────────────

    /// `{ statements }`: the interior span is returned beside the body.
    fn parse_render_block(&mut self, body: Body) -> Result<(Vec<Statement>, Span)> {
        let open = self.expect(&TokenType::OpenBrace, "`{`")?;
        let mut statements = Vec::new();
        while !self.check(&TokenType::CloseBrace) && !self.at_end() {
            statements.push(self.parse_render_statement(body)?);
        }
        let close = self.expect(&TokenType::CloseBrace, "`}`")?;
        let span = Span::new(
            open.end as u32,
            close.offset as u32,
            open.line as u32,
            open.column as u32,
        );
        Ok((statements, span))
    }

    /// A component's body: `event` and `slot` declarations are lifted out.
    fn parse_component_body(&mut self) -> Result<ComponentBody> {
        let open = self.expect(&TokenType::OpenBrace, "`{`")?;
        let mut statements = Vec::new();
        let mut events = Vec::new();
        let mut slots = Vec::new();
        while !self.check(&TokenType::CloseBrace) && !self.at_end() {
            let doc = self.take_docs();
            if self.is_word("event") {
                let mark = self.mark();
                self.advance();
                let name = self.expect_ident("the event's name")?;
                let params = if self.check(&TokenType::OpenParen) {
                    self.parse_params()?
                } else {
                    Vec::new()
                };
                events.push(EventDecl {
                    name,
                    params,
                    doc,
                    span: self.span_since(mark),
                });
                continue;
            }
            if self.is_word("slot") {
                let mark = self.mark();
                self.advance();
                let name = match self.kind() {
                    TokenType::Identifier(w) if !STATEMENT_WORDS.contains(&w.as_str()) => {
                        let w = w.clone();
                        if w.chars().next().is_some_and(char::is_uppercase) {
                            None
                        } else {
                            self.advance();
                            Some(w)
                        }
                    }
                    _ => None,
                };
                slots.push(SlotDecl {
                    name,
                    span: self.span_since(mark),
                });
                continue;
            }
            statements.push(self.parse_render_statement(Body::Component)?);
        }
        let close = self.expect(&TokenType::CloseBrace, "`}`")?;
        let span = Span::new(
            open.end as u32,
            close.offset as u32,
            open.line as u32,
            open.column as u32,
        );
        Ok(ComponentBody {
            statements,
            span,
            events,
            slots,
        })
    }

    fn parse_render_statement(&mut self, body: Body) -> Result<Statement> {
        let mark = self.mark();
        let kind = self.parse_render_statement_kind(body)?;
        Ok(Statement::new(kind, self.span_since(mark)))
    }

    fn parse_render_statement_kind(&mut self, body: Body) -> Result<StatementKind> {
        let Some(word) = self.ident().map(str::to_string) else {
            return Err(self.error_with_hint(
                format!(
                    "Expected an element or a statement, got {}",
                    self.describe()
                ),
                "A render block holds elements (`Card { … }`) and the statements that shape them",
            ));
        };
        if self.is_capitalized() {
            // `Store.load(x)` at the top of a page or component: set-up
            // code, run once when it renders.
            if self.setup_call_ahead(body) {
                return self.parse_imperative_kind();
            }
            return Ok(StatementKind::UIElement(self.parse_element()?));
        }
        if self.setup_call_ahead(body) {
            return self.parse_imperative_kind();
        }
        match word.as_str() {
            "state" => self.parse_state(),
            "derived" => self.parse_derived(),
            "effect" => {
                self.advance();
                let (stmts, _) = self.parse_imperative_block()?;
                Ok(StatementKind::Effect(EffectDecl { body: stmts }))
            }
            "action" => self.parse_action(),
            "use" => {
                self.advance();
                let store_name = self.expect_ident("the store's name")?;
                Ok(StatementKind::Use(UseDecl { store_name }))
            }
            "resource" => self.parse_resource(),
            "if" => self.parse_if(false),
            "for" => self.parse_for(),
            "show" => {
                self.advance();
                let condition = self.parse_expression()?;
                let (stmts, _) = self.parse_render_block(Body::Nested)?;
                Ok(StatementKind::Show(ShowStmt {
                    condition,
                    animate: None,
                    animate_span: None,
                    body: stmts,
                }))
            }
            "match" => self.parse_match_stmt(),
            "children" => {
                self.advance();
                Ok(StatementKind::UIElement(self.slot_use("children")))
            }
            "event" | "slot" => Err(self.error_with_hint(
                format!("`{word}` is declared in a component, before its elements"),
                "Move it to the top of the component's body",
            )),
            "let" | "return" | "navigate" | "log" | "emit" => Err(self.error_with_hint(
                format!("`{word}` is a statement of an action or a handler, not of a page"),
                "Put it inside `on click { … }`, an `action`, or an `effect`",
            )),
            "else" => Err(self.error("`else` without an `if`".into())),
            "on" | "style" | "transition" => Err(self.error_with_hint(
                format!("`{word}` belongs inside an element's block"),
                "Write `Button(\"x\") { on click { … } }`",
            )),
            _ => {
                // A slot use: a bare lowercase word in a component's body.
                let next = self.kind_at(1).clone();
                let bare = !matches!(
                    next,
                    TokenType::OpenParen
                        | TokenType::Dot
                        | TokenType::Equals
                        | TokenType::Colon
                        | TokenType::OpenBrace
                        | TokenType::OpenBracket
                );
                if bare && body == Body::Component || bare && body == Body::Nested {
                    self.advance();
                    return Ok(StatementKind::UIElement(self.slot_use(&word)));
                }
                Err(self.error_with_hint(
                    format!("`{word}` is not an element or a statement a render block can hold"),
                    "Code that does something goes in `on click { … }`, an `action` or an `effect`; an element's name is capitalised",
                ))
            }
        }
    }

    /// Whether a call statement — `load()`, `Store.load(x)` — starts here,
    /// at the top level of a page or component, where it is set-up code.
    fn setup_call_ahead(&self, body: Body) -> bool {
        if !matches!(body, Body::Page | Body::Component) {
            return false;
        }
        const KEYWORDS: &[&str] = &[
            "state",
            "derived",
            "effect",
            "action",
            "use",
            "resource",
            "if",
            "for",
            "show",
            "match",
            "children",
            "event",
            "slot",
            "let",
            "return",
            "emit",
            "else",
            "on",
            "style",
            "transition",
        ];
        let open = match self.kind_at(1) {
            TokenType::OpenParen
                if !self.is_capitalized()
                    && !self.ident().is_some_and(|w| KEYWORDS.contains(&w)) =>
            {
                1
            }
            TokenType::Dot
                if matches!(self.kind_at(2), TokenType::Identifier(m) if m.chars().next().is_some_and(char::is_lowercase))
                    && matches!(self.kind_at(3), TokenType::OpenParen) =>
            {
                3
            }
            _ => return false,
        };
        // A call ends at its parenthesis; an element goes on to a block
        // or a flag (`Row.gap(md) { }` is a mistaken flag, not set-up).
        let mut depth = 0usize;
        let mut i = open;
        loop {
            match self.kind_at(i) {
                TokenType::OpenParen => depth += 1,
                TokenType::CloseParen => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                TokenType::EOF => return false,
                _ => {}
            }
            i += 1;
        }
        !matches!(self.kind_at(i + 1), TokenType::OpenBrace | TokenType::Dot)
    }

    /// The pseudo element that stands for a slot's content.
    fn slot_use(&self, name: &str) -> UIElement {
        let mut el = self.blank_element(ComponentRef::BuiltIn("Children".to_string()));
        if name != "children" {
            el.args.push(Arg::Named(
                "slot".to_string(),
                Expr::StringLiteral(name.to_string()),
            ));
        }
        el
    }

    fn blank_element(&self, component: ComponentRef) -> UIElement {
        UIElement {
            component,
            args: Vec::new(),
            modifiers: Vec::new(),
            children: Vec::new(),
            style_block: None,
            transition_block: None,
            events: Vec::new(),
            slot_fills: Vec::new(),
            span: Span::dummy(),
            paren_span: None,
            body_span: None,
            style_span: None,
            arg_spans: Vec::new(),
            modifier_spans: Vec::new(),
        }
    }

    fn parse_state(&mut self) -> Result<StatementKind> {
        self.expect_word("state")?;
        let name = self.expect_ident("the state's name")?;
        let ty = if self.eat(&TokenType::Colon) {
            Some(self.parse_type_ref()?)
        } else {
            None
        };
        self.expect(&TokenType::Equals, "`=` and an initial value")?;
        let value = self.parse_expression()?;
        Ok(StatementKind::State(StateDecl { name, ty, value }))
    }

    fn parse_derived(&mut self) -> Result<StatementKind> {
        self.expect_word("derived")?;
        let name = self.expect_ident("the derived value's name")?;
        if self.eat(&TokenType::Colon) {
            self.parse_type_ref()?;
        }
        self.expect(&TokenType::Equals, "`=` and the expression")?;
        let value = self.parse_expression()?;
        Ok(StatementKind::Derived(DerivedDecl { name, value }))
    }

    fn parse_action(&mut self) -> Result<StatementKind> {
        self.expect_word("action")?;
        let name = self.expect_ident("the action's name")?;
        let params = if self.check(&TokenType::OpenParen) {
            self.parse_params()?
        } else {
            Vec::new()
        };
        let (stmts, _) = self.parse_imperative_block()?;
        Ok(StatementKind::Action(ActionDecl {
            name,
            params,
            body: stmts,
        }))
    }

    /// `(name: Type, …)` — a parameter's type may be omitted (`Any`).
    fn parse_params(&mut self) -> Result<Vec<ParamDecl>> {
        self.expect(&TokenType::OpenParen, "`(`")?;
        let mut params = Vec::new();
        while !self.check(&TokenType::CloseParen) && !self.at_end() {
            let name = self.expect_ident("a parameter name")?;
            let param_type = if self.eat(&TokenType::Colon) {
                self.parse_type_ref()?
            } else {
                TypeRef::Any
            };
            params.push(ParamDecl { name, param_type });
            if !self.check(&TokenType::CloseParen) {
                self.expect(&TokenType::Comma, "`,` between parameters")?;
            }
        }
        self.expect(&TokenType::CloseParen, "`)`")?;
        Ok(params)
    }

    /// `resource name[: Type] = fetch(url, key: value, …)`.
    fn parse_resource(&mut self) -> Result<StatementKind> {
        self.expect_word("resource")?;
        let name = self.expect_ident("the resource's name")?;
        let ty = if self.eat(&TokenType::Colon) {
            Some(self.parse_type_ref()?)
        } else {
            None
        };
        self.expect(&TokenType::Equals, "`=`")?;
        if !self.is_word("fetch") {
            return Err(self.error_with_hint(
                format!("A resource is a `fetch(…)`, got {}", self.describe()),
                "Write `resource rows = fetch(\"/api/rows\")`",
            ));
        }
        self.advance();
        self.expect(&TokenType::OpenParen, "`(` after `fetch`")?;
        let url = self.parse_expression()?;
        let mut options = Vec::new();
        while self.eat(&TokenType::Comma) {
            if self.check(&TokenType::CloseParen) {
                break;
            }
            let key = self.expect_ident("an option name (`method`, `headers`, `body`)")?;
            self.expect(&TokenType::Colon, "`:`")?;
            let value = self.parse_expression()?;
            options.push(FetchOption { key, value });
        }
        self.expect(&TokenType::CloseParen, "`)`")?;
        Ok(StatementKind::Resource(ResourceDecl {
            name,
            ty,
            url,
            options,
        }))
    }

    /// `if [let x [= e]] cond { … } else if … else { … }` — in a render
    /// block the bodies hold elements, in an imperative one statements.
    fn parse_if(&mut self, imperative: bool) -> Result<StatementKind> {
        self.expect_word("if")?;
        let (binding, condition) = if self.eat_word("let") {
            let name = self.expect_ident("the name to bind")?;
            if self.eat(&TokenType::Equals) {
                (Some(name), self.parse_expression()?)
            } else {
                // `if let hint {` binds the name to itself.
                (Some(name.clone()), Expr::Identifier(name))
            }
        } else {
            (None, self.parse_expression()?)
        };
        let then_body = self.parse_branch(imperative)?;
        let mut else_if_branches = Vec::new();
        let mut else_body = None;
        while self.eat_word("else") {
            if self.eat_word("if") {
                let cond = self.parse_expression()?;
                let body = self.parse_branch(imperative)?;
                else_if_branches.push((cond, body));
            } else {
                else_body = Some(self.parse_branch(imperative)?);
                break;
            }
        }
        Ok(StatementKind::If(IfStmt {
            condition,
            binding,
            animate: None,
            animate_span: None,
            then_body,
            else_if_branches,
            else_body,
        }))
    }

    fn parse_branch(&mut self, imperative: bool) -> Result<Vec<Statement>> {
        Ok(if imperative {
            self.parse_imperative_block()?.0
        } else {
            self.parse_render_block(Body::Nested)?.0
        })
    }

    /// `for item[, index] in iterable [by key] { … }`.
    fn parse_for(&mut self) -> Result<StatementKind> {
        self.expect_word("for")?;
        let item = self.expect_ident("the loop variable")?;
        let index = if self.eat(&TokenType::Comma) {
            Some(self.expect_ident("the index variable")?)
        } else {
            None
        };
        self.expect_word("in")?;
        let iterable = self.parse_expression()?;
        let key = if self.eat_word("by") {
            Some(self.parse_expression()?)
        } else {
            None
        };
        let (stmts, _) = self.parse_render_block(Body::Nested)?;
        Ok(StatementKind::For(ForStmt {
            item,
            index,
            iterable,
            key,
            animate: None,
            animate_span: None,
            body: stmts,
        }))
    }

    /// `match x { loading { } error(e) { } ready(v) { } .case { } else { } }`.
    fn parse_match_stmt(&mut self) -> Result<StatementKind> {
        self.expect_word("match")?;
        let scrutinee = self.parse_expression()?;
        self.expect(&TokenType::OpenBrace, "`{` and the match arms")?;
        let mut arms = Vec::new();
        while !self.check(&TokenType::CloseBrace) && !self.at_end() {
            let mark = self.mark();
            let (pattern, binding) = self.parse_arm_head()?;
            let (body, _) = self.parse_render_block(Body::Nested)?;
            arms.push(MatchArm {
                pattern,
                binding,
                body,
                span: self.span_since(mark),
            });
        }
        self.expect(&TokenType::CloseBrace, "`}`")?;
        if arms.is_empty() {
            return Err(self.error("A `match` needs at least one arm".into()));
        }
        Ok(StatementKind::Match(MatchStmt { scrutinee, arms }))
    }

    fn parse_arm_head(&mut self) -> Result<(ArmPattern, Option<String>)> {
        if self.eat(&TokenType::Dot) {
            let case = self.expect_ident("a case name after `.`")?;
            return Ok((ArmPattern::Case(case), None));
        }
        let word =
            self.expect_ident("a match arm: `loading`, `error(e)`, `ready(v)`, `.case` or `else`")?;
        let pattern = match word.as_str() {
            "loading" => ArmPattern::Loading,
            "error" => ArmPattern::Error,
            "ready" => ArmPattern::Ready,
            "else" => ArmPattern::Else,
            other => {
                return Err(self.error_with_hint(
                    format!("`{other}` is not a match arm"),
                    "Arms are `loading`, `error(e)`, `ready(v)`, `.case` or `else`",
                ));
            }
        };
        let binding = if self.eat(&TokenType::OpenParen) {
            let name = self.expect_ident("the name to bind")?;
            self.expect(&TokenType::CloseParen, "`)`")?;
            if matches!(pattern, ArmPattern::Loading | ArmPattern::Else) {
                return Err(self.error(format!("`{word}` binds nothing")));
            }
            Some(name)
        } else {
            None
        };
        Ok((pattern, binding))
    }

    // ─── Elements ────────────────────────────────────────

    fn parse_element(&mut self) -> Result<UIElement> {
        let mark = self.mark();
        let name = self.expect_ident("an element")?;
        let component = if self.check(&TokenType::Dot)
            && matches!(self.kind_at(1), TokenType::Identifier(part) if part.chars().next().is_some_and(char::is_uppercase))
        {
            self.advance();
            let part = self.expect_ident("a part name")?;
            ComponentRef::SubComponent(name, part)
        } else if crate::lexer::token::ALL_COMPONENT_NAMES.contains(&name.as_str()) {
            ComponentRef::BuiltIn(name)
        } else {
            ComponentRef::UserDefined(name)
        };
        let mut el = self.blank_element(component);

        if self.check(&TokenType::OpenParen) {
            let paren_mark = self.mark();
            let (args, spans) = self.parse_call_args()?;
            el.args = args;
            el.arg_spans = spans;
            el.paren_span = Some(self.span_since(paren_mark));
        }

        // Flags: `.primary.lg`.
        while self.check(&TokenType::Dot) {
            let flag_mark = self.mark();
            self.advance();
            let flag = self.expect_ident("a flag after `.`")?;
            if self.check(&TokenType::OpenParen) {
                return Err(self.error_with_hint(
                    format!("`.{flag}` takes no arguments"),
                    &format!("A flag is a bare word; a value is a prop: `{flag}: …`"),
                ));
            }
            el.modifiers.push(flag);
            el.modifier_spans.push(self.span_since(flag_mark));
        }

        if self.check(&TokenType::OpenBrace) {
            self.parse_element_block(&mut el)?;
        }
        el.span = self.span_since(mark);
        Ok(el)
    }

    /// `( positional?, name: value, … )` — at most one positional, first.
    fn parse_call_args(&mut self) -> Result<(Vec<Arg>, Vec<Span>)> {
        self.expect(&TokenType::OpenParen, "`(`")?;
        let mut args = Vec::new();
        let mut spans = Vec::new();
        while !self.check(&TokenType::CloseParen) && !self.at_end() {
            let mark = self.mark();
            if let Some(name) = self.named_arg_ahead() {
                let width = name.matches('-').count() * 2 + 1;
                for _ in 0..width {
                    self.advance();
                }
                self.expect(&TokenType::Colon, "`:`")?;
                let value = self.parse_expression()?;
                args.push(Arg::Named(name, value));
            } else {
                if !args.is_empty() {
                    return Err(self.error_with_hint(
                        "Only the first argument may be positional".into(),
                        "Name the others: `Button(\"Save\", tone: .primary)`",
                    ));
                }
                let value = self.parse_expression()?;
                args.push(Arg::Positional(value));
            }
            spans.push(self.span_since(mark));
            if !self.check(&TokenType::CloseParen) {
                self.expect(&TokenType::Comma, "`,` between arguments")?;
            }
        }
        self.expect(&TokenType::CloseParen, "`)`")?;
        Ok((args, spans))
    }

    /// The name of a `name:` or `aria-label:` argument at the cursor, when
    /// there is one.
    fn named_arg_ahead(&self) -> Option<String> {
        let TokenType::Identifier(first) = self.kind() else {
            return None;
        };
        let mut name = first.clone();
        let mut i = 1;
        loop {
            match (self.kind_at(i), self.kind_at(i + 1)) {
                (TokenType::Colon, _) => return Some(name),
                (TokenType::Minus, TokenType::Identifier(part)) => {
                    name.push('-');
                    name.push_str(part);
                    i += 2;
                }
                _ => return None,
            }
        }
    }

    /// An element's block: `style`, `transition`, `on …`, fills, children.
    fn parse_element_block(&mut self, el: &mut UIElement) -> Result<()> {
        let open = self.expect(&TokenType::OpenBrace, "`{`")?;
        let mut stage = Stage::Style;
        let mut at = |parser: &Self, wanted: Stage| -> Result<()> {
            if wanted < stage {
                return Err(parser.error_with_hint(
                    format!(
                        "`{}` must come before the {} of the block",
                        wanted.name(),
                        stage.name()
                    ),
                    "An element's block is: style, transition, on handlers, slot fills, then children",
                ));
            }
            stage = wanted;
            Ok(())
        };
        while !self.check(&TokenType::CloseBrace) && !self.at_end() {
            let word = self.ident().map(str::to_string).unwrap_or_default();
            let next_is_brace = matches!(self.kind_at(1), TokenType::OpenBrace);
            match word.as_str() {
                "style" if next_is_brace => {
                    at(self, Stage::Style)?;
                    let style_mark = self.mark();
                    self.advance();
                    el.style_block = Some(self.parse_style_block()?);
                    el.style_span = Some(self.span_since(style_mark));
                }
                "transition" if next_is_brace => {
                    at(self, Stage::Transition)?;
                    let mark = self.mark();
                    self.advance();
                    el.transition_block = Some(self.parse_transition_block(mark)?);
                }
                "on" => {
                    at(self, Stage::Handlers)?;
                    el.events.push(self.parse_handler()?);
                }
                _ if !word.is_empty()
                    && !self.is_capitalized()
                    && next_is_brace
                    && !STATEMENT_WORDS.contains(&word.as_str()) =>
                {
                    at(self, Stage::Fills)?;
                    let mark = self.mark();
                    self.advance();
                    let (body, body_span) = self.parse_render_block(Body::Nested)?;
                    el.slot_fills.push(SlotFill {
                        name: word,
                        body,
                        span: self.span_since(mark),
                        body_span,
                    });
                }
                _ => {
                    // A bare lowercase word is a slot use (`children`,
                    // `trailing`); anything else lowercase and unknown is
                    // code that has no place here.
                    let bare = !matches!(
                        self.kind_at(1),
                        TokenType::OpenParen
                            | TokenType::Dot
                            | TokenType::Equals
                            | TokenType::Colon
                            | TokenType::OpenBracket
                    );
                    let known =
                        self.is_capitalized() || STATEMENT_WORDS.contains(&word.as_str()) || bare;
                    if word.is_empty() || !known {
                        return Err(self.error_with_hint(
                            format!("Loose code in an element's block: {}", self.describe()),
                            "What the element does goes in `on click { … }`; what it shows is an element",
                        ));
                    }
                    at(self, Stage::Children)?;
                    el.children.push(self.parse_render_statement(Body::Nested)?);
                }
            }
        }
        let close = self.expect(&TokenType::CloseBrace, "`}`")?;
        el.body_span = Some(Span::new(
            open.end as u32,
            close.offset as u32,
            open.line as u32,
            open.column as u32,
        ));
        Ok(())
    }

    /// `on name[(param)] { statements }`.
    fn parse_handler(&mut self) -> Result<EventHandler> {
        let mark = self.mark();
        self.expect_word("on")?;
        let event = self.expect_ident("the event's name")?;
        let param = if self.eat(&TokenType::OpenParen) {
            let name = self.expect_ident("the event parameter's name")?;
            self.expect(&TokenType::CloseParen, "`)`")?;
            Some(name)
        } else {
            None
        };
        let (body, _) = self.parse_imperative_block()?;
        Ok(EventHandler {
            event,
            param,
            body,
            span: self.span_since(mark),
        })
    }

    // ─── Imperative blocks ───────────────────────────────

    fn parse_imperative_block(&mut self) -> Result<(Vec<Statement>, Span)> {
        let open = self.expect(&TokenType::OpenBrace, "`{`")?;
        let mut statements = Vec::new();
        while !self.check(&TokenType::CloseBrace) && !self.at_end() {
            if self.eat(&TokenType::Semicolon) {
                continue;
            }
            statements.push(self.parse_imperative_statement()?);
        }
        let close = self.expect(&TokenType::CloseBrace, "`}`")?;
        Ok((
            statements,
            Span::new(
                open.end as u32,
                close.offset as u32,
                open.line as u32,
                open.column as u32,
            ),
        ))
    }

    fn parse_imperative_statement(&mut self) -> Result<Statement> {
        let mark = self.mark();
        let kind = self.parse_imperative_kind()?;
        Ok(Statement::new(kind, self.span_since(mark)))
    }

    fn parse_imperative_kind(&mut self) -> Result<StatementKind> {
        let word = self.ident().map(str::to_string).unwrap_or_default();
        match word.as_str() {
            "let" | "state" => {
                self.advance();
                let name = self.expect_ident("the name")?;
                let ty = if self.eat(&TokenType::Colon) {
                    Some(self.parse_type_ref()?)
                } else {
                    None
                };
                self.expect(&TokenType::Equals, "`=`")?;
                let value = self.parse_expression()?;
                // A local is a local signal, as `state` in an action always was.
                Ok(StatementKind::State(StateDecl { name, ty, value }))
            }
            "if" => self.parse_if(true),
            "return" => {
                self.advance();
                if self.check(&TokenType::CloseBrace) || self.check(&TokenType::Semicolon) {
                    Ok(StatementKind::Return(None))
                } else {
                    Ok(StatementKind::Return(Some(self.parse_expression()?)))
                }
            }
            "navigate" if matches!(self.kind_at(1), TokenType::OpenParen) => {
                self.advance();
                self.advance();
                let target = self.parse_expression()?;
                self.expect(&TokenType::CloseParen, "`)`")?;
                Ok(StatementKind::Navigate(target))
            }
            "log" if matches!(self.kind_at(1), TokenType::OpenParen) => {
                self.advance();
                self.advance();
                let value = self.parse_expression()?;
                self.expect(&TokenType::CloseParen, "`)`")?;
                Ok(StatementKind::Log(value))
            }
            "emit" => {
                self.advance();
                let event = self.expect_ident("the event to emit")?;
                let mut args = Vec::new();
                if self.eat(&TokenType::OpenParen) {
                    while !self.check(&TokenType::CloseParen) && !self.at_end() {
                        args.push(self.parse_expression()?);
                        if !self.check(&TokenType::CloseParen) {
                            self.expect(&TokenType::Comma, "`,`")?;
                        }
                    }
                    self.expect(&TokenType::CloseParen, "`)`")?;
                }
                Ok(StatementKind::Emit(EmitStmt { event, args }))
            }
            "for" | "show" | "match" | "on" | "style" => Err(self.error_with_hint(
                format!("`{word}` renders; it cannot appear inside an action or a handler"),
                "Keep what the page shows in the page's body and change state here",
            )),
            _ if crate::lexer::token::ALL_COMPONENT_NAMES.contains(&word.as_str()) => Err(self
                .error_with_hint(
                    format!(
                        "`{word}` is an element; it cannot appear inside an action or a handler"
                    ),
                    "Keep what the page shows in the page's body and change state here",
                )),
            _ => {
                let target = self.parse_expression()?;
                if self.eat(&TokenType::Equals) {
                    let value = self.parse_expression()?;
                    match &target {
                        Expr::Identifier(_) | Expr::PropertyAccess(..) | Expr::IndexAccess(..) => {}
                        _ => {
                            return Err(self.error(
                                "The left side of `=` must be a name, a field or an index".into(),
                            ));
                        }
                    }
                    Ok(StatementKind::Assignment(Assignment { target, value }))
                } else {
                    Ok(StatementKind::ExprStatement(target))
                }
            }
        }
    }

    // ─── Style blocks ────────────────────────────────────

    fn parse_style_block(&mut self) -> Result<StyleBlock> {
        let open = self.expect(&TokenType::OpenBrace, "`{`")?;
        let mut block = StyleBlock {
            properties: Vec::new(),
            media_queries: Vec::new(),
            pseudo_blocks: Vec::new(),
            body_span: Span::dummy(),
        };
        while !self.check(&TokenType::CloseBrace) && !self.at_end() {
            match self.kind().clone() {
                TokenType::StyleProp(_) => {
                    let prop = self.parse_style_property()?;
                    block.properties.push(prop);
                }
                TokenType::RawSelector(selector) => {
                    let mark = self.mark();
                    self.advance();
                    self.expect(&TokenType::OpenBrace, "`{`")?;
                    let mut properties = Vec::new();
                    while !self.check(&TokenType::CloseBrace) && !self.at_end() {
                        if let TokenType::RawSelector(inner) = self.kind() {
                            return Err(self.error_with_hint(
                                format!("`{inner}` nests inside `{selector}`"),
                                "A style block nests one level: write the rule at the top of the block",
                            ));
                        }
                        properties.push(self.parse_style_property()?);
                    }
                    self.expect(&TokenType::CloseBrace, "`}`")?;
                    let span = self.span_since(mark);
                    if selector.starts_with('@') {
                        block.media_queries.push(MediaQuery {
                            condition: selector,
                            properties,
                            span,
                        });
                    } else {
                        block.pseudo_blocks.push(PseudoBlock {
                            state: selector,
                            properties,
                            span,
                        });
                    }
                }
                TokenType::Semicolon => {
                    self.advance();
                }
                _ => {
                    return Err(self.error(format!(
                        "Expected a declaration or a nested rule in the style block, got {}",
                        self.describe()
                    )));
                }
            }
        }
        let close = self.expect(&TokenType::CloseBrace, "`}`")?;
        block.body_span = Span::new(
            open.end as u32,
            close.offset as u32,
            open.line as u32,
            open.column as u32,
        );
        Ok(block)
    }

    /// `name: value` as the lexer hands it over.
    fn parse_style_declaration(&mut self) -> Result<(String, String)> {
        let name = match self.kind() {
            TokenType::StyleProp(name) => name.clone(),
            _ => {
                return Err(self.error(format!("Expected a property, got {}", self.describe())));
            }
        };
        self.advance();
        let value = match self.kind() {
            TokenType::RawValue(v) => v.clone(),
            _ => return Err(self.error(format!("Expected a value for `{name}`"))),
        };
        self.advance();
        self.eat(&TokenType::Semicolon);
        Ok((name, value))
    }

    fn parse_style_property(&mut self) -> Result<StyleProperty> {
        let mark = self.mark();
        let (name, raw) = self.parse_style_declaration()?;
        let value_token = &self.tokens[self.pos
            - 1
            - usize::from(matches!(
                self.tokens[self.pos - 1].token_type,
                TokenType::Semicolon
            ))];
        let value_span = Span::new(
            value_token.offset as u32,
            value_token.end as u32,
            value_token.line as u32,
            value_token.column as u32,
        );
        let value = self.style_value_expr(&raw)?;
        Ok(StyleProperty {
            name,
            value,
            span: self.span_since(mark),
            value_span,
        })
    }

    /// A raw CSS value as an expression: literal text, `$token`s and
    /// `{expr}` splices, each in its place.
    fn style_value_expr(&self, raw: &str) -> Result<Expr> {
        let mut parts: Vec<StringPart> = Vec::new();
        let mut literal = String::new();
        let chars: Vec<char> = raw.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            match chars[i] {
                '{' => {
                    if !literal.is_empty() {
                        parts.push(StringPart::Literal(std::mem::take(&mut literal)));
                    }
                    let mut depth = 1;
                    let mut inner = String::new();
                    i += 1;
                    while i < chars.len() {
                        match chars[i] {
                            '{' => depth += 1,
                            '}' => {
                                depth -= 1;
                                if depth == 0 {
                                    break;
                                }
                            }
                            _ => {}
                        }
                        inner.push(chars[i]);
                        i += 1;
                    }
                    if depth != 0 {
                        return Err(self.error(format!("Unclosed `{{` in the value `{raw}`")));
                    }
                    parts.push(StringPart::Expression(self.parse_sub_expression(&inner)?));
                    i += 1;
                }
                '$' => {
                    let mut name = String::new();
                    i += 1;
                    while i < chars.len()
                        && (chars[i].is_alphanumeric() || chars[i] == '_' || chars[i] == '-')
                    {
                        name.push(chars[i]);
                        i += 1;
                    }
                    if name.is_empty() {
                        return Err(self.error(format!("A bare `$` in the value `{raw}`")));
                    }
                    if !literal.is_empty() {
                        parts.push(StringPart::Literal(std::mem::take(&mut literal)));
                    }
                    parts.push(StringPart::Expression(Expr::Token(name)));
                }
                c => {
                    literal.push(c);
                    i += 1;
                }
            }
        }
        if !literal.is_empty() {
            parts.push(StringPart::Literal(literal));
        }
        Ok(match parts.len() {
            0 => Expr::StringLiteral(String::new()),
            1 => match parts.pop().unwrap() {
                StringPart::Literal(text) => Expr::StringLiteral(text),
                StringPart::Expression(e) => e,
            },
            _ => Expr::InterpolatedString(parts),
        })
    }

    /// The text of a splice, parsed as an expression.
    fn parse_sub_expression(&self, text: &str) -> Result<Expr> {
        let tokens = LexerV2::new(text, &self.file).tokenize().map_err(|e| {
            let t = self.current();
            WebFluentError::ParseError(Diagnostic::new(
                format!("In `{{{text}}}`: {e}"),
                &self.file,
                t.line,
                t.column,
            ))
        })?;
        let mut sub = ParserV2::new(tokens, &self.file);
        let expr = sub.parse_expression()?;
        if !sub.at_end() {
            return Err(self.error(format!("Unexpected {} in `{{{text}}}`", sub.describe())));
        }
        Ok(expr)
    }

    /// `transition { property: duration [easing] … }`.
    fn parse_transition_block(&mut self, mark: (u32, u32, u32)) -> Result<TransitionBlock> {
        self.expect(&TokenType::OpenBrace, "`{`")?;
        let mut properties = Vec::new();
        while !self.check(&TokenType::CloseBrace) && !self.at_end() {
            let (property, raw) = self.parse_style_declaration()?;
            let mut words = raw.split_whitespace();
            let duration = match words.next() {
                Some("fast") => "150ms".to_string(),
                Some("normal") => "250ms".to_string(),
                Some("slow") => "350ms".to_string(),
                Some(d) => d.to_string(),
                None => {
                    return Err(self.error(format!("`{property}` needs a duration, like `200ms`")));
                }
            };
            let easing = words.next().map(str::to_string);
            properties.push(TransitionProperty {
                property,
                duration,
                easing,
            });
        }
        self.expect(&TokenType::CloseBrace, "`}`")?;
        Ok(TransitionBlock {
            properties,
            span: self.span_since(mark),
        })
    }

    // ─── Expressions ─────────────────────────────────────

    pub fn parse_expression(&mut self) -> Result<Expr> {
        self.parse_coalesce()
    }

    fn parse_coalesce(&mut self) -> Result<Expr> {
        let mut left = self.parse_or()?;
        while self.eat(&TokenType::NullCoalesce) {
            let right = self.parse_or()?;
            left = Expr::BinaryOp(Box::new(left), BinOp::NullCoalesce, Box::new(right));
        }
        Ok(left)
    }

    fn parse_or(&mut self) -> Result<Expr> {
        let mut left = self.parse_and()?;
        while self.eat(&TokenType::Or) {
            let right = self.parse_and()?;
            left = Expr::BinaryOp(Box::new(left), BinOp::Or, Box::new(right));
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expr> {
        let mut left = self.parse_equality()?;
        while self.eat(&TokenType::And) {
            let right = self.parse_equality()?;
            left = Expr::BinaryOp(Box::new(left), BinOp::And, Box::new(right));
        }
        Ok(left)
    }

    fn parse_equality(&mut self) -> Result<Expr> {
        let mut left = self.parse_comparison()?;
        loop {
            let op = match self.kind() {
                TokenType::DoubleEquals => BinOp::Eq,
                TokenType::NotEquals | TokenType::StrictNotEqual => BinOp::Neq,
                _ => break,
            };
            self.advance();
            let right = self.parse_comparison()?;
            left = Expr::BinaryOp(Box::new(left), op, Box::new(right));
        }
        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expr> {
        let mut left = self.parse_additive()?;
        loop {
            let op = match self.kind() {
                TokenType::LessThan => BinOp::Lt,
                TokenType::GreaterThan => BinOp::Gt,
                TokenType::LessEquals => BinOp::Lte,
                TokenType::GreaterEquals => BinOp::Gte,
                _ => break,
            };
            self.advance();
            let right = self.parse_additive()?;
            left = Expr::BinaryOp(Box::new(left), op, Box::new(right));
        }
        Ok(left)
    }

    fn parse_additive(&mut self) -> Result<Expr> {
        let mut left = self.parse_multiplicative()?;
        loop {
            let op = match self.kind() {
                TokenType::Plus => BinOp::Add,
                TokenType::Minus => BinOp::Sub,
                _ => break,
            };
            self.advance();
            let right = self.parse_multiplicative()?;
            left = Expr::BinaryOp(Box::new(left), op, Box::new(right));
        }
        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> Result<Expr> {
        let mut left = self.parse_unary()?;
        loop {
            let op = match self.kind() {
                TokenType::Star => BinOp::Mul,
                TokenType::Slash => BinOp::Div,
                TokenType::Percent => BinOp::Mod,
                _ => break,
            };
            self.advance();
            let right = self.parse_unary()?;
            left = Expr::BinaryOp(Box::new(left), op, Box::new(right));
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr> {
        if self.eat(&TokenType::Not) {
            return Ok(Expr::UnaryOp(UnaryOp::Not, Box::new(self.parse_unary()?)));
        }
        if self.eat(&TokenType::Minus) {
            return Ok(Expr::UnaryOp(UnaryOp::Neg, Box::new(self.parse_unary()?)));
        }
        if self.is_word("await") {
            self.advance();
            return Ok(Expr::Await(Box::new(self.parse_unary()?)));
        }
        self.parse_postfix()
    }

    fn parse_postfix(&mut self) -> Result<Expr> {
        let mut expr = self.parse_primary()?;
        loop {
            if self.check(&TokenType::Dot) {
                // `.name` or `.name(args)`; a `.` before a case is a primary.
                self.advance();
                let name = self.expect_ident("a property or method name")?;
                if self.eat(&TokenType::OpenParen) {
                    let args = self.parse_expr_list(&TokenType::CloseParen)?;
                    expr = Expr::MethodCall(Box::new(expr), name, args);
                } else {
                    expr = Expr::PropertyAccess(Box::new(expr), name);
                }
            } else if self.eat(&TokenType::OpenBracket) {
                let index = self.parse_expression()?;
                self.expect(&TokenType::CloseBracket, "`]`")?;
                expr = Expr::IndexAccess(Box::new(expr), Box::new(index));
            } else {
                break;
            }
        }
        Ok(expr)
    }

    /// Comma-separated expressions up to `close`, which is consumed.
    fn parse_expr_list(&mut self, close: &TokenType) -> Result<Vec<Expr>> {
        let mut items = Vec::new();
        while !self.check(close) && !self.at_end() {
            items.push(self.parse_expression()?);
            if !self.check(close) {
                self.expect(&TokenType::Comma, "`,`")?;
            }
        }
        self.expect(close, "the closing bracket")?;
        Ok(items)
    }

    fn parse_primary(&mut self) -> Result<Expr> {
        match self.kind().clone() {
            TokenType::StringLiteral(s) => {
                self.advance();
                if has_interpolation(&s) {
                    Ok(Expr::InterpolatedString(self.parse_interpolated(&s)?))
                } else {
                    Ok(Expr::StringLiteral(s))
                }
            }
            TokenType::NumberLiteral(n) => {
                self.advance();
                Ok(Expr::NumberLiteral(n))
            }
            TokenType::BoolLiteral(b) => {
                self.advance();
                Ok(Expr::BoolLiteral(b))
            }
            TokenType::Null => {
                self.advance();
                Ok(Expr::Null)
            }
            TokenType::DesignToken(name) => {
                self.advance();
                Ok(Expr::Token(name))
            }
            TokenType::Dot => {
                self.advance();
                let case = self.expect_ident("a case name after `.`")?;
                Ok(Expr::EnumCase(case))
            }
            TokenType::Identifier(word) => match word.as_str() {
                "if" => self.parse_if_expression(),
                "match" => self.parse_match_expression(),
                _ => {
                    self.advance();
                    // `x => body`
                    if self.eat(&TokenType::Arrow) {
                        let body = self.parse_expression()?;
                        return Ok(Expr::Lambda(word, Box::new(body)));
                    }
                    if self.check(&TokenType::OpenParen) {
                        return self.parse_call(word);
                    }
                    Ok(Expr::Identifier(word))
                }
            },
            TokenType::OpenParen => {
                // `(a, b) => body` or a grouped expression.
                if let Some(params) = self.lambda_params_ahead() {
                    for _ in 0..(params.len() * 2 + 1) {
                        self.advance();
                    }
                    self.expect(&TokenType::Arrow, "`=>`")?;
                    let body = self.parse_expression()?;
                    return Ok(Expr::Lambda(params.join(", "), Box::new(body)));
                }
                self.advance();
                let inner = self.parse_expression()?;
                self.expect(&TokenType::CloseParen, "`)`")?;
                Ok(inner)
            }
            TokenType::OpenBracket => {
                self.advance();
                let items = self.parse_expr_list(&TokenType::CloseBracket)?;
                Ok(Expr::ListLiteral(items))
            }
            TokenType::OpenBrace => self.parse_map_literal(),
            _ => Err(self.error(format!("Expected an expression, got {}", self.describe()))),
        }
    }

    /// `name(args)`: a call, or — for a Capitalised name with named
    /// arguments — a record: `Todo(id: 1, title: "x")` is an object.
    fn parse_call(&mut self, name: String) -> Result<Expr> {
        self.expect(&TokenType::OpenParen, "`(`")?;
        let record =
            name.chars().next().is_some_and(char::is_uppercase) && self.named_arg_ahead().is_some();
        if record {
            let mut fields = Vec::new();
            while !self.check(&TokenType::CloseParen) && !self.at_end() {
                let key = self.expect_ident("a field name")?;
                self.expect(&TokenType::Colon, "`:`")?;
                let value = self.parse_expression()?;
                fields.push((key, value));
                if !self.check(&TokenType::CloseParen) {
                    self.expect(&TokenType::Comma, "`,`")?;
                }
            }
            self.expect(&TokenType::CloseParen, "`)`")?;
            return Ok(Expr::MapLiteral(fields));
        }
        let args = self.parse_expr_list(&TokenType::CloseParen)?;
        Ok(Expr::FunctionCall(name, args))
    }

    /// `(a, b) =>` at the cursor: the parameter names.
    fn lambda_params_ahead(&self) -> Option<Vec<String>> {
        let mut params = Vec::new();
        let mut i = 1;
        loop {
            match self.kind_at(i) {
                TokenType::Identifier(name) => params.push(name.clone()),
                _ => return None,
            }
            i += 1;
            match self.kind_at(i) {
                TokenType::Comma => i += 1,
                TokenType::CloseParen => {
                    return matches!(self.kind_at(i + 1), TokenType::Arrow).then_some(params);
                }
                _ => return None,
            }
        }
    }

    fn parse_if_expression(&mut self) -> Result<Expr> {
        self.expect_word("if")?;
        let condition = self.parse_expression()?;
        self.expect(&TokenType::OpenBrace, "`{`")?;
        let then_expr = self.parse_expression()?;
        self.expect(&TokenType::CloseBrace, "`}`")?;
        self.expect_word("else")?;
        let else_expr = if self.is_word("if") {
            self.parse_if_expression()?
        } else {
            self.expect(&TokenType::OpenBrace, "`{`")?;
            let e = self.parse_expression()?;
            self.expect(&TokenType::CloseBrace, "`}`")?;
            e
        };
        Ok(Expr::MethodCall(
            Box::new(condition),
            "__if".to_string(),
            vec![then_expr, else_expr],
        ))
    }

    /// `match x { .a { e1 } .b { e2 } else { e3 } }` as a chain of
    /// conditionals; a resource match is a statement, not an expression.
    fn parse_match_expression(&mut self) -> Result<Expr> {
        self.expect_word("match")?;
        let subject = self.parse_expression()?;
        self.expect(&TokenType::OpenBrace, "`{`")?;
        let mut arms: Vec<(Option<String>, Expr)> = Vec::new();
        while !self.check(&TokenType::CloseBrace) && !self.at_end() {
            let case = if self.eat(&TokenType::Dot) {
                Some(self.expect_ident("a case name")?)
            } else if self.eat_word("else") {
                None
            } else {
                return Err(self.error_with_hint(
                    format!("Expected `.case` or `else`, got {}", self.describe()),
                    "A match expression chooses by enum case; `loading`/`error`/`ready` arms belong to a match statement",
                ));
            };
            self.expect(&TokenType::OpenBrace, "`{`")?;
            let value = self.parse_expression()?;
            self.expect(&TokenType::CloseBrace, "`}`")?;
            arms.push((case, value));
        }
        self.expect(&TokenType::CloseBrace, "`}`")?;
        let Some(fallback) = arms
            .iter()
            .find(|(c, _)| c.is_none())
            .map(|(_, e)| e.clone())
        else {
            return Err(self.error_with_hint(
                "A match expression needs an `else` arm".into(),
                "Every case must produce a value; `else { … }` covers the rest",
            ));
        };
        let mut expr = fallback;
        for (case, value) in arms.into_iter().rev() {
            if let Some(case) = case {
                let cond = Expr::BinaryOp(
                    Box::new(subject.clone()),
                    BinOp::Eq,
                    Box::new(Expr::EnumCase(case)),
                );
                expr = Expr::MethodCall(Box::new(cond), "__if".to_string(), vec![value, expr]);
            }
        }
        Ok(expr)
    }

    fn parse_map_literal(&mut self) -> Result<Expr> {
        self.expect(&TokenType::OpenBrace, "`{`")?;
        let mut pairs = Vec::new();
        while !self.check(&TokenType::CloseBrace) && !self.at_end() {
            let key = match self.kind().clone() {
                TokenType::Identifier(w) => {
                    self.advance();
                    w
                }
                TokenType::StringLiteral(s) => {
                    self.advance();
                    format!("\"{s}\"")
                }
                _ => return Err(self.error(format!("Expected a map key, got {}", self.describe()))),
            };
            self.expect(&TokenType::Colon, "`:`")?;
            let value = self.parse_expression()?;
            pairs.push((key, value));
            if !self.check(&TokenType::CloseBrace) {
                self.expect(&TokenType::Comma, "`,`")?;
            }
        }
        self.expect(&TokenType::CloseBrace, "`}`")?;
        Ok(Expr::MapLiteral(pairs))
    }

    fn parse_interpolated(&self, s: &str) -> Result<Vec<StringPart>> {
        let mut parts = Vec::new();
        let mut literal = String::new();
        let chars: Vec<char> = s.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            if chars[i] == '{' {
                if !literal.is_empty() {
                    parts.push(StringPart::Literal(std::mem::take(&mut literal)));
                }
                let mut depth = 1;
                let mut inner = String::new();
                i += 1;
                while i < chars.len() {
                    match chars[i] {
                        '{' => depth += 1,
                        '}' => {
                            depth -= 1;
                            if depth == 0 {
                                break;
                            }
                        }
                        _ => {}
                    }
                    inner.push(chars[i]);
                    i += 1;
                }
                parts.push(StringPart::Expression(self.parse_sub_expression(&inner)?));
                i += 1;
            } else {
                literal.push(chars[i]);
                i += 1;
            }
        }
        if !literal.is_empty() {
            parts.push(StringPart::Literal(literal));
        }
        Ok(parts)
    }
}

/// Whether a string literal holds a `{name…}` splice (a `{` followed by a
/// name and closed on the same line, with no `:` or `,`).
fn has_interpolation(s: &str) -> bool {
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '{'
            && chars
                .get(i + 1)
                .is_some_and(|c| c.is_alphabetic() || *c == '_')
        {
            let mut j = i + 2;
            let mut valid = true;
            while j < chars.len() && chars[j] != '}' {
                if matches!(chars[j], '\n' | ':' | ',') {
                    valid = false;
                    break;
                }
                j += 1;
            }
            if valid && j < chars.len() {
                return true;
            }
        }
        i += 1;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(src: &str) -> Program {
        parse_v2(src, "<t>").unwrap_or_else(|e| panic!("{e}\n---\n{src}"))
    }

    fn fails(src: &str) -> String {
        match parse_v2(src, "<t>") {
            Ok(_) => panic!("parsed, should not have:\n{src}"),
            Err(e) => e.to_string(),
        }
    }

    fn page_body(src: &str) -> Vec<Statement> {
        match parse(src).declarations.into_iter().next() {
            Some(Declaration::Page(p)) => p.body,
            other => panic!("expected a page, got {other:?}"),
        }
    }

    fn first_element(src: &str) -> UIElement {
        match page_body(src).into_iter().next().map(|s| s.kind) {
            Some(StatementKind::UIElement(el)) => el,
            other => panic!("expected an element, got {other:?}"),
        }
    }

    #[test]
    fn the_whole_reference_example_parses() {
        let src = r#"
enum Tone { neutral, info, danger }
type Todo { id: String, title: String, done: Bool = false, tone: Tone = .neutral }

store Todos {
    state items: [Todo] = []
    derived remaining = items.filter(t => !t.done).length
    action add(title: String) { items.push(Todo(id: uid(), title: title)) }
    action toggle(id: String) {
        let todo = items.find(t => t.id == id)
        todo.done = !todo.done
    }
}

/// One todo with its checkbox. `toggle` fires with the todo's id.
component TodoRow(_ label: String, todo: Todo, compact: Bool = false) {
    event toggle(id: String)
    slot trailing

    Row(align: .center, gap: .sm) {
        style {
            padding: 6px 0
            --accent: {todo.color}
            background: $surface
            &:hover { background: $surface-hover }
            @media (max-width: 768px) { gap: 4px }
        }
        transition { background: 150ms ease-out }
        on click(e) { emit toggle(todo.id) }
        trailing { Icon("x") }
        Checkbox(bind: todo.done, label: label)
        if let hint { Kbd(hint) }
        trailing
    }
}

page Home(path: "/", title: "Todos", layout: AppShell(crumb: "x")) {
    use Todos
    state draft = ""
    resource list = fetch("/api", method: "GET")

    match list {
        loading { Skeleton() }
        error(e) { Alert(e.message).danger }
        ready(v) {
            for t in v by t.id {
                TodoRow(t.title, todo: t) { on toggle(id) { Todos.toggle(id) } }
            }
        }
    }
    Button("Add").primary.lg { on click { Todos.add(draft); draft = "" } }
    Text("{Todos.remaining} left ?? nothing")
}

app { Navbar(brand: "x") { Navbar.Links { Link("Home", to: "/") } }  Router }
"#;
        let program = parse(src);
        assert_eq!(program.declarations.len(), 6);
        let Declaration::Component(row) = &program.declarations[3] else {
            panic!()
        };
        assert_eq!(
            row.doc.as_deref(),
            Some("One todo with its checkbox. `toggle` fires with the todo's id.")
        );
        assert!(row.props[0].positional && row.props[0].name == "label");
        assert_eq!(row.props[2].prop_type, TypeRef::Bool);
        assert_eq!(row.events[0].name, "toggle");
        assert_eq!(row.events[0].params[0].param_type, TypeRef::String);
        assert_eq!(row.slots[0].name.as_deref(), Some("trailing"));
        let StatementKind::UIElement(root) = &row.body[0].kind else {
            panic!()
        };
        assert!(matches!(&root.component, ComponentRef::BuiltIn(n) if n == "Row"));
        assert!(
            matches!(&root.args[0], Arg::Named(k, Expr::EnumCase(c)) if k == "align" && c == "center")
        );
        let style = root.style_block.as_ref().unwrap();
        assert!(matches!(&style.properties[0].value, Expr::StringLiteral(s) if s == "6px 0"));
        assert!(matches!(
            &style.properties[1].value,
            Expr::PropertyAccess(..)
        ));
        assert!(matches!(&style.properties[2].value, Expr::Token(t) if t == "surface"));
        assert_eq!(style.pseudo_blocks[0].state, "&:hover");
        assert_eq!(
            style.media_queries[0].condition,
            "@media (max-width: 768px)"
        );
        let transition = root.transition_block.as_ref().unwrap();
        assert_eq!(transition.properties[0].duration, "150ms");
        assert_eq!(transition.properties[0].easing.as_deref(), Some("ease-out"));
        assert_eq!(root.events[0].param.as_deref(), Some("e"));
        assert!(
            matches!(&root.events[0].body[0].kind, StatementKind::Emit(e) if e.event == "toggle")
        );
        assert_eq!(root.slot_fills[0].name, "trailing");
        assert_eq!(root.children.len(), 3);
        let StatementKind::If(i) = &root.children[1].kind else {
            panic!()
        };
        assert_eq!(i.binding.as_deref(), Some("hint"));
        let StatementKind::UIElement(slot) = &root.children[2].kind else {
            panic!()
        };
        assert_eq!(slot.slot_name(), Some("trailing"));

        let Declaration::Page(home) = &program.declarations[4] else {
            panic!()
        };
        assert_eq!(home.layout.as_ref().unwrap().name, "AppShell");
        assert!(
            matches!(&home.body[2].kind, StatementKind::Resource(r) if r.name == "list" && r.options[0].key == "method")
        );
        let StatementKind::Match(m) = &home.body[3].kind else {
            panic!()
        };
        assert_eq!(m.arms.len(), 3);
        assert_eq!(m.arms[1].binding.as_deref(), Some("e"));
        let StatementKind::For(f) = &m.arms[2].body[0].kind else {
            panic!()
        };
        assert!(f.key.is_some());
        let StatementKind::UIElement(call) = &f.body[0].kind else {
            panic!()
        };
        assert!(matches!(&call.component, ComponentRef::UserDefined(n) if n == "TodoRow"));
        assert_eq!(call.events[0].event, "toggle");
        let StatementKind::UIElement(button) = &home.body[4].kind else {
            panic!()
        };
        assert_eq!(button.modifiers, vec!["primary", "lg"]);
        assert_eq!(button.events[0].body.len(), 2);
    }

    #[test]
    fn a_flag_after_the_element_is_a_modifier_with_its_span() {
        let src = "page P(path: \"/\") { Button(\"Save\").primary.lg }";
        let el = first_element(src);
        assert_eq!(el.modifiers, vec!["primary", "lg"]);
        assert_eq!(el.modifier_spans[1].slice(src), ".lg");
        assert_eq!(el.span.slice(src), "Button(\"Save\").primary.lg");
    }

    #[test]
    fn a_part_is_a_capitalised_dot_and_a_flag_takes_no_arguments() {
        let el = first_element("page P(path: \"/\") { Card.Header { Text(\"x\") } }");
        assert!(
            matches!(&el.component, ComponentRef::SubComponent(o, p) if o == "Card" && p == "Header")
        );
        let err = fails("page P(path: \"/\") { Row.gap(md) { } }");
        assert!(err.contains("`.gap` takes no arguments"), "{err}");
    }

    #[test]
    fn only_the_first_argument_may_be_positional() {
        let err = fails("page P(path: \"/\") { Option(\"a\", \"b\") }");
        assert!(
            err.contains("Only the first argument may be positional"),
            "{err}"
        );
        let el =
            first_element("page P(path: \"/\") { Select.Option(\"Admin\", value: \"admin\") }");
        assert!(matches!(&el.args[0], Arg::Positional(_)));
        assert!(matches!(&el.args[1], Arg::Named(k, _) if k == "value"));
    }

    #[test]
    fn the_block_order_is_enforced_with_a_hint() {
        let err = fails("page P(path: \"/\") { Button(\"x\") { Text(\"y\")  on click { go() } } }");
        assert!(err.contains("must come before"), "{err}");
        let err = fails(
            "page P(path: \"/\") { Button(\"x\") { on click { go() }  style { padding: 0 } } }",
        );
        assert!(err.contains("`style` must come before"), "{err}");
    }

    #[test]
    fn loose_code_in_a_render_block_is_an_error_with_a_hint() {
        let err = fails("page P(path: \"/\") { Button(\"x\") { save() } }");
        assert!(err.contains("Loose code"), "{err}");
        let err = fails("page P(path: \"/\") { count = 1 }");
        assert!(
            err.contains("`count` is not an element or a statement"),
            "{err}"
        );
    }

    #[test]
    fn the_old_grammar_is_named_and_pointed_at_the_migration() {
        let err = fails("page Home(path: \"/\") { }\nPage Other (path: \"/o\") { }");
        assert!(
            err.contains("WebFluent 2 declaration") && err.contains("wf migrate"),
            "{err}"
        );
    }

    #[test]
    fn expressions_carry_cases_tokens_coalescing_and_await() {
        let body = page_body(
            "page P(path: \"/\") {\n  state t = .danger\n  derived c = t == .danger ?? $line\n  action go() { let r = await fetch(\"/x\")  x = r ?? 0 }\n  derived m = match t { .danger { 1 } else { 2 } }\n}",
        );
        assert!(
            matches!(&body[0].kind, StatementKind::State(s) if matches!(&s.value, Expr::EnumCase(c) if c == "danger"))
        );
        assert!(
            matches!(&body[1].kind, StatementKind::Derived(d) if matches!(&d.value, Expr::BinaryOp(_, BinOp::NullCoalesce, r) if matches!(**r, Expr::Token(_))))
        );
        let StatementKind::Action(a) = &body[2].kind else {
            panic!()
        };
        assert!(
            matches!(&a.body[0].kind, StatementKind::State(s) if s.name == "r" && matches!(s.value, Expr::Await(_)))
        );
        assert!(matches!(&a.body[1].kind, StatementKind::Assignment(_)));
        assert!(
            matches!(&body[3].kind, StatementKind::Derived(d) if matches!(&d.value, Expr::MethodCall(_, m, _) if m == "__if"))
        );
    }

    #[test]
    fn a_call_at_the_top_of_a_page_is_set_up_code() {
        let body = page_body(
            "page P(path: \"/\", hash: String) {\n  use BuildStore\n  BuildStore.open(hash)\n  load()\n  Text(\"x\")\n}",
        );
        assert!(
            matches!(&body[1].kind, StatementKind::ExprStatement(Expr::MethodCall(_, m, _)) if m == "open")
        );
        assert!(
            matches!(&body[2].kind, StatementKind::ExprStatement(Expr::FunctionCall(f, _)) if f == "load")
        );
        assert!(matches!(&body[3].kind, StatementKind::UIElement(_)));
        // Inside an element it is still nothing a render block holds.
        let err = fails("page P(path: \"/\") { Card { Store.load() } }");
        assert!(err.contains("`.load` takes no arguments"), "{err}");
        let err = fails("page P(path: \"/\") { Card { load() } }");
        assert!(err.contains("Loose code in an element's block"), "{err}");
    }

    #[test]
    fn lambdas_take_one_parameter_or_a_parenthesised_list() {
        let body = page_body(
            "page P(path: \"/\") {\n  derived a = xs.map(x => x.n)\n  derived b = xs.reduce((n, g) => n + g.k, 0)\n  derived c = xs.map((l, n) => { n: n + 1, t: l.t })\n}",
        );
        let lambda = |i: usize| match &body[i].kind {
            StatementKind::Derived(d) => match &d.value {
                Expr::MethodCall(_, _, args) => match &args[0] {
                    Expr::Lambda(params, _) => params.clone(),
                    other => panic!("{other:?}"),
                },
                other => panic!("{other:?}"),
            },
            other => panic!("{other:?}"),
        };
        assert_eq!(lambda(0), "x");
        assert_eq!(lambda(1), "n, g");
        assert_eq!(lambda(2), "l, n");
    }

    #[test]
    fn a_record_with_named_fields_is_a_map_and_a_positional_call_a_call() {
        let body = page_body(
            "page P(path: \"/\") { state a = Todo(id: 1, title: \"x\")  state b = Number(\"3\") }",
        );
        assert!(
            matches!(&body[0].kind, StatementKind::State(s) if matches!(&s.value, Expr::MapLiteral(f) if f.len() == 2 && f[0].0 == "id"))
        );
        assert!(
            matches!(&body[1].kind, StatementKind::State(s) if matches!(&s.value, Expr::FunctionCall(n, _) if n == "Number"))
        );
    }

    #[test]
    fn a_page_takes_route_params_and_rejects_a_lowercase_layout() {
        let program = parse("page D(path: \"/d/:id\", id: String, title: \"D\") { Text(id) }");
        let Declaration::Page(p) = &program.declarations[0] else {
            panic!()
        };
        assert_eq!(p.params[0].name, "id");
        assert_eq!(p.title.as_deref(), Some("D"));
        let err = fails("page D(path: \"/\", layout: shell) { }");
        assert!(err.contains("not a component name"), "{err}");
    }

    #[test]
    fn the_positional_prop_must_come_first_and_types_read_as_written() {
        let err = fails("component C(a: String, _ b: String) { Text(b) }");
        assert!(err.contains("declared first"), "{err}");
        let program = parse(
            "component C(items: [Todo], hint: String? = null, n: Number = 3) { Text(hint ?? \"\") }",
        );
        let Declaration::Component(c) = &program.declarations[0] else {
            panic!()
        };
        assert_eq!(
            c.props[0].prop_type,
            TypeRef::List(Box::new(TypeRef::Named("Todo".into())))
        );
        assert_eq!(
            c.props[1].prop_type,
            TypeRef::Optional(Box::new(TypeRef::String))
        );
        assert!(c.props[1].optional);
        let err = fails("component C(items: List) { }");
        assert!(err.contains("`List` is not a type"), "{err}");
    }

    #[test]
    fn a_style_value_that_mixes_text_and_splices_is_an_interpolation() {
        let el = first_element(
            "page P(path: \"/\") { Text(\"x\") { style { width: {size}px; border: 1px solid $line } } }",
        );
        let style = el.style_block.unwrap();
        let Expr::InterpolatedString(parts) = &style.properties[0].value else {
            panic!("{:?}", style.properties[0].value)
        };
        assert!(matches!(&parts[0], StringPart::Expression(Expr::Identifier(n)) if n == "size"));
        assert!(matches!(&parts[1], StringPart::Literal(s) if s == "px"));
        let Expr::InterpolatedString(parts) = &style.properties[1].value else {
            panic!()
        };
        assert!(matches!(&parts[0], StringPart::Literal(s) if s == "1px solid "));
        assert!(matches!(&parts[1], StringPart::Expression(Expr::Token(t)) if t == "line"));
    }

    #[test]
    fn a_theme_is_raw_css_values() {
        let program = parse(
            "theme Brand {\n  color-primary: #8B5CF6\n  font-sans: \"Manrope\", sans-serif\n}",
        );
        let Declaration::Theme(t) = &program.declarations[0] else {
            panic!()
        };
        assert_eq!(t.tokens[0].name, "color-primary");
        assert!(matches!(&t.tokens[0].value, Expr::StringLiteral(v) if v == "#8B5CF6"));
        assert!(
            matches!(&t.tokens[1].value, Expr::StringLiteral(v) if v == "\"Manrope\", sans-serif")
        );
    }

    #[test]
    fn handlers_and_actions_hold_statements_not_elements() {
        let err = fails("page P(path: \"/\") { Button(\"x\") { on click { Text(\"no\") } } }");
        assert!(err.contains("`Text` is an element"), "{err}");
        let err = fails("page P(path: \"/\") { action go() { show open { } } }");
        assert!(err.contains("renders"), "{err}");
    }

    #[test]
    fn a_match_expression_needs_an_else_and_a_match_statement_an_arm() {
        let err = fails("page P(path: \"/\") { derived x = match t { .a { 1 } } }");
        assert!(err.contains("needs an `else` arm"), "{err}");
        let err = fails("page P(path: \"/\") { match t { } }");
        assert!(err.contains("at least one arm"), "{err}");
        let err = fails("page P(path: \"/\") { match t { loading(x) { } } }");
        assert!(err.contains("binds nothing"), "{err}");
    }
}
