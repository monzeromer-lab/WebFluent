//! The translations as a table, both ways: every locale's
//! `<dir>/<locale>.json` read into keys — a key with plural forms read as
//! one key with its forms — and the messages the default locale's carry
//! held against each locale's, with the forms each locale's plural rules
//! need, the placeholders each message has, what is missing, and which
//! keys a `t()` call names; and one message written back into its file
//! with every other byte as it was.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::Path;

use serde::Serialize;

use crate::syntax::parser::ast::{
    Arg, Declaration, Expr, Program, Span, StatementKind, StringPart,
};

/// The forms a message may have, in CLDR's order.
pub use super::plurals::CATEGORIES as FORMS;

/// The languages written right to left. A locale is one when its language
/// is: `ar-EG` as `ar`.
pub const RTL: &[&str] = &[
    "ar", "arc", "ckb", "dv", "fa", "he", "iw", "ks", "nqo", "ps", "sd", "syr", "ug", "ur", "yi",
];

/// Whether `locale` is written right to left.
pub fn is_rtl(locale: &str) -> bool {
    let language = locale
        .split(['-', '_'])
        .next()
        .unwrap_or(locale)
        .to_ascii_lowercase();
    RTL.contains(&language.as_str())
}

/// The `{name}` placeholders a message has, in order, each once.
pub fn placeholders(message: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut rest = message;
    while let Some(open) = rest.find('{') {
        let after = &rest[open + 1..];
        let Some(close) = after.find('}') else {
            break;
        };
        let name = &after[..close];
        if !name.is_empty()
            && name.chars().all(|c| c.is_alphanumeric() || c == '_')
            && !out.iter().any(|o| o == name)
        {
            out.push(name.to_string());
        }
        rest = &after[close + 1..];
    }
    out
}

/// The form a table key is of its base: `items.one` is `items`'s `one`.
pub fn split_form(key: &str) -> (&str, Option<&'static str>) {
    for form in FORMS {
        if let Some(base) = key.strip_suffix(form).and_then(|b| b.strip_suffix('.'))
            && !base.is_empty()
        {
            return (base, Some(form));
        }
    }
    (key, None)
}

/// One place a `t()` call is written.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Use {
    /// The key it names; for a key built at run time
    /// (`t("status.{s}")`), the part before the first splice.
    pub key: String,
    /// Whether the key is built at run time, so `key` is only its start.
    pub dynamic: bool,
    /// The names it passes: `["count", "name"]`. None when what it passes
    /// is not written as a map.
    pub passes: Option<Vec<String>>,
    pub file: String,
    pub line: u32,
    pub col: u32,
}

fn key_of(e: &Expr) -> Option<(String, bool)> {
    match e {
        Expr::StringLiteral(k) => Some((k.clone(), false)),
        Expr::InterpolatedString(parts) => {
            let prefix: String = parts
                .iter()
                .map_while(|p| match p {
                    StringPart::Literal(s) => Some(s.as_str()),
                    StringPart::Expression(_) => None,
                })
                .collect();
            Some((prefix, true))
        }
        _ => None,
    }
}

fn walk_expr(e: &Expr, at: Span, file: &str, out: &mut Vec<Use>) {
    if let Expr::FunctionCall(name, args) = e
        && name == "t"
        && let Some((key, dynamic)) = args.first().and_then(key_of)
    {
        let passes = match args.get(1) {
            None => Some(Vec::new()),
            Some(Expr::MapLiteral(pairs)) => Some(
                pairs
                    .iter()
                    .map(|(k, _)| k.trim_matches('"').to_string())
                    .filter(|k| k != "...")
                    .collect(),
            ),
            Some(_) => None,
        };
        out.push(Use {
            key,
            dynamic,
            passes,
            file: file.to_string(),
            line: at.line.max(1),
            col: at.col.max(1),
        });
    }
    for child in e.children() {
        walk_expr(child, at, file, out);
    }
}

