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
use crate::syntax::parser::ast::{
    ApiDecl, Arg, ArmPattern, BinOp, ComponentDecl, ComponentRef, Declaration, EmitStmt, Endpoint,
    EnumDecl, EventHandler, Expr, FieldDecl, IfStmt, LayoutRef, MatchStmt, Program, PropDecl, Span,
    Statement, StatementKind, StringPart, TypeDecl, TypeRef, UIElement, UnaryOp, awaits,
};

mod call;
mod connection;
mod contract;
mod element;
mod expr;
mod fake;
mod handles;
mod holes;
mod host;
mod jsdoc;
mod members;
mod method;
mod names;
mod order;
mod paging;
mod reactivity;
mod read;
mod refine;
mod report;
mod results;
pub mod shape;
mod signature;
mod stmt;
mod tables;
#[cfg(test)]
mod tests;
mod transport;
mod ty;
mod walk;

internal! {
    pub use handles::{browser_value_type, form_names, form_type, image_type, ref_names};
}
pub use holes::{Hole, HoleKind, of_project as holes_of};
pub use jsdoc::jsdoc_type;
internal! {
    pub use members::{Member, members};
}
pub(crate) use names::BUILT_IN_FUNCTIONS;
use names::{GRAMMAR_WORDS, global_type, own_name_for};
internal! {
    pub use names::{built_in_functions, language_name};
}
use refine::{ill_formed, rule_wants};
internal! {
    pub use refine::{VALIDATE_RULES, refinement_fault};
}
use tables::{
    LIST_METHODS, NUMBER_METHODS, SECRET_SINKS, STRING_METHODS, bound_type, case_signature,
    not_serialisable, scalar_method, scalar_methods_hint, string_method, takes_length,
};
use ty::{adds, conversion, counts, disjoint};
internal! {
    pub use ty::{PRIMITIVE_TYPES, Scalar, Type};
}
pub use walk::expr_text;
use walk::{
    arg_value, assigned, bound_names, in_optional_chain, keeps_present, match_arm, narrowed_names,
    non_empty_lists, null_names, read_names, written_names,
};

/// What the program declares, resolved once.
struct World<'p> {
    types: HashMap<&'p str, &'p TypeDecl>,
    enums: HashMap<&'p str, &'p EnumDecl>,
    components: HashMap<&'p str, &'p ComponentDecl>,
    /// Each store's members, typed.
    stores: HashMap<&'p str, HashMap<String, Type>>,
    /// Where each store is declared, by name.
    store_decl: HashMap<&'p str, usize>,
    /// The program's constants, typed.
    consts: HashMap<String, Type>,
    /// The services the program declares.
    apis: HashMap<&'p str, &'p ApiDecl>,
    /// The file each service is declared in, for a fix that edits it.
    api_files: HashMap<&'p str, String>,
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
    /// What `env` is: each name the build knows and the type of its value,
    /// when the checker is told the build's settings. Without them `env`
    /// reads any name, as text.
    env: Option<Vec<(String, Type)>>,
    /// Each store's state the program writes, from anywhere, as
    /// `Store.member`.
    store_written: HashSet<String>,
    /// Whether `Json` is the language's, not a type of the program's own.
    built_in_json: bool,
}

impl<'p> World<'p> {
    /// The service, or the group of one, an expression names: `Backend`,
    /// `Backend.users`.
    fn api_of(&self, e: &Expr) -> Option<String> {
        match e {
            Expr::Identifier(n) if self.apis.contains_key(n.as_str()) => Some(n.clone()),
            Expr::PropertyAccess(owner, group) => {
                let owner = self.api_of(owner)?;
                let name = format!("{owner}.{group}");
                self.apis.contains_key(name.as_str()).then_some(name)
            }
            _ => None,
        }
    }

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

    /// A `TypeRef::Named` may name an enum rather than a record, or another
    /// type by an alias.
    fn resolve(&self, ty: Type) -> Type {
        self.resolve_at(ty, 0)
    }

    fn resolve_at(&self, ty: Type, depth: usize) -> Type {
        match ty {
            // `type Scores = [String: Number]` is read as what it names; an
            // alias that names itself, however far round, is left unknown.
            Type::Record(name)
                if self
                    .types
                    .get(name.as_str())
                    .is_some_and(|t| t.alias.is_some()) =>
            {
                if depth > 16 {
                    return Type::Error;
                }
                let alias = self.types[name.as_str()].alias.as_ref().expect("an alias");
                self.resolve_at(Type::from_ref(alias), depth + 1)
            }
            Type::Record(name) if self.enums.contains_key(name.as_str()) => Type::Enum(name),
            // A name the language knows, unless the program declares a type
            // of its own by that name — which wins, so nothing the language
            // adds can take a name away.
            Type::Record(name)
                if !self.types.contains_key(name.as_str()) && Scalar::of_name(&name).is_some() =>
            {
                Type::Scalar(Scalar::of_name(&name).expect("just checked"))
            }
            // An object the browser hands a program — `e: KeyboardEvent`,
            // `node: Element` — or an instance of a project script's class.
            Type::Record(name)
                if !self.types.contains_key(name.as_str())
                    && (host::is_browser_object(&name)
                        || self.scripts.get(name.as_str()).is_some_and(|(_, n)| {
                            matches!(n.kind, crate::syntax::js_scan::NameKind::Class { .. })
                        })) =>
            {
                Type::Host(name)
            }
            Type::List(inner) => Type::list(self.resolve_at(*inner, depth)),
            Type::Optional(inner) => Type::optional(self.resolve_at(*inner, depth)),
            Type::Dict(k, v) => Type::Dict(
                Box::new(self.resolve_at(*k, depth)),
                Box::new(self.resolve_at(*v, depth)),
            ),
            Type::Func(params, ret) => Type::Func(
                params
                    .into_iter()
                    .map(|p| self.resolve_at(p, depth))
                    .collect(),
                Box::new(self.resolve_at(*ret, depth)),
            ),
            Type::Result(value, error) => Type::Result(
                Box::new(self.resolve_at(*value, depth)),
                Box::new(self.resolve_at(*error, depth)),
            ),
            Type::Promise(inner) => Type::Promise(Box::new(self.resolve_at(*inner, depth))),
            other => other,
        }
    }

