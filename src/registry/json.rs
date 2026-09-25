//! The registry as JSON, for tools outside this crate — the studio's
//! inspector, which offers a dropdown for an enum prop, a toggle for a
//! flag, a handler editor for an event and a drop target for a slot, and
//! needs the same table the compiler checks against.
//!
//! `wf registry --json` prints [`registry_json`]; `wf types --json` prints
//! [`declarations_json`] for a project — its enums, records, components,
//! stores and pages, which are the other half of what a call site takes.

use serde_json::{Value, json};

use super::{AttrFamily, Children, ComponentSig, Legacy, PropSig, PropType, Sink};
use crate::parser::ast::*;

/// Every built-in: components and parts, with their props, cases, flags,
/// events, slots, parts and attribute families, plus the universal props
/// and events every element takes.
pub fn registry_json() -> Value {
    let components: Vec<Value> = super::COMPONENTS.iter().map(component_json).collect();
    json!({
        "version": env!("CARGO_PKG_VERSION"),
        "components": components,
        "universal": {
            "props": super::UNIVERSAL_PROPS.iter().map(|p| prop_json(p, None)).collect::<Vec<_>>(),
            "events": super::UNIVERSAL_EVENTS
                .iter()
                .map(|(name, summary)| json!({ "name": name, "summary": summary }))
                .collect::<Vec<_>>(),
        },
        "retiredModifiers": super::RETIRED_MODIFIERS,
        "icons": super::ICONS,
    })
}

fn component_json(sig: &'static ComponentSig) -> Value {
    let (tag, class) = match sig.ir {
        crate::registry::Ir::BuiltIn(name) => crate::codegen::builtin::builtin_to_html(name),
        crate::registry::Ir::Sub(..) => ("", ""),
    };
    let flags: Vec<Value> = sig.all_props().flat_map(|p| flags_of(sig, p)).collect();
    json!({
        "name": sig.name,
        "qualified": sig.qualified(),
        "owner": sig.owner,
        "group": sig.group,
        "summary": sig.summary,
        // The tag and the class its root carries. A tool that looks at a
        // built page — the studio's inspector, a browser sweep — has to
        // know which element is which, and guessing the name is how the
        // guess drifts from what the compiler writes.
        "tag": tag,
        "class": class,
        "positional": sig.positional.as_ref().map(|p| prop_json(p, Some(sig))),
        "props": sig.props.iter().map(|p| prop_json(p, Some(sig))).collect::<Vec<_>>(),
        "flags": flags,
        "events": sig.events,
        "parts": sig.parts,
        "attributes": sig.attrs.iter().map(|a| attr_family_name(*a)).collect::<Vec<_>>(),
        "children": match sig.children {
            Children::None => "none",
            Children::Elements => "elements",
        },
        "slots": match sig.children {
            Children::None => Vec::<&str>::new(),
            Children::Elements => vec!["children"],
        },
    })
}

/// The words that may follow a `.` on this element for this prop: the prop
/// itself when it is a `Bool`, or each of its cases when no other prop of
/// the element shares the case.
fn flags_of(sig: &'static ComponentSig, prop: &'static PropSig) -> Vec<Value> {
    if !prop.shorthand {
        return Vec::new();
    }
    match prop.ty {
        PropType::Bool => vec![json!({
            "name": prop.name,
            "prop": prop.name,
            "summary": prop.summary,
        })],
        PropType::Enum(cases) => cases
            .iter()
            .filter(|c| matches!(sig.flag(c.name), super::Flag::Case(..)))
            .map(|c| {
                json!({
                    "name": c.name,
                    "prop": prop.name,
                    "case": c.name,
                    "summary": c.summary,
                })
            })
            .collect(),
        _ => Vec::new(),
    }
}

