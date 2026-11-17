//! The YAML an OpenAPI document is written in, read without a dependency:
//! block mappings and sequences, flow `[…]` and `{…}`, plain, quoted and
//! block (`|`, `>`) scalars, comments, and the core schema's nulls,
//! booleans and numbers. What a specification has no use for — anchors and
//! aliases, tags, complex keys, several documents in one file — is refused
//! with the one line that turns the file into JSON, which reads the same.

use super::doc::{Object, ReadError, Value};

/// The command that writes a YAML file as JSON, for what this reader does
/// not take.
const CONVERT: &str = "convert the file to JSON first, which reads the same: `python3 -c 'import sys, yaml, json; json.dump(yaml.safe_load(sys.stdin), sys.stdout)' < openapi.yaml > openapi.json`";

/// Read `text` as one YAML document.
///
/// # Errors
///
/// What the reader cannot read, and the line it is on.
pub(super) fn read(text: &str) -> Result<Value, ReadError> {
    let mut reader = Reader::new(text)?;
    let value = reader.node(-1)?;
    reader.skip_blank();
    if let Some(line) = reader.lines.get(reader.at) {
        return Err(reader.fail_at(
            line.no,
            line.indent + 1,
            "this line is less indented than what it follows, or is in the wrong place",
        ));
    }
    Ok(value)
}

#[derive(Clone)]
struct Line {
    /// Its number in the file, from 1.
    no: usize,
    indent: usize,
    /// Everything after the indentation.
    text: String,
}

struct Reader {
    lines: Vec<Line>,
    at: usize,
}

impl Reader {
    fn new(text: &str) -> Result<Self, ReadError> {
        let mut lines = Vec::new();
        let mut started = false;
        for (i, raw) in text.lines().enumerate() {
            let no = i + 1;
            let raw = raw.trim_end_matches('\r');
            let body = raw.trim_start_matches(' ');
            let indent = raw.len() - body.len();
            if body.starts_with('\t') && !strip_comment(body).trim().is_empty() {
                return Err(ReadError {
                    message: "a tab indents this line; YAML indents with spaces".to_string(),
                    line: no,
                    col: indent + 1,
                });
            }
            if indent == 0 {
                if body.starts_with('%') {
                    continue;
                }
                if body == "---" || body.starts_with("--- ") {
                    if started {
                        return Err(ReadError {
                            message: format!(
                                "a second document starts here; a specification is one — {CONVERT}"
                            ),
                            line: no,
                            col: 1,
                        });
                    }
                    started = true;
                    let rest = body[3..].trim();
                    if strip_comment(rest).is_empty() {
                        continue;
                    }
                    lines.push(Line {
                        no,
                        indent: 4,
                        text: rest.to_string(),
                    });
                    continue;
                }
                if body == "..." {
                    break;
                }
            }
            if !strip_comment(body).trim().is_empty() {
                started = true;
            }
            lines.push(Line {
                no,
                indent,
                text: body.to_string(),
            });
        }
        Ok(Reader { lines, at: 0 })
    }

    fn fail_at(&self, line: usize, col: usize, message: &str) -> ReadError {
        ReadError {
            message: message.to_string(),
            line,
            col,
        }
    }

    fn blank(line: &Line) -> bool {
        strip_comment(&line.text).trim().is_empty()
    }

    fn skip_blank(&mut self) {
        while self.lines.get(self.at).is_some_and(Self::blank) {
            self.at += 1;
        }
    }

    /// The next line that holds something, without moving to it.
    fn peek(&mut self) -> Option<&Line> {
        self.skip_blank();
        self.lines.get(self.at)
    }

    /// A node indented deeper than `parent`, or null when there is none.
    fn node(&mut self, parent: isize) -> Result<Value, ReadError> {
        let Some(line) = self.peek().cloned() else {
            return Ok(Value::Null);
        };
        if to_signed(line.indent) <= parent {
            return Ok(Value::Null);
        }
        let text = strip_comment(&line.text).trim_end().to_string();
        if is_dash(&text) {
            return self.sequence(line.indent);
        }
        if text.starts_with("? ") || text == "?" {
            return Err(self.fail_at(
                line.no,
                line.indent + 1,
                &format!("a complex key (`? …`) — {CONVERT}"),
            ));
        }
        if entry(&text).is_some() {
            return self.mapping(line.indent);
        }
        self.at += 1;
        self.inline(&line.text, line.no, line.indent, to_signed(line.indent) - 1)
    }

