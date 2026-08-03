use crate::lexer::{Token, TokenType};
use crate::error::{Diagnostic, WebFluentError, Result};
use super::ast::*;

/// The parsed `( … )` argument + modifier group of an element, with spans.
/// Produced by [`Parser::parse_paren_args`].
struct ParenArgs {
    args: Vec<Arg>,
    modifiers: Vec<String>,
    arg_spans: Vec<Span>,
    modifier_spans: Vec<Span>,
    /// Span of the whole `( … )` group, parentheses included.
    paren_span: Span,
}

/// The parsed optional `{ … }` body of an element, with spans. Produced by
/// [`Parser::parse_element_body`].
struct ElementBody {
    children: Vec<Statement>,
    events: Vec<EventHandler>,
    style_block: Option<StyleBlock>,
    transition_block: Option<TransitionBlock>,
    /// Interior of the `{ … }` (braces excluded); `None` if there was no block.
    body_span: Option<Span>,
    /// Span of the `style { … }` block; `None` if there was none.
    style_span: Option<Span>,
}

/// The WebFluent parser — converts a token stream into an AST.
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    file: String,
}

impl Parser {
    /// Create a new parser from a token stream.
    ///
    /// `file` is used for error reporting.
    pub fn new(tokens: Vec<Token>, file: &str) -> Self {
        Self {
            tokens,
            pos: 0,
            file: file.to_string(),
        }
    }

    /// Parse the token stream into a [`Program`] AST.
    ///
    /// Returns an error if the token stream contains syntax errors.
    pub fn parse(&mut self) -> Result<Program> {
        let mut declarations = Vec::new();
        while !self.is_at_end() {
            declarations.push(self.parse_declaration()?);
        }
        Ok(Program { declarations })
    }

    // ─── Helpers ─────────────────────────────────────────