/// Every expression a program writes where a `t()` may be: in bodies, in a
/// page's title, description, guard, paths, head and layout, in a prop's
/// default, and in a `const` — each with the file it is in.
fn for_each_expr(
    program: &Program,
    file_of: &dyn Fn(usize) -> String,
    f: &mut dyn FnMut(&Expr, Span, &str),
) {
    for (i, d) in program.declarations.iter().enumerate() {
        let file = file_of(i);
        let mut header = |e: &Expr, at: Span| f(e, at, &file);
        match d {
            Declaration::Page(p) => {
                for e in [&p.title_expr, &p.description_expr, &p.guard, &p.paths]
                    .into_iter()
                    .flatten()
                {
                    header(e, p.header_span);
                }
                if let Some(l) = &p.layout {
                    for a in &l.args {
                        let (Arg::Positional(e) | Arg::Named(_, e)) = a;
                        header(e, l.span);
                    }
                }
                for h in &p.head {
                    for (_, e) in &h.attrs {
                        header(e, h.span);
                    }
                }
                for prop in &p.params {
                    if let Some(e) = &prop.default {
                        header(e, p.header_span);
                    }
                }
            }
            Declaration::Component(c) => {
                for prop in &c.props {
                    if let Some(e) = &prop.default {
                        header(e, c.header_span);
                    }
                }
            }
            Declaration::Const(c) => header(&c.value, c.span),
            _ => {}
        }
        let body: &[crate::syntax::parser::ast::Statement] = match d {
            Declaration::Page(p) => &p.body,
            Declaration::Component(c) => &c.body,
            Declaration::App(a) => &a.body,
            Declaration::Store(s) => &s.body,
            _ => &[],
        };
        crate::syntax::parser::walk::visit(body, &mut |e, at| f(e, at, &file));
    }
}

/// Every `t()` call the program writes: in bodies, in a page's title,
/// description, guard, paths, head and layout, in a prop's default, and in
/// a `const`.
pub fn uses(program: &Program, file_of: &dyn Fn(usize) -> String) -> Vec<Use> {
    let mut out = Vec::new();
    for_each_expr(program, file_of, &mut |e, at, file| {
        walk_expr(e, at, file, &mut out);
    });
    out
}

/// The keys a program may hand to `t()` through a value: when some call's
/// key is neither a literal nor a splice — `t(place.key)` — every string
/// literal the program writes may be one (`Place(key: "nav.learn")` in a
/// `const`). Empty when every call names its key.
fn passed_as_values(program: &Program) -> std::collections::HashSet<String> {
    fn walk(e: &Expr, opaque: &mut bool, literals: &mut std::collections::HashSet<String>) {
        match e {
            Expr::FunctionCall(name, args) if name == "t" => {
                if args.first().is_some_and(|a| key_of(a).is_none()) {
                    *opaque = true;
                }
            }
            Expr::StringLiteral(s) => {
                literals.insert(s.clone());
            }
            _ => {}
        }
        for child in e.children() {
            walk(child, opaque, literals);
        }
    }
    let mut opaque = false;
    let mut literals = std::collections::HashSet::new();
    for_each_expr(program, &|_| String::new(), &mut |e, _, _| {
        walk(e, &mut opaque, &mut literals);
    });
    if opaque {
        literals
    } else {
        std::collections::HashSet::new()
    }
}

/// The keys the runtime asks for on its own: a `validate` rule written
/// without a message shows `form.<rule>`.
pub fn implied(program: &Program) -> BTreeMap<String, &'static str> {
    fn walk(
        stmts: &[crate::syntax::parser::ast::Statement],
        out: &mut BTreeMap<String, &'static str>,
    ) {
        for s in stmts {
            if let StatementKind::Validate(v) = &s.kind {
                for r in v.rules.iter().filter(|r| r.message.is_none()) {
                    out.insert(format!("form.{}", r.name), "a validate rule's own message");
                }
            }
            for b in crate::syntax::parser::walk::child_bodies(s) {
                walk(b, out);
            }
        }
    }

    let mut out = BTreeMap::new();
    for d in &program.declarations {
        walk(crate::syntax::parser::walk::body_of(d), &mut out);
    }
    out
}