    fn mapping(&mut self, indent: usize) -> Result<Value, ReadError> {
        let mut object = Object::new();
        while let Some(line) = self.peek().cloned() {
            if line.indent < indent {
                break;
            }
            if line.indent > indent {
                return Err(self.fail_at(
                    line.no,
                    line.indent + 1,
                    "this line is indented more than the keys around it",
                ));
            }
            let text = strip_comment(&line.text).trim_end().to_string();
            if is_dash(&text) {
                return Err(self.fail_at(
                    line.no,
                    line.indent + 1,
                    "a list item where a key was expected",
                ));
            }
            if text.starts_with("? ") || text == "?" {
                return Err(self.fail_at(
                    line.no,
                    line.indent + 1,
                    &format!("a complex key (`? …`) — {CONVERT}"),
                ));
            }
            let Some((key, rest_at)) = entry(&line.text) else {
                return Err(self.fail_at(line.no, line.indent + 1, "expected `key: value`"));
            };
            if key == "<<" {
                return Err(self.fail_at(
                    line.no,
                    line.indent + 1,
                    &format!("a merge key (`<<`) — {CONVERT}"),
                ));
            }
            self.at += 1;
            let rest = line.text[rest_at..].to_string();
            let value = if strip_comment(&rest).trim().is_empty() {
                // The value is on the lines below: deeper, or a list at the
                // key's own indentation.
                match self.peek() {
                    Some(next) if next.indent > indent => self.node(to_signed(indent))?,
                    Some(next)
                        if next.indent == indent
                            && is_dash(strip_comment(&next.text).trim_end()) =>
                    {
                        self.sequence(indent)?
                    }
                    _ => Value::Null,
                }
            } else {
                let col = line.indent + rest_at + (rest.len() - rest.trim_start().len());
                self.inline(rest.trim_start(), line.no, col, to_signed(indent))?
            };
            object.insert(key, value);
        }
        Ok(Value::Object(object))
    }

    fn sequence(&mut self, indent: usize) -> Result<Value, ReadError> {
        let mut list = Vec::new();
        while let Some(line) = self.peek().cloned() {
            if line.indent < indent {
                break;
            }
            let text = strip_comment(&line.text).trim_end().to_string();
            if line.indent > indent {
                return Err(self.fail_at(
                    line.no,
                    line.indent + 1,
                    "this line is indented more than the list items around it",
                ));
            }
            if !is_dash(&text) {
                break;
            }
            let rest = &line.text[1..];
            let item = rest.trim_start();
            if strip_comment(item).trim().is_empty() {
                self.at += 1;
                list.push(self.node(to_signed(indent))?);
                continue;
            }
            // `- name: id` is a node that starts two columns in: read the
            // rest of the line as a line of its own, at its column.
            let col = line.indent + 1 + (rest.len() - item.len());
            self.lines[self.at] = Line {
                no: line.no,
                indent: col,
                text: item.to_string(),
            };
            list.push(self.node(to_signed(indent))?);
        }
        Ok(Value::Array(list))
    }

