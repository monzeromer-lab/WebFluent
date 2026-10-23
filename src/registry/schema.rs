//! Every component a call can name, the built-ins and the project's own,
//! described in one vocabulary — so an inspector draws a control per prop,
//! an Insert panel lists what can go where, and an agent reads what a call
//! takes, without telling the two kinds apart.
//!
//! A prop says what kind of value it takes (`text`, `number`, `enum` with
//! its cases, `file` with what kind of file, `state`, `path`, a declared
//! `record`, a `list`, one of the language's own types — `date`, `color`,
//! `money` …), whether it is the positional one, its default as it is
//! written, a range where its type declares one (`Number(0..=100)`),
//! whether `.name` alone sets it, and where its value goes. A component
//! says its events, its slots and what each hands its fill, its parts and
//! their owner, which attribute families reach its root, and whether it
//! takes children — which is what makes a drop legal or not.
//!
//! The built-ins come from [`super::COMPONENTS`]; the project's from its
//! declarations, read the way the checker reads a call of them.

use serde::Serialize;

use super::{AttrFamily, Children, ComponentSig, Legacy, PropSig, PropType, Sink};
use crate::parser::ast::*;

/// A component, built-in or the project's.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Component {
    /// As a call names it: `Button`, `Card.Header`, `DeployRow`.
    pub name: String,
    /// The component a part belongs to; a part is placed only inside it.
    pub owner: Option<String>,
    /// `builtin`, or `project`.
    pub source: &'static str,
    /// The registry's group (`Layout`, `Form` …), or `This project`.
    pub group: String,
    /// What it is: the registry's sentence, or the `///` above it.
    pub summary: Option<String>,
    /// Where it is declared, for one of the project's.
    pub file: Option<String>,
    pub line: Option<u32>,
    /// Its props, the positional one first.
    pub props: Vec<Prop>,
    /// The events it fires beyond the DOM's, which every element has.
    pub events: Vec<Event>,
    /// The slots a call fills: `children` for what goes in its block.
    pub slots: Vec<Slot>,
    /// Its parts, as a call names them: `Card.Header`.
    pub parts: Vec<String>,
    /// The attribute families its root takes: `global`, `aria`, `data` …
    pub attributes: Vec<&'static str>,
    /// `elements`, when what is in its block is its children; `none`.
    pub children: &'static str,
    /// The tag and class its root is written with, for a built-in.
    pub tag: Option<&'static str>,
    pub class: Option<&'static str>,
}

/// One prop.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Prop {
    pub name: String,
    /// What kind of value it takes, which is the control an inspector
    /// draws: `text`, `number`, `bool`, `enum`, `state`, `path`,
    /// `declaration`, `file`, `any`, `list`, `map`, `record`, or a type of
    /// the language's own — `date`, `time`, `datetime`, `duration`,
    /// `money`, `url`, `email`, `color`, `uuid`, `upload`, `secret`.
    pub kind: &'static str,
    /// The type as it is written: `String`, `[Todo]`, `Tone?`.
    #[serde(rename = "type")]
    pub ty: String,
    /// An enum's cases, in order.
    pub cases: Vec<Case>,
    /// For `file`: what kind of file — `image`, `video`, `captions` …
    pub file: Option<&'static str>,
    /// For `record`, and a `list` of records or enums: the declared type.
    pub declared: Option<String>,
    /// For `list`: what kind of value each item is.
    pub item: Option<&'static str>,
    /// The bounds its type declares: `Number(0..=100)`.
    pub range: Option<Range>,
    /// Its default, as it is written; `None` when it has none.
    pub default: Option<String>,
    /// Whether a call must give it: no default, and not optional.
    pub required: bool,
    /// Whether it is the one a call may pass without its name.
    pub positional: bool,
    /// Whether `.name` alone sets it — a `Bool` that is a flag.
    pub flag: bool,
    /// Where its value ends up: `attribute`, `class`, `special` for a
    /// built-in, `prop` for one of the project's.
    pub sink: &'static str,
    /// The word a flag or case was written with in the original grammar.
    pub legacy: Option<&'static str>,
    pub summary: Option<String>,
}

/// One case of an enum prop.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Case {
    pub name: String,
    pub summary: Option<String>,
    /// Whether `.case` alone, as a flag, means this prop's case.
    pub flag: bool,
    /// Whether it is the case a prop has when nothing says otherwise.
    pub default: bool,
}

/// The bounds of a number, as its type says them.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Range {
    pub min: Option<f64>,
    pub max: Option<f64>,
    /// Whether `max` is a value the number may take (`..=`), or only
    /// below it (`..`).
    pub inclusive: bool,
}

