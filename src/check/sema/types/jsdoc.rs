//! A project script's JSDoc types, read as the checker's.

use super::Type;

/// What `Form(bind: form)` binds: whether every control is valid, the
/// values by field name, and `reset()`.
/// What a JSDoc type expression means here, with what it cannot say
/// collected in `problems`.
fn read(text: &str, problems: &mut Vec<String>) -> Type {
    /// Split `text` at `sep` where no bracket is open.
    fn split_top(text: &str, sep: char) -> Vec<&str> {
        let mut parts = Vec::new();
        let (mut depth, mut start) = (0i32, 0);
        let mut prev = '\0';
        for (i, c) in text.char_indices() {
            match c {
                '(' | '[' | '{' | '<' => depth += 1,
                // `=>` closes nothing.
                '>' if prev == '=' => {}
                ')' | ']' | '}' | '>' => depth -= 1,
                _ if c == sep && depth == 0 => {
                    parts.push(&text[start..i]);
                    start = i + c.len_utf8();
                }
                _ => {}
            }
            prev = c;
        }
        parts.push(&text[start..]);
        parts
    }
    /// The index just past the bracket closing the one `text` opens with.
    fn close_of(text: &str) -> Option<usize> {
        let mut depth = 0;
        for (i, c) in text.char_indices() {
            match c {
                '(' | '[' | '{' | '<' => depth += 1,
                ')' | ']' | '}' | '>' => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(i + 1);
                    }
                }
                _ => {}
            }
        }
        None
    }
    /// `name: T` / `name?: T` / `T`, as a parameter or a field.
    fn field(text: &str, problems: &mut Vec<String>) -> (String, bool, Type) {
        let text = text.trim();
        match split_top(text, ':').as_slice() {
            [name, ty] => {
                let optional = name.trim().ends_with('?');
                let name = name.trim().trim_end_matches('?').trim().to_string();
                (name, optional, read(ty, problems))
            }
            _ => (String::new(), false, read(text, problems)),
        }
    }

    let t = text.trim();
    let t = t.strip_prefix("...").unwrap_or(t).trim();
    if t.is_empty() {
        return Type::Any;
    }
    if let Some(inner) = t.strip_prefix('?') {
        return Type::optional(read(inner, problems));
    }
    let t = t.strip_prefix('!').unwrap_or(t);
    let alternatives = split_top(t, '|');
    if alternatives.len() > 1 {
        let (nulls, others): (Vec<&str>, Vec<&str>) = alternatives
            .iter()
            .map(|a| a.trim())
            .partition(|a| matches!(*a, "null" | "undefined"));
        // `string | number`: the language has no union to say it with.
        let joined = if let [one] = others.as_slice() {
            read(one, problems)
        } else {
            problems.push(format!(
                "`{t}` is one of several types, and the language has no unions: use two functions, or `{{*}}` for any JSON"
            ));
            Type::Error
        };
        return if nulls.is_empty() {
            joined
        } else {
            Type::optional(joined)
        };
    }
    // `(a: T) => R`, or `(T)` for grouping.
    if t.starts_with('(')
        && let Some(end) = close_of(t)
    {
        let rest = t[end..].trim();
        if let Some(ret) = rest.strip_prefix("=>") {
            let inside = &t[1..end - 1];
            let params = if inside.trim().is_empty() {
                Vec::new()
            } else {
                split_top(inside, ',')
                    .into_iter()
                    .map(|p| field(p, problems).2)
                    .collect()
            };
            return Type::Func(params, Box::new(read(ret, problems)));
        }
        if rest.is_empty() {
            return read(&t[1..end - 1], problems);
        }
    }
    // `function(T, U): R`
    if let Some(after) = t.strip_prefix("function")
        && after.trim_start().starts_with('(')
    {
        let after = after.trim_start();
        if let Some(end) = close_of(after) {
            let inside = &after[1..end - 1];
            let params = if inside.trim().is_empty() {
                Vec::new()
            } else {
                split_top(inside, ',')
                    .into_iter()
                    .map(|p| field(p, problems).2)
                    .collect()
            };
            let ret = after[end..]
                .trim()
                .strip_prefix(':')
                .map_or(Type::Any, |r| read(r, problems));
            return Type::Func(params, Box::new(ret));
        }
    }
    // `{ a: T, b?: U }`: a shape of the fields written; a method in it
    // (`destroy(): void`) is a field the checker takes as given.
    if t.starts_with('{') && t.ends_with('}') {
        let fields = split_top(&t[1..t.len() - 1], ',')
            .into_iter()
            .filter(|f| !f.trim().is_empty())
            .map(|f| {
                let f = f.trim();
                if let Some(paren) = f.find('(')
                    && f[..paren]
                        .chars()
                        .all(|c| c.is_alphanumeric() || c == '_' || c == '$')
                {
                    return (f[..paren].to_string(), Type::Any);
                }
                let (name, optional, ty) = field(f, problems);
                (name, if optional { Type::optional(ty) } else { ty })
            })
            .filter(|(name, _)| !name.is_empty())
            .collect::<Vec<_>>();
        return Type::Shape(fields);
    }
    if let Some(element) = t.strip_suffix("[]") {
        return Type::list(read(element, problems));
    }
    // `Name<T>` and Closure's `Name.<T>`.
    if let Some(open) = t.find('<')
        && t.ends_with('>')
    {
        let name = t[..open].trim_end_matches('.');
        let args = split_top(&t[open + 1..t.len() - 1], ',');
        return match (name, args.as_slice()) {
            ("Array", [element]) => Type::list(read(element, problems)),
            // What a call awaits is the value; an action awaits it.
            ("Promise", [value]) => read(value, problems),
            // `Object<string, T>`, `Record<string, T>`: text keys, and the
            // values the last says.
            ("Object" | "Record" | "Map", [.., value]) => {
                Type::Dict(Box::new(Type::String), Box::new(read(value, problems)))
            }
            _ => Type::Any,
        };
    }
    match t {
        "string" | "String" => Type::String,
        "number" | "Number" => Type::Number,
        "boolean" | "Boolean" | "bool" => Type::Bool,
        // What could be anything is JSON: plain data, matched or decoded.
        "*" | "any" | "unknown" | "object" | "Object" => Type::Enum(super::ty::JSON.to_string()),
        "Array" | "array" => {
            problems.push(
                "`Array` does not say what it holds: write `{string[]}`, `{Row[]}`".to_string(),
            );
            Type::Error
        }
        "void" | "undefined" | "null" => Type::Null,
        // An element of the page, whatever kind.
        "Element" | "HTMLElement" | "Node" | "SVGElement" => Type::Host("Element".to_string()),
        n if n.starts_with("HTML") && n.ends_with("Element") => Type::Host("Element".to_string()),
        // A name: a class or a `@typedef` of the project's, or an object the
        // browser hands over — what the caller resolves.
        n if n.chars().next().is_some_and(char::is_uppercase)
            && n.chars().all(|c| c.is_alphanumeric() || c == '_') =>
        {
            Type::Host(n.to_string())
        }
        _ => Type::Any,
    }
}

/// What a JSDoc type expression means here. A union, or an `Array` that
/// does not say what it holds, is a type the language cannot say: it is
/// unknown, and `jsdoc_problems` says why.
pub fn jsdoc_type(text: &str) -> Type {
    read(text, &mut Vec::new())
}

/// Why a JSDoc type expression is one the language cannot say, if it is.
pub(super) fn jsdoc_problems(text: &str) -> Vec<String> {
    let mut problems = Vec::new();
    read(text, &mut problems);
    problems
}
