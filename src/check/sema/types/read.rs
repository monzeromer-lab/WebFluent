//! A read — `resource users = Backend.users(page: n)` — is one value with
//! three cases: `.loading`, `.failed(e)` and `.loaded(value, refreshing,
//! stale)`. Its members are read from that value and cannot disagree with
//! it; 6.x's `state`, `data` and `refreshing`, its `loading`, `error(e)` and
//! `ready(v)` arms and its `on:` option are refused here, each with its fix.

use super::{Checker, Expr, Type, expr_text};
use crate::diagnostics::fixes::Plan;
use crate::syntax::parser::ast::{ArmPattern, MatchArm, ResourceDecl};

/// What a read's members are, beside its methods.
pub(super) const READ_MEMBERS: &[&str] = &[
    "value",
    "isLoading",
    "isLoaded",
    "isFailed",
    "isRefreshing",
    "error",
    "pending",
    "items",
    "hasMore",
];

/// The cases a read of `T` that fails with `E` is in, and what each carries.
pub(super) fn read_cases(value: &Type, failed: &Type) -> Vec<(String, Vec<Type>)> {
    vec![
        ("loading".to_string(), Vec::new()),
        ("failed".to_string(), vec![failed.clone()]),
        (
            "loaded".to_string(),
            vec![value.clone(), Type::Bool, Type::optional(failed.clone())],
        ),
    ]
}

/// The arm that does what a 6.x arm did, for a fix: its head as 7.0 writes
/// it.
fn arm_now(arm: &MatchArm) -> Option<(&'static str, String)> {
    let bound = |name: &Option<String>| name.as_ref().map(|n| format!("({n})")).unwrap_or_default();
    match arm.pattern {
        ArmPattern::Loading => Some(("loading", ".loading".to_string())),
        ArmPattern::Error => Some(("error", format!(".failed{}", bound(&arm.binding)))),
        ArmPattern::Ready => Some(("ready", format!(".loaded{}", bound(&arm.binding)))),
        _ => None,
    }
}

impl Checker<'_, '_> {
    /// `N08` for `users.data`, `users.state`, `users.refreshing`: 6.x's
    /// members, which the read's one value replaced.
    pub(super) fn old_read_member(&mut self, base: &Expr, field: &str) {
        let subject = expr_text(base);
        let now = match field {
            "data" => Some("value"),
            "refreshing" => Some("isRefreshing"),
            _ => None,
        };
        let hint = match now {
            Some(now) => format!("Write `{subject}.{now}`"),
            None => format!(
                "Ask what it is: `{subject}.isLoading`, `{subject}.isLoaded`, `{subject}.isFailed` — or `match {subject} {{ .loading … .failed(e) … .loaded(v) … }}`"
            ),
        };
        self.error_at_current(
            "N08",
            format!("`{subject}.{field}` is gone: a read is one value, `.loading`, `.failed(e)` or `.loaded(value)`"),
            &hint,
        );
        if let Some(now) = now {
            self.plan_last(
                format!("Write `{subject}.{now}`"),
                Plan::Rename {
                    from: format!("{subject}.{field}"),
                    to: format!("{subject}.{now}"),
                },
            );
        }
    }

    /// `T05` for a member a read does not have.
    pub(super) fn no_read_member(&mut self, base: &Expr, field: &str) {
        self.error_naming(
            "T05",
            format!("`{}` is a read; it has no member `{field}`", expr_text(base)),
            "A read has `value`, `isLoading`, `isLoaded`, `isFailed`, `isRefreshing`, `error` and `pending`",
            field,
            READ_MEMBERS,
            true,
        );
    }

    /// `N08` for each of 6.x's arms in a `match` over a read, with the arm
    /// 7.0 writes in its place.
    pub(super) fn old_read_arms(&mut self, subject: &Expr, arms: &[MatchArm]) {
        for arm in arms {
            let Some((word, now)) = arm_now(arm) else {
                continue;
            };
            self.error(
                arm.span,
                "N08",
                format!(
                    "`{word}` is 6.x's arm: `{}` is one value, matched by its case",
                    expr_text(subject)
                ),
                &format!("Write `{now}` — a read is `.loading`, `.failed(e)` or `.loaded(value, refreshing, stale)`"),
            );
            let head = match (&arm.pattern, &arm.binding) {
                (ArmPattern::Loading, _) => "loading".to_string(),
                (_, Some(name)) => format!("{word}({name})"),
                (_, None) => word.to_string(),
            };
            self.plan_last(
                format!("Write `{now}`"),
                Plan::Replace {
                    start: arm.span.start as usize,
                    end: arm.span.start as usize + head.len(),
                    text: now,
                },
            );
        }
    }

