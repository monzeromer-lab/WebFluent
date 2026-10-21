//! Validation as data: every rule a `validate` block may hold, what it
//! takes and what it says when the author writes nothing; the patterns
//! people actually write, by name; when a `Form` shows its messages; the
//! rules one state is held to, in order, with the ones its type implies;
//! and the `validate` block written back beside the state, one rule at a
//! time.
//!
//! The messages are the runtime's own (`RULE_MESSAGES` in the form
//! module), and the key that replaces each in a project with translations
//! is the one the runtime asks for, `form.<rule>` — so an inspector shows
//! what the page would say.
//!
//! A preview that sets `data-wf-validation="shown"` on the page's root
//! shows every field's message now, as a failed submit would — the
//! *Invalid* side of a form's preview.

use serde::Serialize;

use crate::error::Result;
use crate::parser::ast::{Declaration, Expr, Program, Rule, Statement, StatementKind, TypeRef};

/// What a rule takes between its parentheses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Takes {
    /// `required`, `email`, `url`.
    Nothing,
    /// `minLength(8)`: a whole number of characters.
    Length,
    /// `min(v)`, `max(v)`: a number, a date, a time or a moment, which may
    /// read state (`max(Ledger.today)`).
    Bound,
    /// `pattern(/…/)`: a regular expression.
    Pattern,
    /// `matches(other)`: another value, which the rule follows.
    Value,
    /// `oneOf([…])`: a list of values.
    Values,
    /// `custom "…" { expr }`: an expression that is true when it holds.
    Check,
    /// `async "…" { await … }`: the same, answered later.
    Asked,
}

/// One rule a `validate` block may hold.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RuleSpec {
    pub name: &'static str,
    pub takes: Takes,
    /// When it holds, as the guide's table says.
    pub holds_when: &'static str,
    /// What it says when the author writes no message.
    pub message: &'static str,
    /// The translation key that replaces that message.
    pub key: String,
    /// The kinds of value it is a rule for: `text`, `number`, `date`,
    /// `time`, `moment`, `email`, `url`, or `any`.
    pub applies_to: &'static [&'static str],
    /// Whether an empty value passes it, so a blank optional field shows
    /// one message rather than two (every rule but `required` and
    /// `matches`).
    pub passes_empty: bool,
    /// Whether it is asked only once the rest pass, and once per value.
    pub asked_last: bool,
}

const fn rule(
    name: &'static str,
    takes: Takes,
    holds_when: &'static str,
    message: &'static str,
    applies_to: &'static [&'static str],
) -> (
    &'static str,
    Takes,
    &'static str,
    &'static str,
    &'static [&'static str],
) {
    (name, takes, holds_when, message, applies_to)
}

const ANY: &[&str] = &["any"];
const TEXT: &[&str] = &["text"];
const ORDERED: &[&str] = &["number", "date", "time", "moment"];

/// The rules, in the order the guide gives them; each message is the
/// runtime's.
const TABLE: &[(&str, Takes, &str, &str, &[&str])] = &[
    rule(
        "required",
        Takes::Nothing,
        "Something was given — a non-blank string, a non-empty list, a ticked checkbox",
        "This is required",
        ANY,
    ),
    rule(
        "email",
        Takes::Nothing,
        "It looks like an email address",
        "Enter an email address",
        &["text", "email"],
    ),
    rule(
        "url",
        Takes::Nothing,
        "It looks like a web address",
        "Enter a web address",
        &["text", "url"],
    ),
    rule(
        "minLength",
        Takes::Length,
        "The text is at least that long",
        "Too short",
        TEXT,
    ),
    rule(
        "maxLength",
        Takes::Length,
        "The text is at most that long",
        "Too long",
        TEXT,
    ),
    rule(
        "min",
        Takes::Bound,
        "The number, date or time is at least it",
        "Too small",
        ORDERED,
    ),
    rule(
        "max",
        Takes::Bound,
        "The number, date or time is at most it",
        "Too large",
        ORDERED,
    ),
    rule(
        "pattern",
        Takes::Pattern,
        "The text matches",
        "That is not the right shape",
        TEXT,
    ),
    rule(
        "matches",
        Takes::Value,
        "It equals another value, which it follows",
        "The two do not match",
        ANY,
    ),
    rule(
        "oneOf",
        Takes::Values,
        "It is one of them",
        "Choose one of the options",
        ANY,
    ),
    rule(
        "custom",
        Takes::Check,
        "The expression is true",
        "That is not right",
        ANY,
    ),
    rule(
        "async",
        Takes::Asked,
        "The answer, once it arrives, is yes",
        "That is not right",
        ANY,
    ),
];