    /// Whether `name` is a type the program can decode to: a record, an
    /// enum or an alias it declares, or one the language brings.
    fn decodable(&self, name: &str) -> bool {
        self.types.contains_key(name) || self.enums.contains_key(name)
    }
}

/// The types every program has without declaring them: `Json`, the open
/// JSON a value is matched or decoded out of; `DecodeError`, what a value
/// that does not fit its type is refused with; `HttpError`, what goes wrong
/// between the page and the server; `Problem`, a server's account of a
/// failure; and what the built-ins' props name — `Point`, `CodeFile`,
/// `RangePreset`. A program's own type of any of these names wins.
static BUILT_IN_TYPES: std::sync::LazyLock<Program> = std::sync::LazyLock::new(|| {
    crate::syntax::parse_source(
        "/// Open JSON: a value matched, or decoded into a declared type, before it is read.
enum Json {
    object(fields: [String: Json])
    array(items: [Json])
    string(value: String)
    number(value: Number)
    bool(value: Bool)
    null
}
/// Why a value is not the type it was decoded to: where, and what is wrong.
type DecodeError { at: String, message: String }
/// What went wrong between the page and the server. Every one has
/// `.message`, for showing.
enum HttpError {
    offline(queued: Bool)
    timeout(limit: Duration)
    cancelled
    transport(message: String)
    decode(at: String, message: String)
    status(code: Number, problem: Problem?)
}
/// A server's account of a failure, as RFC 9457 has it.
type Problem {
    type: Url? = null
    title: String? = null
    status: Number? = null
    detail: String? = null
    instance: Url? = null
    extensions: Json = Json.null
}
/// A place on the screen, in CSS pixels: where a layer opens.
type Point { x: Number, y: Number }
/// One file of a `CodeBlock`'s tabs.
type CodeFile { name: String, code: String, language: String = \"wf\" }
type Request { url: String, method: String, headers: [String: String] = {}, body: Json? = null }
/// A write kept while the connection was gone, which the server refused
/// once it was sent: `network.refused`.
type RefusedWrite { method: String, url: String, status: Number, at: DateTime }
/// A range a `DateRangePicker` offers to take at a press.
enum RangePreset { today, last7, last30, thisMonth, lastMonth }
",
        "<built-in>",
    )
    .expect("the built-in types parse")
});

/// The types every program has without declaring them, `Json` and
/// `DecodeError`, for a tool to list.
pub fn built_in_types() -> &'static Program {
    &BUILT_IN_TYPES
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
    /// What each call of the program's own actions passes, by the body
    /// that declares the action, its name and the argument's place — for
    /// the fix of a parameter written with no type (`T32`).
    passed: HashMap<(usize, String, usize), Vec<Type>>,
    /// The parameters written with no type, waiting for what their callers
    /// pass.
    untyped_params: Vec<signature::UntypedParam>,
    pub bindings: Vec<Typed>,
    /// Names read or called that nothing declares — not the program, not
    /// the language, not the browser. The build refuses them (`T13`); a
    /// template rendered with data reads its data's keys by name, so the
    /// template engine does not.
    pub unresolved: Vec<Diagnostic>,
    /// Every place a value's type is unknown, and why, in the order of
    /// their files and lines.
    pub holes: Vec<Hole>,
    /// What each `match` on the language's `Json` is over: a JSON value is
    /// plain at run time, so its case is read from what it is, and the code
    /// generator asks which subjects those are.
    pub json_subjects: Vec<Expr>,
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
    check_sources_with(program, file_of, source_of, None)
}