    /// `N08` for a read's `on:`, which `refresh:` replaced, with its fix
    /// where the policy is one case.
    pub(super) fn read_options(&mut self, r: &ResourceDecl) {
        let mut given: Vec<(&str, &Expr)> = r
            .options
            .iter()
            .map(|o| (o.key.trim_matches('"'), &o.value))
            .collect();
        if let Expr::MethodCall(_, _, args) = &r.url {
            for a in args {
                if let Expr::MapLiteral(pairs) = a {
                    given.extend(pairs.iter().map(|(k, v)| (k.trim_matches('"'), v)));
                }
            }
        }
        for (key, value) in given {
            match key {
                "on" => {
                    let now = match value {
                        Expr::EnumCase(c) if c == "focus" => Some((".focus", ".onFocus")),
                        Expr::EnumCase(c) if c == "reconnect" => {
                            Some((".reconnect", ".onReconnect"))
                        }
                        Expr::EnumCase(c) if c == "never" => Some((".never", ".never")),
                        Expr::CaseValue(c, _) if c == "interval" => Some((".interval(", ".every(")),
                        _ => None,
                    };
                    self.error_at_current(
                        "N08",
                        format!("`on:` on `resource {}` is 6.x's: when a read asks again is `refresh:`", r.name),
                        "Write `refresh: .onFocus`, `.onReconnect` (what it does unset), `.every(30.seconds)` or `.never`",
                    );
                    if let Some((was, now)) = now {
                        self.plan_last(
                            format!("Write `refresh: {now}`"),
                            Plan::Rename {
                                from: format!("on: {was}"),
                                to: format!("refresh: {now}"),
                            },
                        );
                    }
                }
                // 6.x's `paginate: .page`: how an endpoint pages is its
                // contract's to say, and a read writes no state of the page's.
                "paginate" => {
                    let endpoint = match &r.url {
                        Expr::MethodCall(obj, method, _) => {
                            format!("`{}.{method}`", expr_text(obj))
                        }
                        _ => "the endpoint".to_string(),
                    };
                    self.error_at_current(
                        "N08",
                        format!("`paginate:` on `resource {}` is 6.x's: how a read pages is its endpoint's to say", r.name),
                        &format!("Write `pages .number(param: page)` on {endpoint} (or `.cursor(…)`, `.link`), and drop `paginate:` — `{0}.items`, `{0}.hasMore` and `{0}.loadMore()` stay", r.name),
                    );
                }
                "refresh" => self.refresh_policy(value),
                "placeholder" if !matches!(value, Expr::EnumCase(c) if c == "previous") => {
                    self.error_at_current(
                        "T02",
                        format!("`placeholder:` takes `.previous`, not `{}`", expr_text(value)),
                        "`placeholder: .previous` keeps the last value shown while another request loads",
                    );
                }
                _ => {}
            }
        }
    }

    /// `refresh: .onFocus`, `.onReconnect`, `.every(d)`, `.never`, or a
    /// list of them.
    fn refresh_policy(&mut self, value: &Expr) {
        let items: Vec<&Expr> = match value {
            Expr::ListLiteral(items) => items.iter().collect(),
            other => vec![other],
        };
        for item in items {
            match item {
                Expr::EnumCase(c) if matches!(c.as_str(), "onFocus" | "onReconnect" | "never") => {}
                Expr::CaseValue(c, args) if c == "every" && args.len() == 1 => {
                    let given = self.infer(&args[0], Some(&Type::Record("Duration".to_string())));
                    if !given.is_any()
                        && !given.assignable_to(&Type::Number)
                        && !given.assignable_to(
                            &self.world.resolve(Type::Record("Duration".to_string())),
                        )
                    {
                        self.error_at_current(
                            "T01",
                            format!(
                                "`.every(…)` takes a `Duration`, but `{}` is `{given}`",
                                expr_text(&args[0])
                            ),
                            "Write how often: `.every(30.seconds)`",
                        );
                    }
                }
                other => {
                    self.error_at_current(
                        "T02",
                        format!("`refresh:` takes `.onFocus`, `.onReconnect`, `.every(d)` or `.never`, not `{}`", expr_text(other)),
                        "Write when to ask again: `refresh: .onFocus`",
                    );
                }
            }
        }
    }
}
