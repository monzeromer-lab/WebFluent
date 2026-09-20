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

use std::collections::HashMap;

use super::Findings;
use crate::error::Diagnostic;
use crate::parser::ast::*;
use crate::registry::{self, PropType};

/// A type the checker reasons about.
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    /// Unknown, or anything: agrees with every type.
    Any,
    String,
    Number,
    Bool,
    Null,
    /// A design token, `$name`.
    Token,
    List(Box<Type>),
    /// A map with unknown keys (an object literal).
    Map,
    /// A record of a declared `type`.
    Record(String),
    /// A value of a declared `enum`.
    Enum(String),
    /// A case whose enum is not yet known: `.danger` on its own.
    Case(String),
    /// A value that may be `null`.
    Optional(Box<Type>),
    /// An action, a lambda or a store's action.
    Func(Vec<Type>, Box<Type>),
    /// A `resource`, matched with `loading`, `error`, `ready`.
    Resource(Box<Type>),
    /// A store brought into scope with `use`.
    Store(String),
}

impl Type {
    pub fn from_ref(ty: &TypeRef) -> Type {
        match ty {
            TypeRef::String => Type::String,
            TypeRef::Number => Type::Number,
            TypeRef::Bool => Type::Bool,
            TypeRef::Map => Type::Map,
            TypeRef::Any => Type::Any,
            TypeRef::List(inner) => Type::List(Box::new(Type::from_ref(inner))),
            TypeRef::Optional(inner) => Type::Optional(Box::new(Type::from_ref(inner))),
            TypeRef::Named(name) => Type::Record(name.clone()),
        }
    }

    fn list(inner: Type) -> Type {
        Type::List(Box::new(inner))
    }

    fn optional(inner: Type) -> Type {
        match inner {
            Type::Optional(_) | Type::Null => inner,
            other => Type::Optional(Box::new(other)),
        }
    }

    /// The type without its `null`.
    fn unwrapped(&self) -> Type {
        match self {
            Type::Optional(inner) => (**inner).clone(),
            other => other.clone(),
        }
    }

    pub fn is_any(&self) -> bool {
        matches!(self, Type::Any)
    }

    /// Whether a value of `self` may be given where `to` is expected.
    fn assignable_to(&self, to: &Type) -> bool {
        match (self, to) {
            (Type::Any, _) | (_, Type::Any) => true,
            (Type::Case(_), Type::Enum(_)) => true,
            (Type::Null, Type::Optional(_)) => true,
            (Type::Null, Type::Null) => true,
            (Type::Optional(a), Type::Optional(b)) => a.assignable_to(b),
            (from, Type::Optional(b)) => from.assignable_to(b),
            (Type::List(a), Type::List(b)) => a.assignable_to(b),
            (Type::Map, Type::Record(_)) | (Type::Record(_), Type::Map) => true,
            (Type::Func(..), Type::Func(..)) => true,
            (Type::Resource(a), Type::Resource(b)) => a.assignable_to(b),
            (a, b) => a == b,
        }
    }

    /// The type both `a` and `b` fit: their common type, or `Any`.
    fn join(a: Type, b: Type) -> Type {
        match (a, b) {
            (a, b) if a == b => a,
            (Type::Any, _) | (_, Type::Any) => Type::Any,
            (Type::Null, other) | (other, Type::Null) => Type::optional(other),
            (Type::Optional(a), b) | (b, Type::Optional(a)) => Type::optional(Type::join(*a, b)),
            (Type::List(a), Type::List(b)) => Type::list(Type::join(*a, *b)),
            (Type::Case(_), Type::Enum(e)) | (Type::Enum(e), Type::Case(_)) => Type::Enum(e),
            (Type::Case(_), Type::Case(_)) => Type::Any,
            _ => Type::Any,
        }
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Any => write!(f, "Any"),
            Type::String => write!(f, "String"),
            Type::Number => write!(f, "Number"),
            Type::Bool => write!(f, "Bool"),
            Type::Null => write!(f, "null"),
            Type::Token => write!(f, "$token"),
            Type::List(inner) => write!(f, "[{inner}]"),
            Type::Map => write!(f, "Map"),
            Type::Record(name) | Type::Enum(name) | Type::Store(name) => write!(f, "{name}"),
            Type::Case(name) => write!(f, ".{name}"),
            Type::Optional(inner) => write!(f, "{inner}?"),
            Type::Func(params, ret) => {
                let params: Vec<String> = params.iter().map(|p| p.to_string()).collect();
                if **ret == Type::Null {
                    write!(f, "action({})", params.join(", "))
                } else {
                    write!(f, "action({}) -> {ret}", params.join(", "))
                }
            }
            Type::Resource(inner) => write!(f, "resource<{inner}>"),
        }
    }
}

/// What the program declares, resolved once.
struct World<'p> {
    types: HashMap<&'p str, &'p TypeDecl>,
    enums: HashMap<&'p str, &'p EnumDecl>,
    components: HashMap<&'p str, &'p ComponentDecl>,
    /// Each store's members, typed.
    stores: HashMap<&'p str, HashMap<String, Type>>,
}

impl<'p> World<'p> {
    fn record_field(&self, record: &str, field: &str) -> Option<Type> {
        let decl = self.types.get(record)?;
        decl.fields
            .iter()
            .find(|f| f.name == field)
            .map(|f| Type::from_ref(&f.ty))
    }

    /// A `TypeRef::Named` may name an enum rather than a record.
    fn resolve(&self, ty: Type) -> Type {
        match ty {
            Type::Record(name) if self.enums.contains_key(name.as_str()) => Type::Enum(name),
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
#[derive(Default)]
pub struct TypeInfo {
    pub findings: Findings,
    pub bindings: Vec<Typed>,
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
    let mut world = World {
        types: HashMap::new(),
        enums: HashMap::new(),
        components: HashMap::new(),
        stores: HashMap::new(),
    };
    for decl in &program.declarations {
        match decl {
            Declaration::Type(t) => {
                world.types.insert(t.name.as_str(), t);
            }
            Declaration::Enum(e) => {
                world.enums.insert(e.name.as_str(), e);
            }
            Declaration::Component(c) => {
                world.components.insert(c.name.as_str(), c);
            }
            _ => {}
        }
    }
    let mut info = TypeInfo::default();

    // Stores first: their members are read everywhere else. A store's
    // derived values and actions may read other stores, which are `Any`
    // until met.
    for (index, decl) in program.declarations.iter().enumerate() {
        if let Declaration::Store(s) = decl {
            let file = file_of(index);
            let mut cx = Checker::new(&world, &file, index, &mut info);
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
        let mut cx = Checker::new(&world, &file, index, &mut info);
        match decl {
            Declaration::Page(p) => {
                cx.push_scope();
                for param in &p.params {
                    let ty = cx.world.resolve(Type::from_ref(&param.prop_type));
                    cx.bind(&param.name, ty, param.span);
                }
                if let Some(layout) = &p.layout {
                    cx.current_span = layout.span;
                    cx.layout(layout);
                }
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
                    cx.bind(&prop.name, ty, prop.span);
                }
                cx.component = Some(c);
                cx.declare_hoisted(&c.body);
                cx.statements(&c.body, Body::Component);
                cx.pop_scope();
            }
            Declaration::App(a) => {
                cx.push_scope();
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
            Declaration::Store(_) | Declaration::Theme(_) | Declaration::Enum(_) => {}
        }
    }
    info
}

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
    /// The component being checked, for `emit`.
    component: Option<&'p ComponentDecl>,
    /// The `return` types met in the action being checked.
    returns: Vec<Vec<Type>>,
    /// The span of the statement or argument being checked: where an
    /// expression's error lands, since expressions carry no span.
    current_span: Span,
    /// Whether an element's arguments are being checked, where the
    /// resolver already reports a case the prop's enum lacks.
    in_args: bool,
}

impl<'a, 'p> Checker<'a, 'p> {
    fn new(world: &'a World<'p>, file: &'a str, decl: usize, info: &'a mut TypeInfo) -> Self {
        Self {
            world,
            file,
            decl,
            info,
            scopes: Vec::new(),
            component: None,
            returns: Vec::new(),
            current_span: Span::dummy(),
            in_args: false,
        }
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn bind(&mut self, name: &str, ty: Type, span: Span) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), ty.clone());
        }
        self.info.bindings.push(Typed {
            decl: self.decl,
            name: name.to_string(),
            span,
            ty,
        });
    }

