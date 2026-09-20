//! The lexer for WebFluent 3 (`spec/SYNTAX_V2.md`).
//!
//! It differs from the original lexer in what it *does not* do: no word is
//! a keyword here. `page`, `state`, `if`, `Button` are all identifiers, and
//! the parser reserves words by position, so `type:` is an ordinary prop
//! name and `Page` an ordinary page name. What it adds:
//!
//! - `$name` is a design token ([`TokenType::DesignToken`]);
//! - `??` is [`TokenType::NullCoalesce`], `;` a separator, `///` a
//!   [`TokenType::DocComment`];
//! - inside a `style { }`, `transition { }` or `theme X { }` block the text
//!   is CSS, not WebFluent: a declaration is a [`TokenType::StyleProp`]
//!   name and a [`TokenType::RawValue`] that runs to the end of the line or
//!   a `;`, a nested rule is a [`TokenType::RawSelector`] before its `{`.
//!   The parser reads `{expr}` splices and `$token`s out of the raw value.
//!
//! A `.wfx` file is the same grammar with its blocks written by
//! indentation ([`Layout::Offside`]): a line whose next line is indented
//! deeper opens a block, and a dedent closes as many as it leaves. The
//! lexer emits the `{` and `}` the parser expects, zero-width, so the
//! parser is the same. Inside parentheses, brackets and braces the writer
//! wrote, layout is free, as it is in a `.wf` file.
//!
//! Spans are byte offsets, lines and columns count from 1, columns in
//! characters — the same three coordinate systems as the original lexer.

use super::token::{Token, TokenType};
use crate::error::{Diagnostic, Result, WebFluentError};

/// How a file writes its blocks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layout {
    /// `{ … }`, as a `.wf` file has them.
    Braces,
    /// By indentation, as a `.wfx` file has them; `{ … }` still allowed.
    Offside,
}

impl Layout {
    /// The layout a file's name asks for: `.wfx` is offside.
    pub fn of_file(file: &str) -> Layout {
        if file.ends_with(".wfx") {
            Layout::Offside
        } else {
            Layout::Braces
        }
    }
}

pub struct LexerV2 {
    source: Vec<char>,
    pos: usize,
    byte_pos: usize,
    line: usize,
    column: usize,
    file: String,
    /// Brace depth, counting every `{` and `}` seen.
    depth: usize,
    /// The depth at which the innermost style block was opened, while
    /// inside one.
    style_from: Option<usize>,
    /// Whether the next `{` opens a style block.
    style_pending: bool,
    /// Whether a declaration's name was just emitted and its raw value is
    /// the next thing to read.
    pending_value: bool,
    layout: Layout,
    /// Offside: the indentation of each open block, innermost last.
    indents: Vec<String>,
    /// Offside: unclosed `(` and `[` — layout is free inside them.
    parens: usize,
    /// Offside: unclosed `{` the writer wrote — layout is free inside them.
    explicit: usize,
}

impl LexerV2 {
    pub fn new(source: &str, file: &str) -> Self {
        Self::with_layout(source, file, Layout::Braces)
    }

    /// A lexer for the layout the file's name asks for.
    pub fn for_file(source: &str, file: &str) -> Self {
        Self::with_layout(source, file, Layout::of_file(file))
    }