/// The conditions a refined type carries that are not rules an author
/// writes, but that validation holds a value to all the same:
/// `Number(0..10)` is `below(10)`, `Date(after: …)` is `after(…)`.
const IMPLIED: &[(&str, Takes, &str, &str, &[&str])] = &[
    rule(
        "below",
        Takes::Bound,
        "The number is less than it",
        "Too large",
        &["number"],
    ),
    rule(
        "after",
        Takes::Bound,
        "The date or time is later than it",
        "Too early",
        &["date", "time", "moment"],
    ),
    rule(
        "before",
        Takes::Bound,
        "The date or time is earlier than it",
        "Too late",
        &["date", "time", "moment"],
    ),
];

fn spec(
    row: &(
        &'static str,
        Takes,
        &'static str,
        &'static str,
        &'static [&'static str],
    ),
) -> RuleSpec {
    let (name, takes, holds_when, message, applies_to) = *row;
    RuleSpec {
        name,
        takes,
        holds_when,
        message,
        key: format!("form.{name}"),
        applies_to,
        passes_empty: !matches!(name, "required" | "matches"),
        asked_last: name == "async",
    }
}

/// Every rule a `validate` block may hold, in the guide's order.
pub fn rules() -> Vec<RuleSpec> {
    TABLE.iter().map(spec).collect()
}

/// The rules a refined type implies that are not written in a block.
pub fn implied_rules() -> Vec<RuleSpec> {
    IMPLIED.iter().map(spec).collect()
}

/// The rule called `name`, written or implied.
pub fn rule_named(name: &str) -> Option<RuleSpec> {
    TABLE.iter().chain(IMPLIED).find(|r| r.0 == name).map(spec)
}

/// A pattern offered by name: what people write in `pattern(/…/)`.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct NamedPattern {
    /// As a row shows it: `a number`.
    pub name: &'static str,
    /// The expression, as written between the slashes.
    pub source: &'static str,
    pub flags: &'static str,
    /// The message a row suggests for it.
    pub message: &'static str,
}

/// The handful of patterns people actually write. A letter is any
/// script's, not only the Latin alphabet's.
pub const PATTERNS: &[NamedPattern] = &[
    NamedPattern {
        name: "a number",
        source: "[0-9]",
        flags: "",
        message: "Include a number",
    },
    NamedPattern {
        name: "a letter",
        source: "\\p{L}",
        flags: "u",
        message: "Include a letter",
    },
    NamedPattern {
        name: "a capital letter",
        source: "\\p{Lu}",
        flags: "u",
        message: "Include a capital letter",
    },
    NamedPattern {
        name: "a small letter",
        source: "\\p{Ll}",
        flags: "u",
        message: "Include a small letter",
    },
    NamedPattern {
        name: "a symbol",
        source: "[^\\p{L}\\p{N}\\s]",
        flags: "u",
        message: "Include a symbol",
    },
    NamedPattern {
        name: "no spaces",
        source: "^\\S*$",
        flags: "",
        message: "Leave out the spaces",
    },
    NamedPattern {
        name: "only numbers",
        source: "^[0-9]*$",
        flags: "",
        message: "Use only numbers",
    },
    NamedPattern {
        name: "only letters",
        source: "^\\p{L}*$",
        flags: "u",
        message: "Use only letters",
    },
];