fn prop_json(prop: &PropSig, sig: Option<&'static ComponentSig>) -> Value {
    let (ty, cases): (&str, Option<Vec<Value>>) = match prop.ty {
        PropType::Str => ("String", None),
        PropType::Num => ("Number", None),
        PropType::Bool => ("Bool", None),
        PropType::State => ("State", None),
        PropType::Path => ("Path", None),
        PropType::Decl => ("Declaration", None),
        PropType::Any => ("Any", None),
        PropType::Enum(cases) => (
            "Enum",
            Some(
                cases
                    .iter()
                    .map(|c| {
                        // Whether `.case` alone names this prop's case here.
                        let flag = prop.shorthand
                            && sig.is_none_or(|s| matches!(s.flag(c.name), super::Flag::Case(..)));
                        json!({
                            "name": c.name,
                            "summary": c.summary,
                            "default": c.legacy.is_empty(),
                            "flag": flag,
                        })
                    })
                    .collect(),
            ),
        ),
    };
    json!({
        "name": prop.name,
        "type": ty,
        "cases": cases,
        "summary": prop.summary,
        "sink": match prop.sink {
            Sink::Attr => "attribute",
            Sink::Class => "class",
            Sink::Special => "special",
        },
        "flag": prop.shorthand && matches!(prop.ty, PropType::Bool),
        "legacy": match prop.legacy {
            Legacy::Modifier(word) => Some(word),
            Legacy::Attr => None,
        },
    })
}

fn attr_family_name(family: AttrFamily) -> &'static str {
    match family {
        AttrFamily::Global => "global",
        AttrFamily::Aria => "aria",
        AttrFamily::Data => "data",
        AttrFamily::Input => "input",
        AttrFamily::Anchor => "anchor",
        AttrFamily::Media => "media",
        AttrFamily::Form => "form",
        AttrFamily::Table => "table",
    }
}

// ─── A project's own declarations ────────────────────────────────────────