/// One locale, across the top.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Locale {
    pub code: String,
    /// `src/translations/ar.json`, relative to the project.
    pub file: String,
    pub exists: bool,
    pub default: bool,
    pub rtl: bool,
    /// The plural forms its rules use.
    pub forms: Vec<&'static str>,
    /// Of the keys, how many it has no message for, so a reader in it
    /// sees the default locale's instead.
    pub missing: usize,
    /// Of the plural keys it has, how many lack a form its rules use, so
    /// the number reads with `other`'s wording.
    pub forms_missing: usize,
    /// How many cells whose placeholders are not the default locale's.
    pub mismatched: usize,
    /// Why its file could not be read as messages (`expected `,` or `}`
    /// at line 3 column 5`): there is a file, and none of its messages
    /// are counted, so nothing should be written over it until it is fixed.
    pub unreadable: Option<String>,
}

/// One message: a key's form, in one locale.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Cell {
    /// The key in the file: `items.one`, or `greeting`.
    pub entry: String,
    /// The form, for a plural key.
    pub form: Option<&'static str>,
    pub text: Option<String>,
    pub placeholders: Vec<String>,
    /// Whether the locale's plural rules use this form.
    pub needed: bool,
    /// The placeholders differ from the default locale's: one shows as
    /// written, or one the call passes is never shown. `count` is left out
    /// for a plural form, where a language may spell the number.
    pub differs: bool,
    /// The line of the file it is written on.
    pub line: Option<u32>,
}

/// One key, down the side.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Key {
    pub key: String,
    /// Whether it has plural forms: `t("items", { count: n })` picks one.
    pub plural: bool,
    /// Each locale's cells, by locale: one for a plain key, one per form
    /// the key or the locale has for a plural one.
    pub cells: BTreeMap<String, Vec<Cell>>,
    /// The locales with no message for it at all.
    pub missing: Vec<String>,
    /// Where a `t()` names it.
    pub uses: Vec<Use>,
    /// What else asks for it, where nothing names it: the form rules.
    pub implied_by: Option<&'static str>,
    /// Whether a key built at run time may be it (`t("status.{s}")`), or one
    /// handed to `t()` as a value may (`t(place.key)` with `"nav.learn"` written
    /// elsewhere).
    pub maybe_used: bool,
    /// Nothing names it, nothing asks for it.
    pub unused: bool,
}

/// The whole table.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Table {
    /// The translations folder, relative to the project.
    pub dir: String,
    pub default_locale: String,
    pub locales: Vec<Locale>,
    pub keys: Vec<Key>,
    /// The `t()` calls that name a key no locale has (`I01`): the page
    /// shows the key itself.
    pub unknown: Vec<Use>,
    /// The CLDR release whose plural rules decide the forms.
    pub cldr: &'static str,
}

/// The line of `key` as a member name in a JSON text.
fn line_of(text: &str, key: &str) -> Option<u32> {
    let quoted = serde_json::to_string(key).ok()?;
    let mut from = 0;
    while let Some(at) = text[from..].find(&quoted) {
        let start = from + at;
        let after = text[start + quoted.len()..].trim_start();
        if after.starts_with(':') {
            return Some(text[..start].matches('\n').count() as u32 + 1);
        }
        from = start + quoted.len();
    }
    None
}

