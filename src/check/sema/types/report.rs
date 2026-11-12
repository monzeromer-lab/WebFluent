//! Findings: errors and warnings, the names they suggest, and the fixes they carry.

use super::{Checker, Diagnostic, Expr, Span, Type, expr_text};

impl Checker<'_, '_> {
    pub(super) fn warn(&mut self, code: &'static str, message: String, hint: &str) {
        let span = self.located(self.current_span, &message);
        let d = Diagnostic::coded(
            code,
            message,
            self.file,
            span.line as usize,
            span.col as usize,
        )
        .with_span(span, self.source)
        .with_hint(hint);
        self.push(d);
    }

    pub(super) fn error(&mut self, span: Span, code: &'static str, message: String, hint: &str) {
        let d = Diagnostic::coded(
            code,
            message,
            self.file,
            span.line as usize,
            span.col as usize,
        )
        .with_span(span, self.source)
        .with_hint(hint);
        self.push(d);
    }

    /// File a finding under the severity its code has in the registry
    /// (`diagnostics::codes`), whichever helper reported it: a code is an
    /// error or a warning in one place only.
    pub(super) fn push(&mut self, d: Diagnostic) {
        self.last_was_error = d.is_error();
        if d.is_error() {
            self.info.findings.errors.push(d);
        } else {
            self.info.findings.warnings.push(d);
        }
    }

    /// The list the finding reported last was filed in, by its severity.
    fn last_list(&mut self) -> &mut Vec<Diagnostic> {
        if self.last_was_error {
            &mut self.info.findings.errors
        } else {
            &mut self.info.findings.warnings
        }
    }

    /// A method a number, a string or a list does not have: in the browser,
    /// `x.method is not a function`, the first time the line runs.
    pub(super) fn unknown_method(&mut self, on: &Type, method: &str, known: &[&str]) {
        let hint = if crate::diagnostics::fixes::nearest(method, known.iter().copied()).is_some() {
            String::new()
        } else {
            let shown: Vec<String> = known.iter().take(12).map(|k| format!("`{k}`")).collect();
            format!("It has {}, …", shown.join(", "))
        };
        self.error_naming(
            "T16",
            format!("a `{on}` has no method `{method}`"),
            &hint,
            method,
            known,
            true,
        );
    }

    /// `name` resolves to nothing: recorded, for the build to refuse.
    pub(super) fn unresolved(&mut self, name: &str, called: bool) {
        // A store's own action, called where the store is not built yet —
        // its initial values are worked out before its actions exist.
        if called
            // The first store by name that has it, so the finding is the
            // same on every run, whatever order the map holds them in.
            && let Some(store) = self
                .world
                .store_actions
                .iter()
                .filter(|(_, actions)| actions.contains_key(name))
                .map(|(s, _)| s.to_string())
                .min()
        {
            let message = format!(
                "`{name}(…)` is an action of `{store}`, which nothing here reaches by that name"
            );
            let span = self.located(self.current_span, &message);
            let d = Diagnostic::coded("T13", message, self.file, span.line as usize, span.col as usize)
                .with_span(span, self.source)
                .with_hint(format!(
                    "Outside the store, call it `{store}.{name}(…)`. Inside it, a store's initial values are worked out before its actions exist: write the value out, or work it out in a `derived`, which may call them"
                ));
            if !self
                .info
                .unresolved
                .iter()
                .any(|u| u.message == d.message && u.line == d.line)
            {
                self.info.unresolved.push(d);
            }
            return;
        }
        let message = if called {
            format!("`{name}(…)` calls a function nothing declares")
        } else {
            format!("nothing declares `{name}`")
        };
        let span = self.located(self.current_span, &message);
        // What the name may have meant: anything in scope here, or declared
        // at the top of the program.
        let mut known: Vec<&str> = self
            .scopes
            .iter()
            .flat_map(|scope| scope.keys().map(String::as_str))
            .collect();
        known.extend(self.world.consts.keys().map(|k| &**k));
        known.extend(self.world.stores.keys().map(|k| &**k));
        known.extend(self.world.apis.keys().map(|k| &**k));
        known.extend(self.world.scripts.keys().map(|k| &**k));
        let near =
            crate::diagnostics::fixes::nearest(name, known.iter().copied()).map(str::to_string);
        let declare = "Declare it — a `state`, a `const`, an `action`, a prop — or check the spelling. In the browser it would be a ReferenceError";
        let mut d = Diagnostic::coded(
            "T13",
            message,
            self.file,
            span.line as usize,
            span.col as usize,
        )
        .with_span(span, self.source)
        .with_hint(match &near {
            Some(n) => format!("Did you mean `{n}`? {declare}"),
            None => declare.to_string(),
        });
        if let Some(n) = near {
            d = d.with_plan(
                format!("Change to `{n}`"),
                crate::diagnostics::fixes::Plan::Rename {
                    from: name.to_string(),
                    to: n,
                },
            );
        }
        if !self
            .info
            .unresolved
            .iter()
            .any(|u| u.message == d.message && u.line == d.line)
        {
            self.info.unresolved.push(d);
        }
    }

