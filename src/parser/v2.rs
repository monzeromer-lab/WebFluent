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
use crate::lexer::{StringKind, StringLit, Token, TokenType};
use crate::parser::ast::*;

/// Parse a WebFluent 3 source file, in the layout its name asks for: a
/// `.wfx` file writes its blocks by indentation.
pub fn parse_v2(source: &str, file: &str) -> Result<Program> {
    let tokens = LexerV2::for_file(source, file).tokenize()?;
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
    /// A `test` body: elements and `expect` lines.
    Test,
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
    /// The `part` declarations, parsed as components of their own.
    parts: Vec<ComponentDecl>,
}

pub struct ParserV2 {
    tokens: Vec<Token>,
    pos: usize,
    file: String,
    /// Destructuring `let`s seen, for the name of each one's temporary.
    destructures: usize,
    /// Components a `part` declares, to land beside their owner.
    hoisted: Vec<Declaration>,
    /// The `cleanup { … }` block an effect's body ended with.
    pending_cleanup: Option<Vec<Statement>>,
    /// The depth of the imperative block that is an effect's own body,
    /// where `cleanup` may close it.
    effect_body: Option<usize>,
    /// How many imperative blocks are open.
    block_depth: usize,
}

const CLAUSE_WORDS: &[&str] = &["style", "transition", "on"];
const STATEMENT_WORDS: &[&str] = &[
    "state", "persist", "derived", "effect", "action", "use", "resource", "event", "slot", "part",
    "if", "for", "show", "match", "children", "let", "return", "navigate", "log", "emit", "else",
    // `sequence { step { … } }` is orchestration, not a slot fill: it is a
    // lowercase word before a block, which is what a fill looks like.
    "sequence",
];