/// The table, from the files and the program.
pub fn build(
    program: &Program,
    file_of: &dyn Fn(usize) -> String,
    config: &crate::project::config::project::I18nConfig,
    files: &HashMap<String, Option<String>>,
) -> Table {
    let dir = config.dir.trim_end_matches('/').to_string();
    let mut tables: HashMap<String, BTreeMap<String, String>> = HashMap::new();
    let mut unreadable: HashMap<String, String> = HashMap::new();
    for locale in &config.locales {
        let text = files.get(locale).and_then(|t| t.as_deref());
        let parsed = match text.map(serde_json::from_str::<BTreeMap<String, String>>) {
            Some(Ok(t)) => t,
            Some(Err(e)) => {
                unreadable.insert(locale.clone(), e.to_string());
                BTreeMap::new()
            }
            None => BTreeMap::new(),
        };
        tables.insert(locale.clone(), parsed);
    }
    let default = config.default_locale.clone();
    let mut all_uses = uses(program, file_of);
    // Translated by its own words: each string a reader sees is a key,
    // and its words are the default locale's message.
    if config.literals {
        let literal = super::literals::uses(program, file_of);
        if let Some(t) = tables.get_mut(&default) {
            for u in &literal {
                t.entry(u.key.clone()).or_insert_with(|| u.key.clone());
            }
        }
        all_uses.extend(literal);
    }
    let implied = implied(program);
    let passed = passed_as_values(program);

    // The keys: every base any locale has, a plural one once.
    let mut bases: BTreeMap<String, bool> = BTreeMap::new();
    for t in tables.values() {
        for k in t.keys() {
            let (base, form) = split_form(k);
            // A key is plural when it has an `other` form somewhere, or
            // any form of one that does; `x.one` alone is a plain key.
            let plural = form.is_some()
                && tables
                    .values()
                    .any(|t| t.contains_key(&format!("{base}.other")));
            let entry = bases
                .entry(if plural { base.to_string() } else { k.clone() })
                .or_insert(false);
            *entry |= plural;
        }
    }
    let rtl_default = default.clone();
    let mut locales: Vec<Locale> = config
        .locales
        .iter()
        .map(|code| Locale {
            code: code.clone(),
            file: format!("{dir}/{code}.json"),
            exists: files.get(code).is_some_and(std::option::Option::is_some),
            default: *code == rtl_default,
            rtl: is_rtl(code),
            forms: super::plurals::categories(code),
            missing: 0,
            forms_missing: 0,
            mismatched: 0,
            unreadable: unreadable.get(code).cloned(),
        })
        .collect();

    let mut keys = Vec::new();
    for (key, plural) in &bases {
        let base_of = |t: &BTreeMap<String, String>, form: &str| -> Option<String> {
            t.get(&format!("{key}.{form}")).cloned()
        };
        let default_table = tables.get(&default);
        let default_names = |form: Option<&str>| -> Option<Vec<String>> {
            let t = default_table?;
            let text = match form {
                Some(f) => base_of(t, f)
                    .or_else(|| base_of(t, "other"))
                    .or_else(|| t.get(key).cloned()),
                None => t.get(key).cloned(),
            }?;
            Some(placeholders(&text))
        };
        let mut cells = BTreeMap::new();
        let mut missing = Vec::new();
        for (li, loc) in config.locales.iter().enumerate() {
            let table = &tables[loc];
            let source = files.get(loc).and_then(|t| t.as_deref()).unwrap_or("");
            let rules = super::plurals::categories(loc);
            let mut row = Vec::new();
            if *plural {
                // The forms its rules use, then any other the file has.
                let mut forms: Vec<&'static str> = rules.clone();
                for f in FORMS {
                    if table.contains_key(&format!("{key}.{f}")) && !forms.contains(&f) {
                        forms.push(f);
                    }
                }
                forms.sort_by_key(|f| FORMS.iter().position(|x| x == f));
                // `items` itself, beside its forms: what every form falls
                // back to last.
                if let Some(text) = table.get(key) {
                    row.push(Cell {
                        entry: key.clone(),
                        form: None,
                        placeholders: placeholders(text),
                        needed: false,
                        differs: false,
                        line: line_of(source, key),
                        text: Some(text.clone()),
                    });
                }
                for f in forms {
                    let entry = format!("{key}.{f}");
                    let text = table.get(&entry).cloned();
                    let names = text.as_deref().map(placeholders).unwrap_or_default();
                    // The number that picks the form need not be said:
                    // `count`, or a literal's one number (`{files} files`).
                    let counted = placeholders(key);
                    let without_count = |v: &[String]| {
                        v.iter()
                            .filter(|n| *n != "count" && !(counted.len() == 1 && **n == counted[0]))
                            .cloned()
                            .collect::<BTreeSet<_>>()
                    };
                    let differs = text.is_some()
                        && *loc != default
                        && default_names(Some(f))
                            .is_some_and(|d| without_count(&d) != without_count(&names));
                    row.push(Cell {
                        line: text.as_ref().and_then(|_| line_of(source, &entry)),
                        entry,
                        form: Some(f),
                        placeholders: names,
                        needed: rules.contains(&f),
                        differs,
                        text,
                    });
                }
                let has_any = row.iter().any(|c| c.text.is_some()) || table.contains_key(key);
                if !has_any {
                    missing.push(loc.clone());
                    locales[li].missing += 1;
                } else if row.iter().any(|c| c.needed && c.text.is_none()) {
                    locales[li].forms_missing += 1;
                }
            } else {
                let text = table.get(key).cloned();
                let names = text.as_deref().map(placeholders).unwrap_or_default();
                let differs = text.is_some()
                    && *loc != default
                    && default_names(None).is_some_and(|d| {
                        d.iter().collect::<BTreeSet<_>>() != names.iter().collect::<BTreeSet<_>>()
                    });
                if text.is_none() {
                    missing.push(loc.clone());
                    locales[li].missing += 1;
                }
                row.push(Cell {
                    line: text.as_ref().and_then(|_| line_of(source, key)),
                    entry: key.clone(),
                    form: None,
                    placeholders: names,
                    needed: true,
                    differs,
                    text,
                });
            }
            locales[li].mismatched += row.iter().filter(|c| c.differs).count();
            cells.insert(loc.clone(), row);
        }
        let key_uses: Vec<Use> = all_uses
            .iter()
            .filter(|u| !u.dynamic && u.key == *key)
            .cloned()
            .collect();
        let maybe_used = passed.contains(key)
            || all_uses
                .iter()
                .any(|u| u.dynamic && !u.key.is_empty() && key.starts_with(&u.key));
        let implied_by = implied.get(key).copied();
        keys.push(Key {
            unused: key_uses.is_empty() && !maybe_used && implied_by.is_none(),
            key: key.clone(),
            plural: *plural,
            cells,
            missing,
            uses: key_uses,
            implied_by,
            maybe_used,
        });
    }
    let unknown = all_uses
        .iter()
        .filter(|u| {
            !u.dynamic
                && !bases.contains_key(&u.key)
                && !tables.values().any(|t| t.contains_key(&u.key))
        })
        .cloned()
        .collect();
    Table {
        dir,
        default_locale: default,
        locales,
        keys,
        unknown,
        cldr: super::plurals_data::CLDR_VERSION,
    }
}

