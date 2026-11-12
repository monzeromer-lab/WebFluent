//! The type checker: what every name is, inferred from what it is declared
//! as or given, and the mistakes that follow from mixing them up.
//!
//! Types come from declarations — `state x: T`, props, params, `type`
//! fields, `enum` cases, route parameters, `resource x: [T]` — and from
//! values: literals, records, lists, the built-in methods of strings,
//! numbers and lists, the members of stores. Anything that cannot be
//! resolved is `Any`, and `Any` agrees with everything, so a program that
//! declares no types checks as it always did; every annotation narrows
//! what the checker can say.
//!
//! The checks report what is certainly wrong, each with a hint: a value
//! of the wrong type given to a prop, a state, a field or a parameter; a
//! case an enum does not have; a field a record does not have; a member a
//! store does not have; a value that may be `null` read as if it were
//! not; a call with the wrong number of arguments; an `emit` that does
//! not match its event; a `bind:` on a control of the wrong type; a `for`
//! over something that is not a list; a `match` on something with no
//! arms to match; a list used as a condition.

use std::collections::{HashMap, HashSet};

use super::Findings;
use crate::diagnostics::error::Diagnostic;
use crate::registry::{self, PropType};
use crate::syntax::parser::ast::*;

mod call;
mod element;
mod expr;
mod handles;
mod jsdoc;
mod members;
mod method;
mod names;
mod reactivity;
mod refine;
mod report;
mod stmt;
mod tables;
#[cfg(test)]
mod tests;
mod ty;
mod walk;

pub use handles::{browser_value_type, form_names, form_type, image_type, ref_names};
pub use jsdoc::jsdoc_type;
pub use members::{Member, members};
pub(crate) use names::BUILT_IN_FUNCTIONS;
use names::*;
pub use names::{built_in_functions, language_name};
use refine::*;
pub use refine::{VALIDATE_RULES, refinement_fault};
use tables::*;
use ty::*;
pub use ty::{PRIMITIVE_TYPES, Scalar, Type};
pub use walk::expr_text;
use walk::*;

/// What the program declares, resolved once.
struct World<'p> {
    types: HashMap<&'p str, &'p TypeDecl>,
    enums: HashMap<&'p str, &'p EnumDecl>,
    components: HashMap<&'p str, &'p ComponentDecl>,
    /// Each store's members, typed.
    stores: HashMap<&'p str, HashMap<String, Type>>,
    /// The program's constants, typed.
    consts: HashMap<String, Type>,
    /// The services the program declares.
    apis: HashMap<&'p str, &'p ApiDecl>,
    /// What the project's scripts make global, by name, with the file each
    /// is in.
    scripts: HashMap<&'p str, (&'p str, &'p crate::syntax::js_scan::Name)>,
    /// The top-level names nothing may assign to, and what each is: a
    /// `const`, a `data` constant, an `image`.
    fixed: HashMap<String, &'static str>,
    /// Each store's members nothing may assign to: its derived values and
    /// its actions.
    store_fixed: HashMap<&'p str, HashMap<String, &'static str>>,
    /// Each store's actions, by name, for what they write.
    store_actions: HashMap<&'p str, HashMap<String, &'p [Statement]>>,
    /// Each store's state the program writes, from anywhere, as
    /// `Store.member`.
    store_written: HashSet<String>,
}

impl<'p> World<'p> {
    /// One endpoint of a service, by name.
    fn endpoint(&self, api: &str, name: &str) -> Option<&'p Endpoint> {
        self.apis
            .get(api)?
            .endpoints
            .iter()
            .find(|e| e.name == name)
    }

    fn record_field(&self, record: &str, field: &str) -> Option<Type> {
        self.record_fields(record)?
            .into_iter()
            .find(|f| f.name == field)
            .map(|f| Type::from_ref(&f.ty))
    }

    /// Every field of a record, the extended record's included.
    fn record_fields(&self, record: &str) -> Option<Vec<&'p FieldDecl>> {
        let decl = self.types.get(record)?;
        Some(decl.all_fields(&|name| self.types.get(name).copied()))
    }

    /// A `TypeRef::Named` may name an enum rather than a record.
    fn resolve(&self, ty: Type) -> Type {
        match ty {
            Type::Record(name) if self.enums.contains_key(name.as_str()) => Type::Enum(name),
            // A name the language knows, unless the program declares a type
            // of its own by that name — which wins, so nothing the language
            // adds can take a name away.
            Type::Record(name)
                if !self.types.contains_key(name.as_str()) && Scalar::of_name(&name).is_some() =>
            {
                Type::Scalar(Scalar::of_name(&name).expect("just checked"))
            }
            Type::List(inner) => Type::list(self.resolve(*inner)),
            Type::Optional(inner) => Type::optional(self.resolve(*inner)),
            other => other,
        }
    }
}

