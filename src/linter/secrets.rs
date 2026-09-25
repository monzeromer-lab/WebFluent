//! `env` names a page may not read.
//!
//! `env.X` is replaced by its value at build time and the value lands in
//! the bundle, where anyone who opens the site can read it. That is right
//! for an API base or a feature switch and wrong for an API key, and
//! nothing in the language said which was which — so a key in `env` was a
//! leak with no message attached to it.
//!
//! Now a name is public when it says so (`PUBLIC_…`) or when the config
//! lists it, and any other name read from a page, a component, a store or
//! an `api` block is an error at the line that reads it. The values are
//! still there for `wf render`, which runs on a server.

use crate::config::ProjectConfig;
use crate::error::Diagnostic;
use std::path::Path;

/// Every `env.NAME` in the project's sources that a page may not read.
///
/// It reads the **tokens**, not the text: `env.KEY` inside a string is a
/// page writing about `env`, which the documentation site does on a whole
/// chapter, and only a splice inside that string would be a page reading
/// it — which the lexer turns into tokens too, so that one is still found.
pub fn lint_env(project_dir: &Path, config: &ProjectConfig, files: &[String]) -> Vec<Diagnostic> {
    let mut seen: Vec<&String> = Vec::new();
    let mut out = Vec::new();
    for file in files {
        if seen.contains(&file) {
            continue;
        }
        seen.push(file);
        let Ok(text) = std::fs::read_to_string(project_dir.join(file)) else {
            continue;
        };
        let Ok(tokens) = crate::syntax::tokens(&text, file) else {
            continue; // a file that does not lex has a parse error of its own
        };
        for (name, line, column) in env_names(&tokens) {
            if config.env_is_public(&name) {
                continue;
            }
            let known = config.env.contains_key(&name);
            out.push(
                Diagnostic::new(
                    format!("`env.{name}` is not public, and a page reads what is in the bundle"),
                    file,
                    line,
                    column,
                )
                .with_hint(if known {
                    "Rename it `PUBLIC_…`, or add it to `public_env` in webfluent.app.json — either way, say that anyone may read it"
                } else {
                    "Name it `PUBLIC_…` if the page may read it; a value the page must not read belongs behind a request"
                }),
            );
        }
    }
    out
}

/// The `env.NAME` references among `tokens`, each with where it is written.
///
/// Two things are not a read of the build's `env`. A **method call** —
/// `env.filter(…)` on a list a page happens to have called `env` — and a
/// name the program **declared for itself**: `state env = "production"`
/// shadows the build's for as long as the declaration that holds it. A
/// real project had both on the first run, and each one reported as a
/// leaked secret is a reason to stop reading the warnings.
fn env_names(tokens: &[crate::lexer::Token]) -> Vec<(String, usize, usize)> {
    use crate::lexer::TokenType;
    let mut out = Vec::new();
    // The brace depth a shadowing declaration was made at; `env` is the
    // program's own until the block that declared it closes.
    let mut shadowed_at: Option<usize> = None;
    let mut depth: usize = 0;
    for (i, token) in tokens.iter().enumerate() {
        match &token.token_type {
            TokenType::OpenBrace => depth += 1,
            TokenType::CloseBrace => {
                depth = depth.saturating_sub(1);
                if shadowed_at.is_some_and(|at| depth < at) {
                    shadowed_at = None;
                }
            }
            _ => {}
        }
        // A string keeps its splices inside it until the parser splits
        // them, and a splice is a page reading the value.
        if let TokenType::StringLiteral(lit) = &token.token_type {
            if lit.kind != crate::lexer::token::StringKind::Raw && shadowed_at.is_none() {
                for name in spliced(&lit.spelling) {
                    out.push((name, token.line, token.column));
                }
            }
            continue;
        }
        let TokenType::Identifier(word) = &token.token_type else {
            continue;
        };
        // `state env`, `derived env`, `persist env`, `let env`, and a prop
        // or parameter called `env`: the program's own name from here on.
        if declares_env(tokens, i) {
            shadowed_at = Some(depth);
            continue;
        }
        if word != "env"
            || shadowed_at.is_some()
            || !matches!(
                tokens.get(i + 1).map(|t| &t.token_type),
                Some(TokenType::Dot)
            )
        {
            continue;
        }
        // `a.env.KEY` is a field of something else, not the build's `env`.
        if i > 0 && matches!(tokens[i - 1].token_type, TokenType::Dot) {
            continue;
        }
        let Some(TokenType::Identifier(name)) = tokens.get(i + 2).map(|t| &t.token_type) else {
            continue;
        };
        // `env.filter(…)` is a method call on something, not a value read.
        if matches!(
            tokens.get(i + 3).map(|t| &t.token_type),
            Some(TokenType::OpenParen)
        ) {
            continue;
        }
        out.push((name.clone(), token.line, token.column));
    }
    out
}