    /// Narrow a name within the current scope, without recording a binding.
    fn narrow(&mut self, name: &str, ty: Type) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), ty);
        }
    }

    fn lookup(&self, name: &str) -> Option<Type> {
        self.scopes.iter().rev().find_map(|s| s.get(name).cloned())
    }

    fn error(&mut self, span: Span, code: &str, message: String, hint: &str) {
        let d = Diagnostic::new(
            format!("[{code}] {message}"),
            self.file,
            span.line as usize,
            span.col as usize,
        );
        self.info.findings.errors.push(if hint.is_empty() {
            d
        } else {
            d.with_hint(hint)
        });
    }

    /// Report `given` where `wanted` was expected, at `span`, naming `what`.
    fn expect(&mut self, given: &Type, wanted: &Type, span: Span, what: &str) {
        if given.assignable_to(wanted) {
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
                    decl.cases
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
    fn declare_hoisted(&mut self, stmts: &[Statement]) {
        // Two passes: state and resources first (their types are declared
        // or read off their values), then derived values and actions, whose
        // types may depend on them.
        for stmt in stmts {
            self.current_span = stmt.span;
            match &stmt.kind {
                StatementKind::State(s) => {
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
                        None => Type::Any,
                    };
                    self.bind(&r.name, Type::Resource(Box::new(inner)), stmt.span);
                }
                StatementKind::Use(u) => {
                    self.bind(&u.store_name, Type::Store(u.store_name.clone()), stmt.span);
                }
                _ => {}
            }
        }
        for stmt in stmts {
            self.current_span = stmt.span;
            match &stmt.kind {
                StatementKind::Action(a) => {
                    let params: Vec<Type> = a
                        .params
                        .iter()
                        .map(|p| self.world.resolve(Type::from_ref(&p.param_type)))
                        .collect();
                    // The return type is found when the body is checked.
                    self.bind(&a.name, Type::Func(params, Box::new(Type::Any)), stmt.span);
                }
                StatementKind::Derived(d) => {
                    let ty = self.infer(&d.value, None);
                    self.bind(&d.name, ty, stmt.span);
                }
                _ => {}
            }
        }
    }

    // ─── Statements ──────────────────────────────────────

    fn statements(&mut self, stmts: &[Statement], body: Body) {
        for stmt in stmts {
            self.statement(stmt, body);
        }
    }

    fn statement(&mut self, stmt: &Statement, body: Body) {
        let span = stmt.span;
        self.current_span = span;
        match &stmt.kind {
            StatementKind::State(s) => {
                let declared = s.ty.as_ref().map(|t| self.world.resolve(Type::from_ref(t)));
                let given = self.infer(&s.value, declared.as_ref());
                if let Some(declared) = &declared {
                    self.expect(&given, declared, span, &format!("`{}`", s.name));
                }
                if body == Body::Imperative {
                    // A local: bound here, in order.
                    self.bind(&s.name, declared.unwrap_or(given), span);
                }
            }
            StatementKind::Derived(d) => {
                let ty = self.infer(&d.value, None);
                self.narrow(&d.name, ty);
            }
            StatementKind::Resource(r) => {
                let url = self.infer(&r.url, Some(&Type::String));
                self.expect(&url, &Type::String, span, "a resource's URL");
                for option in &r.options {
                    self.infer(&option.value, None);
                }
            }
            StatementKind::Use(_) => {}
            StatementKind::Action(a) => {
                self.push_scope();
                for p in &a.params {
                    let ty = self.world.resolve(Type::from_ref(&p.param_type));
                    self.bind(&p.name, ty, span);
                }
                self.returns.push(Vec::new());
                self.statements(&a.body, Body::Imperative);
                let returned = self.returns.pop().unwrap_or_default();
                self.pop_scope();
                let ret = returned
                    .into_iter()
                    .reduce(Type::join)
                    .unwrap_or(Type::Null);
                let params: Vec<Type> = a
                    .params
                    .iter()
                    .map(|p| self.world.resolve(Type::from_ref(&p.param_type)))
                    .collect();
                let func = Type::Func(params, Box::new(ret));
                self.narrow(&a.name, func.clone());
                if let Some(t) =
                    self.info.bindings.iter_mut().rev().find(|t| {
                        t.decl == self.decl && t.name == a.name && t.span.start == span.start
                    })
                {
                    t.ty = func;
                }
            }
            StatementKind::Effect(e) => {
                self.push_scope();
                self.statements(&e.body, Body::Imperative);
                self.pop_scope();
            }
            StatementKind::EventHandler(h) => {
                self.push_scope();
                if let Some(param) = &h.param {
                    self.bind(param, Type::Any, h.span);
                }
                self.statements(&h.body, Body::Imperative);
                self.pop_scope();
            }
            StatementKind::UIElement(el) => self.element(el, span),
            StatementKind::If(i) => self.if_statement(i, span, body),
            StatementKind::For(f) => {
                let iterable = self.infer(&f.iterable, None);
                let item = match &iterable {
                    Type::List(inner) => (**inner).clone(),
                    Type::Any | Type::Map => Type::Any,
                    other => {
                        self.error(
                            span,
                            "T08",
                            format!(
                                "`for` loops over a list, but `{}` is `{other}`",
                                expr_text(&f.iterable)
                            ),
                            "Give it a list, or `.split(…)` a string first",
                        );
                        Type::Any
                    }
                };
                self.push_scope();
                self.bind(&f.item, item, span);
                if let Some(index) = &f.index {
                    self.bind(index, Type::Number, span);
                }
                if let Some(key) = &f.key {
                    self.infer(key, None);
                }
                self.statements(&f.body, body);
                self.pop_scope();
            }
            StatementKind::Show(s) => {
                let cond = self.infer(&s.condition, Some(&Type::Bool));
                self.condition(&cond, &s.condition, span, "show");
                self.statements(&s.body, body);
            }
            StatementKind::Match(m) => self.match_statement(m, span, body),
            StatementKind::Assignment(a) => {
                let target = match &a.target {
                    // An undeclared name is a plain variable; the compiler
                    // has always let it be.
                    Expr::Identifier(name) => self.lookup(name).unwrap_or(Type::Any),
                    other => self.infer(other, None),
                };
                let value = self.infer(&a.value, Some(&target));
                let what = format!("`{}`", expr_text(&a.target));
                self.expect(&value, &target, span, &what);
            }
            StatementKind::MethodCall(mc) => {
                self.infer(
                    &Expr::MethodCall(
                        Box::new(mc.object.clone()),
                        mc.method.clone(),
                        mc.args.clone(),
                    ),
                    None,
                );
            }
            StatementKind::ExprStatement(e) => {
                self.infer(e, None);
            }
            StatementKind::Navigate(e) => {
                let ty = self.infer(e, Some(&Type::String));
                self.expect(&ty, &Type::String, span, "`navigate`'s path");
            }
            StatementKind::Log(e) => {
                self.infer(e, None);
            }
            StatementKind::Return(e) => {
                let ty = match e {
                    Some(e) => self.infer(e, None),
                    None => Type::Null,
                };
                if let Some(returns) = self.returns.last_mut() {
                    returns.push(ty);
                }
            }
            StatementKind::Emit(e) => self.emit(e, span),
            StatementKind::Animate(_) => {}
            StatementKind::Fetch(f) => {
                self.infer(&f.url, Some(&Type::String));
                if let Some(b) = &f.loading_block {
                    self.statements(b, body);
                }
                if let Some((name, b)) = &f.error_block {
                    self.push_scope();
                    self.bind(name, Type::Any, span);
                    self.statements(b, body);
                    self.pop_scope();
                }
                if let Some(b) = &f.success_block {
                    self.push_scope();
                    self.bind(&f.variable, Type::Any, span);
                    self.statements(b, body);
                    self.pop_scope();
                }
            }
        }
    }

    /// A condition: a `Bool`, or anything JavaScript reads as one — a
    /// number, a string, a value that may be null — but not a list or a
    /// record, which are always true.
    fn condition(&mut self, ty: &Type, expr: &Expr, span: Span, keyword: &str) {
        match ty {
            Type::List(_) => self.error(
                span,
                "T07",
                format!(
                    "`{keyword}` reads `{}` as a condition, but a list is always true",
                    expr_text(expr)
                ),
                "Ask about its length: `items.length > 0`",
            ),
            Type::Record(name) => self.error(
                span,
                "T07",
                format!(
                    "`{keyword}` reads `{}` as a condition, but a `{name}` is always true",
                    expr_text(expr)
                ),
                "Compare one of its fields",
            ),
            Type::Func(..) => self.error(
                span,
                "T07",
                format!(
                    "`{keyword}` reads `{}` as a condition, but an action is always true",
                    expr_text(expr)
                ),
                "Call it: `{}()`",
            ),
            _ => {}
        }
    }

    fn if_statement(&mut self, i: &IfStmt, span: Span, body: Body) {
        self.push_scope();
        match &i.binding {
            Some(name) => {
                let ty = self.infer(&i.condition, None);
                self.bind(name, ty.unwrapped(), span);
            }
            None => {
                let ty = self.infer(&i.condition, Some(&Type::Bool));
                self.condition(&ty, &i.condition, span, "if");
                // `if x != null { }`, `if x { }`: `x` is not null inside.
                for name in narrowed_names(&i.condition) {
                    if let Some(Type::Optional(inner)) = self.lookup(&name) {
                        self.narrow(&name, *inner);
                    }
                }
            }
        }
        self.statements(&i.then_body, body);
        self.pop_scope();
        for (cond, branch) in &i.else_if_branches {
            self.push_scope();
            let ty = self.infer(cond, Some(&Type::Bool));
            self.condition(&ty, cond, span, "else if");
            for name in narrowed_names(cond) {
                if let Some(Type::Optional(inner)) = self.lookup(&name) {
                    self.narrow(&name, *inner);
                }
            }
            self.statements(branch, body);
            self.pop_scope();
        }
        if let Some(b) = &i.else_body {
            self.push_scope();
            self.statements(b, body);
            self.pop_scope();
        }
    }

    fn match_statement(&mut self, m: &MatchStmt, span: Span, body: Body) {
        let scrutinee = self.infer(&m.scrutinee, None);
        match &scrutinee {
            Type::Resource(inner) => {
                for arm in &m.arms {
                    self.push_scope();
                    match (&arm.pattern, &arm.binding) {
                        (ArmPattern::Ready, Some(name)) => {
                            self.bind(name, (**inner).clone(), arm.span)
                        }
                        (ArmPattern::Error, Some(name)) => self.bind(name, Type::Any, arm.span),
                        (ArmPattern::Case(case), _) => self.error(
                            arm.span,
                            "T11",
                            format!(
                                "`{}` is a resource; `.{case}` is not one of its states",
                                expr_text(&m.scrutinee)
                            ),
                            "A resource matches `loading`, `error(e)`, `ready(value)` and `else`",
                        ),
                        _ => {}
                    }
                    self.statements(&arm.body, body);
                    self.pop_scope();
                }
            }
            Type::Enum(name) => {
                let cases: Vec<String> = self
                    .world
                    .enums
                    .get(name.as_str())
                    .map(|e| e.cases.clone())
                    .unwrap_or_default();
                for arm in &m.arms {
                    match &arm.pattern {
                        ArmPattern::Case(case) if !cases.contains(case) => self.error(
                            arm.span,
                            "T02",
                            format!("`{name}` has no case `.{case}`"),
                            &format!(
                                "`{name}` takes {}",
                                cases.iter().map(|c| format!(".{c}")).collect::<Vec<_>>().join(", ")
                            ),
                        ),
                        ArmPattern::Loading | ArmPattern::Error | ArmPattern::Ready => self.error(
                            arm.span,
                            "T11",
                            format!(
                                "`{}` is a `{name}`, not a resource; it has no `loading`, `error` or `ready`",
                                expr_text(&m.scrutinee)
                            ),
                            "Match its cases: `.case { … }`",
                        ),
                        _ => {}
                    }
                    self.push_scope();
                    if let Some(name) = &arm.binding {
                        self.bind(name, Type::Any, arm.span);
                    }
                    self.statements(&arm.body, body);
                    self.pop_scope();
                }
            }
            Type::Any | Type::Case(_) => {
                for arm in &m.arms {
                    self.push_scope();
                    if let Some(name) = &arm.binding {
                        self.bind(name, Type::Any, arm.span);
                    }
                    self.statements(&arm.body, body);
                    self.pop_scope();
                }
            }
            other => {
                self.error(
                    span,
                    "T11",
                    format!(
                        "`match` needs a resource or an enum, but `{}` is `{other}`",
                        expr_text(&m.scrutinee)
                    ),
                    "Use `if` for a condition; `match` chooses among a resource's states or an enum's cases",
                );
                for arm in &m.arms {
                    self.push_scope();
                    if let Some(name) = &arm.binding {
                        self.bind(name, Type::Any, arm.span);
                    }
                    self.statements(&arm.body, body);
                    self.pop_scope();
                }
            }
        }
    }

    fn emit(&mut self, e: &EmitStmt, span: Span) {
        let Some(component) = self.component else {
            return;
        };
        let Some(event) = component.events.iter().find(|ev| ev.name == e.event) else {
            return; // sema reports the undeclared event
        };
        if event.params.len() != e.args.len() {
            self.error(
                span,
                "T09",
                format!(
                    "`emit {}` passes {} argument{}, but the event takes {}",
                    e.event,
                    e.args.len(),
                    if e.args.len() == 1 { "" } else { "s" },
                    event.params.len()
                ),
                &format!(
                    "`event {}({})`",
                    event.name,
                    event
                        .params
                        .iter()
                        .map(|p| format!("{}: {}", p.name, Type::from_ref(&p.param_type)))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            );
        }
        for (arg, param) in e.args.iter().zip(&event.params) {
            let wanted = self.world.resolve(Type::from_ref(&param.param_type));
            let given = self.infer(arg, Some(&wanted));
            self.expect(
                &given,
                &wanted,
                span,
                &format!("`{}` of `emit {}`", param.name, e.event),
            );
        }
    }

    fn layout(&mut self, layout: &LayoutRef) {
        let Some(component) = self.world.components.get(layout.name.as_str()) else {
            return;
        };
        self.component_args(&layout.args, &[], component, layout.span);
    }

    // ─── Elements ────────────────────────────────────────

    fn element(&mut self, el: &UIElement, span: Span) {
        match &el.component {
            ComponentRef::UserDefined(name) => {
                if let Some(component) = self.world.components.get(name.as_str()) {
                    self.component_args(&el.args, &el.arg_spans, component, span);
                } else {
                    for arg in &el.args {
                        self.infer(arg_value(arg), None);
                    }
                }
            }
            ComponentRef::BuiltIn(name) => {
                let sig = registry::component(name);
                self.builtin_args(el, sig, span);
            }
            ComponentRef::SubComponent(owner, part) => {
                let sig = registry::part(owner, part);
                self.builtin_args(el, sig, span);
            }
        }
        if let Some(style) = &el.style_block {
            for prop in &style.properties {
                self.current_span = prop.span;
                self.infer(&prop.value, None);
            }
        }
        self.current_span = span;
        for handler in &el.events {
            self.push_scope();
            if let Some(param) = &handler.param {
                // A declared event's handler receives its first parameter;
                // a DOM handler receives the event.
                let ty = match &el.component {
                    ComponentRef::UserDefined(name) => self
                        .world
                        .components
                        .get(name.as_str())
                        .and_then(|c| c.events.iter().find(|e| e.name == handler.event))
                        .and_then(|e| e.params.first())
                        .map(|p| self.world.resolve(Type::from_ref(&p.param_type)))
                        .unwrap_or(Type::Any),
                    _ => Type::Any,
                };
                self.bind(param, ty, handler.span);
            }
            self.statements(&handler.body, Body::Imperative);
            self.pop_scope();
        }
        for fill in &el.slot_fills {
            self.statements(&fill.body, Body::Page);
        }
        self.statements(&el.children, Body::Page);
    }

    /// The arguments of a call to a user component, against its props.
    fn component_args(
        &mut self,
        args: &[Arg],
        spans: &[Span],
        component: &ComponentDecl,
        span: Span,
    ) {
        self.in_args = true;
        for (i, arg) in args.iter().enumerate() {
            let at = spans.get(i).copied().unwrap_or(span);
            self.current_span = at;
            let prop = match arg {
                Arg::Positional(_) => component
                    .props
                    .iter()
                    .find(|p| p.positional)
                    .or(component.props.first()),
                Arg::Named(key, _) => component.props.iter().find(|p| &p.name == key),
            };
            let value = arg_value(arg);
            let Some(prop) = prop else {
                self.infer(value, None);
                continue;
            };
            let mut wanted = self.world.resolve(Type::from_ref(&prop.prop_type));
            if prop.optional {
                wanted = Type::optional(wanted);
            }
            let given = self.infer(value, Some(&wanted));
            self.expect(
                &given,
                &wanted,
                at,
                &format!("`{}` of `{}`", prop.name, component.name),
            );
        }
        self.in_args = false;
    }

    /// The arguments of a built-in, against the registry's prop types.
    fn builtin_args(
        &mut self,
        el: &UIElement,
        sig: Option<&'static registry::ComponentSig>,
        span: Span,
    ) {
        let name = match &el.component {
            ComponentRef::BuiltIn(n) => n.clone(),
            ComponentRef::SubComponent(o, p) => format!("{o}.{p}"),
            ComponentRef::UserDefined(n) => n.clone(),
        };
        self.in_args = true;
        for (i, arg) in el.args.iter().enumerate() {
            let at = el.arg_spans.get(i).copied().unwrap_or(span);
            self.current_span = at;
            let prop = match (arg, sig) {
                (Arg::Positional(_), Some(sig)) => sig.positional.as_ref(),
                (Arg::Named(key, _), Some(sig)) => sig.prop(key),
                _ => None,
            };
            let value = arg_value(arg);
            let Some(prop) = prop else {
                self.infer(value, None);
                continue;
            };
            match (prop.ty, prop.name) {
                // `bind:` — the control's value type.
                (PropType::State, "bind") => {
                    let wanted = bound_type(&name);
                    let given = self.infer(value, wanted.as_ref());
                    if let Some(wanted) = wanted {
                        // A state that starts as `null` may hold the value later.
                        let given = match given {
                            Type::Optional(inner) => *inner,
                            other => other,
                        };
                        self.expect(&given, &wanted, at, &format!("`bind:` on `{name}`"));
                    }
                }
                (PropType::Str | PropType::Path, _) => {
                    let given = self.infer(value, Some(&Type::String));
                    // Text-like props take a number or a bool as text.
                    if !matches!(given, Type::Number | Type::Bool) {
                        self.expect(
                            &given,
                            &Type::String,
                            at,
                            &format!("`{}:` on `{name}`", prop.name),
                        );
                    }
                }
                (PropType::Num, _) => {
                    let given = self.infer(value, Some(&Type::Number));
                    self.expect(
                        &given,
                        &Type::Number,
                        at,
                        &format!("`{}:` on `{name}`", prop.name),
                    );
                }
                (PropType::Bool, _) => {
                    let given = self.infer(value, Some(&Type::Bool));
                    self.expect(
                        &given,
                        &Type::Bool,
                        at,
                        &format!("`{}:` on `{name}`", prop.name),
                    );
                }
                _ => {
                    self.infer(value, None);
                }
            }
        }
        self.in_args = false;
    }

    // ─── Expressions ─────────────────────────────────────

    /// The type of `expr`; `expected` guides a case or a lambda.
    fn infer(&mut self, expr: &Expr, expected: Option<&Type>) -> Type {
        match expr {
            Expr::StringLiteral(_) => Type::String,
            Expr::InterpolatedString(parts) => {
                for part in parts {
                    if let StringPart::Expression(e) = part {
                        self.infer(e, None);
                    }
                }
                Type::String
            }
            Expr::NumberLiteral(_) => Type::Number,
            Expr::BoolLiteral(_) => Type::Bool,
            Expr::Null => Type::Null,
            Expr::Token(_) => Type::Token,
            Expr::EnumCase(case) => match expected.map(|t| t.unwrapped()) {
                Some(Type::Enum(name)) => {
                    let known = self.world.enums.get(name.as_str()).map(|e| e.cases.clone());
                    if let Some(cases) = known
                        && !cases.contains(case)
                        && !self.in_args
                    {
                        self.error_at_current(
                            "T02",
                            format!("`{name}` has no case `.{case}`"),
                            &format!(
                                "`{name}` takes {}",
                                cases
                                    .iter()
                                    .map(|c| format!(".{c}"))
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            ),
                        );
                    }
                    Type::Enum(name)
                }
                _ => Type::Case(case.clone()),
            },
            Expr::Identifier(name) => self.lookup(name).unwrap_or_else(|| global_type(name)),
            Expr::PropertyAccess(base, field) => {
                let base_ty = self.infer(base, None);
                self.property(&base_ty, base, field)
            }
            Expr::IndexAccess(base, index) => {
                let base_ty = self.infer(base, None);
                self.infer(index, None);
                match base_ty.unwrapped() {
                    Type::List(inner) => *inner,
                    Type::String => Type::String,
                    _ => Type::Any,
                }
            }
            Expr::BinaryOp(l, op, r) => {
                let lt = self.infer(l, None);
                let rt = self.infer(r, None);
                match op {
                    BinOp::Add => {
                        if lt.unwrapped() == Type::String || rt.unwrapped() == Type::String {
                            Type::String
                        } else if lt == Type::Number && rt == Type::Number {
                            Type::Number
                        } else {
                            Type::Any
                        }
                    }
                    BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod => Type::Number,
                    BinOp::Eq | BinOp::Neq | BinOp::Lt | BinOp::Gt | BinOp::Lte | BinOp::Gte => {
                        Type::Bool
                    }
                    BinOp::And | BinOp::Or => Type::Bool,
                    BinOp::NullCoalesce => Type::join(lt.unwrapped(), rt),
                }
            }
            Expr::UnaryOp(op, e) => {
                self.infer(e, None);
                match op {
                    UnaryOp::Not => Type::Bool,
                    UnaryOp::Neg => Type::Number,
                }
            }
            Expr::MethodCall(obj, method, args) => self.method_call(obj, method, args),
            Expr::FunctionCall(name, args) => self.function_call(name, args),
            Expr::ListLiteral(items) => {
                let expected_item = match expected {
                    Some(Type::List(inner)) => Some((**inner).clone()),
                    _ => None,
                };
                let mut item = None;
                for e in items {
                    let ty = self.infer(e, expected_item.as_ref());
                    item = Some(match item {
                        None => ty,
                        Some(prev) => Type::join(prev, ty),
                    });
                }
                Type::list(item.unwrap_or(Type::Any))
            }
            Expr::MapLiteral(pairs) => {
                for (_, v) in pairs {
                    self.infer(v, None);
                }
                Type::Map
            }
            Expr::Record(name, fields) => {
                if let Some(decl) = self.world.types.get(name.as_str()) {
                    for (key, value) in fields {
                        match decl.fields.iter().find(|f| &f.name == key) {
                            Some(field) => {
                                let wanted = self.world.resolve(Type::from_ref(&field.ty));
                                self.infer(value, Some(&wanted));
                            }
                            None => {
                                self.infer(value, None);
                            }
                        };
                    }
                    Type::Record(name.clone())
                } else if self.world.enums.contains_key(name.as_str()) {
                    Type::Enum(name.clone())
                } else {
                    for (_, v) in fields {
                        self.infer(v, None);
                    }
                    Type::Any
                }
            }
            Expr::Lambda(params, body) => {
                let names: Vec<&str> = params.split(',').map(str::trim).collect();
                let param_types: Vec<Type> = match expected {
                    Some(Type::Func(types, _)) => names
                        .iter()
                        .enumerate()
                        .map(|(i, _)| types.get(i).cloned().unwrap_or(Type::Any))
                        .collect(),
                    _ => names.iter().map(|_| Type::Any).collect(),
                };
                self.push_scope();
                for (name, ty) in names.iter().zip(&param_types) {
                    self.narrow(name, ty.clone());
                }
                let ret = self.infer(body, None);
                self.pop_scope();
                Type::Func(param_types, Box::new(ret))
            }
            Expr::Await(e) => {
                let ty = self.infer(e, None);
                match ty {
                    Type::Resource(inner) => *inner,
                    other => other,
                }
            }
        }
    }

    fn property(&mut self, base_ty: &Type, base: &Expr, field: &str) -> Type {
        match base_ty {
            Type::Record(name) => match self.world.record_field(name, field) {
                Some(ty) => self.world.resolve(ty),
                None => {
                    let fields: Vec<String> = self
                        .world
                        .types
                        .get(name.as_str())
                        .map(|t| t.fields.iter().map(|f| format!("`{}`", f.name)).collect())
                        .unwrap_or_default();
                    self.error_at_current(
                        "T05",
                        format!("`{name}` has no field `{field}`"),
                        &format!("Its fields are {}", fields.join(", ")),
                    );
                    Type::Any
                }
            },
            Type::Store(name) => match self.world.stores.get(name.as_str()) {
                Some(members) => match members.get(field) {
                    Some(ty) => ty.clone(),
                    None => {
                        let mut names: Vec<&String> = members.keys().collect();
                        names.sort();
                        self.error_at_current(
                            "T06",
                            format!("`{name}` has no member `{field}`"),
                            &format!(
                                "Its members are {}",
                                names
                                    .iter()
                                    .map(|n| format!("`{n}`"))
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            ),
                        );
                        Type::Any
                    }
                },
                None => Type::Any,
            },
            Type::Optional(inner) => {
                self.error_at_current(
                    "T04",
                    format!(
                        "`{}` may be null, so `.{field}` may fail",
                        expr_text(base)
                    ),
                    "Unwrap it first: `if let x = value { … }`, `value ?? fallback`, or a check for `!= null`",
                );
                self.property(inner, base, field)
            }
            Type::List(_) => match field {
                "length" => Type::Number,
                _ => Type::Any,
            },
            Type::String => match field {
                "length" => Type::Number,
                _ => Type::Any,
            },
            Type::Resource(inner) => match field {
                "data" => Type::optional((**inner).clone()),
                "state" => Type::String,
                "error" => Type::optional(Type::Any),
                _ => Type::Any,
            },
            _ => Type::Any,
        }
    }

    fn method_call(&mut self, obj: &Expr, method: &str, args: &[Expr]) -> Type {
        let obj_ty = self.infer(obj, None);
        if let Type::Optional(_) = obj_ty {
            self.error_at_current(
                "T04",
                format!("`{}` may be null, so `.{method}()` may fail", expr_text(obj)),
                "Unwrap it first: `if let x = value { … }`, `value ?? fallback`, or a check for `!= null`",
            );
        }
        let obj_ty = obj_ty.unwrapped();
        match &obj_ty {
            Type::Store(name) => {
                let member = self
                    .world
                    .stores
                    .get(name.as_str())
                    .and_then(|m| m.get(method).cloned());
                match member {
                    Some(Type::Func(params, ret)) => {
                        self.call_args(&params, args, &format!("`{name}.{method}`"));
                        *ret
                    }
                    Some(_) => {
                        for a in args {
                            self.infer(a, None);
                        }
                        Type::Any
                    }
                    None => {
                        if self.world.stores.contains_key(name.as_str()) {
                            self.error_at_current(
                                "T06",
                                format!("`{name}` has no action `{method}`"),
                                "",
                            );
                        }
                        for a in args {
                            self.infer(a, None);
                        }
                        Type::Any
                    }
                }
            }
            Type::List(item) => {
                let item = (**item).clone();
                let lambda =
                    |ret: Type| Type::Func(vec![item.clone(), Type::Number], Box::new(ret));
                match method {
                    "map" => {
                        let f = self.infer_arg(args, 0, Some(&lambda(Type::Any)));
                        let ret = match f {
                            Type::Func(_, ret) => *ret,
                            _ => Type::Any,
                        };
                        Type::list(ret)
                    }
                    "filter" | "slice" | "concat" | "reverse" | "sort" => {
                        for (i, _) in args.iter().enumerate() {
                            self.infer_arg(args, i, Some(&lambda(Type::Bool)));
                        }
                        Type::list(item)
                    }
                    "find" => {
                        self.infer_arg(args, 0, Some(&lambda(Type::Bool)));
                        Type::optional(item)
                    }
                    "some" | "every" | "includes" => {
                        self.infer_arg(args, 0, Some(&lambda(Type::Bool)));
                        Type::Bool
                    }
                    "findIndex" | "indexOf" | "push" | "sum" => {
                        for (i, _) in args.iter().enumerate() {
                            self.infer_arg(args, i, Some(&lambda(Type::Bool)));
                        }
                        Type::Number
                    }
                    "join" => {
                        self.infer_arg(args, 0, None);
                        Type::String
                    }
                    "reduce" => {
                        let init = self.infer_arg(args, 1, None);
                        self.infer_arg(
                            args,
                            0,
                            Some(&Type::Func(
                                vec![init.clone(), item],
                                Box::new(init.clone()),
                            )),
                        );
                        init
                    }
                    "toString" => Type::String,
                    _ => {
                        for a in args {
                            self.infer(a, None);
                        }
                        Type::Any
                    }
                }
            }
            Type::String => {
                for a in args {
                    self.infer(a, None);
                }
                match method {
                    "toLowerCase" | "toUpperCase" | "trim" | "replace" | "slice" | "substring"
                    | "charAt" | "toString" | "padStart" | "padEnd" | "repeat" => Type::String,
                    "indexOf" | "length" | "charCodeAt" => Type::Number,
                    "includes" | "startsWith" | "endsWith" => Type::Bool,
                    "split" => Type::list(Type::String),
                    _ => Type::Any,
                }
            }
            Type::Number => {
                for a in args {
                    self.infer(a, None);
                }
                match method {
                    "toFixed" | "toString" => Type::String,
                    _ => Type::Any,
                }
            }
            Type::Record(name) => {
                for a in args {
                    self.infer(a, None);
                }
                self.error_at_current(
                    "T05",
                    format!("`{name}` has no method `{method}`; a record holds fields"),
                    "",
                );
                Type::Any
            }
            _ => {
                for a in args {
                    self.infer(a, None);
                }
                Type::Any
            }
        }
    }

    fn infer_arg(&mut self, args: &[Expr], i: usize, expected: Option<&Type>) -> Type {
        match args.get(i) {
            Some(e) => self.infer(e, expected),
            None => Type::Any,
        }
    }

    fn function_call(&mut self, name: &str, args: &[Expr]) -> Type {
        match self.lookup(name) {
            Some(Type::Func(params, ret)) => {
                self.call_args(&params, args, &format!("`{name}`"));
                *ret
            }
            Some(_) => {
                for a in args {
                    self.infer(a, None);
                }
                Type::Any
            }
            None => {
                for a in args {
                    self.infer(a, None);
                }
                match name {
                    "String" | "t" => Type::String,
                    "Number" => Type::Number,
                    "Bool" | "Boolean" => Type::Bool,
                    _ => Type::Any,
                }
            }
        }
    }

    fn call_args(&mut self, params: &[Type], args: &[Expr], what: &str) {
        if params.len() != args.len() {
            self.error_at_current(
                "T10",
                format!(
                    "{what} takes {} argument{}, but {} {} given",
                    params.len(),
                    if params.len() == 1 { "" } else { "s" },
                    args.len(),
                    if args.len() == 1 { "is" } else { "are" }
                ),
                "",
            );
        }
        for (i, arg) in args.iter().enumerate() {
            let wanted = params.get(i).cloned().unwrap_or(Type::Any);
            let given = self.infer(arg, Some(&wanted));
            if !given.assignable_to(&wanted) {
                self.error_at_current(
                    "T01",
                    format!(
                        "argument {} of {what} is `{given}`, but `{wanted}` is wanted",
                        i + 1
                    ),
                    "",
                );
            }
        }
    }

    /// An error at the statement or argument being checked. Expressions
    /// carry no span of their own; the nearest enclosing one is used.
    fn error_at_current(&mut self, code: &str, message: String, hint: &str) {
        let span = self.current_span;
        self.error(span, code, message, hint);
    }
}

/// The value an argument carries.
fn arg_value(arg: &Arg) -> &Expr {
    match arg {
        Arg::Positional(e) | Arg::Named(_, e) => e,
    }
}

/// The type a control's `bind:` state must hold.
fn bound_type(component: &str) -> Option<Type> {
    match component {
        "Checkbox" | "Switch" => Some(Type::Bool),
        "Slider" => Some(Type::Number),
        "Input" => None, // text or number, by its `type`
        _ => None,
    }
}

/// The names a condition proves not null in its branch: `x`, `x != null`,
/// `x && …`.
fn narrowed_names(cond: &Expr) -> Vec<String> {
    match cond {
        Expr::Identifier(name) => vec![name.clone()],
        Expr::BinaryOp(l, BinOp::Neq, r) => match (&**l, &**r) {
            (Expr::Identifier(name), Expr::Null) | (Expr::Null, Expr::Identifier(name)) => {
                vec![name.clone()]
            }
            _ => Vec::new(),
        },
        Expr::BinaryOp(l, BinOp::And, r) => {
            let mut names = narrowed_names(l);
            names.extend(narrowed_names(r));
            names
        }
        _ => Vec::new(),
    }
}

/// The types of the names every program can read.
fn global_type(name: &str) -> Type {
    match name {
        "event" | "params" | "window" | "document" | "console" | "localStorage"
        | "sessionStorage" | "JSON" | "Math" | "Date" | "navigator" | "location" | "fetch"
        | "setTimeout" | "clearTimeout" | "setInterval" | "clearInterval" | "Object" | "Array"
        | "Promise" | "locale" | "dir" => Type::Any,
        _ => Type::Any,
    }
}

/// A short rendering of an expression, for a message.
pub fn expr_text(expr: &Expr) -> String {
    match expr {
        Expr::StringLiteral(s) => format!("\"{s}\""),
        Expr::InterpolatedString(_) => "\"…\"".to_string(),
        Expr::NumberLiteral(n) => format!("{n}"),
        Expr::BoolLiteral(b) => b.to_string(),
        Expr::Null => "null".to_string(),
        Expr::Identifier(n) => n.clone(),
        Expr::PropertyAccess(b, f) => format!("{}.{f}", expr_text(b)),
        Expr::IndexAccess(b, i) => format!("{}[{}]", expr_text(b), expr_text(i)),
        Expr::BinaryOp(l, _, r) => format!("{} … {}", expr_text(l), expr_text(r)),
        Expr::UnaryOp(_, e) => format!("!{}", expr_text(e)),
        Expr::MethodCall(o, m, _) => format!("{}.{m}(…)", expr_text(o)),
        Expr::FunctionCall(n, _) => format!("{n}(…)"),
        Expr::ListLiteral(_) => "[…]".to_string(),
        Expr::MapLiteral(_) => "{…}".to_string(),
        Expr::Record(n, _) => format!("{n}(…)"),
        Expr::Lambda(p, _) => format!("{p} => …"),
        Expr::EnumCase(c) => format!(".{c}"),
        Expr::Token(t) => format!("${t}"),
        Expr::Await(e) => format!("await {}", expr_text(e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::v2::parse_v2;

    fn errors(src: &str) -> Vec<String> {
        let program = parse_v2(src, "<t>").expect("parse");
        let info = check(&program, &|_| "<t>".to_string());
        info.findings
            .errors
            .iter()
            .map(|d| match &d.hint {
                Some(h) => format!("{}\n  {h}", d.message),
                None => d.message.clone(),
            })
            .collect()
    }

    fn clean(src: &str) {
        let found = errors(src);
        assert!(found.is_empty(), "{found:#?}");
    }

    fn has(src: &str, needle: &str) {
        let found = errors(src);
        assert!(
            found.iter().any(|e| e.contains(needle)),
            "no error containing {needle:?} in {found:#?}"
        );
    }

    fn type_of(src: &str, name: &str) -> String {
        let program = parse_v2(src, "<t>").expect("parse");
        let info = check(&program, &|_| "<t>".to_string());
        info.bindings
            .iter()
            .rev()
            .find(|b| b.name == name)
            .map(|b| b.ty.to_string())
            .unwrap_or_else(|| panic!("{name} has no type; bindings: {:?}", info.bindings))
    }

    const TODOS: &str = "enum Tone { calm, loud }\ntype Todo { id: String, title: String, done: Bool = false, tone: Tone = .calm, note: String? = null }\n";

    #[test]
    fn declared_and_inferred_types_of_names() {
        let src = format!(
            "{TODOS}store Todos {{\n  state items: [Todo] = []\n  state draft = \"\"\n  state n = 0\n  state sel = null\n  derived remaining = items.filter(t => !t.done).length\n  derived titles = items.map(t => t.title)\n  derived first = items.find(t => t.done)\n  action add(title: String) {{ items = items.concat([Todo(id: \"1\", title: title)]) }}\n  action count() {{ return items.length }}\n}}\npage P(path: \"/\", id: String) {{\n  use Todos\n  resource rows: [Todo] = fetch(\"/x\")\n  state t = Todo(id: \"a\", title: \"b\")\n  for todo in Todos.items by todo.id {{ Text(todo.title) }}\n  match rows {{ ready(list) {{ for r in list {{ Text(r.title) }} }} else {{ }} }}\n}}"
        );
        assert_eq!(type_of(&src, "items"), "[Todo]");
        assert_eq!(type_of(&src, "draft"), "String");
        assert_eq!(type_of(&src, "n"), "Number");
        assert_eq!(type_of(&src, "sel"), "Any?");
        assert_eq!(type_of(&src, "remaining"), "Number");
        assert_eq!(type_of(&src, "titles"), "[String]");
        assert_eq!(type_of(&src, "first"), "Todo?");
        assert_eq!(type_of(&src, "add"), "action(String)");
        assert_eq!(type_of(&src, "count"), "action() -> Number");
        assert_eq!(type_of(&src, "id"), "String");
        assert_eq!(type_of(&src, "rows"), "resource<[Todo]>");
        assert_eq!(type_of(&src, "t"), "Todo");
        assert_eq!(type_of(&src, "todo"), "Todo");
        assert_eq!(type_of(&src, "list"), "[Todo]");
        assert_eq!(type_of(&src, "r"), "Todo");
        clean(&src);
    }

    #[test]
    fn a_value_of_the_wrong_type_is_reported_where_it_is_given() {
        has(
            &format!("{TODOS}page P(path: \"/\") {{ state n: Number = \"x\" }}"),
            "[T01] `n` is `String`, but `Number` is wanted",
        );
        has(
            &format!(
                "{TODOS}component C(count: Number) {{ Text(count) }}\npage P(path: \"/\") {{ C(count: \"3\") }}"
            ),
            "`count` of `C` is `String`, but `Number` is wanted",
        );
        has(
            &format!(
                "{TODOS}component C(_ label: String) {{ Text(label) }}\npage P(path: \"/\") {{ C(3) }}"
            ),
            "`label` of `C` is `Number`, but `String` is wanted",
        );
        has(
            &format!(
                "{TODOS}page P(path: \"/\") {{ state x = Todo(id: \"a\", title: \"b\")\n Text(x.title) }}\ncomponent D(todo: Todo) {{ Text(todo.title) }}\npage Q(path: \"/q\") {{ state s = \"\"\n D(todo: s) }}"
            ),
            "`todo` of `D` is `String`, but `Todo` is wanted",
        );
        has(
            &format!("{TODOS}type Row {{ n: Number = \"no\" }}"),
            "the default of `n` is `String`, but `Number` is wanted",
        );
        has(
            &format!(
                "{TODOS}page P(path: \"/\") {{ state n = 1\n Button(\"x\") {{ on click {{ n = \"two\" }} }} }}"
            ),
            "`n` is `String`, but `Number` is wanted",
        );
    }

    #[test]
    fn enum_cases_are_checked_against_their_enum() {
        // A case an element argument gets wrong is the resolver's to report;
        // everywhere else it is the checker's.
        clean(&format!(
            "{TODOS}component C(tone: Tone) {{ Text(\"x\") }}\npage P(path: \"/\") {{ C(tone: .angry) }}"
        ));
        has(
            &format!("{TODOS}page P(path: \"/\") {{ state t: Tone = .angry }}"),
            "[T02] `Tone` has no case `.angry`",
        );
        clean(&format!(
            "{TODOS}component C(tone: Tone = .calm) {{ Text(\"x\") }}\npage P(path: \"/\") {{ C(tone: .loud)\n state t: Tone = .calm\n match t {{ .calm {{ Text(\"a\") }} else {{ Text(\"b\") }} }} }}"
        ));
        has(
            &format!(
                "{TODOS}page P(path: \"/\") {{ state t: Tone = .calm\n match t {{ .angry {{ Text(\"a\") }} else {{ }} }} }}"
            ),
            "[T02] `Tone` has no case `.angry`",
        );
    }

    #[test]
    fn fields_members_and_methods_that_do_not_exist() {
        has(
            &format!(
                "{TODOS}page P(path: \"/\") {{ state t = Todo(id: \"a\", title: \"b\")\n Text(t.name) }}"
            ),
            "[T05] `Todo` has no field `name`",
        );
        has(
            "store S { state n = 0 }\npage P(path: \"/\") { use S\n Text(S.count) }",
            "[T06] `S` has no member `count`",
        );
        has(
            "store S { state n = 0 action bump() { n = n + 1 } }\npage P(path: \"/\") { use S\n Button(\"x\") { on click { S.reset() } } }",
            "[T06] `S` has no action `reset`",
        );
        clean(
            "store S { state n = 0 action bump() { n = n + 1 } }\npage P(path: \"/\") { use S\n Text(\"{S.n}\")\n Button(\"x\") { on click { S.bump() } } }",
        );
    }

    #[test]
    fn a_value_that_may_be_null_must_be_unwrapped() {
        has(
            &format!(
                "{TODOS}page P(path: \"/\") {{ state t = Todo(id: \"a\", title: \"b\")\n Text(t.note.length) }}"
            ),
            "[T04] `t.note` may be null",
        );
        clean(&format!(
            "{TODOS}page P(path: \"/\") {{ state t = Todo(id: \"a\", title: \"b\")\n state sel: Todo? = null\n if let n = t.note {{ Text(n.length) }}\n Text(t.note ?? \"none\")\n if sel != null {{ Text(sel.title) }}\n if sel {{ Text(sel.title) }}\n if sel && t.done {{ Text(sel.title) }} }}"
        ));
        has(
            &format!("{TODOS}page P(path: \"/\") {{ state sel: Todo? = null\n Text(sel.title) }}"),
            "[T04] `sel` may be null, so `.title` may fail",
        );
    }

    #[test]
    fn calls_and_emits_take_the_declared_arguments() {
        has(
            "page P(path: \"/\") { action add(title: String, n: Number) { }\n Button(\"x\") { on click { add(\"a\") } } }",
            "[T10] `add` takes 2 arguments, but 1 is given",
        );
        has(
            "page P(path: \"/\") { action add(n: Number) { }\n Button(\"x\") { on click { add(\"a\") } } }",
            "argument 1 of `add` is `String`, but `Number` is wanted",
        );
        has(
            "component C { event pick(id: String)\n Button(\"x\") { on click { emit pick(1, 2) } } }",
            "[T09] `emit pick` passes 2 arguments, but the event takes 1",
        );
        has(
            "component C { event pick(id: String)\n Button(\"x\") { on click { emit pick(1) } } }",
            "`id` of `emit pick` is `Number`, but `String` is wanted",
        );
        clean(
            "component C { event pick(id: String)\n Button(\"x\") { on click { emit pick(\"a\") } } }\npage P(path: \"/\") { C { on pick(id) { log(id.length) } } }",
        );
    }

    #[test]
    fn controls_bind_state_of_their_own_kind() {
        has(
            "page P(path: \"/\") { state name = \"\"\n Checkbox(bind: name, label: \"x\") }",
            "`bind:` on `Checkbox` is `String`, but `Bool` is wanted",
        );
        has(
            "page P(path: \"/\") { state on = true\n Slider(bind: on) }",
            "`bind:` on `Slider` is `Bool`, but `Number` is wanted",
        );
        clean(
            "page P(path: \"/\") { state on = true\n state n = 0\n state s = \"\"\n Checkbox(bind: on, label: \"x\")\n Slider(bind: n)\n Input(bind: s, label: \"y\")\n Switch(bind: on, label: \"z\") }",
        );
    }

    #[test]
    fn loops_conditions_and_matches_need_the_right_shapes() {
        has(
            "page P(path: \"/\") { state name = \"x\"\n for c in name { Text(c) } }",
            "[T08] `for` loops over a list, but `name` is `String`",
        );
        has(
            "page P(path: \"/\") { state items = [1]\n if items { Text(\"some\") } }",
            "[T07] `if` reads `items` as a condition, but a list is always true",
        );
        has(
            "page P(path: \"/\") { state n = 1\n match n { else { Text(\"x\") } } }",
            "[T11] `match` needs a resource or an enum, but `n` is `Number`",
        );
        has(
            "page P(path: \"/\") { resource r = fetch(\"/x\")\n match r { .calm { } else { } } }",
            "`r` is a resource; `.calm` is not one of its states",
        );
        clean(
            "page P(path: \"/\") { state items = [1]\n state n = 0\n state s = \"\"\n if items.length > 0 { Text(\"some\") }\n if n { Text(\"n\") }\n if s { Text(\"s\") }\n show n > 1 { Text(\"x\") } }",
        );
    }

    #[test]
    fn the_layout_and_the_universal_props_are_typed_too() {
        has(
            "component Shell(crumb: String) { slot  children }\npage P(path: \"/\", layout: Shell(crumb: 3)) { Text(\"x\") }",
            "`crumb` of `Shell` is `Number`, but `String` is wanted",
        );
        clean(
            "component Shell(crumb: String) { slot  children }\npage P(path: \"/\", layout: Shell(crumb: \"Home\")) { Text(\"x\")\n Card(exit: .fadeOut, delay: \"100ms\").fadeIn { Text(\"y\") } }",
        );
    }

    #[test]
    fn untyped_programs_check_as_they_always_did() {
        clean(
            "store S { state items = [] state q = \"\" action add(item: Map) { items.push(item) } derived hits = items.filter(i => i.name.includes(q)) }\npage P(path: \"/\") { use S\n state user = null\n derived h = { a: 1 }\n Button(\"x\") { on click { user = { name: \"x\" }  S.add({ name: q })  h.a = 2  log(JSON.stringify(user))  localStorage.setItem(\"k\", \"v\") } }\n if user { Text(user.name) }\n for it in S.hits { Text(it.name) }\n Text(\"{S.q.length}\") }",
        );
    }
}