    /// A value written on the line, after a key or a dash, that may go on to
    /// the lines below: a scalar, a quoted one, a block scalar, or a flow
    /// collection. `parent` is the indentation a continuation must exceed.
    fn inline(
        &mut self,
        text: &str,
        no: usize,
        col: usize,
        parent: isize,
    ) -> Result<Value, ReadError> {
        let first = strip_comment(text).trim().to_string();
        match first.chars().next() {
            Some('&' | '*') => {
                return Err(self.fail_at(no, col + 1, &format!("an anchor or alias — {CONVERT}")));
            }
            Some('!') => {
                return Err(self.fail_at(no, col + 1, &format!("a tag — {CONVERT}")));
            }
            Some('|' | '>') => return self.block_scalar(&first, no, col, parent),
            Some('[' | '{') => {
                let mut buf = first.clone();
                while !balanced(&buf) {
                    let Some(next) = self.lines.get(self.at) else {
                        return Err(self.fail_at(no, col + 1, "a `[` or `{` that never closes"));
                    };
                    buf.push(' ');
                    buf.push_str(strip_comment(next.text.trim()).trim());
                    self.at += 1;
                }
                let mut flow = Flow {
                    chars: buf.chars().collect(),
                    at: 0,
                    no,
                    col,
                };
                let value = flow.value()?;
                flow.ws();
                if flow.at < flow.chars.len() {
                    return Err(self.fail_at(no, col + 1, "something follows the closing bracket"));
                }
                return Ok(value);
            }
            Some(q @ ('"' | '\'')) => {
                let mut buf = text.trim_start().to_string();
                while quoted_end(&buf, q).is_none() {
                    let Some(next) = self.lines.get(self.at) else {
                        return Err(self.fail_at(no, col + 1, "a quoted string that never closes"));
                    };
                    buf.push('\n');
                    buf.push_str(&next.text);
                    self.at += 1;
                }
                let end = quoted_end(&buf, q).unwrap_or(buf.len());
                let after = strip_comment(&buf[end..]).trim().to_string();
                if !after.is_empty() {
                    return Err(self.fail_at(no, col + 1, "something follows the closing quote"));
                }
                return unquote(&buf[..end], q)
                    .map(Value::String)
                    .map_err(|m| self.fail_at(no, col + 1, &m));
            }
            _ => {}
        }
        // A plain scalar, which may go on over lines indented deeper than
        // its parent; they are joined by spaces, a blank line by a newline.
        let mut value = first;
        let mut multi = false;
        let mut breaks = 0;
        while let Some(next) = self.lines.get(self.at) {
            let body = next.text.trim();
            if body.is_empty() {
                breaks += 1;
                self.at += 1;
                continue;
            }
            if to_signed(next.indent) <= parent || body.starts_with('#') {
                break;
            }
            let more = strip_comment(body).trim().to_string();
            if entry(&more).is_some() {
                return Err(self.fail_at(
                    next.no,
                    next.indent + 1,
                    "this line is indented more than the keys around it",
                ));
            }
            if breaks > 0 {
                for _ in 0..breaks {
                    value.push('\n');
                }
            } else {
                value.push(' ');
            }
            breaks = 0;
            value.push_str(&more);
            multi = true;
            self.at += 1;
        }
        // Blank lines that ended the scalar belong to what follows.
        while breaks > 0 && self.at > 0 && self.lines[self.at - 1].text.trim().is_empty() {
            self.at -= 1;
            breaks -= 1;
        }
        Ok(if multi {
            Value::String(value)
        } else {
            scalar(&value)
        })
    }

    /// `|`, `|-`, `>+`, `|2`: the lines indented below, kept (`|`) or folded
    /// (`>`), with their trailing breaks clipped, stripped or kept.
    fn block_scalar(
        &mut self,
        header: &str,
        no: usize,
        col: usize,
        parent: isize,
    ) -> Result<Value, ReadError> {
        let folded = header.starts_with('>');
        let mut chomp = ' ';
        let mut indicator = None;
        for c in header[1..].chars() {
            match c {
                '-' | '+' => chomp = c,
                '1'..='9' => indicator = c.to_digit(10),
                c if c.is_whitespace() => break,
                _ => return Err(self.fail_at(no, col + 1, "a block scalar's header is `|` or `>`, then `-` or `+`, then an indentation digit")),
            }
        }
        let base = usize::try_from(parent.max(0)).unwrap_or(0);
        let mut content_indent = indicator.map(|n| base + n as usize);
        let mut lines: Vec<String> = Vec::new();
        while let Some(next) = self.lines.get(self.at) {
            if next.text.trim().is_empty() {
                lines.push(String::new());
                self.at += 1;
                continue;
            }
            let indent = *content_indent.get_or_insert(next.indent);
            if next.indent < indent || to_signed(next.indent) <= parent {
                break;
            }
            lines.push(format!("{}{}", " ".repeat(next.indent - indent), next.text));
            self.at += 1;
        }
        let mut trail = 0;
        while lines.last().is_some_and(String::is_empty) {
            lines.pop();
            trail += 1;
        }
        // Blank lines past the scalar's end belong to what follows.
        let kept = trail;
        let mut out = String::new();
        if folded {
            let mut pending = 0;
            let mut first = true;
            let mut prev_more = false;
            for line in &lines {
                if line.is_empty() {
                    pending += 1;
                    continue;
                }
                let more = line.starts_with(' ');
                if first {
                    out.push_str(&"\n".repeat(pending));
                } else if pending == 0 {
                    out.push(if more || prev_more { '\n' } else { ' ' });
                } else {
                    if more || prev_more {
                        out.push('\n');
                    }
                    out.push_str(&"\n".repeat(pending));
                }
                out.push_str(line);
                pending = 0;
                first = false;
                prev_more = more;
            }
        } else {
            out = lines.join("\n");
        }
        match chomp {
            '-' => {}
            '+' => {
                out.push('\n');
                out.push_str(&"\n".repeat(kept));
            }
            _ if !out.is_empty() => out.push('\n'),
            _ => {}
        }
        Ok(Value::String(out))
    }
}

