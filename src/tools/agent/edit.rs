//! An agent's anchored edit: replace the text it names with new text,
//! where the text it names appears exactly once. A model copies text back
//! imperfectly — a space at the end of a line, another indent, a curly
//! quote for a straight one — so each `old_text` is looked for in four
//! ways, from strict to forgiving, and the first way that finds it once
//! wins:
//!
//! 1. exactly;
//! 2. line by line, ignoring what ends each line;
//! 3. line by line, ignoring the indent too, with the new text moved to
//!    the indent it replaces;
//! 4. as 3, with typographic quotes, dashes and spaces read as their ASCII.
//!
//! Text found more than once is refused with the lines it was found on,
//! and text found nowhere with the lines nearest to it. The edits apply in
//! order, each to the text the one before it left, and all of them or
//! none.

use std::fmt;

use serde::{Deserialize, Serialize};

/// One replacement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Edit {
    pub old_text: String,
    pub new_text: String,
}

/// Why an edit could not be made.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Miss {
    /// `old_text` is empty: there is nothing to anchor to.
    Empty,
    /// `old_text` and `new_text` are the same.
    Unchanged,
    /// Found nowhere. The lines nearest to it, with their numbers (from 1).
    NotFound { nearest: Vec<(usize, String)> },
    /// Found more than once, starting on these lines (from 1).
    Ambiguous { lines: Vec<usize> },
}

/// The edit that could not be made: its place among the edits (from 0),
/// and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditError {
    pub index: usize,
    pub of: usize,
    pub miss: Miss,
}

impl fmt::Display for EditError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.of > 1 {
            write!(f, "Edit {} of {}: ", self.index + 1, self.of)?;
        }
        match &self.miss {
            Miss::Empty => write!(
                f,
                "`old_text` is empty. Name the text to replace; to make a new file, write it whole."
            ),
            Miss::Unchanged => write!(
                f,
                "`old_text` and `new_text` are the same, so it changes nothing."
            ),
            Miss::NotFound { nearest } if nearest.is_empty() => {
                write!(
                    f,
                    "`old_text` matches nothing in the file. Read the file again and copy the text exactly."
                )
            }
            Miss::NotFound { nearest } => {
                write!(
                    f,
                    "`old_text` matches nothing in the file. The nearest lines are "
                )?;
                let lines: Vec<String> = nearest
                    .iter()
                    .map(|(n, t)| format!("{n}: `{}`", t.trim()))
                    .collect();
                write!(
                    f,
                    "{}. Read the file again and copy the text exactly.",
                    lines.join(", ")
                )
            }
            Miss::Ambiguous { lines } => {
                let names: Vec<String> = lines.iter().map(|n| n.to_string()).collect();
                let listed = match names.len() {
                    0 | 1 => names.join(""),
                    _ => format!(
                        "{} and {}",
                        names[..names.len() - 1].join(", "),
                        names[names.len() - 1]
                    ),
                };
                write!(
                    f,
                    "`old_text` matches {} places, on lines {listed}. Add a line around it so it matches one.",
                    lines.len()
                )
            }
        }
    }
}

impl std::error::Error for EditError {}

/// `text` with every edit made, in order; or the first that could not be,
/// and `text` is left as it was.
pub fn apply(text: &str, edits: &[Edit]) -> Result<String, EditError> {
    let mut out = text.to_owned();
    for (index, e) in edits.iter().enumerate() {
        let fail = |miss| EditError {
            index,
            of: edits.len(),
            miss,
        };
        if e.old_text.is_empty() {
            return Err(fail(Miss::Empty));
        }
        if e.old_text == e.new_text {
            return Err(fail(Miss::Unchanged));
        }
        out = replace(&out, e).map_err(fail)?;
    }
    Ok(out)
}

