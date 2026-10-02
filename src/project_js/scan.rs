//! Reading a plain browser script for the names it makes global.
//!
//! This is not a JavaScript parser. It is a tokenizer that knows exactly
//! what a parser must know to find where one token ends — strings, template
//! literals with `${}` inside them, comments, and whether a `/` begins a
//! regular expression or divides — plus the bracket depth, and a reader over
//! those tokens that looks only at the top level and at class bodies. It
//! never evaluates anything, and what it cannot read it says so rather than
//! guess.
//!
//! What a browser makes global from a classic script, and so what is read:
//! a top-level `function`, `class`, `var`, `let` or `const` (a destructuring
//! one included), and `window.x = …` / `globalThis.x = …` at any depth. A
//! top-level `import` or `export` makes the file a module, which a
//! `<script src>` cannot run.

/// What one top-level name is.
#[derive(Debug, Clone, PartialEq)]
pub enum NameKind {
    /// A function: its parameters as written, a `...rest` one included.
    Function { params: Vec<Param>, is_async: bool },
    /// A class: what a call to it takes (its constructor's parameters) and
    /// the methods its body declares.
    Class {
        params: Vec<Param>,
        methods: Vec<Method>,
    },
    /// Any other value.
    Value,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: String,
    /// It has a default value, so a call may leave it out.
    pub optional: bool,
    /// `...rest`: any number of trailing arguments.
    pub rest: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Method {
    pub name: String,
    pub params: Vec<Param>,
    pub is_static: bool,
    /// `get x()`: read as a field, not called.
    pub getter: bool,
    pub doc: Option<String>,
}

/// One name a script makes global.
#[derive(Debug, Clone, PartialEq)]
pub struct Name {
    pub name: String,
    pub kind: NameKind,
    /// 1-based, where the name is written.
    pub line: usize,
    pub col: usize,
    /// The `/** … */` comment directly above the declaration, as written.
    pub doc: Option<String>,
}

/// What reading a file found.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Scan {
    pub names: Vec<Name>,
    /// Where the file has a top-level `import` or `export`: it is a module.
    pub module_at: Option<(usize, usize)>,
    /// What could not be read, with where.
    pub problems: Vec<Problem>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Problem {
    pub message: String,
    pub line: usize,
    pub col: usize,
}

// ─── Tokens ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Ident(String),
    Punct(&'static str),
    Str,
    Num,
    Regex,
    /// A whole template literal; its `${}` parts were scanned and dropped.
    Template,
}

#[derive(Debug, Clone)]
struct Token {
    tok: Tok,
    line: usize,
    col: usize,
    /// A line break comes before it: what automatic semicolon insertion
    /// reads.
    newline_before: bool,
    /// The `/** … */` comment that ends right before it, if any.
    doc: Option<String>,
}

/// Punctuators, longest first so the longest match wins.
const PUNCTS: &[&str] = &[
    ">>>=", "...", "===", "!==", "**=", "<<=", ">>=", ">>>", "&&=", "||=", "??=", "=>", "==", "!=",
    "<=", ">=", "&&", "||", "??", "?.", "++", "--", "+=", "-=", "*=", "/=", "%=", "&=", "|=", "^=",
    "**", "<<", ">>", "{", "}", "(", ")", "[", "]", ";", ",", "<", ">", "+", "-", "*", "/", "%",
    "&", "|", "^", "!", "~", "?", ":", "=", ".", "@",
];

/// Words after which a `/` begins a regular expression, not a division.
const REGEX_AFTER_WORD: &[&str] = &[
    "return",
    "typeof",
    "instanceof",
    "in",
    "of",
    "new",
    "delete",
    "void",
    "throw",
    "case",
    "do",
    "else",
    "yield",
    "await",
];

struct Lexer<'a> {
    src: &'a [u8],
    text: &'a str,
    at: usize,
    line: usize,
    col: usize,
    tokens: Vec<Token>,
    problems: Vec<Problem>,
}

impl<'a> Lexer<'a> {
    fn new(text: &'a str) -> Self {
        Lexer {
            src: text.as_bytes(),
            text,
            at: 0,
            line: 1,
            col: 1,
            tokens: Vec::new(),
            problems: Vec::new(),
        }
    }

    fn peek(&self, ahead: usize) -> Option<u8> {
        self.src.get(self.at + ahead).copied()
    }

    /// Step over one character (a whole UTF-8 sequence), keeping line and
    /// column.
    fn bump(&mut self) {
        let Some(&b) = self.src.get(self.at) else {
            return;
        };
        let len = match b {
            0x00..=0x7F => 1,
            0xC0..=0xDF => 2,
            0xE0..=0xEF => 3,
            _ => 4,
        };
        self.at = (self.at + len).min(self.src.len());
        if b == b'\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
    }

    fn problem(&mut self, message: &str, line: usize, col: usize) {
        self.problems.push(Problem {
            message: message.to_string(),
            line,
            col,
        });
    }

