//! The registry: what every built-in component is, in one place.
//!
//! Sixty-eight built-ins used to be described in twenty-four places — the
//! lexer's keyword table, the parser's component list, `builtin_to_html`,
//! the special emitters, the CSS section names, the pruning table, the
//! language server's hand-written reference, the editor grammars, the
//! documentation — and six of those carried their own copy of the modifier
//! vocabulary. This module is the one description the others derive from:
//! a component's name, its one positional argument, the props it takes with
//! their types, the enum cases those props accept, the events it emits, the
//! parts it is made of (`Card.Header`), and the HTML attribute families that
//! reach its root.
//!
//! Every enum case and every boolean prop also records the *legacy word* it
//! was spelled with in the original grammar (`variant: .primary` was
//! `primary`; `size: .lg` was `large` on a Button and `lg` on a Spacer), so
//! the new grammar lowers onto the code generators unchanged, and the
//! migration knows the reverse map.
//!
//! The table lives in [`builtins`]; this file is its vocabulary and the
//! lookups over it.

pub mod builtins;
pub mod json;

pub use builtins::{COMPONENTS, UNIVERSAL_EVENTS, UNIVERSAL_PROPS};

/// How the compiler refers to a component internally: the `ComponentRef`
/// the parsers and code generators agree on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ir {
    /// `ComponentRef::BuiltIn(name)`.
    BuiltIn(&'static str),
    /// `ComponentRef::SubComponent(owner, name)`.
    Sub(&'static str, &'static str),
}

/// What kind of value a prop takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropType {
    Str,
    Num,
    Bool,
    /// A state variable the control reads and writes (`bind:`), or reads
    /// to open and close (`visible:`).
    State,
    /// A route path.
    Path,
    /// A declared page or component, by name.
    Decl,
    /// Anything: an expression the component forwards.
    Any,
    /// One of a fixed set of cases, written `.case`.
    Enum(&'static [CaseSig]),
}

/// Where a prop's value ends up.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sink {
    /// An HTML attribute on the root element (the generic path).
    Attr,
    /// A class on the root element (variants, sizes, shapes).
    Class,
    /// Read by the component's own emitter (`bind`, `to`, `label`, …).
    Special,
}

/// How a boolean prop was spelled in the original grammar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Legacy {
    /// A bare modifier word that became a class or a tag: `elevated`,
    /// `ordered`, `header`.
    Modifier(&'static str),
    /// A named argument or a bare word that became an attribute:
    /// `disabled`, `required`, `multiple`.
    Attr,
}

/// One case of an enum-typed prop.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CaseSig {
    pub name: &'static str,
    /// The modifier word this case was in the original grammar — the word
    /// the code generators still read — or `""` for a default that has no
    /// class of its own.
    pub legacy: &'static str,
    pub summary: &'static str,
}

/// A named prop of a component.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PropSig {
    pub name: &'static str,
    pub ty: PropType,
    pub summary: &'static str,
    pub sink: Sink,
    /// For a `Bool` prop, how it was spelled before.
    pub legacy: Legacy,
    /// Whether a bare `.word` may set this prop (`.primary`, `.lg`). Off
    /// for a prop whose cases another prop shares, such as `exit`, which is
    /// written `exit: .fadeOut`.
    pub shorthand: bool,
}

/// Which HTML attributes reach a component's root beyond its props.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttrFamily {
    /// `id`, `role`, `tabindex`, `title`, `hidden`, `lang`, `dir`, `class`.
    Global,
    /// `aria-*`.
    Aria,
    /// `data-*`.
    Data,
    /// `name`, `autocomplete`, `autofocus`, `readonly`, `pattern`,
    /// `maxlength`, `minlength`, `inputmode`, `spellcheck`.
    Input,
    /// `href`, `target`, `rel`, `download`.
    Anchor,
    /// `poster`, `preload`, `muted`, `loop`, `playsinline`.
    Media,
    /// `action`, `method`, `enctype`, `novalidate`.
    Form,
    /// `colspan`, `rowspan`, `scope`.
    Table,
}