/// The text with one edit made.
fn replace(text: &str, e: &Edit) -> Result<String, Miss> {
    // 1. Exactly.
    let found: Vec<usize> = text
        .match_indices(e.old_text.as_str())
        .map(|(i, _)| i)
        .collect();
    match found.len() {
        1 => {
            let at = found[0];
            return Ok(format!(
                "{}{}{}",
                &text[..at],
                e.new_text,
                &text[at + e.old_text.len()..]
            ));
        }
        n if n > 1 => {
            return Err(Miss::Ambiguous {
                lines: found.iter().map(|&i| line_of(text, i)).collect(),
            });
        }
        _ => {}
    }
    // 2–4. Line by line, each way more forgiving than the last.
    let lines: Vec<&str> = text.split_inclusive('\n').collect();
    let wanted: Vec<&str> = e.old_text.lines().collect();
    let ways: [fn(&str) -> String; 3] = [
        |l| l.trim_end().to_owned(),
        |l| l.trim().to_owned(),
        |l| ascii(l.trim()),
    ];
    for (way, same) in ways.iter().enumerate() {
        let want: Vec<String> = wanted.iter().map(|l| same(l)).collect();
        // A single line that is part of a line is looked for in the line.
        if want.len() == 1 && way > 0 {
            let needle = &want[0];
            if needle.is_empty() {
                continue;
            }
            let hits: Vec<usize> = (0..lines.len())
                .filter(|&i| same(lines[i]).contains(needle.as_str()))
                .collect();
            match hits.len() {
                1 => {
                    return Ok(within_line(
                        &lines,
                        hits[0],
                        wanted[0].trim(),
                        &e.new_text,
                        way == 2,
                    ));
                }
                n if n > 1 => {
                    return Err(Miss::Ambiguous {
                        lines: hits.iter().map(|i| i + 1).collect(),
                    });
                }
                _ => continue,
            }
        }
        if want.len() > lines.len() {
            continue;
        }
        let starts: Vec<usize> = (0..=lines.len() - want.len())
            .filter(|&i| {
                want.iter()
                    .enumerate()
                    .all(|(k, w)| same(lines[i + k]) == *w)
            })
            .collect();
        match starts.len() {
            1 => {
                let at = starts[0];
                let replaced = &lines[at..at + want.len()];
                let new_text = if way == 0 {
                    e.new_text.clone()
                } else {
                    reindent(&e.new_text, &e.old_text, replaced)
                };
                let ends_line = replaced.last().is_some_and(|l| l.ends_with('\n'));
                let mut out: String = lines[..at].concat();
                out.push_str(&new_text);
                if ends_line && !new_text.ends_with('\n') {
                    out.push('\n');
                }
                out.push_str(&lines[at + want.len()..].concat());
                return Ok(out);
            }
            n if n > 1 => {
                return Err(Miss::Ambiguous {
                    lines: starts.iter().map(|i| i + 1).collect(),
                });
            }
            _ => {}
        }
    }
    Err(Miss::NotFound {
        nearest: nearest(&lines, &e.old_text),
    })
}

/// The line (from 1) byte `at` of `text` is on.
fn line_of(text: &str, at: usize) -> usize {
    text[..at].matches('\n').count() + 1
}

/// Line `i` with the part of it that reads as `old` replaced by `new`.
fn within_line(lines: &[&str], i: usize, old: &str, new: &str, typographic: bool) -> String {
    let line = lines[i];
    let replaced = match line.find(old) {
        Some(at) => format!("{}{}{}", &line[..at], new.trim(), &line[at + old.len()..]),
        None => {
            // Found only once its quotes and dashes were read as ASCII: the
            // line is matched as ASCII, and its own characters kept elsewhere.
            let plain = if typographic {
                ascii(line)
            } else {
                line.to_owned()
            };
            let old = if typographic {
                ascii(old)
            } else {
                old.to_owned()
            };
            match plain.find(&old) {
                Some(at) if plain.len() == line.len() => {
                    format!("{}{}{}", &line[..at], new.trim(), &line[at + old.len()..])
                }
                Some(at) => format!("{}{}{}", &plain[..at], new.trim(), &plain[at + old.len()..]),
                None => line.to_owned(),
            }
        }
    };
    let mut out: String = lines[..i].concat();
    out.push_str(&replaced);
    out.push_str(&lines[i + 1..].concat());
    out
}