    /// Whether a `/` here begins a regular expression: it does wherever an
    /// expression may begin, which the token before it decides.
    fn regex_allowed(&self) -> bool {
        match self.tokens.last().map(|t| &t.tok) {
            None => true,
            Some(Tok::Ident(w)) => REGEX_AFTER_WORD.contains(&w.as_str()),
            Some(Tok::Punct(p)) => !matches!(*p, ")" | "]" | "++" | "--"),
            // `}` ends a block far more often than an object a division
            // follows; a number, string or regex is an operand.
            Some(Tok::Str | Tok::Num | Tok::Regex | Tok::Template) => false,
        }
    }

    fn run(mut self) -> (Vec<Token>, Vec<Problem>) {
        let mut newline = false;
        let mut doc: Option<String> = None;
        // The brace depth at which each open `${` resumes its template.
        let mut template_stack: Vec<usize> = Vec::new();
        let mut braces = 0usize;
        // A hashbang is a comment, on the first line only.
        if self.text.starts_with("#!") {
            while self.peek(0).is_some_and(|b| b != b'\n') {
                self.bump();
            }
        }
        while let Some(b) = self.peek(0) {
            let (line, col) = (self.line, self.col);
            match b {
                b'\n' => {
                    newline = true;
                    self.bump();
                }
                b' ' | b'\t' | b'\r' | 0x0B | 0x0C => self.bump(),
                b'/' if self.peek(1) == Some(b'/') => {
                    while self.peek(0).is_some_and(|b| b != b'\n') {
                        self.bump();
                    }
                }
                b'/' if self.peek(1) == Some(b'*') => {
                    let start = self.at;
                    let is_doc = self.peek(2) == Some(b'*') && self.peek(3) != Some(b'/');
                    self.bump();
                    self.bump();
                    loop {
                        match self.peek(0) {
                            None => {
                                self.problem("a `/*` comment is never closed", line, col);
                                break;
                            }
                            Some(b'*') if self.peek(1) == Some(b'/') => {
                                self.bump();
                                self.bump();
                                break;
                            }
                            Some(b'\n') => {
                                newline = true;
                                self.bump();
                            }
                            _ => self.bump(),
                        }
                    }
                    doc = is_doc.then(|| self.text[start..self.at].to_string());
                    continue;
                }
                b'"' | b'\'' => {
                    self.string(b, line, col);
                    self.push(Tok::Str, line, col, &mut newline, &mut doc);
                }
                b'`' => {
                    self.bump();
                    if self.template_rest(line, col) {
                        template_stack.push(braces);
                        braces += 1;
                    } else {
                        self.push(Tok::Template, line, col, &mut newline, &mut doc);
                    }
                }
                b'}' if template_stack.last() == Some(&braces.saturating_sub(1)) => {
                    // The `}` that closes a `${`: the template goes on.
                    template_stack.pop();
                    braces -= 1;
                    self.bump();
                    if self.template_rest(line, col) {
                        template_stack.push(braces);
                        braces += 1;
                    } else {
                        self.push(Tok::Template, line, col, &mut newline, &mut doc);
                    }
                }
                b'/' if self.regex_allowed() => {
                    self.regex(line, col);
                    self.push(Tok::Regex, line, col, &mut newline, &mut doc);
                }
                b'0'..=b'9' => {
                    self.number();
                    self.push(Tok::Num, line, col, &mut newline, &mut doc);
                }
                b'.' if self.peek(1).is_some_and(|c| c.is_ascii_digit()) => {
                    self.number();
                    self.push(Tok::Num, line, col, &mut newline, &mut doc);
                }
                _ if is_ident_start(self.char_at()) => {
                    let start = self.at;
                    self.bump();
                    while self.at < self.src.len() && is_ident_part(self.char_at()) {
                        self.bump();
                    }
                    let word = self.text[start..self.at].to_string();
                    self.push(Tok::Ident(word), line, col, &mut newline, &mut doc);
                }
                _ => {
                    let rest = &self.text[self.at..];
                    if let Some(p) = PUNCTS.iter().find(|p| rest.starts_with(**p)) {
                        for _ in 0..p.len() {
                            self.bump();
                        }
                        match *p {
                            "{" => braces += 1,
                            "}" => braces = braces.saturating_sub(1),
                            _ => {}
                        }
                        self.push(Tok::Punct(p), line, col, &mut newline, &mut doc);
                    } else {
                        self.problem(
                            &format!(
                                "`{}` is not a character a script may have here",
                                self.char_at()
                            ),
                            line,
                            col,
                        );
                        self.bump();
                    }
                }
            }
        }
        if !template_stack.is_empty() {
            let (line, col) = (self.line, self.col);
            self.problem("a template literal's `${` is never closed", line, col);
        }
        (self.tokens, self.problems)
    }

    fn char_at(&self) -> char {
        self.text[self.at..].chars().next().unwrap_or('\0')
    }

