//! `wf explain T05`: what a code means, a program that draws it, what the
//! compiler says, and the fix — the guide's entry, in the terminal. With no
//! code, every code and its title.

use crate::diagnostics::codes::{self, CODES};
use crate::diagnostics::fixes::nearest;
use crate::error::{Result, WebFluentError};

/// The diagnostics chapter, which holds an entry for every code (the docs
/// test sees to it).
const CHAPTER: &str = include_str!("../../md-docs/39-diagnostics.md");

pub fn run_explain(code: Option<&str>) -> Result<()> {
    match code {
        None => {
            print!("{}", index());
            Ok(())
        }
        Some(code) => match explanation(code) {
            Some(text) => {
                print!("{text}");
                Ok(())
            }
            None => {
                let wanted = code.to_uppercase();
                let near = nearest(&wanted, CODES.iter().map(|c| c.code))
                    .map(|c| format!(" — did you mean `{c}`?"))
                    .unwrap_or_default();
                Err(WebFluentError::ConfigError(format!(
                    "`{code}` is no code{near} `wf explain` with no code lists them all"
                )))
            }
        },
    }
}

/// Every code, its severity and its title, by family.
pub fn index() -> String {
    let mut out = String::new();
    let mut family = "";
    for c in CODES {
        let f = codes::family(c.code);
        if f != family {
            if !family.is_empty() {
                out.push('\n');
            }
            family = f;
        }
        out.push_str(&format!(
            "{:<5} {:<8} {}\n",
            c.code,
            c.severity.word(),
            plain(c.title)
        ));
    }
    out.push_str(
        "\n`wf explain CODE` shows one: what it means, a program that draws it, and the fix.\n",
    );
    out
}

/// A code's entry: its title and severity, its summary, the guide's section
/// for it, and where the guide is.
pub fn explanation(code: &str) -> Option<String> {
    let info = codes::info(&code.to_uppercase())?;
    let mut out = format!(
        "{} — {} ({})\n\n{}\n",
        info.code,
        plain(info.title),
        info.severity.word(),
        plain(info.summary)
    );
    if let Some(section) = section(info.code) {
        out.push('\n');
        out.push_str(&terminal(section));
    }
    out.push_str(&format!("\nIn the guide: {}\n", codes::docs_url(info.code)));
    Some(out)
}

/// The text of the chapter's `### CODE — …` section, without its heading.
fn section(code: &str) -> Option<&'static str> {
    let heading = format!("### {code} — ");
    let start = CHAPTER.find(&heading)?;
    let body = &CHAPTER[start..];
    let body = &body[body.find('\n')? + 1..];
    let end = body
        .match_indices("\n#")
        .map(|(i, _)| i)
        .find(|&i| body[i + 1..].starts_with("## ") || body[i + 1..].starts_with("### "))
        .unwrap_or(body.len());
    Some(body[..end].trim())
}

/// Markdown made for a terminal: code blocks indented and unfenced, the
/// emphasis marks off.
fn terminal(markdown: &str) -> String {
    let mut out = String::new();
    let mut in_code = false;
    for line in markdown.lines() {
        if line.trim_start().starts_with("```") {
            in_code = !in_code;
            continue;
        }
        if in_code {
            out.push_str("    ");
            out.push_str(line);
        } else {
            out.push_str(&line.replace("**", ""));
        }
        out.push('\n');
    }
    out
}

/// A title without its Markdown backticks.
fn plain(text: &str) -> String {
    text.replace('`', "")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_code_explains_itself() {
        for c in CODES {
            let text = explanation(c.code).unwrap();
            assert!(text.starts_with(&format!("{} — ", c.code)), "{text}");
            assert!(
                section(c.code).is_some(),
                "{} has no section in the chapter",
                c.code
            );
        }
    }

    #[test]
    fn an_entry_shows_its_program_unfenced() {
        let text = explanation("e101").unwrap();
        assert!(text.contains("    page P("), "{text}");
        assert!(!text.contains("```"), "{text}");
        assert!(text.contains("In the guide: https://"), "{text}");
    }
}