/// What a project declares: its enums and records, its components with
/// their props, events and slots, its stores with their members, and its
/// pages with their routes — keyed the way the registry is, so a tool can
/// read a call site of `UserCard` as it reads one of `Button`.
pub fn declarations_json(program: &Program, file_of: &dyn Fn(usize) -> String) -> Value {
    let mut enums = Vec::new();
    let mut types = Vec::new();
    let mut components = Vec::new();
    let mut stores = Vec::new();
    let mut pages = Vec::new();
    let mut externals: Vec<Value> = Vec::new();
    for (ix, decl) in program.declarations.iter().enumerate() {
        let file = file_of(ix);
        match decl {
            Declaration::Enum(e) => enums.push(json!({
                "name": e.name,
                "cases": e.case_names(),
                "payloads": e.cases.iter().filter(|c| !c.fields.is_empty()).map(|c| json!({
                    "case": c.name,
                    "fields": c.fields.iter().map(|f| json!({
                        "name": f.name,
                        "type": type_ref_json(&f.ty),
                        "doc": f.doc,
                    })).collect::<Vec<_>>(),
                })).collect::<Vec<_>>(),
                "doc": e.doc,
                "file": file,
                "line": e.span.line,
            })),
            Declaration::Type(t) => types.push(json!({
                "name": t.name,
                "extends": t.extends,
                "doc": t.doc,
                "file": file,
                "line": t.span.line,
                "fields": t.all_fields(&|name| program.declarations.iter().find_map(|d| match d {
                    Declaration::Type(other) if other.name == name => Some(other),
                    _ => None,
                })).iter().map(|f| json!({
                    "name": f.name,
                    "type": type_ref_json(&f.ty),
                    "default": f.default.as_ref().map(crate::sema::types::expr_text),
                    "doc": f.doc,
                })).collect::<Vec<_>>(),
            })),
            // Somebody else's code, as this project described it — the
            // studio's inspector needs it for the same reason it needs a
            // component's: to know what a call site may pass.
            Declaration::External(e) => externals.push(json!({
                "name": e.name,
                "kind": match e.kind {
                    crate::parser::ast::ExternalKind::Module => "module",
                    crate::parser::ast::ExternalKind::Element => "element",
                },
                "from": e.from,
                "integrity": e.integrity,
                "doc": e.doc,
                "file": file,
                "line": e.span.line,
                "functions": e.functions.iter().map(|f| json!({
                    "name": f.name,
                    "params": f.params.iter().map(|p| json!({
                        "name": p.name,
                        "type": type_ref_json(&p.prop_type),
                    })).collect::<Vec<_>>(),
                    "returns": f.returns.as_ref().map(type_ref_json),
                    "doc": f.doc,
                })).collect::<Vec<_>>(),
                "types": e.types.iter().map(|t| json!({
                    "name": t.name,
                    "methods": t.methods.iter().map(|m| json!({
                        "name": m.name,
                        "params": m.params.iter().map(|p| json!({
                            "name": p.name,
                            "type": type_ref_json(&p.prop_type),
                        })).collect::<Vec<_>>(),
                        "returns": m.returns.as_ref().map(type_ref_json),
                    })).collect::<Vec<_>>(),
                    "fields": t.fields.iter().map(|f| json!({
                        "name": f.name,
                        "type": type_ref_json(&f.ty),
                    })).collect::<Vec<_>>(),
                })).collect::<Vec<_>>(),
                "props": e.props.iter().map(|p| json!({
                    "name": p.name,
                    "type": type_ref_json(&p.prop_type),
                    "flag": p.prop_type == TypeRef::Bool,
                })).collect::<Vec<_>>(),
                "events": e.events.iter().map(|v| json!(v.name)).collect::<Vec<_>>(),
            })),
            Declaration::Component(c) => {
                let mut slots: Vec<Value> = c
                    .slots
                    .iter()
                    .map(|s| json!(s.name.clone().unwrap_or_else(|| "children".to_string())))
                    .collect();
                if !slots.iter().any(|s| s == "children") && uses_children(&c.body) {
                    slots.push(json!("children"));
                }
                components.push(json!({
                    "name": c.name,
                    "doc": c.doc,
                    "file": file,
                    "line": c.span.line,
                    "positional": c.props.iter().find(|p| p.positional).map(|p| p.name.clone()),
                    "props": c.props.iter().map(|p| json!({
                        "name": p.name,
                        "type": type_ref_json(&p.prop_type),
                        "positional": p.positional,
                        "default": p.default.as_ref().map(crate::sema::types::expr_text),
                        "flag": p.prop_type == TypeRef::Bool,
                        "doc": p.doc,
                    })).collect::<Vec<_>>(),
                    "events": c.events.iter().map(|e| json!({
                        "name": e.name,
                        "params": e.params.iter().map(|p| json!({
                            "name": p.name,
                            "type": type_ref_json(&p.param_type),
                        })).collect::<Vec<_>>(),
                    })).collect::<Vec<_>>(),
                    "slots": slots,
                    "scoped": c.slots.iter().filter(|s| !s.params.is_empty()).map(|s| json!({
                        "name": s.name.clone().unwrap_or_else(|| "children".to_string()),
                        "params": s.params.iter().map(|p| json!({
                            "name": p.name,
                            "type": type_ref_json(&p.param_type),
                        })).collect::<Vec<_>>(),
                    })).collect::<Vec<_>>(),
                    "parts": c.parts,
                }));
            }
            Declaration::Store(s) => stores.push(json!({
                "name": s.name,
                "file": file,
                "line": s.span.line,
                "members": s.body.iter().filter_map(|stmt| match &stmt.kind {
                    StatementKind::State(st) => Some(json!({ "name": st.name, "kind": "state", "type": st.ty.as_ref().map(type_ref_json) })),
                    StatementKind::Derived(d) => Some(json!({ "name": d.name, "kind": "derived" })),
                    StatementKind::Action(a) => Some(json!({
                        "name": a.name,
                        "kind": "action",
                        "params": a.params.iter().map(|p| json!({ "name": p.name, "type": type_ref_json(&p.param_type) })).collect::<Vec<_>>(),
                    })),
                    _ => None,
                }).collect::<Vec<_>>(),
            })),
            Declaration::Page(p) => pages.push(json!({
                "name": p.name,
                "path": p.path,
                "title": p.title,
                "file": file,
                "line": p.span.line,
                "params": p.params.iter().map(|q| json!({ "name": q.name, "type": type_ref_json(&q.prop_type) })).collect::<Vec<_>>(),
                "layout": p.layout.as_ref().map(|l| l.name.clone()),
            })),
            _ => {}
        }
    }
    json!({
        "version": env!("CARGO_PKG_VERSION"),
        "enums": enums,
        "types": types,
        "components": components,
        "stores": stores,
        "pages": pages,
        "externals": externals,
    })
}