/// The named pattern a written one is, if it is one.
pub fn named_pattern(source: &str, flags: &str) -> Option<&'static NamedPattern> {
    PATTERNS
        .iter()
        .find(|p| p.source == source && p.flags == flags)
}

/// When a `Form` shows its messages, in the reader's terms.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Showing {
    /// As `show:` takes it: `onBlur`.
    pub name: String,
    /// The registry's summary.
    pub summary: String,
    /// As a sentence ends: "messages show *when you leave a field*".
    pub phrase: &'static str,
    pub default: bool,
}

/// `Form(show:)`'s cases, from the registry.
pub fn showing() -> Vec<Showing> {
    let cases = crate::registry::component("Form")
        .and_then(|f| f.props.iter().find(|p| p.name == "show"))
        .map(|p| match p.ty {
            crate::registry::PropType::Enum(c) => c,
            _ => &[],
        })
        .unwrap_or(&[]);
    cases
        .iter()
        .map(|c| Showing {
            name: c.name.to_string(),
            summary: c.summary.to_string(),
            phrase: match c.name {
                "onBlur" => "when you leave a field",
                "onSubmit" => "when you submit the form",
                "live" => "as you type",
                _ => "",
            },
            default: c.name == "onBlur",
        })
        .collect()
}

/// The attribute, and the value, that shows every message now.
pub const SHOWN: &str = "data-wf-validation=\"shown\"";

/// What a row's message is.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Message {
    /// None is written: the rule's own, or in a project with translations
    /// the key that replaces it.
    Rule { text: &'static str, key: String },
    /// `"Use at least 8 characters"`.
    Written { text: String },
    /// `t("signup.short")`.
    Key { key: String },
    /// Anything else — read from state, built from text — as written.
    Expression { source: String },
}

/// One rule a state is held to.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Row {
    pub rule: String,
    /// Each argument as written: `8`, `/[0-9]/`, `password`, `Ledger.today`.
    pub args: Vec<String>,
    /// For `pattern`, the named pattern it is, when it is one.
    pub pattern: Option<&'static str>,
    pub message: Message,
    /// What `custom` and `async` check, as written.
    pub check: Option<String>,
    /// Where it is written; for a rule the type implies, where the type is.
    pub line: u32,
    pub start: u32,
    pub end: u32,
    /// One the state's type implies rather than one the block writes.
    pub from_type: bool,
}

/// The rules one state is held to.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Must {
    /// The page, component or store the state is declared in.
    pub owner: String,
    pub state: String,
    /// Its type as written, when it has one.
    pub ty: Option<String>,
    /// Where the `validate` block is: its line, start and end. None when
    /// there is none yet; writing a rule makes one beside the state.
    pub block: Option<(u32, u32, u32)>,
    /// The rules its type implies, then the block's, in order.
    pub rows: Vec<Row>,
    /// Whether the page checks them as the reader types: only a state with
    /// a `validate` block is registered with its form. Without one, a
    /// refined type holds only the values the compiler can read.
    pub checked: bool,
    /// Whether the project has translations, so an empty message is a key.
    pub translated: bool,
}

fn body_of(d: &Declaration) -> Option<(&str, &[Statement])> {
    match d {
        Declaration::Page(p) => Some((&p.name, &p.body)),
        Declaration::Component(c) => Some((&c.name, &c.body)),
        Declaration::Store(s) => Some((&s.name, &s.body)),
        _ => None,
    }
}

fn message_of(rule: &str, written: &Option<Expr>) -> Message {
    match written {
        None => Message::Rule {
            text: rule_named(rule)
                .map(|r| r.message)
                .unwrap_or("That is not right"),
            key: format!("form.{rule}"),
        },
        Some(Expr::StringLiteral(s)) => Message::Written { text: s.clone() },
        Some(Expr::FunctionCall(f, args))
            if f == "t" && matches!(args.first(), Some(Expr::StringLiteral(_))) =>
        {
            let Some(Expr::StringLiteral(k)) = args.first() else {
                unreachable!()
            };
            Message::Key { key: k.clone() }
        }
        Some(other) => Message::Expression {
            source: crate::print::expr(other),
        },
    }
}