/// A name's type, recorded where it is declared, for the editor.
#[derive(Debug, Clone)]
pub struct Typed {
    /// The index of the declaration it belongs to.
    pub decl: usize,
    pub name: String,
    /// The span of the statement, prop or parameter that declares it.
    pub span: Span,
    pub ty: Type,
}

/// The types of every name declared in the program, in the order they
/// were met, beside the findings.
#[derive(Debug, Default)]
pub struct TypeInfo {
    pub findings: Findings,
    pub bindings: Vec<Typed>,
    /// Names read or called that nothing declares — not the program, not
    /// the language, not the browser. The build refuses them (`T13`); a
    /// template rendered with data reads its data's keys by name, so the
    /// template engine does not.
    pub unresolved: Vec<Diagnostic>,
}

impl TypeInfo {
    /// The type of the name declared by the binding at `span` in `decl`.
    pub fn type_at(&self, decl: usize, name: &str, span: Span) -> Option<&Type> {
        self.bindings
            .iter()
            .find(|t| t.decl == decl && t.name == name && t.span.start == span.start)
            .map(|t| &t.ty)
    }

    /// The type of a store's member.
    pub fn member(&self, store_decl: usize, name: &str) -> Option<&Type> {
        self.bindings
            .iter()
            .find(|t| t.decl == store_decl && t.name == name)
            .map(|t| &t.ty)
    }
}

/// Check every declaration of the program.
pub fn check(program: &Program, file_of: &dyn Fn(usize) -> String) -> TypeInfo {
    check_in(program, file_of, &|_| None)
}

/// [`check`], with the source text of each declaration's file at hand:
/// an error is then placed at the expression it names inside its
/// statement, not at the statement's start.
pub fn check_in(
    program: &Program,
    file_of: &dyn Fn(usize) -> String,
    source_of: &dyn Fn(usize) -> Option<String>,
) -> TypeInfo {
    let texts: Vec<Option<String>> = (0..program.declarations.len()).map(source_of).collect();
    check_sources(program, file_of, &|i| {
        texts.get(i).and_then(|t| t.as_deref())
    })
}

