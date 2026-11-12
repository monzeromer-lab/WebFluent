//! Refined types: the conditions a value is held to, and the literals that break them.

use super::{Expr, Scalar, Type, TypeRef};

/// What a rule is a rule for, or `None` when there is no such rule.
/// The rules a `validate` block may hold, for a tool to offer;
/// `rule_wants` reads each.
pub const VALIDATE_RULES: &[&str] = &[
    "required",
    "email",
    "url",
    "minLength",
    "maxLength",
    "min",
    "max",
    "pattern",
    "matches",
    "oneOf",
    "custom",
    "async",
];

pub(super) fn rule_wants(name: &str) -> Option<Vec<Type>> {
    Some(match name {
        // A value of any type is either there or not.
        "required" | "custom" | "async" | "matches" | "oneOf" => {
            vec![Type::Any, Type::String, Type::Number, Type::Bool]
        }
        "email" => vec![Type::String, Type::Scalar(Scalar::Email)],
        "url" => vec![Type::String, Type::Scalar(Scalar::Url)],
        "minLength" | "maxLength" | "pattern" => vec![Type::String],
        "min" | "max" => vec![
            Type::Number,
            Type::Scalar(Scalar::Date),
            Type::Scalar(Scalar::DateTime),
            Type::Scalar(Scalar::Time),
        ],
        _ => return None,
    })
}

/// Why a literal is not the scalar it was written for, or `None` when it
/// is one. The message says how the type is written.
pub(super) fn ill_formed(scalar: Scalar, text: &str) -> Option<String> {
    let hex = |s: &str| s.chars().all(|c| c.is_ascii_hexdigit());
    let fits = match scalar {
        Scalar::Date => {
            let p: Vec<&str> = text.split('-').collect();
            let shaped = p.len() == 3
                && p[0].len() == 4
                && p[1].len() == 2
                && p[2].len() == 2
                && p.iter().all(|s| s.chars().all(|c| c.is_ascii_digit()));
            if shaped {
                // A day the calendar has: `2026-02-30` is not one.
                let (y, m, d): (u32, u32, u32) = (
                    p[0].parse().unwrap_or(0),
                    p[1].parse().unwrap_or(0),
                    p[2].parse().unwrap_or(0),
                );
                let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
                let days = match m {
                    1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
                    4 | 6 | 9 | 11 => 30,
                    2 if leap => 29,
                    2 => 28,
                    _ => 0,
                };
                if days == 0 || d == 0 || d > days {
                    return Some(if days == 0 {
                        format!("There is no month `{}`: months are `01` to `12`", p[1])
                    } else {
                        format!("{} has {days} days", month_name(m))
                    });
                }
            }
            shaped
        }
        Scalar::Time => {
            let head = text.split(['+', 'Z']).next().unwrap_or(text);
            let p: Vec<&str> = head.split(':').collect();
            (2..=3).contains(&p.len())
                && p[0].len() == 2
                && p[1].len() == 2
                && p.iter().all(|s| {
                    s.split('.')
                        .next()
                        .unwrap_or(s)
                        .chars()
                        .all(|c| c.is_ascii_digit())
                })
        }
        Scalar::DateTime => match text.split_once('T') {
            Some((d, t)) => {
                ill_formed(Scalar::Date, d).is_none() && ill_formed(Scalar::Time, t).is_none()
            }
            None => false,
        },
        Scalar::Url => text.contains("://") && text.len() > 8,
        Scalar::Email => match text.split_once('@') {
            Some((user, host)) => !user.is_empty() && host.contains('.') && !host.starts_with('.'),
            None => false,
        },
        Scalar::Color => {
            let body = text.trim();
            match body.strip_prefix('#') {
                Some(digits) => matches!(digits.len(), 3 | 4 | 6 | 8) && hex(digits),
                // A CSS colour of any other spelling: a function, or a name.
                None => body.contains('(') || body.chars().all(|c| c.is_ascii_alphabetic()),
            }
        }
        Scalar::Uuid => {
            let p: Vec<&str> = text.split('-').collect();
            p.len() == 5
                && [8, 4, 4, 4, 12] == [p[0].len(), p[1].len(), p[2].len(), p[3].len(), p[4].len()]
                && p.iter().all(|s| hex(s))
        }
        // Anything may be a secret; a file is never a literal.
        Scalar::Secret => true,
        _ => true,
    };
    if fits {
        return None;
    }
    Some(
        match scalar {
            Scalar::Date => "A date is written `2026-03-14`, or `@2026-03-14`",
            Scalar::Time => "A time is written `09:30`, or `@09:30`",
            Scalar::DateTime => {
                "A moment is written `2026-03-14T09:30:00Z`, or `@2026-03-14T09:30Z`"
            }
            Scalar::Url => "A URL has a scheme: `https://example.com`",
            Scalar::Email => "An address is `name@example.com`",
            Scalar::Color => "A colour is `#0F766E`, `rgb(15, 118, 110)` or a CSS name",
            Scalar::Uuid => "A UUID is `3f2b1c4d-5e6f-7081-92a3-b4c5d6e7f809`",
            _ => "",
        }
        .to_string(),
    )
}

/// Why a value does not meet its type's condition, when it plainly does
/// not: `Number(0..=100)` given `120`, `String(minLength: 8)` given `"abc"`.
///
/// Only a value the compiler can read is checked — a literal, or an
/// expression over literals. Anything that arrives at run time is
/// validation's to refuse, from the same condition.
pub(super) fn month_name(m: u32) -> &'static str {
    [
        "",
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ][m as usize]
}

pub fn refinement_fault(ty: &TypeRef, value: &Expr) -> Option<String> {
    use crate::project::static_eval::{Scope, Static, eval};
    let args = ty.refinement();
    if args.is_empty() {
        return None;
    }
    let scope = Scope::default();
    let got = eval(value, &scope)?;
    let number = |v: &Static| match v {
        Static::Num(n) => Some(*n),
        _ => None,
    };
    for (name, bound) in args {
        let want = eval(bound, &scope)?;
        let fault = match name.as_str() {
            "min" => number(&got)? < number(&want)?,
            "max" => number(&got)? > number(&want)?,
            "below" => number(&got)? >= number(&want)?,
            "minLength" => (got.to_text().chars().count() as f64) < number(&want)?,
            "maxLength" => (got.to_text().chars().count() as f64) > number(&want)?,
            "after" => got.to_text().as_str() <= want.to_text().as_str(),
            "before" => got.to_text().as_str() >= want.to_text().as_str(),
            "pattern" => match &want {
                Static::Regex(p, f) => !crate::project::static_eval::matches(&got.to_text(), p, f)?,
                _ => false,
            },
            _ => false,
        };
        if fault {
            let said = match name.as_str() {
                "min" => format!("is below {}", want.to_text()),
                "max" => format!("is above {}", want.to_text()),
                "below" => format!("is not below {}", want.to_text()),
                "minLength" => format!("is shorter than {}", want.to_text()),
                "maxLength" => format!("is longer than {}", want.to_text()),
                "after" => format!("is not after {}", want.to_text()),
                "before" => format!("is not before {}", want.to_text()),
                _ => format!("does not match {}", want.to_text()),
            };
            return Some(said);
        }
    }
    None
}
