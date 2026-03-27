use crate::error::{Diagnostic, WebFluentError, Result};
use super::token::{Token, TokenType, keyword_or_identifier};

pub struct Lexer {
    source: Vec<char>,
    pos: usize,
    line: usize,
    column: usize,
    file: String,
}

impl Lexer {
    pub fn new(source: &str, file: &str) -> Self {
        Self {
            source: source.chars().collect(),
            pos: 0,
            line: 1,
            column: 1,
            file: file.to_string(),
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();

        while self.pos < self.source.len() {
            self.skip_whitespace();
            if self.pos >= self.source.len() {
                break;
            }

            let ch = self.current();

            // Skip comments
            if ch == '/' {
                if self.peek() == Some('/') {
                    self.skip_line_comment();
                    continue;
                } else if self.peek() == Some('*') {
                    self.skip_block_comment()?;
                    continue;
                }
            }

            let token = match ch {
                // String literals
                '"' => self.read_string()?,

                // Numbers
                '0'..='9' => self.read_number()?,

                // Identifiers and keywords
                'a'..='z' | 'A'..='Z' | '_' => self.read_identifier()?,

                // Operators
                '+' => self.single_token(TokenType::Plus),
                '-' => self.single_token(TokenType::Minus),
                '*' => self.single_token(TokenType::Star),
                '%' => self.single_token(TokenType::Percent),
                '/' => self.single_token(TokenType::Slash),

                '=' => {
                    if self.peek() == Some('=') {
                        let t = Token::new(TokenType::DoubleEquals, self.line, self.column);
                        self.advance();
                        self.advance();
                        t
                    } else if self.peek() == Some('>') {
                        let t = Token::new(TokenType::Arrow, self.line, self.column);
                        self.advance();
                        self.advance();
                        t
                    } else {
                        self.single_token(TokenType::Equals)
                    }
                }

                '!' => {
                    if self.peek() == Some('=') {
                        let t = Token::new(TokenType::NotEquals, self.line, self.column);
                        self.advance();
                        self.advance();
                        t
                    } else {
                        self.single_token(TokenType::Not)
                    }
                }

                '<' => {
                    if self.peek() == Some('=') {
                        let t = Token::new(TokenType::LessEquals, self.line, self.column);
                        self.advance();
                        self.advance();
                        t
                    } else {
                        self.single_token(TokenType::LessThan)
                    }
                }

                '>' => {
                    if self.peek() == Some('=') {
                        let t = Token::new(TokenType::GreaterEquals, self.line, self.column);
                        self.advance();
                        self.advance();
                        t
                    } else {
                        self.single_token(TokenType::GreaterThan)
                    }
                }

                '&' => {
                    if self.peek() == Some('&') {
                        let t = Token::new(TokenType::And, self.line, self.column);
                        self.advance();
                        self.advance();
                        t
                    } else {
                        return Err(WebFluentError::LexerError(
                            Diagnostic::new("Unexpected character '&', did you mean '&&'?", &self.file, self.line, self.column)
                        ));
                    }
                }

                '|' => {
                    if self.peek() == Some('|') {
                        let t = Token::new(TokenType::Or, self.line, self.column);
                        self.advance();
                        self.advance();
                        t
                    } else {
                        return Err(WebFluentError::LexerError(
                            Diagnostic::new("Unexpected character '|', did you mean '||'?", &self.file, self.line, self.column)
                        ));
                    }
                }

                // Punctuation
                '(' => self.single_token(TokenType::OpenParen),
                ')' => self.single_token(TokenType::CloseParen),
                '{' => self.single_token(TokenType::OpenBrace),
                '}' => self.single_token(TokenType::CloseBrace),
                '[' => self.single_token(TokenType::OpenBracket),
                ']' => self.single_token(TokenType::CloseBracket),
                ':' => self.single_token(TokenType::Colon),
                ',' => self.single_token(TokenType::Comma),
                '.' => self.single_token(TokenType::Dot),
                '?' => self.single_token(TokenType::QuestionMark),

                // @ directives — used in style blocks for @media, @keyframes, etc.
                '@' => self.read_at_rule(),

                _ => {
                    return Err(WebFluentError::LexerError(
                        Diagnostic::new(format!("Unexpected character '{}'", ch), &self.file, self.line, self.column)
                    ));
                }
            };

            tokens.push(token);
        }

        tokens.push(Token::new(TokenType::EOF, self.line, self.column));
        Ok(tokens)
    }

    fn current(&self) -> char {
        self.source[self.pos]
    }

    fn peek(&self) -> Option<char> {
        if self.pos + 1 < self.source.len() {
            Some(self.source[self.pos + 1])
        } else {
            None
        }
    }

    fn advance(&mut self) {
        if self.pos < self.source.len() {
            if self.source[self.pos] == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
            self.pos += 1;
        }
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.source.len() && self.source[self.pos].is_whitespace() {
            self.advance();
        }
    }

    fn skip_line_comment(&mut self) {
        while self.pos < self.source.len() && self.source[self.pos] != '\n' {
            self.advance();
        }
    }

    fn skip_block_comment(&mut self) -> Result<()> {
        let start_line = self.line;
        let start_col = self.column;
        self.advance(); // skip /
        self.advance(); // skip *
        while self.pos < self.source.len() {
            if self.current() == '*' && self.peek() == Some('/') {
                self.advance();
                self.advance();
                return Ok(());
            }
            self.advance();
        }
        Err(WebFluentError::LexerError(
            Diagnostic::new("Unterminated block comment", &self.file, start_line, start_col)
        ))
    }

    fn single_token(&mut self, token_type: TokenType) -> Token {
        let t = Token::new(token_type, self.line, self.column);
        self.advance();
        t
    }

    fn read_at_rule(&mut self) -> Token {
        let start_col = self.column;
        self.advance(); // skip @
        let mut name = String::from("@");
        while self.pos < self.source.len() {
            let c = self.source[self.pos];
            if c.is_alphanumeric() || c == '_' || c == '-' {
                name.push(c);
                self.advance();
            } else {
                break;
            }
        }
        Token::new(TokenType::Identifier(name), self.line, start_col)
    }

    fn read_string(&mut self) -> Result<Token> {
        let start_line = self.line;
        let start_col = self.column;
        self.advance(); // skip opening "

        let mut value = String::new();
        while self.pos < self.source.len() && self.current() != '"' {
            if self.current() == '\\' {
                self.advance();
                if self.pos >= self.source.len() {
                    return Err(WebFluentError::LexerError(
                        Diagnostic::new("Unterminated string literal", &self.file, start_line, start_col)
                    ));
                }
                match self.current() {
                    'n' => value.push('\n'),
                    't' => value.push('\t'),
                    'r' => value.push('\r'),
                    '\\' => value.push('\\'),
                    '"' => value.push('"'),
                    '{' => value.push('\u{FFFE}'), // Escaped brace placeholder
                    '}' => value.push('\u{FFFF}'), // Escaped brace placeholder
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
            return Err(WebFluentError::LexerError(
                Diagnostic::new("Unterminated string literal", &self.file, start_line, start_col)
            ));
        }

        self.advance(); // skip closing "
        Ok(Token::new(TokenType::StringLiteral(value), start_line, start_col))
    }

    fn read_number(&mut self) -> Result<Token> {
        let start_line = self.line;
        let start_col = self.column;
        let mut num_str = String::new();
        let mut has_dot = false;

        while self.pos < self.source.len() {
            let ch = self.current();
            if ch.is_ascii_digit() {
                num_str.push(ch);
                self.advance();
            } else if ch == '.' && !has_dot {
                // Check if next char is a digit (otherwise it's a method call dot)
                if let Some(next) = self.peek() {
                    if next.is_ascii_digit() {
                        has_dot = true;
                        num_str.push(ch);
                        self.advance();
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        let value: f64 = num_str.parse().map_err(|_| {
            WebFluentError::LexerError(
                Diagnostic::new(format!("Invalid number '{}'", num_str), &self.file, start_line, start_col)
            )
        })?;

        Ok(Token::new(TokenType::NumberLiteral(value), start_line, start_col))
    }

    fn read_identifier(&mut self) -> Result<Token> {
        let start_line = self.line;
        let start_col = self.column;
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

        // Check for event syntax: on:eventname
        if word == "on" && self.pos < self.source.len() && self.current() == ':' {
            self.advance(); // skip :
            let mut event_name = String::new();
            while self.pos < self.source.len() {
                let ch = self.current();
                if ch.is_alphanumeric() || ch == '_' {
                    event_name.push(ch);
                    self.advance();
                } else {
                    break;
                }
            }
            if event_name.is_empty() {
                return Err(WebFluentError::LexerError(
                    Diagnostic::new("Expected event name after 'on:'", &self.file, start_line, start_col)
                ));
            }
            return Ok(Token::new(TokenType::Event(event_name), start_line, start_col));
        }

        // Check for dot-accessed sub-components like Navbar.Brand, Card.Header, etc.
        // These are handled as separate tokens: Identifier + Dot + Identifier

        let token_type = keyword_or_identifier(&word);
        Ok(Token::new(token_type, start_line, start_col))
    }
}