fn to_signed(n: usize) -> isize {
    isize::try_from(n).unwrap_or(isize::MAX)
}

/// A list item: `- x`, or a `-` alone on its line.
fn is_dash(text: &str) -> bool {
    text == "-" || text.starts_with("- ")
}

/// Everything before a comment: a `#` at the start or after white space,
/// outside quotes.
fn strip_comment(text: &str) -> &str {
    let mut quote: Option<char> = None;
    let mut prev = ' ';
    let mut chars = text.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        match quote {
            Some('"') => {
                if c == '\\' {
                    chars.next();
                } else if c == '"' {
                    quote = None;
                }
            }
            Some(_) => {
                if c == '\'' {
                    if chars.peek().is_some_and(|(_, n)| *n == '\'') {
                        chars.next();
                    } else {
                        quote = None;
                    }
                }
            }
            None => {
                if c == '#' && prev.is_whitespace() {
                    return &text[..i];
                }
                if (c == '"' || c == '\'')
                    && (prev.is_whitespace() || matches!(prev, '[' | '{' | ',' | ':'))
                {
                    quote = Some(c);
                }
            }
        }
        prev = c;
    }
    text
}

/// `key: …` — the key, and where what follows the colon starts. A quoted
/// key is read whole; a plain one ends at the first `: ` or a colon that
/// ends the line.
fn entry(text: &str) -> Option<(String, usize)> {
    let text = strip_comment(text);
    if let Some(q @ ('"' | '\'')) = text.chars().next() {
        let end = quoted_end(text, q)?;
        let after = &text[end..];
        let colon = after.len() - after.trim_start().len();
        let after = after.trim_start();
        if after.starts_with(':')
            && (after.len() == 1 || after[1..].starts_with(char::is_whitespace))
        {
            return Some((unquote(&text[..end], q).ok()?, end + colon + 1));
        }
        return None;
    }
    if text.starts_with(['[', '{', '|', '>', '&', '*', '!']) || is_dash(text) {
        return None;
    }
    let bytes = text.as_bytes();
    for (i, b) in bytes.iter().enumerate() {
        if *b == b':' && (i + 1 == bytes.len() || bytes[i + 1] == b' ') {
            let key = text[..i].trim_end();
            if key.is_empty() {
                return None;
            }
            return Some((key.to_string(), i + 1));
        }
    }
    None
}

/// Where the quoted string that `text` opens with ends, just past its
/// closing quote.
fn quoted_end(text: &str, q: char) -> Option<usize> {
    let mut chars = text.char_indices().skip(1).peekable();
    while let Some((i, c)) = chars.next() {
        if q == '"' && c == '\\' {
            chars.next();
        } else if c == q {
            if q == '\'' && chars.peek().is_some_and(|(_, n)| *n == '\'') {
                chars.next();
                continue;
            }
            return Some(i + c.len_utf8());
        }
    }
    None
}

/// A quoted string's text: `''` is a quote in single quotes, escapes are
/// read in double ones, and a line break inside either folds to a space (a
/// blank line to a newline).
fn unquote(text: &str, q: char) -> Result<String, String> {
    let inner = &text[1..text.len() - 1];
    let folded = fold_lines(inner);
    if q == '\'' {
        return Ok(folded.replace("''", "'"));
    }
    let mut out = String::new();
    let mut chars = folded.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some('r') => out.push('\r'),
            Some('b') => out.push('\u{8}'),
            Some('f') => out.push('\u{c}'),
            Some('0') => out.push('\0'),
            Some('/') => out.push('/'),
            Some(' ') => out.push(' '),
            Some('"') => out.push('"'),
            Some('\\') => out.push('\\'),
            Some(kind @ ('x' | 'u' | 'U')) => {
                let n = match kind {
                    'x' => 2,
                    'u' => 4,
                    _ => 8,
                };
                let hex: String = chars.by_ref().take(n).collect();
                let code = u32::from_str_radix(&hex, 16)
                    .map_err(|_| format!("`\\{kind}{hex}` is not an escape"))?;
                out.push(
                    char::from_u32(code)
                        .ok_or_else(|| format!("`\\{kind}{hex}` is not a character"))?,
                );
            }
            Some(other) => return Err(format!("`\\{other}` is not an escape YAML has")),
            None => out.push('\\'),
        }
    }
    Ok(out)
}

