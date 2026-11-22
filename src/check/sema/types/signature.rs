//! What another declaration sees is declared; what only the body sees is
//! inferred. A `state` or `persist` is inferred only from a value that
//! says everything — `0`, `"x"`, `.calm`, `Todo(…)`, `[1, 2]` — and an
//! empty list, an empty map, `null` or open JSON says nothing, so it is
//! declared (`T25`, with the type the body later gives it as its fix). A
//! parameter nothing gives a type to is declared too (`T32`).

use std::collections::HashMap;

use super::{Checker, Span, Type};
use crate::diagnostics::fixes::Plan;

/// Whether a state's first value leaves its type unsaid: unknown anywhere
/// in it, or JSON, which could be anything.
pub(super) fn says_nothing(ty: &Type) -> bool {
    fn unknown(ty: &Type) -> bool {
        match ty {
            Type::Any | Type::Map => true,
            Type::Shape(fields) if fields.is_empty() => true,
            Type::List(t) | Type::Optional(t) | Type::Promise(t) => unknown(t),
            Type::Dict(k, v) => unknown(k) || unknown(v),
            Type::Shape(fields) => fields.iter().any(|(_, t)| unknown(t)),
            _ => false,
        }
    }
    let open = matches!(ty.unwrapped(), Type::Enum(n) if n == super::ty::JSON);
    unknown(ty) || open
}

/// The state a list is worked out from — `items` in `items.filter(…)`,
/// `items.slice().sort(…)` or `items` itself — when each step keeps the
/// items as they are.
fn list_root(e: &super::Expr) -> Option<&str> {
    use super::Expr;
    match e {
        Expr::Identifier(n) => Some(n),
        Expr::MethodCall(obj, method, _)
            if matches!(
                method.as_str(),
                "filter"
                    | "slice"
                    | "sort"
                    | "sortBy"
                    | "reverse"
                    | "toSorted"
                    | "toReversed"
                    | "take"
                    | "unique"
                    | "concat"
            ) =>
        {
            list_root(obj)
        }
        _ => None,
    }
}

/// A state whose value says nothing, waiting for the body to say what is
/// assigned to it.
pub(super) struct Untyped {
    pub name: String,
    /// What it started as, for the hint: `[]`, `{}`, `null`, `…`.
    pub start: &'static str,
    pub span: Span,
    pub persist: bool,
    /// It starts as `null`, so what is assigned it later may be null too.
    pub null: bool,
}

/// Whether `ty` can be written as an annotation: a map literal's shape, a
/// handle or an unknown cannot.
fn writable(ty: &Type) -> bool {
    match ty {
        Type::String
        | Type::Number
        | Type::Bool
        | Type::Record(_)
        | Type::Enum(_)
        | Type::Scalar(_) => true,
        Type::List(t) | Type::Optional(t) => writable(t),
        Type::Dict(k, v) => writable(k) && writable(v),
        Type::Func(params, ret) => {
            params.iter().all(writable) && (**ret == Type::Null || writable(ret))
        }
        _ => false,
    }
}

impl Checker<'_, '_> {
    /// A record drafted from the shape of a map literal assigned to the
    /// state `state`: its name — the state's, capitalised, unless the
    /// program has that name — and its declaration.
    fn draft_record(&self, state: &str, ty: &Type) -> Option<(String, String)> {
        let Type::Shape(fields) = ty else {
            return None;
        };
        if fields.is_empty() || !fields.iter().all(|(_, t)| writable(t)) {
            return None;
        }
        let mut chars = state.chars();
        let first = chars.next()?;
        let base: String = first.to_uppercase().chain(chars).collect();
        let taken = |n: &str| {
            self.world.types.contains_key(n)
                || self.world.enums.contains_key(n)
                || self.world.components.contains_key(n)
                || self.world.stores.contains_key(n)
        };
        let name = if taken(&base) {
            format!("{base}Record")
        } else {
            base
        };
        if taken(&name) {
            return None;
        }
        let written: Vec<String> = fields.iter().map(|(n, t)| format!("{n}: {t}")).collect();
        Some((
            name.clone(),
            format!("type {name} {{ {} }}", written.join(", ")),
        ))
    }

    /// What `value`, assigned to `name`, says the state holds: its own
    /// type, or — where that is unknown only because it reads the state
    /// itself (`items.concat([t])`, `[...seen, id]`) — the type of what it
    /// adds.
    pub(super) fn assigned_type(&mut self, name: &str, value: &super::Expr, ty: &Type) -> Type {
        use super::Expr;
        if !says_nothing(ty) {
            return ty.clone();
        }
        let own = |e: &Expr| matches!(e, Expr::Identifier(n) if n == name);
        match value {
            Expr::MethodCall(obj, method, args)
                if own(obj) && method == "concat" && args.len() == 1 =>
            {
                self.infer_quiet(&args[0])
            }
            Expr::ListLiteral(items) => {
                let added: Vec<&Expr> = items
                    .iter()
                    .filter(|i| !matches!(i, Expr::Spread(e) if own(e)))
                    .collect();
                match added.first() {
                    Some(Expr::Spread(e)) => self.infer_quiet(e),
                    Some(e) => Type::list(self.infer_quiet(e)),
                    None => ty.clone(),
                }
            }
            other => {
                // `live = items.filter(…)` where `items` says nothing yet
                // either: `live` holds what `items` is later found to hold.
                if let Some(root) = list_root(other)
                    && root != name
                    && self.untyped.iter().any(|u| u.name == root)
                {
                    self.assigned_from
                        .push((name.to_string(), root.to_string()));
                }
                ty.clone()
            }
        }
    }

