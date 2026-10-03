//! `// wf-allow(U01)`: one finding, at one place, accepted on purpose.
//!
//! A comment of its own covers the next line that is code; a comment at the
//! end of a line covers that line. It names codes (`U01`) or families
//! (`U`), comma-separated. Only what `lints` could lower may be allowed — a
//! warning, or an error that ships no broken page — and an allow that
//! matches nothing is itself a finding (`U07`), so allows cannot outlive
//! what they were written for.

use super::{Diagnostic, codes};

/// One `wf-allow(…)` comment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Allow {
    /// 1-based line and column of the comment.
    pub line: usize,
    pub column: usize,
    /// 1-based column one past its closing `)`.
    pub end_column: usize,
    /// The line it covers.
    pub covers: usize,
    /// What it names, as written.
    pub codes: Vec<String>,
}

/// The `// wf-allow(…)` comments of a file.
pub fn allows(source: &str) -> Vec<Allow> {
    let lines: Vec<&str> = source.lines().collect();
    let mut out = Vec::new();
    for (i, text) in lines.iter().enumerate() {
        let Some(at) = comment_at(text) else {
            continue;
        };
        let comment = text[at + 2..].trim_start();
        let Some(rest) = comment.strip_prefix("wf-allow(") else {
            continue;
        };
        let rest_at = text.len() - rest.len();
        let Some(close) = rest.find(')') else {
            continue;
        };
        let codes: Vec<String> = rest[..close]
            .split(',')
            .map(|c| c.trim().to_string())
            .filter(|c| !c.is_empty())
            .collect();
        let own_line = text[..at].trim().is_empty();
        let covers = if own_line {
            // The next line that is code: blank lines and other comments —
            // a second allow, a note — sit between.
            (i + 1..lines.len())
                .find(|&j| {
                    let t = lines[j].trim();
                    !t.is_empty() && !t.starts_with("//")
                })
                .map_or(0, |j| j + 1)
        } else {
            i + 1
        };
        out.push(Allow {
            line: i + 1,
            column: text[..at].chars().count() + 1,
            end_column: text[..rest_at + close + 1].chars().count() + 1,
            covers,
            codes,
        });
    }
    out
}

/// The byte offset of a `//` comment on the line, outside any string.
fn comment_at(text: &str) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut in_string = false;
    let mut i = 0;
    while i + 1 < bytes.len() {
        match bytes[i] {
            b'\\' if in_string => i += 1,
            b'"' => in_string = !in_string,
            b'/' if !in_string && bytes[i + 1] == b'/' => return Some(i),
            _ => {}
        }
        i += 1;
    }
    None
}

fn names(written: &str, code: &str) -> bool {
    written == code || (written.len() == 1 && codes::family(code) == written)
}

/// Drop every finding an allow covers, and add a `U07` for each allow — or
/// each code in one — that covered nothing or that no allow may silence.
/// `source_of` reads a file by the name a finding gives it.
pub fn apply(
    out: &mut Vec<Diagnostic>,
    files: &[String],
    source_of: &dyn Fn(&str) -> Option<String>,
) {
    let mut findings = Vec::new();
    for file in files {
        let Some(source) = source_of(file) else {
            continue;
        };
        if !source.contains("wf-allow(") {
            continue;
        }
        for allow in allows(&source) {
            let mut used = vec![false; allow.codes.len()];
            out.retain(|d| {
                if d.file != *file || d.line != allow.covers {
                    return true;
                }
                match allow
                    .codes
                    .iter()
                    .position(|c| names(c, d.code) && codes::lowerable(d.code))
                {
                    Some(k) => {
                        used[k] = true;
                        false
                    }
                    None => true,
                }
            });
            for (k, code) in allow.codes.iter().enumerate() {
                if used[k] {
                    continue;
                }
                let known = codes::info(code).is_some()
                    || (code.len() == 1
                        && codes::CODES
                            .iter()
                            .any(|c| c.code.starts_with(code.as_str())));
                let (message, hint) = if !known {
                    (
                        format!("`wf-allow({code})` names no code or family"),
                        "`wf explain` lists what a code means; a family is its letter".to_string(),
                    )
                } else if codes::info(code).is_some() && !codes::lowerable(code) {
                    (
                        format!("`wf-allow({code})` cannot silence `{code}`, an error that ships a broken page"),
                        "Fix what it reports; an allow is for a warning, or an error that breaks nothing".to_string(),
                    )
                } else {
                    (
                        format!(
                            "`wf-allow({code})` allows nothing: no `{code}` is reported on the line it covers"
                        ),
                        "Remove it — what it was written for is gone".to_string(),
                    )
                };
                findings.push(
                    Diagnostic::coded("U07", message, file.clone(), allow.line, allow.column)
                        .with_end(allow.line, allow.end_column)
                        .with_hint(hint),
                );
            }
        }
    }
    out.extend(findings);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_allow_covers_the_next_line_of_code_or_its_own() {
        let src = "page P {\n    // wf-allow(U01)\n\n    state a = 1\n    state b = 2 // wf-allow(U01, A)\n    Text(\"// wf-allow(U01)\")\n}\n";
        let found = allows(src);
        assert_eq!(found.len(), 2, "{found:?}");
        assert_eq!((found[0].line, found[0].covers), (2, 4));
        assert_eq!((found[1].line, found[1].covers), (5, 5));
        assert_eq!(found[1].codes, vec!["U01", "A"]);
    }

    #[test]
    fn what_it_covers_goes_and_what_it_does_not_is_reported() {
        let src = "// wf-allow(U01)\nstate a = 1\n// wf-allow(U02, T05, X99)\nstate b = 2\n";
        let mut out = vec![
            Diagnostic::coded("U01", "`a` is never read", "a.wf", 2, 1),
            Diagnostic::coded("T05", "no field", "a.wf", 4, 1),
        ];
        apply(&mut out, &["a.wf".to_string()], &|_| Some(src.to_string()));
        let codes: Vec<(&str, usize)> = out.iter().map(|d| (d.code, d.line)).collect();
        // U01 allowed; T05 stays (an error no allow silences); U02 matched
        // nothing, T05 cannot be allowed, X99 is no code: three U07s.
        assert_eq!(
            codes,
            vec![("T05", 4), ("U07", 3), ("U07", 3), ("U07", 3)],
            "{out:#?}"
        );
    }
}