impl ParserV2 {
    pub fn new(tokens: Vec<Token>, file: &str) -> Self {
        Self {
            tokens,
            pos: 0,
            file: file.to_string(),
            destructures: 0,
            hoisted: Vec::new(),
            pending_cleanup: None,
            effect_body: None,
            block_depth: 0,
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
    /// Whether the token `ahead` of the cursor is a string literal.
    ///
    /// `type "Ada" into "Name"` is a step; `type Todo { … }` is a
    /// declaration, and the difference is what follows the word.
    fn is_string_at(&self, ahead: usize) -> bool {
        matches!(self.kind_at(ahead), TokenType::StringLiteral(_))
    }

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
            TokenType::StringLiteral(s) => format!("the string \"{}\"", s.spelling),
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
                "api" => self.parse_api(doc)?,
                "external" => self.parse_external(doc)?,
                "const" => self.parse_const_decl(doc)?,
                "animation" => self.parse_animation_decl(doc)?,
                "test" if matches!(self.kind_at(1), TokenType::StringLiteral(_)) => {
                    self.parse_test_decl()?
                }
                "data" if matches!(self.kind_at(1), TokenType::Identifier(_)) => {
                    self.parse_data_decl(doc)?
                }
                // `image hero = "media/hero.jpg"`: read at build time for
                // its size, its colour and every width the page asks for.
                "image"
                    if matches!(self.kind_at(1), TokenType::Identifier(_))
                        && matches!(self.kind_at(2), TokenType::Equals | TokenType::Colon) =>
                {
                    self.parse_data_decl(doc)?
                }
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
                            "Expected a declaration — page, component, store, theme, app, type, enum, const, animation, data or test — got {}",
                            self.describe()
                        ),
                        "Every file is a list of declarations; elements live inside a page or component",
                    ));
                }
            };
            declarations.push(decl);
            declarations.append(&mut self.hoisted);
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
            head: Vec::new(),
            paths: None,
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
                    // The values a static build renders a `:param` page for.
                    "paths" => page.paths = Some(self.parse_expression()?),
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
        let (mut body, body_span) = self.parse_render_block(Body::Page)?;
        // `head { … }` was parsed as a statement of the body; it is the
        // page's own.
        body.retain(|stmt| match &stmt.kind {
            StatementKind::UIElement(el)
                if matches!(&el.component, ComponentRef::BuiltIn(n) if n == "__Head") =>
            {
                page.head.extend(el.children.iter().filter_map(|c| match &c.kind {
                    StatementKind::UIElement(tag) => Some(HeadTag {
                        tag: match &tag.component {
                            ComponentRef::BuiltIn(t) | ComponentRef::UserDefined(t) => {
                                t.to_lowercase()
                            }
                            ComponentRef::SubComponent(a, b) => format!("{a}.{b}"),
                        },
                        attrs: tag
                            .args
                            .iter()
                            .filter_map(|a| match a {
                                Arg::Named(k, v) => Some((k.clone(), v.clone())),
                                Arg::Positional(_) => None,
                            })
                            .collect(),
                        span: c.span,
                    }),
                    _ => None,
                }));
                false
            }
            _ => true,
        });
        page.body = body;
        page.body_span = body_span;
        page.span = self.span_since(mark);
        Ok(Declaration::Page(page))
    }

    fn expect_string(&mut self, what: &str) -> Result<String> {
        match self.kind() {
            TokenType::StringLiteral(s) => {
                let s = string_text(s);
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
        self.parse_component_like("component", None, doc)
    }

    /// `component Name(props) { … }`, or a `part Name(props) { … }` of
    /// `owner`, which is the component `Owner.Name`.
    fn parse_component_like(
        &mut self,
        keyword: &str,
        owner: Option<&str>,
        doc: Option<String>,
    ) -> Result<Declaration> {
        let mark = self.mark();
        self.expect_word(keyword)?;
        let mut name = self.expect_ident(&format!("the {keyword}'s name"))?;
        if let Some(owner) = owner {
            name = format!("{owner}.{name}");
        }
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
            parts,
        } = self.parse_component_body(&name)?;
        let part_names = parts
            .iter()
            .map(|p| p.name.rsplit('.').next().unwrap_or(&p.name).to_string())
            .collect();
        self.hoisted
            .extend(parts.into_iter().map(Declaration::Component));
        Ok(Declaration::Component(ComponentDecl {
            name,
            props,
            events,
            slots,
            parts: part_names,
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
        // `Number(0..=100)`, `String(minLength: 8)`, `Date(after: @2026-01-01)`
        // — what the values of this type must be, checked at every
        // assignment and read by validation to say why one was refused.
        let base = if self.eat(&TokenType::OpenParen) {
            let mut args: Vec<(String, Expr)> = Vec::new();
            while !self.check(&TokenType::CloseParen) && !self.at_end() {
                if let TokenType::Identifier(name) = self.kind().clone()
                    && matches!(self.kind_at(1), TokenType::Colon)
                {
                    self.advance();
                    self.advance();
                    args.push((name, self.parse_expression()?));
                } else {
                    // A range is the two ends: `0..=100` is `min` and `max`.
                    match self.parse_expression()? {
                        Expr::Range(from, to, inclusive) => {
                            args.push(("min".to_string(), *from));
                            args.push(if inclusive {
                                ("max".to_string(), *to)
                            } else {
                                ("below".to_string(), *to)
                            });
                        }
                        other => args.push(("is".to_string(), other)),
                    }
                }
                if !self.check(&TokenType::CloseParen) {
                    self.expect(&TokenType::Comma, "`,`")?;
                }
            }
            self.expect(&TokenType::CloseParen, "`)` to close the condition")?;
            TypeRef::Refined(Box::new(base), args)
        } else {
            base
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
        let (scope, eager) = self.parse_store_options()?;
        let header_span = self.span_since(mark);
        let (body, body_span) = self.parse_render_block(Body::Store)?;
        Ok(Declaration::Store(StoreDecl {
            name,
            scope,
            eager,
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
        // `type Admin = User { … }` extends `User`.
        let extends = if self.eat(&TokenType::Equals) {
            Some(self.expect_ident("the record to extend")?)
        } else {
            None
        };
        let header_span = self.span_since(mark);
        self.expect(&TokenType::OpenBrace, "`{`")?;
        let fields = self.parse_field_list(&TokenType::CloseBrace)?;
        self.expect(&TokenType::CloseBrace, "`}`")?;
        Ok(Declaration::Type(TypeDecl {
            name,
            extends,
            fields,
            doc,
            span: self.span_since(mark),
            header_span,
        }))
    }

    /// `name: Type = default, …` up to `close`, which is left in place: a
    /// record's fields, or an enum case's payload.
    fn parse_field_list(&mut self, close: &TokenType) -> Result<Vec<FieldDecl>> {
        let mut fields = Vec::new();
        while !self.check(close) && !self.at_end() {
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
            if !self.check(close) {
                self.eat(&TokenType::Comma);
            }
        }
        Ok(fields)
    }

    /// `animation Pulse { from { opacity: 1 } 50% { opacity: 0.4 } to { opacity: 1 } }`.
    fn parse_animation_decl(&mut self, doc: Option<String>) -> Result<Declaration> {
        let mark = self.mark();
        self.expect_word("animation")?;
        let name = self.expect_ident("the animation's name")?;
        if !name.starts_with(char::is_uppercase) {
            return Err(self.error_with_hint(
                format!("`{name}` is not an animation name"),
                "An animation's name is capitalised, like a component's: `animation Pulse`",
            ));
        }
        self.expect(&TokenType::OpenBrace, "`{`")?;
        let mut frames = Vec::new();
        while !self.check(&TokenType::CloseBrace) && !self.at_end() {
            let TokenType::RawSelector(selector) = self.kind().clone() else {
                return Err(self.error_with_hint(
                    format!("Expected a keyframe, got {}", self.describe()),
                    "A keyframe is `from { … }`, `to { … }` or a percentage: `50% { … }`",
                ));
            };
            self.advance();
            self.expect(&TokenType::OpenBrace, "`{`")?;
            let mut properties = Vec::new();
            while !self.check(&TokenType::CloseBrace) && !self.at_end() {
                properties.push(self.parse_style_property()?);
            }
            self.expect(&TokenType::CloseBrace, "`}`")?;
            frames.push(KeyFrame {
                selector,
                properties,
            });
        }
        self.expect(&TokenType::CloseBrace, "`}`")?;
        if frames.is_empty() {
            return Err(self.error(format!("`animation {name}` has no keyframes")));
        }
        Ok(Declaration::Animation(AnimationDecl {
            name,
            frames,
            doc,
            span: self.span_since(mark),
        }))
    }

    /// `data posts = "posts.json"`, `data posts: [Post] = "content/posts.json"`.
    /// `data posts = "posts.json"`, and `image hero = "media/hero.jpg"` —
    /// both a constant whose value the build reads from a file.
    fn parse_data_decl(&mut self, doc: Option<String>) -> Result<Declaration> {
        let mark = self.mark();
        let is_image = self.is_word("image");
        if is_image {
            self.advance();
        } else {
            self.expect_word("data")?;
        }
        let name = self.expect_ident(if is_image {
            "the image's name"
        } else {
            "the data's name"
        })?;
        let ty = if self.eat(&TokenType::Colon) {
            Some(self.parse_type_ref()?)
        } else {
            None
        };
        self.expect(&TokenType::Equals, "`=` and the file's name")?;
        let file = match self.kind().clone() {
            TokenType::StringLiteral(f) => {
                self.advance();
                string_text(&f)
            }
            _ => {
                return Err(self.error_with_hint(
                    "Expected the file's name, in quotes".into(),
                    "Write `data posts = \"posts.json\"`; the file is read at build time",
                ));
            }
        };
        if is_image {
            let known = [".png", ".jpg", ".jpeg", ".webp", ".gif", ".svg"];
            if !known.iter().any(|e| file.to_lowercase().ends_with(e)) {
                return Err(self.error_with_hint(
                    format!("`{file}` is not an image"),
                    "An image is a .png, .jpg, .webp, .gif or .svg file",
                ));
            }
        } else if !file.ends_with(".json") {
            return Err(self.error_with_hint(
                format!("`{file}` is not a JSON file"),
                "A data file is JSON: a list, a map, or a value",
            ));
        }
        Ok(Declaration::Data(DataDecl {
            name,
            ty,
            file,
            doc,
            span: self.span_since(mark),
            is_image,
        }))
    }

    /// `test "name"(data: { … }) { elements  expect "text"  expect not "x" }`.
    fn parse_test_decl(&mut self) -> Result<Declaration> {
        let mark = self.mark();
        self.expect_word("test")?;
        let name = match self.kind().clone() {
            TokenType::StringLiteral(s) => {
                self.advance();
                string_text(&s)
            }
            _ => return Err(self.error("Expected the test's name, in quotes".into())),
        };
        let mut data = None;
        if self.eat(&TokenType::OpenParen) {
            while !self.check(&TokenType::CloseParen) && !self.at_end() {
                let key = self.expect_ident("`data`")?;
                self.expect(&TokenType::Colon, "`:`")?;
                let value = self.parse_expression()?;
                if key == "data" {
                    data = Some(value);
                } else {
                    return Err(self.error_with_hint(
                        format!("`{key}` is not something a test takes"),
                        "A test takes `data: { … }`, the values its body renders over",
                    ));
                }
                if !self.check(&TokenType::CloseParen) {
                    self.expect(&TokenType::Comma, "`,`")?;
                }
            }
            self.expect(&TokenType::CloseParen, "`)`")?;
        }
        let (statements, _) = self.parse_render_block(Body::Test)?;
        let mut body = Vec::new();
        let mut steps = Vec::new();
        for stmt in statements {
            let StatementKind::UIElement(el) = &stmt.kind else {
                body.push(stmt);
                continue;
            };
            let ComponentRef::BuiltIn(marker) = &el.component else {
                body.push(stmt);
                continue;
            };
            // The steps are parsed as markers so they keep their place
            // among the elements: what a test does and what it then
            // expects only mean anything in the order they were written.
            let positional = |n: usize| {
                el.args
                    .iter()
                    .filter_map(|a| match a {
                        Arg::Positional(e) => Some(e.clone()),
                        Arg::Named(..) => None,
                    })
                    .nth(n)
            };
            let span = stmt.span;
            match marker.as_str() {
                "__Expect" => steps.push(Step::Expect {
                    text: positional(0).unwrap_or(Expr::StringLiteral(String::new())),
                    negated: el.modifiers.iter().any(|m| m == "not"),
                    span,
                }),
                "__Click" => steps.push(Step::Click {
                    target: positional(0).unwrap_or(Expr::StringLiteral(String::new())),
                    span,
                }),
                "__Type" => steps.push(Step::Type {
                    text: positional(0).unwrap_or(Expr::StringLiteral(String::new())),
                    into: positional(1).unwrap_or(Expr::StringLiteral(String::new())),
                    span,
                }),
                "__Press" => steps.push(Step::Press {
                    key: positional(0).unwrap_or(Expr::StringLiteral(String::new())),
                    target: positional(1),
                    span,
                }),
                _ => body.push(stmt),
            }
        }
        if !steps.iter().any(|s| matches!(s, Step::Expect { .. })) {
            return Err(self.error_with_hint(
                format!("test \"{name}\" expects nothing"),
                "End it with what the page must show: `expect \"text\"`, or `expect not \"text\"`",
            ));
        }
        Ok(Declaration::Test(TestDecl {
            name,
            data,
            body,
            steps,
            span: self.span_since(mark),
        }))
    }

    /// `const API = "/api"`, `const LIMIT: Number = 10`.
    fn parse_const_decl(&mut self, doc: Option<String>) -> Result<Declaration> {
        let mark = self.mark();
        self.expect_word("const")?;
        let name = self.expect_ident("the constant's name")?;
        let ty = if self.eat(&TokenType::Colon) {
            Some(self.parse_type_ref()?)
        } else {
            None
        };
        self.expect(&TokenType::Equals, "`=`")?;
        let value = self.parse_expression()?;
        Ok(Declaration::Const(ConstDecl {
            name,
            ty,
            value,
            doc,
            span: self.span_since(mark),
        }))
    }

    fn parse_enum_decl(&mut self, doc: Option<String>) -> Result<Declaration> {
        let mark = self.mark();
        self.expect_word("enum")?;
        let name = self.expect_ident("the enum's name")?;
        let header_span = self.span_since(mark);
        self.expect(&TokenType::OpenBrace, "`{`")?;
        let mut cases: Vec<EnumCase> = Vec::new();
        while !self.check(&TokenType::CloseBrace) && !self.at_end() {
            self.take_docs();
            self.eat(&TokenType::Dot);
            let case_name = self.expect_ident("a case name")?;
            // `failed(reason: String)`: the payload the case carries.
            let fields = if self.eat(&TokenType::OpenParen) {
                let fields = self.parse_field_list(&TokenType::CloseParen)?;
                self.expect(&TokenType::CloseParen, "`)`")?;
                if fields.is_empty() {
                    return Err(self.error_with_hint(
                        format!("`{case_name}()` carries nothing"),
                        "Name what the case carries, `failed(reason: String)`, or drop the parentheses",
                    ));
                }
                fields
            } else {
                Vec::new()
            };
            if cases.iter().any(|c| c.name == case_name) {
                return Err(self.error(format!("`enum {name}` has `{case_name}` twice")));
            }
            cases.push(EnumCase {
                name: case_name,
                fields,
            });
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
            self.push_render_statement(body, &mut statements)?;
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

    /// A component's body: `event`, `slot` and `part` declarations are
    /// lifted out.
    fn parse_component_body(&mut self, owner: &str) -> Result<ComponentBody> {
        let open = self.expect(&TokenType::OpenBrace, "`{`")?;
        let mut statements = Vec::new();
        let mut events = Vec::new();
        let mut slots = Vec::new();
        let mut parts: Vec<ComponentDecl> = Vec::new();
        while !self.check(&TokenType::CloseBrace) && !self.at_end() {
            let doc = self.take_docs();
            // `part Header(_ text: String) { … }`: a component of the
            // owner's, called `Owner.Header`.
            if self.is_word("part") && matches!(self.kind_at(1), TokenType::Identifier(_)) {
                if owner.contains('.') {
                    return Err(self.error_with_hint(
                        format!("`{owner}` is a part; a part declares no parts of its own"),
                        "Declare it on the owning component",
                    ));
                }
                let Declaration::Component(part) =
                    self.parse_component_like("part", Some(owner), doc)?
                else {
                    unreachable!("a part parses as a component")
                };
                if !part
                    .name
                    .rsplit('.')
                    .next()
                    .unwrap_or("")
                    .starts_with(char::is_uppercase)
                {
                    return Err(self.error_with_hint(
                        format!("`{}` is not a part name", part.name),
                        "A part's name is capitalised, like a component's: `part Header`",
                    ));
                }
                if parts.iter().any(|p| p.name == part.name) {
                    return Err(self.error(format!("`{}` is declared twice", part.name)));
                }
                parts.push(part);
                continue;
            }
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
                let slot_line = self.current().line;
                self.advance();
                // A slot's name is on the slot's own line: a bare `slot`
                // followed by `on key(…)` on the next names no slot `on`.
                let name = match self.kind() {
                    TokenType::Identifier(w)
                        if !STATEMENT_WORDS.contains(&w.as_str())
                            && self.current().line == slot_line =>
                    {
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
                // `slot row(item: Todo)`: a scoped slot hands its fill values.
                let params = if name.is_some() && self.check(&TokenType::OpenParen) {
                    self.parse_params()?
                } else {
                    Vec::new()
                };
                slots.push(SlotDecl {
                    name,
                    params,
                    span: self.span_since(mark),
                });
                continue;
            }
            self.push_render_statement(Body::Component, &mut statements)?;
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
            parts,
        })
    }

    /// One statement of a render block — or, where it says `sequence`, the
    /// several a sequence spells out.
    fn push_render_statement(&mut self, body: Body, out: &mut Vec<Statement>) -> Result<()> {
        if self.is_word("sequence") && matches!(self.kind_at(1), TokenType::OpenBrace) {
            out.extend(self.parse_sequence()?);
            return Ok(());
        }
        out.push(self.parse_render_statement(body)?);
        Ok(())
    }

    /// `sequence { step { … } step(after: "120ms") { … } }` — the elements
    /// of each step, with the sequence's clock written onto them as a
    /// `delay:`.
    ///
    /// It is orchestration and nothing else: the steps are the elements
    /// themselves, so everything downstream — the linters, the static
    /// paint, the template engine — sees the program it would have seen
    /// had the delays been written by hand. `after:` is measured from the
    /// step before it; a step without one starts when that step does.
    fn parse_sequence(&mut self) -> Result<Vec<Statement>> {
        self.advance();
        self.expect(&TokenType::OpenBrace, "`{`")?;
        let mut out = Vec::new();
        let mut clock = 0f64;
        while !self.check(&TokenType::CloseBrace) && !self.at_end() {
            if !self.is_word("step") {
                return Err(self.error_with_hint(
                    format!("A sequence holds steps, and this is {}", self.describe()),
                    "Write `sequence { step { … } step(after: \"120ms\") { … } }`",
                ));
            }
            let at = self.mark();
            self.advance();
            if self.check(&TokenType::OpenParen) {
                self.advance();
                if !self.is_word("after") {
                    return Err(self.error_with_hint(
                        format!("A step takes `after:`, and this is {}", self.describe()),
                        "`step(after: \"120ms\")` starts it that long after the step before",
                    ));
                }
                self.advance();
                self.expect(&TokenType::Colon, "`:`")?;
                let TokenType::StringLiteral(lit) = self.current().token_type.clone() else {
                    return Err(self.error_with_hint(
                        format!(
                            "`after:` is a length of time, and this is {}",
                            self.describe()
                        ),
                        "Write it as a string: `step(after: \"120ms\")`",
                    ));
                };
                let text = lit.spelling.clone();
                let Some(ms) = duration_ms(&text) else {
                    return Err(self.error_with_hint(
                        format!("`after: \"{text}\"` is not a length of time"),
                        "Write milliseconds or seconds: `\"120ms\"`, `\"0.4s\"`",
                    ));
                };
                self.advance();
                self.expect(&TokenType::CloseParen, "`)`")?;
                clock += ms;
            }
            let (stmts, _) = self.parse_render_block(Body::Nested)?;
            let span = self.span_since(at);
            for mut stmt in stmts {
                if clock > 0.0
                    && let StatementKind::UIElement(el) = &mut stmt.kind
                    && !el
                        .args
                        .iter()
                        .any(|a| matches!(a, Arg::Named(k, _) if k == "delay"))
                {
                    el.args.push(Arg::Named(
                        "delay".to_string(),
                        Expr::StringLiteral(format!("{}ms", clock.round() as i64)),
                    ));
                    el.arg_spans.push(span);
                }
                out.push(stmt);
            }
        }
        self.expect(&TokenType::CloseBrace, "`}`")?;
        Ok(out)
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
            // `head { meta(…) link(…) script(…) }`: the page's own head tags,
            // gathered by the page parser.
            "head" if body == Body::Page && matches!(self.kind_at(1), TokenType::OpenBrace) => {
                self.advance();
                self.expect(&TokenType::OpenBrace, "`{`")?;
                let mut tags = Vec::new();
                while !self.check(&TokenType::CloseBrace) && !self.at_end() {
                    let mark = self.mark();
                    let tag = self.expect_ident("a head tag: `meta`, `link` or `script`")?;
                    if !matches!(tag.as_str(), "meta" | "link" | "script") {
                        return Err(self.error_with_hint(
                            format!("`{tag}` is not a head tag"),
                            "A page's head holds `meta(…)`, `link(…)` and `script(…)`; its title and description are page attributes",
                        ));
                    }
                    let (args, _) = self.parse_call_args()?;
                    if args.iter().any(|a| matches!(a, Arg::Positional(_))) {
                        return Err(self.error_with_hint(
                            format!("`{tag}` takes named attributes"),
                            &format!("Write `{tag}(name: \"…\", content: \"…\")`"),
                        ));
                    }
                    let mut el = self.blank_element(ComponentRef::BuiltIn(tag));
                    el.args = args;
                    tags.push(Statement::new(
                        StatementKind::UIElement(el),
                        self.span_since(mark),
                    ));
                }
                self.expect(&TokenType::CloseBrace, "`}`")?;
                let mut holder = self.blank_element(ComponentRef::BuiltIn("__Head".to_string()));
                holder.children = tags;
                Ok(StatementKind::UIElement(holder))
            }
            // `expect "text"` / `expect not "text"` in a test.
            "expect" if body == Body::Test => {
                self.advance();
                let negated = self.eat_word("not");
                let text = self.parse_expression()?;
                let mut el = self.blank_element(ComponentRef::BuiltIn("__Expect".to_string()));
                el.args.push(Arg::Positional(text));
                if negated {
                    el.modifiers.push("not".to_string());
                }
                Ok(StatementKind::UIElement(el))
            }
            // `click "Save"` — whatever carries that name: a button, a
            // link, a control with that label. A test names what a reader
            // would name, because that is what the page promises.
            "click" if body == Body::Test => {
                self.advance();
                let target = self.parse_expression()?;
                let mut el = self.blank_element(ComponentRef::BuiltIn("__Click".to_string()));
                el.args.push(Arg::Positional(target));
                Ok(StatementKind::UIElement(el))
            }
            // `type "Ada" into "Name"`.
            "type" if body == Body::Test && self.is_string_at(1) => {
                self.advance();
                let text = self.parse_expression()?;
                self.expect_word("into")?;
                let into = self.parse_expression()?;
                let mut el = self.blank_element(ComponentRef::BuiltIn("__Type".to_string()));
                el.args.push(Arg::Positional(text));
                el.args.push(Arg::Positional(into));
                Ok(StatementKind::UIElement(el))
            }
            // `press "Enter"`, or `press "Escape" in "Search"`.
            "press" if body == Body::Test => {
                self.advance();
                let key = self.parse_expression()?;
                let mut el = self.blank_element(ComponentRef::BuiltIn("__Press".to_string()));
                el.args.push(Arg::Positional(key));
                if self.is_word("in") {
                    self.advance();
                    let target = self.parse_expression()?;
                    el.args.push(Arg::Positional(target));
                }
                Ok(StatementKind::UIElement(el))
            }
            "state" => self.parse_state(false),
            // `persist theme = "light"`: state kept across visits.
            "persist" if matches!(self.kind_at(1), TokenType::Identifier(_)) => {
                if !matches!(body, Body::Page | Body::Component | Body::Store) {
                    return Err(self.error_with_hint(
                        "`persist` is declared at the top of a page, a component or a store".into(),
                        "Move it beside the `state` declarations",
                    ));
                }
                self.parse_state(true)
            }
            "derived" => self.parse_derived(),
            "effect" => {
                self.advance();
                let outer = self.effect_body.replace(self.block_depth + 1);
                let parsed = self.parse_imperative_block();
                self.effect_body = outer;
                let (mut stmts, _) = parsed?;
                // `cleanup { … }` closes the effect's body.
                let mut cleanup = Vec::new();
                if let Some(last) = stmts.last()
                    && let StatementKind::ExprStatement(Expr::Identifier(w)) = &last.kind
                    && w == "__cleanup"
                {
                    stmts.pop();
                    cleanup = self.pending_cleanup.take().unwrap_or_default();
                }
                Ok(StatementKind::Effect(EffectDecl {
                    body: stmts,
                    cleanup,
                }))
            }
            // `every(1000) { tick() }`, `after(500) { hide() }`.
            "every" | "after" if matches!(self.kind_at(1), TokenType::OpenParen) => {
                let every = word == "every";
                self.advance();
                self.expect(&TokenType::OpenParen, "`(`")?;
                let interval = self.parse_expression()?;
                self.expect(&TokenType::CloseParen, "`)`")?;
                let (body, _) = self.parse_imperative_block()?;
                Ok(StatementKind::Timer(TimerStmt {
                    every,
                    interval,
                    body,
                }))
            }
            // `on key("ctrl+k") { … }` on the page itself: the document
            // listens.
            "on" if matches!(self.kind_at(1), TokenType::Identifier(w) if w == "key") => {
                Ok(StatementKind::EventHandler(self.parse_handler()?))
            }
            "action" => self.parse_action(),
            "use" => {
                self.advance();
                let store_name = self.expect_ident("the store's name")?;
                Ok(StatementKind::Use(UseDecl { store_name }))
            }
            "resource" => self.parse_resource(),
            // `validate email { required  email }`: what the value must be
            // for a form to accept it, said beside the state it guards.
            "validate" if matches!(self.kind_at(2), TokenType::OpenBrace) => self.parse_validate(),
            // A connection the page holds open, and the scope closes.
            "socket" if matches!(self.kind_at(2), TokenType::Equals) => {
                self.parse_connection(ConnectionKind::Socket)
            }
            "stream" if matches!(self.kind_at(2), TokenType::Equals) => {
                self.parse_connection(ConnectionKind::Stream)
            }
            "channel" if matches!(self.kind_at(2), TokenType::Equals) => {
                self.parse_connection(ConnectionKind::Channel)
            }
            "if" => self.parse_if(false),
            "for" => self.parse_for(false),
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
                // A slot use: a bare lowercase word in a component's body, or
                // one with named values for a scoped slot, `row(item: t)`.
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
                let scoped = matches!(next, TokenType::OpenParen)
                    && matches!(self.kind_at(2), TokenType::Identifier(_))
                    && matches!(self.kind_at(3), TokenType::Colon);
                if (bare || scoped) && matches!(body, Body::Component | Body::Nested) {
                    self.advance();
                    let mut el = self.slot_use(&word);
                    if scoped {
                        let (args, _) = self.parse_call_args()?;
                        for arg in args {
                            match arg {
                                Arg::Named(k, v) => el.args.push(Arg::Named(k, v)),
                                Arg::Positional(_) => {
                                    return Err(self.error_with_hint(
                                        format!("`{word}` hands its values by name"),
                                        &format!("Write `{word}(name: value)`"),
                                    ));
                                }
                            }
                        }
                    }
                    return Ok(StatementKind::UIElement(el));
                }
                Err(self.error_with_hint(
                    format!("`{word}` is not an element or a statement a render block can hold"),
                    "Code that does something goes in `on click { … }`, an `action` or an `effect`; an element's name is capitalised",
                ))
            }
        }
    }

    /// Whether a fill with parameters — `row(item) {` — starts at the
    /// current word: a parenthesised list of names and then a block.
    fn fill_with_params_ahead(&self) -> bool {
        if !matches!(self.kind_at(1), TokenType::OpenParen) {
            return false;
        }
        let mut i = 2;
        loop {
            match self.kind_at(i) {
                TokenType::Identifier(_) => i += 1,
                TokenType::Comma => i += 1,
                TokenType::CloseParen => {
                    return matches!(self.kind_at(i + 1), TokenType::OpenBrace);
                }
                _ => return false,
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
        // `row(item: t)` hands a scoped slot its values: not a call.
        let scoped = matches!(self.kind_at(1), TokenType::OpenParen)
            && matches!(self.kind_at(2), TokenType::Identifier(_))
            && matches!(self.kind_at(3), TokenType::Colon);
        let open = match self.kind_at(1) {
            TokenType::OpenParen
                if !self.is_capitalized()
                    && !scoped
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

    fn parse_state(&mut self, persist: bool) -> Result<StatementKind> {
        self.expect_word(if persist { "persist" } else { "state" })?;
        let name = self.expect_ident("the state's name")?;
        let ty = if self.eat(&TokenType::Colon) {
            Some(self.parse_type_ref()?)
        } else {
            None
        };
        self.expect(&TokenType::Equals, "`=` and an initial value")?;
        let value = self.parse_expression()?;
        // `persist items = [] { in: .session  version: 2  migrate 1 -> 2 { … } }`
        let policy = if persist && self.check(&TokenType::OpenBrace) {
            Some(self.parse_persist_policy()?)
        } else {
            None
        };
        Ok(StatementKind::State(StateDecl {
            name,
            ty,
            value,
            persist,
            policy,
        }))
    }

    /// `store Cart(scope: .route, eager: true)` — the header's options.
    fn parse_store_options(&mut self) -> Result<(StoreScope, bool)> {
        let mut scope = StoreScope::App;
        let mut eager = false;
        if !self.check(&TokenType::OpenParen) {
            return Ok((scope, eager));
        }
        self.advance();
        while !self.check(&TokenType::CloseParen) && !self.at_end() {
            let key = self.expect_ident("`scope` or `eager`")?;
            self.expect(&TokenType::Colon, "`:`")?;
            match key.as_str() {
                "scope" => {
                    self.expect(&TokenType::Dot, "a case: `.app`, `.session` or `.route`")?;
                    let case = self.expect_ident("a case")?;
                    scope = match case.as_str() {
                        "app" => StoreScope::App,
                        "session" => StoreScope::Session,
                        "route" => StoreScope::Route,
                        other => {
                            return Err(self.error_with_hint(
                                format!("A store has no scope `.{other}`"),
                                "It is `.app` (the default), `.session` or `.route`",
                            ));
                        }
                    };
                }
                "eager" => {
                    eager = match self.current().token_type.clone() {
                        TokenType::BoolLiteral(b) => b,
                        TokenType::Identifier(w) if w == "true" => true,
                        TokenType::Identifier(w) if w == "false" => false,
                        _ => {
                            return Err(self.error_with_hint(
                                format!(
                                    "`eager:` is true or false, and this is {}",
                                    self.describe()
                                ),
                                "`store X(eager: true)` builds it at boot instead of on first read",
                            ));
                        }
                    };
                    self.advance();
                }
                other => {
                    return Err(self.error_with_hint(
                        format!("A store takes no `{other}:`"),
                        "It takes `scope:` (`.app`, `.session`, `.route`) and `eager:`",
                    ));
                }
            }
            if !self.eat(&TokenType::Comma) {
                break;
            }
        }
        self.expect(&TokenType::CloseParen, "`)`")?;
        Ok((scope, eager))
    }

    /// The block a `persist` may carry: where the value is written, the
    /// version of its shape, whether other tabs are followed, and how a
    /// value an older build left is brought forward.
    fn parse_persist_policy(&mut self) -> Result<PersistPolicy> {
        let mark = self.mark();
        self.expect(&TokenType::OpenBrace, "`{`")?;
        let mut policy = PersistPolicy::default();
        while !self.check(&TokenType::CloseBrace) && !self.at_end() {
            let at = self.mark();
            if self.is_word("migrate") {
                self.advance();
                let from = self.expect_version("the version it is coming from")?;
                if !(self.eat(&TokenType::Minus) && self.eat(&TokenType::GreaterThan)) {
                    return Err(self.error_with_hint(
                        format!("Expected `->`, got {}", self.describe()),
                        "A step is written `migrate 1 -> 2 { … }`",
                    ));
                }
                let to = self.expect_version("the version it is going to")?;
                if to != from + 1 {
                    return Err(self.error_with_hint(
                        format!("`migrate {from} -> {to}` skips a version"),
                        "Each step moves one version on, so every value can be brought forward",
                    ));
                }
                self.expect(&TokenType::OpenBrace, "`{` and what the value becomes")?;
                let body = self.parse_expression()?;
                self.expect(&TokenType::CloseBrace, "`}`")?;
                policy.migrations.push(Migration {
                    from,
                    to,
                    body,
                    span: self.span_since(at),
                });
                continue;
            }
            let key = self.expect_ident("`in`, `version`, `sync` or `migrate`")?;
            self.expect(&TokenType::Colon, "`:`")?;
            match key.as_str() {
                "in" => {
                    self.expect(&TokenType::Dot, "`.local` or `.session`")?;
                    let case = self.expect_ident("a case")?;
                    if !matches!(case.as_str(), "local" | "session") {
                        return Err(self.error_with_hint(
                            format!("There is nowhere called `.{case}` to keep it"),
                            "It is `.local` (across visits) or `.session` (this tab only)",
                        ));
                    }
                    policy.storage = Some(case);
                }
                "version" => policy.version = Some(self.expect_version("a version")?),
                "sync" => {
                    policy.sync = Some(match self.current().token_type.clone() {
                        TokenType::BoolLiteral(b) => b,
                        TokenType::Identifier(w) if w == "true" => true,
                        TokenType::Identifier(w) if w == "false" => false,
                        _ => {
                            return Err(self.error_with_hint(
                                format!(
                                    "`sync:` is true or false, and this is {}",
                                    self.describe()
                                ),
                                "`sync: false` keeps the value this tab's own",
                            ));
                        }
                    });
                    self.advance();
                }
                other => {
                    return Err(self.error_with_hint(
                        format!("A `persist` block takes no `{other}:`"),
                        "It takes `in:`, `version:`, `sync:` and `migrate n -> n+1 { … }`",
                    ));
                }
            }
        }
        self.expect(&TokenType::CloseBrace, "`}`")?;
        policy.span = self.span_since(mark);
        Ok(policy)
    }

    /// A whole number, as a version is written.
    fn expect_version(&mut self, what: &str) -> Result<u32> {
        let TokenType::NumberLiteral(n) = self.current().token_type.clone() else {
            return Err(self.error_with_hint(
                format!("Expected {what}, got {}", self.describe()),
                "A version is a whole number: `version: 2`",
            ));
        };
        if n < 1.0 || n.fract() != 0.0 {
            return Err(self.error_with_hint(
                format!("`{n}` is not a version"),
                "A version is a whole number from 1 up",
            ));
        }
        self.advance();
        Ok(n as u32)
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
    /// `api Backend(base: "/api/v1") { … }` — one place a service is
    /// described, so every call site is typed, cached and cancellable.
    /// `external Chart from "chart.js" { fn Chart(…) -> Handle  type Handle { … } }`
    /// and `external element Stripe("stripe-pricing-table") { prop … event … }`.
    ///
    /// The compiler cannot read the other side, so the declaration **is**
    /// the contract: every call site is checked against what is written
    /// here, and nothing else about the module is assumed.
    fn parse_external(&mut self, doc: Option<String>) -> Result<Declaration> {
        let mark = self.mark();
        self.expect_word("external")?;
        let element = self.is_word("element");
        if element {
            self.advance();
        }
        let name = self.expect_ident("the name this project calls it by")?;
        let (kind, from) = if element {
            self.expect(&TokenType::OpenParen, "`(` and the element's tag name")?;
            let tag = self.expect_string("the custom element's tag name")?;
            if !tag.contains('-') {
                return Err(self.error_with_hint(
                    format!("`{tag}` is not a custom element's name"),
                    "A custom element's tag holds a hyphen: `stripe-pricing-table`",
                ));
            }
            self.expect(&TokenType::CloseParen, "`)`")?;
            (ExternalKind::Element, tag)
        } else {
            self.expect_word("from")?;
            (
                ExternalKind::Module,
                self.expect_string("the module it comes from")?,
            )
        };

        let mut decl = ExternalDecl {
            name,
            kind,
            from,
            integrity: None,
            functions: Vec::new(),
            types: Vec::new(),
            props: Vec::new(),
            events: Vec::new(),
            doc,
            span: Span::default(),
        };
        if self.eat(&TokenType::OpenBrace) {
            while !self.check(&TokenType::CloseBrace) && !self.at_end() {
                let doc = self.take_docs();
                let word = self.ident().map(str::to_string).unwrap_or_default();
                match word.as_str() {
                    "fn" => {
                        self.advance();
                        decl.functions.push(self.parse_external_fn(doc)?);
                    }
                    "type" => {
                        self.advance();
                        decl.types.push(self.parse_external_type()?);
                    }
                    "prop" => {
                        self.advance();
                        decl.props.push(self.parse_prop_decl(false)?);
                    }
                    "event" => {
                        let at = self.mark();
                        self.advance();
                        let name = self.expect_ident("the event's name")?;
                        let params = if self.check(&TokenType::OpenParen) {
                            self.parse_params()?
                        } else {
                            Vec::new()
                        };
                        decl.events.push(EventDecl {
                            name,
                            params,
                            doc,
                            span: self.span_since(at),
                        });
                    }
                    // `integrity: "sha384-…"` for a module from another origin.
                    "integrity" => {
                        self.advance();
                        self.expect(&TokenType::Colon, "`:`")?;
                        decl.integrity = Some(self.expect_string("the hash")?);
                    }
                    other => {
                        return Err(self.error_with_hint(
                            format!("`{other}` is not part of an `external`"),
                            "It holds `fn`, `type`, `prop`, `event` and `integrity:`",
                        ));
                    }
                }
            }
            self.expect(&TokenType::CloseBrace, "`}`")?;
        }
        decl.span = self.span_since(mark);
        Ok(Declaration::External(decl))
    }

    /// `fn Chart(canvas: Any, config: Map) -> ChartHandle`, and the same
    /// shape for a type's method.
    fn parse_external_fn(&mut self, doc: Option<String>) -> Result<ExternalFn> {
        let mark = self.mark();
        let name = self.expect_ident("the function's name")?;
        let mut params = Vec::new();
        if self.eat(&TokenType::OpenParen) {
            while !self.check(&TokenType::CloseParen) && !self.at_end() {
                params.push(self.parse_prop_decl(false)?);
                if !self.check(&TokenType::CloseParen) {
                    self.expect(&TokenType::Comma, "`,`")?;
                }
            }
            self.expect(&TokenType::CloseParen, "`)`")?;
        }
        let returns = if self.eat(&TokenType::Minus) {
            self.expect(&TokenType::GreaterThan, "`->` and what it gives back")?;
            Some(self.parse_type_ref()?)
        } else {
            None
        };
        Ok(ExternalFn {
            name,
            params,
            returns,
            doc,
            span: self.span_since(mark),
        })
    }

    /// `type ChartHandle { update(data: Map), destroy(), width: Number }`
    fn parse_external_type(&mut self) -> Result<ExternalType> {
        let mark = self.mark();
        let name = self.expect_ident("the type's name")?;
        let mut methods = Vec::new();
        let mut fields = Vec::new();
        self.expect(&TokenType::OpenBrace, "`{` and what it has")?;
        while !self.check(&TokenType::CloseBrace) && !self.at_end() {
            let doc = self.take_docs();
            // A method takes arguments; a field takes a type.
            if matches!(self.kind_at(1), TokenType::OpenParen) {
                methods.push(self.parse_external_fn(doc)?);
            } else {
                let at = self.mark();
                let field = self.expect_ident("a method or a field")?;
                self.expect(&TokenType::Colon, "`:` and the field's type")?;
                let ty = self.parse_type_ref()?;
                fields.push(FieldDecl {
                    name: field,
                    ty,
                    default: None,
                    doc,
                    span: self.span_since(at),
                });
            }
            self.eat(&TokenType::Comma);
        }
        self.expect(&TokenType::CloseBrace, "`}`")?;
        Ok(ExternalType {
            name,
            methods,
            fields,
            span: self.span_since(mark),
        })
    }

    fn parse_api(&mut self, doc: Option<String>) -> Result<Declaration> {
        let mark = self.mark();
        self.expect_word("api")?;
        let name = self.expect_ident("the service's name")?;
        // `api Backend from "openapi.json" (base: …)`: the surface is read
        // from a specification, and every endpoint comes from it.
        let from = if self.is_word("from") {
            self.advance();
            Some(self.expect_string("the specification's file")?)
        } else {
            None
        };
        let mut settings = Vec::new();
        if self.eat(&TokenType::OpenParen) {
            while !self.check(&TokenType::CloseParen) && !self.at_end() {
                let key = self.expect_ident("a setting (`base`, `timeout`, `retry`)")?;
                self.expect(&TokenType::Colon, "`:`")?;
                settings.push((key, self.parse_expression()?));
                if !self.check(&TokenType::CloseParen) {
                    self.expect(&TokenType::Comma, "`,`")?;
                }
            }
            self.expect(&TokenType::CloseParen, "`)`")?;
        }

        let mut headers = Vec::new();
        let mut hooks = Vec::new();
        let mut endpoints = Vec::new();
        if self.eat(&TokenType::OpenBrace) {
            while !self.check(&TokenType::CloseBrace) && !self.at_end() {
                let doc = self.take_docs();
                let word = self.ident().map(str::to_string).unwrap_or_default();
                match word.as_str() {
                    // `headers { Authorization: "Bearer {token}" }`.
                    "headers" => {
                        self.advance();
                        self.expect(&TokenType::OpenBrace, "`{` after `headers`")?;
                        while !self.check(&TokenType::CloseBrace) && !self.at_end() {
                            let key = self.header_name()?;
                            self.expect(&TokenType::Colon, "`:`")?;
                            headers.push((key, self.parse_expression()?));
                            self.eat(&TokenType::Comma);
                        }
                        self.expect(&TokenType::CloseBrace, "`}`")?;
                    }
                    // `on request(r) { … }`, `on response(r)`, `on error(e)`.
                    "on" => hooks.push(self.parse_handler()?),
                    m if is_http_method(m) => {
                        endpoints.push(self.parse_endpoint(doc)?);
                    }
                    // Anything else is a setting, written as `timeout: 10.seconds`.
                    _ if !word.is_empty() => {
                        let key = self.expect_ident("a setting")?;
                        self.expect(&TokenType::Colon, "`:`")?;
                        settings.push((key, self.parse_expression()?));
                        self.eat(&TokenType::Comma);
                    }
                    _ => {
                        return Err(self.error_with_hint(
                            format!("Expected a setting or an endpoint, got {}", self.describe()),
                            "An endpoint is `get users(page: Number) -> [User]`",
                        ));
                    }
                }
            }
            self.expect(&TokenType::CloseBrace, "`}` to close the service")?;
        }
        Ok(Declaration::Api(ApiDecl {
            name,
            settings,
            headers,
            hooks,
            endpoints,
            doc,
            span: self.span_since(mark),
            from,
        }))
    }

    /// `get users(page: Number = 1, q: String?) -> [User]`, with the path
    /// after `at` when it is not the name, and the failures it declares.
    fn parse_endpoint(&mut self, doc: Option<String>) -> Result<Endpoint> {
        let mark = self.mark();
        let method = self.expect_ident("an HTTP method")?.to_uppercase();
        let name = self.expect_ident("the endpoint's name")?;
        let mut params = Vec::new();
        if self.eat(&TokenType::OpenParen) {
            while !self.check(&TokenType::CloseParen) && !self.at_end() {
                params.push(self.parse_prop_decl(params.is_empty())?);
                if !self.check(&TokenType::CloseParen) {
                    self.expect(&TokenType::Comma, "`,`")?;
                }
            }
            self.expect(&TokenType::CloseParen, "`)`")?;
        }
        // `at "users/:id"` when the path is not the name.
        let path = if self.is_word("at") {
            self.advance();
            self.expect_string("the endpoint's path")?
        } else {
            name.clone()
        };
        let returns = if self.eat(&TokenType::Minus) && self.eat(&TokenType::GreaterThan) {
            Some(self.parse_type_ref()?)
        } else {
            None
        };
        let mut settings = Vec::new();
        let mut errors = Vec::new();
        loop {
            if self.is_word("errors") {
                self.advance();
                self.expect(&TokenType::OpenBrace, "`{` after `errors`")?;
                while !self.check(&TokenType::CloseBrace) && !self.at_end() {
                    let code = match self.kind().clone() {
                        TokenType::NumberLiteral(n) => {
                            self.advance();
                            n as u16
                        }
                        _ => return Err(self.error("Expected a status code".into())),
                    };
                    self.expect(&TokenType::Minus, "`->`")?;
                    self.expect(&TokenType::GreaterThan, "`->`")?;
                    errors.push((code, self.parse_type_ref()?));
                    self.eat(&TokenType::Comma);
                }
                self.expect(&TokenType::CloseBrace, "`}`")?;
                continue;
            }
            // `cache: .swr(60.seconds)`, `as: .blob`, `.progress`.
            if let TokenType::Identifier(word) = self.kind().clone()
                && matches!(self.kind_at(1), TokenType::Colon)
                && matches!(word.as_str(), "cache" | "as" | "errorAs" | "fileField")
            {
                self.advance();
                self.advance();
                settings.push((word, self.parse_expression()?));
                continue;
            }
            break;
        }
        Ok(Endpoint {
            method,
            name,
            path,
            params,
            returns,
            errors,
            settings,
            doc,
            span: self.span_since(mark),
        })
    }

    /// A header's name, which may be hyphenated: `X-Request-Id`.
    fn header_name(&mut self) -> Result<String> {
        if let TokenType::StringLiteral(s) = self.kind().clone() {
            self.advance();
            return Ok(string_text(&s));
        }
        let mut name = self.expect_ident("a header's name")?;
        while self.check(&TokenType::Minus) && matches!(self.kind_at(1), TokenType::Identifier(_)) {
            self.advance();
            name.push('-');
            name.push_str(&self.expect_ident("the rest of the header's name")?);
        }
        Ok(name)
    }

    /// `validate email { required  minLength(8) "Use at least 8" }`.
    fn parse_validate(&mut self) -> Result<StatementKind> {
        let mark = self.mark();
        self.expect_word("validate")?;
        let name = self.expect_ident("the state to validate")?;
        self.expect(&TokenType::OpenBrace, "`{` and the rules")?;
        let mut rules = Vec::new();
        while !self.check(&TokenType::CloseBrace) && !self.at_end() {
            let at = self.mark();
            let rule = self.expect_ident("a rule")?;
            let mut args = Vec::new();
            if self.eat(&TokenType::OpenParen) {
                args = self.parse_expr_list(&TokenType::CloseParen)?;
            }
            // The message, when the rule's own is not the one wanted.
            let message = match self.kind().clone() {
                TokenType::StringLiteral(text) => {
                    self.advance();
                    Some(self.string_expr(&text)?)
                }
                _ => None,
            };
            // `custom { expr }` and `async { await … }` say what to check.
            let body = if self.check(&TokenType::OpenBrace) {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect(&TokenType::CloseBrace, "`}`")?;
                Some(expr)
            } else {
                None
            };
            rules.push(Rule {
                name: rule,
                args,
                message,
                body,
                span: self.span_since(at),
            });
        }
        self.expect(&TokenType::CloseBrace, "`}` to close the rules")?;
        Ok(StatementKind::Validate(ValidateDecl {
            name,
            rules,
            span: self.span_since(mark),
        }))
    }

    /// `socket chat = ws("wss://…") { … }`, `stream t = sse("/events")`,
    /// `channel c = broadcast("cart")`.
    fn parse_connection(&mut self, kind: ConnectionKind) -> Result<StatementKind> {
        let word = match kind {
            ConnectionKind::Socket => "socket",
            ConnectionKind::Stream => "stream",
            ConnectionKind::Channel => "channel",
        };
        self.expect_word(word)?;
        let name = self.expect_ident("the connection's name")?;
        self.expect(&TokenType::Equals, "`=`")?;
        let opener = match kind {
            ConnectionKind::Socket => "ws",
            ConnectionKind::Stream => "sse",
            ConnectionKind::Channel => "broadcast",
        };
        if !self.is_word(opener) {
            return Err(self.error_with_hint(
                format!(
                    "A `{word}` opens with `{opener}(…)`, got {}",
                    self.describe()
                ),
                &format!("Write `{word} name = {opener}(\"…\")`"),
            ));
        }
        self.advance();
        self.expect(&TokenType::OpenParen, "`(`")?;
        let url = self.parse_expression()?;
        let mut options = Vec::new();
        while self.eat(&TokenType::Comma) {
            if self.check(&TokenType::CloseParen) {
                break;
            }
            let key = self.expect_ident("an option name")?;
            self.expect(&TokenType::Colon, "`:`")?;
            options.push((key, self.parse_expression()?));
        }
        self.expect(&TokenType::CloseParen, "`)`")?;

        // `{ send Outgoing  receive Incoming  on message(m) { … } }`.
        let mut sends = None;
        let mut receives = None;
        let mut handlers = Vec::new();
        if self.eat(&TokenType::OpenBrace) {
            while !self.check(&TokenType::CloseBrace) && !self.at_end() {
                let word = self.ident().map(str::to_string).unwrap_or_default();
                match word.as_str() {
                    "send" => {
                        self.advance();
                        sends = Some(self.parse_type_ref()?);
                    }
                    "receive" => {
                        self.advance();
                        receives = Some(self.parse_type_ref()?);
                    }
                    // `on message(m) { … }` — what arrives, handled here.
                    "on" => handlers.push(self.parse_handler()?),
                    _ if !word.is_empty() => {
                        let key = self.expect_ident("an option name")?;
                        self.expect(&TokenType::Colon, "`:`")?;
                        options.push((key, self.parse_expression()?));
                        self.eat(&TokenType::Comma);
                    }
                    _ => return Err(self.error("Expected an option".into())),
                }
            }
            self.expect(&TokenType::CloseBrace, "`}`")?;
        }
        Ok(StatementKind::Connection(ConnectionDecl {
            kind,
            name,
            url,
            options,
            sends,
            receives,
            handlers,
        }))
    }

    fn parse_resource(&mut self) -> Result<StatementKind> {
        self.expect_word("resource")?;
        let name = self.expect_ident("the resource's name")?;
        let ty = if self.eat(&TokenType::Colon) {
            Some(self.parse_type_ref()?)
        } else {
            None
        };
        self.expect(&TokenType::Equals, "`=`")?;
        // `resource rows = Backend.rows(page: n)`: an endpoint of a
        // declared service, which carries its own address and settings.
        if !self.is_word("fetch") {
            if self.is_capitalized() {
                let call = self.parse_expression()?;
                return Ok(StatementKind::Resource(ResourceDecl {
                    name,
                    ty,
                    url: call,
                    options: Vec::new(),
                }));
            }
            return Err(self.error_with_hint(
                format!(
                    "A resource is a `fetch(…)` or an endpoint, got {}",
                    self.describe()
                ),
                "Write `resource rows = fetch(\"/api/rows\")`, or `resource rows = Backend.rows()`",
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

    /// `for item[, index] in iterable [by key] { … }`. In an imperative block
    /// the body is statements and the loop runs once, in order.
    fn parse_for(&mut self, imperative: bool) -> Result<StatementKind> {
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
        let (stmts, _) = if imperative {
            self.parse_imperative_block()?
        } else {
            self.parse_render_block(Body::Nested)?
        };
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
            let (pattern, binding, bindings) = self.parse_arm_head()?;
            let (body, _) = self.parse_render_block(Body::Nested)?;
            arms.push(MatchArm {
                pattern,
                binding,
                bindings,
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

    /// `(a, b)` after a `.case` arm: the names its payload is bound to.
    fn parse_arm_bindings(&mut self) -> Result<Vec<String>> {
        let mut names = Vec::new();
        if self.eat(&TokenType::OpenParen) {
            loop {
                names.push(self.expect_ident("a name for the case's payload")?);
                if !self.eat(&TokenType::Comma) {
                    break;
                }
            }
            self.expect(&TokenType::CloseParen, "`)`")?;
        }
        Ok(names)
    }

    fn parse_arm_head(&mut self) -> Result<(ArmPattern, Option<String>, Vec<String>)> {
        if self.eat(&TokenType::Dot) {
            let case = self.expect_ident("a case name after `.`")?;
            return Ok((ArmPattern::Case(case), None, self.parse_arm_bindings()?));
        }
        let word = self.expect_ident(
            "a match arm: `loading`, `error(e)`, `ready(v)`, `.case`, a connection's state, or `else`",
        )?;
        let pattern = match word.as_str() {
            "loading" => ArmPattern::Loading,
            "error" => ArmPattern::Error,
            "ready" => ArmPattern::Ready,
            "else" => ArmPattern::Else,
            // The states a connection is in.
            "connecting" | "open" | "closed" => ArmPattern::State(word.clone()),
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
        Ok((pattern, binding, Vec::new()))
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
                    && (next_is_brace || self.fill_with_params_ahead())
                    && !STATEMENT_WORDS.contains(&word.as_str()) =>
                {
                    at(self, Stage::Fills)?;
                    let mark = self.mark();
                    self.advance();
                    // `row(item, index) { … }` names what a scoped slot hands over.
                    let mut params = Vec::new();
                    if self.eat(&TokenType::OpenParen) {
                        while !self.check(&TokenType::CloseParen) && !self.at_end() {
                            params.push(self.expect_ident("a name for the slot's value")?);
                            if !self.check(&TokenType::CloseParen) {
                                self.expect(&TokenType::Comma, "`,`")?;
                            }
                        }
                        self.expect(&TokenType::CloseParen, "`)`")?;
                    }
                    let (body, body_span) = self.parse_render_block(Body::Nested)?;
                    el.slot_fills.push(SlotFill {
                        name: word,
                        params,
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
                    // `row(item: t)` hands a scoped slot its values.
                    let scoped = matches!(self.kind_at(1), TokenType::OpenParen)
                        && matches!(self.kind_at(2), TokenType::Identifier(_))
                        && matches!(self.kind_at(3), TokenType::Colon);
                    let known = self.is_capitalized()
                        || STATEMENT_WORDS.contains(&word.as_str())
                        || bare
                        || scoped;
                    if word.is_empty() || !known {
                        return Err(self.error_with_hint(
                            format!("Loose code in an element's block: {}", self.describe()),
                            "What the element does goes in `on click { … }`; what it shows is an element",
                        ));
                    }
                    at(self, Stage::Children)?;
                    self.push_render_statement(Body::Nested, &mut el.children)?;
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
        let mut key = None;
        let mut param = None;
        if self.eat(&TokenType::OpenParen) {
            // `on key("ctrl+k")`, or `on key("Escape", e)`.
            if event == "key" {
                match self.kind().clone() {
                    TokenType::StringLiteral(spelling) => {
                        self.advance();
                        key = Some(string_text(&spelling));
                    }
                    _ => {
                        return Err(self.error_with_hint(
                            "`on key` names the key it answers to".into(),
                            "Write `on key(\"ctrl+k\") { … }`; `on keydown(e) { … }` handles every key",
                        ));
                    }
                }
                if self.eat(&TokenType::Comma) {
                    param = Some(self.expect_ident("the event parameter's name")?);
                }
            } else {
                param = Some(self.expect_ident("the event parameter's name")?);
            }
            self.expect(&TokenType::CloseParen, "`)`")?;
        }
        if event == "key" && key.is_none() {
            return Err(self.error_with_hint(
                "`on key` names the key it answers to".into(),
                "Write `on key(\"ctrl+k\") { … }`",
            ));
        }
        let (body, _) = self.parse_imperative_block()?;
        Ok(EventHandler {
            event,
            param,
            key,
            body,
            span: self.span_since(mark),
        })
    }

    // ─── Imperative blocks ───────────────────────────────

    fn parse_imperative_block(&mut self) -> Result<(Vec<Statement>, Span)> {
        let open = self.expect(&TokenType::OpenBrace, "`{`")?;
        self.block_depth += 1;
        let parsed = self.parse_imperative_statements();
        self.block_depth -= 1;
        let statements = parsed?;
        let close = self.expect(&TokenType::CloseBrace, "`}`")?;
        let span = Span::new(
            open.end as u32,
            close.offset as u32,
            open.line as u32,
            open.column as u32,
        );
        Ok((statements, span))
    }

    /// The statements of an imperative block, up to its `}`.
    fn parse_imperative_statements(&mut self) -> Result<Vec<Statement>> {
        let mut statements = Vec::new();
        while !self.check(&TokenType::CloseBrace) && !self.at_end() {
            if self.eat(&TokenType::Semicolon) {
                continue;
            }
            // `let { a, b } = m` and `let [x, y] = l` are several locals:
            // the value once, then one per name.
            if self.is_word("let")
                && matches!(
                    self.kind_at(1),
                    TokenType::OpenBrace | TokenType::OpenBracket
                )
            {
                statements.extend(self.parse_destructuring_let()?);
                continue;
            }
            statements.push(self.parse_imperative_statement()?);
        }
        Ok(statements)
    }

    /// `let { a, b } = m` → `let __d = m; let a = __d.a; let b = __d.b`;
    /// `let [x, y] = l` → `let x = __d[0]; let y = __d[1]`.
    fn parse_destructuring_let(&mut self) -> Result<Vec<Statement>> {
        let mark = self.mark();
        self.expect_word("let")?;
        let by_index = self.check(&TokenType::OpenBracket);
        self.advance();
        let close = if by_index {
            TokenType::CloseBracket
        } else {
            TokenType::CloseBrace
        };
        let mut names = Vec::new();
        while !self.check(&close) && !self.at_end() {
            names.push(self.expect_ident("a name to bind")?);
            if !self.check(&close) {
                self.expect(&TokenType::Comma, "`,`")?;
            }
        }
        self.expect(&close, "the closing bracket")?;
        self.expect(&TokenType::Equals, "`=`")?;
        let value = self.parse_expression()?;
        let span = self.span_since(mark);
        self.destructures += 1;
        // A name a page could not write, without a leading `_`, which the
        // emitters read as a plain (not a state) name.
        let temp = format!("d__{}", self.destructures);
        let mut out = vec![Statement::new(
            StatementKind::State(StateDecl {
                name: temp.clone(),
                ty: None,
                value,
                persist: false,
                policy: None,
            }),
            span,
        )];
        for (i, name) in names.into_iter().enumerate() {
            let read = if by_index {
                Expr::IndexAccess(
                    Box::new(Expr::Identifier(temp.clone())),
                    Box::new(Expr::NumberLiteral(i as f64)),
                )
            } else {
                Expr::PropertyAccess(Box::new(Expr::Identifier(temp.clone())), name.clone())
            };
            out.push(Statement::new(
                StatementKind::State(StateDecl {
                    name,
                    ty: None,
                    value: read,
                    persist: false,
                    policy: None,
                }),
                span,
            ));
        }
        Ok(out)
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
                Ok(StatementKind::State(StateDecl {
                    name,
                    ty,
                    value,
                    persist: false,
                    policy: None,
                }))
            }
            "if" => self.parse_if(true),
            "for" if matches!(self.kind_at(1), TokenType::Identifier(_)) => self.parse_for(true),
            // `cleanup { … }` closes an effect's body: kept aside for the
            // effect, and a marker left in its place.
            "cleanup" if matches!(self.kind_at(1), TokenType::OpenBrace) => {
                if self.effect_body != Some(self.block_depth) {
                    return Err(self.error_with_hint(
                        "`cleanup` belongs at the end of an `effect`".into(),
                        "Write `effect { … cleanup { … } }`",
                    ));
                }
                self.advance();
                let (body, _) = self.parse_imperative_block()?;
                if !self.check(&TokenType::CloseBrace) {
                    return Err(self.error_with_hint(
                        "`cleanup` closes an effect's body".into(),
                        "Nothing follows it inside the effect",
                    ));
                }
                self.pending_cleanup = Some(body);
                Ok(StatementKind::ExprStatement(Expr::Identifier(
                    "__cleanup".to_string(),
                )))
            }
            "try" if matches!(self.kind_at(1), TokenType::OpenBrace) => {
                self.advance();
                let (body, _) = self.parse_imperative_block()?;
                self.expect_word("catch")?;
                let param = match self.kind() {
                    TokenType::Identifier(name) if !self.check(&TokenType::OpenBrace) => {
                        let name = name.clone();
                        self.advance();
                        Some(name)
                    }
                    _ => None,
                };
                let (catch_body, _) = self.parse_imperative_block()?;
                Ok(StatementKind::Try(TryStmt {
                    body,
                    param,
                    catch_body,
                }))
            }
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
            // A keyword that renders — unless it is a name being assigned
            // or read: `on = !on`, `show.x`.
            "show" | "match" | "on" | "style"
                if matches!(
                    self.kind_at(1),
                    TokenType::Identifier(_) | TokenType::OpenBrace | TokenType::Dot
                ) && !(word == "on" && matches!(self.kind_at(1), TokenType::Dot)) =>
            {
                Err(self.error_with_hint(
                    format!("`{word}` renders; it cannot appear inside an action or a handler"),
                    "Keep what the page shows in the page's body and change state here",
                ))
            }
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
        let expr = sub.parse_expression().map_err(|e| {
            let t = self.current();
            let message = match &e {
                WebFluentError::ParseError(d) => d.message.clone(),
                other => other.to_string(),
            };
            WebFluentError::ParseError(Diagnostic::new(
                format!("In `{{{text}}}`: {message}"),
                &self.file,
                t.line,
                t.column,
            ))
        })?;
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
            // A duration or an easing may be a theme token: `$d-fast`,
            // `$ease-standard`, which are `var(--…)` in the CSS.
            let token = |word: &str| match word.strip_prefix('$') {
                Some(name) => format!("var(--{name})"),
                None => word.to_string(),
            };
            let mut words = raw.split_whitespace();
            let duration = match words.next() {
                Some("fast") => "150ms".to_string(),
                Some("normal") => "250ms".to_string(),
                Some("slow") => "350ms".to_string(),
                Some(d) => token(d),
                None => {
                    return Err(self.error(format!("`{property}` needs a duration, like `200ms`")));
                }
            };
            let easing = words.next().map(token);
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
        self.parse_range()
    }

    /// `a..b` and `a..=b`, the loosest binding: `1..n + 1` is `1..(n + 1)`.
    fn parse_range(&mut self) -> Result<Expr> {
        let start = self.parse_coalesce()?;
        if self.eat(&TokenType::DotDot) {
            let end = self.parse_coalesce()?;
            return Ok(Expr::Range(Box::new(start), Box::new(end), false));
        }
        if self.eat(&TokenType::DotDotEq) {
            let end = self.parse_coalesce()?;
            return Ok(Expr::Range(Box::new(start), Box::new(end), true));
        }
        Ok(start)
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
                // `3.days`, `90.minutes`, `250.ms` — a length of time, which
                // is the only thing a number's `.name` can mean.
                if let Expr::NumberLiteral(n) = expr
                    && let Some(ms) = duration_unit(&name)
                    && !self.check(&TokenType::OpenParen)
                {
                    expr = Expr::Typed(
                        "Duration".to_string(),
                        Box::new(Expr::NumberLiteral(n * ms)),
                    );
                    continue;
                }
                if self.eat(&TokenType::OpenParen) {
                    let args = self.parse_argument_exprs()?;
                    expr = Expr::MethodCall(Box::new(expr), name, args);
                } else {
                    expr = Expr::PropertyAccess(Box::new(expr), name);
                }
            } else if self.eat(&TokenType::OpenBracket) {
                let index = self.parse_expression()?;
                self.expect(&TokenType::CloseBracket, "`]`")?;
                expr = Expr::IndexAccess(Box::new(expr), Box::new(index));
            } else if self.eat(&TokenType::OptionalChain) {
                // `a?.b`, `a?.m()`, `a?.[i]`.
                if self.eat(&TokenType::OpenBracket) {
                    let index = self.parse_expression()?;
                    self.expect(&TokenType::CloseBracket, "`]`")?;
                    expr = Expr::OptionalIndex(Box::new(expr), Box::new(index));
                } else {
                    let name = self.expect_ident("a property or method name after `?.`")?;
                    if self.eat(&TokenType::OpenParen) {
                        let args = self.parse_argument_exprs()?;
                        expr = Expr::OptionalMethod(Box::new(expr), name, args);
                    } else {
                        expr = Expr::OptionalProperty(Box::new(expr), name);
                    }
                }
            } else {
                break;
            }
        }
        Ok(expr)
    }

    /// A call's arguments: expressions, and `name: value` pairs, which are
    /// gathered into one map and passed last.
    ///
    /// `due.plus(days: 3, months: 1)` reads as it means, and arrives as the
    /// one map the runtime takes.
    fn parse_argument_exprs(&mut self) -> Result<Vec<Expr>> {
        let mut args = Vec::new();
        let mut named: Vec<(String, Expr)> = Vec::new();
        while !self.check(&TokenType::CloseParen) && !self.at_end() {
            if let TokenType::Identifier(name) = self.kind().clone()
                && matches!(self.kind_at(1), TokenType::Colon)
            {
                self.advance();
                self.advance();
                named.push((name, self.parse_expression()?));
            } else if self.eat(&TokenType::Ellipsis) {
                args.push(Expr::Spread(Box::new(self.parse_expression()?)));
            } else {
                args.push(self.parse_expression()?);
            }
            if !self.check(&TokenType::CloseParen) {
                self.expect(&TokenType::Comma, "`,`")?;
            }
        }
        self.expect(&TokenType::CloseParen, "`)`")?;
        if !named.is_empty() {
            args.push(Expr::MapLiteral(named));
        }
        Ok(args)
    }

    /// Comma-separated expressions up to `close`, which is consumed.
    fn parse_expr_list(&mut self, close: &TokenType) -> Result<Vec<Expr>> {
        let mut items = Vec::new();
        while !self.check(close) && !self.at_end() {
            // `...items` in a list: the items, in place.
            if self.eat(&TokenType::Ellipsis) {
                items.push(Expr::Spread(Box::new(self.parse_expression()?)));
            } else {
                items.push(self.parse_expression()?);
            }
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
                self.string_expr(&s)
            }
            TokenType::NumberLiteral(n) => {
                self.advance();
                Ok(Expr::NumberLiteral(n))
            }
            TokenType::TemporalLiteral(text) => {
                self.advance();
                let Some(kind) = temporal_kind(&text) else {
                    return Err(self.error_with_hint(
                        format!("`@{text}` is not a date, a time or both"),
                        "Write `@2026-03-14`, `@09:30` or `@2026-03-14T09:30Z`",
                    ));
                };
                Ok(Expr::Typed(
                    kind.to_string(),
                    Box::new(Expr::StringLiteral(text)),
                ))
            }
            TokenType::ColorLiteral(digits) => {
                self.advance();
                Ok(Expr::Typed(
                    "Color".to_string(),
                    Box::new(Expr::StringLiteral(format!("#{digits}"))),
                ))
            }
            TokenType::MoneyLiteral(symbol, amount) => {
                self.advance();
                let currency = match symbol.as_str() {
                    "€" => "EUR",
                    "£" => "GBP",
                    "¥" => "JPY",
                    _ => "USD",
                };
                // Minor units, so the arithmetic is a whole number's.
                let minor = (amount.parse::<f64>().unwrap_or(0.0) * 100.0).round();
                Ok(Expr::Typed(
                    "Money".to_string(),
                    Box::new(Expr::MapLiteral(vec![
                        ("amount".to_string(), Expr::NumberLiteral(minor)),
                        (
                            "currency".to_string(),
                            Expr::StringLiteral(currency.to_string()),
                        ),
                    ])),
                ))
            }
            TokenType::RegexLiteral(pattern, flags) => {
                self.advance();
                Ok(Expr::Regex(pattern, flags))
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
                // `.failed("x")`, `.backoff(times: 3)`: the case with its
                // payload, named or not.
                if self.eat(&TokenType::OpenParen) {
                    let args = self.parse_argument_exprs()?;
                    return Ok(Expr::CaseValue(case, args));
                }
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
                    // `(`, the names with their commas, `)`.
                    let tokens = if params.is_empty() {
                        2
                    } else {
                        params.len() * 2 + 1
                    };
                    for _ in 0..tokens {
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
            return Ok(Expr::Record(name, fields));
        }
        let args = self.parse_argument_exprs()?;
        Ok(Expr::FunctionCall(name, args))
    }

    /// `(a, b) =>` at the cursor: the parameter names.
    fn lambda_params_ahead(&self) -> Option<Vec<String>> {
        let mut params = Vec::new();
        let mut i = 1;
        // `() => …`: a lambda of no parameters.
        if matches!(self.kind_at(1), TokenType::CloseParen) {
            return matches!(self.kind_at(2), TokenType::Arrow).then_some(params);
        }
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

    /// `if c { a } else { b }`, or `if let x = e { a } else { b }`, which
    /// binds `x` to `e` in `a` when `e` is not null. The binding form is
    /// encoded as `e.__iflet(x => a, b)`, so every reader of the tree sees a
    /// lambda whose parameter is the name.
    fn parse_if_expression(&mut self) -> Result<Expr> {
        self.expect_word("if")?;
        if self.eat_word("let") {
            let name = self.expect_ident("the name to bind")?;
            self.expect(&TokenType::Equals, "`=`")?;
            let value = self.parse_expression()?;
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
            return Ok(Expr::MethodCall(
                Box::new(value),
                "__iflet".to_string(),
                vec![Expr::Lambda(name, Box::new(then_expr)), else_expr],
            ));
        }
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
        // An arm: its case and the name it binds, or neither for `else`.
        type Arm = (Option<(String, Option<String>)>, Expr);
        let mut arms: Vec<Arm> = Vec::new();
        while !self.check(&TokenType::CloseBrace) && !self.at_end() {
            let case = if self.eat(&TokenType::Dot) {
                let case = self.expect_ident("a case name")?;
                let bindings = self.parse_arm_bindings()?;
                if bindings.len() > 1 {
                    return Err(self.error_with_hint(
                        format!("`.{case}` binds one name in a match expression"),
                        "A match statement binds every part of a payload: `.case(a, b) { … }`",
                    ));
                }
                Some((case, bindings.into_iter().next()))
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
            let Some((case, binding)) = case else {
                continue;
            };
            expr = match binding {
                // `.failed(r) { … }`: the payload is bound where the case
                // matches, as an `if let` over it.
                Some(name) => Expr::MethodCall(
                    Box::new(Expr::MethodCall(
                        Box::new(subject.clone()),
                        "__payload".to_string(),
                        vec![Expr::StringLiteral(case)],
                    )),
                    "__iflet".to_string(),
                    vec![Expr::Lambda(name, Box::new(value)), expr],
                ),
                None => {
                    let cond = Expr::MethodCall(
                        Box::new(subject.clone()),
                        "__is".to_string(),
                        vec![Expr::StringLiteral(case)],
                    );
                    Expr::MethodCall(Box::new(cond), "__if".to_string(), vec![value, expr])
                }
            };
        }
        Ok(expr)
    }

    fn parse_map_literal(&mut self) -> Result<Expr> {
        self.expect(&TokenType::OpenBrace, "`{`")?;
        let mut pairs = Vec::new();
        while !self.check(&TokenType::CloseBrace) && !self.at_end() {
            // `...other`: the entries of another map, in place.
            if self.eat(&TokenType::Ellipsis) {
                let value = self.parse_expression()?;
                pairs.push(("...".to_string(), value));
                if !self.check(&TokenType::CloseBrace) {
                    self.expect(&TokenType::Comma, "`,`")?;
                }
                continue;
            }
            let key = match self.kind().clone() {
                TokenType::Identifier(w) => {
                    self.advance();
                    w
                }
                TokenType::StringLiteral(s) => {
                    self.advance();
                    format!("\"{}\"", string_text(&s))
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

    /// A string literal as an expression: its escapes resolved, the
    /// indentation of a block form gone, and each `{…}` splice parsed.
    ///
    /// This is the only place a literal becomes text, so an escaped brace
    /// is a brace from the moment it is understood and nothing after here
    /// has to know it was written `\{`.
    fn string_expr(&self, lit: &StringLit) -> Result<Expr> {
        let mut parts: Vec<StringPart> = Vec::new();
        let mut literal = String::new();
        for piece in pieces(lit) {
            match piece {
                Piece::Text(text) => literal.push_str(&text),
                Piece::Splice(source) => match self.splice_expr(&source)? {
                    Some(expr) => {
                        if !literal.is_empty() {
                            parts.push(StringPart::Literal(std::mem::take(&mut literal)));
                        }
                        parts.push(StringPart::Expression(expr));
                    }
                    // A brace group that is prose, not a splice.
                    None => {
                        literal.push('{');
                        literal.push_str(&source);
                        literal.push('}');
                    }
                },
            }
        }
        if parts.is_empty() {
            return Ok(Expr::StringLiteral(literal));
        }
        if !literal.is_empty() {
            parts.push(StringPart::Literal(literal));
        }
        Ok(Expr::InterpolatedString(parts))
    }

    /// One splice's source as an expression, `None` when the group is
    /// prose — `{name: value}` written in text — rather than a slip.
    ///
    /// `{total:.currency}` is the formatted form: the value, then how to
    /// show it, which is `format(total, .currency)` written where it is
    /// read.
    fn splice_expr(&self, source: &str) -> Result<Option<Expr>> {
        if let Some((value, spec)) = split_format_spec(source) {
            let value = self.parse_sub_expression(value)?;
            let (style, option) = match spec.split_once('(') {
                Some((style, rest)) => (style, Some(rest.trim_end_matches(')').trim())),
                None => (spec, None),
            };
            let mut args = vec![value, Expr::EnumCase(style.trim().to_string())];
            if let Some(option) = option.filter(|o| !o.is_empty()) {
                args.push(match option.parse::<f64>() {
                    Ok(n) => Expr::NumberLiteral(n),
                    Err(_) => Expr::StringLiteral(option.trim_matches('"').to_string()),
                });
            }
            return Ok(Some(Expr::FunctionCall("format".to_string(), args)));
        }
        match self.parse_sub_expression(source) {
            Ok(expr) => Ok(Some(expr)),
            // A brace group with a `:` or a `,` that is not an expression —
            // `{name: value}` in prose, a fragment of code — is text; one
            // without is a splice with a slip in it, and says so.
            Err(_) if source.contains([':', ',']) => Ok(None),
            Err(e) => Err(e),
        }
    }
}

/// Whether a word names an HTTP method.
fn is_http_method(word: &str) -> bool {
    matches!(
        word,
        "get" | "post" | "put" | "patch" | "delete" | "head" | "options"
    )
}

/// The milliseconds one of the units a number may carry is worth.
fn duration_unit(name: &str) -> Option<f64> {
    Some(match name {
        "ms" => 1.0,
        "second" | "seconds" => 1_000.0,
        "minute" | "minutes" => 60_000.0,
        "hour" | "hours" => 3_600_000.0,
        "day" | "days" => 86_400_000.0,
        "week" | "weeks" => 604_800_000.0,
        _ => return None,
    })
}

/// Which of the three a `@…` literal is, read from its shape.
fn temporal_kind(text: &str) -> Option<&'static str> {
    let date = |s: &str| {
        let parts: Vec<&str> = s.split('-').collect();
        parts.len() == 3
            && parts[0].len() == 4
            && parts[1].len() == 2
            && parts[2].len() == 2
            && parts.iter().all(|p| p.chars().all(|c| c.is_ascii_digit()))
    };
    let time = |s: &str| {
        let s = s.trim_end_matches('Z');
        let s = s.split(['+']).next().unwrap_or(s);
        let parts: Vec<&str> = s.split(':').collect();
        (2..=3).contains(&parts.len())
            && parts[0].len() == 2
            && parts[1].len() == 2
            && parts
                .iter()
                .all(|p| p.chars().all(|c| c.is_ascii_digit() || c == '.'))
    };
    match text.split_once('T') {
        Some((d, t)) if date(d) && time(t) => Some("DateTime"),
        None if date(text) => Some("Date"),
        None if time(text) => Some("Time"),
        _ => None,
    }
}

/// One piece of a string literal: text as it will be shown, or the source
/// of a `{…}` splice.
enum Piece {
    Text(String),
    Splice(String),
}

/// A literal's pieces: what the text says, and where the splices are.
///
/// A raw literal is one piece of text, whatever is in it. A block literal
/// loses the indentation the source gave it first, and is then read like
/// any other: escapes resolved, splices split out.
fn pieces(lit: &StringLit) -> Vec<Piece> {
    match lit.kind {
        StringKind::Raw => vec![Piece::Text(lit.spelling.clone())],
        StringKind::Block => split_pieces(&dedent(&lit.spelling)),
        StringKind::Plain => split_pieces(&lit.spelling),
    }
}

/// The escapes resolved and the splices taken out, in one pass.
fn split_pieces(spelling: &str) -> Vec<Piece> {
    let chars: Vec<char> = spelling.chars().collect();
    let mut out: Vec<Piece> = Vec::new();
    let mut text = String::new();
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            '\\' if i + 1 < chars.len() => {
                match chars[i + 1] {
                    'n' => text.push('\n'),
                    't' => text.push('\t'),
                    'r' => text.push('\r'),
                    '\\' => text.push('\\'),
                    '"' => text.push('"'),
                    '{' => text.push('{'),
                    '}' => text.push('}'),
                    c => {
                        text.push('\\');
                        text.push(c);
                    }
                }
                i += 2;
            }
            '{' => match splice_end(&chars, i) {
                Some(end) => {
                    if !text.is_empty() {
                        out.push(Piece::Text(std::mem::take(&mut text)));
                    }
                    out.push(Piece::Splice(splice_source(&chars[i + 1..end - 1])));
                    i = end;
                }
                None => {
                    text.push('{');
                    i += 1;
                }
            },
            c => {
                text.push(c);
                i += 1;
            }
        }
    }
    if !text.is_empty() {
        out.push(Piece::Text(text));
    }
    out
}

/// A splice's source is code, not text, so the escapes a string needs are
/// undone: `{a ?? \"x\"}` is the older way to write `{a ?? "x"}`, and both
/// mean the same expression.
///
/// Only outside a string the splice writes with plain quotes, though. Inside
/// one, an escape is that string's own — `{("a\"b").length}` — and undoing it
/// ended the inner string early, so the build refused an expression the
/// language allows.
fn splice_source(chars: &[char]) -> String {
    let mut out = String::with_capacity(chars.len());
    let mut in_string = false;
    let mut i = 0;
    while i < chars.len() {
        match (chars[i], chars.get(i + 1)) {
            ('\\', Some(&next)) if in_string => {
                out.push('\\');
                out.push(next);
                i += 2;
            }
            ('"', _) => {
                in_string = !in_string;
                out.push('"');
                i += 1;
            }
            ('\\', Some('"')) => {
                out.push('"');
                i += 2;
            }
            ('\\', Some('\\')) => {
                out.push('\\');
                i += 2;
            }
            (c, _) => {
                out.push(c);
                i += 1;
            }
        }
    }
    out
}

/// The index just past the `}` closing the splice that opens at `at`: a
/// `{` followed by a name, a `[` or a `(`, balanced and closed on the
/// same line, with any string inside it kept whole. A lone `{` is a
/// character like any other.
///
/// A string in a splice is written `"…"`, or — the older spelling —
/// `\"…\"`, whose escaped quotes delimit it. Only the first was recognised,
/// so the second never closed and `{a ?? \"x\"}` was shown as text. The
/// lexer and the parser each had a copy of this; there is one now, which
/// both call.
pub(crate) fn splice_end(chars: &[char], at: usize) -> Option<usize> {
    let opener = *chars.get(at + 1)?;
    if !(opener.is_alphabetic() || matches!(opener, '_' | '[' | '(')) {
        return None;
    }
    let mut depth = 0usize;
    let mut in_string = false;
    let mut in_escaped_string = false;
    let mut i = at;
    while i < chars.len() {
        let c = chars[i];
        let escaped_quote = c == '\\' && chars.get(i + 1) == Some(&'"');
        if in_string {
            match c {
                '\\' => i += 1,
                '"' => in_string = false,
                '\n' => return None,
                _ => {}
            }
        } else if in_escaped_string {
            if escaped_quote {
                in_escaped_string = false;
                i += 1;
            } else if c == '\n' {
                return None;
            }
        } else if escaped_quote {
            in_escaped_string = true;
            i += 1;
        } else {
            match c {
                '"' => in_string = true,
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(i + 1);
                    }
                }
                '\n' => return None,
                _ => {}
            }
        }
        i += 1;
    }
    None
}

/// A splice written `value:.style` or `value:.style(option)`, split into
/// the two. The `:` is the last one outside brackets and strings, so
/// `{a ?? "x:y":.currency}` reads the way it looks.
fn split_format_spec(source: &str) -> Option<(&str, &str)> {
    let chars: Vec<char> = source.chars().collect();
    let mut depth = 0usize;
    let mut in_string = false;
    let mut found = None;
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if in_string {
            match c {
                '\\' => i += 1,
                '"' => in_string = false,
                _ => {}
            }
        } else {
            match c {
                '"' => in_string = true,
                '(' | '[' | '{' => depth += 1,
                ')' | ']' | '}' => depth = depth.saturating_sub(1),
                ':' if depth == 0 => found = Some(i),
                _ => {}
            }
        }
        i += 1;
    }
    let at = found?;
    let (value, rest) = source.split_at(byte_of(&chars, at));
    let spec = rest.strip_prefix(':')?.trim();
    // Only `.style` is a format; anything else is a `:` in prose.
    let style = spec.strip_prefix('.')?;
    let head = style.split('(').next().unwrap_or(style).trim();
    if head.is_empty() || !head.chars().all(|c| c.is_ascii_alphabetic()) {
        return None;
    }
    (!value.trim().is_empty()).then_some((value.trim(), style))
}

/// The byte offset of the `n`th character.
fn byte_of(chars: &[char], n: usize) -> usize {
    chars[..n].iter().map(|c| c.len_utf8()).sum()
}

/// A block literal without the indentation the source gave it.
///
/// The line the opening delimiter sat on goes, and so does the
/// whitespace the closing delimiter stands in — which is what says how
/// far the text was indented.
fn dedent(spelling: &str) -> String {
    let mut lines: Vec<&str> = spelling.split('\n').collect();
    if lines.first().is_some_and(|l| l.trim().is_empty()) {
        lines.remove(0);
    }
    let indent: String = match lines.last() {
        Some(last) if last.trim().is_empty() && lines.len() > 1 => {
            let indent = last.to_string();
            lines.pop();
            indent
        }
        _ => lines
            .iter()
            .filter(|l| !l.trim().is_empty())
            .map(|l| &l[..l.len() - l.trim_start().len()])
            .min_by_key(|w| w.len())
            .unwrap_or("")
            .to_string(),
    };
    lines
        .iter()
        .map(|l| l.strip_prefix(indent.as_str()).unwrap_or(l.trim_start()))
        .collect::<Vec<_>>()
        .join("\n")
}

/// The text of a string literal that is not an expression — a route, a
/// title, a file's name, a key. A splice in one of those is text.
fn string_text(lit: &StringLit) -> String {
    pieces(lit)
        .into_iter()
        .map(|p| match p {
            Piece::Text(t) => t,
            Piece::Splice(s) => format!("{{{s}}}"),
        })
        .collect()
}

/// `"120ms"`, `"0.4s"` — milliseconds, or nothing where it is neither.
fn duration_ms(text: &str) -> Option<f64> {
    let text = text.trim();
    let (number, scale) = match text.strip_suffix("ms") {
        Some(n) => (n, 1.0),
        None => (text.strip_suffix('s')?, 1000.0),
    };
    let n: f64 = number.trim().parse().ok()?;
    (n >= 0.0).then_some(n * scale)
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
    fn a_transition_takes_tokens_for_its_timing() {
        let body = page_body(
            "page P(path: \"/\") { Card { transition { background: $d-fast $ease-standard\n opacity: 200ms ease } } }",
        );
        let StatementKind::UIElement(el) = &body[0].kind else {
            panic!()
        };
        let t = el.transition_block.as_ref().unwrap();
        assert_eq!(t.properties[0].duration, "var(--d-fast)");
        assert_eq!(
            t.properties[0].easing.as_deref(),
            Some("var(--ease-standard)")
        );
        assert_eq!(t.properties[1].duration, "200ms");
        assert_eq!(t.properties[1].easing.as_deref(), Some("ease"));
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
            matches!(&body[0].kind, StatementKind::State(s) if matches!(&s.value, Expr::Record(n, f) if n == "Todo" && f.len() == 2 && f[0].0 == "id"))
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
    fn a_bare_slot_before_a_page_level_handler_names_no_slot() {
        let p =
            parse("component S {\n    slot\n    on key(\"Escape\") { log(1) }\n    children\n}");
        let Declaration::Component(c) = &p.declarations[0] else {
            panic!()
        };
        assert_eq!(c.slots.len(), 1);
        assert_eq!(c.slots[0].name, None);
        assert!(c.body.iter().any(|s| matches!(&s.kind, StatementKind::EventHandler(h) if h.key.as_deref() == Some("Escape"))), "{:?}", c.body);
    }

    #[test]
    fn a_lambda_may_take_no_parameters() {
        let p = parse(
            "page P(path: \"/\") {\n state n = 0\n action bump() { n = n + 1 }\n effect { setTimeout(() => bump(), 10) }\n Text(\"{n}\")\n}",
        );
        let Declaration::Page(page) = &p.declarations[0] else {
            panic!()
        };
        let StatementKind::Effect(e) = &page.body[2].kind else {
            panic!("{:?}", page.body[2])
        };
        assert!(
            format!("{:?}", e.body).contains("Lambda(\"\""),
            "{:?}",
            e.body
        );
    }

    #[test]
    fn a_splice_may_hold_a_call_with_arguments_and_a_list() {
        let el = first_element(
            "page P(path: \"/\") { Text(\"Total {format(total, .currency)} of {[1, 2].length}\") }",
        );
        let Some(Arg::Positional(Expr::InterpolatedString(parts))) = el.args.first() else {
            panic!("{:?}", el.args)
        };
        assert!(
            matches!(&parts[1], StringPart::Expression(Expr::FunctionCall(n, a)) if n == "format" && a.len() == 2)
        );
        assert!(matches!(
            &parts[3],
            StringPart::Expression(Expr::PropertyAccess(..))
        ));
    }

    #[test]
    fn a_brace_group_that_is_not_an_expression_stays_text_only_when_it_could_be_prose() {
        // `{name: value}` in prose, a code fragment: text.
        let el =
            first_element("page P(path: \"/\") { Text(\"Write {key: value} pairs, {a, b}\") }");
        assert!(
            matches!(el.args.first(), Some(Arg::Positional(Expr::StringLiteral(s))) if s == "Write {key: value} pairs, {a, b}"),
            "{:?}",
            el.args
        );
        // A splice with a slip in it is an error, not silently text.
        let err = fails("page P(path: \"/\") { Text(\"Hi {name +}\") }");
        assert!(err.contains("name +"), "{err}");
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