    fn push(
        &mut self,
        tok: Tok,
        line: usize,
        col: usize,
        newline: &mut bool,
        doc: &mut Option<String>,
    ) {
        self.tokens.push(Token {
            tok,
            line,
            col,
            newline_before: *newline,
            doc: doc.take(),
        });
        *newline = false;
    }

    fn string(&mut self, quote: u8, line: usize, col: usize) {
        self.bump();
        loop {
            match self.peek(0) {
                None | Some(b'\n') => {
                    self.problem("a string is never closed", line, col);
                    return;
                }
                Some(b'\\') => {
                    self.bump();
                    self.bump();
                }
                Some(b) if b == quote => {
                    self.bump();
                    return;
                }
                _ => self.bump(),
            }
        }
    }

    /// The rest of a template literal, from just after its opening backtick
    /// or a `${}`'s closing brace. True when it stops at a `${`, whose
    /// expression the main loop then reads as tokens.
    fn template_rest(&mut self, line: usize, col: usize) -> bool {
        loop {
            match self.peek(0) {
                None => {
                    self.problem("a template literal is never closed", line, col);
                    return false;
                }
                Some(b'\\') => {
                    self.bump();
                    self.bump();
                }
                Some(b'`') => {
                    self.bump();
                    return false;
                }
                Some(b'$') if self.peek(1) == Some(b'{') => {
                    self.bump();
                    self.bump();
                    return true;
                }
                _ => self.bump(),
            }
        }
    }

    fn regex(&mut self, line: usize, col: usize) {
        self.bump();
        let mut in_class = false;
        loop {
            match self.peek(0) {
                None | Some(b'\n') => {
                    self.problem("a regular expression is never closed", line, col);
                    return;
                }
                Some(b'\\') => {
                    self.bump();
                    self.bump();
                }
                Some(b'[') => {
                    in_class = true;
                    self.bump();
                }
                Some(b']') => {
                    in_class = false;
                    self.bump();
                }
                Some(b'/') if !in_class => {
                    self.bump();
                    break;
                }
                _ => self.bump(),
            }
        }
        while self.peek(0).is_some_and(|b| b.is_ascii_alphabetic()) {
            self.bump();
        }
    }

    fn number(&mut self) {
        // Digits, a fraction, an exponent with its sign, a `0x` prefix, `_`
        // separators and a BigInt `n`: enough to step over any of them.
        while let Some(b) = self.peek(0) {
            let exponent_sign = matches!(b, b'+' | b'-')
                && matches!(self.src.get(self.at.wrapping_sub(1)), Some(b'e' | b'E'))
                && !self.text[..self.at].to_ascii_lowercase().contains("0x");
            if b.is_ascii_alphanumeric() || b == b'_' || b == b'.' || exponent_sign {
                self.bump();
            } else {
                break;
            }
        }
    }
}

fn is_ident_start(c: char) -> bool {
    c == '$' || c == '_' || c == '#' || c.is_alphabetic()
}

fn is_ident_part(c: char) -> bool {
    c == '$' || c == '_' || c.is_alphanumeric() || c == '\u{200C}' || c == '\u{200D}'
}

// ─── The top level ───────────────────────────────────────────────────

/// Read `source` for the names it makes global.
pub fn scan(source: &str) -> Scan {
    let (tokens, problems) = Lexer::new(source).run();
    let mut out = Scan {
        problems,
        ..Scan::default()
    };
    Reader {
        t: &tokens,
        i: 0,
        out: &mut out,
    }
    .read();
    out
}

struct Reader<'t, 'o> {
    t: &'t [Token],
    i: usize,
    out: &'o mut Scan,
}

impl Reader<'_, '_> {
    fn ident_at(&self, i: usize) -> Option<&str> {
        match self.t.get(i).map(|t| &t.tok) {
            Some(Tok::Ident(w)) => Some(w.as_str()),
            _ => None,
        }
    }

