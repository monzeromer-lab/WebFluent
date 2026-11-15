//! Patterns: what a `match` arm or an `if let` takes of a value, as the
//! test that it matches and the place each name it binds is read from.
//!
//! A case with a payload is `[case, …payload]` at run time, so the value
//! at place `i` of it is `v[i + 1]`; a `Json` value carries itself, so its
//! case's one place is the value.

use super::JsCodegen;
use crate::syntax::parser::ast::{Expr, MatchArm, Pattern};

impl JsCodegen {
    /// The test that the value `v` matches `p`, or `None` when every value
    /// does. `json` when `v` is the language's `Json`.
    pub(super) fn pattern_test(&self, p: &Pattern, v: &str, json: bool) -> Option<String> {
        match p {
            Pattern::Bind(_) | Pattern::Wildcard => None,
            Pattern::Literal(Expr::Null) => Some(format!("{v} == null")),
            Pattern::Literal(e) => Some(format!("{v} === {}", self.emit_expr(e))),
            Pattern::Case(case, inner) => {
                let of = if json { "jsonCase" } else { "caseOf" };
                let mut tests = vec![format!("WF.{of}({v}) === \"{case}\"")];
                for (i, p) in inner.iter().enumerate() {
                    if let Some(t) = self.pattern_test(p, &place(v, i, json), false) {
                        tests.push(t);
                    }
                }
                Some(tests.join(" && "))
            }
        }
    }

    /// Each name `p` binds, and where in the value `v` it is read.
    pub(super) fn pattern_binds(p: &Pattern, v: &str, json: bool, out: &mut Vec<(String, String)>) {
        match p {
            Pattern::Bind(name) => out.push((name.clone(), v.to_string())),
            Pattern::Wildcard | Pattern::Literal(_) => {}
            Pattern::Case(_, inner) => {
                for (i, p) in inner.iter().enumerate() {
                    Self::pattern_binds(p, &place(v, i, json), false, out);
                }
            }
        }
    }

    /// An arm as one pattern: its case, and what each place of its payload
    /// must match. `None` for an arm that is not a case — `else`.
    pub(super) fn arm_pattern(arm: &MatchArm) -> Option<Pattern> {
        let crate::syntax::parser::ast::ArmPattern::Case(case) = &arm.pattern else {
            return None;
        };
        let inner = if arm.patterns.is_empty() {
            arm.bindings
                .iter()
                .map(|b| Pattern::Bind(b.clone()))
                .collect()
        } else {
            arm.patterns.clone()
        };
        Some(Pattern::Case(case.clone(), inner))
    }

    /// Whether a `match` has an arm that takes only some of its case's
    /// values — a case or a literal in a place of the payload — and so is
    /// chosen by testing each arm in turn.
    pub(super) fn has_refutable_arm(arms: &[MatchArm]) -> bool {
        arms.iter()
            .any(|a| a.patterns.iter().any(|p| !p.is_irrefutable()))
    }
}

/// Place `i` of a case's payload in the value `v`.
fn place(v: &str, i: usize, json: bool) -> String {
    if json {
        v.to_string()
    } else {
        format!("{v}[{}]", i + 1)
    }
}