impl AttrFamily {
    /// Whether `name` belongs to this family.
    pub fn accepts(self, name: &str) -> bool {
        match self {
            AttrFamily::Global => matches!(
                name,
                "id" | "role" | "tabindex" | "title" | "hidden" | "lang" | "dir" | "class"
            ),
            AttrFamily::Aria => name.starts_with("aria-"),
            AttrFamily::Data => name.starts_with("data-"),
            AttrFamily::Input => matches!(
                name,
                "name"
                    | "autocomplete"
                    | "autofocus"
                    | "readonly"
                    | "pattern"
                    | "maxlength"
                    | "minlength"
                    | "inputmode"
                    | "spellcheck"
            ),
            AttrFamily::Anchor => matches!(name, "href" | "target" | "rel" | "download"),
            AttrFamily::Media => matches!(
                name,
                "poster" | "preload" | "muted" | "loop" | "playsinline"
            ),
            AttrFamily::Form => matches!(name, "action" | "method" | "enctype" | "novalidate"),
            AttrFamily::Table => matches!(name, "colspan" | "rowspan" | "scope"),
        }
    }
}

/// What a component's block may hold.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Children {
    /// No block.
    None,
    /// Child elements (the default slot).
    Elements,
}

/// A built-in component, or a part of one (`owner` set).
#[derive(Debug, Clone, Copy)]
pub struct ComponentSig {
    pub name: &'static str,
    /// The owner of a part: `Some("Card")` for `Card.Header`.
    pub owner: Option<&'static str>,
    pub group: &'static str,
    pub summary: &'static str,
    pub ir: Ir,
    /// The one positional prop, if the component has one.
    pub positional: Option<PropSig>,
    pub props: &'static [PropSig],
    /// Events beyond the universal DOM events.
    pub events: &'static [&'static str],
    /// The names of its parts, resolved against the table with this
    /// component as owner.
    pub parts: &'static [&'static str],
    pub attrs: &'static [AttrFamily],
    pub children: Children,
}

/// What a bare `.word` on an element means.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Flag {
    /// A boolean prop set to true.
    Bool(&'static PropSig),
    /// A case of an enum-typed prop.
    Case(&'static PropSig, &'static CaseSig),
    /// The word is a case of more than one enum prop.
    Ambiguous(Vec<&'static str>),
    Unknown,
}

impl ComponentSig {
    /// Every prop this component takes: the positional one (which may also
    /// be passed by name), its own, then the universal ones.
    pub fn all_props(&'static self) -> impl Iterator<Item = &'static PropSig> {
        self.positional
            .as_ref()
            .into_iter()
            .chain(self.props.iter())
            .chain(UNIVERSAL_PROPS.iter())
    }

    /// The prop called `name`, positional included.
    pub fn prop(&'static self, name: &str) -> Option<&'static PropSig> {
        self.all_props().find(|p| p.name == name)
    }

    /// Whether a named argument called `name` is accepted: a prop, or an
    /// attribute of one of the component's families.
    pub fn accepts_named(&'static self, name: &str) -> bool {
        self.prop(name).is_some() || self.attrs.iter().any(|f| f.accepts(name))
    }

    /// Resolve a bare `.word`: the boolean prop of that name, or the one
    /// enum prop that has a case of that name.
    pub fn flag(&'static self, word: &str) -> Flag {
        if let Some(p) = self
            .all_props()
            .find(|p| p.shorthand && p.name == word && p.ty == PropType::Bool)
        {
            return Flag::Bool(p);
        }
        let mut hits: Vec<(&'static PropSig, &'static CaseSig)> = Vec::new();
        for p in self.all_props().filter(|p| p.shorthand) {
            if let PropType::Enum(cases) = p.ty
                && let Some(c) = cases.iter().find(|c| c.name == word)
            {
                hits.push((p, c));
            }
        }
        match hits.len() {
            0 => Flag::Unknown,
            1 => Flag::Case(hits[0].0, hits[0].1),
            _ => Flag::Ambiguous(hits.into_iter().map(|(p, _)| p.name).collect()),
        }
    }

    /// The legacy modifier word a `.word` lowers to, when it is one.
    pub fn legacy_word(&'static self, word: &str) -> Option<&'static str> {
        match self.flag(word) {
            Flag::Bool(p) => match p.legacy {
                Legacy::Modifier(w) => Some(w),
                Legacy::Attr => None,
            },
            Flag::Case(_, c) if !c.legacy.is_empty() => Some(c.legacy),
            _ => None,
        }
    }

    /// The spelling a legacy modifier word has in the new grammar, as the
    /// migration writes it: `Some(".lg")` for `large` on a Button.
    pub fn spelling_of_legacy(&'static self, word: &str) -> Option<String> {
        for p in self.all_props() {
            match p.ty {
                PropType::Bool
                    if matches!(p.legacy, Legacy::Modifier(w) if w == word)
                        || (p.legacy == Legacy::Attr && p.name == word) =>
                {
                    return Some(format!(".{}", p.name));
                }
                PropType::Enum(cases) => {
                    if let Some(c) = cases.iter().find(|c| c.legacy == word && !word.is_empty()) {
                        return Some(match self.flag(c.name) {
                            Flag::Case(..) => format!(".{}", c.name),
                            _ => format!("{}: .{}", p.name, c.name),
                        });
                    }
                }
                _ => {}
            }
        }
        None
    }

    /// The qualified name: `Card.Header` for a part, `Card` otherwise.
    pub fn qualified(&self) -> String {
        match self.owner {
            Some(owner) => format!("{owner}.{}", self.name),
            None => self.name.to_string(),
        }
    }
}

/// Modifier words of the original grammar that select nothing any more:
/// `medium`/`md` were the defaults, and the rest named classes no
/// stylesheet defined, or nothing at all. A migration drops them.
pub const RETIRED_MODIFIERS: &[&str] = &[
    "medium",
    "md",
    "error",
    "loading",
    "square",
    "fit",
    "bordered",
    "dismissible",
];

/// The top-level component called `name`.
pub fn component(name: &str) -> Option<&'static ComponentSig> {
    COMPONENTS
        .iter()
        .find(|c| c.owner.is_none() && c.name == name)
}

/// The part `name` of `owner`: `part("Card", "Header")`.
pub fn part(owner: &str, name: &str) -> Option<&'static ComponentSig> {
    COMPONENTS
        .iter()
        .find(|c| c.owner == Some(owner) && c.name == name)
}

/// The component the compiler knows as `ir` — a built-in name, or an
/// `owner.name` pair.
pub fn by_ir(ir: Ir) -> Option<&'static ComponentSig> {
    COMPONENTS.iter().find(|c| c.ir == ir)
}

/// Every top-level component, in table order.
pub fn components() -> impl Iterator<Item = &'static ComponentSig> {
    COMPONENTS.iter().filter(|c| c.owner.is_none())
}