fn row(r: &Rule) -> Row {
    let pattern = match (r.name.as_str(), r.args.first()) {
        ("pattern", Some(Expr::Regex(s, f))) => named_pattern(s, f).map(|p| p.name),
        _ => None,
    };
    Row {
        rule: r.name.clone(),
        args: r.args.iter().map(crate::print::expr).collect(),
        pattern,
        message: message_of(&r.name, &r.message),
        check: r.body.as_ref().map(crate::print::expr),
        line: r.span.line,
        start: r.span.start,
        end: r.span.end,
        from_type: false,
    }
}

/// The rules a refined type implies, as the build registers them.
fn implied(ty: &TypeRef, at: (u32, u32, u32)) -> Vec<Row> {
    ty.refinement()
        .iter()
        .filter(|(name, _)| rule_named(name).is_some())
        .map(|(name, bound)| Row {
            rule: name.clone(),
            args: vec![crate::print::expr(bound)],
            pattern: match bound {
                Expr::Regex(s, f) => named_pattern(s, f).map(|p| p.name),
                _ => None,
            },
            message: message_of(name, &None),
            check: None,
            line: at.0,
            start: at.1,
            end: at.2,
            from_type: true,
        })
        .collect()
}

/// The rules `state` in `owner` is held to, or none when `owner` declares
/// no such state.
pub fn must_be(program: &Program, owner: &str, state: &str, translated: bool) -> Option<Must> {
    let (_, body) = program
        .declarations
        .iter()
        .filter_map(body_of)
        .find(|(name, _)| *name == owner)?;
    let decl = body.iter().find_map(|s| match &s.kind {
        StatementKind::State(d) if d.name == state => Some((d, s.span)),
        _ => None,
    })?;
    let block = body.iter().find_map(|s| match &s.kind {
        StatementKind::Validate(v) if v.name == state => Some((v, s.span)),
        _ => None,
    });
    let mut rows = decl
        .0
        .ty
        .as_ref()
        .map(|t| implied(t, (decl.1.line, decl.1.start, decl.1.end)))
        .unwrap_or_default();
    if let Some((v, _)) = block {
        rows.extend(v.rules.iter().map(row));
    }
    Some(Must {
        owner: owner.to_string(),
        state: state.to_string(),
        ty: decl.0.ty.as_ref().map(crate::print::type_ref),
        block: block.map(|(_, s)| (s.line, s.start, s.end)),
        checked: block.is_some(),
        rows,
        translated,
    })
}

/// Every state in the program that is held to a rule, written or implied.
pub fn all(program: &Program, translated: bool) -> Vec<Must> {
    let mut out = Vec::new();
    for (owner, body) in program.declarations.iter().filter_map(body_of) {
        for s in body {
            if let StatementKind::State(d) = &s.kind
                && let Some(m) = must_be(program, owner, &d.name, translated)
                && !m.rows.is_empty()
            {
                out.push(m);
            }
        }
    }
    out
}

/// The rules `state` in `owner` is held to, in the project at `dir` as the
/// editor holds it: an unsaved edit is the one read.
pub fn must_be_via(
    vfs: &dyn crate::vfs::Vfs,
    dir: &std::path::Path,
    owner: &str,
    state: &str,
) -> Result<Option<Must>> {
    let (program, _, _) = crate::build::read_project_collecting_via(vfs, dir, None)?;
    let config = crate::config::ProjectConfig::load_via(vfs, dir)?;
    Ok(must_be(&program, owner, state, config.i18n.is_some()))
}

/// The state a control is bound to: `Input(bind: password)` is `password`.
pub fn bound_state(el: &crate::parser::ast::UIElement) -> Option<&str> {
    el.args.iter().find_map(|a| match a {
        crate::parser::ast::Arg::Named(k, Expr::Identifier(name)) if k == "bind" => {
            Some(name.as_str())
        }
        _ => None,
    })
}

