//! The browser, typed: its globals, its functions and the members of the
//! objects it hands a program, read from the browser table
//! (`registry::browser::BROWSER_TYPES`). Anything the table does
//! not list is `T30`, and reached from a project script whose JSDoc says
//! its types.

use super::{Checker, Expr, Scalar, Type, expr_text};
use crate::registry::browser::{BrowserFn, BrowserMember, browser};
use crate::syntax::parser::ast::TypeRef;

/// What `T30` says to do about a name the table does not type.
const THROUGH_A_SCRIPT: &str = "Reach it from a script under src/ and say its types in JSDoc; the call is then checked like any other";

impl Checker<'_, '_> {
    /// A type the browser table writes, as the checker reads it: an object
    /// the table declares is that object, anything else the language's.
    pub(super) fn host_type(&self, t: &TypeRef) -> Type {
        self.host_type_of(Type::from_ref(t))
    }

    /// [`host_type`](Self::host_type), of a type already read.
    pub(super) fn host_type_of(&self, t: Type) -> Type {
        fn walk(t: Type) -> Type {
            match t {
                Type::Record(n) if browser().object(&n).is_some() => Type::Host(n),
                Type::List(i) => Type::list(walk(*i)),
                Type::Optional(i) => Type::optional(walk(*i)),
                Type::Dict(k, v) => Type::Dict(Box::new(walk(*k)), Box::new(walk(*v))),
                Type::Func(ps, r) => {
                    Type::Func(ps.into_iter().map(walk).collect(), Box::new(walk(*r)))
                }
                other => other,
            }
        }
        self.world.resolve(walk(t))
    }

    /// `T30`: `what` is a part of the browser the language does not type.
    pub(super) fn untyped_browser(&mut self, what: &str, wrong: &str, known: &[&str]) -> Type {
        self.error_naming(
            "T30",
            format!("the language does not type `{what}`"),
            THROUGH_A_SCRIPT,
            wrong,
            known,
            false,
        );
        Type::Error
    }

    /// A global the browser has, read by name: `window`, `Math`. `None`
    /// when the table does not list it.
    pub(super) fn browser_global(&self, name: &str) -> Option<Type> {
        browser().value(name).map(|t| self.host_type(t))
    }

    /// A field of a browser object, `e.key`; a method read without being
    /// called is the function it is.
    pub(super) fn host_field(&mut self, object: &str, base: &Expr, field: &str) -> Type {
        if let Some(ty) = self.class_member(object, base, field, None) {
            return ty;
        }
        match browser().member(object, field) {
            Some(BrowserMember::Field(t)) => self.host_type(t),
            Some(BrowserMember::Method(f)) => {
                let f = f.clone();
                self.host_fn_type(&f)
            }
            None => {
                let names = browser().member_names(object);
                self.untyped_browser(&format!("{}.{field}", expr_text(base)), field, &names)
            }
        }
    }

    /// A method of a browser object called, `el.focus()`.
    pub(super) fn host_method(
        &mut self,
        object: &str,
        base: &Expr,
        method: &str,
        args: &[Expr],
    ) -> Type {
        if let Some(ty) = self.class_member(object, base, method, Some(args)) {
            return ty;
        }
        match browser().member(object, method) {
            Some(BrowserMember::Method(f)) => {
                let f = f.clone();
                self.host_call(&f, args, &format!("`{}.{method}`", expr_text(base)))
            }
            Some(BrowserMember::Field(t)) => {
                let ty = self.host_type(t);
                for a in args {
                    self.infer(a, None);
                }
                self.error_at_current(
                    "T10",
                    format!("`{}.{method}` is a `{ty}`, not a function", expr_text(base)),
                    "Read it without calling it",
                );
                Type::Error
            }
            None => {
                for a in args {
                    self.infer(a, None);
                }
                let names = browser().member_names(object);
                self.untyped_browser(&format!("{}.{method}", expr_text(base)), method, &names)
            }
        }
    }

