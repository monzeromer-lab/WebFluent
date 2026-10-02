//! What a `/** … */` comment says about the function under it.
//!
//! JSDoc is a comment, so a script that carries it is still exactly what
//! the browser runs. The compiler reads three things from it: the prose
//! (the editor's hover), each `@param {Type} name`, and `@returns {Type}`.
//! The type expressions are kept as written; [`crate::sema`] decides what
//! each means, and anything it does not know is `Any`.

/// A doc comment, read.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Doc {
    /// The prose before the first tag, with the comment's `*` margin off.
    pub summary: String,
    pub params: Vec<DocParam>,
    /// `@returns {T}` (or `@return`): the type as written.
    pub returns: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DocParam {
    pub name: String,
    /// The type as written, between the braces.
    pub ty: Option<String>,
    /// `[name]` or `[name=default]`: a call may leave it out.
    pub optional: bool,
    pub description: String,
}

impl Doc {
    pub fn param(&self, name: &str) -> Option<&DocParam> {
        self.params.iter().find(|p| p.name == name)
    }
}

/// Read a doc comment, `/**` and `*/` included or not.
pub fn parse(comment: &str) -> Doc {
    let body = comment
        .trim()
        .trim_start_matches("/**")
        .trim_end_matches("*/");
    // The comment's left margin: a leading `*` and one space after it.
    let lines: Vec<&str> = body
        .lines()
        .map(|l| {
            let l = l.trim_start();
            let l = l.strip_prefix('*').unwrap_or(l);
            l.strip_prefix(' ').unwrap_or(l).trim_end()
        })
        .collect();
    let text = lines.join("\n");

    let mut doc = Doc::default();
    // Split at each tag that starts a line (or the comment).
    let mut sections: Vec<String> = Vec::new();
    let mut current = String::new();
    for line in text.lines() {
        if line.trim_start().starts_with('@') && !current.is_empty() {
            sections.push(std::mem::take(&mut current));
        }
        if !current.is_empty() {
            current.push('\n');
        }
        current.push_str(line);
    }
    if !current.is_empty() {
        sections.push(current);
    }
    for (i, section) in sections.iter().enumerate() {
        let section = section.trim();
        if !section.starts_with('@') {
            if i == 0 {
                doc.summary = section.to_string();
            }
            continue;
        }
        let (tag, rest) = section[1..]
            .split_once(char::is_whitespace)
            .unwrap_or((&section[1..], ""));
        let rest = rest.trim_start();
        match tag {
            "param" | "arg" | "argument" => {
                let (ty, rest) = braced(rest);
                let rest = rest.trim_start();
                let (raw_name, description) = if let Some(after) = rest.strip_prefix('[') {
                    // `[name=default]`, whose default may hold brackets.
                    let mut depth = 1;
                    let end = after
                        .char_indices()
                        .find(|&(_, c)| {
                            match c {
                                '[' => depth += 1,
                                ']' => depth -= 1,
                                _ => {}
                            }
                            depth == 0
                        })
                        .map(|(i, _)| i)
                        .unwrap_or(after.len());
                    (
                        format!("[{}]", &after[..end]),
                        after.get(end + 1..).unwrap_or("").trim(),
                    )
                } else {
                    let (n, d) = rest.split_once(char::is_whitespace).unwrap_or((rest, ""));
                    (n.to_string(), d.trim())
                };
                let optional = raw_name.starts_with('[');
                let name = raw_name
                    .trim_start_matches('[')
                    .trim_end_matches(']')
                    .split('=')
                    .next()
                    .unwrap_or("")
                    .trim()
                    .to_string();
                // `opts.max` documents a field of `opts`, not a parameter.
                if name.is_empty() || name.contains('.') {
                    continue;
                }
                // `{number=}` is Closure's spelling of an optional one.
                let optional = optional || ty.as_deref().is_some_and(|t| t.ends_with('='));
                let ty = ty.map(|t| t.trim_end_matches('=').trim().to_string());
                doc.params.push(DocParam {
                    name,
                    ty,
                    optional,
                    description: description.trim_start_matches("- ").trim().to_string(),
                });
            }
            "returns" | "return" => {
                doc.returns = braced(rest).0;
            }
            _ => {}
        }
    }
    doc
}

/// `{Type} rest` → (`Some("Type")`, `rest`), the braces balanced so an
/// object type `{{ a: number }}` reads whole.
fn braced(text: &str) -> (Option<String>, &str) {
    let Some(inner) = text.strip_prefix('{') else {
        return (None, text);
    };
    let mut depth = 1;
    for (i, c) in inner.char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return (Some(inner[..i].trim().to_string()), &inner[i + 1..]);
                }
            }
            _ => {}
        }
    }
    (None, text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_prose_the_parameters_and_what_it_returns() {
        let doc = parse(
            "/**\n * Tilt an element toward the pointer.\n * Smoothly.\n *\n * @param {HTMLElement} node - the element\n * @param {{ max?: number, glare?: boolean }} [opts={}] how far\n * @param {...string} names\n * @returns {{ destroy(): void }}\n */",
        );
        assert_eq!(
            doc.summary,
            "Tilt an element toward the pointer.\nSmoothly."
        );
        let got: Vec<(&str, Option<&str>, bool, &str)> = doc
            .params
            .iter()
            .map(|p| {
                (
                    p.name.as_str(),
                    p.ty.as_deref(),
                    p.optional,
                    p.description.as_str(),
                )
            })
            .collect();
        assert_eq!(
            got,
            [
                ("node", Some("HTMLElement"), false, "the element"),
                (
                    "opts",
                    Some("{ max?: number, glare?: boolean }"),
                    true,
                    "how far"
                ),
                ("names", Some("...string"), false, ""),
            ]
        );
        assert_eq!(doc.returns.as_deref(), Some("{ destroy(): void }"));
    }

    #[test]
    fn a_one_line_comment_and_the_other_spellings() {
        let doc = parse("/** @param {number=} n @return {string} */");
        // One line: the first tag swallows the rest, which is fine for a
        // name and a type.
        assert_eq!(doc.params[0].name, "n");
        assert!(doc.params[0].optional);
        let doc = parse(
            "/**\n * @arg {boolean} on\n * @param {Object} opts\n * @param {number} opts.max\n * @return {Promise<number>}\n */",
        );
        assert_eq!(
            doc.params
                .iter()
                .map(|p| p.name.as_str())
                .collect::<Vec<_>>(),
            ["on", "opts"]
        );
        assert_eq!(doc.returns.as_deref(), Some("Promise<number>"));
    }

    #[test]
    fn a_default_with_brackets_in_it() {
        let doc = parse("/** \n * @param {number[]} [xs=[1, 2]] the list\n */");
        assert_eq!(doc.params[0].name, "xs");
        assert!(doc.params[0].optional);
        assert_eq!(doc.params[0].description, "the list");
    }
}