/// A quoted string over several lines: each break a space, a blank line a
/// newline, the white space around a break gone.
fn fold_lines(text: &str) -> String {
    if !text.contains('\n') {
        return text.to_string();
    }
    let lines: Vec<&str> = text.split('\n').collect();
    let mut out = String::new();
    let mut pending = 0;
    for (i, line) in lines.iter().enumerate() {
        let line = if i == 0 {
            line.trim_end()
        } else if i + 1 == lines.len() {
            line.trim_start()
        } else {
            line.trim()
        };
        if i > 0 && line.is_empty() && i + 1 < lines.len() {
            pending += 1;
            continue;
        }
        if i > 0 {
            if pending > 0 {
                out.push_str(&"\n".repeat(pending));
            } else {
                out.push(' ');
            }
        }
        pending = 0;
        out.push_str(line);
    }
    out
}

/// Whether every bracket a flow collection opens is closed, outside quotes.
fn balanced(text: &str) -> bool {
    let mut depth = 0i32;
    let mut quote: Option<char> = None;
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        match quote {
            Some('"') if c == '\\' => {
                chars.next();
            }
            Some(q) if c == q => {
                if q == '\'' && chars.peek() == Some(&'\'') {
                    chars.next();
                } else {
                    quote = None;
                }
            }
            Some(_) => {}
            None => match c {
                '"' | '\'' => quote = Some(c),
                '[' | '{' => depth += 1,
                ']' | '}' => depth -= 1,
                _ => {}
            },
        }
    }
    depth <= 0 && quote.is_none()
}

/// A plain scalar as the core schema reads it: null, a boolean, a number,
/// or a string.
fn scalar(text: &str) -> Value {
    match text {
        "" | "~" | "null" | "Null" | "NULL" => return Value::Null,
        "true" | "True" | "TRUE" => return Value::Bool(true),
        "false" | "False" | "FALSE" => return Value::Bool(false),
        ".inf" | ".Inf" | ".INF" | "+.inf" => return Value::Number(f64::INFINITY),
        "-.inf" | "-.Inf" | "-.INF" => return Value::Number(f64::NEG_INFINITY),
        ".nan" | ".NaN" | ".NAN" => return Value::Number(f64::NAN),
        _ => {}
    }
    if let Some(hex) = text.strip_prefix("0x")
        && let Ok(n) = i64::from_str_radix(hex, 16)
    {
        #[expect(clippy::cast_precision_loss, reason = "JSON numbers are doubles")]
        return Value::Number(n as f64);
    }
    if let Some(oct) = text.strip_prefix("0o")
        && let Ok(n) = i64::from_str_radix(oct, 8)
    {
        #[expect(clippy::cast_precision_loss, reason = "JSON numbers are doubles")]
        return Value::Number(n as f64);
    }
    if is_number(text)
        && let Ok(n) = text.parse::<f64>()
    {
        return Value::Number(n);
    }
    Value::String(text.to_string())
}

/// `[-+]?(\.[0-9]+|[0-9]+(\.[0-9]*)?)([eE][-+]?[0-9]+)?`, the core schema's
/// numbers.
fn is_number(text: &str) -> bool {
    let s = text.strip_prefix(['-', '+']).unwrap_or(text);
    let (mantissa, exponent) = match s.find(['e', 'E']) {
        Some(at) => (&s[..at], Some(&s[at + 1..])),
        None => (s, None),
    };
    let digits = |t: &str| !t.is_empty() && t.bytes().all(|b| b.is_ascii_digit());
    let mantissa_ok = match mantissa.split_once('.') {
        Some((whole, frac)) => {
            (digits(whole) && (frac.is_empty() || digits(frac)))
                || (whole.is_empty() && digits(frac))
        }
        None => digits(mantissa),
    };
    let exponent_ok = exponent.is_none_or(|e| digits(e.strip_prefix(['-', '+']).unwrap_or(e)));
    mantissa_ok && exponent_ok
}