/// [`check_in`], borrowing each declaration's source text rather than
/// taking a copy of the file for every declaration in it.
pub fn check_sources<'s>(
    program: &Program,
    file_of: &dyn Fn(usize) -> String,
    source_of: &dyn Fn(usize) -> Option<&'s str>,
) -> TypeInfo {
    let mut world = World {
        types: HashMap::new(),
        enums: HashMap::new(),
        components: HashMap::new(),
        stores: HashMap::new(),
        consts: HashMap::new(),
        apis: HashMap::new(),
        scripts: HashMap::new(),
        fixed: HashMap::new(),
        store_fixed: HashMap::new(),
        store_actions: HashMap::new(),
        store_written: HashSet::new(),
    };
    for decl in &program.declarations {
        match decl {
            Declaration::Script(script) => {
                for n in &script.names {
                    world
                        .scripts
                        .entry(n.name.as_str())
                        .or_insert((script.path.as_str(), n));
                }
            }
            Declaration::Type(t) => {
                world.types.insert(t.name.as_str(), t);
            }
            Declaration::Enum(e) => {
                world.enums.insert(e.name.as_str(), e);
            }
            Declaration::Api(a) => {
                world.apis.insert(a.name.as_str(), a);
            }
            Declaration::Component(c) => {
                world.components.insert(c.name.as_str(), c);
            }
            Declaration::Store(store) => {
                let mut fixed = HashMap::new();
                for stmt in &store.body {
                    match &stmt.kind {
                        StatementKind::Derived(d) => {
                            fixed.insert(d.name.clone(), "a `derived` value");
                        }
                        StatementKind::Action(a) => {
                            fixed.insert(a.name.clone(), "an action");
                        }
                        StatementKind::Resource(r) => {
                            fixed.insert(r.name.clone(), "a `resource`");
                        }
                        _ => {}
                    }
                }
                world.store_fixed.insert(store.name.as_str(), fixed);
                let actions: HashMap<String, &[Statement]> = store
                    .body
                    .iter()
                    .filter_map(|s| match &s.kind {
                        StatementKind::Action(a) => Some((a.name.clone(), a.body.as_slice())),
                        _ => None,
                    })
                    .collect();
                world.store_actions.insert(store.name.as_str(), actions);
            }
            Declaration::Const(c) => {
                world.fixed.insert(c.name.clone(), "a `const`");
            }
            Declaration::Data(d) => {
                world.fixed.insert(
                    d.name.clone(),
                    if d.is_image {
                        "an `image`"
                    } else {
                        "a `data` constant"
                    },
                );
            }
            _ => {}
        }
    }
    // What the program writes of each store's state: its own code by the
    // bare name, everything else as `Store.member`.
    for decl in &program.declarations {
        let (store, bodies): (Option<&str>, Vec<&[Statement]>) = match decl {
            Declaration::Store(s) => (Some(s.name.as_str()), vec![&s.body]),
            Declaration::Page(p) => (None, vec![&p.body]),
            Declaration::Component(c) => (None, vec![&c.body]),
            Declaration::App(a) => (None, vec![&a.body]),
            Declaration::Api(a) => (None, a.hooks.iter().map(|h| h.body.as_slice()).collect()),
            _ => continue,
        };
        for body in bodies {
            for name in written_names(body) {
                if name.contains('.') {
                    world.store_written.insert(name);
                } else if let Some(store) = store {
                    world.store_written.insert(format!("{store}.{name}"));
                }
            }
        }
    }
    let mut info = TypeInfo::default();

    // Constants first: plain values, read everywhere. A `data` declaration
    // the build has not resolved yet is what it declares, or `Any`.
    for (index, decl) in program.declarations.iter().enumerate() {
        if let Declaration::Data(d) = decl {
            // An `image` is the asset the build made of it; `data` what it
            // declares, or `Any`.
            let ty = if d.is_image {
                image_type()
            } else {
                d.ty.as_ref()
                    .map_or(Type::Any, |t| world.resolve(Type::from_ref(t)))
            };
            info.bindings.push(Typed {
                decl: index,
                name: d.name.clone(),
                span: d.span,
                ty: ty.clone(),
            });
            world.consts.insert(d.name.clone(), ty);
        }
        if let Declaration::Const(c) = decl {
            let file = file_of(index);
            let source = source_of(index);
            let mut cx = Checker::new(&world, &file, index, &mut info, source);
            cx.current_span = c.span;
            let declared = c.ty.as_ref().map(|t| cx.world.resolve(Type::from_ref(t)));
            let given = cx.infer(&c.value, declared.as_ref());
            if let Some(declared) = &declared {
                let what = format!("`const {}`", c.name);
                cx.expect(&given, declared, c.span, &what);
            }
            let ty = declared.unwrap_or(given);
            cx.info.bindings.push(Typed {
                decl: index,
                name: c.name.clone(),
                span: c.span,
                ty: ty.clone(),
            });
            world.consts.insert(c.name.clone(), ty);
        }
    }

    // Stores next: their members are read everywhere else. A store's
    // derived values and actions may read other stores, which are `Any`
    // until met.
    for (index, decl) in program.declarations.iter().enumerate() {
        if let Declaration::Store(s) = decl {
            let file = file_of(index);
            let source = source_of(index);
            let mut cx = Checker::new(&world, &file, index, &mut info, source);
            cx.written = world
                .store_written
                .iter()
                .filter_map(|w| w.strip_prefix(s.name.as_str())?.strip_prefix('.'))
                .map(str::to_string)
                .collect();
            cx.push_scope();
            cx.declare_hoisted(&s.body);
            let members = cx.scopes.last().cloned().unwrap_or_default();
            cx.statements(&s.body, Body::Store);
            cx.pop_scope();
            world.stores.insert(s.name.as_str(), members);
        }
    }

    for (index, decl) in program.declarations.iter().enumerate() {
        let file = file_of(index);
        let source = source_of(index);
        let mut cx = Checker::new(&world, &file, index, &mut info, source);
        match decl {
            Declaration::Page(p) => {
                cx.push_scope();
                for param in &p.params {
                    let ty = cx.world.resolve(Type::from_ref(&param.prop_type));
                    cx.bind_fixed(&param.name, ty, param.span, "a route parameter");
                }
                if let Some(layout) = &p.layout {
                    cx.current_span = layout.span;
                    cx.layout(layout);
                }
                cx.bound = bound_names(&p.body);
                cx.written = written_names(&p.body);
                cx.declare_hoisted(&p.body);
                cx.statements(&p.body, Body::Page);
                cx.pop_scope();
            }
            Declaration::Component(c) => {
                cx.push_scope();
                for prop in &c.props {
                    let mut ty = cx.world.resolve(Type::from_ref(&prop.prop_type));
                    if prop.optional {
                        ty = Type::optional(ty);
                    }
                    if let Some(default) = &prop.default {
                        cx.current_span = prop.span;
                        let given = cx.infer(default, Some(&ty));
                        cx.expect(
                            &given,
                            &ty,
                            prop.span,
                            &format!("the default of `{}`", prop.name),
                        );
                    }
                    cx.bind_fixed(&prop.name, ty, prop.span, "a prop");
                }
                cx.component = Some(c);
                cx.bound = bound_names(&c.body);
                cx.written = written_names(&c.body);
                cx.declare_hoisted(&c.body);
                cx.statements(&c.body, Body::Component);
                cx.pop_scope();
            }
            Declaration::App(a) => {
                cx.push_scope();
                cx.written = written_names(&a.body);
                cx.declare_hoisted(&a.body);
                cx.statements(&a.body, Body::Page);
                cx.pop_scope();
            }
            Declaration::Type(t) => {
                for field in &t.fields {
                    if let Some(default) = &field.default {
                        cx.current_span = field.span;
                        let ty = cx.world.resolve(Type::from_ref(&field.ty));
                        let given = cx.infer(default, Some(&ty));
                        cx.expect(
                            &given,
                            &ty,
                            field.span,
                            &format!("the default of `{}`", field.name),
                        );
                    }
                }
            }
            // A service's own code runs like any other: the settings and
            // headers are expressions, and a hook is a body. The hook's
            // parameter is the request, the reply or the error, whose shape
            // the runtime gives it, so it is `Any`.
            Declaration::Api(a) => {
                for (_, e) in a.settings.iter().chain(a.headers.iter()) {
                    cx.current_span = a.span;
                    cx.infer(e, None);
                }
                for endpoint in &a.endpoints {
                    for (_, e) in &endpoint.settings {
                        cx.current_span = endpoint.span;
                        cx.infer(e, None);
                    }
                    // Every `:name` in the path is filled by the parameter of
                    // that name; one that is not stays `:name` in the
                    // address, and the value it was meant to carry goes as a
                    // query instead (`/users/:id?userId=42`).
                    let params: Vec<&str> =
                        endpoint.params.iter().map(|p| p.name.as_str()).collect();
                    for segment in endpoint.path.split('/') {
                        let Some(name) = segment.strip_prefix(':') else {
                            continue;
                        };
                        if !params.contains(&name) {
                            let hint = if params.is_empty() {
                                format!(
                                    "Give it the parameter: `{}({name}: String)`",
                                    endpoint.name
                                )
                            } else {
                                format!(
                                    "Name a parameter `{name}`; it takes {}",
                                    params
                                        .iter()
                                        .map(|p| format!("`{p}`"))
                                        .collect::<Vec<_>>()
                                        .join(", ")
                                )
                            };
                            cx.error(
                                endpoint.span,
                                "T10",
                                format!(
                                    "`{}.{}` is at `{}`, and no parameter fills `:{name}`",
                                    a.name, endpoint.name, endpoint.path
                                ),
                                &hint,
                            );
                        }
                    }
                }
                for hook in &a.hooks {
                    cx.push_scope();
                    cx.handler_names(
                        hook,
                        &[Type::Any],
                        &format!("an `api`'s `on {}` hook receives one value", hook.event),
                    );
                    cx.returns.push(Vec::new());
                    cx.async_ok = true;
                    cx.in_handler += 1;
                    cx.statements(&hook.body, Body::Imperative);
                    cx.in_handler -= 1;
                    cx.async_ok = false;
                    cx.returns.pop();
                    cx.pop_scope();
                }
            }
            Declaration::Store(_)
            | Declaration::Theme(_)
            | Declaration::Enum(_)
            | Declaration::Script(_)
            | Declaration::Const(_)
            | Declaration::Animation(_)
            | Declaration::Test(_)
            | Declaration::Data(_) => {}
        }
    }
    info
}