    /// A member of an instance of a project script's class — `tilt.setMax(3)`
    /// — typed by its method's JSDoc: called when `args` is given, read
    /// (a getter) when it is not. `None` when `object` is no class of the
    /// project's.
    fn class_member(
        &mut self,
        object: &str,
        base: &Expr,
        member: &str,
        args: Option<&[Expr]>,
    ) -> Option<Type> {
        use crate::syntax::js_scan::{Name, NameKind};
        if browser().object(object).is_some() {
            return None;
        }
        let (file, class) = self.world.scripts.get(object).copied()?;
        let NameKind::Class { methods, .. } = &class.kind else {
            return None;
        };
        let Some(m) = methods.iter().find(|m| m.name == member && !m.is_static) else {
            let known: Vec<&str> = methods
                .iter()
                .filter(|m| !m.is_static)
                .map(|m| m.name.as_str())
                .collect();
            if let Some(args) = args {
                for a in args {
                    self.infer(a, None);
                }
            }
            self.error_naming(
                "T05",
                format!(
                    "`{}` is a `{object}`; it has no member `{member}`",
                    expr_text(base)
                ),
                &format!(
                    "A `{object}` has {}",
                    known
                        .iter()
                        .map(|k| format!("`{k}`"))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                member,
                &known,
                true,
            );
            return Some(Type::Error);
        };
        let as_function = Name {
            name: format!("{object}.{}", m.name),
            kind: NameKind::Function {
                params: m.params.clone(),
                is_async: false,
            },
            line: class.line,
            col: class.col,
            doc: m.doc.clone(),
        };
        Some(match args {
            Some(args) => self.script_call(file, &as_function, args),
            None if m.getter => self.script_call(file, &as_function, &[]),
            None => Type::Error,
        })
    }

    /// A function of the browser called by name, `setTimeout(…)`.
    pub(super) fn browser_function(&mut self, name: &str, args: &[Expr]) -> Option<Type> {
        let f = browser().function(name)?.clone();
        Some(self.host_call(&f, args, &format!("`{name}`")))
    }

    /// A browser function as a value: what passing it, or a method read
    /// without a call, gives.
    fn host_fn_type(&self, f: &BrowserFn) -> Type {
        let params = f
            .params
            .iter()
            .map(|p| p.ty.as_ref().map_or(Type::Error, |t| self.host_type(t)))
            .collect();
        let ret = self.host_type(&f.ret);
        Type::Func(params, Box::new(self.host_result(f, ret)))
    }

    fn host_result(&self, f: &BrowserFn, ret: Type) -> Type {
        if f.is_async {
            Type::Promise(Box::new(ret))
        } else {
            ret
        }
    }

    /// The arguments of a browser function against its parameters: as many
    /// as it takes — fewer where the last may be left out, any number for
    /// a `...rest` — each of the type it says.
    fn host_call(&mut self, f: &BrowserFn, args: &[Expr], what: &str) -> Type {
        let required = f.params.iter().filter(|p| !p.optional && !p.rest).count();
        let rest = f.params.last().is_some_and(|p| p.rest);
        let most = if rest { usize::MAX } else { f.params.len() };
        if args.len() < required || args.len() > most {
            let takes = if rest {
                format!("at least {required}")
            } else if required == f.params.len() {
                required.to_string()
            } else {
                format!("{required} to {}", f.params.len())
            };
            self.error_at_current(
                "T10",
                format!(
                    "{what} takes {takes} argument{}, but {} {} given",
                    if takes == "1" { "" } else { "s" },
                    args.len(),
                    if args.len() == 1 { "is" } else { "are" }
                ),
                "",
            );
        }
        for (i, arg) in args.iter().enumerate() {
            let param = f
                .params
                .get(i)
                .or_else(|| f.params.last().filter(|p| p.rest));
            let Some(param) = param else {
                self.infer(arg, None);
                continue;
            };
            let Some(t) = &param.ty else {
                // Any value but a secret: `console.log(…)`.
                let given = self.infer(arg, None);
                if given.unwrapped() == Type::Scalar(Scalar::Secret) {
                    self.error_at_current(
                        "T12",
                        format!(
                            "`{}` is a `Secret`, and {what} would show it",
                            expr_text(arg)
                        ),
                        "A secret must not reach the page; log what it is for, never the secret",
                    );
                }
                continue;
            };
            let wanted = self.host_type(t);
            let given = self.infer(arg, Some(&wanted));
            if given.unwrapped() == Type::Scalar(Scalar::Secret) {
                self.error_at_current(
                    "T12",
                    format!("`{}` is a `Secret`, and {what} would give it to anyone who opens the page", expr_text(arg)),
                    "A secret goes to the server only — an `api`'s headers, a request body — never into the browser's own hands",
                );
            } else if !given.unwrapped().is_any() && !given.assignable_to(&wanted) {
                self.error_at_current(
                    "T01",
                    format!(
                        "`{}` of {what} is a `{wanted}`, but `{}` is `{given}`",
                        param.name,
                        expr_text(arg)
                    ),
                    "",
                );
            }
        }
        let ret = self.host_type(&f.ret);
        self.host_result(f, ret)
    }

    /// What a DOM event's handler is given: a `KeyboardEvent` for
    /// `keydown`, an `Event` for one the table does not name.
    pub(super) fn event_type(event: &str) -> Type {
        Type::Host(browser().event(event).to_string())
    }
}

/// Whether the browser table declares an object of this name.
pub(super) fn is_browser_object(name: &str) -> bool {
    browser().object(name).is_some()
}