/// A flow collection, `[a, b]` or `{ key: value }`, possibly gathered from
/// several lines.
struct Flow {
    chars: Vec<char>,
    at: usize,
    no: usize,
    col: usize,
}

impl Flow {
    fn fail(&self, message: &str) -> ReadError {
        ReadError {
            message: message.to_string(),
            line: self.no,
            col: self.col + 1,
        }
    }

    fn ws(&mut self) {
        while self.chars.get(self.at).is_some_and(|c| c.is_whitespace()) {
            self.at += 1;
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.at).copied()
    }

    fn value(&mut self) -> Result<Value, ReadError> {
        self.ws();
        match self.peek() {
            Some('[') => {
                self.at += 1;
                let mut list = Vec::new();
                loop {
                    self.ws();
                    match self.peek() {
                        Some(']') => {
                            self.at += 1;
                            break;
                        }
                        None => return Err(self.fail("a `[` that never closes")),
                        _ => {}
                    }
                    list.push(self.value()?);
                    self.ws();
                    match self.peek() {
                        Some(',') => self.at += 1,
                        Some(']') => {}
                        _ => return Err(self.fail("expected `,` or `]` in a list")),
                    }
                }
                Ok(Value::Array(list))
            }
            Some('{') => {
                self.at += 1;
                let mut object = Object::new();
                loop {
                    self.ws();
                    match self.peek() {
                        Some('}') => {
                            self.at += 1;
                            break;
                        }
                        None => return Err(self.fail("a `{` that never closes")),
                        _ => {}
                    }
                    let key = match self.peek() {
                        Some(q @ ('"' | '\'')) => self.quoted(q)?,
                        _ => self.plain(true),
                    };
                    self.ws();
                    let value = if self.peek() == Some(':') {
                        self.at += 1;
                        self.ws();
                        if matches!(self.peek(), Some(',' | '}')) {
                            Value::Null
                        } else {
                            self.value()?
                        }
                    } else {
                        Value::Null
                    };
                    object.insert(key, value);
                    self.ws();
                    match self.peek() {
                        Some(',') => self.at += 1,
                        Some('}') => {}
                        _ => return Err(self.fail("expected `,` or `}` in a mapping")),
                    }
                }
                Ok(Value::Object(object))
            }
            Some(q @ ('"' | '\'')) => self.quoted(q).map(Value::String),
            Some('&' | '*') => Err(self.fail(&format!("an anchor or alias — {CONVERT}"))),
            Some('!') => Err(self.fail(&format!("a tag — {CONVERT}"))),
            _ => Ok(scalar(&self.plain(false))),
        }
    }

    fn quoted(&mut self, q: char) -> Result<String, ReadError> {
        let rest: String = self.chars[self.at..].iter().collect();
        let end =
            quoted_end(&rest, q).ok_or_else(|| self.fail("a quoted string that never closes"))?;
        let text = unquote(&rest[..end], q).map_err(|m| self.fail(&m))?;
        self.at += rest[..end].chars().count();
        Ok(text)
    }

