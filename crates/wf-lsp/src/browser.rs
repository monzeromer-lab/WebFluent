//! The browser as the compiler types it: the globals a program may name
//! unprefixed (`localStorage`, `Math`, `setTimeout`) and what each object
//! they hand it has after a dot.
//!
//! The compiler keeps its table (`registry::browser::BROWSER_TYPES`) to
//! itself, so this reads the same text from the compiler's source: one
//! table, and what the editor offers is exactly what the checker accepts —
//! a member it does not list is a `T30` there and absent here.

use std::sync::OnceLock;

/// The compiler's source file that holds the table.
const SOURCE: &str = include_str!("../../../src/registry/browser.rs");

/// A field or a method of a browser object, as written in the table.
#[derive(Debug, Clone)]
pub struct Member {
    pub name: String,
    pub method: bool,
    /// Its line in the table: `getItem(key: String) -> String?`,
    /// `length: Number`.
    pub signature: String,
    /// A field's type, or what a method returns.
    pub ty: String,
}

/// A global: a value (`let localStorage: Storage`) or a function
/// (`fn setTimeout(…) -> Number`).
#[derive(Debug, Clone)]
pub struct Global {
    pub name: String,
    pub function: bool,
    pub signature: String,
    pub ty: String,
}

#[derive(Debug, Default)]
pub struct Table {
    pub globals: Vec<Global>,
    objects: Vec<(String, Option<String>, Vec<Member>)>,
}

/// The table, read once.
pub fn table() -> &'static Table {
    static TABLE: OnceLock<Table> = OnceLock::new();
    TABLE.get_or_init(|| read(table_text()))
}

/// The raw string the compiler's `BROWSER_TYPES` holds.
fn table_text() -> &'static str {
    let marker = "BROWSER_TYPES: &str = r\"";
    let Some(start) = SOURCE.find(marker).map(|i| i + marker.len()) else {
        return "";
    };
    let end = SOURCE[start..]
        .find("\n\";")
        .map_or(SOURCE.len(), |i| start + i);
    &SOURCE[start..end]
}

/// What a method's line returns: the text after its `->`.
fn returns(line: &str) -> String {
    line.rsplit_once("->")
        .map(|(_, r)| r.trim().to_string())
        .unwrap_or_default()
}

fn read(text: &str) -> Table {
    let mut out = Table::default();
    let mut object: Option<(String, Option<String>, Vec<Member>)> = None;
    for line in text.lines().map(str::trim) {
        if line.is_empty() || line.starts_with("//") {
            continue;
        }
        if let Some((_, _, members)) = object.as_mut() {
            if line == "}" {
                out.objects.extend(object.take());
                continue;
            }
            let plain = line.strip_prefix("async ").unwrap_or(line);
            let paren = plain.find('(');
            let colon = plain.find(':');
            if let Some(p) = paren.filter(|p| colon.is_none_or(|c| *p < c)) {
                members.push(Member {
                    name: plain[..p].trim().to_string(),
                    method: true,
                    signature: line.to_string(),
                    ty: returns(plain),
                });
            } else if let Some((name, ty)) = plain.split_once(':') {
                members.push(Member {
                    name: name.trim().to_string(),
                    method: false,
                    signature: line.to_string(),
                    ty: ty.trim().to_string(),
                });
            }
            continue;
        }
        if let Some(rest) = line.strip_prefix("let ") {
            if let Some((name, ty)) = rest.split_once(':') {
                out.globals.push(Global {
                    name: name.trim().to_string(),
                    function: false,
                    signature: rest.to_string(),
                    ty: ty.trim().to_string(),
                });
            }
        } else if let Some(rest) = line.strip_prefix("fn ") {
            if let Some(p) = rest.find('(') {
                out.globals.push(Global {
                    name: rest[..p].trim().to_string(),
                    function: true,
                    signature: rest.to_string(),
                    ty: returns(rest),
                });
            }
        } else if let Some(rest) = line.strip_prefix("object ") {
            let head = rest.trim_end_matches('{').trim();
            let (name, base) = match head.split_once(':') {
                Some((n, b)) => (n.trim().to_string(), Some(b.trim().to_string())),
                None => (head.to_string(), None),
            };
            object = Some((name, base, Vec::new()));
        }
    }
    out
}

impl Table {
    /// The global called `name`.
    pub fn global(&self, name: &str) -> Option<&Global> {
        self.globals.iter().find(|g| g.name == name)
    }

    /// Every member of the object `name`, its base's after its own.
    pub fn members(&self, name: &str) -> Vec<&Member> {
        let mut out: Vec<&Member> = Vec::new();
        let mut next = Some(name.to_string());
        // A base names another object; the chain is short, and bounded.
        for _ in 0..8 {
            let Some(current) = next.take() else {
                break;
            };
            let Some((_, base, members)) = self.objects.iter().find(|(n, _, _)| *n == current)
            else {
                break;
            };
            for m in members {
                if !out.iter().any(|o| o.name == m.name) {
                    out.push(m);
                }
            }
            next.clone_from(base);
        }
        out
    }

    /// The object a type names, when the table declares one: `Storage` for
    /// `Storage`, `Element?` and `Element` alike; not a list.
    pub fn object_of(&self, ty: &str) -> Option<&str> {
        let name = ty.trim().trim_end_matches('?');
        self.objects
            .iter()
            .find(|(n, _, _)| n == name)
            .map(|(n, _, _)| n.as_str())
    }

    /// The object a chain of names reads — `localStorage` is a `Storage`,
    /// `document.body` an `Element` — when every link is a field the table
    /// types.
    pub fn object_of_chain(&self, chain: &[&str]) -> Option<&str> {
        let (first, rest) = chain.split_first()?;
        let global = self.global(first).filter(|g| !g.function)?;
        let mut object = self.object_of(&global.ty)?;
        for link in rest {
            let member = self
                .members(object)
                .into_iter()
                .find(|m| m.name == *link && !m.method)?;
            object = self.object_of(&member.ty)?;
        }
        Some(object)
    }
}