/// What a `match` arm's binding is, for a write to it.
const ARM: &str = "a value a `match` arm binds";
/// What a DOM event's handler is given: the event, and nothing beside it.
const DOM_SENDS: &str = "a DOM event sends one value, the event";

/// The kind of body being checked, for what a statement may be.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Body {
    Page,
    Component,
    Store,
    /// An action, handler or effect.
    Imperative,
}

struct Checker<'a, 'p> {
    world: &'a World<'p>,
    file: &'a str,
    decl: usize,
    info: &'a mut TypeInfo,
    scopes: Vec<HashMap<String, Type>>,
    /// The names in each scope nothing may assign to, and what each is: a
    /// derived value, a prop, a loop variable. Parallel to `scopes`.
    fixed: Vec<HashMap<String, &'static str>>,
    /// Each name narrowed in a scope, and its type before: what it may be
    /// assigned, whatever a condition proved it holds now. Parallel to
    /// `scopes`.
    widened: Vec<HashMap<String, Type>>,
    /// The component being checked, for `emit`.
    component: Option<&'p ComponentDecl>,
    /// The `return` types met in the action being checked.
    returns: Vec<Vec<Type>>,
    /// The span of the statement or argument being checked: where an
    /// expression's error lands, since expressions carry no span.
    current_span: Span,
    /// The file's text, when the caller has it: where an error is placed
    /// at the expression it names.
    source: Option<&'a str>,
    /// Whether an element's arguments are being checked, where the
    /// resolver already reports a case the prop's enum lacks.
    in_args: bool,
    /// Whether the body being checked compiles to an async function — an
    /// action, a handler, a timer, a hook — where `await` may be written.
    async_ok: bool,
    /// Whether a `derived` value is being checked, for what `await` there
    /// should be instead.
    in_derived: bool,
    /// How many handlers enclose what is being checked: `event` (`e`),
    /// and `value` and `key`, are names only there.
    in_handler: usize,
    /// Inside a `Header` or `Footer`: `page` and `pages` are names there.
    in_running: usize,
    /// The states whose declared type says what their values must be —
    /// `Number(1..=30)` — held to it at every assignment, not only the
    /// first.
    refined: HashMap<String, TypeRef>,
    /// The lists an enclosing condition proves are not empty — `if
    /// items.length > 0 { items[0] }` — as the expressions they are, so
    /// two lists that print alike are never mistaken for one.
    non_empty: Vec<Expr>,
    /// Whether the expression being checked is a later arm of a `match`
    /// expression whose first arm has already been checked as a whole.
    in_chain: bool,
    /// The names some control in the body binds, which a `validate` block
    /// guards.
    bound: HashSet<String>,
    /// The names the body writes — and, for a store, its state anything in
    /// the program writes: a state among them changes on purpose.
    written: HashSet<String>,
    /// The body's own actions, by name, for what they write.
    own_actions: HashMap<String, &'p [Statement]>,
    /// How deeply [`Checker::infer`] is nested in itself — held under the
    /// parser's limit, for a tree that did not come from the parser.
    depth: usize,
    /// Whether `E120` has been reported for this declaration.
    too_deep: bool,
    /// Whether the finding reported last was an error, for the fix that
    /// follows it ([`Checker::plan_last`]).
    last_was_error: bool,
}