    fn current(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn current_type(&self) -> &TokenType {
        &self.tokens[self.pos].token_type
    }

    fn is_at_end(&self) -> bool {
        matches!(self.current_type(), TokenType::EOF)
    }

    fn advance(&mut self) -> &Token {
        let token = &self.tokens[self.pos];
        if !self.is_at_end() {
            self.pos += 1;
        }
        token
    }

    fn expect(&mut self, expected: &TokenType) -> Result<&Token> {
        if std::mem::discriminant(self.current_type()) == std::mem::discriminant(expected) {
            Ok(self.advance())
        } else {
            Err(self.error(format!("Expected {}, got {}", expected, self.current_type())))
        }
    }

    fn check(&self, token_type: &TokenType) -> bool {
        std::mem::discriminant(self.current_type()) == std::mem::discriminant(token_type)
    }

    fn match_token(&mut self, token_type: &TokenType) -> bool {
        if self.check(token_type) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn error(&self, message: String) -> WebFluentError {
        let token = self.current();
        WebFluentError::ParseError(
            Diagnostic::new(message, &self.file, token.line, token.column)
        )
    }

    // ─── Span helpers ────────────────────────────────────
    //
    // A node's span runs from the start of the first token that belongs to it
    // to the end of the last token consumed for it. Callers `mark()` the current
    // token before parsing and `span_since(mark)` once the node is complete.

    /// Byte offset + 1-based line/column of the current token's start.
    fn mark(&self) -> (u32, u32, u32) {
        let t = self.current();
        (t.offset as u32, t.line as u32, t.column as u32)
    }

    /// End byte offset (exclusive) of the most recently consumed token; 0 before
    /// any token has been consumed.
    fn prev_end(&self) -> u32 {
        if self.pos == 0 {
            0
        } else {
            self.tokens[self.pos - 1].end as u32
        }
    }

    /// Build a [`Span`] from a `mark()` taken before parsing to the end of the
    /// last consumed token.
    fn span_since(&self, mark: (u32, u32, u32)) -> Span {
        Span::new(mark.0, self.prev_end(), mark.1, mark.2)
    }

    fn is_builtin_component(&self) -> bool {
        matches!(self.current_type(),
            TokenType::Container | TokenType::Row | TokenType::Column |
            TokenType::Grid | TokenType::Stack | TokenType::Spacer | TokenType::Divider |
            TokenType::Navbar | TokenType::Sidebar | TokenType::Breadcrumb |
            TokenType::Link | TokenType::Menu | TokenType::Tabs | TokenType::TabPage |
            TokenType::Card | TokenType::Table | TokenType::Thead | TokenType::Tbody |
            TokenType::Trow | TokenType::Tcell | TokenType::Badge |
            TokenType::Avatar | TokenType::Tooltip | TokenType::Tag |
            TokenType::Input | TokenType::Select | TokenType::Option |
            TokenType::Checkbox | TokenType::Radio | TokenType::Switch |
            TokenType::Slider | TokenType::DatePicker | TokenType::FileUpload | TokenType::Form |
            TokenType::Alert | TokenType::Toast | TokenType::Modal | TokenType::Dialog |
            TokenType::Spinner | TokenType::Progress | TokenType::Skeleton |
            TokenType::Button | TokenType::IconButton | TokenType::ButtonGroup | TokenType::Dropdown |
            TokenType::Image | TokenType::Video | TokenType::Icon | TokenType::Carousel |
            TokenType::Text | TokenType::Heading | TokenType::Code | TokenType::Blockquote |
            TokenType::Router | TokenType::Route |
            TokenType::TypeList | // List component
            TokenType::Document | TokenType::Section | TokenType::Paragraph |
            TokenType::PageBreak | TokenType::Header | TokenType::Footer |
            TokenType::Presentation | TokenType::Slide | TokenType::TitleSlide |
            TokenType::SectionSlide | TokenType::TwoColumn | TokenType::ImageSlide
        )
    }

    fn builtin_name(&self) -> String {
        format!("{}", self.current_type())
    }

    // ─── Top-level declarations ──────────────────────────

    fn parse_declaration(&mut self) -> Result<Declaration> {
        match self.current_type() {
            TokenType::Page => Ok(Declaration::Page(self.parse_page()?)),
            TokenType::Component => Ok(Declaration::Component(self.parse_component_decl()?)),
            TokenType::Store => Ok(Declaration::Store(self.parse_store()?)),
            TokenType::App => Ok(Declaration::App(self.parse_app()?)),
            _ => Err(self.error(format!("Expected Page, Component, Store, or App declaration, got {}", self.current_type()))),
        }
    }

    // ─── Page ────────────────────────────────────────────

    fn parse_page(&mut self) -> Result<PageDecl> {
        let decl_mark = self.mark();
        self.expect(&TokenType::Page)?;
        let name = self.expect_declaration_name()?;
        self.expect(&TokenType::OpenParen)?;

        let mut path = String::new();
        let mut title = None;
        let mut guard = None;
        let mut redirect = None;

        // Parse page attributes
        while !self.check(&TokenType::CloseParen) {
            let attr_name = self.expect_identifier()?;
            self.expect(&TokenType::Colon)?;
            match attr_name.as_str() {
                "path" => path = self.expect_string()?,
                "title" => title = Some(self.expect_string()?),
                "guard" => guard = Some(self.parse_expression()?),
                "redirect" => redirect = Some(self.expect_string()?),
                _ => return Err(self.error(format!("Unknown page attribute '{}'", attr_name))),
            }
            if !self.check(&TokenType::CloseParen) {
                self.expect(&TokenType::Comma)?;
            }
        }

        self.expect(&TokenType::CloseParen)?;
        let header_span = self.span_since(decl_mark);
        let (body, body_span) = self.parse_block_spanned()?;

        Ok(PageDecl {
            name, path, title, guard, redirect, body,
            span: self.span_since(decl_mark),
            header_span,
            body_span,
        })
    }

    // ─── Component ───────────────────────────────────────

    fn parse_component_decl(&mut self) -> Result<ComponentDecl> {
        let decl_mark = self.mark();
        self.expect(&TokenType::Component)?;
        let name = self.expect_declaration_name()?;
        self.expect(&TokenType::OpenParen)?;

        let mut props = Vec::new();
        while !self.check(&TokenType::CloseParen) {
            props.push(self.parse_prop_decl()?);
            if !self.check(&TokenType::CloseParen) {
                self.expect(&TokenType::Comma)?;
            }
        }

        self.expect(&TokenType::CloseParen)?;
        let header_span = self.span_since(decl_mark);
        let (body, body_span) = self.parse_block_spanned()?;

        Ok(ComponentDecl {
            name, props, body,
            span: self.span_since(decl_mark),
            header_span,
            body_span,
        })
    }

    fn parse_prop_decl(&mut self) -> Result<PropDecl> {
        let name = self.expect_identifier()?;
        let optional = self.match_token(&TokenType::QuestionMark);
        self.expect(&TokenType::Colon)?;
        let prop_type = self.parse_type()?;
        let default = if self.match_token(&TokenType::Equals) {
            Some(self.parse_expression()?)
        } else {
            None
        };
        Ok(PropDecl { name, prop_type, optional, default })
    }

    fn parse_type(&mut self) -> Result<WfType> {
        match self.current_type() {
            TokenType::TypeString => { self.advance(); Ok(WfType::String) }
            TokenType::TypeNumber => { self.advance(); Ok(WfType::Number) }
            TokenType::TypeBool => { self.advance(); Ok(WfType::Bool) }
            TokenType::TypeList => { self.advance(); Ok(WfType::List) }
            TokenType::TypeMap => { self.advance(); Ok(WfType::Map) }
            _ => Err(self.error(format!("Expected type (String, Number, Bool, List, Map), got {}", self.current_type()))),
        }
    }

    // ─── Store ───────────────────────────────────────────

    fn parse_store(&mut self) -> Result<StoreDecl> {
        let decl_mark = self.mark();
        self.expect(&TokenType::Store)?;
        let name = self.expect_identifier()?;
        let header_span = self.span_since(decl_mark);
        let (body, body_span) = self.parse_block_spanned()?;
        Ok(StoreDecl {
            name, body,
            span: self.span_since(decl_mark),
            header_span,
            body_span,
        })
    }

    // ─── App ─────────────────────────────────────────────

    fn parse_app(&mut self) -> Result<AppDecl> {
        self.expect(&TokenType::App)?;
        let body = self.parse_block()?;
        Ok(AppDecl { body })
    }

    // ─── Block ───────────────────────────────────────────

    fn parse_block(&mut self) -> Result<Vec<Statement>> {
        Ok(self.parse_block_spanned()?.0)
    }

    /// Like [`parse_block`], but also returns the span of the block's interior
    /// (between the braces, exclusive of them) for declaration `body_span`s.
    fn parse_block_spanned(&mut self) -> Result<(Vec<Statement>, Span)> {
        // Interior starts one byte past `{` and ends at the byte before `}`.
        let body_start = self.current().end as u32;
        let body_line = self.current().line as u32;
        let body_col = self.current().column as u32;
        self.expect(&TokenType::OpenBrace)?;
        let mut stmts = Vec::new();
        while !self.check(&TokenType::CloseBrace) && !self.is_at_end() {
            stmts.push(self.parse_statement()?);
        }
        let body_end = self.current().offset as u32; // start of `}`
        self.expect(&TokenType::CloseBrace)?;
        Ok((stmts, Span::new(body_start, body_end, body_line, body_col)))
    }

    // ─── Statement ───────────────────────────────────────

    fn parse_statement(&mut self) -> Result<Statement> {
        // Single choke point for statement spans: every statement flows through
        // here, so stamping the whole-statement span once covers them all.
        let stmt_mark = self.mark();
        let kind = self.parse_statement_kind()?;
        Ok(Statement { kind, span: self.span_since(stmt_mark) })
    }

    fn parse_statement_kind(&mut self) -> Result<StatementKind> {
        match self.current_type() {
            TokenType::State => self.parse_state_decl(),
            TokenType::Derived => self.parse_derived_decl(),
            TokenType::Effect => self.parse_effect_decl(),
            TokenType::Action => self.parse_action_decl(),
            TokenType::Use => self.parse_use_decl(),
            TokenType::If => self.parse_if_stmt(),
            TokenType::For => self.parse_for_stmt(),
            TokenType::Show => self.parse_show_stmt(),
            TokenType::Fetch => self.parse_fetch_decl(),
            TokenType::Navigate => self.parse_navigate(),
            TokenType::Log => self.parse_log(),
            TokenType::Return => self.parse_return(),
            TokenType::Children => {
                let node_mark = self.mark();
                self.advance();
                Ok(StatementKind::UIElement(UIElement {
                    component: ComponentRef::BuiltIn("Children".to_string()),
                    args: Vec::new(),
                    modifiers: Vec::new(),
                    children: Vec::new(),
                    style_block: None,
                    transition_block: None,
                    events: Vec::new(),
                    span: self.span_since(node_mark),
                    paren_span: None,
                    body_span: None,
                    style_span: None,
                    arg_spans: Vec::new(),
                    modifier_spans: Vec::new(),
                }))
            }
            TokenType::Animate => self.parse_animate_stmt(),
            TokenType::Style => self.parse_style_statement(),
            TokenType::Event(_) => {
                let handler = self.parse_event_handler()?;
                Ok(StatementKind::EventHandler(handler))
            }
            _ if self.is_builtin_component() => {
                let elem = self.parse_ui_element()?;
                Ok(StatementKind::UIElement(elem))
            }
            TokenType::Identifier(_) => self.parse_identifier_statement(),
            _ => Err(self.error(format!("Unexpected token {}", self.current_type()))),
        }
    }

    // ─── State declarations ─────────────────────────────

    fn parse_state_decl(&mut self) -> Result<StatementKind> {
        self.expect(&TokenType::State)?;
        let name = self.expect_identifier()?;
        self.expect(&TokenType::Equals)?;
        let value = self.parse_expression()?;
        Ok(StatementKind::State(StateDecl { name, value }))
    }

    fn parse_derived_decl(&mut self) -> Result<StatementKind> {
        self.expect(&TokenType::Derived)?;
        let name = self.expect_identifier()?;
        self.expect(&TokenType::Equals)?;
        let value = self.parse_expression()?;
        Ok(StatementKind::Derived(DerivedDecl { name, value }))
    }

    fn parse_effect_decl(&mut self) -> Result<StatementKind> {
        self.expect(&TokenType::Effect)?;
        let body = self.parse_block()?;
        Ok(StatementKind::Effect(EffectDecl { body }))
    }

    fn parse_action_decl(&mut self) -> Result<StatementKind> {
        self.expect(&TokenType::Action)?;
        let name = self.expect_identifier()?;
        self.expect(&TokenType::OpenParen)?;

        let mut params = Vec::new();
        while !self.check(&TokenType::CloseParen) {
            let param_name = self.expect_identifier()?;
            self.expect(&TokenType::Colon)?;
            let param_type = self.parse_type()?;
            params.push(ParamDecl { name: param_name, param_type });
            if !self.check(&TokenType::CloseParen) {
                self.expect(&TokenType::Comma)?;
            }
        }
        self.expect(&TokenType::CloseParen)?;
        let body = self.parse_block()?;

        Ok(StatementKind::Action(ActionDecl { name, params, body }))
    }

    fn parse_use_decl(&mut self) -> Result<StatementKind> {
        self.expect(&TokenType::Use)?;
        let store_name = self.expect_identifier()?;
        Ok(StatementKind::Use(UseDecl { store_name }))
    }

    fn parse_navigate(&mut self) -> Result<StatementKind> {
        self.expect(&TokenType::Navigate)?;
        self.expect(&TokenType::OpenParen)?;
        let expr = self.parse_expression()?;
        self.expect(&TokenType::CloseParen)?;
        Ok(StatementKind::Navigate(expr))
    }

    fn parse_log(&mut self) -> Result<StatementKind> {
        self.expect(&TokenType::Log)?;
        self.expect(&TokenType::OpenParen)?;
        let expr = self.parse_expression()?;
        self.expect(&TokenType::CloseParen)?;
        Ok(StatementKind::Log(expr))
    }

    fn parse_return(&mut self) -> Result<StatementKind> {
        self.expect(&TokenType::Return)?;
        // Return with optional expression - if next token starts an expression, parse it
        if self.check(&TokenType::CloseBrace) || self.is_at_end() {
            Ok(StatementKind::Return(None))
        } else {
            let expr = self.parse_expression()?;
            Ok(StatementKind::Return(Some(expr)))
        }
    }

    // ─── Control flow ────────────────────────────────────

    fn parse_if_stmt(&mut self) -> Result<StatementKind> {
        self.expect(&TokenType::If)?;
        let condition = self.parse_expression()?;

        // Optional animate clause: , animate(...)
        let animate = self.parse_optional_animate_clause()?;

        let then_body = self.parse_block()?;

        let mut else_if_branches = Vec::new();
        let mut else_body = None;

        while self.match_token(&TokenType::Else) {
            if self.match_token(&TokenType::If) {
                let cond = self.parse_expression()?;
                let body = self.parse_block()?;
                else_if_branches.push((cond, body));
            } else {
                else_body = Some(self.parse_block()?);
                break;
            }
        }

        Ok(StatementKind::If(IfStmt {
            condition,
            animate,
            then_body,
            else_if_branches,
            else_body,
        }))
    }

    fn parse_for_stmt(&mut self) -> Result<StatementKind> {
        self.expect(&TokenType::For)?;
        let item = self.expect_identifier()?;
        let index = if self.match_token(&TokenType::Comma) {
            // Check if next is an identifier (index var) or `animate` keyword
            if self.check(&TokenType::Animate) {
                None
            } else {
                Some(self.expect_identifier()?)
            }
        } else {
            None
        };
        self.expect(&TokenType::In)?;
        let iterable = self.parse_expression()?;

        // Optional animate clause: , animate(...)
        let animate = self.parse_optional_animate_clause()?;

        let body = self.parse_block()?;

        Ok(StatementKind::For(ForStmt { item, index, iterable, animate, body }))
    }

    fn parse_show_stmt(&mut self) -> Result<StatementKind> {
        self.expect(&TokenType::Show)?;
        let condition = self.parse_expression()?;

        // Optional animate clause: , animate(...)
        let animate = self.parse_optional_animate_clause()?;

        let body = self.parse_block()?;
        Ok(StatementKind::Show(ShowStmt { condition, animate, body }))
    }

    /// Parse optional `, animate(enter, exit, duration: "300ms", ...)` clause
    fn parse_optional_animate_clause(&mut self) -> Result<Option<AnimateConfig>> {
        // Check for comma followed by `animate`
        if self.check(&TokenType::Comma) {
            // Look ahead: is next token `animate`?
            if self.pos + 1 < self.tokens.len() && matches!(self.tokens[self.pos + 1].token_type, TokenType::Animate) {
                self.advance(); // skip comma
                return Ok(Some(self.parse_animate_config()?));
            }
        }
        // Also accept `animate` directly without comma (for `for` where comma was already consumed)
        if self.check(&TokenType::Animate) {
            return Ok(Some(self.parse_animate_config()?));
        }
        Ok(None)
    }

    fn parse_animate_config(&mut self) -> Result<AnimateConfig> {
        self.expect(&TokenType::Animate)?;
        self.expect(&TokenType::OpenParen)?;

        // First positional: enter animation name
        let enter = self.expect_identifier()?;
        let mut exit = None;
        let mut duration = None;
        let mut delay = None;
        let mut stagger = None;
        let mut easing = None;

        while self.match_token(&TokenType::Comma) {
            if self.check(&TokenType::CloseParen) {
                break;
            }
            // Check if named arg
            if self.is_named_arg() {
                let key = self.expect_identifier()?;
                self.expect(&TokenType::Colon)?;
                let val = self.expect_string()?;
                match key.as_str() {
                    "duration" => duration = Some(val),
                    "delay" => delay = Some(val),
                    "stagger" => stagger = Some(val),
                    "easing" => easing = Some(val),
                    _ => {}
                }
            } else {
                // Second positional: exit animation name
                if exit.is_none() {
                    exit = Some(self.expect_identifier()?);
                } else {
                    // Skip unknown positional
                    let _ = self.parse_expression()?;
                }
            }
        }

        self.expect(&TokenType::CloseParen)?;

        Ok(AnimateConfig { enter, exit, duration, delay, stagger, easing })
    }

    // ─── Fetch ───────────────────────────────────────────

    fn parse_fetch_decl(&mut self) -> Result<StatementKind> {
        self.expect(&TokenType::Fetch)?;
        let variable = self.expect_identifier()?;
        self.expect(&TokenType::From)?;
        let url = self.parse_fetch_url_expression()?;

        // Optional fetch options
        let mut options = Vec::new();
        if self.match_token(&TokenType::OpenParen) {
            while !self.check(&TokenType::CloseParen) {
                let key = self.expect_identifier()?;
                self.expect(&TokenType::Colon)?;
                let value = self.parse_expression()?;
                options.push(FetchOption { key, value });
                if !self.check(&TokenType::CloseParen) {
                    self.expect(&TokenType::Comma)?;
                }
            }
            self.expect(&TokenType::CloseParen)?;
        }

        self.expect(&TokenType::OpenBrace)?;

        let mut loading_block = None;
        let mut error_block = None;
        let mut success_block = None;

        while !self.check(&TokenType::CloseBrace) {
            match self.current_type() {
                TokenType::Loading => {
                    self.advance();
                    loading_block = Some(self.parse_block()?);
                }
                TokenType::Error => {
                    self.advance();
                    let err_var = if self.match_token(&TokenType::OpenParen) {
                        let name = self.expect_identifier()?;
                        self.expect(&TokenType::CloseParen)?;
                        name
                    } else {
                        "err".to_string()
                    };
                    let body = self.parse_block()?;
                    error_block = Some((err_var, body));
                }
                TokenType::Success => {
                    self.advance();
                    success_block = Some(self.parse_block()?);
                }
                _ => return Err(self.error("Expected 'loading', 'error', or 'success' in fetch block".to_string())),
            }
        }

        self.expect(&TokenType::CloseBrace)?;

        Ok(StatementKind::Fetch(FetchDecl {
            variable,
            url,
            options,
            loading_block,
            error_block,
            success_block,
        }))
    }

    /// Parse an expression for the fetch URL context.
    /// This avoids consuming `(` as a function call when the `(` starts fetch options
    /// (i.e., `(identifier: ...)`).
    fn parse_fetch_url_expression(&mut self) -> Result<Expr> {
        self.parse_fetch_url_or()
    }

    fn parse_fetch_url_or(&mut self) -> Result<Expr> {
        let mut left = self.parse_fetch_url_and()?;
        while self.match_token(&TokenType::Or) {
            let right = self.parse_fetch_url_and()?;
            left = Expr::BinaryOp(Box::new(left), BinOp::Or, Box::new(right));
        }
        Ok(left)
    }

    fn parse_fetch_url_and(&mut self) -> Result<Expr> {
        let mut left = self.parse_fetch_url_equality()?;
        while self.match_token(&TokenType::And) {
            let right = self.parse_fetch_url_equality()?;
            left = Expr::BinaryOp(Box::new(left), BinOp::And, Box::new(right));
        }
        Ok(left)
    }

    fn parse_fetch_url_equality(&mut self) -> Result<Expr> {
        let mut left = self.parse_fetch_url_comparison()?;
        loop {
            if self.match_token(&TokenType::DoubleEquals) {
                let right = self.parse_fetch_url_comparison()?;
                left = Expr::BinaryOp(Box::new(left), BinOp::Eq, Box::new(right));
            } else if self.match_token(&TokenType::NotEquals) || self.match_token(&TokenType::StrictNotEqual) {
                let right = self.parse_fetch_url_comparison()?;
                left = Expr::BinaryOp(Box::new(left), BinOp::Neq, Box::new(right));
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_fetch_url_comparison(&mut self) -> Result<Expr> {
        let mut left = self.parse_fetch_url_addition()?;
        loop {
            if self.match_token(&TokenType::LessThan) {
                let right = self.parse_fetch_url_addition()?;
                left = Expr::BinaryOp(Box::new(left), BinOp::Lt, Box::new(right));
            } else if self.match_token(&TokenType::GreaterThan) {
                let right = self.parse_fetch_url_addition()?;
                left = Expr::BinaryOp(Box::new(left), BinOp::Gt, Box::new(right));
            } else if self.match_token(&TokenType::LessEquals) {
                let right = self.parse_fetch_url_addition()?;
                left = Expr::BinaryOp(Box::new(left), BinOp::Lte, Box::new(right));
            } else if self.match_token(&TokenType::GreaterEquals) {
                let right = self.parse_fetch_url_addition()?;
                left = Expr::BinaryOp(Box::new(left), BinOp::Gte, Box::new(right));
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_fetch_url_addition(&mut self) -> Result<Expr> {
        let mut left = self.parse_fetch_url_multiplication()?;
        loop {
            if self.match_token(&TokenType::Plus) {
                let right = self.parse_fetch_url_multiplication()?;
                left = Expr::BinaryOp(Box::new(left), BinOp::Add, Box::new(right));
            } else if self.match_token(&TokenType::Minus) {
                let right = self.parse_fetch_url_multiplication()?;
                left = Expr::BinaryOp(Box::new(left), BinOp::Sub, Box::new(right));
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_fetch_url_multiplication(&mut self) -> Result<Expr> {
        let mut left = self.parse_fetch_url_unary()?;
        loop {
            if self.match_token(&TokenType::Star) {
                let right = self.parse_fetch_url_unary()?;
                left = Expr::BinaryOp(Box::new(left), BinOp::Mul, Box::new(right));
            } else if self.match_token(&TokenType::Slash) {
                let right = self.parse_fetch_url_unary()?;
                left = Expr::BinaryOp(Box::new(left), BinOp::Div, Box::new(right));
            } else if self.match_token(&TokenType::Percent) {
                let right = self.parse_fetch_url_unary()?;
                left = Expr::BinaryOp(Box::new(left), BinOp::Mod, Box::new(right));
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_fetch_url_unary(&mut self) -> Result<Expr> {
        if self.match_token(&TokenType::Not) {
            let expr = self.parse_fetch_url_unary()?;
            return Ok(Expr::UnaryOp(UnaryOp::Not, Box::new(expr)));
        }
        if self.match_token(&TokenType::Minus) {
            let expr = self.parse_fetch_url_unary()?;
            return Ok(Expr::UnaryOp(UnaryOp::Neg, Box::new(expr)));
        }
        self.parse_fetch_url_postfix()
    }

    fn parse_fetch_url_postfix(&mut self) -> Result<Expr> {
        let mut expr = self.parse_fetch_url_primary()?;
        loop {
            if self.match_token(&TokenType::Dot) {
                let prop = self.expect_identifier()?;
                if self.check(&TokenType::OpenParen) {
                    self.advance();
                    let mut args = Vec::new();
                    while !self.check(&TokenType::CloseParen) {
                        args.push(self.parse_expression()?);
                        if !self.check(&TokenType::CloseParen) {
                            self.expect(&TokenType::Comma)?;
                        }
                    }
                    self.expect(&TokenType::CloseParen)?;
                    expr = Expr::MethodCall(Box::new(expr), prop, args);
                } else {
                    expr = Expr::PropertyAccess(Box::new(expr), prop);
                }
            } else if self.match_token(&TokenType::OpenBracket) {
                let index = self.parse_expression()?;
                self.expect(&TokenType::CloseBracket)?;
                expr = Expr::IndexAccess(Box::new(expr), Box::new(index));
            } else {
                break;
            }
        }
        Ok(expr)
    }

    /// Like parse_primary but does NOT consume `(` as function call on identifiers
    /// when it looks like fetch options (identifier followed by colon).
    fn parse_fetch_url_primary(&mut self) -> Result<Expr> {
        match self.current_type().clone() {
            TokenType::StringLiteral(s) => {
                let s = s.clone();
                self.advance();
                if has_interpolation(&s) {
                    let parts = self.parse_interpolated_string(&s)?;
                    Ok(Expr::InterpolatedString(parts))
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
            TokenType::Identifier(name) => {
                let name = name.clone();
                self.advance();

                // Check for lambda
                if self.check(&TokenType::Arrow) {
                    self.advance();
                    let body = self.parse_expression()?;
                    return Ok(Expr::Lambda(name, Box::new(body)));
                }

                // Check for function call BUT NOT if it looks like fetch options
                if self.check(&TokenType::OpenParen) {
                    // Look ahead: if `(` is followed by identifier + `:`, it's fetch options
                    if self.is_fetch_options_start() {
                        return Ok(Expr::Identifier(name));
                    }
                    self.advance();
                    let mut args = Vec::new();
                    while !self.check(&TokenType::CloseParen) {
                        args.push(self.parse_expression()?);
                        if !self.check(&TokenType::CloseParen) {
                            self.expect(&TokenType::Comma)?;
                        }
                    }
                    self.expect(&TokenType::CloseParen)?;
                    return Ok(Expr::FunctionCall(name, args));
                }

                Ok(Expr::Identifier(name))
            }
            TokenType::OpenParen => {
                // Check if this is fetch options rather than a grouped expression
                if self.is_fetch_options_start() {
                    return Err(self.error("Unexpected token in fetch URL expression".to_string()));
                }
                self.advance();
                let expr = self.parse_expression()?;
                self.expect(&TokenType::CloseParen)?;
                Ok(expr)
            }
            TokenType::OpenBracket => {
                self.advance();
                let mut items = Vec::new();
                while !self.check(&TokenType::CloseBracket) {
                    items.push(self.parse_expression()?);
                    if !self.check(&TokenType::CloseBracket) {
                        self.expect(&TokenType::Comma)?;
                    }
                }
                self.expect(&TokenType::CloseBracket)?;
                Ok(Expr::ListLiteral(items))
            }
            _ => Err(self.error(format!("Expected expression, got {}", self.current_type()))),
        }
    }

    /// Check if the current `(` token starts a fetch options block
    /// i.e., `(identifier: ...)` pattern.
    fn is_fetch_options_start(&self) -> bool {
        if !self.check(&TokenType::OpenParen) {
            return false;
        }
        // Look ahead: ( + identifier + colon
        if self.pos + 2 < self.tokens.len() {
            let after_paren = &self.tokens[self.pos + 1].token_type;
            let after_ident = &self.tokens[self.pos + 2].token_type;
            if let TokenType::Identifier(_) = after_paren {
                return matches!(after_ident, TokenType::Colon);
            }
        }
        false
    }

    // ─── UI Elements ─────────────────────────────────────

    fn parse_ui_element(&mut self) -> Result<UIElement> {
        let node_mark = self.mark();
        let component = self.parse_component_ref()?;

        let (args, modifiers, arg_spans, modifier_spans, paren_span) =
            if self.check(&TokenType::OpenParen) {
                let p = self.parse_paren_args()?;
                (p.args, p.modifiers, p.arg_spans, p.modifier_spans, Some(p.paren_span))
            } else {
                (Vec::new(), Vec::new(), Vec::new(), Vec::new(), None)
            };

        let body = self.parse_element_body()?;

        Ok(UIElement {
            component,
            args,
            modifiers,
            children: body.children,
            style_block: body.style_block,
            transition_block: body.transition_block,
            events: body.events,
            span: self.span_since(node_mark),
            paren_span,
            body_span: body.body_span,
            style_span: body.style_span,
            arg_spans,
            modifier_spans,
        })
    }

    /// Parse a `( … )` argument + modifier group. The current token MUST be an
    /// `OpenParen`. `paren_span` covers the whole group (parentheses included);
    /// each argument and modifier also records its own span for surgical edits.
    fn parse_paren_args(&mut self) -> Result<ParenArgs> {
        let paren_mark = self.mark();
        self.advance(); // consume '('
        let mut args = Vec::new();
        let mut modifiers = Vec::new();
        let mut arg_spans = Vec::new();
        let mut modifier_spans = Vec::new();
        while !self.check(&TokenType::CloseParen) && !self.is_at_end() {
            let item_mark = self.mark();
            // Named argument: identifier followed by a colon.
            if self.is_named_arg() {
                let name = self.expect_identifier()?;
                self.expect(&TokenType::Colon)?;
                let value = self.parse_expression()?;
                args.push(Arg::Named(name, value));
                arg_spans.push(self.span_since(item_mark));
            } else if self.is_modifier() {
                let mod_name = self.expect_identifier()?;
                modifiers.push(mod_name);
                modifier_spans.push(self.span_since(item_mark));
            } else {
                let expr = self.parse_expression()?;
                args.push(Arg::Positional(expr));
                arg_spans.push(self.span_since(item_mark));
            }
            if !self.check(&TokenType::CloseParen) {
                self.expect(&TokenType::Comma)?;
            }
        }
        self.expect(&TokenType::CloseParen)?;
        Ok(ParenArgs {
            args,
            modifiers,
            arg_spans,
            modifier_spans,
            paren_span: self.span_since(paren_mark),
        })
    }

    /// Parse the optional `{ … }` body shared by built-in and user-defined
    /// elements: children, event handlers, an optional style block, and an
    /// optional transition block. `body_span` is the interior of the braces
    /// (exclusive); both spans are `None` when the element has no block.
    fn parse_element_body(&mut self) -> Result<ElementBody> {
        let mut children = Vec::new();
        let mut events = Vec::new();
        let mut style_block = None;
        let mut style_span = None;
        let mut transition_block = None;
        let mut body_span = None;

        if self.check(&TokenType::OpenBrace) {
            // Interior starts one byte past `{` and ends at the byte before `}`.
            let body_start = self.current().end as u32;
            let body_line = self.current().line as u32;
            let body_col = self.current().column as u32;
            self.expect(&TokenType::OpenBrace)?;
            while !self.check(&TokenType::CloseBrace) && !self.is_at_end() {
                match self.current_type() {
                    TokenType::Event(_) => {
                        events.push(self.parse_event_handler()?);
                    }
                    TokenType::Style => {
                        let style_mark = self.mark();
                        style_block = Some(self.parse_style_block()?);
                        style_span = Some(self.span_since(style_mark));
                    }
                    TokenType::Transition => {
                        transition_block = Some(self.parse_transition_block()?);
                    }
                    _ => {
                        children.push(self.parse_statement()?);
                    }
                }
            }
            let body_end = self.current().offset as u32; // start of `}`
            self.expect(&TokenType::CloseBrace)?;
            body_span = Some(Span::new(body_start, body_end, body_line, body_col));
        }

        Ok(ElementBody {
            children,
            events,
            style_block,
            transition_block,
            body_span,
            style_span,
        })
    }

    fn parse_component_ref(&mut self) -> Result<ComponentRef> {
        let name = self.builtin_name();
        self.advance();

        // Check for sub-component: Navbar.Brand, Card.Header, Card.Footer, etc.
        if self.check(&TokenType::Dot) {
            self.advance();
            // Accept identifiers and keywords (Header/Footer are now keyword tokens)
            let sub = if let TokenType::Identifier(_) = self.current_type() {
                self.expect_identifier()?
            } else if self.is_builtin_component() {
                let n = self.builtin_name();
                self.advance();
                n
            } else {
                return Err(self.error(format!("Expected sub-component name, got {}", self.current_type())));
            };
            Ok(ComponentRef::SubComponent(name, sub))
        } else {
            Ok(ComponentRef::BuiltIn(name))
        }
    }

    fn is_named_arg(&self) -> bool {
        if let TokenType::Identifier(_) = self.current_type() {
            if self.pos + 1 < self.tokens.len() {
                return matches!(self.tokens[self.pos + 1].token_type, TokenType::Colon);
            }
        }
        false
    }

    fn is_modifier(&self) -> bool {
        // Keywords that can also be used as modifiers
        if matches!(self.current_type(), TokenType::Success | TokenType::Error | TokenType::Loading) {
            return true;
        }
        // The vocabulary lives in `parser::vocabulary` as data (one source of
        // truth for the parser, the lint and the LSP), not as a match arm here.
        if let TokenType::Identifier(name) = self.current_type() {
            crate::parser::vocabulary::is_modifier_keyword(name)
        } else {
            false
        }
    }

    // ─── Style ───────────────────────────────────────────

    fn parse_style_block(&mut self) -> Result<StyleBlock> {
        self.expect(&TokenType::Style)?;
        // Interior span: one byte past `{` to the byte before `}`.
        let body_start = self.current().end as u32;
        let body_line = self.current().line as u32;
        let body_col = self.current().column as u32;
        self.expect(&TokenType::OpenBrace)?;
        let mut properties = Vec::new();
        let mut media_queries = Vec::new();
        while !self.check(&TokenType::CloseBrace) && !self.is_at_end() {
            // Check for @media query
            if self.check_at_rule() {
                media_queries.push(self.parse_media_query()?);
                continue;
            }
            let prop = self.parse_style_property()?;
            properties.push(prop);
        }
        let body_end = self.current().offset as u32;
        self.expect(&TokenType::CloseBrace)?;
        Ok(StyleBlock {
            properties,
            media_queries,
            body_span: Span::new(body_start, body_end, body_line, body_col),
        })
    }

    fn parse_style_property(&mut self) -> Result<StyleProperty> {
        // Support hyphenated property names: border-radius, font-size, etc.
        // Also accept keywords (transition, etc.) as CSS property names
        let prop_mark = self.mark();
        let mut name = self.expect_css_property_name()?;
        while self.check(&TokenType::Minus) {
            self.advance(); // consume -
            let part = self.expect_css_property_name()?;
            name = format!("{}-{}", name, part);
        }
        self.expect(&TokenType::Colon)?;
        let value_mark = self.mark();
        let value = self.parse_expression()?;
        let value_span = self.span_since(value_mark);
        Ok(StyleProperty { name, value, span: self.span_since(prop_mark), value_span })
    }

    fn expect_css_property_name(&mut self) -> Result<String> {
        match self.current_type().clone() {
            TokenType::Identifier(name) => { self.advance(); Ok(name) }
            // Allow WebFluent keywords as CSS property names in style blocks
            TokenType::Transition => { self.advance(); Ok("transition".to_string()) }
            TokenType::Loading => { self.advance(); Ok("loading".to_string()) }
            TokenType::Error => { self.advance(); Ok("error".to_string()) }
            TokenType::Success => { self.advance(); Ok("success".to_string()) }
            TokenType::Action => { self.advance(); Ok("action".to_string()) }
            TokenType::State => { self.advance(); Ok("state".to_string()) }
            TokenType::Style => { self.advance(); Ok("style".to_string()) }
            TokenType::Show => { self.advance(); Ok("show".to_string()) }
            _ => Err(self.error(format!("Expected CSS property name, got {}", self.current_type()))),
        }
    }

    fn check_at_rule(&self) -> bool {
        if let TokenType::Identifier(s) = self.current_type() {
            s.starts_with('@')
        } else {
            false
        }
    }

    fn parse_media_query(&mut self) -> Result<MediaQuery> {
        // Build the @media condition by consuming tokens until '{'
        // Reconstruct CSS-style spacing: join with hyphens for Ident-Minus-Ident,
        // no space inside parens, space after colon.
        let mut parts: Vec<String> = Vec::new();

        while !self.check(&TokenType::OpenBrace) && !self.is_at_end() {
            let tok = self.current_type().clone();
            let text = match &tok {
                TokenType::Identifier(s) => s.clone(),
                TokenType::StringLiteral(s) => format!("\"{}\"", s),
                TokenType::NumberLiteral(n) => {
                    if *n == (*n as i64) as f64 {
                        format!("{}", *n as i64)
                    } else {
                        format!("{}", n)
                    }
                }
                TokenType::OpenParen => "(".to_string(),
                TokenType::CloseParen => ")".to_string(),
                TokenType::Colon => ":".to_string(),
                TokenType::Minus => "-".to_string(),
                _ => format!("{}", tok),
            };
            parts.push(text);
            self.advance();
        }

        // Reconstruct: @media (max-width: 768px)
        let mut condition = String::new();
        for (i, part) in parts.iter().enumerate() {
            if i > 0 {
                let prev = &parts[i - 1];
                let is_unit = part.chars().next().map_or(false, |c| c.is_alphabetic())
                    && prev.chars().all(|c| c.is_ascii_digit() || c == '.');
                // No space: around hyphens, inside parens, before colon, number+unit (768px)
                if part == "-" || prev == "-"
                    || prev == "(" || part == ")"
                    || part == ":"
                    || is_unit
                {
                    // no space
                } else if prev == ":" {
                    condition.push(' ');
                } else {
                    condition.push(' ');
                }
            }
            condition.push_str(part);
        }

        self.expect(&TokenType::OpenBrace)?;
        let mut properties = Vec::new();
        while !self.check(&TokenType::CloseBrace) && !self.is_at_end() {
            properties.push(self.parse_style_property()?);
        }
        self.expect(&TokenType::CloseBrace)?;

        Ok(MediaQuery { condition, properties })
    }

    fn parse_style_statement(&mut self) -> Result<StatementKind> {
        let node_mark = self.mark();
        let block = self.parse_style_block()?;
        let span = self.span_since(node_mark);
        Ok(StatementKind::UIElement(UIElement {
            component: ComponentRef::BuiltIn("_StyleBlock".to_string()),
            args: Vec::new(),
            modifiers: Vec::new(),
            children: Vec::new(),
            style_block: Some(block),
            transition_block: None,
            events: Vec::new(),
            span,
            paren_span: None,
            body_span: None,
            style_span: Some(span),
            arg_spans: Vec::new(),
            modifier_spans: Vec::new(),
        }))
    }

    // ─── Transition ──────────────────────────────────────

    fn parse_transition_block(&mut self) -> Result<TransitionBlock> {
        self.expect(&TokenType::Transition)?;
        self.expect(&TokenType::OpenBrace)?;
        let mut properties = Vec::new();
        while !self.check(&TokenType::CloseBrace) && !self.is_at_end() {
            let property = self.expect_identifier()?;
            // Duration: next token should be a string like "200ms" or an identifier like "fast"
            let duration = match self.current_type().clone() {
                TokenType::StringLiteral(s) => { self.advance(); s }
                TokenType::Identifier(s) => {
                    self.advance();
                    match s.as_str() {
                        "fast" => "150ms".to_string(),
                        "normal" => "250ms".to_string(),
                        "slow" => "350ms".to_string(),
                        _ => s,
                    }
                }
                TokenType::NumberLiteral(n) => {
                    self.advance();
                    format!("{}ms", n as i32)
                }
                _ => return Err(self.error("Expected duration value in transition".to_string())),
            };
            // Optional easing
            let easing = if let TokenType::Identifier(name) = self.current_type() {
                if matches!(name.as_str(), "ease" | "linear" | "easeIn" | "easeOut" | "easeInOut" | "spring" | "bouncy" | "smooth") {
                    let e = name.clone();
                    self.advance();
                    Some(e)
                } else {
                    None
                }
            } else {
                None
            };
            properties.push(TransitionProperty { property, duration, easing });
        }
        self.expect(&TokenType::CloseBrace)?;
        Ok(TransitionBlock { properties })
    }

    // ─── Animate statement ───────────────────────────────

    fn parse_animate_stmt(&mut self) -> Result<StatementKind> {
        self.expect(&TokenType::Animate)?;
        self.expect(&TokenType::OpenParen)?;
        let target = self.expect_identifier()?;
        self.expect(&TokenType::Comma)?;
        let animation = self.expect_identifier()?;
        let duration = if self.match_token(&TokenType::Comma) {
            Some(self.expect_string()?)
        } else {
            None
        };
        self.expect(&TokenType::CloseParen)?;
        Ok(StatementKind::Animate(AnimateStmt { target, animation, duration }))
    }

    // ─── Events ──────────────────────────────────────────

    fn parse_event_handler(&mut self) -> Result<EventHandler> {
        let event = if let TokenType::Event(name) = self.current_type().clone() {
            self.advance();
            name
        } else {
            return Err(self.error("Expected event handler (on:click, on:submit, etc.)".to_string()));
        };
        let body = self.parse_block()?;
        Ok(EventHandler { event, body })
    }

    // ─── Identifier-led statements ───────────────────────

    fn parse_identifier_statement(&mut self) -> Result<StatementKind> {
        // Could be: assignment, method call, user-defined component, or store access
        let node_mark = self.mark();
        let name = self.expect_identifier()?;

        // Check for dot access (store.method(), object.property = value)
        if self.check(&TokenType::Dot) {
            let mut expr = Expr::Identifier(name);

            while self.match_token(&TokenType::Dot) {
                let prop = self.expect_identifier()?;

                if self.check(&TokenType::OpenParen) {
                    // Method call
                    self.advance();
                    let mut args = Vec::new();
                    while !self.check(&TokenType::CloseParen) {
                        args.push(self.parse_expression()?);
                        if !self.check(&TokenType::CloseParen) {
                            self.expect(&TokenType::Comma)?;
                        }
                    }
                    self.expect(&TokenType::CloseParen)?;
                    expr = Expr::MethodCall(Box::new(expr), prop, args);
                } else {
                    expr = Expr::PropertyAccess(Box::new(expr), prop);
                }
            }

            // Check for assignment
            if self.match_token(&TokenType::Equals) {
                let value = self.parse_expression()?;
                return Ok(StatementKind::Assignment(Assignment { target: expr, value }));
            }

            // It's an expression statement (method call result, etc.)
            return Ok(StatementKind::ExprStatement(expr));
        }

        // Check for index access
        if self.check(&TokenType::OpenBracket) {
            let mut expr = Expr::Identifier(name);
            while self.match_token(&TokenType::OpenBracket) {
                let index = self.parse_expression()?;
                self.expect(&TokenType::CloseBracket)?;
                expr = Expr::IndexAccess(Box::new(expr), Box::new(index));
            }

            if self.match_token(&TokenType::Equals) {
                let value = self.parse_expression()?;
                return Ok(StatementKind::Assignment(Assignment { target: expr, value }));
            }

            return Ok(StatementKind::ExprStatement(expr));
        }

        // Assignment: name = expr
        if self.match_token(&TokenType::Equals) {
            let value = self.parse_expression()?;
            return Ok(StatementKind::Assignment(Assignment {
                target: Expr::Identifier(name.clone()),
                value,
            }));
        }

        // Function call: name(args)
        if self.check(&TokenType::OpenParen) {
            // Could be a user-defined component or a function call
            // Treat uppercase-starting names as components
            if name.chars().next().map_or(false, |c| c.is_uppercase()) {
                // User-defined component with a `( … )` argument group.
                let p = self.parse_paren_args()?;
                let body = self.parse_element_body()?;
                return Ok(StatementKind::UIElement(UIElement {
                    component: ComponentRef::UserDefined(name),
                    args: p.args,
                    modifiers: p.modifiers,
                    children: body.children,
                    style_block: body.style_block,
                    transition_block: body.transition_block,
                    events: body.events,
                    span: self.span_since(node_mark),
                    paren_span: Some(p.paren_span),
                    body_span: body.body_span,
                    style_span: body.style_span,
                    arg_spans: p.arg_spans,
                    modifier_spans: p.modifier_spans,
                }));
            } else {
                // Regular function call
                self.advance(); // skip (
                let mut args = Vec::new();
                while !self.check(&TokenType::CloseParen) {
                    args.push(self.parse_expression()?);
                    if !self.check(&TokenType::CloseParen) {
                        self.expect(&TokenType::Comma)?;
                    }
                }
                self.expect(&TokenType::CloseParen)?;
                return Ok(StatementKind::ExprStatement(Expr::FunctionCall(name, args)));
            }
        }

        // Bare identifier — could be a component usage without parens
        if name.chars().next().map_or(false, |c| c.is_uppercase()) {
            let body = self.parse_element_body()?;
            return Ok(StatementKind::UIElement(UIElement {
                component: ComponentRef::UserDefined(name),
                args: Vec::new(),
                modifiers: Vec::new(),
                children: body.children,
                style_block: body.style_block,
                transition_block: body.transition_block,
                events: body.events,
                span: self.span_since(node_mark),
                paren_span: None,
                body_span: body.body_span,
                style_span: body.style_span,
                arg_spans: Vec::new(),
                modifier_spans: Vec::new(),
            }));
        }

        Ok(StatementKind::ExprStatement(Expr::Identifier(name)))
    }

    // ─── Expressions ─────────────────────────────────────

    pub fn parse_expression(&mut self) -> Result<Expr> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Result<Expr> {
        let mut left = self.parse_and()?;
        while self.match_token(&TokenType::Or) {
            let right = self.parse_and()?;
            left = Expr::BinaryOp(Box::new(left), BinOp::Or, Box::new(right));
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expr> {
        let mut left = self.parse_equality()?;
        while self.match_token(&TokenType::And) {
            let right = self.parse_equality()?;
            left = Expr::BinaryOp(Box::new(left), BinOp::And, Box::new(right));
        }
        Ok(left)
    }

    fn parse_equality(&mut self) -> Result<Expr> {
        let mut left = self.parse_comparison()?;
        loop {
            if self.match_token(&TokenType::DoubleEquals) {
                let right = self.parse_comparison()?;
                left = Expr::BinaryOp(Box::new(left), BinOp::Eq, Box::new(right));
            } else if self.match_token(&TokenType::NotEquals) || self.match_token(&TokenType::StrictNotEqual) {
                let right = self.parse_comparison()?;
                left = Expr::BinaryOp(Box::new(left), BinOp::Neq, Box::new(right));
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expr> {
        let mut left = self.parse_addition()?;
        loop {
            if self.match_token(&TokenType::LessThan) {
                let right = self.parse_addition()?;
                left = Expr::BinaryOp(Box::new(left), BinOp::Lt, Box::new(right));
            } else if self.match_token(&TokenType::GreaterThan) {
                let right = self.parse_addition()?;
                left = Expr::BinaryOp(Box::new(left), BinOp::Gt, Box::new(right));
            } else if self.match_token(&TokenType::LessEquals) {
                let right = self.parse_addition()?;
                left = Expr::BinaryOp(Box::new(left), BinOp::Lte, Box::new(right));
            } else if self.match_token(&TokenType::GreaterEquals) {
                let right = self.parse_addition()?;
                left = Expr::BinaryOp(Box::new(left), BinOp::Gte, Box::new(right));
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_addition(&mut self) -> Result<Expr> {
        let mut left = self.parse_multiplication()?;
        loop {
            if self.match_token(&TokenType::Plus) {
                let right = self.parse_multiplication()?;
                left = Expr::BinaryOp(Box::new(left), BinOp::Add, Box::new(right));
            } else if self.match_token(&TokenType::Minus) {
                let right = self.parse_multiplication()?;
                left = Expr::BinaryOp(Box::new(left), BinOp::Sub, Box::new(right));
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_multiplication(&mut self) -> Result<Expr> {
        let mut left = self.parse_unary()?;
        loop {
            if self.match_token(&TokenType::Star) {
                let right = self.parse_unary()?;
                left = Expr::BinaryOp(Box::new(left), BinOp::Mul, Box::new(right));
            } else if self.match_token(&TokenType::Slash) {
                let right = self.parse_unary()?;
                left = Expr::BinaryOp(Box::new(left), BinOp::Div, Box::new(right));
            } else if self.match_token(&TokenType::Percent) {
                let right = self.parse_unary()?;
                left = Expr::BinaryOp(Box::new(left), BinOp::Mod, Box::new(right));
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr> {
        if self.match_token(&TokenType::Not) {
            let expr = self.parse_unary()?;
            return Ok(Expr::UnaryOp(UnaryOp::Not, Box::new(expr)));
        }
        if self.match_token(&TokenType::Minus) {
            let expr = self.parse_unary()?;
            return Ok(Expr::UnaryOp(UnaryOp::Neg, Box::new(expr)));
        }
        self.parse_postfix()
    }

    fn parse_postfix(&mut self) -> Result<Expr> {
        let mut expr = self.parse_primary()?;

        loop {
            if self.match_token(&TokenType::Dot) {
                let prop = self.expect_identifier()?;
                if self.check(&TokenType::OpenParen) {
                    self.advance();
                    let mut args = Vec::new();
                    while !self.check(&TokenType::CloseParen) {
                        args.push(self.parse_expression()?);
                        if !self.check(&TokenType::CloseParen) {
                            self.expect(&TokenType::Comma)?;
                        }
                    }
                    self.expect(&TokenType::CloseParen)?;
                    expr = Expr::MethodCall(Box::new(expr), prop, args);
                } else {
                    expr = Expr::PropertyAccess(Box::new(expr), prop);
                }
            } else if self.match_token(&TokenType::OpenBracket) {
                let index = self.parse_expression()?;
                self.expect(&TokenType::CloseBracket)?;
                expr = Expr::IndexAccess(Box::new(expr), Box::new(index));
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr> {
        match self.current_type().clone() {
            TokenType::StringLiteral(s) => {
                let s = s.clone();
                self.advance();
                // Check for interpolation: only if { is followed by an identifier char
                if has_interpolation(&s) {
                    let parts = self.parse_interpolated_string(&s)?;
                    Ok(Expr::InterpolatedString(parts))
                } else {
                    Ok(Expr::StringLiteral(s))
                }
            }
            TokenType::NumberLiteral(n) => {
                let n = n;
                self.advance();
                Ok(Expr::NumberLiteral(n))
            }
            TokenType::BoolLiteral(b) => {
                let b = b;
                self.advance();
                Ok(Expr::BoolLiteral(b))
            }
            TokenType::Null => {
                self.advance();
                Ok(Expr::Null)
            }
            TokenType::Identifier(name) => {
                let name = name.clone();
                self.advance();

                // Check for lambda: name => expr
                if self.check(&TokenType::Arrow) {
                    self.advance();
                    let body = self.parse_expression()?;
                    return Ok(Expr::Lambda(name, Box::new(body)));
                }

                // Check for function call
                if self.check(&TokenType::OpenParen) {
                    self.advance();
                    let mut args = Vec::new();
                    while !self.check(&TokenType::CloseParen) {
                        args.push(self.parse_expression()?);
                        if !self.check(&TokenType::CloseParen) {
                            self.expect(&TokenType::Comma)?;
                        }
                    }
                    self.expect(&TokenType::CloseParen)?;
                    return Ok(Expr::FunctionCall(name, args));
                }

                Ok(Expr::Identifier(name))
            }
            TokenType::OpenParen => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect(&TokenType::CloseParen)?;
                Ok(expr)
            }
            TokenType::OpenBracket => {
                self.advance();
                let mut items = Vec::new();
                while !self.check(&TokenType::CloseBracket) {
                    items.push(self.parse_expression()?);
                    if !self.check(&TokenType::CloseBracket) {
                        self.expect(&TokenType::Comma)?;
                    }
                }
                self.expect(&TokenType::CloseBracket)?;
                Ok(Expr::ListLiteral(items))
            }
            TokenType::OpenBrace => {
                self.advance();
                let mut entries = Vec::new();
                while !self.check(&TokenType::CloseBrace) {
                    let key = self.expect_map_key()?;
                    self.expect(&TokenType::Colon)?;
                    let value = self.parse_expression()?;
                    entries.push((key, value));
                    if !self.check(&TokenType::CloseBrace) {
                        self.expect(&TokenType::Comma)?;
                    }
                }
                self.expect(&TokenType::CloseBrace)?;
                Ok(Expr::MapLiteral(entries))
            }
            TokenType::If => {
                // if expression (for derived values)
                self.advance();
                let condition = self.parse_expression()?;
                self.expect(&TokenType::OpenBrace)?;
                let then_expr = self.parse_expression()?;
                self.expect(&TokenType::CloseBrace)?;
                self.expect(&TokenType::Else)?;
                if self.check(&TokenType::If) {
                    let else_expr = self.parse_primary()?;
                    // Wrap in a conditional chain - simplify to nested ternary-like
                    Ok(Expr::MethodCall(
                        Box::new(condition),
                        "__if".to_string(),
                        vec![then_expr, else_expr],
                    ))
                } else {
                    self.expect(&TokenType::OpenBrace)?;
                    let else_expr = self.parse_expression()?;
                    self.expect(&TokenType::CloseBrace)?;
                    Ok(Expr::MethodCall(
                        Box::new(condition),
                        "__if".to_string(),
                        vec![then_expr, else_expr],
                    ))
                }
            }
            // A built-in component name used where a plain name is expected — most
            // importantly `Route(path: "/menu", page: Menu)`, which is the only way
            // to reach a page called `Menu` and used to be a parse error. Component
            // names lex as keyword tokens, which is right in element position and
            // wrong here; in expression position they are just names.
            ref other if crate::lexer::token::component_name(other).is_some() => {
                let name = crate::lexer::token::component_name(other)
                    .expect("guarded by the match arm")
                    .to_string();
                self.advance();
                Ok(Expr::Identifier(name))
            }
            _ => Err(self.error(format!("Expected expression, got {}", self.current_type()))),
        }
    }

    fn parse_interpolated_string(&self, s: &str) -> Result<Vec<StringPart>> {
        let mut parts = Vec::new();
        let mut current = String::new();
        let mut chars = s.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '{' {
                if !current.is_empty() {
                    parts.push(StringPart::Literal(current.clone()));
                    current.clear();
                }
                let mut expr_str = String::new();
                let mut depth = 1;
                while let Some(c) = chars.next() {
                    if c == '{' {
                        depth += 1;
                        expr_str.push(c);
                    } else if c == '}' {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                        expr_str.push(c);
                    } else {
                        expr_str.push(c);
                    }
                }
                // Parse the expression inside { }
                let mut lexer = crate::lexer::Lexer::new(&expr_str, &self.file);
                let tokens = lexer.tokenize().map_err(|e| {
                    WebFluentError::ParseError(Diagnostic::new(
                        format!("Error in string interpolation: {}", e),
                        &self.file, 0, 0,
                    ))
                })?;
                let mut parser = Parser::new(tokens, &self.file);
                let expr = parser.parse_expression()?;
                parts.push(StringPart::Expression(expr));
            } else {
                current.push(ch);
            }
        }

        if !current.is_empty() {
            parts.push(StringPart::Literal(current));
        }

        Ok(parts)
    }

    // ─── Utility ─────────────────────────────────────────

    /// A **declaration name** — like [`Self::expect_identifier`], but a built-in
    /// component name is also accepted.
    ///
    /// `Page Menu`, `Page Card`, `Component Section` are ordinary things to want,
    /// and a page called `Menu` is the likeliest page a restaurant site will ever
    /// have. Component names lex as keyword tokens, so they were rejected here and
    /// (worse) at `Route(page: Menu)`, leaving such a page declarable but
    /// unreachable. Only *declaration* names are widened: element position still
    /// resolves a builtin keyword to the builtin, so `Card { }` inside a page is
    /// unchanged whether or not a page named `Card` exists.
    fn expect_declaration_name(&mut self) -> Result<String> {
        if let Some(name) = crate::lexer::token::component_name(self.current_type()) {
            self.advance();
            return Ok(name.to_string());
        }
        self.expect_identifier()
    }

    fn expect_identifier(&mut self) -> Result<String> {
        match self.current_type().clone() {
            TokenType::Identifier(name) => {
                self.advance();
                Ok(name)
            }
            // Allow some keywords to be used as identifiers in certain contexts
            TokenType::Loading => { self.advance(); Ok("loading".to_string()) }
            TokenType::Error => { self.advance(); Ok("error".to_string()) }
            TokenType::Success => { self.advance(); Ok("success".to_string()) }
            _ => Err(self.error(format!("Expected identifier, got {}", self.current_type()))),
        }
    }

    /// Accept any token that can be a map key: identifiers, string literals, or any keyword.
    fn expect_map_key(&mut self) -> Result<String> {
        // String literal keys
        if let TokenType::StringLiteral(s) = self.current_type().clone() {
            self.advance();
            // Wrap in quotes to distinguish from identifier keys during codegen
            return Ok(format!("\"{}\"", s));
        }
        // Identifier keys
        if let TokenType::Identifier(name) = self.current_type().clone() {
            self.advance();
            return Ok(name);
        }
        // Accept any keyword token as a map key
        let key = match self.current_type() {
            TokenType::State => "state",
            TokenType::Derived => "derived",
            TokenType::Effect => "effect",
            TokenType::Action => "action",
            TokenType::Use => "use",
            TokenType::Fetch => "fetch",
            TokenType::From => "from",
            TokenType::Navigate => "navigate",
            TokenType::Log => "log",
            TokenType::Return => "return",
            TokenType::If => "if",
            TokenType::Else => "else",
            TokenType::For => "for",
            TokenType::In => "in",
            TokenType::Show => "show",
            TokenType::Loading => "loading",
            TokenType::Error => "error",
            TokenType::Success => "success",
            TokenType::Style => "style",
            TokenType::Theme => "Theme",
            TokenType::Token => "token",
            TokenType::Animate => "animate",
            TokenType::Transition => "transition",
            TokenType::Children => "children",
            TokenType::Page => "Page",
            TokenType::Component => "Component",
            TokenType::Store => "Store",
            TokenType::App => "App",
            TokenType::Router => "Router",
            TokenType::Route => "Route",
            TokenType::Null => "null",
            TokenType::TypeString => "String",
            TokenType::TypeNumber => "Number",
            TokenType::TypeBool => "Bool",
            TokenType::TypeList => "List",
            TokenType::TypeMap => "Map",
            // Built-in UI components
            TokenType::Container => "Container",
            TokenType::Row => "Row",
            TokenType::Column => "Column",
            TokenType::Grid => "Grid",
            TokenType::Stack => "Stack",
            TokenType::Spacer => "Spacer",
            TokenType::Divider => "Divider",
            TokenType::Navbar => "Navbar",
            TokenType::Sidebar => "Sidebar",
            TokenType::Breadcrumb => "Breadcrumb",
            TokenType::Link => "Link",
            TokenType::Menu => "Menu",
            TokenType::Tabs => "Tabs",
            TokenType::TabPage => "TabPage",
            TokenType::Card => "Card",
            TokenType::Table => "Table",
            TokenType::Thead => "Thead",
            TokenType::Tbody => "Tbody",
            TokenType::Trow => "Trow",
            TokenType::Tcell => "Tcell",
            TokenType::Badge => "Badge",
            TokenType::Avatar => "Avatar",
            TokenType::Tooltip => "Tooltip",
            TokenType::Tag => "Tag",
            TokenType::Input => "Input",
            TokenType::Select => "Select",
            TokenType::Option => "Option",
            TokenType::Checkbox => "Checkbox",
            TokenType::Radio => "Radio",
            TokenType::Switch => "Switch",
            TokenType::Slider => "Slider",
            TokenType::DatePicker => "DatePicker",
            TokenType::FileUpload => "FileUpload",
            TokenType::Form => "Form",
            TokenType::Alert => "Alert",
            TokenType::Toast => "Toast",
            TokenType::Modal => "Modal",
            TokenType::Dialog => "Dialog",
            TokenType::Spinner => "Spinner",
            TokenType::Progress => "Progress",
            TokenType::Skeleton => "Skeleton",
            TokenType::Button => "Button",
            TokenType::IconButton => "IconButton",
            TokenType::ButtonGroup => "ButtonGroup",
            TokenType::Dropdown => "Dropdown",
            TokenType::Image => "Image",
            TokenType::Video => "Video",
            TokenType::Icon => "Icon",
            TokenType::Carousel => "Carousel",
            TokenType::Text => "Text",
            TokenType::Heading => "Heading",
            TokenType::Code => "Code",
            TokenType::Blockquote => "Blockquote",
            TokenType::Document => "Document",
            TokenType::Section => "Section",
            TokenType::Paragraph => "Paragraph",
            TokenType::PageBreak => "PageBreak",
            TokenType::Header => "Header",
            TokenType::Footer => "Footer",
            TokenType::Presentation => "Presentation",
            TokenType::Slide => "Slide",
            TokenType::TitleSlide => "TitleSlide",
            TokenType::SectionSlide => "SectionSlide",
            TokenType::TwoColumn => "TwoColumn",
            TokenType::ImageSlide => "ImageSlide",
            _ => return Err(self.error(format!("Expected map key (identifier, string, or keyword), got {}", self.current_type()))),
        };
        let result = key.to_string();
        self.advance();
        Ok(result)
    }

    fn expect_string(&mut self) -> Result<String> {
        match self.current_type().clone() {
            TokenType::StringLiteral(s) => {
                self.advance();
                Ok(s)
            }
            _ => Err(self.error(format!("Expected string literal, got {}", self.current_type()))),
        }
    }
}

/// Check if a string contains valid interpolation patterns like {identifier} or {expr}.
/// A valid interpolation: { followed by identifier char, content has no newlines,
/// and closes with } on the same "line segment".
fn has_interpolation(s: &str) -> bool {
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '{' {
            if i + 1 < chars.len() {
                let next = chars[i + 1];
                if next.is_alphabetic() || next == '_' {
                    // Scan to matching } — must not contain \n, :, or ,
                    // (Those indicate map/object literals, not interpolation)
                    let mut j = i + 2;
                    let mut valid = true;
                    while j < chars.len() && chars[j] != '}' {
                        if chars[j] == '\n' || chars[j] == ':' || chars[j] == ',' {
                            valid = false;
                            break;
                        }
                        j += 1;
                    }
                    if valid && j < chars.len() && chars[j] == '}' {
                        return true;
                    }
                }
            }
        }
        i += 1;
    }
    false
}

#[cfg(test)]
mod builtin_name_tests {
    //! A page or component may be NAMED after a built-in, and still be routed to.
    //!
    //! Built-in component names lex as keyword tokens. That is right in element
    //! position and wrong wherever a plain name is expected, so `Page Menu` could
    //! not be declared and `Route(path: "/menu", page: Menu)` could not be written
    //! — leaving the most obvious page a restaurant site could have unreachable.
    use super::*;
    use crate::lexer::Lexer;

    fn parse(src: &str) -> Result<Program> {
        let tokens = Lexer::new(src, "<test>").tokenize().expect("lex failed");
        Parser::new(tokens, "<test>").parse()
    }

    /// Every built-in name works as a page name AND as its route target.
    #[test]
    fn builtin_names_work_as_page_names_and_route_targets() {
        for name in [
            "Menu", "Card", "Table", "Form", "Alert", "Modal", "Link", "Text", "Image", "Icon",
            "Footer", "Badge", "Header", "Section", "Progress", "Tag",
        ] {
            let src = format!(
                "App {{ Router {{ Route(path: \"/x\", page: {name}) }} }}\n\
                 Page {name} (path: \"/x\") {{ Container {{ Text(\"hi\") }} }}\n"
            );
            assert!(parse(&src).is_ok(), "Page {name} should parse: {:?}", parse(&src).err());
        }
    }

    /// The widening must not shadow the built-ins themselves: a page named `Menu`
    /// that also *uses* `Menu` and `Card` still resolves both as elements.
    #[test]
    fn a_builtin_named_page_can_still_use_that_builtin() {
        let src = "Page Menu (path: \"/menu\") {\n  Container {\n    Card(elevated) { Card.Body { Text(\"x\") } }\n    Menu(trigger: \"More\") { }\n  }\n}\n";
        let program = parse(src).expect("should parse");
        // One page, and the elements inside it are the BUILTINS, not references to
        // the page — element position is untouched by the declaration-name change.
        assert_eq!(program.declarations.len(), 1);
    }

    /// Components may be named after built-ins too.
    #[test]
    fn builtin_names_work_as_component_names() {
        assert!(parse("Component Section (title: String) { Container { Text(title) } }\n").is_ok());
        assert!(parse("Component Header (title: String) { Text(title) }\n").is_ok());
    }

    /// Control-flow and declaration keywords stay reserved — the widening covers
    /// component names only.
    #[test]
    fn control_flow_keywords_are_still_reserved() {
        assert!(parse("Page if (path: \"/x\") { Text(\"x\") }").is_err());
        assert!(parse("Page for (path: \"/x\") { Text(\"x\") }").is_err());
        assert!(parse("Page style (path: \"/x\") { Text(\"x\") }").is_err());
    }
}

#[cfg(test)]
mod span_tests {
    //! Slice 1 acceptance: every parsed node carries a source span that slices
    //! back to the exact text it was parsed from, and body spans are the precise
    //! `{ … }` interior. These guard the invariant the edit engine (§1.3) relies on.
    use super::*;
    use crate::lexer::Lexer;

    fn parse(src: &str) -> Program {
        let tokens = Lexer::new(src, "<test>").tokenize().expect("lex failed");
        Parser::new(tokens, "<test>").parse().expect("parse failed")
    }

    /// Recursively collect every `UIElement` in a statement list (pre-order).
    fn collect<'a>(stmts: &'a [Statement], out: &mut Vec<&'a UIElement>) {
        for s in stmts {
            match &s.kind {
                StatementKind::UIElement(ui) => {
                    out.push(ui);
                    collect(&ui.children, out);
                }
                StatementKind::If(i) => {
                    collect(&i.then_body, out);
                    for (_, b) in &i.else_if_branches { collect(b, out); }
                    if let Some(b) = &i.else_body { collect(b, out); }
                }
                StatementKind::For(f) => collect(&f.body, out),
                StatementKind::Show(sh) => collect(&sh.body, out),
                StatementKind::Effect(e) => collect(&e.body, out),
                StatementKind::Action(a) => collect(&a.body, out),
                _ => {}
            }
        }
    }

    fn ui_elements(program: &Program) -> Vec<&UIElement> {
        let mut out = Vec::new();
        for d in &program.declarations {
            match d {
                Declaration::Page(p) => collect(&p.body, &mut out),
                Declaration::Component(c) => collect(&c.body, &mut out),
                Declaration::App(a) => collect(&a.body, &mut out),
                Declaration::Store(_) => {}
            }
        }
        out
    }

    #[test]
    fn ui_element_spans_round_trip() {
        let src = "Page Home (path: \"/\") {\n\
                   \x20 Heading(\"Welcome\", h1)\n\
                   \x20 Button(\"Click me\", primary, large) {\n\
                   \x20   on:click { log(\"hi\") }\n\
                   \x20 }\n\
                   }\n";
        let program = parse(src);
        let uis = ui_elements(&program);

        let heading = uis.iter().find(|u| matches!(&u.component, ComponentRef::BuiltIn(n) if n == "Heading")).unwrap();
        assert_eq!(heading.span.slice(src), "Heading(\"Welcome\", h1)");
        assert_eq!(heading.paren_span.unwrap().slice(src), "(\"Welcome\", h1)");
        assert_eq!(heading.arg_spans[0].slice(src), "\"Welcome\"");
        assert_eq!(heading.modifiers, vec!["h1"]);
        assert_eq!(heading.modifier_spans[0].slice(src), "h1");

        let button = uis.iter().find(|u| matches!(&u.component, ComponentRef::BuiltIn(n) if n == "Button")).unwrap();
        assert!(button.span.slice(src).starts_with("Button("));
        assert!(button.span.slice(src).ends_with('}'));
        assert_eq!(button.arg_spans[0].slice(src), "\"Click me\"");
        assert_eq!(button.modifier_spans[0].slice(src), "primary");
        assert_eq!(button.modifier_spans[1].slice(src), "large");
        // body_span is exactly the `{ … }` interior (braces excluded).
        assert_eq!(button.body_span.unwrap().slice(src).trim(), "on:click { log(\"hi\") }");
    }

    #[test]
    fn decl_spans_round_trip() {
        let src = "Page Home (path: \"/\", title: \"Hi\") {\n  Text(\"x\")\n}\n";
        let program = parse(src);
        let page = match &program.declarations[0] {
            Declaration::Page(p) => p,
            _ => panic!("expected page"),
        };
        assert!(page.span.slice(src).starts_with("Page Home"));
        assert!(page.span.slice(src).ends_with('}'));
        assert_eq!(page.header_span.slice(src), "Page Home (path: \"/\", title: \"Hi\")");
        assert_eq!(page.body_span.slice(src).trim(), "Text(\"x\")");
    }

    #[test]
    fn spans_are_byte_offsets_through_multibyte_text() {
        // The lexer collects `chars()` but must report BYTE offsets — a
        // multibyte prefix would corrupt every later span if it counted chars.
        let src = "Page P (path: \"/\") {\n  Text(\"héllo → café\")\n}\n";
        let program = parse(src);
        let uis = ui_elements(&program);
        let text = uis.iter().find(|u| matches!(&u.component, ComponentRef::BuiltIn(n) if n == "Text")).unwrap();
        assert_eq!(text.arg_spans[0].slice(src), "\"héllo → café\"");
        assert_eq!(text.span.slice(src), "Text(\"héllo → café\")");
    }

    #[test]
    fn statement_spans_round_trip() {
        // Every statement carries a whole-statement span via the Statement wrapper.
        let src = "Page P (path: \"/\") {\n\
                   \x20 state count = 0\n\
                   \x20 Text(\"hello\")\n\
                   \x20 Button(\"Go\", primary)\n\
                   }\n";
        let program = parse(src);
        let body = match &program.declarations[0] {
            Declaration::Page(p) => &p.body,
            _ => panic!("expected page"),
        };
        assert_eq!(body[0].span.slice(src), "state count = 0");
        assert_eq!(body[1].span.slice(src), "Text(\"hello\")");
        assert_eq!(body[2].span.slice(src), "Button(\"Go\", primary)");
        // A statement's span agrees with the element span it wraps.
        if let StatementKind::UIElement(ui) = &body[1].kind {
            assert_eq!(ui.span.slice(src), body[1].span.slice(src));
        } else {
            panic!("expected UIElement statement");
        }
    }
}