/// One rule as an inspector writes it: each argument, the message and the
/// check as source text (`8`, `/[0-9]/`, `"Use at least 8"`, `t("k")`).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct RuleText {
    pub rule: String,
    pub args: Vec<String>,
    /// None leaves the rule's own message.
    pub message: Option<String>,
    pub check: Option<String>,
}

impl RuleText {
    pub fn new(rule: &str) -> Self {
        RuleText {
            rule: rule.to_string(),
            ..Default::default()
        }
    }

    fn line(&self) -> String {
        let mut out = self.rule.clone();
        if !self.args.is_empty() {
            out.push_str(&format!("({})", self.args.join(", ")));
        }
        if let Some(m) = &self.message {
            out.push(' ');
            out.push_str(m);
        }
        if let Some(c) = &self.check {
            out.push_str(&format!(" {{ {c} }}"));
        }
        out
    }
}

/// The rules as the parser reads them, refusing a rule that is not one or
/// text that does not read as its argument.
fn parsed(rules: &[RuleText]) -> Result<Vec<Rule>> {
    let err = |m: String| crate::error::WebFluentError::EditError(m);
    for r in rules {
        if !TABLE.iter().any(|t| t.0 == r.rule) {
            return Err(err(format!(
                "`{}` is not a rule a validate block may hold",
                r.rule
            )));
        }
    }
    let lines: Vec<String> = rules
        .iter()
        .map(|r| format!("        {}", r.line()))
        .collect();
    let probe = format!(
        "page ZzRules(path: \"/\") {{\n    state zz = \"\"\n    validate zz {{\n{}\n    }}\n}}\n",
        lines.join("\n")
    );
    let program = crate::syntax::parse_source(&probe, "rules.wf")?;
    let Some(Declaration::Page(p)) = program.declarations.first() else {
        return Err(err("the rules do not read as a validate block".into()));
    };
    let got = p.body.iter().find_map(|s| match &s.kind {
        StatementKind::Validate(v) => Some(v.rules.clone()),
        _ => None,
    });
    match got {
        // Each reads as the one rule it was given: its name, as many
        // arguments, a message and a check where it has them.
        Some(r)
            if r.len() == rules.len()
                && r.iter().zip(rules).all(|(a, b)| {
                    a.name == b.rule
                        && a.args.len() == b.args.len()
                        && a.message.is_some() == b.message.is_some()
                        && a.body.is_some() == b.check.is_some()
                }) =>
        {
            Ok(r)
        }
        _ => Err(err("the rules do not read as one rule a line".into())),
    }
}

fn is_state_or_block(s: &Statement) -> bool {
    matches!(
        &s.kind,
        StatementKind::State(_) | StatementKind::Validate(_)
    )
}

/// Where a new block for the state at `at` goes. Written with each state's
/// block beside it, right after the state; written as the states and then
/// their blocks, among the blocks in the states' order — after the last
/// state when there are none yet.
fn place_for(body: &[Statement], at: usize) -> usize {
    let mut first = at;
    while first > 0 && is_state_or_block(&body[first - 1]) {
        first -= 1;
    }
    let mut last = at;
    while last + 1 < body.len() && is_state_or_block(&body[last + 1]) {
        last += 1;
    }
    let run = &body[first..=last];
    let states: Vec<&str> = run
        .iter()
        .filter_map(|s| match &s.kind {
            StatementKind::State(d) => Some(d.name.as_str()),
            _ => None,
        })
        .collect();
    let last_state = run
        .iter()
        .rposition(|s| matches!(&s.kind, StatementKind::State(_)))
        .unwrap_or(0);
    let interleaved = run[..last_state]
        .iter()
        .any(|s| matches!(&s.kind, StatementKind::Validate(_)));
    if interleaved {
        return at + 1;
    }
    let StatementKind::State(mine) = &body[at].kind else {
        return at + 1;
    };
    let order = |name: &str| states.iter().position(|s| *s == name).unwrap_or(usize::MAX);
    let me = order(&mine.name);
    // Before the first block of a state declared after this one.
    for (i, s) in run.iter().enumerate().skip(last_state + 1) {
        if let StatementKind::Validate(v) = &s.kind
            && order(&v.name) > me
        {
            return first + i;
        }
    }
    last + 1
}