/// The table of the project at `dir`, as the editor holds it: an unsaved
/// translation, or an unsaved page, is the one read. None when the project
/// has no `i18n`.
///
/// # Errors
///
/// [`IoError`](crate::diagnostics::error::WebFluentError::IoError) when the config or a source file cannot be read
/// (or `src/` holds none), and [`ConfigError`](crate::diagnostics::error::WebFluentError::ConfigError) when the config is
/// missing or is not one.
pub fn table_via(
    vfs: &dyn crate::base::vfs::Vfs,
    dir: &Path,
) -> crate::diagnostics::error::Result<Option<Table>> {
    let config = crate::project::config::ProjectConfig::load_via(vfs, dir)?;
    let Some(i18n) = &config.i18n else {
        return Ok(None);
    };
    let (program, files, _) = crate::project::sources::read_project_collecting_via(vfs, dir, None)?;
    let texts: HashMap<String, Option<String>> = i18n
        .locales
        .iter()
        .map(|l| {
            let path = dir.join(&i18n.dir).join(format!("{l}.json"));
            (l.clone(), vfs.read_to_string(&path).ok())
        })
        .collect();
    Ok(Some(build(
        &program,
        &|i| files.get(i).cloned().unwrap_or_default(),
        i18n,
        &texts,
    )))
}

