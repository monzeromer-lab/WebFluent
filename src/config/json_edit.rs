//! Editing a JSON file a person also edits by hand — `webfluent.app.json`,
//! a translation file — without reformatting it: a value set or removed
//! touches only its own bytes, so a diff of the change is the change.
//!
//! The text is read once into the places of its keys and values; a set
//! replaces a value's bytes, or adds a member at the end of its object in
//! the object's own style (one line, or one member a line at its
//! indentation); a remove takes the member and the comma that joined it.
//! Key order, indentation, spacing and the final newline stay as written.

use serde_json::Value;

/// What went wrong with an edit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditError(pub String);

impl std::fmt::Display for EditError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for EditError {}

/// A value and where it is.
#[derive(Debug)]
enum Node {
    Object {
        open: usize,
        close: usize,
        members: Vec<Member>,
    },
    Array {
        open: usize,
        close: usize,
    },
    Scalar {
        start: usize,
        end: usize,
    },
}

impl Node {
    fn span(&self) -> (usize, usize) {
        match self {
            Node::Object { open, close, .. } | Node::Array { open, close } => (*open, close + 1),
            Node::Scalar { start, end } => (*start, *end),
        }
    }
}

#[derive(Debug)]
struct Member {
    key: String,
    /// Where its key's opening quote is.
    start: usize,
    value: Node,
}

struct Parser<'a> {
    text: &'a str,
    bytes: &'a [u8],
    at: usize,
}

impl<'a> Parser<'a> {
    fn fail<T>(&self, what: &str) -> Result<T, EditError> {
        let line = self.text[..self.at.min(self.text.len())]
            .matches('\n')
            .count()
            + 1;
        Err(EditError(format!("{what} at line {line}")))
    }

    fn space(&mut self) {
        while self.at < self.bytes.len()
            && matches!(self.bytes[self.at], b' ' | b'\t' | b'\n' | b'\r')
        {
            self.at += 1;
        }
    }

    fn string(&mut self) -> Result<(String, usize, usize), EditError> {
        let start = self.at;
        if self.bytes.get(self.at) != Some(&b'"') {
            return self.fail("a string was expected");
        }
        self.at += 1;
        while self.at < self.bytes.len() && self.bytes[self.at] != b'"' {
            if self.bytes[self.at] == b'\\' {
                self.at += 1;
            }
            self.at += 1;
        }
        if self.at >= self.bytes.len() {
            return self.fail("a string is not closed");
        }
        self.at += 1;
        let raw = &self.text[start..self.at];
        let decoded: String = serde_json::from_str(raw).map_err(|e| EditError(e.to_string()))?;
        Ok((decoded, start, self.at))
    }

    fn value(&mut self) -> Result<Node, EditError> {
        self.space();
        match self.bytes.get(self.at) {
            Some(b'{') => {
                let open = self.at;
                self.at += 1;
                let mut members = Vec::new();
                self.space();
                if self.bytes.get(self.at) == Some(&b'}') {
                    let close = self.at;
                    self.at += 1;
                    return Ok(Node::Object {
                        open,
                        close,
                        members,
                    });
                }
                loop {
                    self.space();
                    let (key, start, _) = self.string()?;
                    self.space();
                    if self.bytes.get(self.at) != Some(&b':') {
                        return self.fail("`:` was expected");
                    }
                    self.at += 1;
                    let value = self.value()?;
                    members.push(Member { key, start, value });
                    self.space();
                    match self.bytes.get(self.at) {
                        Some(b',') => self.at += 1,
                        Some(b'}') => {
                            let close = self.at;
                            self.at += 1;
                            return Ok(Node::Object {
                                open,
                                close,
                                members,
                            });
                        }
                        _ => return self.fail("`,` or `}` was expected"),
                    }
                }
            }
            Some(b'[') => {
                let open = self.at;
                self.at += 1;
                self.space();
                if self.bytes.get(self.at) == Some(&b']') {
                    let close = self.at;
                    self.at += 1;
                    return Ok(Node::Array { open, close });
                }
                loop {
                    self.value()?;
                    self.space();
                    match self.bytes.get(self.at) {
                        Some(b',') => self.at += 1,
                        Some(b']') => {
                            let close = self.at;
                            self.at += 1;
                            return Ok(Node::Array { open, close });
                        }
                        _ => return self.fail("`,` or `]` was expected"),
                    }
                }
            }
            Some(b'"') => {
                let (_, start, end) = self.string()?;
                Ok(Node::Scalar { start, end })
            }
            Some(_) => {
                let start = self.at;
                while self.at < self.bytes.len()
                    && !matches!(
                        self.bytes[self.at],
                        b',' | b'}' | b']' | b' ' | b'\t' | b'\n' | b'\r'
                    )
                {
                    self.at += 1;
                }
                let word = &self.text[start..self.at];
                if serde_json::from_str::<Value>(word).is_err() {
                    return self.fail(&format!("`{word}` is not a JSON value"));
                }
                Ok(Node::Scalar {
                    start,
                    end: self.at,
                })
            }
            None => self.fail("a value was expected"),
        }
    }
}