/// An event a component fires.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Event {
    pub name: String,
    pub params: Vec<Param>,
    pub summary: Option<String>,
}

/// A slot a call fills.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Slot {
    /// `children` for the default slot.
    pub name: String,
    /// What it hands its fill: `slot row(item: Todo)`.
    pub params: Vec<Param>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Param {
    pub name: String,
    #[serde(rename = "type")]
    pub ty: String,
}

/// Everything at once: every component, the props and events every
/// element takes, and the attributes of each family.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Catalog {
    pub version: &'static str,
    pub components: Vec<Component>,
    /// What every element takes: `animate`, `exit`, `delay` …
    pub universal: Vec<Prop>,
    /// The DOM's events, which every element fires.
    pub dom_events: Vec<Event>,
}

fn family(f: AttrFamily) -> &'static str {
    match f {
        AttrFamily::Global => "global",
        AttrFamily::Aria => "aria",
        AttrFamily::Data => "data",
        AttrFamily::Input => "input",
        AttrFamily::Anchor => "anchor",
        AttrFamily::Media => "media",
        AttrFamily::Form => "form",
        AttrFamily::Table => "table",
        AttrFamily::Any => "any",
    }
}

// ─── The built-ins ───────────────────────────────────────

fn builtin_prop(p: &'static PropSig, sig: Option<&'static ComponentSig>, positional: bool) -> Prop {
    let (kind, ty) = match p.ty {
        PropType::Str => ("text", "String"),
        PropType::File(_) => ("file", "String"),
        PropType::Num => ("number", "Number"),
        PropType::Bool => ("bool", "Bool"),
        PropType::State => ("state", "State"),
        PropType::Path => ("path", "Path"),
        PropType::Decl => ("declaration", "Declaration"),
        PropType::Any => ("any", "Any"),
        PropType::Enum(_) => ("enum", "Enum"),
    };
    let cases = match p.ty {
        PropType::Enum(cases) => cases
            .iter()
            .map(|c| Case {
                name: c.name.to_string(),
                summary: Some(c.summary.to_string()).filter(|s| !s.is_empty()),
                flag: p.shorthand
                    && sig.is_none_or(|s| matches!(s.flag(c.name), super::Flag::Case(..))),
                default: c.legacy.is_empty(),
            })
            .collect(),
        _ => Vec::new(),
    };
    Prop {
        name: p.name.to_string(),
        kind,
        ty: ty.to_string(),
        cases,
        file: match p.ty {
            PropType::File(k) => Some(k.word()),
            _ => None,
        },
        declared: None,
        item: None,
        range: None,
        default: None,
        required: p.required,
        positional,
        flag: p.shorthand && p.ty == PropType::Bool,
        sink: match p.sink {
            Sink::Attr => "attribute",
            Sink::Class => "class",
            Sink::Special => "special",
        },
        legacy: match p.legacy {
            Legacy::Modifier(word) => Some(word),
            Legacy::Attr => None,
        },
        summary: Some(p.summary.to_string()),
    }
}

/// A built-in, by the name a call gives it: `Button`, `Card.Header`.
pub fn builtin(name: &str) -> Option<Component> {
    super::COMPONENTS
        .iter()
        .find(|c| c.qualified() == name)
        .map(builtin_of)
}

fn builtin_of(sig: &'static ComponentSig) -> Component {
    let (tag, class) = match sig.ir {
        super::Ir::BuiltIn(name) => {
            let (t, c) = crate::codegen::builtin::builtin_to_html(name);
            (Some(t), Some(c))
        }
        super::Ir::Sub(..) => (None, None),
    };
    let mut props: Vec<Prop> = sig
        .positional
        .iter()
        .map(|p| builtin_prop(p, Some(sig), true))
        .collect();
    props.extend(sig.props.iter().map(|p| builtin_prop(p, Some(sig), false)));
    Component {
        name: sig.qualified(),
        owner: sig.owner.map(str::to_string),
        source: "builtin",
        group: sig.group.to_string(),
        summary: Some(sig.summary.to_string()),
        file: None,
        line: None,
        props,
        events: sig
            .events
            .iter()
            .map(|e| Event {
                name: e.to_string(),
                params: Vec::new(),
                summary: None,
            })
            .collect(),
        slots: match sig.children {
            Children::Elements => vec![Slot {
                name: "children".into(),
                params: Vec::new(),
            }],
            Children::None => Vec::new(),
        },
        parts: sig
            .parts
            .iter()
            .map(|p| format!("{}.{p}", sig.qualified()))
            .collect(),
        attributes: sig.attrs.iter().map(|a| family(*a)).collect(),
        children: match sig.children {
            Children::Elements => "elements",
            Children::None => "none",
        },
        tag: tag.filter(|t| !t.is_empty()),
        class: class.filter(|c| !c.is_empty()),
    }
}