/// `new` moved from the indent `old` was written at to the indent of the
/// lines it replaces.
fn reindent(new: &str, old: &str, replaced: &[&str]) -> String {
    let indent = |l: &str| l.len() - l.trim_start().len();
    fn first<'a>(mut ls: impl Iterator<Item = &'a str>) -> Option<(usize, String)> {
        let l = ls.find(|l| !l.trim().is_empty())?;
        let n = l.len() - l.trim_start().len();
        Some((n, l[..n].to_owned()))
    }
    let (Some((written, _)), Some((there, prefix))) =
        (first(old.lines()), first(replaced.iter().copied()))
    else {
        return new.to_owned();
    };
    let mut out = String::new();
    for line in new.split_inclusive('\n') {
        if line.trim().is_empty() {
            out.push_str(line);
            continue;
        }
        let own = indent(line);
        let body = &line[own..];
        let depth = (own + there).saturating_sub(written);
        let unit = if prefix.starts_with('\t') { "\t" } else { " " };
        let lead = if depth >= there {
            format!("{prefix}{}", unit.repeat(depth - there))
        } else {
            unit.repeat(depth)
        };
        out.push_str(&lead);
        out.push_str(body);
    }
    out
}

/// Typographic punctuation and spaces as their ASCII.
fn ascii(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '\u{2018}' | '\u{2019}' | '\u{201A}' | '\u{2032}' => '\'',
            '\u{201C}' | '\u{201D}' | '\u{201E}' | '\u{2033}' => '"',
            '\u{2010}' | '\u{2011}' | '\u{2012}' | '\u{2013}' | '\u{2014}' | '\u{2212}' => '-',
            '\u{00A0}' | '\u{2007}' | '\u{202F}' | '\u{2009}' => ' ',
            other => other,
        })
        .collect()
}

/// The three lines most like the first line of `old`, with their numbers.
fn nearest(lines: &[&str], old: &str) -> Vec<(usize, String)> {
    let Some(probe) = old.lines().map(|l| ascii(l.trim())).find(|l| !l.is_empty()) else {
        return Vec::new();
    };
    let mut scored: Vec<(f64, usize)> = lines
        .iter()
        .enumerate()
        .filter(|(_, l)| !l.trim().is_empty())
        .map(|(i, l)| (likeness(&probe, &ascii(l.trim())), i))
        .filter(|(s, _)| *s > 0.3)
        .collect();
    scored.sort_by(|a, b| {
        b.0.partial_cmp(&a.0)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.1.cmp(&b.1))
    });
    scored
        .into_iter()
        .take(3)
        .map(|(_, i)| (i + 1, lines[i].trim_end().to_owned()))
        .collect()
}