    fn punct_at(&self, i: usize) -> Option<&'static str> {
        match self.t.get(i).map(|t| &t.tok) {
            Some(Tok::Punct(p)) => Some(p),
            _ => None,
        }
    }

    fn is_punct(&self, i: usize, p: &str) -> bool {
        self.punct_at(i) == Some(p)
    }

    /// The index just past the bracket that closes the one at `open`.
    fn matching(&self, open: usize) -> usize {
        let mut depth = 0i32;
        let mut i = open;
        while i < self.t.len() {
            match self.punct_at(i) {
                Some("(" | "[" | "{") => depth += 1,
                Some(")" | "]" | "}") => {
                    depth -= 1;
                    if depth == 0 {
                        return i + 1;
                    }
                }
                _ => {}
            }
            i += 1;
        }
        self.t.len()
    }

    /// Every bracket closed by its own kind, said where one is not.
    fn check_brackets(&mut self) {
        let mut open: Vec<(&'static str, usize, usize)> = Vec::new();
        for t in self.t {
            let Tok::Punct(p) = t.tok else { continue };
            match p {
                "(" | "[" | "{" => open.push((p, t.line, t.col)),
                ")" | "]" | "}" => {
                    let wanted = match p {
                        ")" => "(",
                        "]" => "[",
                        _ => "{",
                    };
                    match open.pop() {
                        Some((o, _, _)) if o == wanted => {}
                        Some((o, line, col)) => {
                            self.out.problems.push(Problem {
                                message: format!(
                                    "`{o}` here is closed by `{p}` at {}:{}",
                                    t.line, t.col
                                ),
                                line,
                                col,
                            });
                            return;
                        }
                        None => {
                            self.out.problems.push(Problem {
                                message: format!("a `{p}` nothing opened"),
                                line: t.line,
                                col: t.col,
                            });
                            return;
                        }
                    }
                }
                _ => {}
            }
        }
        if let Some((o, line, col)) = open.pop() {
            self.out.problems.push(Problem {
                message: format!("a `{o}` is never closed"),
                line,
                col,
            });
        }
    }

    fn read(&mut self) {
        // Only a file whose brackets balance is read for names: where one
        // does not, nothing about its top level can be trusted.
        self.check_brackets();
        if !self.out.problems.is_empty() {
            return;
        }
        let mut depth = 0i32;
        // Where the statement at the top level began: a declaration keyword
        // only counts where a statement may start.
        let mut statement_start = true;
        while self.i < self.t.len() {
            let i = self.i;
            // `window.x = …` / `globalThis.x = …`, at any depth.
            if let Some(owner) = self.ident_at(i)
                && matches!(owner, "window" | "globalThis" | "self")
                && self.is_punct(i + 1, ".")
                && self.ident_at(i + 2).is_some()
                && self.is_punct(i + 3, "=")
                && (i == 0 || !self.is_punct(i - 1, "."))
            {
                let name_tok = &self.t[i + 2];
                let name = self.ident_at(i + 2).unwrap_or_default().to_string();
                let doc = self.doc_before(i);
                let kind = self.value_kind(i + 4);
                self.add(name, kind, name_tok.line, name_tok.col, doc);
            }
            if depth == 0 && (statement_start || self.t[i].newline_before) {
                if let Some(next) = self.top_level_statement(i) {
                    self.i = next;
                    statement_start = true;
                    continue;
                }
            }
            match self.punct_at(i) {
                Some("{" | "(" | "[") => depth += 1,
                Some("}" | ")" | "]") => depth -= 1,
                _ => {}
            }
            statement_start = depth == 0 && matches!(self.punct_at(i), Some(";" | "}"));
            self.i += 1;
        }
    }

    /// A declaration starting at `i` on the top level: the names it makes,
    /// and the index to go on from. `None` when `i` starts no declaration.
    fn top_level_statement(&mut self, i: usize) -> Option<usize> {
        let word = self.ident_at(i)?;
        match word {
            "import" => {
                // `import(…)` and `import.meta` are expressions; anything
                // else is a module's import.
                if !matches!(self.punct_at(i + 1), Some("(" | ".")) {
                    self.mark_module(i);
                }
                None
            }
            "export" => {
                self.mark_module(i);
                None
            }
            "async"
                if self.ident_at(i + 1) == Some("function") && !self.t[i + 1].newline_before =>
            {
                self.function(i, i + 1, true)
            }
            "function" => self.function(i, i, false),
            "class" => self.class(i),
            "var" | "let" | "const" => self.variables(i),
            _ => None,
        }
    }

    fn mark_module(&mut self, i: usize) {
        if self.out.module_at.is_none() {
            self.out.module_at = Some((self.t[i].line, self.t[i].col));
        }
    }

    fn doc_before(&self, i: usize) -> Option<String> {
        self.t.get(i).and_then(|t| t.doc.clone())
    }

    fn add(&mut self, name: String, kind: NameKind, line: usize, col: usize, doc: Option<String>) {
        if let Some(existing) = self.out.names.iter_mut().find(|n| n.name == name) {
            // `var x; … x = function…`: the later, richer reading wins.
            if existing.kind == NameKind::Value {
                existing.kind = kind;
            }
            return;
        }
        self.out.names.push(Name {
            name,
            kind,
            line,
            col,
            doc,
        });
    }

    /// `function f(…) { … }` (or `async function`, or `function*`), where
    /// `start` is its first token and `kw` the `function` keyword.
    fn function(&mut self, start: usize, kw: usize, is_async: bool) -> Option<usize> {
        let mut at = kw + 1;
        if self.is_punct(at, "*") {
            at += 1;
        }
        let name = self.ident_at(at)?.to_string();
        let (line, col) = (self.t[at].line, self.t[at].col);
        if !self.is_punct(at + 1, "(") {
            return None;
        }
        let params = self.params(at + 1);
        let close = self.matching(at + 1);
        let doc = self.doc_before(start);
        self.add(
            name,
            NameKind::Function { params, is_async },
            line,
            col,
            doc,
        );
        // Past the body.
        Some(if self.is_punct(close, "{") {
            self.matching(close)
        } else {
            close
        })
    }

    /// `class C [extends X] { … }`.
    fn class(&mut self, start: usize) -> Option<usize> {
        let name = self.ident_at(start + 1)?.to_string();
        if name == "extends" {
            return None;
        }
        let (line, col) = (self.t[start + 1].line, self.t[start + 1].col);
        let mut at = start + 2;
        if self.ident_at(at) == Some("extends") {
            // The heritage is an expression; skip to the body's brace at
            // this depth.
            at += 1;
            while at < self.t.len() && !self.is_punct(at, "{") {
                if matches!(self.punct_at(at), Some("(" | "[")) {
                    at = self.matching(at);
                } else {
                    at += 1;
                }
            }
        }
        if !self.is_punct(at, "{") {
            return None;
        }
        let end = self.matching(at);
        let (params, methods) = self.class_body(at, end);
        let doc = self.doc_before(start);
        self.add(name, NameKind::Class { params, methods }, line, col, doc);
        Some(end)
    }

    /// The members of a class body between `open` (its `{`) and `end`.
    fn class_body(&self, open: usize, end: usize) -> (Vec<Param>, Vec<Method>) {
        let mut params = Vec::new();
        let mut methods = Vec::new();
        let mut i = open + 1;
        let end = end.saturating_sub(1);
        while i < end {
            let first = i;
            let mut is_static = false;
            let mut getter = false;
            // Modifiers, each only where a name (not `(` or `=`) follows.
            while let Some(word @ ("static" | "async" | "get" | "set")) = self.ident_at(i) {
                if matches!(self.punct_at(i + 1), Some("(" | "=" | ";" | "}")) {
                    break;
                }
                is_static |= word == "static";
                getter |= word == "get";
                i += 1;
            }
            if self.is_punct(i, "*") {
                i += 1;
            }
            if let Some(name) = self.ident_at(i)
                && self.is_punct(i + 1, "(")
            {
                let member = self.params(i + 1);
                let close = self.matching(i + 1);
                if name == "constructor" {
                    params = member;
                } else if !name.starts_with('#') {
                    methods.push(Method {
                        name: name.to_string(),
                        params: member,
                        is_static,
                        getter,
                        doc: self.doc_before(first),
                    });
                }
                i = if self.is_punct(close, "{") {
                    self.matching(close)
                } else {
                    close
                };
                continue;
            }
            // A field, a static block, a computed name or anything else:
            // step over it to the next member.
            i = self.skip_member(i, end);
        }
        (params, methods)
    }

    /// The index of the member after the one at `i` in a class body that
    /// ends at `end`: past a `;`, past a body that follows `)` or `static`
    /// (members of a minified class sit side by side), or at a line break
    /// where a new member begins.
    fn skip_member(&self, i: usize, end: usize) -> usize {
        let mut j = i;
        let mut first = true;
        while j < end {
            match self.punct_at(j) {
                Some(";") => return j + 1,
                Some("{") => {
                    let body = self.is_punct(j.wrapping_sub(1), ")")
                        || self.ident_at(j.wrapping_sub(1)) == Some("static");
                    j = self.matching(j);
                    if body {
                        return j;
                    }
                    first = false;
                    continue;
                }
                Some("(" | "[") => {
                    j = self.matching(j);
                    first = false;
                    continue;
                }
                _ => {}
            }
            if !first && self.t[j].newline_before && self.starts_statement(j) {
                return j;
            }
            first = false;
            j += 1;
        }
        end
    }

    /// The parameters of the list whose `(` is at `open`.
    fn params(&self, open: usize) -> Vec<Param> {
        let close = self.matching(open).saturating_sub(1);
        let mut out = Vec::new();
        let mut i = open + 1;
        let mut index = 0;
        while i < close {
            let rest = self.is_punct(i, "...");
            if rest {
                i += 1;
            }
            let name = match self.ident_at(i) {
                Some(w) => w.to_string(),
                // A destructured parameter has no name of its own.
                None if matches!(self.punct_at(i), Some("{" | "[")) => {
                    if self.is_punct(i, "{") {
                        "options".to_string()
                    } else {
                        format!("arg{index}")
                    }
                }
                None => break,
            };
            // To the comma that ends this parameter, at this depth.
            let mut optional = false;
            while i < close && !self.is_punct(i, ",") {
                if self.is_punct(i, "=") {
                    optional = true;
                }
                if matches!(self.punct_at(i), Some("(" | "[" | "{")) {
                    i = self.matching(i);
                } else {
                    i += 1;
                }
            }
            out.push(Param {
                name,
                optional,
                rest,
            });
            index += 1;
            i += 1;
        }
        out
    }

    /// `var|let|const` declarations starting at `kw`: every name bound, and
    /// the index past the statement.
    fn variables(&mut self, kw: usize) -> Option<usize> {
        let doc = self.doc_before(kw);
        let mut i = kw + 1;
        // `let` alone is also a name in sloppy code; a declaration names
        // something next.
        if self.ident_at(i).is_none() && !matches!(self.punct_at(i), Some("{" | "[")) {
            return None;
        }
        loop {
            if let Some(name) = self.ident_at(i) {
                let name = name.to_string();
                let (line, col) = (self.t[i].line, self.t[i].col);
                i += 1;
                let kind = if self.is_punct(i, "=") {
                    self.value_kind(i + 1)
                } else {
                    NameKind::Value
                };
                self.add(name, kind, line, col, doc.clone());
            } else if matches!(self.punct_at(i), Some("{" | "[")) {
                let end = self.matching(i);
                for (name, line, col) in self.pattern_names(i, end) {
                    self.add(name, NameKind::Value, line, col, doc.clone());
                }
                i = end;
            } else {
                return Some(i);
            }
            // The initialiser, to the comma that starts the next declarator
            // or the end of the statement.
            while i < self.t.len() {
                // A line break where a statement may begin ends it (after
                // `=` it cannot: an operator ends no operand).
                if self.t[i].newline_before && self.starts_statement(i) {
                    return Some(i);
                }
                match self.punct_at(i) {
                    Some(",") => {
                        i += 1;
                        break;
                    }
                    Some(";") => return Some(i + 1),
                    Some("(" | "[" | "{") => {
                        i = self.matching(i);
                        continue;
                    }
                    Some("}" | ")" | "]") => return Some(i),
                    _ => {}
                }
                i += 1;
            }
            if i >= self.t.len() {
                return Some(i);
            }
        }
    }

    /// Whether the token at `i`, after a line break, begins a new statement
    /// rather than continuing an expression.
    fn starts_statement(&self, i: usize) -> bool {
        let prev_ends_operand = match self.t.get(i.wrapping_sub(1)).map(|t| &t.tok) {
            Some(Tok::Ident(_) | Tok::Str | Tok::Num | Tok::Regex | Tok::Template) => true,
            Some(Tok::Punct(p)) => matches!(*p, ")" | "]" | "}" | "++" | "--"),
            None => true,
        };
        let starts_operand = match &self.t[i].tok {
            Tok::Ident(w) => !matches!(w.as_str(), "in" | "of" | "instanceof"),
            Tok::Str | Tok::Num | Tok::Regex | Tok::Template => true,
            Tok::Punct(p) => matches!(*p, "{" | "!" | "~" | "++" | "--"),
        };
        prev_ends_operand && starts_operand
    }

    /// The names a destructuring pattern between `open` and `end` binds.
    fn pattern_names(&self, open: usize, end: usize) -> Vec<(String, usize, usize)> {
        let mut out = Vec::new();
        let mut i = open + 1;
        while i < end.saturating_sub(1) {
            match &self.t[i].tok {
                Tok::Punct("=") => {
                    // A default: skip its expression.
                    i += 1;
                    while i < end - 1 && !matches!(self.punct_at(i), Some("," | "}" | "]")) {
                        if matches!(self.punct_at(i), Some("(" | "[" | "{")) {
                            i = self.matching(i);
                        } else {
                            i += 1;
                        }
                    }
                }
                Tok::Ident(w)
                    if !self.is_punct(i + 1, ":")
                        && !self.is_punct(i.wrapping_sub(1), ".")
                        && matches!(self.punct_at(i + 1), Some("," | "}" | "]" | "=")) =>
                {
                    out.push((w.clone(), self.t[i].line, self.t[i].col));
                    i += 1;
                }
                _ => i += 1,
            }
        }
        out
    }

    /// What the initialiser starting at `i` makes: a function when it is a
    /// `function`/`async function`/arrow/`class` expression, else a value.
    fn value_kind(&self, i: usize) -> NameKind {
        let mut at = i;
        let mut is_async = false;
        if self.ident_at(at) == Some("async")
            && !self.t.get(at + 1).is_some_and(|t| t.newline_before)
        {
            is_async = true;
            at += 1;
        }
        match (self.ident_at(at), self.punct_at(at)) {
            (Some("function"), _) => {
                let mut p = at + 1;
                if self.is_punct(p, "*") {
                    p += 1;
                }
                if self.ident_at(p).is_some() {
                    p += 1;
                }
                if self.is_punct(p, "(") {
                    return NameKind::Function {
                        params: self.params(p),
                        is_async,
                    };
                }
                NameKind::Value
            }
            (Some("class"), _) => {
                let mut p = at + 1;
                while p < self.t.len() && !self.is_punct(p, "{") {
                    p += 1;
                }
                if p >= self.t.len() {
                    return NameKind::Value;
                }
                let end = self.matching(p);
                let (params, methods) = self.class_body(p, end);
                NameKind::Class { params, methods }
            }
            // `x => …`
            (Some(param), _) if self.is_punct(at + 1, "=>") => NameKind::Function {
                params: vec![Param {
                    name: param.to_string(),
                    optional: false,
                    rest: false,
                }],
                is_async,
            },
            // `(a, b) => …`
            (_, Some("(")) if self.is_punct(self.matching(at), "=>") => NameKind::Function {
                params: self.params(at),
                is_async,
            },
            _ => NameKind::Value,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(src: &str) -> Vec<String> {
        let s = scan(src);
        assert!(s.problems.is_empty(), "{:?}", s.problems);
        s.names.into_iter().map(|n| n.name).collect()
    }

    fn one(src: &str, name: &str) -> Name {
        scan(src)
            .names
            .into_iter()
            .find(|n| n.name == name)
            .unwrap_or_else(|| panic!("no `{name}` in {src}"))
    }

    fn params(kind: &NameKind) -> Vec<(String, bool, bool)> {
        let ps = match kind {
            NameKind::Function { params, .. } | NameKind::Class { params, .. } => params,
            NameKind::Value => panic!("not callable: {kind:?}"),
        };
        ps.iter()
            .map(|p| (p.name.clone(), p.optional, p.rest))
            .collect()
    }

    #[test]
    fn a_function_and_its_parameters() {
        let n = one(
            "function initTilt(node, opts = {}, ...rest) { return 1 }",
            "initTilt",
        );
        assert_eq!(
            params(&n.kind),
            vec![
                ("node".into(), false, false),
                ("opts".into(), true, false),
                ("rest".into(), false, true)
            ]
        );
        assert_eq!((n.line, n.col), (1, 10));
        let n = one("function draw({ max, glare }, [a, b]) {}", "draw");
        assert_eq!(
            params(&n.kind),
            vec![
                ("options".into(), false, false),
                ("arg1".into(), false, false)
            ]
        );
    }

    #[test]
    fn async_functions_and_generators() {
        assert!(matches!(
            one("async function load(url) {}", "load").kind,
            NameKind::Function { is_async: true, .. }
        ));
        assert_eq!(names("function* ids() { yield 1 }"), ["ids"]);
        // `async` on its own line is a name, and the function after it is
        // not async.
        assert!(matches!(
            one("var async\nfunction f() {}", "f").kind,
            NameKind::Function {
                is_async: false,
                ..
            }
        ));
    }

    #[test]
    fn a_class_its_constructor_and_its_methods() {
        let src = "class Tilt extends Base(mixin) {\n  #secret = 1\n  static count = 0\n  max = 15\n  constructor(node, opts) { super() }\n  /** Stop. */\n  destroy() {}\n  static of(n) {}\n  get angle() { return 1 }\n  set angle(v) {}\n  async *frames() {}\n  #hidden() {}\n  static { Tilt.count = 1 }\n  [Symbol.iterator]() {}\n  after() {}\n}";
        let n = one(src, "Tilt");
        assert_eq!(
            params(&n.kind),
            vec![("node".into(), false, false), ("opts".into(), false, false)]
        );
        let NameKind::Class { methods, .. } = &n.kind else {
            panic!()
        };
        let m: Vec<(&str, bool, bool)> = methods
            .iter()
            .map(|m| (m.name.as_str(), m.is_static, m.getter))
            .collect();
        assert_eq!(
            m,
            [
                ("destroy", false, false),
                ("of", true, false),
                ("angle", false, true),
                ("angle", false, false),
                ("frames", false, false),
                ("after", false, false)
            ]
        );
        assert_eq!(methods[0].doc.as_deref(), Some("/** Stop. */"));
    }

    #[test]
    fn a_minified_class_and_minified_functions() {
        let n = one(
            "class A{a(){}b(x,y){}static c(){}}function d(){}var e=1,f=function(g){},h=(i)=>i",
            "A",
        );
        let NameKind::Class { methods, .. } = &n.kind else {
            panic!()
        };
        assert_eq!(
            methods.iter().map(|m| m.name.as_str()).collect::<Vec<_>>(),
            ["a", "b", "c"]
        );
        let s = scan(
            "class A{a(){}b(x,y){}static c(){}}function d(){}var e=1,f=function(g){},h=(i)=>i",
        );
        assert_eq!(
            s.names.iter().map(|n| n.name.as_str()).collect::<Vec<_>>(),
            ["A", "d", "e", "f", "h"]
        );
        assert!(matches!(s.names[3].kind, NameKind::Function { .. }));
        assert!(matches!(s.names[4].kind, NameKind::Function { .. }));
    }

    #[test]
    fn variables_arrows_and_function_expressions() {
        let s = scan(
            "const money = (n, cur = \"EUR\") => n\nlet double = x => x * 2\nvar f = async function () {}\nconst LIMIT = 10, other = [1, 2]\nconst Cls = class { go() {} }",
        );
        let got: Vec<(&str, &str)> = s
            .names
            .iter()
            .map(|n| {
                (
                    n.name.as_str(),
                    match n.kind {
                        NameKind::Function { .. } => "fn",
                        NameKind::Class { .. } => "class",
                        NameKind::Value => "value",
                    },
                )
            })
            .collect();
        assert_eq!(
            got,
            [
                ("money", "fn"),
                ("double", "fn"),
                ("f", "fn"),
                ("LIMIT", "value"),
                ("other", "value"),
                ("Cls", "class")
            ]
        );
        assert_eq!(
            params(&s.names[0].kind),
            vec![("n".into(), false, false), ("cur".into(), true, false)]
        );
    }

    #[test]
    fn destructuring_binds_every_name_it_names() {
        assert_eq!(
            names(
                "const { a, b: renamed, c = f(x, y), d: { e }, ...rest } = obj\nconst [first, , third = 3] = list"
            ),
            ["a", "renamed", "c", "e", "rest", "first", "third"]
        );
    }

    #[test]
    fn what_a_function_or_block_declares_stays_private() {
        assert_eq!(
            names(
                "function outer() { function inner() {} const x = 1 }\nif (on) { var y = 1; let z = 2 }\n(function () { const hidden = 1 })()\nconst o = { method() {}, key: function () {} }"
            ),
            ["outer", "o"]
        );
    }

    #[test]
    fn window_and_global_this_assignments_at_any_depth() {
        let s = scan(
            "(function () {\n  function helper() {}\n  /** Tilt it. */\n  window.initTilt = function (node) { helper() }\n  globalThis.VERSION = \"1\"\n  window.a.b = 1\n  if (window.x == 1) {}\n})()",
        );
        assert_eq!(
            s.names.iter().map(|n| n.name.as_str()).collect::<Vec<_>>(),
            ["initTilt", "VERSION"]
        );
        assert!(matches!(s.names[0].kind, NameKind::Function { .. }));
        assert_eq!(s.names[0].doc.as_deref(), Some("/** Tilt it. */"));
    }

    #[test]
    fn a_regex_and_a_division_are_told_apart() {
        // Each would close a brace or open a string if misread.
        assert_eq!(names("const r = /[}/]\"/g\nfunction a() {}"), ["r", "a"]);
        assert_eq!(
            names("const half = (w + h) / 2 / 1\nconst q = list[0] / 3\nfunction b() {}"),
            ["half", "q", "b"]
        );
        assert_eq!(
            names("function c(s) { return /}/.test(s) }\nfunction d() {}"),
            ["c", "d"]
        );
        assert_eq!(names("if (x) {}\n/{/.test(s)\nfunction e() {}"), ["e"]);
        assert_eq!(names("let n = 1\nn++ / 2\nfunction f() {}"), ["n", "f"]);
    }

    #[test]
    fn strings_templates_and_comments_hide_their_braces() {
        assert_eq!(
            names(
                "const a = \"{ function no() {} \\\" }\"\nconst b = '}'\nconst c = `x ${ {k: `}`}.k } y ${`${1}`}`\n// function no() {\n/* class No { */\nfunction yes() {}"
            ),
            ["a", "b", "c", "yes"]
        );
    }

    #[test]
    fn a_line_break_ends_a_declaration_where_a_statement_may_begin() {
        assert_eq!(
            names("const a = 1\nfunction b() {}\nlet c = a\n  + 2, d = 3\nvar e"),
            ["a", "b", "c", "d", "e"]
        );
    }

    #[test]
    fn only_a_doc_comment_is_documentation() {
        assert_eq!(
            one("/** Draws. */\nfunction draw() {}", "draw")
                .doc
                .as_deref(),
            Some("/** Draws. */")
        );
        assert_eq!(one("/* not docs */\nfunction draw() {}", "draw").doc, None);
        assert_eq!(
            one("/** Draws. */\nconst x = 1\nfunction draw() {}", "draw").doc,
            None
        );
    }

    #[test]
    fn a_module_is_noticed_and_a_dynamic_import_is_not_one() {
        assert_eq!(scan("export function f() {}").module_at, Some((1, 1)));
        assert_eq!(
            scan("const x = 1\nimport { y } from \"./y.js\"").module_at,
            Some((2, 1))
        );
        assert_eq!(
            scan("const m = import(\"./m.js\")\nfunction f() { return import.meta }").module_at,
            None
        );
    }

    #[test]
    fn what_cannot_be_read_is_said_with_where() {
        let s = scan("function ok() {}\nconst broken = \"never closed\n");
        assert_eq!(s.problems.len(), 1, "{:?}", s.problems);
        assert_eq!((s.problems[0].line, s.problems[0].col), (2, 16));
        assert!(!scan("function f( {").problems.is_empty());
        assert!(!scan("const t = `${1").problems.is_empty());
    }

    #[test]
    fn numbers_hashbangs_and_unicode_names() {
        assert_eq!(
            names(
                "#!/usr/bin/env node\nconst a = 1e-3, b = 0x1F, c = 1_000n, d = .5\nfunction café() {}"
            ),
            ["a", "b", "c", "d", "café"]
        );
    }
}