impl<'a, 'p> Checker<'a, 'p> {
    fn new(
        world: &'a World<'p>,
        file: &'a str,
        decl: usize,
        info: &'a mut TypeInfo,
        source: Option<&'a str>,
    ) -> Self {
        Self {
            world,
            file,
            decl,
            info,
            scopes: Vec::new(),
            fixed: Vec::new(),
            widened: Vec::new(),
            component: None,
            returns: Vec::new(),
            current_span: Span::dummy(),
            in_args: false,
            async_ok: false,
            in_derived: false,
            in_handler: 0,
            in_running: 0,
            refined: HashMap::new(),
            non_empty: Vec::new(),
            in_chain: false,
            bound: HashSet::new(),
            written: HashSet::new(),
            own_actions: HashMap::new(),
            depth: 0,
            too_deep: false,
            last_was_error: false,
            source,
        }
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
        self.fixed.push(HashMap::new());
        self.widened.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
        self.fixed.pop();
        self.widened.pop();
    }

    /// What `name` may be assigned: its type before any condition narrowed
    /// it. `if x == null { return }` proves `x` holds a value from there on;
    /// `x = null` after it is still a value of its type.
    fn assignable(&self, name: &str) -> Option<Type> {
        self.widened
            .iter()
            .find_map(|w| w.get(name).cloned())
            .or_else(|| self.lookup(name))
    }