/// Write `state`'s `validate` block in `owner` as `rules`, in that order:
/// changing the block in its narrowest piece, creating it after the state
/// when there is none, and taking it out when `rules` is empty. Every
/// other byte of the file stays as it was.
pub fn write_rules(
    source: &str,
    file: &str,
    owner: &str,
    state: &str,
    rules: &[RuleText],
) -> Result<String> {
    let new = parsed(rules)?;
    let err = |m: String| crate::error::WebFluentError::EditError(m);
    let mut missing = false;
    let out = crate::edit::rewrite_body(source, file, owner, |body| {
        let at_block = body
            .iter()
            .position(|s| matches!(&s.kind, StatementKind::Validate(v) if v.name == state));
        match at_block {
            Some(i) if new.is_empty() => {
                body.remove(i);
            }
            Some(i) => {
                if let StatementKind::Validate(v) = &mut body[i].kind {
                    v.rules = new;
                }
            }
            None if new.is_empty() => {}
            None => {
                let Some(after) = body
                    .iter()
                    .position(|s| matches!(&s.kind, StatementKind::State(d) if d.name == state))
                else {
                    missing = true;
                    return;
                };
                let stmt = Statement {
                    kind: StatementKind::Validate(crate::parser::ast::ValidateDecl {
                        name: state.to_string(),
                        rules: new,
                        span: Default::default(),
                    }),
                    span: Default::default(),
                };
                body.insert(place_for(body, after), stmt);
            }
        }
    })?;
    if missing {
        return Err(err(format!("{owner} declares no state `{state}`")));
    }
    Ok(keep_one_line(source, out, file, owner, state, rules))
}

/// A block written on one line stays on one line while it fits, so an
/// edit keeps the shape the author chose.
fn keep_one_line(
    source: &str,
    out: String,
    file: &str,
    owner: &str,
    state: &str,
    rules: &[RuleText],
) -> String {
    if file.ends_with(".wfx") || rules.is_empty() {
        return out;
    }
    let block = |text: &str| {
        let program = crate::syntax::parse_source(text, file).ok()?;
        must_be(&program, owner, state, false)?.block
    };
    let (Some((_, s0, e0)), Some((_, s1, e1))) = (block(source), block(&out)) else {
        return out;
    };
    if source[s0 as usize..e0 as usize].contains('\n') {
        return out;
    }
    let line = format!(
        "validate {state} {{ {} }}",
        rules
            .iter()
            .map(RuleText::line)
            .collect::<Vec<_>>()
            .join("  ")
    );
    let (s1, e1) = (s1 as usize, e1 as usize);
    let column = s1 - out[..s1].rfind('\n').map(|i| i + 1).unwrap_or(0);
    if column + line.chars().count() > 100 {
        return out;
    }
    let one = format!("{}{line}{}", &out[..s1], &out[e1..]);
    // Held to what it says: the same rules, in the same order.
    let same = |text: &str| {
        let program = crate::syntax::parse_source(text, file).ok()?;
        Some(
            must_be(&program, owner, state, false)?
                .rows
                .into_iter()
                .map(|r| (r.rule, r.args, r.message, r.check))
                .collect::<Vec<_>>(),
        )
    };
    match (same(&one), same(&out)) {
        (Some(a), Some(b)) if a == b => one,
        _ => out,
    }
}

/// The block's rules as source text, to edit one and write them back.
pub fn rule_texts(source: &str, file: &str, owner: &str, state: &str) -> Result<Vec<RuleText>> {
    let program = crate::syntax::parse_source(source, file)?;
    if must_be(&program, owner, state, false).is_none() {
        return Err(crate::error::WebFluentError::EditError(format!(
            "{owner} declares no state `{state}`"
        )));
    }
    let body = program
        .declarations
        .iter()
        .filter_map(body_of)
        .find(|(n, _)| *n == owner)
        .map(|(_, b)| b)
        .unwrap_or(&[]);
    let written: Vec<&Rule> = body
        .iter()
        .find_map(|s| match &s.kind {
            StatementKind::Validate(v) if v.name == state => Some(v.rules.iter().collect()),
            _ => None,
        })
        .unwrap_or_default();
    Ok(written
        .into_iter()
        .map(|r| RuleText {
            rule: r.name.clone(),
            args: r.args.iter().map(crate::print::expr).collect(),
            message: r.message.as_ref().map(crate::print::expr),
            check: r.body.as_ref().map(crate::print::expr),
        })
        .collect())
}

