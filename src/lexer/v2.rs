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

use super::token::{StringKind, StringLit, Token, TokenType};
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
            // `#"…"#` is a raw string; `#0F766E` is a colour.
            '#' if self.raw_string_ahead() => self.read_raw_string()?,
            '#' if self.color_ahead() => self.read_color(),
            // `@2026-03-14`, `@09:30`, `@2026-03-14T09:30Z`.
            '@' if self.peek().is_some_and(|c| c.is_ascii_digit()) => self.read_temporal(),
            // `€12.99`, `$12.99` — an amount in the currency the symbol names.
            '€' | '£' | '¥' if self.peek().is_some_and(|c| c.is_ascii_digit()) => {
                self.read_money()
            }
            '$' if self.peek().is_some_and(|c| c.is_ascii_digit()) => self.read_money(),
            '0'..='9' => self.read_number()?,
            'a'..='z' | 'A'..='Z' | '_' => self.read_word(),
            '$' => self.read_token_name()?,
            '+' => single(self, TokenType::Plus),
            '-' => single(self, TokenType::Minus),
            '*' => single(self, TokenType::Star),
            '%' => single(self, TokenType::Percent),
            // `/` divides after an operand; anywhere else it opens a regular
            // expression, as it does in JavaScript.
            '/' if !after_operand(before) => self.read_regex()?,
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
                Some('.') => self.two(TokenType::OptionalChain),
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
            '.' => match (self.peek(), self.source.get(self.pos + 2).copied()) {
                (Some('.'), Some('.')) => {
                    self.advance();
                    self.advance();
                    self.advance();
                    Token::new(TokenType::Ellipsis, line, column)
                }
                (Some('.'), Some('=')) => {
                    self.advance();
                    self.advance();
                    self.advance();
                    Token::new(TokenType::DotDotEq, line, column)
                }
                (Some('.'), _) => self.two(TokenType::DotDot),
                _ => single(self, TokenType::Dot),
            },
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

    /// `/pattern/flags`: the pattern to the closing `/` — a `\/` and a `/`
    /// inside `[…]` do not close it — then the flags.
    fn read_regex(&mut self) -> Result<Token> {
        let (line, column) = (self.line, self.column);
        self.advance(); // /
        let mut pattern = String::new();
        let mut in_class = false;
        loop {
            if self.pos >= self.source.len() || self.current() == '\n' {
                return Err(WebFluentError::LexerError(Diagnostic::new(
                    "Unterminated regular expression",
                    &self.file,
                    line,
                    column,
                )));
            }
            let ch = self.current();
            match ch {
                '\\' => {
                    pattern.push(ch);
                    self.advance();
                    if self.pos < self.source.len() {
                        pattern.push(self.current());
                        self.advance();
                    }
                    continue;
                }
                '[' => in_class = true,
                ']' => in_class = false,
                '/' if !in_class => break,
                _ => {}
            }
            pattern.push(ch);
            self.advance();
        }
        self.advance(); // /
        let mut flags = String::new();
        while self.pos < self.source.len() && self.current().is_ascii_alphabetic() {
            flags.push(self.current());
            self.advance();
        }
        Ok(Token::new(
            TokenType::RegexLiteral(pattern, flags),
            line,
            column,
        ))
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
            // A keyframe of an `animation`: `from {`, `to {`, `50% {`.
            _ if self.keyframe_ahead() => {
                let mut text = String::new();
                while self.pos < self.source.len() && self.current() != '{' {
                    text.push(self.current());
                    self.advance();
                }
                Ok(Token::new(
                    TokenType::RawSelector(text.trim().to_string()),
                    line,
                    column,
                ))
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

    /// Whether a keyframe selector — `from`, `to`, or percentages such as
    /// `50%` or `0%, 100%` — followed by `{` starts here.
    fn keyframe_ahead(&self) -> bool {
        let rest: String = self.source[self.pos..]
            .iter()
            .take_while(|c| **c != '\n')
            .collect();
        let Some((head, _)) = rest.split_once('{') else {
            return false;
        };
        let head = head.trim();
        if head == "from" || head == "to" {
            return true;
        }
        !head.is_empty()
            && head.split(',').all(|part| {
                let part = part.trim();
                part.ends_with('%') && part[..part.len() - 1].parse::<f64>().is_ok()
            })
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

    /// A string literal: `"…"`, the block form `"""…"""`, or the raw form
    /// `#"…"#`.
    ///
    /// What comes back is the spelling between the delimiters, exactly as
    /// written — the escapes are the parser's to resolve, in the one pass
    /// that also splits the splices out.
    fn read_string(&mut self) -> Result<Token> {
        if self.peek() == Some('"') && self.source.get(self.pos + 2) == Some(&'"') {
            return self.read_block_string();
        }
        let (line, column) = (self.line, self.column);
        self.advance();
        let mut value = String::new();
        while self.pos < self.source.len() && self.current() != '"' {
            if self.current() == '\\' {
                // Kept whole: `\"` does not end the string, and what the
                // escape means is decided once, in the parser.
                value.push('\\');
                self.advance();
                if self.pos >= self.source.len() {
                    break;
                }
                value.push(self.current());
            } else if self.current() == '{' {
                // A splice may hold a string of its own — `{a ?? "none"}` —
                // so a balanced brace group on this line is taken whole,
                // quotes and all; a lone `{` is a character like any other.
                match self.splice_end() {
                    Some(end) => {
                        while self.pos < end {
                            value.push(self.current());
                            self.advance();
                        }
                        continue;
                    }
                    None => value.push('{'),
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
        Ok(Token::new(
            TokenType::StringLiteral(StringLit::new(value, StringKind::Plain)),
            line,
            column,
        ))
    }

    /// `"""…"""`: everything between the delimiters, newlines and all. The
    /// parser takes the indentation off, since only it knows where the
    /// closing delimiter sat.
    fn read_block_string(&mut self) -> Result<Token> {
        let (line, column) = (self.line, self.column);
        for _ in 0..3 {
            self.advance();
        }
        let start = self.pos;
        loop {
            if self.pos + 2 >= self.source.len() {
                return Err(WebFluentError::LexerError(Diagnostic::new(
                    "Unterminated block string: no closing `\"\"\"`",
                    &self.file,
                    line,
                    column,
                )));
            }
            if self.current() == '"'
                && self.source[self.pos + 1] == '"'
                && self.source[self.pos + 2] == '"'
            {
                break;
            }
            self.advance();
        }
        let value: String = self.source[start..self.pos].iter().collect();
        for _ in 0..3 {
            self.advance();
        }
        Ok(Token::new(
            TokenType::StringLiteral(StringLit::new(value, StringKind::Block)),
            line,
            column,
        ))
    }

    /// Whether a colour opens here: `#` and three to eight hex digits that
    /// a word does not run on from.
    fn color_ahead(&self) -> bool {
        let mut n = 0;
        while self
            .source
            .get(self.pos + 1 + n)
            .is_some_and(|c| c.is_ascii_hexdigit())
        {
            n += 1;
        }
        matches!(n, 3 | 4 | 6 | 8)
            && !self
                .source
                .get(self.pos + 1 + n)
                .is_some_and(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
    }

    /// `#0F766E` — the hex digits, without the hash.
    fn read_color(&mut self) -> Token {
        let (line, column) = (self.line, self.column);
        self.advance();
        let mut digits = String::new();
        while self.pos < self.source.len() && self.current().is_ascii_hexdigit() {
            digits.push(self.current());
            self.advance();
        }
        Token::new(TokenType::ColorLiteral(digits), line, column)
    }

    /// `@2026-03-14`, `@09:30:15`, `@2026-03-14T09:30:00Z` — everything a
    /// date, a time or both can be written with, in one token.
    fn read_temporal(&mut self) -> Token {
        let (line, column) = (self.line, self.column);
        self.advance(); // `@`
        let mut text = String::new();
        while self.pos < self.source.len() {
            let c = self.current();
            let part = match c {
                c if c.is_ascii_digit() => true,
                // Fractional seconds; a `.` before anything else is the
                // method the literal is about to be asked for.
                '.' => self.peek().is_some_and(|n| n.is_ascii_digit()),
                // `T` joins a date to a time, and only there.
                'T' => text.contains('-'),
                '-' | ':' | 'Z' | '+' => true,
                _ => false,
            };
            if !part {
                break;
            }
            text.push(c);
            self.advance();
        }
        Token::new(TokenType::TemporalLiteral(text), line, column)
    }

    /// `€12.99` — the symbol says the currency, the digits the amount.
    fn read_money(&mut self) -> Token {
        let (line, column) = (self.line, self.column);
        let symbol = self.current().to_string();
        self.advance();
        let mut amount = String::new();
        while self.pos < self.source.len()
            && (self.current().is_ascii_digit() || self.current() == '.')
        {
            amount.push(self.current());
            self.advance();
        }
        Token::new(TokenType::MoneyLiteral(symbol, amount), line, column)
    }

    /// Whether a raw string opens here: `#`s and then a quote.
    fn raw_string_ahead(&self) -> bool {
        let mut i = self.pos;
        while self.source.get(i) == Some(&'#') {
            i += 1;
        }
        self.source.get(i) == Some(&'"')
    }

    /// `#"…"#`, with as many `#` as the text needs: no escapes, no
    /// splices, so a sample of code goes in whole.
    fn read_raw_string(&mut self) -> Result<Token> {
        let (line, column) = (self.line, self.column);
        let mut hashes = 0usize;
        while self.pos < self.source.len() && self.current() == '#' {
            hashes += 1;
            self.advance();
        }
        if self.pos >= self.source.len() || self.current() != '"' {
            return Err(WebFluentError::LexerError(Diagnostic::new(
                "Expected `\"` after `#`: a raw string is written `#\"…\"#`",
                &self.file,
                line,
                column,
            )));
        }
        self.advance();
        let start = self.pos;
        let closes = |lexer: &Self| {
            if lexer.current() != '"' {
                return false;
            }
            (1..=hashes).all(|n| lexer.source.get(lexer.pos + n) == Some(&'#'))
        };
        while self.pos < self.source.len() && !closes(self) {
            self.advance();
        }
        if self.pos >= self.source.len() {
            return Err(WebFluentError::LexerError(Diagnostic::new(
                "Unterminated raw string",
                &self.file,
                line,
                column,
            )));
        }
        let value: String = self.source[start..self.pos].iter().collect();
        for _ in 0..=hashes {
            self.advance();
        }
        Ok(Token::new(
            TokenType::StringLiteral(StringLit::new(value, StringKind::Raw)),
            line,
            column,
        ))
    }

    /// The index just past the `}` that closes the splice opening at the
    /// current `{`, when the group is balanced on this line — a `{`
    /// followed by a name, a `[` or a `(`, with any string inside it kept
    /// whole — else `None`.
    fn splice_end(&self) -> Option<usize> {
        crate::parser::v2::splice_end(&self.source, self.pos)
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
                // Two spaces and the next declaration: `color-primary: #0F766E
                // radius-md: 14px` on one line is two tokens, as two
                // statements on one line are everywhere else in the language.
                // The first value used to run on to the end of the line and
                // swallow the second, so a one-line theme set only its first
                // token, to a value no browser understands.
                ' ' if parens == 0 && braces == 0 && self.next_declaration_after_spaces() => break,
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

    /// At a space: whether two or more spaces here are followed by what
    /// starts another declaration — a property (`name:`, `--name:`), a nested
    /// rule (`&…`) or an at-rule (`@…`).
    fn next_declaration_after_spaces(&self) -> bool {
        let mut i = self.pos;
        while i < self.source.len() && self.source[i] == ' ' {
            i += 1;
        }
        if i - self.pos < 2 || i >= self.source.len() {
            return false;
        }
        match self.source[i] {
            '&' | '@' => true,
            c if c.is_ascii_alphabetic() || c == '-' => {
                let mut j = i;
                while j < self.source.len()
                    && (self.source[j].is_ascii_alphanumeric() || self.source[j] == '-')
                {
                    j += 1;
                }
                while j < self.source.len() && self.source[j] == ' ' {
                    j += 1;
                }
                j < self.source.len() && self.source[j] == ':'
            }
            _ => false,
        }
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

/// Whether the token before a `/` is the end of an operand, so the `/`
/// divides: a name, a literal, a closing bracket. After an operator, a
/// comma, `(`, `{` or `return`, a `/` opens a regular expression.
fn after_operand(before: &[Token]) -> bool {
    match before.last().map(|t| &t.token_type) {
        // A keyword that a value follows is not an operand.
        Some(TokenType::Identifier(word)) => !matches!(
            word.as_str(),
            "return"
                | "in"
                | "if"
                | "else"
                | "emit"
                | "await"
                | "let"
                | "state"
                | "derived"
                | "show"
                | "match"
        ),
        Some(
            TokenType::StringLiteral(_)
            | TokenType::NumberLiteral(_)
            | TokenType::BoolLiteral(_)
            | TokenType::Null
            | TokenType::DesignToken(_)
            | TokenType::RegexLiteral(..)
            | TokenType::CloseParen
            | TokenType::CloseBracket
            | TokenType::CloseBrace,
        ) => true,
        _ => false,
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
        ] if keyword == "theme" || keyword == "animation" => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_splice_keeps_a_string_of_its_own_and_a_lone_brace_is_text() {
        let strings = |src: &str| -> Vec<String> {
            kinds(src)
                .into_iter()
                .filter_map(|t| match t {
                    TokenType::StringLiteral(s) => Some(s.spelling),
                    _ => None,
                })
                .collect()
        };
        assert_eq!(
            strings("\"B {a ?? \"x\"} end\""),
            vec!["B {a ?? \"x\"} end"]
        );
        assert_eq!(
            strings("\"{if ok { \"y\" } else { \"n\" }}\""),
            vec!["{if ok { \"y\" } else { \"n\" }}"]
        );
        // No balanced group on the line: the `{` is a character, and the
        // string ends at the next quote as it always did.
        assert_eq!(strings("\"a { b\" \"c\""), vec!["a { b", "c"]);
        assert_eq!(strings("\"{\" \"d\""), vec!["{", "d"]);
        // A group with a `\"` inside it, the older spelling, still lexes;
        // the escape is the parser's to resolve, so the token keeps it.
        assert_eq!(strings("\"{a ?? \\\"x\\\"}\""), vec!["{a ?? \\\"x\\\"}"]);
    }

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
    fn two_spaces_end_a_value_before_the_next_declaration() {
        let values: Vec<String> = kinds(
            "theme T { color-primary: #0F766E  radius-md: 14px  font-family: \"A  b: c\", serif  --x: 1  &:hover { color: red } }",
        )
        .into_iter()
        .filter_map(|t| match t {
            TokenType::RawValue(v) => Some(v),
            _ => None,
        })
        .collect();
        assert_eq!(values[0], "#0F766E");
        assert_eq!(values[1], "14px");
        assert_eq!(
            values[2], "\"A  b: c\", serif",
            "two spaces inside quotes are the value's"
        );
        assert_eq!(values[3], "1");
        // One space and a colon is still one value: `url(data:…)`, `a b: c`.
        let one: Vec<String> = kinds("style { background: url(data:x)  grid-area: a b }")
            .into_iter()
            .filter_map(|t| match t {
                TokenType::RawValue(v) => Some(v),
                _ => None,
            })
            .collect();
        assert_eq!(one, vec!["url(data:x)".to_string(), "a b".to_string()]);
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
        // `#fff` is a colour, wherever it is written; a `#` that opens
        // neither a colour nor a raw string is the stray one.
        let err = LexerV2::new("Text(#zz)", "<t>").tokenize().unwrap_err();
        assert!(err.to_string().contains("style { }"), "{err}");
        let ok = LexerV2::new("Text(#0F766E)", "<t>")
            .tokenize()
            .expect("a colour");
        assert!(ok.iter().any(|t| matches!(
            &t.token_type,
            TokenType::ColorLiteral(c) if c == "0F766E"
        )));
    }
}