/// How alike two lines are, from 0 to 1: one less the share of characters
/// an edit distance changes.
fn likeness(a: &str, b: &str) -> f64 {
    let (a, b): (Vec<char>, Vec<char>) = (a.chars().collect(), b.chars().collect());
    let longest = a.len().max(b.len());
    if longest == 0 {
        return 1.0;
    }
    let mut row: Vec<usize> = (0..=b.len()).collect();
    for i in 1..=a.len() {
        let mut last = row[0];
        row[0] = i;
        for j in 1..=b.len() {
            let here = row[j];
            row[j] = if a[i - 1] == b[j - 1] {
                last
            } else {
                1 + last.min(row[j]).min(row[j - 1])
            };
            last = here;
        }
    }
    1.0 - row[b.len()] as f64 / longest as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn edit(old: &str, new: &str) -> Edit {
        Edit {
            old_text: old.to_owned(),
            new_text: new.to_owned(),
        }
    }

    const PAGE: &str = "page Deploys(path: \"/deploys\") {\n    use DeployStore\n    Heading(\"Deploys\").h1\n    Button(\"Close\") {\n        on click { DeployStore.closeMenus() }\n    }\n}\n";

    #[test]
    fn exact_text_found_once_is_replaced() {
        let out = apply(
            PAGE,
            &[edit("DeployStore.closeMenus()", "DeployStore.closeMenu()")],
        )
        .unwrap();
        assert!(out.contains("on click { DeployStore.closeMenu() }"));
        assert_eq!(out.len(), PAGE.len() - 1);
    }

    #[test]
    fn what_ends_a_line_is_not_held_against_it() {
        let old = "    Heading(\"Deploys\").h1   \n    Button(\"Close\") {  ";
        let out = apply(
            PAGE,
            &[edit(
                old,
                "    Heading(\"Every deploy\").h1\n    Button(\"Close\") {",
            )],
        )
        .unwrap();
        assert!(
            out.contains(
                "    Heading(\"Every deploy\").h1\n    Button(\"Close\") {\n        on click"
            ),
            "{out}"
        );
    }

    #[test]
    fn another_indent_is_found_and_the_new_text_moved_to_the_file_s() {
        // Copied without its indent, and the new line written at the same.
        let old = "Button(\"Close\") {\n    on click { DeployStore.closeMenus() }\n}";
        let new =
            "Button(\"Close\") {\n    on click { DeployStore.closeMenu() }\n}\nText(\"Closed\")";
        let out = apply(PAGE, &[edit(old, new)]).unwrap();
        assert!(out.contains("    Button(\"Close\") {\n        on click { DeployStore.closeMenu() }\n    }\n    Text(\"Closed\")\n}\n"), "{out}");
    }

    #[test]
    fn curly_quotes_and_dashes_read_as_their_ascii() {
        let out = apply(
            PAGE,
            &[edit(
                "Heading(“Deploys”).h1",
                "Heading(\"Deploys — all\").h1",
            )],
        )
        .unwrap();
        assert!(out.contains("Heading(\"Deploys — all\").h1"), "{out}");
    }

    #[test]
    fn text_found_twice_names_both_lines() {
        let text = "Text(\"a\")\nText(\"b\")\nText(\"a\")\n";
        let e = apply(text, &[edit("Text(\"a\")", "Text(\"c\")")]).unwrap_err();
        assert_eq!(e.miss, Miss::Ambiguous { lines: vec![1, 3] });
        assert_eq!(
            e.to_string(),
            "`old_text` matches 2 places, on lines 1 and 3. Add a line around it so it matches one."
        );
    }

    #[test]
    fn text_found_nowhere_names_the_nearest_lines() {
        let e = apply(PAGE, &[edit("DeployStore.closeMenuz()", "x")]).unwrap_err();
        let Miss::NotFound { nearest } = &e.miss else {
            panic!("{e:?}")
        };
        assert_eq!(nearest[0].0, 5);
        assert!(
            e.to_string()
                .contains("5: `on click { DeployStore.closeMenus() }`"),
            "{e}"
        );
        let e = apply(PAGE, &[edit("nothing like any of it at all, nowhere", "x")]).unwrap_err();
        assert_eq!(e.miss, Miss::NotFound { nearest: vec![] });
    }

    #[test]
    fn edits_apply_in_order_and_all_or_none() {
        let out = apply(
            PAGE,
            &[
                edit("\"Deploys\").h1", "\"Every deploy\").h1"),
                edit("Every deploy", "All deploys"),
            ],
        )
        .unwrap();
        assert!(out.contains("Heading(\"All deploys\").h1"));
        let e = apply(
            PAGE,
            &[
                edit("\"Deploys\").h1", "\"X\").h1"),
                edit("no such thing here", "y"),
            ],
        )
        .unwrap_err();
        assert_eq!((e.index, e.of), (1, 2));
        assert!(e.to_string().starts_with("Edit 2 of 2: "));
    }

    #[test]
    fn an_empty_or_unchanged_edit_is_refused() {
        assert_eq!(apply(PAGE, &[edit("", "x")]).unwrap_err().miss, Miss::Empty);
        assert_eq!(
            apply(PAGE, &[edit("use DeployStore", "use DeployStore")])
                .unwrap_err()
                .miss,
            Miss::Unchanged
        );
    }

    #[test]
    fn a_whole_line_removed_leaves_no_blank_line() {
        let out = apply(PAGE, &[edit("    use DeployStore\n", "")]).unwrap();
        assert!(
            out.starts_with("page Deploys(path: \"/deploys\") {\n    Heading"),
            "{out}"
        );
    }
}