// ─── The project's ───────────────────────────────────────

/// How a declared type reads as a prop's kind.
struct Kind {
    kind: &'static str,
    cases: Vec<String>,
    declared: Option<String>,
    item: Option<&'static str>,
    range: Option<Range>,
    optional: bool,
}

fn scalar_kind(name: &str) -> Option<&'static str> {
    Some(match name {
        "Date" => "date",
        "Time" => "time",
        "DateTime" => "datetime",
        "Duration" => "duration",
        "Money" => "money",
        "Url" => "url",
        "Email" => "email",
        "Color" => "color",
        "Uuid" => "uuid",
        "File" => "upload",
        "Secret" => "secret",
        _ => return None,
    })
}

fn kind_of(ty: &TypeRef, program: &Program) -> Kind {
    let plain = |kind| Kind {
        kind,
        cases: Vec::new(),
        declared: None,
        item: None,
        range: None,
        optional: false,
    };
    match ty {
        TypeRef::String => plain("text"),
        TypeRef::Number => plain("number"),
        TypeRef::Bool => plain("bool"),
        TypeRef::Map => plain("map"),
        TypeRef::Any => plain("any"),
        TypeRef::Optional(inner) => Kind {
            optional: true,
            ..kind_of(inner, program)
        },
        TypeRef::List(inner) => {
            let item = kind_of(inner, program);
            Kind {
                kind: "list",
                cases: Vec::new(),
                declared: item.declared,
                item: Some(item.kind),
                range: None,
                optional: false,
            }
        }
        TypeRef::Refined(base, args) => {
            let mut k = kind_of(base, program);
            let num = |key: &str| {
                args.iter().find_map(|(k, v)| match (k.as_str() == key, v) {
                    (true, Expr::NumberLiteral(n)) => Some(*n),
                    (true, Expr::UnaryOp(UnaryOp::Neg, inner)) => match inner.as_ref() {
                        Expr::NumberLiteral(n) => Some(-n),
                        _ => None,
                    },
                    _ => None,
                })
            };
            if k.kind == "number" {
                let (max, inclusive) = match (num("max"), num("below")) {
                    (Some(m), _) => (Some(m), true),
                    (None, Some(b)) => (Some(b), false),
                    (None, None) => (None, true),
                };
                let min = num("min");
                if min.is_some() || max.is_some() {
                    k.range = Some(Range {
                        min,
                        max,
                        inclusive,
                    });
                }
            }
            k
        }
        TypeRef::Named(name) => {
            if let Some(kind) = scalar_kind(name) {
                return plain(kind);
            }
            let declared = program.declarations.iter().find_map(|d| match d {
                Declaration::Enum(e) if e.name == *name => Some(("enum", e.case_names())),
                Declaration::Type(t) if t.name == *name => Some(("record", Vec::new())),
                _ => None,
            });
            match declared {
                Some((kind, cases)) => Kind {
                    kind,
                    cases,
                    declared: Some(name.clone()),
                    item: None,
                    range: None,
                    optional: false,
                },
                // A name nothing declares: the checker says so; here it is
                // anything.
                None => Kind {
                    declared: Some(name.clone()),
                    ..plain("any")
                },
            }
        }
    }
}

fn written(ty: &TypeRef) -> String {
    crate::print::type_ref(ty)
}

fn params(ps: &[ParamDecl]) -> Vec<Param> {
    ps.iter()
        .map(|p| Param {
            name: p.name.clone(),
            ty: written(&p.param_type),
        })
        .collect()
}

