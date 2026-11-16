//! `fake OneUser for Backend.users { list(page) { .ok([…]) } }`: a group's
//! endpoints answered without the network, held to the group's contract
//! (`N06`) — every endpoint it answers is one the group has, every name it
//! binds one of that endpoint's parameters, and every answer the result
//! the endpoint declares.

use super::Checker;
use super::ty::Type;
use crate::syntax::parser::ast::FakeDecl;

impl Checker<'_, '_> {
    /// A parameter of the endpoint, by the name the contract gives it —
    /// `after`, which is also the language's — so the fake reads it as the
    /// endpoint was declared to take it.
    fn bind_param(&mut self, name: &str, ty: Type) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), ty);
        }
    }

    pub(super) fn fake(&mut self, f: &FakeDecl) {
        self.current_span = f.header_span;
        if !self.world.apis.contains_key(f.target.as_str()) {
            let known: Vec<&str> = self.world.apis.keys().copied().collect();
            self.error_naming(
                "N06",
                format!("`fake {}` answers for `{}`, which is not a service or a group of one", f.name, f.target),
                "A fake answers for an `api` the program declares: `fake OneUser for Backend.users { … }`",
                &f.target,
                &known,
                true,
            );
            return;
        }
        let endpoints: Vec<&str> = self.world.apis[f.target.as_str()]
            .endpoints
            .iter()
            .map(|e| e.name.as_str())
            .collect();
        let mut seen: Vec<&str> = Vec::new();
        for answer in &f.answers {
            self.current_span = answer.span;
            let what = format!("{}.{}", f.target, answer.endpoint);
            if seen.contains(&answer.endpoint.as_str()) {
                self.error(
                    answer.span,
                    "N06",
                    format!("`fake {}` answers `{}` twice", f.name, answer.endpoint),
                    "Keep one answer an endpoint; branch inside it with `if` or `match`",
                );
                continue;
            }
            seen.push(&answer.endpoint);
            let Some(endpoint) = self.world.endpoint(&f.target, &answer.endpoint) else {
                self.error_naming(
                    "N06",
                    format!(
                        "`{}` has no endpoint `{}` for `fake {}` to answer",
                        f.target, answer.endpoint, f.name
                    ),
                    &format!(
                        "It has {}",
                        endpoints
                            .iter()
                            .map(|e| format!("`{e}`"))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                    &answer.endpoint,
                    &endpoints,
                    false,
                );
                continue;
            };
            self.push_scope();
            let names: Vec<&str> = endpoint.params.iter().map(|p| p.name.as_str()).collect();
            let mut bound: Vec<&str> = Vec::new();
            for name in &answer.params {
                if bound.contains(&name.as_str()) {
                    self.error(
                        answer.span,
                        "N06",
                        format!(
                            "`{}` of `fake {}` names `{name}` twice",
                            answer.endpoint, f.name
                        ),
                        "Name each parameter once",
                    );
                    continue;
                }
                bound.push(name);
                if let Some(p) = endpoint.params.iter().find(|p| p.name == *name) {
                    let mut ty = self.world.resolve(Type::from_ref(&p.prop_type));
                    if p.optional && p.default.is_none() {
                        ty = Type::optional(ty);
                    }
                    self.bind_param(name, ty);
                } else {
                    self.error_naming(
                            "N06",
                            format!("`{what}` has no parameter `{name}`"),
                            &if names.is_empty() {
                                "It takes none: write `()`".to_string()
                            } else {
                                format!(
                                    "A fake names the parameters it reads, as the endpoint names them: {}",
                                    names.iter().map(|n| format!("`{n}`")).collect::<Vec<_>>().join(", ")
                                )
                            },
                            name,
                            &names,
                            false,
                        );
                    self.bind_param(name, Type::Error);
                }
            }
            let want = endpoint
                .returns
                .as_ref()
                .map(|t| self.world.resolve(Type::from_ref(t)));
            self.current_span = answer.span;
            let given = self.infer(&answer.body, want.as_ref());
            if let Some(want) = want
                && !given.assignable_to(&want)
            {
                self.error(
                    answer.span,
                    "N06",
                    format!(
                        "`{}` of `fake {}` answers `{given}`, but `{what}` answers `{want}`",
                        answer.endpoint, f.name
                    ),
                    "Answer as the endpoint does: `.ok(value)`, or `.err(error)` with its error",
                );
            }
            self.pop_scope();
        }
    }
}