    /// Note that `name = value` gave a state its type, when the state's own
    /// value did not.
    pub(super) fn note_assigned(&mut self, name: &str, ty: &Type) {
        if self.untyped.iter().any(|u| u.name == name)
            && !says_nothing(ty)
            && !matches!(ty, Type::Error | Type::Null)
        {
            self.assigned
                .entry(name.to_string())
                .or_insert_with(|| ty.clone());
        }
    }

    /// `T25` for each state of the body whose value says nothing, with the
    /// type the body assigns it as the fix.
    pub(super) fn report_untyped(&mut self) {
        let untyped = std::mem::take(&mut self.untyped);
        let mut assigned: HashMap<String, Type> = std::mem::take(&mut self.assigned);
        let from = std::mem::take(&mut self.assigned_from);
        // A state assigned a list of another's items holds what that one
        // holds, through as many steps as there are.
        for _ in 0..from.len() {
            let mut changed = false;
            for (to, root) in &from {
                if !assigned.contains_key(to)
                    && let Some(ty) = assigned.get(root).filter(|t| matches!(t, Type::List(_)))
                {
                    assigned.insert(to.clone(), ty.clone());
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }
        for u in untyped {
            let keyword = if u.persist { "persist" } else { "state" };
            // A map literal assigned it is a record, drafted from its
            // fields and named after the state: `user = { name: "Ada" }`
            // gives `type User { name: String }`.
            let drafted = assigned
                .get(&u.name)
                .and_then(|ty| self.draft_record(&u.name, ty));
            let now = match &drafted {
                Some((name, _)) => Some(Type::Record(name.clone())),
                None => assigned.get(&u.name).cloned(),
            }
            .map(|ty| if u.null { Type::optional(ty) } else { ty })
            .filter(writable);
            let now = now.as_ref();
            let hint = if let Some(ty) = now {
                format!("Write what it holds: `{keyword} {}: {ty} = …`", u.name)
            } else {
                // Nothing the body assigns says it: the hint names no type
                // the program may not have, only the shape of one.
                let (example, what) = match u.start {
                    "[]" => ("[T]", ", `T` the type of its items"),
                    "{}" => ("[String: T]", ", `T` the type of its values"),
                    "null" => ("T?", ", `T` the type of what it holds"),
                    _ => ("Json", ""),
                };
                format!(
                    "Write what it holds: `{keyword} {}: {example} = {}`{what}",
                    u.name, u.start
                )
            };
            self.error(
                u.span,
                "T25",
                format!(
                    "`{keyword} {}`'s first value does not say what it holds",
                    u.name
                ),
                &hint,
            );
            if let Some(ty) = now {
                let declare = Plan::Rename {
                    from: format!("{keyword} {} =", u.name),
                    to: format!("{keyword} {}: {ty} =", u.name),
                };
                let plan = match &drafted {
                    Some((_, record)) => Plan::All(vec![
                        declare,
                        Plan::InsertAtLine {
                            file: self.file.to_string(),
                            line: 1,
                            text: format!("{record}\n\n"),
                        },
                    ]),
                    None => declare,
                };
                self.plan_last(format!("Declare `{}: {ty}`", u.name), plan);
            }
        }
    }
}

/// A parameter of an action written with no type.
pub(super) struct UntypedParam {
    pub decl: usize,
    pub span: Span,
    pub action: String,
    pub index: usize,
    pub name: String,
}

impl std::fmt::Debug for UntypedParam {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({})", self.action, self.name)
    }
}

impl Checker<'_, '_> {
    /// Note what a call of the action `action`, declared in the body
    /// `decl`, passes each parameter that says nothing of its type.
    pub(super) fn note_passed(
        &mut self,
        decl: usize,
        action: &str,
        params: &[Type],
        args: &[super::Expr],
    ) {
        for (index, (param, arg)) in params.iter().zip(args).enumerate() {
            if !param.is_any() {
                continue;
            }
            let ty = self.infer_quiet(arg);
            self.info
                .passed
                .entry((decl, action.to_string(), index))
                .or_default()
                .push(ty);
        }
    }

    /// `T32` for a parameter with no type, and the type its callers pass
    /// as the fix when they all pass one.
    pub(super) fn report_untyped_param(&mut self, p: &UntypedParam, passed: &[Type]) {
        let mut kinds: Vec<Type> = passed
            .iter()
            .filter(|t| !matches!(t, Type::Error) && !says_nothing(t))
            .cloned()
            .collect();
        kinds.dedup_by(|a, b| a.to_string() == b.to_string());
        let agreed = match kinds.as_slice() {
            [one] if writable(one) && passed.iter().all(|t| t.to_string() == one.to_string()) => {
                Some(one.clone())
            }
            _ => None,
        };
        self.error(
            p.span,
            "T32",
            format!("`{}` of `{}` says nothing of its type", p.name, p.action),
            &match &agreed {
                Some(ty) => format!("Every call passes a `{ty}`: write `{}: {ty}`", p.name),
                None => format!("Write what it takes: `{}: String`", p.name),
            },
        );
        if let Some(ty) = agreed {
            let before = if p.index == 0 { "(" } else { ", " };
            self.plan_last(
                format!("Declare `{}: {ty}`", p.name),
                Plan::Rename {
                    from: format!("{before}{}", p.name),
                    to: format!("{before}{}: {ty}", p.name),
                },
            );
        }
    }
}