/// One of the project's components, as a call of it is read.
pub fn project(c: &ComponentDecl, program: &Program, file: Option<String>) -> Component {
    // A flag is a Bool prop, or a case only one enum prop of it has.
    let kinds: Vec<Kind> = c
        .props
        .iter()
        .map(|p| kind_of(&p.prop_type, program))
        .collect();
    let case_owners = |case: &str| {
        kinds
            .iter()
            .filter(|k| k.kind == "enum" && k.cases.iter().any(|c| c == case))
            .count()
    };
    let mut props: Vec<Prop> = Vec::new();
    for (p, k) in c.props.iter().zip(&kinds) {
        let prop = Prop {
            name: p.name.clone(),
            kind: k.kind,
            ty: written(&p.prop_type),
            cases: k
                .cases
                .iter()
                .map(|case| Case {
                    name: case.clone(),
                    summary: None,
                    flag: case_owners(case) == 1,
                    default: p
                        .default
                        .as_ref()
                        .is_some_and(|d| matches!(d, Expr::EnumCase(x) if x == case)),
                })
                .collect(),
            file: None,
            declared: k.declared.clone(),
            item: k.item,
            range: k.range.clone(),
            default: p.default.as_ref().map(crate::print::expr),
            required: p.default.is_none() && !k.optional && !p.optional,
            positional: p.positional,
            flag: p.prop_type == TypeRef::Bool,
            sink: "prop",
            legacy: None,
            summary: p.doc.clone(),
        };
        if p.positional {
            props.insert(0, prop);
        } else {
            props.push(prop);
        }
    }
    let mut slots: Vec<Slot> = c
        .slots
        .iter()
        .map(|s| Slot {
            name: s.name.clone().unwrap_or_else(|| "children".to_string()),
            params: params(&s.params),
        })
        .collect();
    if !slots.iter().any(|s| s.name == "children") && super::json::uses_children(&c.body) {
        slots.push(Slot {
            name: "children".into(),
            params: Vec::new(),
        });
    }
    let (owner, name) = match c.name.rsplit_once('.') {
        Some((owner, _)) => (Some(owner.to_string()), c.name.clone()),
        None => (None, c.name.clone()),
    };
    Component {
        parts: c.parts.iter().map(|p| format!("{name}.{p}")).collect(),
        children: if slots.iter().any(|s| s.name == "children") {
            "elements"
        } else {
            "none"
        },
        name,
        owner,
        source: "project",
        group: "This project".to_string(),
        summary: c.doc.clone(),
        file,
        line: Some(c.span.line),
        props,
        events: c
            .events
            .iter()
            .map(|e| Event {
                name: e.name.clone(),
                params: params(&e.params),
                summary: e.doc.clone(),
            })
            .collect(),
        slots,
        // What reaches the root of one of the project's: the global
        // attributes, `aria-*` and `data-*`.
        attributes: vec!["global", "aria", "data"],
        tag: None,
        class: None,
    }
}

/// Every component a call can name: the built-ins, then — given a program
/// — the project's, in the order they are declared.
pub fn catalog(program: Option<(&Program, &dyn Fn(usize) -> String)>) -> Catalog {
    let mut components: Vec<Component> = super::COMPONENTS.iter().map(builtin_of).collect();
    if let Some((program, file_of)) = program {
        for (i, d) in program.declarations.iter().enumerate() {
            if let Declaration::Component(c) = d {
                components.push(project(c, program, Some(file_of(i))));
            }
        }
    }
    Catalog {
        version: env!("CARGO_PKG_VERSION"),
        components,
        universal: super::UNIVERSAL_PROPS
            .iter()
            .map(|p| builtin_prop(p, None, false))
            .collect(),
        dom_events: super::UNIVERSAL_EVENTS
            .iter()
            .map(|(name, summary)| Event {
                name: name.to_string(),
                params: Vec::new(),
                summary: Some(summary.to_string()),
            })
            .collect(),
    }
}

impl Catalog {
    /// The component a call names.
    pub fn get(&self, name: &str) -> Option<&Component> {
        self.components.iter().find(|c| c.name == name)
    }

    /// Whether `child` may be dropped directly inside `parent`: a part only
    /// inside its owner, nothing inside what takes no children.
    pub fn may_hold(&self, parent: &str, child: &str) -> Result<(), String> {
        let p = self
            .get(parent)
            .ok_or_else(|| format!("`{parent}` is not a component"))?;
        let c = self
            .get(child)
            .ok_or_else(|| format!("`{child}` is not a component"))?;
        if p.children == "none" && !p.parts.iter().any(|x| x == &c.name) {
            return Err(format!("`{parent}` takes no children"));
        }
        if let Some(owner) = &c.owner
            && owner != parent
        {
            return Err(format!("`{child}` goes only inside `{owner}`"));
        }
        Ok(())
    }
}

/// Every component a call in the project at `dir` can name, its sources
/// read through `vfs` — an editor's unsaved buffers count. A file that
/// does not parse leaves out only what it declares; a folder that is no
/// project gives the built-ins alone.
pub fn of_project_via(vfs: &dyn crate::vfs::Vfs, dir: &std::path::Path) -> Catalog {
    match crate::build::read_project_collecting_via(vfs, dir, None) {
        Ok((program, files, _)) => {
            let file_of = |i: usize| files.get(i).cloned().unwrap_or_default();
            catalog(Some((&program, &file_of)))
        }
        Err(_) => catalog(None),
    }
}