    pub(super) fn error_at_current(&mut self, code: &'static str, message: String, hint: &str) {
        let span = self.located(self.current_span, &message);
        self.error(span, code, message, hint);
    }

    /// A fix for the warning just reported.
    pub(super) fn plan_last_warning(
        &mut self,
        title: String,
        plan: crate::diagnostics::fixes::Plan,
    ) {
        let list = self.last_list();
        if let Some(d) = list.pop() {
            list.push(d.with_plan(title, plan));
        }
    }

    /// The nearest of an enum's `cases` to `.case`, as a fix for the error
    /// just reported.
    pub(super) fn plan_nearest_case(&mut self, case: &str, cases: &[String]) {
        if let Some(near) =
            crate::diagnostics::fixes::nearest(case, cases.iter().map(String::as_str))
        {
            let near = near.to_string();
            self.plan_last(
                format!("Change to `.{near}`"),
                crate::diagnostics::fixes::Plan::Rename {
                    from: format!(".{case}"),
                    to: format!(".{near}"),
                },
            );
        }
    }

    /// `(await x)` for a promise read as if it were its value — where an
    /// `await` may be written; elsewhere it would be another T17.
    pub(super) fn plan_await(&mut self, promise: &Expr) {
        if !self.async_ok {
            return;
        }
        let Some(source) = self.source else {
            return;
        };
        // The promise's own text in the statement: `expr_text` shortens a
        // call's arguments, so the call is found by what comes before its
        // `(` and taken to its matching `)`.
        let shown = expr_text(promise);
        let head = shown.split('(').next().unwrap_or(&shown).to_string();
        let (from, to) = (
            self.current_span.start as usize,
            (self.current_span.end as usize).min(source.len()),
        );
        if head.is_empty()
            || from >= to
            || !source.is_char_boundary(from)
            || !source.is_char_boundary(to)
        {
            return;
        }
        let Some(at) = source[from..to].find(&head).map(|i| from + i) else {
            return;
        };
        let mut end = at + head.len();
        if shown.contains('(') {
            if source.as_bytes().get(end) != Some(&b'(') {
                return;
            }
            let mut depth = 0usize;
            let mut i = end;
            let bytes = source.as_bytes();
            loop {
                match bytes.get(i) {
                    Some(b'(') => depth += 1,
                    Some(b')') => {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                    Some(_) => {}
                    None => return,
                }
                i += 1;
            }
            end = i + 1;
        }
        let text = format!("(await {})", &source[at..end]);
        self.plan_last(
            "Await it".to_string(),
            crate::diagnostics::fixes::Plan::Replace {
                start: at,
                end,
                text,
            },
        );
    }

    /// A fix for the error just reported.
    pub(super) fn plan_last(&mut self, title: String, plan: crate::diagnostics::fixes::Plan) {
        let list = self.last_list();
        if let Some(d) = list.pop() {
            list.push(d.with_plan(title, plan));
        }
    }

    /// An error about a name that is wrong — `.nmae`, `Cart.totl`, `cuont`:
    /// the nearest of `known` is suggested, and offered as a fix. `dotted`
    /// names a member, matched with the `.` before it.
    pub(super) fn error_naming(
        &mut self,
        code: &'static str,
        message: String,
        hint: &str,
        wrong: &str,
        known: &[&str],
        dotted: bool,
    ) {
        let near = crate::diagnostics::fixes::nearest(wrong, known.iter().copied());
        let hint = match near {
            Some(n) if hint.is_empty() => format!("Did you mean `{n}`?"),
            Some(n) => format!("Did you mean `{n}`? {hint}"),
            None => hint.to_string(),
        };
        // At the name that is wrong — the member, the field — which is what
        // the fix renames, not the first name the message gives (`Store`):
        // the build's line and column and the editor's then agree.
        let at_wrong = self.located(self.current_span, &format!("`{wrong}`"));
        let span =
            if (at_wrong.start, at_wrong.end) == (self.current_span.start, self.current_span.end) {
                self.located(self.current_span, &message)
            } else {
                at_wrong
            };
        let mut d = Diagnostic::coded(
            code,
            message,
            self.file,
            span.line as usize,
            span.col as usize,
        )
        .with_span(span, self.source)
        .with_hint(hint);
        if let Some(n) = near {
            let (from, to) = if dotted {
                (format!(".{wrong}"), format!(".{n}"))
            } else {
                (wrong.to_string(), n.to_string())
            };
            d = d.with_plan(
                format!("Change to `{n}`"),
                crate::diagnostics::fixes::Plan::Rename { from, to },
            );
        }
        self.push(d);
    }
}