/// [`check_sources`], with the build's `env` known: each name and its
/// value. `env` is then a record of exactly those names, so one the build
/// does not know is `T05`, with the nearest name.
pub fn check_sources_with<'s>(
    program: &Program,
    file_of: &dyn Fn(usize) -> String,
    source_of: &dyn Fn(usize) -> Option<&'s str>,
    env: Option<&std::collections::BTreeMap<String, serde_json::Value>>,
) -> TypeInfo {
    let env = env.map(|env| {
        env.iter()
            .map(|(name, value)| {
                let ty = match value {
                    serde_json::Value::Number(_) => Type::Number,
                    serde_json::Value::Bool(_) => Type::Bool,
                    serde_json::Value::String(_) => Type::String,
                    _ => Type::Enum(ty::JSON.to_string()),
                };
                (name.clone(), ty)
            })
            .collect()
    });
    let mut world = World {
        env,
        types: HashMap::new(),
        enums: HashMap::new(),
        components: HashMap::new(),
        stores: HashMap::new(),
        store_decl: program
            .declarations
            .iter()
            .enumerate()
            .filter_map(|(i, d)| match d {
                Declaration::Store(s) => Some((s.name.as_str(), i)),
                _ => None,
            })
            .collect(),
        consts: HashMap::new(),
        apis: HashMap::new(),
        api_files: HashMap::new(),
        scripts: HashMap::new(),
        fixed: HashMap::new(),
        store_fixed: HashMap::new(),
        store_actions: HashMap::new(),
        store_written: HashSet::new(),
        built_in_json: false,
    };
    for (index, decl) in program.declarations.iter().enumerate() {
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
                world.api_files.insert(a.name.as_str(), file_of(index));
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
            Declaration::Middleware(m) => {
                world.fixed.insert(m.name.clone(), "a middleware");
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
    for decl in &BUILT_IN_TYPES.declarations {
        let taken = |name: &str| world.types.contains_key(name) || world.enums.contains_key(name);
        match decl {
            Declaration::Enum(e) if !taken(&e.name) => {
                world.built_in_json |= e.name == ty::JSON;
                world.enums.insert(e.name.as_str(), e);
            }
            Declaration::Type(t) if !taken(&t.name) => {
                world.types.insert(t.name.as_str(), t);
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
                let file = file_of(index);
                let mut cx = Checker::new(&world, &file, index, &mut info, None);
                cx.current_span = d.span;
                match &d.ty {
                    Some(t) => {
                        cx.written(t, &d.name);
                        world.resolve(Type::from_ref(t))
                    }
                    None => cx.hole(HoleKind::UntypedData, &d.name),
                }
            };
            info.bindings.push(Typed {
                decl: index,
                name: d.name.clone(),
                span: d.span,
                ty: ty.clone(),
            });
            world.consts.insert(d.name.clone(), ty);
        }
        if let Declaration::Middleware(m) = decl {
            world
                .consts
                .insert(m.name.clone(), transport::middleware_type());
        }
        if let Declaration::Const(c) = decl {
            let file = file_of(index);
            let source = source_of(index);
            let mut cx = Checker::new(&world, &file, index, &mut info, source);
            cx.current_span = c.span;
            if let Some(t) = &c.ty {
                cx.written(t, &c.name);
            }
            let declared = c.ty.as_ref().map(|t| cx.world.resolve(Type::from_ref(t)));
            // A `data` file is data from outside the program: held to its
            // type as JSON, and a fault named by the file and the place in
            // it (`T33`). What reads it then reads the declared type, and
            // the literal nobody wrote is not reported again.
            let unfit = match (&c.source, &c.ty) {
                (Some(file), Some(ty)) => data_fault(program, &c.value, ty).map(|fault| {
                    cx.error(
                        c.span,
                        "T33",
                        format!("`{file}` is not what `data {}` says it is: {fault}", c.name),
                        &format!(
                            "Correct the file at `{}`, or change the type to what the file holds",
                            fault.at
                        ),
                    );
                }),
                _ => None,
            };
            let given = if unfit.is_some() {
                declared.clone().unwrap_or(Type::Error)
            } else {
                cx.infer(&c.value, declared.as_ref())
            };
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

    // Stores next: their members are read everywhere else, and a store's
    // derived values may read another's, declared before it or after. So
    // the stores are worked out quietly until what each holds stops
    // changing — a chain of stores reading stores settles in as many
    // rounds as it is long — and only then checked, with what they report.
    let cross = store_cycles(program);
    let rounds = program
        .declarations
        .iter()
        .filter(|d| matches!(d, Declaration::Store(_)))
        .count()
        + 1;
    for _ in 0..rounds {
        let mut scratch = TypeInfo::default();
        let before: Vec<String> = store_signature(&world);
        for (index, decl) in program.declarations.iter().enumerate() {
            if let Declaration::Store(s) = decl {
                let file = file_of(index);
                let mut cx = Checker::new(&world, &file, index, &mut scratch, None);
                cx.cyclic = cross
                    .iter()
                    .filter(|(st, _, _)| *st == index)
                    .map(|(_, n, _)| n.clone())
                    .collect();
                cx.push_scope();
                cx.declare_hoisted(&s.body);
                let members = cx.scopes.last().cloned().unwrap_or_default();
                cx.pop_scope();
                world.stores.insert(s.name.as_str(), members);
            }
        }
        if store_signature(&world) == before {
            break;
        }
    }
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
            cx.cyclic = cross
                .iter()
                .filter(|(st, _, _)| *st == index)
                .map(|(_, n, _)| n.clone())
                .collect();
            for (_, name, others) in cross.iter().filter(|(st, _, _)| *st == index) {
                if let Some(stmt) = s
                    .body
                    .iter()
                    .find(|st| matches!(&st.kind, StatementKind::Derived(d) if d.name == *name))
                {
                    cx.error(
                        stmt.span,
                        "T35",
                        format!(
                            "`{}.{name}` and {others} are each worked out from the other, so neither says what it is",
                            s.name
                        ),
                        &format!("Declare what one of them is: `derived {name}: T = …`"),
                    );
                }
            }
            cx.push_scope();
            cx.declare_hoisted(&s.body);
            let members = cx.scopes.last().cloned().unwrap_or_default();
            cx.statements(&s.body, Body::Store);
            cx.report_untyped();
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
                let mut route = Vec::new();
                for param in &p.params {
                    cx.current_span = param.span;
                    cx.written(&param.prop_type, &param.name);
                    let ty = cx.world.resolve(Type::from_ref(&param.prop_type));
                    route.push((param.name.clone(), ty.clone()));
                    cx.bind_fixed(&param.name, ty, param.span, "a route parameter");
                }
                // `params` is a record of the page's own parameters.
                if let Some(scope) = cx.scopes.last_mut() {
                    scope.insert("params".to_string(), Type::Shape(route));
                }
                if let Some(layout) = &p.layout {
                    cx.current_span = layout.span;
                    cx.layout(layout);
                }
                cx.bound = bound_names(&p.body);
                cx.written = written_names(&p.body);
                cx.declare_hoisted(&p.body);
                cx.statements(&p.body, Body::Page);
                cx.report_untyped();
                cx.pop_scope();
            }
            Declaration::Component(c) => {
                cx.push_scope();
                for prop in &c.props {
                    cx.current_span = prop.span;
                    cx.written(&prop.prop_type, &prop.name);
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
                for event in &c.events {
                    cx.current_span = event.span;
                    for p in &event.params {
                        cx.written_param(
                            &p.param_type,
                            &format!("{}({})", event.name, p.name),
                            p.untyped,
                        );
                        if p.untyped {
                            cx.error_at_current(
                                "T32",
                                format!(
                                    "`{}` of the event `{}` says nothing of its type",
                                    p.name, event.name
                                ),
                                &format!(
                                    "Write what it sends: `event {}({}: String)`",
                                    event.name, p.name
                                ),
                            );
                        }
                    }
                }
                cx.current_span = c.header_span;
                for slot in &c.slots {
                    for p in &slot.params {
                        cx.written_param(&p.param_type, &p.name, p.untyped);
                        if p.untyped {
                            cx.error_at_current(
                                "T32",
                                format!(
                                    "`{}` of the slot `{}` says nothing of its type",
                                    p.name,
                                    slot.name.as_deref().unwrap_or("children")
                                ),
                                &format!(
                                    "Write what it hands over: `slot {}({}: Todo)`",
                                    slot.name.as_deref().unwrap_or("row"),
                                    p.name
                                ),
                            );
                        }
                    }
                }
                cx.component = Some(c);
                cx.bound = bound_names(&c.body);
                cx.written = written_names(&c.body);
                cx.declare_hoisted(&c.body);
                cx.statements(&c.body, Body::Component);
                cx.report_untyped();
                cx.pop_scope();
            }
            Declaration::App(a) => {
                cx.push_scope();
                cx.written = written_names(&a.body);
                cx.declare_hoisted(&a.body);
                cx.statements(&a.body, Body::Page);
                cx.report_untyped();
                cx.pop_scope();
            }
            Declaration::Type(t) => {
                for field in &t.fields {
                    cx.current_span = field.span;
                    cx.written(&field.ty, &format!("{}.{}", t.name, field.name));
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
                for (key, e) in a.settings.iter().chain(a.headers.iter()) {
                    cx.current_span = a.span;
                    if key == "middleware" {
                        cx.middleware_list(e);
                    } else {
                        cx.infer(e, None);
                    }
                }
                cx.old_transport(a);
                cx.group_of(a);
                cx.service_blocks(a);
                for endpoint in &a.endpoints {
                    cx.current_span = endpoint.span;
                    let what = format!("{}.{}", a.name, endpoint.name);
                    if let Some(returns) = &endpoint.returns {
                        let ty = cx.world.resolve(Type::from_ref(returns));
                        cx.result_error(&ty, endpoint.span, &format!("`{what}`"));
                        cx.boundary_written(
                            returns,
                            endpoint.span,
                            &format!("what `{what}` answers"),
                        );
                    }
                    for p in &endpoint.params {
                        cx.boundary_written(
                            &p.prop_type,
                            endpoint.span,
                            &format!("`{}` of `{what}`", p.name),
                        );
                    }
                    cx.endpoint_form(a, endpoint);
                    for p in &endpoint.params {
                        cx.at_boundary = true;
                        cx.written_param(&p.prop_type, &format!("{what}({})", p.name), false);
                        cx.at_boundary = false;
                    }
                    for t in endpoint
                        .returns
                        .iter()
                        .chain(endpoint.errors.iter().map(|(_, t)| t))
                    {
                        cx.written(t, &what);
                    }
                    // `invalidates [list, byId(id)]` names endpoints and
                    // arguments, which `contract` checks, not values.
                    for (k, e) in &endpoint.settings {
                        if k == "invalidates" || k == "pages" {
                            continue;
                        }
                        cx.current_span = endpoint.span;
                        cx.infer(e, None);
                    }
                    cx.contract(a, endpoint);
                    cx.paging(a, endpoint);
                }
                for hook in &a.hooks {
                    cx.push_scope();
                    cx.current_span = hook.span;
                    let given = cx.hole(HoleKind::ServiceHook, format!("on {}", hook.event));
                    cx.handler_names(
                        hook,
                        &[given],
                        &format!("an `api`'s `on {}` hook receives one value", hook.event),
                    );
                    cx.returns.push(Vec::new());
                    cx.declared_returns.push(None);
                    cx.async_ok = true;
                    cx.in_handler += 1;
                    cx.statements(&hook.body, Body::Imperative);
                    cx.in_handler -= 1;
                    cx.async_ok = false;
                    cx.returns.pop();
                    cx.declared_returns.pop();
                    cx.pop_scope();
                }
            }
            Declaration::Enum(e) => cx.error_enum_shape(e),
            Declaration::Middleware(m) => cx.middleware(m),
            Declaration::Fake(f) => {
                // An answer `from examples` is JSON from the spec: held to
                // the endpoint's answer as data is (`T33`), the fault named
                // in the example, and not checked again as code.
                let mut unfit = Vec::new();
                for (i, a) in f.answers.iter().enumerate() {
                    let Some(label) = &a.example else { continue };
                    let Some(endpoint) = cx.world.endpoint(&f.target, &a.endpoint) else {
                        continue;
                    };
                    let (Some(TypeRef::Result(value, _)), Expr::CaseValue(_, args)) =
                        (&endpoint.returns, &a.body)
                    else {
                        continue;
                    };
                    let Some(fault) = args.first().and_then(|e| data_fault(program, e, value))
                    else {
                        continue;
                    };
                    cx.error(
                        a.span,
                        "T33",
                        format!("{label} is not what `{}.{}` answers: {fault}", f.target, a.endpoint),
                        &format!(
                            "Correct the example in the spec at `{}`, or answer `{}` in the fake yourself",
                            fault.at, a.endpoint
                        ),
                    );
                    unfit.push(i);
                }
                cx.fake(f, &unfit);
            }
            Declaration::Store(_)
            | Declaration::Theme(_)
            | Declaration::Script(_)
            | Declaration::Const(_)
            | Declaration::Animation(_)
            | Declaration::Test(_)
            | Declaration::Data(_) => {}
        }
    }
    // A parameter written with no type, reported once every call of it has
    // been seen, with the type they pass when they agree.
    for pending in std::mem::take(&mut info.untyped_params) {
        let file = file_of(pending.decl);
        let source = source_of(pending.decl);
        let passed = info
            .passed
            .get(&(pending.decl, pending.action.clone(), pending.index))
            .cloned()
            .unwrap_or_default();
        let mut cx = Checker::new(&world, &file, pending.decl, &mut info, source);
        cx.report_untyped_param(&pending, &passed);
    }
    info.holes.extend(holes::unchecked_tests(program, file_of));
    // An expression inferred twice — quietly, then for real — is one hole.
    info.holes.sort();
    info.holes.dedup();
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

#[expect(
    clippy::struct_excessive_bools,
    reason = "independent facts about where the checker is"
)]
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
    /// What the action being checked declares it returns (`-> T`), beside
    /// `returns`; `None` where nothing is declared.
    declared_returns: Vec<Option<Type>>,
    /// The errors each action being read passes up with `try` while it
    /// declares nothing it returns: said once, at the action, with the
    /// `-> ! E` that says so.
    untyped_tries: Vec<Vec<Type>>,
    /// Where a service meets the program, whose `N11` says what `T23` and
    /// `T24` would.
    at_boundary: bool,
    /// What an `if` value read next is wanted as, for its arms.
    if_wanted: Option<Type>,
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
    /// What one item of each enclosing built-in that shows a list is — a
    /// `DataTable`'s row, a `Tree`'s item — innermost last: what its props,
    /// its parts and its slots mean by `Item`.
    items: Vec<Type>,
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
    /// How many `try { } catch` bodies enclose what is checked: a result
    /// there is not taken with `try`, whose failure would skip the `catch`.
    catching: usize,
    /// Whether the finding reported last was an error, for the fix that
    /// follows it ([`Checker::plan_last`]).
    last_was_error: bool,
    /// The store's `derived` values that read another store's, which read
    /// them (`T35`): they have no type to be worked out.
    cyclic: HashSet<String>,
    /// The body's states whose first value says nothing (`T25`), and what
    /// the body assigns each.
    untyped: Vec<signature::Untyped>,
    assigned: HashMap<String, Type>,
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
            declared_returns: Vec::new(),
            untyped_tries: Vec::new(),
            at_boundary: false,
            if_wanted: None,
            current_span: Span::dummy(),
            in_args: false,
            async_ok: false,
            in_derived: false,
            in_handler: 0,
            in_running: 0,
            items: Vec::new(),
            refined: HashMap::new(),
            non_empty: Vec::new(),
            in_chain: false,
            bound: HashSet::new(),
            written: HashSet::new(),
            own_actions: HashMap::new(),
            depth: 0,
            too_deep: false,
            catching: 0,
            last_was_error: false,
            cyclic: HashSet::new(),
            untyped: Vec::new(),
            assigned: HashMap::new(),
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
    /// The body a failure of this request is declared to carry: what the
    /// endpoint's `errors { 422 -> T }` names, when every status it lists
    /// carries the one type; `Any` otherwise, and for a plain `fetch`.
    /// What a read over `url` fails with: the error its endpoint declares
    /// (`-> T ! E`), or an `HttpError` — what `.err` holds for the same
    /// request.
    fn read_failure(&self, url: &Expr) -> Type {
        let http = || self.world.resolve(Type::Record(ty::HTTP_ERROR.to_string()));
        let Expr::MethodCall(obj, method, _) = url else {
            return http();
        };
        let Some(api) = self.world.api_of(obj) else {
            return http();
        };
        match self
            .world
            .endpoint(&api, method)
            .and_then(|e| e.returns.as_ref())
            .map(|t| self.world.resolve(Type::from_ref(t)))
        {
            Some(Type::Result(_, error)) => self.world.resolve(*error),
            _ => http(),
        }
    }

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
        // A map built by hand, or a record, where the other is wanted:
        // agreement is field by field.
        if let Some((code, message, hint)) = self.record_fault(given, wanted) {
            self.error(span, code, format!("{what}: {message}"), &hint);
            return;
        }
        if let Some((message, hint)) = conversion(given, wanted) {
            self.error(span, "T34", format!("{what} is `{given}`, {message}"), hint);
            return;
        }
        if given.assignable_to(wanted) {
            self.loosely(given, wanted, what);
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
        // A result where its value is wanted: `try` gives the value, and
        // passes the failure up.
        let result_of_wanted =
            matches!(given, Type::Result(value, _) if value.assignable_to(wanted));
        let hint = if result_of_wanted && self.catching > 0 {
            "A result is not thrown, so the `catch` around it never hears a failure: `match await … { .ok(value) { … } .err(e) { … } }`".to_string()
        } else if result_of_wanted {
            "Take its value with `try`, which passes a failure up: `let rows: [Row] = try await http.get(url)`".to_string()
        } else {
            hint
        };
        self.error(
            span,
            "T01",
            format!("{what} is `{given}`, but `{wanted}` is wanted"),
            &hint,
        );
        // Inside `try { } catch` the failure would leave the action, not
        // reach the `catch`: a person writes the `match`.
        if result_of_wanted && self.catching == 0 {
            self.plan_last(
                "Write `try await`".to_string(),
                crate::diagnostics::fixes::Plan::Rename {
                    from: "await".to_string(),
                    to: "try await".to_string(),
                },
            );
        }
    }

    /// Why a map literal's shape — or a record — does not fit where the
    /// other is wanted, looking through lists and `null`: a field the
    /// record lacks is `T05`, a required one left out `C01`, one of the
    /// wrong type `T01`. `None` when the two agree, or are not a shape and
    /// a record.
    fn record_fault(&self, given: &Type, wanted: &Type) -> Option<(&'static str, String, String)> {
        match (given, wanted) {
            (Type::Optional(a), Type::Optional(b)) | (Type::List(a), Type::List(b)) => {
                self.record_fault(a, b)
            }
            (_, Type::Optional(b)) => self.record_fault(given, b),
            (Type::Shape(fields), Type::Record(record)) => {
                let all = self.world.record_fields(record)?;
                let names: Vec<String> = all.iter().map(|f| format!("`{}`", f.name)).collect();
                for (name, ty) in fields {
                    let Some(field) = all.iter().find(|f| &f.name == name) else {
                        return Some((
                            "T05",
                            format!("`{record}` has no field `{name}`"),
                            format!("Its fields are {}", names.join(", ")),
                        ));
                    };
                    let field_ty = self.world.resolve(Type::from_ref(&field.ty));
                    if let Some(inner) = self.record_fault(ty, &field_ty) {
                        return Some(inner);
                    }
                    if !ty.assignable_to(&field_ty) {
                        return Some((
                            "T01",
                            format!(
                                "`{name}` of `{record}` is `{field_ty}`, but this one is `{ty}`"
                            ),
                            String::new(),
                        ));
                    }
                }
                let missing = all.iter().find(|f| {
                    f.default.is_none()
                        && !matches!(f.ty, TypeRef::Optional(_))
                        && !fields.iter().any(|(n, _)| n == &f.name)
                })?;
                Some((
                    "C01",
                    format!(
                        "the map leaves out `{}` of `{record}`, which has no default",
                        missing.name
                    ),
                    format!("Give it `{}`", missing.name),
                ))
            }
            // A record given where a map literal's shape is wanted: it has
            // every field the shape has, each of its type.
            (Type::Record(record), Type::Shape(fields)) => {
                let all = self.world.record_fields(record)?;
                for (name, ty) in fields {
                    let Some(field) = all.iter().find(|f| &f.name == name) else {
                        return Some((
                            "T05",
                            format!("`{record}` has no field `{name}`, which this map has"),
                            String::new(),
                        ));
                    };
                    let field_ty = self.world.resolve(Type::from_ref(&field.ty));
                    if !field_ty.assignable_to(ty) {
                        return Some((
                            "T01",
                            format!("`{name}` of `{record}` is `{field_ty}`, but `{ty}` is wanted"),
                            String::new(),
                        ));
                    }
                }
                None
            }
            _ => None,
        }
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
                    // A state that starts as `null` will hold something
                    // later: a value that may be null.
                    let ty = if let Some(t) = &s.ty {
                        self.written(t, &s.name);
                        self.world.resolve(Type::from_ref(t))
                    } else {
                        {
                            let ty = match self.infer(&s.value, None) {
                                Type::Null => {
                                    Type::optional(self.hole(HoleKind::NullState, &s.name))
                                }
                                other => other,
                            };
                            if signature::says_nothing(&ty) {
                                self.untyped.push(signature::Untyped {
                                    name: s.name.clone(),
                                    span: stmt.span,
                                    persist: s.persist,
                                    null: matches!(s.value, Expr::Null),
                                    start: match &s.value {
                                        Expr::Null => "null",
                                        Expr::ListLiteral(_) => "[]",
                                        Expr::MapLiteral(_) => "{}",
                                        _ => "…",
                                    },
                                });
                            }
                            ty
                        }
                    };
                    self.bind(&s.name, ty, stmt.span);
                }
                StatementKind::Resource(r) => {
                    self.resource_source(r, stmt.span);
                    self.current_span = stmt.span;
                    self.read_options(r);
                    let inner = match &r.ty {
                        Some(t) => {
                            self.at_boundary = true;
                            self.written(t, &r.name);
                            self.at_boundary = false;
                            self.world.resolve(Type::from_ref(t))
                        }
                        // An endpoint says what it returns, so a resource
                        // over one is typed without an annotation.
                        None => match &r.url {
                            Expr::MethodCall(obj, method, _) => {
                                match self.world.api_of(obj) {
                                    Some(api) => {
                                        match self.world.endpoint(&api, method) {
                                            // A paged read holds every
                                            // page's items, gathered.
                                            Some(e) if e.pages().is_some() => {
                                                self.paged_items(e, true).unwrap_or(Type::Error)
                                            }
                                            // A read of `-> T ! E` holds the `T`.
                                            Some(e) if e.returns.is_some() => {
                                                answer_of(self.world.resolve(Type::from_ref(
                                                    e.returns.as_ref().expect("checked"),
                                                )))
                                            }
                                            _ => self.hole(HoleKind::UntypedResource, &r.name),
                                        }
                                    }
                                    None => self.hole(HoleKind::UntypedResource, &r.name),
                                }
                            }
                            _ => self.hole(HoleKind::UntypedResource, &r.name),
                        },
                    };
                    // A read is of what can be read: a resource over a
                    // write would send it every time the page drew it.
                    if let Expr::MethodCall(obj, method, _) = &r.url
                        && let Some(api) = self.world.api_of(obj)
                        && let Some(endpoint) = self.world.endpoint(&api, method)
                        && !matches!(endpoint.method.as_str(), "GET" | "HEAD")
                    {
                        self.error(
                            stmt.span,
                            "N02",
                            format!(
                                "`resource {}` reads `{api}.{method}`, a `{}`: a write is sent when something happens, not whenever the page draws",
                                r.name, endpoint.method
                            ),
                            &format!("Call it from an action: `action save() {{ match await {api}.{method}(…) {{ .ok(v) {{ … }} .err(e) {{ … }} }} }}`"),
                        );
                    }
                    // An annotation says what the endpoint returns, or it
                    // is wrong: `resource u: [User] = Backend.user(…)`.
                    if r.ty.is_some()
                        && let Expr::MethodCall(obj, method, _) = &r.url
                        && let Some(api) = self.world.api_of(obj)
                        && let Some(endpoint) = self.world.endpoint(&api, method)
                        && let Some(returns) = endpoint.returns.as_ref()
                    {
                        let returns = match self.paged_items(endpoint, true) {
                            Some(items) => items,
                            None => answer_of(self.world.resolve(Type::from_ref(returns))),
                        };
                        if !returns.assignable_to(&inner) {
                            self.error(
                                stmt.span,
                                "T01",
                                format!(
                                    "`{}` is declared `{inner}`, but `{api}.{method}` returns `{returns}`",
                                    r.name
                                ),
                                &format!("Declare it `{returns}`, or leave the type out: the endpoint says it"),
                            );
                        }
                    }
                    let failed = self.read_failure(&r.url);
                    self.bind_fixed(
                        &r.name,
                        Type::Resource(Box::new(inner), Box::new(failed)),
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
                    let handle = self.connection_type(c);
                    self.bind_fixed(&c.name, handle, stmt.span, "a connection");
                }
                _ => {}
            }
            // `ref: name` anywhere in the body declares `name`: a handle on
            // the element, read like the element itself.
            for name in ref_names(std::slice::from_ref(stmt)) {
                let handle = Type::Host("Element".to_string());
                self.bind_fixed(&name, handle, stmt.span, "an element's handle");
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
                for (index, p) in a.params.iter().enumerate() {
                    self.written_param(
                        &p.param_type,
                        &format!("{}({})", a.name, p.name),
                        p.untyped,
                    );
                    if p.untyped {
                        self.info.untyped_params.push(signature::UntypedParam {
                            decl: self.decl,
                            span: stmt.span,
                            action: a.name.clone(),
                            index,
                            name: p.name.clone(),
                        });
                    }
                }
                let params: Vec<Type> = a
                    .params
                    .iter()
                    .map(|p| self.world.resolve(Type::from_ref(&p.param_type)))
                    .collect();
                // What it declares it returns; else the return type is
                // found when the body is checked. An action that awaits
                // hands back a promise of it.
                let result = if let Some(t) = &a.returns {
                    self.written(t, &a.name);
                    self.world.resolve(Type::from_ref(t))
                } else if holes::returns_value(&a.body) {
                    self.hole(HoleKind::ActionResult, &a.name)
                } else {
                    Type::Any
                };
                let ret = if awaits(&a.body) {
                    Type::Promise(Box::new(result))
                } else {
                    result
                };
                self.bind_fixed(
                    &a.name,
                    Type::Func(params, Box::new(ret)),
                    stmt.span,
                    "an action",
                );
            }
        }
        // Derived values after what they read, so the order they are
        // written in cannot change a type; two that read each other with
        // neither declared have nothing to start from (`T35`).
        let derived: Vec<(&Statement, &crate::syntax::parser::ast::DerivedDecl)> = stmts
            .iter()
            .filter_map(|s| match &s.kind {
                StatementKind::Derived(d) => Some((s, d)),
                _ => None,
            })
            .collect();
        let names: Vec<&str> = derived.iter().map(|(_, d)| d.name.as_str()).collect();
        let values: Vec<&Expr> = derived.iter().map(|(_, d)| &d.value).collect();
        let deps = order::bare_deps(&names, &values);
        for group in order::components(&deps) {
            let cyclic =
                order::is_cycle(&group, &deps) && group.iter().all(|&i| derived[i].1.ty.is_none());
            if cyclic {
                for &i in &group {
                    let (stmt, d) = derived[i];
                    let others: Vec<String> = group
                        .iter()
                        .filter(|&&j| j != i)
                        .map(|&j| format!("`{}`", names[j]))
                        .collect();
                    let message = if others.is_empty() {
                        format!(
                            "`{}` is worked out from itself, so nothing says what it is",
                            d.name
                        )
                    } else {
                        format!(
                            "`{}` and {} are each worked out from the other, so neither says what it is",
                            d.name,
                            others.join(" and ")
                        )
                    };
                    self.error(
                        stmt.span,
                        "T35",
                        message,
                        &format!("Declare what one of them is: `derived {}: T = …`", d.name),
                    );
                    self.bind_fixed(&d.name, Type::Error, stmt.span, "a `derived` value");
                }
                continue;
            }
            for &i in &group {
                let (stmt, d) = derived[i];
                self.current_span = stmt.span;
                if self.cyclic.contains(&d.name) {
                    self.bind_fixed(&d.name, Type::Error, stmt.span, "a `derived` value");
                    continue;
                }
                if let Some(t) = &d.ty {
                    self.written(t, &d.name);
                }
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

/// What a read of an endpoint holds: its answer, a result's value apart
/// from what it fails with.
/// What every store's members are, written out, to tell when another
/// round of working them out changed nothing.
fn store_signature(world: &World<'_>) -> Vec<String> {
    let mut out: Vec<String> = world
        .stores
        .iter()
        .flat_map(|(store, members)| {
            members
                .iter()
                .map(move |(name, ty)| format!("{store}.{name}: {ty}"))
        })
        .collect();
    out.sort();
    out
}

/// The `derived` values of two or more stores that read each other with
/// none of them declared: each one's store (its declaration's index), its
/// name, and the others, for `T35`. A cycle inside one store is that
/// store's body's to report.
fn store_cycles(program: &Program) -> Vec<(usize, String, String)> {
    let mut nodes: Vec<(usize, &str, &str, &crate::syntax::parser::ast::DerivedDecl)> = Vec::new();
    for (index, decl) in program.declarations.iter().enumerate() {
        if let Declaration::Store(s) = decl {
            for stmt in &s.body {
                if let StatementKind::Derived(d) = &stmt.kind {
                    nodes.push((index, s.name.as_str(), d.name.as_str(), d));
                }
            }
        }
    }
    let deps: Vec<Vec<usize>> = nodes
        .iter()
        .map(|(_, store, _, d)| {
            let mut read = Vec::new();
            order::reads(&d.value, &mut read);
            let mut out: Vec<usize> = read
                .iter()
                .filter_map(|(owner, member)| {
                    let owner = owner.as_deref().unwrap_or(store);
                    nodes
                        .iter()
                        .position(|(_, s, n, _)| *s == owner && *n == member.as_str())
                })
                .collect();
            out.sort_unstable();
            out.dedup();
            out
        })
        .collect();
    let mut found = Vec::new();
    for group in order::components(&deps) {
        let stores: HashSet<&str> = group.iter().map(|&i| nodes[i].1).collect();
        if stores.len() < 2
            || !order::is_cycle(&group, &deps)
            || group.iter().any(|&i| nodes[i].3.ty.is_some())
        {
            continue;
        }
        for &i in &group {
            let others: Vec<String> = group
                .iter()
                .filter(|&&j| j != i)
                .map(|&j| format!("`{}.{}`", nodes[j].1, nodes[j].2))
                .collect();
            found.push((nodes[i].0, nodes[i].2.to_string(), others.join(" and ")));
        }
    }
    found
}

/// Where a `data` constant's value — its file's JSON — is not the type it
/// declares.
fn data_fault(program: &Program, value: &Expr, ty: &TypeRef) -> Option<shape::Fault> {
    let json =
        crate::project::static_eval::eval(value, &crate::project::static_eval::Scope::of([]))?
            .to_json();
    let shapes = shape::Shapes::of(&program.declarations);
    shape::fault(&json, &shapes.shape(ty))
}

fn answer_of(ty: Type) -> Type {
    match ty {
        Type::Result(value, _) => *value,
        other => other,
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