fn edit_error(e: impl std::fmt::Display) -> crate::diagnostics::error::WebFluentError {
    crate::diagnostics::error::WebFluentError::edit(e.to_string())
}

/// `text` (a locale's file; empty when there is none yet) with the message
/// at `entry` set to `message`, every other byte as it was: an existing
/// message changed in place, a new one added at the end.
///
/// # Errors
///
/// [`EditError`](crate::diagnostics::error::WebFluentError::EditError) when `text` is not JSON, or is not an object.
pub fn set_message(
    text: &str,
    entry: &str,
    message: &str,
) -> crate::diagnostics::error::Result<String> {
    if text.trim().is_empty() {
        let mut map = serde_json::Map::new();
        map.insert(
            entry.to_string(),
            serde_json::Value::String(message.to_string()),
        );
        return Ok(format!(
            "{}\n",
            serde_json::to_string_pretty(&serde_json::Value::Object(map)).map_err(edit_error)?
        ));
    }
    crate::project::config::json_edit::set(
        text,
        &[entry],
        &serde_json::Value::String(message.to_string()),
    )
    .map_err(edit_error)
}

/// `text` without the message at `entry`.
///
/// # Errors
///
/// [`EditError`](crate::diagnostics::error::WebFluentError::EditError) when `text` is not JSON.
pub fn remove_message(text: &str, entry: &str) -> crate::diagnostics::error::Result<String> {
    crate::project::config::json_edit::remove(text, &[entry]).map_err(edit_error)
}

/// What a `Text(t(…))` binding is, for the inspector: its key, what it
/// passes, the default message and its placeholders, whether `count`
/// picks a form, and whether the table has it.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Call {
    pub key: String,
    /// Each name it passes, and the value as written.
    pub args: Vec<(String, String)>,
    pub plural: bool,
    /// The default locale's message (its `other` form, for a plural key).
    pub message: Option<String>,
    /// The placeholders that message has: one field each.
    pub placeholders: Vec<String>,
    /// No locale has the key, so the page shows the key itself.
    pub missing: bool,
}

/// The binding `e` is, when it is a `t()` with a written key.
pub fn call(e: &Expr, table: &Table) -> Option<Call> {
    let Expr::FunctionCall(name, args) = e else {
        return None;
    };
    if name != "t" {
        return None;
    }
    let Some(Expr::StringLiteral(key)) = args.first() else {
        return None;
    };
    let passed: Vec<(String, String)> = match args.get(1) {
        Some(Expr::MapLiteral(pairs)) => pairs
            .iter()
            .map(|(k, v)| {
                (
                    k.trim_matches('"').to_string(),
                    crate::syntax::print::expr(v),
                )
            })
            .collect(),
        _ => Vec::new(),
    };
    let row = table.keys.iter().find(|k| k.key == *key);
    let message = row.and_then(|k| {
        let cells = k.cells.get(&table.default_locale)?;
        cells
            .iter()
            .find(|c| c.form == Some("other"))
            .or_else(|| cells.first())
            .and_then(|c| c.text.clone())
    });
    Some(Call {
        key: key.clone(),
        args: passed,
        plural: row.is_some_and(|k| k.plural),
        placeholders: message.as_deref().map(placeholders).unwrap_or_default(),
        message,
        missing: row.is_none(),
    })
}

/// A `t()` call as source: `t("greeting", { name: user.name })`, each
/// value written as given.
pub fn call_text(key: &str, args: &[(String, String)]) -> String {
    let key = crate::syntax::print::string(key);
    if args.is_empty() {
        return format!("t({key})");
    }
    let pairs: Vec<String> = args.iter().map(|(k, v)| format!("{k}: {v}")).collect();
    format!("t({key}, {{ {} }})", pairs.join(", "))
}