fn type_ref_json(ty: &TypeRef) -> String {
    match ty {
        TypeRef::String => "String".into(),
        TypeRef::Number => "Number".into(),
        TypeRef::Bool => "Bool".into(),
        TypeRef::Map => "Map".into(),
        TypeRef::Any => "Any".into(),
        TypeRef::List(inner) => format!("[{}]", type_ref_json(inner)),
        TypeRef::Optional(inner) => format!("{}?", type_ref_json(inner)),
        TypeRef::Named(name) => name.clone(),
        // A condition is the editor's to show as a hint, not part of the
        // type's name.
        TypeRef::Refined(inner, _) => type_ref_json(inner),
    }
}

/// Whether a body places `children` anywhere — a component with a default
/// slot it never declared, as the original grammar wrote them.
fn uses_children(body: &[Statement]) -> bool {
    body.iter().any(|s| match &s.kind {
        StatementKind::UIElement(el) => {
            matches!(&el.component, ComponentRef::BuiltIn(n) if n == "Children")
                || uses_children(&el.children)
                || el.slot_fills.iter().any(|f| uses_children(&f.body))
        }
        StatementKind::If(i) => {
            uses_children(&i.then_body)
                || i.else_if_branches.iter().any(|(_, b)| uses_children(b))
                || i.else_body.as_deref().is_some_and(uses_children)
        }
        StatementKind::For(f) => uses_children(&f.body),
        StatementKind::Show(s) => uses_children(&s.body),
        StatementKind::Match(m) => m.arms.iter().any(|a| uses_children(&a.body)),
        _ => false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_registry_describes_a_button_for_a_tool() {
        let v = registry_json();
        let button = v["components"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["name"] == "Button")
            .unwrap();
        assert_eq!(button["positional"]["name"], "label");
        let tone = button["props"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["name"] == "tone")
            .unwrap();
        assert_eq!(tone["type"], "Enum");
        assert!(
            tone["cases"]
                .as_array()
                .unwrap()
                .iter()
                .any(|c| c["name"] == "primary")
        );
        let flags: Vec<&str> = button["flags"]
            .as_array()
            .unwrap()
            .iter()
            .map(|f| f["name"].as_str().unwrap())
            .collect();
        assert!(flags.contains(&"primary") && flags.contains(&"lg") && flags.contains(&"outlined"));
        let cell = v["components"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["qualified"] == "Table.Cell")
            .unwrap();
        assert_eq!(cell["owner"], "Table");
        assert!(
            v["universal"]["props"]
                .as_array()
                .unwrap()
                .iter()
                .any(|p| p["name"] == "exit")
        );
    }

    #[test]
    fn a_projects_declarations_are_described_the_same_way() {
        let src = "enum Tone { calm, loud }\ntype Todo { id: String, done: Bool = false }\n/// A row.\ncomponent Row2(_ label: String, tone: Tone = .calm, wide: Bool = false) { event pick(id: String)\n slot trailing\n Text(label) children }\nstore S { state n: Number = 0 action bump(by: Number) { n = n + by } }\npage P(path: \"/p/:id\", id: String, layout: Row2(\"x\")) { Text(\"x\") }";
        let program = crate::syntax::parse_source(src, "t.wf").unwrap();
        let v = declarations_json(&program, &|_| "t.wf".to_string());
        assert_eq!(v["enums"][0]["cases"], json!(["calm", "loud"]));
        assert_eq!(v["types"][0]["fields"][1]["default"], "false");
        let row = &v["components"][0];
        assert_eq!(row["positional"], "label");
        assert_eq!(row["props"][2]["flag"], true);
        assert_eq!(row["events"][0]["params"][0]["type"], "String");
        assert_eq!(row["slots"], json!(["trailing", "children"]));
        assert_eq!(v["stores"][0]["members"][1]["params"][0]["name"], "by");
        assert_eq!(v["pages"][0]["params"][0]["name"], "id");
        assert_eq!(v["pages"][0]["layout"], "Row2");
    }
}
