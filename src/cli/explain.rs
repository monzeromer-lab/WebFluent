//! `wf explain T05`: what a code means, a program that draws it, what the
//! compiler says, and the fix — the guide's entry, in the terminal. With no
//! code, every code and its title. With `--json`, the same as data, for the
//! documentation site and any other tool.

use crate::diagnostics::codes::{self, CODES};
use crate::diagnostics::error::{Result, WebFluentError};
use crate::diagnostics::fixes::nearest;

pub(crate) fn run_explain(code: Option<&str>, json: bool) -> Result<()> {
    match code {
        None if json => {
            let all: Vec<serde_json::Value> = CODES.iter().map(|c| entry(c.code)).collect();
            println!("{}", serde_json::to_string_pretty(&all).unwrap_or_default());
            Ok(())
        }
        None => {
            print!("{}", index());
            Ok(())
        }
        Some(code) => {
            if json && codes::info(&code.to_uppercase()).is_some() {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&entry(&code.to_uppercase())).unwrap_or_default()
                );
                Ok(())
            } else if let Some(text) = explanation(code) {
                print!("{text}");
                Ok(())
            } else {
                let wanted = code.to_uppercase();
                let near = nearest(&wanted, CODES.iter().map(|c| c.code))
                    .map(|c| format!(" — did you mean `{c}`?"))
                    .unwrap_or_default();
                Err(WebFluentError::config(format!(
                    "`{code}` is no code{near} `wf explain` with no code lists them all"
                )))
            }
        }
    }
}

/// Every code, its severity and its title, by family.
pub(crate) fn index() -> String {
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

/// A code as data: what `wf explain CODE` prints, with the guide's section
/// as the Markdown it is written in.
pub(crate) fn entry(code: &str) -> serde_json::Value {
    let Some(info) = codes::info(code) else {
        return serde_json::Value::Null;
    };
    serde_json::json!({
        "code": info.code,
        "family": codes::family(info.code),
        "severity": info.severity.word(),
        "title": info.title,
        "summary": info.summary,
        "lowerable": codes::lowerable(info.code),
        "unnecessary": info.unnecessary,
        "section": section(info.code).unwrap_or_default(),
        "url": codes::docs_url(info.code),
    })
}

/// A code's entry: its title and severity, its summary, the guide's section
/// for it, and where the guide is.
pub(crate) fn explanation(code: &str) -> Option<String> {
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
    codes::guide_section(code)
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
    fn every_code_is_data_too() {
        for c in CODES {
            let e = entry(c.code);
            assert_eq!(e["code"], c.code);
            assert_eq!(e["severity"], c.severity.word());
            assert!(!e["section"].as_str().unwrap().is_empty(), "{}", c.code);
        }
        assert_eq!(entry("R01")["lowerable"], true);
        assert_eq!(entry("T05")["lowerable"], false);
    }

    #[test]
    fn an_entry_shows_its_program_unfenced() {
        let text = explanation("e101").unwrap();
        assert!(text.contains("    page P("), "{text}");
        assert!(!text.contains("```"), "{text}");
        assert!(text.contains("In the guide: https://"), "{text}");
    }
}