/// Whether the token at `i` is a declaration of a name called `env`.
fn declares_env(tokens: &[crate::lexer::Token], i: usize) -> bool {
    use crate::lexer::TokenType;
    let word = |at: usize| match tokens.get(at).map(|t| &t.token_type) {
        Some(TokenType::Identifier(w)) => Some(w.as_str()),
        _ => None,
    };
    if word(i) != Some("env") {
        return false;
    }
    // `state env = …`, `derived env = …`, `persist env = …`, `let env = …`
    if matches!(
        word(i.wrapping_sub(1)),
        Some("state" | "derived" | "persist" | "let")
    ) && i > 0
    {
        return true;
    }
    // A prop or a parameter: `env: String`, `_ env: Map`.
    matches!(
        tokens.get(i + 1).map(|t| &t.token_type),
        Some(TokenType::Colon)
    ) && matches!(
        tokens.get(i.wrapping_sub(1)).map(|t| &t.token_type),
        Some(TokenType::OpenParen | TokenType::Comma)
    ) && i > 0
}

/// The `env.NAME` references inside a string's splices.
fn spliced(spelling: &str) -> Vec<String> {
    let mut out = Vec::new();
    let bytes = spelling.as_bytes();
    let mut from = 0;
    while let Some(open) = spelling[from..].find('{') {
        let start = from + open + 1;
        // `\{` is a brace, not a splice.
        if start >= 2 && bytes[start - 2] == b'\\' {
            from = start;
            continue;
        }
        let Some(close) = spelling[start..].find('}') else {
            break;
        };
        let inside = &spelling[start..start + close];
        from = start + close;
        for at in inside.match_indices("env.").map(|(at, _)| at) {
            let before_ok = at == 0 || {
                let c = inside.as_bytes()[at - 1] as char;
                !(c.is_alphanumeric() || c == '_' || c == '.')
            };
            if !before_ok {
                continue;
            }
            let name: String = inside[at + 4..]
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if !name.is_empty() {
                out.push(name);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_reference_is_read_where_it_is_written_and_nowhere_else() {
        let names = |src: &str| {
            let tokens = crate::syntax::tokens(src, "t.wf").expect("lex");
            env_names(&tokens)
                .into_iter()
                .map(|(n, _, _)| n)
                .collect::<Vec<_>>()
        };
        assert_eq!(
            names("page P(path: \"/\") { Text(env.API_KEY) }"),
            ["API_KEY"]
        );
        assert_eq!(
            names("page P(path: \"/\") { Text(env.API ?? env.PUBLIC_API) }"),
            ["API", "PUBLIC_API"]
        );
        // A page writing *about* `env` is not a page reading it — which is
        // what a documentation site does for a whole chapter.
        assert!(names("page P(path: \"/\") { Text(\"env.API_KEY is secret\") }").is_empty());
        assert!(names("page P(path: \"/\") { Code(#\"env.API_KEY\"#) }").is_empty());
        // A splice inside a string is a page reading it, and is found.
        assert_eq!(
            names("page P(path: \"/\") { Text(\"base {env.API_KEY}\") }"),
            ["API_KEY"]
        );
        // A field of something else is not the build's `env`.
        assert!(names("page P(path: \"/\") { Text(config.env.KEY) }").is_empty());
        // A method call on a list a page happens to have called `env`.
        assert!(names("page P(path: \"/\") { Text(env.filter(x => x)) }").is_empty());
        // And a name the program declared for itself, for as long as the
        // declaration that holds it.
        assert!(names("store S { state env = \"production\"  derived n = env.length }").is_empty());
        assert!(
            names("component C(env: Map) { Text(env.NAME) }").is_empty(),
            "a prop called `env` is the caller's value"
        );
        // Outside that declaration, `env` is the build's again.
        assert_eq!(
            names("store S { state env = \"p\" }\npage P(path: \"/\") { Text(env.API_KEY) }"),
            ["API_KEY"]
        );
    }
}