fn parse(text: &str) -> Result<Node, EditError> {
    let mut p = Parser {
        text,
        bytes: text.as_bytes(),
        at: 0,
    };
    let root = p.value()?;
    p.space();
    if p.at != text.len() {
        return p.fail("text after the value");
    }
    Ok(root)
}

/// The leading whitespace of the line `at` is on.
fn indent_at(text: &str, at: usize) -> &str {
    let start = text[..at].rfind('\n').map_or(0, |n| n + 1);
    let line = &text[start..];
    &line[..line.len() - line.trim_start_matches([' ', '\t']).len()]
}

/// The file's step of indentation: what the first indented line begins
/// with, or two spaces.
fn unit_of(text: &str) -> String {
    text.lines()
        .map(|l| &l[..l.len() - l.trim_start_matches([' ', '\t']).len()])
        .find(|w| !w.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| "  ".to_string())
}

/// How a list or an object is laid out: as the value it replaces was, or —
/// for a new one — a short list of plain values on one line and an object
/// a member a line.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Lay {
    Lines,
    OneLine,
    AsUsual,
}

/// `value` as JSON text written at `indent`, a step being `unit`.
fn render(value: &Value, indent: &str, unit: &str) -> String {
    render_as(value, indent, unit, Lay::AsUsual)
}