/// Every part of `owner`, in table order.
pub fn parts_of(owner: &str) -> impl Iterator<Item = &'static ComponentSig> {
    COMPONENTS.iter().filter(move |c| c.owner == Some(owner))
}

/// The icons the runtime draws, by name: an `Icon("home")` or an `icon:`
/// that names another shows the name as text.
pub const ICONS: &[&str] = &[
    "close",
    "menu",
    "search",
    "home",
    "user",
    "settings",
    "check",
    "plus",
    "minus",
    "edit",
    "trash",
    "star",
    "heart",
    "mail",
    "bell",
    "download",
    "upload",
    "eye",
    "link",
    "calendar",
    "filter",
    "chevron-down",
    "chevron-right",
    "chevron-left",
    "info",
    "warning",
    "arrow-left",
    "arrow-right",
    "logout",
    "copy",
    "sun",
    "moon",
];

/// Whether `name` is a universal DOM event every element accepts.
pub fn is_dom_event(name: &str) -> bool {
    UNIVERSAL_EVENTS.iter().any(|(e, _)| *e == name)
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_icon_list_is_the_runtimes() {
        let runtime = include_str!("../runtime/modules/icons.js");
        let start = runtime.find("const _ICONS = {").expect("the icon table");
        let end = runtime[start..].find("\n  };").expect("its end") + start;
        let mut drawn: Vec<&str> = runtime[start..end]
            .lines()
            .skip(1)
            .filter_map(|l| l.trim().split(':').next())
            .map(|k| k.trim_matches('"'))
            .collect();
        drawn.sort_unstable();
        let mut listed: Vec<&str> = super::ICONS.to_vec();
        listed.sort_unstable();
        assert_eq!(listed, drawn);
    }

    use super::*;
    use crate::codegen::builtin::builtin_to_html;
    use crate::parser::vocabulary::MODIFIER_KEYWORDS;

    #[test]
    fn every_component_the_parser_accepts_is_described_and_vice_versa() {
        for name in crate::lexer::token::ALL_COMPONENT_NAMES {
            assert!(
                component(name).is_some(),
                "the parser accepts `{name}` but the registry does not describe it"
            );
        }
        for sig in components() {
            let Ir::BuiltIn(ir) = sig.ir else {
                panic!("a top-level component is a BuiltIn: {}", sig.name)
            };
            assert!(
                crate::lexer::token::ALL_COMPONENT_NAMES.contains(&ir),
                "the registry describes `{ir}`, which the parser does not accept"
            );
            // The HTML mapping knows every component that renders on the web.
            let (tag, _) = builtin_to_html(ir);
            assert!(!tag.is_empty(), "{ir} has no tag");
        }
    }

    #[test]
    fn parts_resolve_to_the_ir_the_code_generators_know() {
        let item = part("Sidebar", "Item").expect("Sidebar.Item");
        assert_eq!(item.ir, Ir::Sub("Sidebar", "Item"));
        let cell = part("Table", "Cell").expect("Table.Cell");
        assert_eq!(cell.ir, Ir::BuiltIn("Tcell"));
        for sig in COMPONENTS.iter().filter(|c| c.owner.is_some()) {
            let owner = component(sig.owner.unwrap())
                .unwrap_or_else(|| panic!("{} has no owner entry", sig.qualified()));
            assert!(
                owner.parts.contains(&sig.name),
                "{} is not listed among {}'s parts",
                sig.qualified(),
                owner.name
            );
        }
        for sig in components() {
            for p in sig.parts {
                assert!(part(sig.name, p).is_some(), "{}.{p} has no entry", sig.name);
            }
        }
    }

    #[test]
    fn every_modifier_word_has_a_spelling_somewhere() {
        // A word may be retired (it never selected anything a program could
        // see) but must be retired on purpose.
        for word in MODIFIER_KEYWORDS {
            if RETIRED_MODIFIERS.contains(word) {
                continue;
            }
            let spelled = COMPONENTS
                .iter()
                .any(|c| c.spelling_of_legacy(word).is_some());
            assert!(
                spelled,
                "the modifier `{word}` has no place in the registry"
            );
        }
    }

    #[test]
    fn flags_resolve_uniquely_or_say_why_not() {
        let button = component("Button").unwrap();
        assert!(
            matches!(button.flag("primary"), Flag::Case(p, c) if p.name == "tone" && c.legacy == "primary")
        );
        assert!(
            matches!(button.flag("lg"), Flag::Case(p, c) if p.name == "size" && c.legacy == "large")
        );
        assert!(matches!(button.flag("disabled"), Flag::Bool(p) if p.name == "disabled"));
        assert!(matches!(button.flag("huge"), Flag::Unknown));
        let row = component("Row").unwrap();
        assert!(
            matches!(row.flag("center"), Flag::Ambiguous(props) if props == vec!["align", "justify"])
        );
        // The universal props reach every element.
        assert!(
            matches!(component("Card").unwrap().flag("fadeIn"), Flag::Case(p, _) if p.name == "animate")
        );
        assert_eq!(button.legacy_word("lg"), Some("large"));
        assert_eq!(component("Spacer").unwrap().legacy_word("lg"), Some("lg"));
        assert_eq!(button.spelling_of_legacy("large"), Some(".lg".to_string()));
        assert_eq!(
            row.spelling_of_legacy("center"),
            None,
            "not a modifier on Row"
        );
    }

    #[test]
    fn named_arguments_are_accepted_by_prop_or_by_family() {
        let input = component("Input").unwrap();
        assert!(input.accepts_named("placeholder"));
        assert!(input.accepts_named("aria-label"));
        assert!(input.accepts_named("autocomplete"));
        assert!(input.accepts_named("class"));
        assert!(!input.accepts_named("lable"));
        assert!(!component("Text").unwrap().accepts_named("href"));
        assert!(component("Link").unwrap().accepts_named("target"));
    }

    #[test]
    fn every_case_of_an_enum_has_a_distinct_name_and_a_known_legacy_word() {
        for sig in COMPONENTS.iter() {
            for p in sig.all_props() {
                if let PropType::Enum(cases) = p.ty {
                    let mut names: Vec<&str> = cases.iter().map(|c| c.name).collect();
                    names.sort();
                    names.dedup();
                    assert_eq!(
                        names.len(),
                        cases.len(),
                        "{}.{} repeats a case",
                        sig.qualified(),
                        p.name
                    );
                    for c in cases {
                        assert!(
                            c.legacy.is_empty()
                                || MODIFIER_KEYWORDS.contains(&c.legacy)
                                || p.sink != Sink::Class,
                            "{}.{}: legacy word `{}` is not a modifier",
                            sig.qualified(),
                            p.name,
                            c.legacy
                        );
                    }
                }
            }
        }
    }
}
