use crate::lexer::{Token, TokenType};
use crate::error::{Diagnostic, WebFluentError, Result};
use super::ast::*;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    file: String,
}

impl Parser {
    pub fn new(tokens: Vec<Token>, file: &str) -> Self {
        Self {
            tokens,
            pos: 0,
            file: file.to_string(),
        }
    }

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
            TokenType::TypeList // List component
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
        self.expect(&TokenType::Page)?;
        let name = self.expect_identifier()?;
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
        let body = self.parse_block()?;

        Ok(PageDecl { name, path, title, guard, redirect, body })
    }

    // ─── Component ───────────────────────────────────────

    fn parse_component_decl(&mut self) -> Result<ComponentDecl> {
        self.expect(&TokenType::Component)?;
        let name = self.expect_identifier()?;
        self.expect(&TokenType::OpenParen)?;

        let mut props = Vec::new();
        while !self.check(&TokenType::CloseParen) {
            props.push(self.parse_prop_decl()?);
            if !self.check(&TokenType::CloseParen) {
                self.expect(&TokenType::Comma)?;
            }
        }

        self.expect(&TokenType::CloseParen)?;
        let body = self.parse_block()?;

        Ok(ComponentDecl { name, props, body })
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
        self.expect(&TokenType::Store)?;
        let name = self.expect_identifier()?;
        let body = self.parse_block()?;
        Ok(StoreDecl { name, body })
    }

    // ─── App ─────────────────────────────────────────────

    fn parse_app(&mut self) -> Result<AppDecl> {
        self.expect(&TokenType::App)?;
        let body = self.parse_block()?;
        Ok(AppDecl { body })
    }

    // ─── Block ───────────────────────────────────────────

    fn parse_block(&mut self) -> Result<Vec<Statement>> {
        self.expect(&TokenType::OpenBrace)?;
        let mut stmts = Vec::new();
        while !self.check(&TokenType::CloseBrace) && !self.is_at_end() {
            stmts.push(self.parse_statement()?);
        }
        self.expect(&TokenType::CloseBrace)?;
        Ok(stmts)
    }

    // ─── Statement ───────────────────────────────────────

    fn parse_statement(&mut self) -> Result<Statement> {
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
            TokenType::Children => {
                self.advance();
                Ok(Statement::UIElement(UIElement {
                    component: ComponentRef::BuiltIn("Children".to_string()),
                    args: Vec::new(),
                    modifiers: Vec::new(),
                    children: Vec::new(),
                    style_block: None,
                    transition_block: None,
                    events: Vec::new(),
                }))
            }
            TokenType::Animate => self.parse_animate_stmt(),
            TokenType::Style => self.parse_style_statement(),
            TokenType::Event(_) => {
                let handler = self.parse_event_handler()?;
                Ok(Statement::EventHandler(handler))
            }
            _ if self.is_builtin_component() => {
                let elem = self.parse_ui_element()?;
                Ok(Statement::UIElement(elem))
            }
            TokenType::Identifier(_) => self.parse_identifier_statement(),
            _ => Err(self.error(format!("Unexpected token {}", self.current_type()))),
        }
    }

    // ─── State declarations ─────────────────────────────

    fn parse_state_decl(&mut self) -> Result<Statement> {
        self.expect(&TokenType::State)?;
        let name = self.expect_identifier()?;
        self.expect(&TokenType::Equals)?;
        let value = self.parse_expression()?;
        Ok(Statement::State(StateDecl { name, value }))
    }

    fn parse_derived_decl(&mut self) -> Result<Statement> {
        self.expect(&TokenType::Derived)?;
        let name = self.expect_identifier()?;
        self.expect(&TokenType::Equals)?;
        let value = self.parse_expression()?;
        Ok(Statement::Derived(DerivedDecl { name, value }))
    }

    fn parse_effect_decl(&mut self) -> Result<Statement> {
        self.expect(&TokenType::Effect)?;
        let body = self.parse_block()?;
        Ok(Statement::Effect(EffectDecl { body }))
    }

    fn parse_action_decl(&mut self) -> Result<Statement> {
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

        Ok(Statement::Action(ActionDecl { name, params, body }))
    }

    fn parse_use_decl(&mut self) -> Result<Statement> {
        self.expect(&TokenType::Use)?;
        let store_name = self.expect_identifier()?;
        Ok(Statement::Use(UseDecl { store_name }))
    }

    fn parse_navigate(&mut self) -> Result<Statement> {
        self.expect(&TokenType::Navigate)?;
        self.expect(&TokenType::OpenParen)?;
        let expr = self.parse_expression()?;
        self.expect(&TokenType::CloseParen)?;
        Ok(Statement::Navigate(expr))
    }

    fn parse_log(&mut self) -> Result<Statement> {
        self.expect(&TokenType::Log)?;
        self.expect(&TokenType::OpenParen)?;
        let expr = self.parse_expression()?;
        self.expect(&TokenType::CloseParen)?;
        Ok(Statement::Log(expr))
    }

    // ─── Control flow ────────────────────────────────────

    fn parse_if_stmt(&mut self) -> Result<Statement> {
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

        Ok(Statement::If(IfStmt {
            condition,
            animate,
            then_body,
            else_if_branches,
            else_body,
        }))
    }

    fn parse_for_stmt(&mut self) -> Result<Statement> {
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

        Ok(Statement::For(ForStmt { item, index, iterable, animate, body }))
    }

    fn parse_show_stmt(&mut self) -> Result<Statement> {
        self.expect(&TokenType::Show)?;
        let condition = self.parse_expression()?;

        // Optional animate clause: , animate(...)
        let animate = self.parse_optional_animate_clause()?;

        let body = self.parse_block()?;
        Ok(Statement::Show(ShowStmt { condition, animate, body }))
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

    fn parse_fetch_decl(&mut self) -> Result<Statement> {
        self.expect(&TokenType::Fetch)?;
        let variable = self.expect_identifier()?;
        self.expect(&TokenType::From)?;
        let url = self.parse_expression()?;

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

        Ok(Statement::Fetch(FetchDecl {
            variable,
            url,
            options,
            loading_block,
            error_block,
            success_block,
        }))
    }

    // ─── UI Elements ─────────────────────────────────────

    fn parse_ui_element(&mut self) -> Result<UIElement> {
        let component = self.parse_component_ref()?;
        let mut args = Vec::new();
        let mut modifiers = Vec::new();

        // Parse arguments in parentheses
        if self.match_token(&TokenType::OpenParen) {
            while !self.check(&TokenType::CloseParen) && !self.is_at_end() {
                // Check if it's a named argument: identifier followed by colon
                if self.is_named_arg() {
                    let name = self.expect_identifier()?;
                    self.expect(&TokenType::Colon)?;
                    let value = self.parse_expression()?;
                    args.push(Arg::Named(name, value));
                } else if self.is_modifier() {
                    let mod_name = self.expect_identifier()?;
                    modifiers.push(mod_name);
                } else {
                    let expr = self.parse_expression()?;
                    args.push(Arg::Positional(expr));
                }
                if !self.check(&TokenType::CloseParen) {
                    self.expect(&TokenType::Comma)?;
                }
            }
            self.expect(&TokenType::CloseParen)?;
        }

        // Parse optional block (children, events, style, transition)
        let mut children = Vec::new();
        let mut style_block = None;
        let mut transition_block = None;
        let mut events = Vec::new();

        if self.check(&TokenType::OpenBrace) {
            self.expect(&TokenType::OpenBrace)?;
            while !self.check(&TokenType::CloseBrace) && !self.is_at_end() {
                match self.current_type() {
                    TokenType::Event(_) => {
                        events.push(self.parse_event_handler()?);
                    }
                    TokenType::Style => {
                        style_block = Some(self.parse_style_block()?);
                    }
                    TokenType::Transition => {
                        transition_block = Some(self.parse_transition_block()?);
                    }
                    _ => {
                        children.push(self.parse_statement()?);
                    }
                }
            }
            self.expect(&TokenType::CloseBrace)?;
        }

        Ok(UIElement {
            component,
            args,
            modifiers,
            children,
            style_block,
            transition_block,
            events,
        })
    }

    fn parse_component_ref(&mut self) -> Result<ComponentRef> {
        let name = self.builtin_name();
        self.advance();

        // Check for sub-component: Navbar.Brand, Card.Header, etc.
        if self.check(&TokenType::Dot) {
            self.advance();
            let sub = self.expect_identifier()?;
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
        if let TokenType::Identifier(name) = self.current_type() {
            matches!(name.as_str(),
                // Size
                "small" | "medium" | "large" |
                // Color
                "primary" | "secondary" | "success" | "danger" | "warning" | "info" |
                // Shape
                "rounded" | "pill" | "square" |
                // Elevation
                "flat" | "elevated" | "outlined" |
                // Width
                "full" | "fit" |
                // Text
                "bold" | "italic" | "underline" | "uppercase" | "lowercase" |
                // Alignment
                "left" | "center" | "right" |
                // Typography
                "heading" | "subtitle" | "muted" |
                // Heading levels
                "h1" | "h2" | "h3" | "h4" | "h5" | "h6" |
                // Other
                "dismissible" | "block" | "bordered" | "controls" | "autoplay" |
                // Input types
                "text" | "email" | "password" | "number" | "search" | "tel" | "url" |
                "date" | "time" | "datetime" | "color" |
                // Button types
                "submit" | "reset" |
                // Animation modifiers
                "fadeIn" | "fadeOut" | "slideUp" | "slideDown" |
                "slideLeft" | "slideRight" | "scaleIn" | "scaleOut" |
                "bounce" | "shake" | "pulse" | "spin" |
                // Animation speed
                "fast" | "slow"
            )
        } else {
            false
        }
    }

    // ─── Style ───────────────────────────────────────────

    fn parse_style_block(&mut self) -> Result<StyleBlock> {
        self.expect(&TokenType::Style)?;
        self.expect(&TokenType::OpenBrace)?;
        let mut properties = Vec::new();
        while !self.check(&TokenType::CloseBrace) {
            let name = self.expect_identifier()?;
            self.expect(&TokenType::Colon)?;
            let value = self.parse_expression()?;
            properties.push(StyleProperty { name, value });
        }
        self.expect(&TokenType::CloseBrace)?;
        Ok(StyleBlock { properties })
    }

    fn parse_style_statement(&mut self) -> Result<Statement> {
        let block = self.parse_style_block()?;
        Ok(Statement::UIElement(UIElement {
            component: ComponentRef::BuiltIn("_StyleBlock".to_string()),
            args: Vec::new(),
            modifiers: Vec::new(),
            children: Vec::new(),
            style_block: Some(block),
            transition_block: None,
            events: Vec::new(),
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

    fn parse_animate_stmt(&mut self) -> Result<Statement> {
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
        Ok(Statement::Animate(AnimateStmt { target, animation, duration }))
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

    fn parse_identifier_statement(&mut self) -> Result<Statement> {
        // Could be: assignment, method call, user-defined component, or store access
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
                return Ok(Statement::Assignment(Assignment { target: expr, value }));
            }

            // It's an expression statement (method call result, etc.)
            return Ok(Statement::ExprStatement(expr));
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
                return Ok(Statement::Assignment(Assignment { target: expr, value }));
            }

            return Ok(Statement::ExprStatement(expr));
        }

        // Assignment: name = expr
        if self.match_token(&TokenType::Equals) {
            let value = self.parse_expression()?;
            return Ok(Statement::Assignment(Assignment {
                target: Expr::Identifier(name.clone()),
                value,
            }));
        }

        // Function call: name(args)
        if self.check(&TokenType::OpenParen) {
            // Could be a user-defined component or a function call
            // Treat uppercase-starting names as components
            if name.chars().next().map_or(false, |c| c.is_uppercase()) {
                // User-defined component
                let mut args = Vec::new();
                let mut modifiers = Vec::new();
                self.advance(); // skip (

                while !self.check(&TokenType::CloseParen) && !self.is_at_end() {
                    if self.is_named_arg() {
                        let arg_name = self.expect_identifier()?;
                        self.expect(&TokenType::Colon)?;
                        let value = self.parse_expression()?;
                        args.push(Arg::Named(arg_name, value));
                    } else if self.is_modifier() {
                        let mod_name = self.expect_identifier()?;
                        modifiers.push(mod_name);
                    } else {
                        let expr = self.parse_expression()?;
                        args.push(Arg::Positional(expr));
                    }
                    if !self.check(&TokenType::CloseParen) {
                        self.expect(&TokenType::Comma)?;
                    }
                }
                self.expect(&TokenType::CloseParen)?;

                let mut children = Vec::new();
                let mut style_block = None;
                let mut transition_block = None;
                let mut events = Vec::new();

                if self.check(&TokenType::OpenBrace) {
                    self.expect(&TokenType::OpenBrace)?;
                    while !self.check(&TokenType::CloseBrace) && !self.is_at_end() {
                        match self.current_type() {
                            TokenType::Event(_) => events.push(self.parse_event_handler()?),
                            TokenType::Style => style_block = Some(self.parse_style_block()?),
                            TokenType::Transition => transition_block = Some(self.parse_transition_block()?),
                            _ => children.push(self.parse_statement()?),
                        }
                    }
                    self.expect(&TokenType::CloseBrace)?;
                }

                return Ok(Statement::UIElement(UIElement {
                    component: ComponentRef::UserDefined(name),
                    args,
                    modifiers,
                    children,
                    style_block,
                    transition_block,
                    events,
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
                return Ok(Statement::ExprStatement(Expr::FunctionCall(name, args)));
            }
        }

        // Bare identifier — could be a component usage without parens
        if name.chars().next().map_or(false, |c| c.is_uppercase()) {
            let mut children = Vec::new();
            let mut events = Vec::new();
            let mut style_block = None;
            let mut transition_block = None;

            if self.check(&TokenType::OpenBrace) {
                self.expect(&TokenType::OpenBrace)?;
                while !self.check(&TokenType::CloseBrace) && !self.is_at_end() {
                    match self.current_type() {
                        TokenType::Event(_) => events.push(self.parse_event_handler()?),
                        TokenType::Style => style_block = Some(self.parse_style_block()?),
                        TokenType::Transition => transition_block = Some(self.parse_transition_block()?),
                        _ => children.push(self.parse_statement()?),
                    }
                }
                self.expect(&TokenType::CloseBrace)?;
            }

            return Ok(Statement::UIElement(UIElement {
                component: ComponentRef::UserDefined(name),
                args: Vec::new(),
                modifiers: Vec::new(),
                children,
                style_block,
                transition_block,
                events,
            }));
        }

        Ok(Statement::ExprStatement(Expr::Identifier(name)))
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
            } else if self.match_token(&TokenType::NotEquals) {
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
                    let key = self.expect_identifier()?;
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