    /// A plain scalar inside a flow collection: up to a `,`, a closing
    /// bracket, or — a key — a `:` followed by a space.
    fn plain(&mut self, key: bool) -> String {
        let start = self.at;
        while let Some(c) = self.peek() {
            if matches!(c, ',' | ']' | '}') {
                break;
            }
            if c == ':'
                && (key
                    || self
                        .chars
                        .get(self.at + 1)
                        .is_none_or(|n| n.is_whitespace() || matches!(n, ',' | ']' | '}')))
            {
                break;
            }
            self.at += 1;
        }
        self.chars[start..self.at]
            .iter()
            .collect::<String>()
            .trim()
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn json(text: &str) -> serde_json::Value {
        read(text).unwrap_or_else(|e| panic!("{e}")).to_json()
    }

    #[test]
    fn mappings_sequences_and_scalars() {
        let got = json(
            "openapi: 3.0.3\ninfo:\n  title: Pets  # the name\n  version: '1.0'\nservers:\n  - url: https://api.example.com/v1\n    description: live\n  - url: /v2\ntags: [pets, 'store', \"users\"]\nempty:\nflag: true\ncount: 12\nratio: 1.5\nnothing: ~\n",
        );
        assert_eq!(
            got,
            serde_json::json!({
                "openapi": "3.0.3",
                "info": { "title": "Pets", "version": "1.0" },
                "servers": [{ "url": "https://api.example.com/v1", "description": "live" }, { "url": "/v2" }],
                "tags": ["pets", "store", "users"],
                "empty": null, "flag": true, "count": 12, "ratio": 1.5, "nothing": null
            })
        );
    }

    #[test]
    fn keys_keep_the_order_they_were_written_in() {
        let v = read("zeta: 1\nalpha: 2\nmid: 3\n").unwrap();
        let keys: Vec<&String> = v.as_object().unwrap().keys().collect();
        assert_eq!(keys, ["zeta", "alpha", "mid"]);
    }

    #[test]
    fn a_list_at_its_keys_indentation_and_nested_items() {
        let got = json(
            "required:\n- id\n- name\nparameters:\n  - name: id\n    in: path\n    schema:\n      type: string\n  -\n    name: q\n",
        );
        assert_eq!(
            got,
            serde_json::json!({
                "required": ["id", "name"],
                "parameters": [{ "name": "id", "in": "path", "schema": { "type": "string" } }, { "name": "q" }]
            })
        );
    }

    #[test]
    fn paths_status_codes_and_braces_are_keys() {
        let got = json(
            "paths:\n  /users/{id}:\n    get:\n      responses:\n        '200':\n          description: ok\n        404:\n          description: gone\n",
        );
        assert_eq!(
            got,
            serde_json::json!({ "paths": { "/users/{id}": { "get": { "responses": {
                "200": { "description": "ok" }, "404": { "description": "gone" }
            } } } } })
        );
    }

    #[test]
    fn block_scalars_keep_or_fold_their_lines() {
        let got = json(
            "a: |\n  one\n  two\n\nb: >\n  folded\n  line\n\n  next\nc: |-\n  stripped\nd: plain that\n  goes on\ne: \"a \\u00e9 \\\"q\\\"\"\nf: 'it''s'\n",
        );
        assert_eq!(
            got,
            serde_json::json!({
                "a": "one\ntwo\n", "b": "folded line\nnext\n", "c": "stripped",
                "d": "plain that goes on", "e": "a é \"q\"", "f": "it's"
            })
        );
    }

    #[test]
    fn flow_collections_over_several_lines() {
        let got = json(
            "enum: [\n  prod,\n  test, # a comment\n]\nexample: { id: 7, name: \"Ada\", tags: [a, b] }\n",
        );
        assert_eq!(
            got,
            serde_json::json!({ "enum": ["prod", "test"], "example": { "id": 7, "name": "Ada", "tags": ["a", "b"] } })
        );
    }

    #[test]
    fn a_document_marker_and_comments() {
        let got = json("# a spec\n---\nopenapi: 3.1.0 # version\nurl: http://x.example/a#b\n");
        assert_eq!(
            got,
            serde_json::json!({ "openapi": "3.1.0", "url": "http://x.example/a#b" })
        );
    }

    #[test]
    fn what_a_specification_has_no_use_for_is_refused_with_the_conversion() {
        for (text, line) in [
            ("a: &base\n  x: 1\nb: *base\n", 1),
            ("a: !!str 1\n", 1),
            ("a: 1\n---\nb: 2\n", 2),
            ("? complex\n: value\n", 1),
        ] {
            let e = read(text).unwrap_err();
            assert!(e.message.contains("json.dump"), "{text}: {e}");
            assert_eq!(e.line, line, "{text}: {e}");
        }
    }

    #[test]
    fn a_tab_or_a_stray_indentation_says_where() {
        assert_eq!(read("a:\n\tb: 1\n").unwrap_err().line, 2);
        assert_eq!(read("a: 1\n    b: 2\n").unwrap_err().line, 2);
    }

    #[test]
    fn numbers_versions_and_words() {
        let got = json("a: 3.0.3\nb: 1e3\nc: -2\nd: 0x1F\ne: yes\nf: '12'\ng: .5\n");
        assert_eq!(
            got,
            serde_json::json!({ "a": "3.0.3", "b": 1000, "c": -2, "d": 31, "e": "yes", "f": "12", "g": 0.5 })
        );
    }
}