    pub fn with_layout(source: &str, file: &str, layout: Layout) -> Self {
        Self {
            source: source.chars().collect(),
            pos: 0,
            byte_pos: 0,
            line: 1,
            column: 1,
            file: file.to_string(),
            depth: 0,
            style_from: None,
            style_pending: false,
            pending_value: false,
            layout,
            indents: Vec::new(),
            parens: 0,
            explicit: 0,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>> {
        let mut tokens: Vec<Token> = Vec::new();
        loop {
            self.skip_space_and_comments(&mut tokens)?;
            if self.pos >= self.source.len() {
                break;
            }
            let start_byte = self.byte_pos;
            let mut token = if self.style_from.is_some() {
                self.style_token()?
            } else {
                self.normal_token(&tokens)?
            };
            token.offset = start_byte;
            token.end = self.byte_pos;
            self.track_layout(&token);
            self.track_braces(&token, &tokens);
            tokens.push(token);
        }
        // Every block still open closes at the end of the file.
        while self.indents.pop().is_some() {
            self.synthetic(TokenType::CloseBrace, &mut tokens);
        }
        let mut eof = Token::new(TokenType::EOF, self.line, self.column);
        eof.offset = self.byte_pos;
        eof.end = self.byte_pos;
        tokens.push(eof);
        Ok(tokens)
    }

    /// Count what suspends the offside rule: the writer's own parentheses,
    /// brackets and braces.
    fn track_layout(&mut self, token: &Token) {
        if self.layout != Layout::Offside {
            return;
        }
        match &token.token_type {
            TokenType::OpenParen | TokenType::OpenBracket => self.parens += 1,
            TokenType::CloseParen | TokenType::CloseBracket => {
                self.parens = self.parens.saturating_sub(1)
            }
            TokenType::OpenBrace => self.explicit += 1,
            TokenType::CloseBrace => self.explicit = self.explicit.saturating_sub(1),
            _ => {}
        }
    }

    /// A brace the layout implies: zero width, at the end of the last token
    /// (the end of the line that opened the block, or of the block's last
    /// line).
    fn synthetic(&mut self, kind: TokenType, tokens: &mut Vec<Token>) {
        let (line, column, at) = match tokens.last() {
            Some(t) => (t.line, t.column + self.source_width(t), t.end),
            None => (1, 1, 0),
        };
        let mut token = Token::new(kind, line, column);
        token.offset = at;
        token.end = at;
        self.track_braces(&token, tokens);
        tokens.push(token);
    }

    /// The width of a token in characters, for the column after it.
    fn source_width(&self, token: &Token) -> usize {
        let mut chars = 0;
        let mut bytes = 0;
        for ch in &self.source {
            if bytes >= token.end {
                break;
            }
            if bytes >= token.offset {
                chars += 1;
            }
            bytes += ch.len_utf8();
        }
        chars
    }

    /// Offside: a line of code starts here, indented by `indent`. Deeper
    /// than the block around it opens a block; shallower closes every block
    /// it leaves; the same continues it.
    fn line_layout(&mut self, indent: &str, tokens: &mut Vec<Token>) -> Result<()> {
        if self.parens > 0 || self.explicit > 0 {
            return Ok(());
        }
        let current = self.indents.last().cloned().unwrap_or_default();
        if indent == current {
            return Ok(());
        }
        if indent.starts_with(&current) {
            if tokens.is_empty() {
                return Err(self.error("The first line of a file is not indented"));
            }
            self.indents.push(indent.to_string());
            self.synthetic(TokenType::OpenBrace, tokens);
            return Ok(());
        }
        loop {
            let current = self.indents.last().cloned().unwrap_or_default();
            if indent == current {
                return Ok(());
            }
            if current.starts_with(indent) && self.indents.pop().is_some() {
                self.synthetic(TokenType::CloseBrace, tokens);
                continue;
            }
            return Err(WebFluentError::LexerError(
                Diagnostic::new(
                    "This line's indentation matches no block around it",
                    &self.file,
                    self.line,
                    self.column,
                )
                .with_hint("Indent it to the block it belongs to, with the same spaces or tabs as the lines around it"),
            ));
        }
    }

    /// Enter and leave style mode on braces: a `{` after `style`,
    /// `transition` or `theme Name` opens one; the `}` that balances it
    /// closes it.
    fn track_braces(&mut self, token: &Token, before: &[Token]) {
        match &token.token_type {
            TokenType::OpenBrace => {
                self.depth += 1;
                if self.style_from.is_none() {
                    let opens_style = self.style_pending || opens_style_block(before);
                    if opens_style {
                        self.style_from = Some(self.depth);
                    }
                }
                self.style_pending = false;
            }
            TokenType::CloseBrace => {
                if let Some(from) = self.style_from
                    && self.depth == from
                {
                    self.style_from = None;
                }
                self.depth = self.depth.saturating_sub(1);
            }
            TokenType::RawSelector(_) => self.style_pending = true,
            _ => {}
        }
    }

    // ─── Normal mode ─────────────────────────────────────

    fn normal_token(&mut self, before: &[Token]) -> Result<Token> {
        let ch = self.current();
        let (line, column) = (self.line, self.column);
        let single = |lexer: &mut Self, t: TokenType| {
            lexer.advance();
            Token::new(t, line, column)
        };
        Ok(match ch {
            '"' => self.read_string()?,
            '0'..='9' => self.read_number()?,
            'a'..='z' | 'A'..='Z' | '_' => self.read_word(),
            '$' => self.read_token_name()?,
            '+' => single(self, TokenType::Plus),
            '-' => single(self, TokenType::Minus),
            '*' => single(self, TokenType::Star),
            '%' => single(self, TokenType::Percent),
            '/' => single(self, TokenType::Slash),
            '=' => match self.peek() {
                Some('=') => self.two(TokenType::DoubleEquals),
                Some('>') => self.two(TokenType::Arrow),
                _ => single(self, TokenType::Equals),
            },
            '!' => {
                if self.peek() == Some('=') {
                    self.advance();
                    self.advance();
                    if self.pos < self.source.len() && self.current() == '=' {
                        self.advance();
                        Token::new(TokenType::StrictNotEqual, line, column)
                    } else {
                        Token::new(TokenType::NotEquals, line, column)
                    }
                } else {
                    single(self, TokenType::Not)
                }
            }
            '<' => match self.peek() {
                Some('=') => self.two(TokenType::LessEquals),
                _ => single(self, TokenType::LessThan),
            },
            '>' => match self.peek() {
                Some('=') => self.two(TokenType::GreaterEquals),
                _ => single(self, TokenType::GreaterThan),
            },
            '&' => match self.peek() {
                Some('&') => self.two(TokenType::And),
                _ => single(self, TokenType::BitwiseAnd),
            },
            '|' => match self.peek() {
                Some('|') => self.two(TokenType::Or),
                _ => return Err(self.error("Unexpected character '|', did you mean '||'?")),
            },
            '?' => match self.peek() {
                Some('?') => self.two(TokenType::NullCoalesce),
                _ => single(self, TokenType::QuestionMark),
            },
            '(' => single(self, TokenType::OpenParen),
            ')' => single(self, TokenType::CloseParen),
            '{' => single(self, TokenType::OpenBrace),
            '}' => single(self, TokenType::CloseBrace),
            '[' => single(self, TokenType::OpenBracket),
            ']' => single(self, TokenType::CloseBracket),
            ':' => single(self, TokenType::Colon),
            ',' => single(self, TokenType::Comma),
            '.' => single(self, TokenType::Dot),
            ';' => single(self, TokenType::Semicolon),
            _ => {
                let hint = if before.is_empty() {
                    ""
                } else {
                    " (a CSS value belongs inside a style { } block)"
                };
                return Err(self.error(&format!("Unexpected character '{ch}'{hint}")));
            }
        })
    }

    fn two(&mut self, t: TokenType) -> Token {
        let token = Token::new(t, self.line, self.column);
        self.advance();
        self.advance();
        token
    }

    /// A word: an identifier, or one of the three literal words.
    fn read_word(&mut self) -> Token {
        let (line, column) = (self.line, self.column);
        let mut word = String::new();
        while self.pos < self.source.len() {
            let ch = self.current();
            if ch.is_alphanumeric() || ch == '_' {
                word.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        let kind = match word.as_str() {
            "true" => TokenType::BoolLiteral(true),
            "false" => TokenType::BoolLiteral(false),
            "null" => TokenType::Null,
            _ => TokenType::Identifier(word),
        };
        Token::new(kind, line, column)
    }

    /// `$name`, hyphens included: `$surface-hover`.
    fn read_token_name(&mut self) -> Result<Token> {
        let (line, column) = (self.line, self.column);
        self.advance(); // $
        let mut name = String::new();
        while self.pos < self.source.len() {
            let ch = self.current();
            if ch.is_alphanumeric() || ch == '_' || ch == '-' {
                name.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        if name.is_empty() {
            return Err(WebFluentError::LexerError(Diagnostic::new(
                "Expected a design token name after `$`, like `$color-primary`",
                &self.file,
                line,
                column,
            )));
        }
        Ok(Token::new(TokenType::DesignToken(name), line, column))
    }

    // ─── Style mode ──────────────────────────────────────

    fn style_token(&mut self) -> Result<Token> {
        let ch = self.current();
        let (line, column) = (self.line, self.column);
        match ch {
            '}' => {
                self.advance();
                Ok(Token::new(TokenType::CloseBrace, line, column))
            }
            '{' => {
                self.advance();
                Ok(Token::new(TokenType::OpenBrace, line, column))
            }
            ';' => {
                self.advance();
                Ok(Token::new(TokenType::Semicolon, line, column))
            }
            '&' | '@' | '.' | ':' | '[' | '>' | '*' | '+' | '~' => {
                // A nested rule: its selector, up to the `{`.
                let mut text = String::new();
                while self.pos < self.source.len()
                    && self.current() != '{'
                    && self.current() != '\n'
                {
                    text.push(self.current());
                    self.advance();
                }
                // Offside: the rule's block is the indented lines below.
                let offside_block = self.layout == Layout::Offside
                    && (self.pos >= self.source.len() || self.current() == '\n');
                if !offside_block && (self.pos >= self.source.len() || self.current() != '{') {
                    return Err(WebFluentError::LexerError(Diagnostic::new(
                        format!("Expected `{{` after the selector `{}`", text.trim()),
                        &self.file,
                        line,
                        column,
                    )));
                }
                Ok(Token::new(
                    TokenType::RawSelector(text.trim().to_string()),
                    line,
                    column,
                ))
            }
            _ => self.style_declaration(),
        }
    }

    /// `name: value` — the name as a token, then the value as raw text to
    /// the end of the line, a `;`, or the closing brace, whichever comes
    /// first outside parentheses, braces and quotes. A line that ends with
    /// a comma, or inside parentheses, continues on the next.
    fn style_declaration(&mut self) -> Result<Token> {
        let (line, column) = (self.line, self.column);
        let mut name = String::new();
        while self.pos < self.source.len() {
            let ch = self.current();
            if ch.is_alphanumeric() || ch == '_' || ch == '-' {
                name.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        if name.is_empty() {
            return Err(WebFluentError::LexerError(Diagnostic::new(
                format!(
                    "Unexpected `{}` in a style block; expected `property: value` or a nested `&:state {{ }}` rule",
                    self.current()
                ),
                &self.file,
                line,
                column,
            )));
        }
        // The name token; the value follows as its own token.
        self.skip_inline_space();
        if self.pos >= self.source.len() || self.current() != ':' {
            return Err(WebFluentError::LexerError(Diagnostic::new(
                format!("Expected `:` after the property `{name}`"),
                &self.file,
                line,
                column,
            )));
        }
        // Emit the property now; the value is read by the next call.
        self.pending_value = true;
        self.advance(); // `:`
        Ok(Token::new(TokenType::StyleProp(name), line, column))
    }

    fn skip_inline_space(&mut self) {
        while self.pos < self.source.len() && matches!(self.current(), ' ' | '\t') {
            self.advance();
        }
    }

    // ─── Shared pieces ───────────────────────────────────

    fn read_string(&mut self) -> Result<Token> {
        let (line, column) = (self.line, self.column);
        self.advance();
        let mut value = String::new();
        while self.pos < self.source.len() && self.current() != '"' {
            if self.current() == '\\' {
                self.advance();
                if self.pos >= self.source.len() {
                    break;
                }
                match self.current() {
                    'n' => value.push('\n'),
                    't' => value.push('\t'),
                    'r' => value.push('\r'),
                    '\\' => value.push('\\'),
                    '"' => value.push('"'),
                    '{' => value.push('\u{FFFE}'),
                    '}' => value.push('\u{FFFF}'),
                    c => {
                        value.push('\\');
                        value.push(c);
                    }
                }
            } else {
                value.push(self.current());
            }
            self.advance();
        }
        if self.pos >= self.source.len() {
            return Err(WebFluentError::LexerError(Diagnostic::new(
                "Unterminated string literal",
                &self.file,
                line,
                column,
            )));
        }
        self.advance();
        Ok(Token::new(TokenType::StringLiteral(value), line, column))
    }

    fn read_number(&mut self) -> Result<Token> {
        let (line, column) = (self.line, self.column);
        let mut text = String::new();
        while self.pos < self.source.len() && self.current().is_ascii_digit() {
            text.push(self.current());
            self.advance();
        }
        if self.pos < self.source.len()
            && self.current() == '.'
            && self.peek().is_some_and(|c| c.is_ascii_digit())
        {
            text.push('.');
            self.advance();
            while self.pos < self.source.len() && self.current().is_ascii_digit() {
                text.push(self.current());
                self.advance();
            }
        }
        let value: f64 = text.parse().map_err(|_| {
            WebFluentError::LexerError(Diagnostic::new(
                format!("Invalid number `{text}`"),
                &self.file,
                line,
                column,
            ))
        })?;
        Ok(Token::new(TokenType::NumberLiteral(value), line, column))
    }

    /// Whitespace and comments. A `///` comment is a token; a `//` or
    /// `/* */` comment is dropped. Inside a style block a `//` counts as a
    /// comment only at the start of a line (after indentation), so a URL's
    /// `//` survives in a value.
    fn skip_space_and_comments(&mut self, tokens: &mut Vec<Token>) -> Result<()> {
        loop {
            if self.style_from.is_some() && self.pending_value {
                // The value of a declaration whose name was just emitted.
                self.pending_value = false;
                self.skip_inline_space();
                let start_byte = self.byte_pos;
                let (line, column) = (self.line, self.column);
                let text = self.read_raw_value();
                let mut token = Token::new(TokenType::RawValue(text), line, column);
                token.offset = start_byte;
                token.end = self.byte_pos;
                tokens.push(token);
                continue;
            }
            let mut new_line = self.pos == 0 && self.layout == Layout::Offside;
            while self.pos < self.source.len() && self.current().is_whitespace() {
                if self.current() == '\n' {
                    new_line = self.layout == Layout::Offside;
                }
                self.advance();
            }
            if self.pos >= self.source.len() {
                return Ok(());
            }
            if new_line && !self.at_comment() {
                // The indentation of this line: from the line's start to here.
                let line_start = self.source[..self.pos]
                    .iter()
                    .rposition(|c| *c == '\n')
                    .map_or(0, |i| i + 1);
                let indent: String = self.source[line_start..self.pos].iter().collect();
                self.line_layout(&indent, tokens)?;
            }
            if self.current() == '/' && self.peek() == Some('/') {
                let third = self.source.get(self.pos + 2).copied();
                if third == Some('/') && self.style_from.is_none() {
                    let start_byte = self.byte_pos;
                    let (line, column) = (self.line, self.column);
                    self.advance();
                    self.advance();
                    self.advance();
                    let mut text = String::new();
                    while self.pos < self.source.len() && self.current() != '\n' {
                        text.push(self.current());
                        self.advance();
                    }
                    let mut token =
                        Token::new(TokenType::DocComment(text.trim().to_string()), line, column);
                    token.offset = start_byte;
                    token.end = self.byte_pos;
                    tokens.push(token);
                    continue;
                }
                while self.pos < self.source.len() && self.current() != '\n' {
                    self.advance();
                }
                continue;
            }
            if self.current() == '/' && self.peek() == Some('*') {
                let (line, column) = (self.line, self.column);
                self.advance();
                self.advance();
                loop {
                    if self.pos >= self.source.len() {
                        return Err(WebFluentError::LexerError(Diagnostic::new(
                            "Unterminated block comment",
                            &self.file,
                            line,
                            column,
                        )));
                    }
                    if self.current() == '*' && self.peek() == Some('/') {
                        self.advance();
                        self.advance();
                        break;
                    }
                    self.advance();
                }
                continue;
            }
            return Ok(());
        }
    }

    /// The raw text of a style value: to the end of the line, a `;`, or the
    /// block's `}` — outside parentheses, `{ }` splices and quotes — with a
    /// line that ends in `,` or leaves a parenthesis open continuing.
    fn read_raw_value(&mut self) -> String {
        let mut text = String::new();
        let mut parens = 0usize;
        let mut braces = 0usize;
        let mut quote: Option<char> = None;
        while self.pos < self.source.len() {
            let ch = self.current();
            if let Some(q) = quote {
                text.push(ch);
                self.advance();
                if ch == q {
                    quote = None;
                }
                continue;
            }
            match ch {
                '"' | '\'' => {
                    quote = Some(ch);
                    text.push(ch);
                    self.advance();
                }
                '(' => {
                    parens += 1;
                    text.push(ch);
                    self.advance();
                }
                ')' => {
                    parens = parens.saturating_sub(1);
                    text.push(ch);
                    self.advance();
                }
                '{' => {
                    braces += 1;
                    text.push(ch);
                    self.advance();
                }
                '}' if braces > 0 => {
                    braces -= 1;
                    text.push(ch);
                    self.advance();
                }
                '}' => break,
                ';' if parens == 0 && braces == 0 => break,
                '\n' => {
                    let trimmed = text.trim_end();
                    if parens > 0 || braces > 0 || trimmed.ends_with(',') {
                        text.push(' ');
                        self.advance();
                        // Indentation of the continued line is one space.
                        self.skip_inline_space();
                    } else {
                        break;
                    }
                }
                '/' if parens == 0 && self.peek() == Some('/') && text.ends_with(' ') => {
                    // A trailing comment after the value.
                    break;
                }
                _ => {
                    text.push(ch);
                    self.advance();
                }
            }
        }
        text.trim().to_string()
    }

    /// Whether a `//` or `/* */` comment — not a `///` doc — starts here.
    fn at_comment(&self) -> bool {
        self.current() == '/'
            && (self.peek() == Some('*')
                || (self.peek() == Some('/') && self.source.get(self.pos + 2) != Some(&'/')))
    }

    fn current(&self) -> char {
        self.source[self.pos]
    }

    fn peek(&self) -> Option<char> {
        self.source.get(self.pos + 1).copied()
    }

    fn advance(&mut self) {
        if self.pos < self.source.len() {
            let ch = self.source[self.pos];
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
            self.byte_pos += ch.len_utf8();
            self.pos += 1;
        }
    }

    fn error(&self, message: &str) -> WebFluentError {
        WebFluentError::LexerError(Diagnostic::new(message, &self.file, self.line, self.column))
    }
}

/// Whether the `{` about to be emitted opens a style block: it follows
/// `style`, `transition`, or `theme Name`.
fn opens_style_block(before: &[Token]) -> bool {
    match before {
        [
            ..,
            Token {
                token_type: TokenType::Identifier(word),
                ..
            },
        ] if word == "style" || word == "transition" => true,
        [
            ..,
            Token {
                token_type: TokenType::Identifier(keyword),
                ..
            },
            Token {
                token_type: TokenType::Identifier(_),
                ..
            },
        ] if keyword == "theme" => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(src: &str) -> Vec<TokenType> {
        LexerV2::new(src, "<t>")
            .tokenize()
            .expect("lex")
            .into_iter()
            .map(|t| t.token_type)
            .filter(|t| !matches!(t, TokenType::EOF))
            .collect()
    }

    fn ident(s: &str) -> TokenType {
        TokenType::Identifier(s.to_string())
    }

    #[test]
    fn no_word_is_a_keyword() {
        assert_eq!(
            kinds("page state Button type true null"),
            vec![
                ident("page"),
                ident("state"),
                ident("Button"),
                ident("type"),
                TokenType::BoolLiteral(true),
                TokenType::Null
            ]
        );
    }

    #[test]
    fn tokens_coalesce_semicolons_and_doc_comments_are_lexed() {
        assert_eq!(
            kinds("$surface-hover ?? x; /// the doc\n// not a doc\ny"),
            vec![
                TokenType::DesignToken("surface-hover".to_string()),
                TokenType::NullCoalesce,
                ident("x"),
                TokenType::Semicolon,
                TokenType::DocComment("the doc".to_string()),
                ident("y"),
            ]
        );
    }

    #[test]
    fn a_style_block_is_css() {
        let src = "style {\n  padding: 6px 0\n  --accent: {todo.color}px\n  background: $surface; color: red\n  font: 500 13px/1.3 \"Manrope\", sans-serif\n  &:hover { background: $surface-hover }\n  @media (max-width: 768px) {\n    gap: 4px\n  }\n  transition: background 150ms,\n    color 150ms\n}\nButton(\"x\")";
        let k = kinds(src);
        let expected = vec![
            ident("style"),
            TokenType::OpenBrace,
            TokenType::StyleProp("padding".into()),
            TokenType::RawValue("6px 0".into()),
            TokenType::StyleProp("--accent".into()),
            TokenType::RawValue("{todo.color}px".into()),
            TokenType::StyleProp("background".into()),
            TokenType::RawValue("$surface".into()),
            TokenType::Semicolon,
            TokenType::StyleProp("color".into()),
            TokenType::RawValue("red".into()),
            TokenType::StyleProp("font".into()),
            TokenType::RawValue("500 13px/1.3 \"Manrope\", sans-serif".into()),
            TokenType::RawSelector("&:hover".into()),
            TokenType::OpenBrace,
            TokenType::StyleProp("background".into()),
            TokenType::RawValue("$surface-hover".into()),
            TokenType::CloseBrace,
            TokenType::RawSelector("@media (max-width: 768px)".into()),
            TokenType::OpenBrace,
            TokenType::StyleProp("gap".into()),
            TokenType::RawValue("4px".into()),
            TokenType::CloseBrace,
            TokenType::StyleProp("transition".into()),
            TokenType::RawValue("background 150ms, color 150ms".into()),
            TokenType::CloseBrace,
            ident("Button"),
            TokenType::OpenParen,
            TokenType::StringLiteral("x".into()),
            TokenType::CloseParen,
        ];
        assert_eq!(k, expected);
    }

    #[test]
    fn a_url_in_a_value_keeps_its_slashes_and_a_theme_is_a_style_block() {
        let k = kinds(
            "theme Brand {\n  hero: url(https://x.y/a.png) // the hero\n  color-primary: #8B5CF6\n}",
        );
        assert_eq!(
            k,
            vec![
                ident("theme"),
                ident("Brand"),
                TokenType::OpenBrace,
                TokenType::StyleProp("hero".into()),
                TokenType::RawValue("url(https://x.y/a.png)".into()),
                TokenType::StyleProp("color-primary".into()),
                TokenType::RawValue("#8B5CF6".into()),
                TokenType::CloseBrace,
            ]
        );
    }

    #[test]
    fn spans_are_byte_offsets() {
        let src = "Text(\"héllo\") $tok";
        let tokens = LexerV2::new(src, "<t>").tokenize().unwrap();
        let text = &tokens[2];
        assert_eq!(&src[text.offset..text.end], "\"héllo\"");
        let tok = &tokens[4];
        assert_eq!(&src[tok.offset..tok.end], "$tok");
    }

    fn offside(src: &str) -> Vec<TokenType> {
        LexerV2::with_layout(src, "<t>", Layout::Offside)
            .tokenize()
            .expect("lex")
            .into_iter()
            .map(|t| t.token_type)
            .filter(|t| !matches!(t, TokenType::EOF))
            .collect()
    }

    #[test]
    fn indentation_writes_the_braces() {
        let wfx = "page Home(path: \"/\")\n    state open = true\n    Row(gap: .sm)\n        style\n            padding: 6px 0\n            &:hover\n                background: $surface-hover\n        on click\n            open = !open\n        Text(\"a\").bold\n    if open\n        Spinner\n    else\n        Text(\"b\")\n\n    // a comment at any indent\n  // another\n    Router\napp\n    Router\n";
        let wf = "page Home(path: \"/\") {\n    state open = true\n    Row(gap: .sm) {\n        style {\n            padding: 6px 0\n            &:hover {\n                background: $surface-hover\n            }\n        }\n        on click {\n            open = !open\n        }\n        Text(\"a\").bold\n    }\n    if open {\n        Spinner\n    } else {\n        Text(\"b\")\n    }\n    Router\n}\napp {\n    Router\n}\n";
        assert_eq!(offside(wfx), kinds(wf));
    }

    #[test]
    fn layout_is_free_inside_parentheses_and_written_braces() {
        let wfx = "page P(\n    path: \"/\",\n    title: \"x\"\n)\n    state user = {\n        name: \"a\",\n        tags: [\n            1\n        ]\n    }\n    Button(\"x\") { on click { save() } }\n    Text(user.name)\n";
        let wf = "page P(path: \"/\", title: \"x\") {\n    state user = { name: \"a\", tags: [1] }\n    Button(\"x\") { on click { save() } }\n    Text(user.name)\n}\n";
        assert_eq!(offside(wfx), kinds(wf));
    }

    #[test]
    fn a_synthetic_brace_sits_at_the_end_of_its_line() {
        let tokens = LexerV2::with_layout(
            "page P(path: \"/\")\n    Text(\"a\")\n",
            "<t>",
            Layout::Offside,
        )
        .tokenize()
        .unwrap();
        let open = tokens
            .iter()
            .find(|t| t.token_type == TokenType::OpenBrace)
            .unwrap();
        assert_eq!(
            (open.line, open.column, open.offset, open.end),
            (1, 18, 17, 17)
        );
        let close = tokens
            .iter()
            .find(|t| t.token_type == TokenType::CloseBrace)
            .unwrap();
        assert_eq!((close.line, close.offset), (2, 31));
    }

    #[test]
    fn indentation_that_matches_no_block_is_an_error() {
        let err = LexerV2::with_layout(
            "page P(path: \"/\")\n        Text(\"a\")\n    Text(\"b\")\n",
            "<t>",
            Layout::Offside,
        )
        .tokenize()
        .unwrap_err()
        .to_string();
        assert!(err.contains("matches no block"), "{err}");
        let err = LexerV2::with_layout("    page P(path: \"/\")\n", "<t>", Layout::Offside)
            .tokenize()
            .unwrap_err()
            .to_string();
        assert!(err.contains("first line"), "{err}");
    }

    #[test]
    fn a_stray_css_character_outside_a_style_block_is_an_error() {
        let err = LexerV2::new("Text(#fff)", "<t>").tokenize().unwrap_err();
        assert!(err.to_string().contains("style { }"), "{err}");
    }
}