/// Set the rule at `index` of the block (the rules the block writes, not
/// the ones the type implies) to `rule`, or add it at the end when `index`
/// is none.
pub fn set_rule(
    source: &str,
    file: &str,
    owner: &str,
    state: &str,
    index: Option<usize>,
    rule: RuleText,
) -> Result<String> {
    let mut rules = rule_texts(source, file, owner, state)?;
    match index {
        Some(i) if i < rules.len() => rules[i] = rule,
        Some(i) => {
            return Err(crate::error::WebFluentError::EditError(format!(
                "the block has no rule {i}"
            )));
        }
        None => rules.push(rule),
    }
    write_rules(source, file, owner, state, &rules)
}

/// Take out the rule at `index`; the block goes with its last rule.
pub fn remove_rule(
    source: &str,
    file: &str,
    owner: &str,
    state: &str,
    index: usize,
) -> Result<String> {
    let mut rules = rule_texts(source, file, owner, state)?;
    if index >= rules.len() {
        return Err(crate::error::WebFluentError::EditError(format!(
            "the block has no rule {index}"
        )));
    }
    rules.remove(index);
    write_rules(source, file, owner, state, &rules)
}

/// Move the rule at `from` to `to`: the rules are checked, and their
/// messages shown, in order.
pub fn move_rule(
    source: &str,
    file: &str,
    owner: &str,
    state: &str,
    from: usize,
    to: usize,
) -> Result<String> {
    let mut rules = rule_texts(source, file, owner, state)?;
    if from >= rules.len() || to >= rules.len() {
        return Err(crate::error::WebFluentError::EditError(
            "no such rule".into(),
        ));
    }
    let r = rules.remove(from);
    rules.insert(to, r);
    write_rules(source, file, owner, state, &rules)
}

/// Everything a Must be section offers.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Catalogue {
    pub rules: Vec<RuleSpec>,
    pub implied: Vec<RuleSpec>,
    pub patterns: &'static [NamedPattern],
    pub showing: Vec<Showing>,
    /// The attribute a preview sets on the page's root to show every
    /// message now.
    pub shown_attribute: &'static str,
}

pub fn catalogue() -> Catalogue {
    Catalogue {
        rules: rules(),
        implied: implied_rules(),
        patterns: PATTERNS,
        showing: showing(),
        shown_attribute: SHOWN,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_rules_are_the_checker_s() {
        let names: Vec<&str> = rules().iter().map(|r| r.name).collect();
        assert_eq!(names, crate::sema::types::VALIDATE_RULES);
        assert_eq!(rule_named("minLength").unwrap().key, "form.minLength");
        assert!(!rule_named("required").unwrap().passes_empty);
        assert!(rule_named("async").unwrap().asked_last);
        assert_eq!(rule_named("below").unwrap().message, "Too large");
    }

    #[test]
    fn the_form_s_cases_each_have_a_phrase() {
        let s = showing();
        assert_eq!(
            s.iter().map(|c| c.name.as_str()).collect::<Vec<_>>(),
            ["onBlur", "onSubmit", "live"]
        );
        assert!(s.iter().all(|c| !c.phrase.is_empty()));
        assert_eq!(s.iter().filter(|c| c.default).count(), 1);
    }

    #[test]
    fn a_named_pattern_is_recognised() {
        assert_eq!(named_pattern("[0-9]", "").map(|p| p.name), Some("a number"));
        assert_eq!(named_pattern("[0-9]", "u"), None);
    }
}