fn render_as(value: &Value, indent: &str, unit: &str, lay: Lay) -> String {
    match value {
        Value::Object(map) if !map.is_empty() && lay == Lay::OneLine => {
            let members: Vec<String> = map
                .iter()
                .map(|(k, v)| {
                    format!(
                        "{}: {}",
                        serde_json::to_string(k).unwrap_or_default(),
                        render_as(v, indent, unit, Lay::OneLine)
                    )
                })
                .collect();
            format!("{{ {} }}", members.join(", "))
        }
        Value::Object(map) if !map.is_empty() => {
            let inner = format!("{indent}{unit}");
            let members: Vec<String> = map
                .iter()
                .map(|(k, v)| {
                    format!(
                        "{inner}{}: {}",
                        serde_json::to_string(k).unwrap_or_default(),
                        render(v, &inner, unit)
                    )
                })
                .collect();
            format!("{{\n{}\n{indent}}}", members.join(",\n"))
        }
        Value::Array(items) if !items.is_empty() => {
            let flat = items.iter().all(|i| !i.is_object() && !i.is_array());
            let one_line = format!(
                "[{}]",
                items
                    .iter()
                    .map(|i| serde_json::to_string(i).unwrap_or_default())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            if lay == Lay::OneLine || (lay == Lay::AsUsual && flat && one_line.len() <= 72) {
                one_line
            } else {
                let inner = format!("{indent}{unit}");
                let rows: Vec<String> = items
                    .iter()
                    .map(|i| format!("{inner}{}", render(i, &inner, unit)))
                    .collect();
                format!("[\n{}\n{indent}]", rows.join(",\n"))
            }
        }
        other => serde_json::to_string(other).unwrap_or_default(),
    }
}

/// `value` at `path` of `text`: the value there replaced, or the members it
/// needs added — at the end of their object, in its style.
pub fn set(text: &str, path: &[&str], value: &Value) -> Result<String, EditError> {
    if path.is_empty() {
        return Err(EditError("an edit names a key".into()));
    }
    let root = parse(text)?;
    let unit = unit_of(text);
    let mut node = &root;
    for (i, key) in path.iter().enumerate() {
        let Node::Object {
            open,
            close,
            members,
        } = node
        else {
            return Err(EditError(format!(
                "`{}` is not an object",
                path[..i].join(".")
            )));
        };
        match members.iter().find(|m| m.key == *key) {
            Some(m) if i + 1 == path.len() => {
                let (start, end) = m.value.span();
                let old: Option<Value> = serde_json::from_str(&text[start..end]).ok();
                // The value it already is: nothing to write.
                if old.as_ref() == Some(value) {
                    return Ok(text.to_string());
                }
                // An object over an object, member by member: what stays is
                // left as written, in its order.
                if let (Some(Value::Object(was)), Value::Object(now)) = (&old, value) {
                    let mut out = text.to_string();
                    for (k, v) in now {
                        let mut deeper: Vec<&str> = path.to_vec();
                        deeper.push(k);
                        out = set(&out, &deeper, v)?;
                    }
                    for k in was.keys().filter(|k| !now.contains_key(*k)) {
                        let mut deeper: Vec<&str> = path.to_vec();
                        deeper.push(k);
                        out = remove_keeping(&out, &deeper)?;
                    }
                    return Ok(out);
                }
                let indent = indent_at(text, m.start);
                // As the value it replaces was laid out.
                let lay = match m.value {
                    Node::Scalar { .. } => Lay::AsUsual,
                    _ if text[start..end].contains('\n') => Lay::Lines,
                    _ if end - start > 2 => Lay::OneLine,
                    _ => Lay::AsUsual,
                };
                let mut out = text.to_string();
                out.replace_range(start..end, &render_as(value, indent, &unit, lay));
                return Ok(out);
            }
            Some(m) => node = &m.value,
            None => {
                // What is missing, as one member: the rest of the path
                // nested inside it.
                let nested = path[i + 1..]
                    .iter()
                    .rev()
                    .fold(value.clone(), |inner, k| serde_json::json!({ *k: inner }));
                return Ok(insert(text, *open, *close, members, key, &nested, &unit));
            }
        }
    }
    unreachable!("the loop returns at the path's end")
}

fn insert(
    text: &str,
    open: usize,
    close: usize,
    members: &[Member],
    key: &str,
    value: &Value,
    unit: &str,
) -> String {
    let key_text = serde_json::to_string(key).unwrap_or_default();
    let mut out = text.to_string();
    match members.last() {
        None if !text[open..close].contains('\n') && open > 0 => {
            // `{}` as a person wrote it: filled on its line, its value too.
            let indent = indent_at(text, open).to_string();
            let member = format!(
                "{key_text}: {}",
                render_as(value, &indent, unit, Lay::OneLine)
            );
            out.replace_range(open + 1..close, &format!(" {member} "));
        }
        None => {
            let outer = indent_at(text, open).to_string();
            let inner = format!("{outer}{unit}");
            let member = format!("{key_text}: {}", render(value, &inner, unit));
            out.replace_range(open + 1..close, &format!("\n{inner}{member}\n{outer}"));
        }
        Some(last) => {
            let (_, end) = last.value.span();
            let first = &members[0];
            let multi_line = text[open..first.start].contains('\n');
            if multi_line {
                let indent = indent_at(text, last.start).to_string();
                let member = format!("{key_text}: {}", render(value, &indent, unit));
                out.insert_str(end, &format!(",\n{indent}{member}"));
            } else {
                // An object on one line: the member, and its value, too.
                let indent = indent_at(text, open).to_string();
                let member = format!(
                    "{key_text}: {}",
                    render_as(value, &indent, unit, Lay::OneLine)
                );
                out.insert_str(end, &format!(", {member}"));
            }
        }
    }
    out
}

/// `text` without the member at `path`, and the comma that joined it; an
/// object left empty is `{}`. A path that is not there leaves it as it is.
pub fn remove(text: &str, path: &[&str]) -> Result<String, EditError> {
    let root = parse(text)?;
    let mut node = &root;
    for (i, key) in path.iter().enumerate() {
        let Node::Object {
            open,
            close,
            members,
        } = node
        else {
            return Ok(text.to_string());
        };
        let Some(at) = members.iter().position(|m| m.key == *key) else {
            return Ok(text.to_string());
        };
        if i + 1 < path.len() {
            node = &members[at].value;
            continue;
        }
        let mut out = text.to_string();
        let (_, end) = members[at].value.span();
        if members.len() == 1 {
            // A section that held only this goes too: an empty section
            // means nothing, and an add and a remove then leave the file
            // as it was — unless it was written empty to begin with.
            if i > 0 {
                return remove(text, &path[..i]);
            }
            out.replace_range(open + 1..*close, "");
        } else if at + 1 < members.len() {
            // Up to the next member's key: the comma and the space after it.
            out.replace_range(members[at].start..members[at + 1].start, "");
        } else {
            // The last: from the end of the one before it.
            let (_, before) = members[at - 1].value.span();
            out.replace_range(before..end, "");
        }
        return Ok(out);
    }
    Ok(text.to_string())
}

/// [`remove`], but an object left with nothing stays as `{}`: what an
/// object set to fewer members leaves.
fn remove_keeping(text: &str, path: &[&str]) -> Result<String, EditError> {
    let root = parse(text)?;
    let mut node = &root;
    for key in &path[..path.len() - 1] {
        let Node::Object { members, .. } = node else {
            return Ok(text.to_string());
        };
        let Some(m) = members.iter().find(|m| m.key == *key) else {
            return Ok(text.to_string());
        };
        node = &m.value;
    }
    match node {
        Node::Object {
            open,
            close,
            members,
        } if members.len() == 1 && members[0].key == path[path.len() - 1] => {
            let mut out = text.to_string();
            out.replace_range(open + 1..*close, "");
            Ok(out)
        }
        _ => remove(text, path),
    }
}

/// The value at `path` of `text`, read as JSON.
pub fn get(text: &str, path: &[&str]) -> Result<Option<Value>, EditError> {
    let value: Value = serde_json::from_str(text).map_err(|e| EditError(e.to_string()))?;
    Ok(path.iter().try_fold(&value, |v, k| v.get(*k)).cloned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const CONFIG: &str = "{\n  \"name\": \"halyard\",\n  \"build\": {\n    \"ssg\": true,\n    \"base_path\": \"/Halyard\"\n  },\n  \"meta\": { \"title\": \"Halyard\", \"lang\": \"en\" }\n}\n";

    #[test]
    fn a_value_changes_and_nothing_else_does() {
        let out = set(CONFIG, &["build", "base_path"], &json!("/Docs")).unwrap();
        assert_eq!(out, CONFIG.replace("\"/Halyard\"", "\"/Docs\""));
        let out = set(CONFIG, &["build", "ssg"], &json!(false)).unwrap();
        assert_eq!(out, CONFIG.replace("\"ssg\": true", "\"ssg\": false"));
        let out = set(CONFIG, &["meta", "lang"], &json!("ar")).unwrap();
        assert_eq!(out, CONFIG.replace("\"lang\": \"en\"", "\"lang\": \"ar\""));
    }

    #[test]
    fn a_new_member_goes_at_the_end_of_its_object_in_its_style() {
        // A member a line, at the members' indentation.
        let out = set(CONFIG, &["build", "minify"], &json!(false)).unwrap();
        assert_eq!(
            out,
            CONFIG.replace(
                "\"base_path\": \"/Halyard\"\n",
                "\"base_path\": \"/Halyard\",\n    \"minify\": false\n"
            )
        );
        // On one line, on that line.
        let out = set(CONFIG, &["meta", "site_url"], &json!("https://x.dev")).unwrap();
        assert!(out.contains("\"meta\": { \"title\": \"Halyard\", \"lang\": \"en\", \"site_url\": \"https://x.dev\" }"), "{out}");
        // A section the file does not have yet, nested in full.
        let out = set(CONFIG, &["dev", "port"], &json!(4000)).unwrap();
        assert!(
            out.ends_with("\"meta\": { \"title\": \"Halyard\", \"lang\": \"en\" },\n  \"dev\": {\n    \"port\": 4000\n  }\n}\n"),
            "{out}"
        );
        serde_json::from_str::<Value>(&out).expect("still JSON");
    }

    #[test]
    fn a_list_and_an_object_are_written_as_a_person_would() {
        let out = set(
            CONFIG,
            &["i18n"],
            &json!({ "default_locale": "en", "locales": ["en", "ar"] }),
        )
        .unwrap();
        assert!(
            out.contains("  \"i18n\": {\n    \"default_locale\": \"en\",\n    \"locales\": [\"en\", \"ar\"]\n  }\n}"),
            "{out}"
        );
        let empty = "{}\n";
        assert_eq!(
            set(empty, &["name"], &json!("p")).unwrap(),
            "{\n  \"name\": \"p\"\n}\n"
        );
    }

    #[test]
    fn a_removed_member_takes_its_comma() {
        let out = remove(CONFIG, &["build", "ssg"]).unwrap();
        assert_eq!(out, CONFIG.replace("    \"ssg\": true,\n", ""));
        let out = remove(CONFIG, &["build", "base_path"]).unwrap();
        assert_eq!(
            out,
            CONFIG.replace(",\n    \"base_path\": \"/Halyard\"", "")
        );
        let out = remove(CONFIG, &["meta", "title"]).unwrap();
        assert!(out.contains("\"meta\": { \"lang\": \"en\" }"), "{out}");
        // A section goes with its last member.
        let out = remove(
            &remove(CONFIG, &["build", "ssg"]).unwrap(),
            &["build", "base_path"],
        )
        .unwrap();
        assert!(!out.contains("\"build\""), "{out}");
        let out = remove(
            &remove(CONFIG, &["meta", "title"]).unwrap(),
            &["meta", "lang"],
        )
        .unwrap();
        assert!(!out.contains("\"meta\""), "{out}");
        serde_json::from_str::<Value>(&out).expect("still JSON");
        // A `{}` written by hand is filled on its line; emptied again, it
        // goes, as an empty section means nothing.
        let empty_theme = "{\n  \"name\": \"g\",\n  \"theme\": {}\n}\n";
        let added = set(empty_theme, &["theme", "name"], &json!("Day")).unwrap();
        assert!(
            added.contains("\"theme\": { \"name\": \"Day\" }"),
            "{added}"
        );
        assert_eq!(
            remove(&added, &["theme", "name"]).unwrap(),
            "{\n  \"name\": \"g\"\n}\n"
        );
        let added = set(CONFIG, &["build", "media", "pipeline"], &json!(false)).unwrap();
        assert_eq!(
            remove(&added, &["build", "media", "pipeline"]).unwrap(),
            CONFIG
        );
        assert_eq!(
            remove(CONFIG, &["nothing", "here"]).unwrap(),
            CONFIG,
            "a path not there is no change"
        );
        for out in [
            remove(CONFIG, &["build", "ssg"]).unwrap(),
            remove(CONFIG, &["meta", "lang"]).unwrap(),
        ] {
            serde_json::from_str::<Value>(&out).expect("still JSON");
        }
    }

    #[test]
    fn the_file_s_own_indentation_and_escapes_are_kept() {
        let tabs = "{\n\t\"name\": \"t\",\n\t\"build\": {\n\t\t\"ssg\": true\n\t}\n}";
        let out = set(tabs, &["build", "csp"], &json!(true)).unwrap();
        assert_eq!(
            out,
            "{\n\t\"name\": \"t\",\n\t\"build\": {\n\t\t\"ssg\": true,\n\t\t\"csp\": true\n\t}\n}"
        );
        let escaped = "{ \"a\\\"b\": 1, \"title\": \"say \\\"hi\\\"\" }";
        let out = set(escaped, &["title"], &json!("new \"one\"")).unwrap();
        assert_eq!(out, "{ \"a\\\"b\": 1, \"title\": \"new \\\"one\\\"\" }");
        assert_eq!(get(&out, &["a\"b"]).unwrap(), Some(json!(1)));
    }

    #[test]
    fn an_object_set_over_an_object_keeps_its_order_and_layout() {
        let text = "{\n  \"meta\": {\n    \"scripts\": [\n      {\n        \"src\": \"/a.js\",\n        \"load\": \"after\"\n      }\n    ],\n    \"owner_details\": { \"zeta\": 1, \"alpha\": 2 }\n  }\n}\n";
        // Its own value: unchanged, though a map sorts its keys.
        let same = get(text, &["meta", "owner_details"]).unwrap().unwrap();
        assert_eq!(set(text, &["meta", "owner_details"], &same).unwrap(), text);
        let scripts = get(text, &["meta", "scripts"]).unwrap().unwrap();
        assert_eq!(set(text, &["meta", "scripts"], &scripts).unwrap(), text);
        // One member changed, one added, one gone: in place, in order.
        let out = set(
            text,
            &["meta", "owner_details"],
            &json!({ "zeta": 9, "beta": 3 }),
        )
        .unwrap();
        assert!(
            out.contains("\"owner_details\": { \"zeta\": 9, \"beta\": 3 }"),
            "{out}"
        );
        let out = set(text, &["meta", "owner_details"], &json!({})).unwrap();
        assert!(out.contains("\"owner_details\": {}"), "{out}");
    }

    /// A translation file, as LN-1 edits it: a flat key with dots in it is
    /// one key, text in another script is kept as written, and a nested
    /// group is reached through its keys.
    #[test]
    fn a_translation_file_is_edited_in_place() {
        let ar = "{\n  \"nav.home\": \"الرئيسية\",\n  \"greet\": \"أهلاً، {name}!\",\n  \"cart\": {\n    \"one\": \"منتج واحد\",\n    \"other\": \"{count} منتجات\"\n  }\n}\n";
        let out = set(ar, &["nav.home"], &json!("البداية")).unwrap();
        assert_eq!(out, ar.replace("\"الرئيسية\"", "\"البداية\""));
        let out = set(ar, &["nav.docs"], &json!("الوثائق")).unwrap();
        assert!(out.contains("\"nav.docs\": \"الوثائق\"\n}"), "{out}");
        assert!(
            !out.contains("\\u"),
            "the script is written as itself, not escaped: {out}"
        );
        let out = set(ar, &["cart", "two"], &json!("منتجان")).unwrap();
        assert!(
            out.contains("    \"other\": \"{count} منتجات\",\n    \"two\": \"منتجان\"\n  }"),
            "{out}"
        );
        assert_eq!(
            remove(
                &set(ar, &["cart", "two"], &json!("x")).unwrap(),
                &["cart", "two"]
            )
            .unwrap(),
            ar
        );
        assert_eq!(
            get(ar, &["cart", "other"]).unwrap(),
            Some(json!("{count} منتجات"))
        );
    }

    #[test]
    fn what_is_not_json_is_said_and_not_edited() {
        assert!(set("{ \"a\": 1,, }", &["a"], &json!(2)).is_err());
        assert!(set("{ \"a\": 1 } x", &["a"], &json!(2)).is_err());
        let err = set("{ \"build\": 3 }", &["build", "ssg"], &json!(true)).unwrap_err();
        assert_eq!(err.0, "`build` is not an object");
    }

    /// A round trip of every member of Halyard's own config through `set`
    /// with its own value leaves the file byte-identical.
    #[test]
    fn setting_a_value_to_itself_changes_nothing() {
        let text = "{\n  \"name\": \"p\",\n  \"build\": { \"ssg\": true, \"budget\": { \"app.js\": \"40 kB\" } },\n  \"meta\": {\n    \"title\": \"P\",\n    \"fonts\": [\"https://a\", \"https://b\"]\n  }\n}\n";
        for path in [
            vec!["name"],
            vec!["build", "ssg"],
            vec!["build", "budget", "app.js"],
            vec!["meta", "title"],
            vec!["meta", "fonts"],
        ] {
            let value = get(text, &path).unwrap().unwrap();
            assert_eq!(set(text, &path, &value).unwrap(), text, "{path:?}");
        }
    }
}