    /// Bind a name nothing may assign to, saying what it is.
    fn bind_fixed(&mut self, name: &str, ty: Type, span: Span, what: &'static str) {
        self.bind(name, ty, span);
        if let Some(fixed) = self.fixed.last_mut() {
            fixed.insert(name.to_string(), what);
        }
    }

    /// What `name` is, when it is something nothing may assign to: the
    /// innermost binding of it decides.
    fn fixed_kind(&self, name: &str) -> Option<&'static str> {
        for (i, scope) in self.scopes.iter().enumerate().rev() {
            if scope.contains_key(name) {
                return self.fixed.get(i).and_then(|f| f.get(name).copied());
            }
        }
        self.world.fixed.get(name).copied()
    }

    fn bind(&mut self, name: &str, ty: Type, span: Span) {
        self.refuse_language_name(name, span);
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), ty.clone());
        }
        if let Some(fixed) = self.fixed.last_mut() {
            fixed.remove(name);
        }
        self.info.bindings.push(Typed {
            decl: self.decl,
            name: name.to_string(),
            span,
            ty,
        });
    }

    /// `E118`: a program's own name may not be one the language has. A
    /// `state env` hid the build's settings in its page, and the compiled
    /// page read the one where it wrote the other; a loop's `t` hid the
    /// translation function inside the loop. Each read as the program
    /// meant only by luck, so none is a name of the program's own.
    fn refuse_language_name(&mut self, name: &str, span: Span) {
        let Some(what) = language_name(name) else {
            return;
        };
        // A name bound twice at one place is reported once. What is
        // already reported is asked, not remembered apart: a `derived`
        // value is inferred once quietly, its findings dropped, before it
        // is checked. The place is a file's: one program's findings share
        // this list, and `state query` on line 12 of two stores is two.
        let message = format!("`{name}` is a name the language already has: {what}");
        if self.info.findings.errors.iter().any(|d| {
            d.code == "E118"
                && d.file == self.file
                && d.line == span.line as usize
                && d.column == span.col as usize
                && d.message == message
        }) {
            return;
        }
        let hint = match own_name_for(name) {
            _ if GRAMMAR_WORDS.contains(&name) => format!(
                "Name it something of your own: `{name}` is how a statement is written, and a line that used the name would read as one"
            ),
            Some(own) => format!(
                "Name it something of your own — {own}: inside its scope every `{name}` would mean this, and the language's would have no name"
            ),
            None => format!(
                "Name it something of your own: inside its scope every `{name}` would mean this, and the language's would have no name"
            ),
        };
        self.error(span, "E118", message, &hint);
    }

    /// Narrow a name within the current scope, without recording a binding.
    fn narrow(&mut self, name: &str, ty: Type) {
        // A narrowed name is still what it was: a derived value checked
        // for null is still not assignable.
        let what = self.fixed_kind(name);
        if let Some(before) = self.lookup(name)
            && let Some(widened) = self.widened.last_mut()
        {
            widened.entry(name.to_string()).or_insert(before);
        }
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), ty);
        }
        if let (Some(what), Some(fixed)) = (what, self.fixed.last_mut()) {
            fixed.insert(name.to_string(), what);
        }
    }

    fn lookup(&self, name: &str) -> Option<Type> {
        self.scopes.iter().rev().find_map(|s| s.get(name).cloned())
    }

    /// Report `given` where `wanted` was expected, at `span`, naming `what`.
    fn expect(&mut self, given: &Type, wanted: &Type, span: Span, what: &str) {
        // A bare case fits an enum only when the enum has it: `.quiet`
        // given for a `Tone` that has `.calm` and `.loud` is never equal
        // to anything a `Tone` holds.
        if let (Type::Case(case), Type::Enum(e)) = (given, wanted.unwrapped())
            && let Some(decl) = self.world.enums.get(e.as_str())
            && !decl.case_names().contains(case)
        {
            self.error(
                span,
                "T02",
                format!("`{e}` has no case `.{case}`"),
                &format!(
                    "`{e}` takes {}",
                    decl.case_names()
                        .iter()
                        .map(|c| format!(".{c}"))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            );
            let cases = decl.case_names();
            self.plan_nearest_case(case, &cases);
            return;
        }
        if given.assignable_to(wanted) {
            return;
        }
        if let Type::Promise(inner) = given {
            self.error(
                span,
                "T17",
                format!("{what} is an async action's result, which is a promise of `{inner}` until it is awaited"),
                "`await` it inside an action or a handler: `let value = await load()`",
            );
            return;
        }
        let hint = match (given, wanted) {
            (Type::Optional(_), _) => {
                "Unwrap it first: `if let x = value { … }`, `value ?? fallback`, or a check for `!= null`".to_string()
            }
            (Type::String, Type::Number) => "Convert it: `Number(value)`".to_string(),
            (Type::Number, Type::String) => "Convert it: `String(value)` or `\"{value}\"`".to_string(),
            (_, Type::Enum(e)) => match self.world.enums.get(e.as_str()) {
                Some(decl) => format!(
                    "`{e}` takes {}",
                    decl.case_names()
                        .iter()
                        .map(|c| format!(".{c}"))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                None => String::new(),
            },
            _ => String::new(),
        };
        self.error(
            span,
            "T01",
            format!("{what} is `{given}`, but `{wanted}` is wanted"),
            &hint,
        );
    }

    // ─── Declarations first ──────────────────────────────

    /// Bind the names a body declares before checking it, as the compiler
    /// resolves them regardless of order: actions may call each other, a
    /// derived value may read a later state.
    fn declare_hoisted(&mut self, stmts: &'p [Statement]) {
        // Two passes: state and resources first (their types are declared
        // or read off their values), then derived values and actions, whose
        // types may depend on them.
        for stmt in stmts {
            self.current_span = stmt.span;
            match &stmt.kind {
                StatementKind::State(s) => {
                    if let Some(t) = &s.ty
                        && !t.refinement().is_empty()
                    {
                        self.refined.insert(s.name.clone(), t.clone());
                    }
                    let ty = match &s.ty {
                        Some(t) => self.world.resolve(Type::from_ref(t)),
                        // A state that starts as `null` will hold something
                        // later: a value that may be null.
                        None => match self.infer(&s.value, None) {
                            Type::Null => Type::optional(Type::Any),
                            other => other,
                        },
                    };
                    self.bind(&s.name, ty, stmt.span);
                }
                StatementKind::Resource(r) => {
                    let inner = match &r.ty {
                        Some(t) => self.world.resolve(Type::from_ref(t)),
                        // An endpoint says what it returns, so a resource
                        // over one is typed without an annotation.
                        None => match &r.url {
                            Expr::MethodCall(obj, method, _) => {
                                match (&**obj, self.world.apis.is_empty()) {
                                    (Expr::Identifier(api), false) => self
                                        .world
                                        .endpoint(api, method)
                                        .and_then(|e| e.returns.as_ref())
                                        .map_or(Type::Any, |t| {
                                            self.world.resolve(Type::from_ref(t))
                                        }),
                                    _ => Type::Any,
                                }
                            }
                            _ => Type::Any,
                        },
                    };
                    self.bind_fixed(
                        &r.name,
                        Type::Resource(Box::new(inner)),
                        stmt.span,
                        "a `resource`",
                    );
                }
                StatementKind::Use(u) => {
                    self.bind_fixed(
                        &u.store_name,
                        Type::Store(u.store_name.clone()),
                        stmt.span,
                        "a store",
                    );
                }
                // `socket chat = ws(…)`, `peer link = rtc(…)`: the handle,
                // bound before the body so a handler may name another
                // connection declared after it.
                StatementKind::Connection(c) => {
                    self.bind_fixed(&c.name, Type::Any, stmt.span, "a connection");
                }
                _ => {}
            }
            // `ref: name` anywhere in the body declares `name`: a handle on
            // the element, read like the element itself.
            for name in ref_names(std::slice::from_ref(stmt)) {
                self.bind_fixed(&name, Type::Any, stmt.span, "an element's handle");
            }
            // `Form(bind: form)` declares `form`: its validity and values.
            for name in form_names(std::slice::from_ref(stmt)) {
                self.bind_fixed(&name, form_type(), stmt.span, "a form's handle");
            }
        }
        // Actions before derived values, and all of them before any is
        // inferred: a store's members are mutually visible, and the runtime
        // binds its actions first. Inferring a derived value in source order
        // reported an action declared below it as undeclared.
        for stmt in stmts {
            self.current_span = stmt.span;
            if let StatementKind::Action(a) = &stmt.kind {
                self.own_actions.insert(a.name.clone(), &a.body);
                let params: Vec<Type> = a
                    .params
                    .iter()
                    .map(|p| self.world.resolve(Type::from_ref(&p.param_type)))
                    .collect();
                // The return type is found when the body is checked; an
                // action that awaits hands back a promise of it.
                let ret = if awaits(&a.body) {
                    Type::Promise(Box::new(Type::Any))
                } else {
                    Type::Any
                };
                self.bind_fixed(
                    &a.name,
                    Type::Func(params, Box::new(ret)),
                    stmt.span,
                    "an action",
                );
            }
        }
        for stmt in stmts {
            self.current_span = stmt.span;
            if let StatementKind::Derived(d) = &stmt.kind {
                let declared = d.ty.as_ref().map(|t| self.world.resolve(Type::from_ref(t)));
                let given = self.infer_quiet(&d.value);
                let ty = declared.unwrap_or(given);
                self.bind_fixed(&d.name, ty, stmt.span, "a `derived` value");
            }
        }
    }

    // ─── Statements ──────────────────────────────────────

    // ─── Elements ────────────────────────────────────────

    // ─── Expressions ─────────────────────────────────────

    /// `span`, moved to the first thing the message names in backticks
    /// that the statement's text holds — `sel?.title`, `.angry`, `nam` —
    /// when the source is at hand.
    fn located(&self, span: Span, message: &str) -> Span {
        let Some(source) = self.source else {
            return span;
        };
        let (start, end) = (span.start as usize, span.end as usize);
        if end <= start
            || end > source.len()
            || !source.is_char_boundary(start)
            || !source.is_char_boundary(end)
        {
            return span;
        }
        let text = &source[start..end];
        let mut rest = message;
        while let Some(open) = rest.find('`') {
            let after = &rest[open + 1..];
            let Some(close) = after.find('`') else {
                break;
            };
            let needle = &after[..close];
            rest = &after[close + 1..];
            if needle.is_empty() {
                continue;
            }
            if let Some(at) = text.find(needle) {
                let before = &text[..at];
                let line = span.line as usize + before.matches('\n').count();
                let col = match before.rfind('\n') {
                    Some(nl) => before[nl + 1..].chars().count() + 1,
                    None => span.col as usize + before.chars().count(),
                };
                let mut located = span;
                located.line = line as u32;
                located.col = col as u32;
                located.start = (start + at) as u32;
                located.end = (start + at + needle.len()) as u32;
                return located;
            }
        }
        span
    }
}

#[cfg(test)]
mod once {
    /// Each mistake is reported once: an argument used to be inferred once
    /// to look for a secret, again by a copy of that check, and a third
    /// time for its type, and said everything it found three times.
    #[test]
    fn a_mistake_in_an_argument_is_reported_once() {
        let src = "type User { name: String }\npage P(path: \"/\") {\n    state user = User(name: \"Ada\")\n    Heading(user.nmae).h1\n    Text(user.nmae).bold\n    Button(user.nmae) { on click { log(1) } }\n}\n";
        let program = crate::syntax::parser::v2::parse_v2(src, "t").unwrap();
        let info = super::check(&program, &|_| "t".into());
        let lines: Vec<usize> = info.findings.errors.iter().map(|e| e.line).collect();
        assert_eq!(lines, vec![4, 5, 6], "{:?}", info.findings.errors);
    }
}
