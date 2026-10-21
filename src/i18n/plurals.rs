//! Which plural form a number takes in a language, by CLDR's cardinal
//! rules: `en` has `one` and `other`, `ar` six, `ja` only `other`. The
//! rules are CLDR's own text ([`plurals_data`](super::plurals_data)),
//! read by a small evaluator of their syntax, so a language CLDR adds is a
//! regenerated table, not new code.
//!
//! A number is read as it is written: `1` is `one` in English and `1.0`
//! is `other`, because CLDR's operands count the visible fraction digits.

use super::plurals_data::RULES;

/// Every category a rule can name, in CLDR's order.
pub const CATEGORIES: [&str; 6] = ["zero", "one", "two", "few", "many", "other"];

/// CLDR's operands of a number written in decimal.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Operands {
    /// The absolute value.
    pub n: f64,
    /// Its integer digits.
    pub i: u64,
    /// How many fraction digits are written, and how many without the
    /// trailing zeros.
    pub v: u32,
    pub w: u32,
    /// The fraction digits as a number, with and without trailing zeros.
    pub f: u64,
    pub t: u64,
    /// The exponent of a compact number (`1.2c6`); 0 otherwise.
    pub e: u32,
}

impl Operands {
    /// The operands of `text`: `"3"`, `"1.50"`, `"-2"`, `"1.2c6"` (a
    /// compact number, as CLDR's samples write it). None when it is not a
    /// number.
    pub fn of(text: &str) -> Option<Operands> {
        let text = text.trim().trim_start_matches('-');
        let (mantissa, exponent) = match text.split_once(['c', 'e']) {
            Some((m, e)) => (m, e.parse::<u32>().ok()?),
            None => (text, 0),
        };
        let (int_part, frac_part) = mantissa.split_once('.').unwrap_or((mantissa, ""));
        if int_part.is_empty() && frac_part.is_empty()
            || !int_part.chars().all(|c| c.is_ascii_digit())
            || !frac_part.chars().all(|c| c.is_ascii_digit())
        {
            return None;
        }
        // A compact number moves the point: 1.2c6 is 1200000.
        let (int_part, frac_part) = if exponent > 0 {
            let mut digits = format!("{int_part}{frac_part}");
            let point = int_part.len() + exponent as usize;
            while digits.len() < point {
                digits.push('0');
            }
            let (a, b) = digits.split_at(point);
            (a.to_string(), b.to_string())
        } else {
            (int_part.to_string(), frac_part.to_string())
        };
        let trimmed = frac_part.trim_end_matches('0');
        let i: u64 = if int_part.is_empty() {
            0
        } else {
            int_part.parse().ok()?
        };
        let n: f64 = format!(
            "{}.{}",
            if int_part.is_empty() { "0" } else { &int_part },
            if frac_part.is_empty() {
                "0"
            } else {
                &frac_part
            }
        )
        .parse()
        .ok()?;
        Some(Operands {
            n,
            i,
            v: frac_part.len() as u32,
            w: trimmed.len() as u32,
            f: if frac_part.is_empty() {
                0
            } else {
                frac_part.parse().ok()?
            },
            t: if trimmed.is_empty() {
                0
            } else {
                trimmed.parse().ok()?
            },
            e: exponent,
        })
    }

    /// The operands of a number as JavaScript writes it, which is how the
    /// page reads a count: `1` and not `1.0`.
    pub fn of_number(n: f64) -> Operands {
        let text = if n.fract() == 0.0 && n.abs() < 1e15 {
            format!("{}", n.abs() as u64)
        } else {
            format!("{}", n.abs())
        };
        Operands::of(&text).unwrap_or(Operands {
            n: n.abs(),
            i: n.abs() as u64,
            v: 0,
            w: 0,
            f: 0,
            t: 0,
            e: 0,
        })
    }

    fn get(&self, name: char) -> Option<f64> {
        Some(match name {
            'n' => self.n,
            'i' => self.i as f64,
            'v' => self.v as f64,
            'w' => self.w as f64,
            'f' => self.f as f64,
            't' => self.t as f64,
            'c' | 'e' => self.e as f64,
            _ => return None,
        })
    }
}

/// The condition of a rule, without its samples.
fn condition(rule: &str) -> &str {
    rule.split('@').next().unwrap_or("").trim()
}

/// Whether `rule`'s condition holds for `x`. An empty condition (`other`'s)
/// holds for everything.
fn holds(rule: &str, x: &Operands) -> bool {
    let cond = condition(rule);
    if cond.is_empty() {
        return true;
    }
    cond.split(" or ").any(|and| {
        and.split(" and ")
            .all(|relation| relation_holds(relation.trim(), x))
    })
}

/// `i % 10 = 2..4,7` or `n != 1`.
fn relation_holds(relation: &str, x: &Operands) -> bool {
    let (left, right, negated) = match relation.split_once("!=") {
        Some((l, r)) => (l, r, true),
        None => match relation.split_once('=') {
            Some((l, r)) => (l, r, false),
            None => return false,
        },
    };
    let left = left.trim();
    let mut parts = left.splitn(2, '%');
    let operand = parts.next().unwrap_or("").trim();
    let Some(mut value) = operand.chars().next().and_then(|c| x.get(c)) else {
        return false;
    };
    if let Some(m) = parts.next() {
        let Ok(m) = m.trim().parse::<f64>() else {
            return false;
        };
        value %= m;
    }
    let integer = value.fract() == 0.0;
    let within = right.split(',').any(|item| {
        let item = item.trim();
        match item.split_once("..") {
            Some((a, b)) => {
                let (Ok(a), Ok(b)) = (a.trim().parse::<f64>(), b.trim().parse::<f64>()) else {
                    return false;
                };
                // A range holds integers only: `n = 2..4` is not 2.5.
                integer && a <= value && value <= b
            }
            None => item.parse::<f64>().is_ok_and(|v| v == value),
        }
    });
    within != negated
}

/// The rules for `locale`: its own, else its language's (`pt-PT`, then
/// `pt`), else CLDR's root, which has only `other`.
pub fn rules(locale: &str) -> &'static [(&'static str, &'static str)] {
    let find = |l: &str| {
        RULES
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case(l))
            .map(|(_, r)| *r)
    };
    let normal = locale.replace('_', "-");
    find(&normal)
        .or_else(|| find(normal.split('-').next().unwrap_or("")))
        .or_else(|| find("root"))
        .unwrap_or(&[("other", "")])
}

/// The forms `locale` uses, in CLDR's order: `["one", "other"]` for
/// English, all six for Arabic.
pub fn categories(locale: &str) -> Vec<&'static str> {
    rules(locale).iter().map(|(c, _)| *c).collect()
}

/// The form `x` takes in `locale`.
pub fn select(locale: &str, x: &Operands) -> &'static str {
    rules(locale)
        .iter()
        .find(|(cat, rule)| *cat != "other" && holds(rule, x))
        .map(|(c, _)| *c)
        .unwrap_or("other")
}

/// The form the number `n` takes in `locale`, as the page picks it.
pub fn select_number(locale: &str, n: f64) -> &'static str {
    select(locale, &Operands::of_number(n))
}

/// The samples CLDR gives for one category of `locale`'s rules: each
/// written as CLDR writes it (`1`, `1.0`, `1c6`), the ranges (`2~16`)
/// opened up a few at a time.
pub fn samples(locale: &str, category: &str) -> Vec<String> {
    let Some((_, rule)) = rules(locale).iter().find(|(c, _)| *c == category) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for part in rule.split('@').skip(1) {
        let body = part
            .trim_start_matches("integer")
            .trim_start_matches("decimal");
        for item in body.split(',') {
            let item = item.trim();
            if item.is_empty() || item == "…" {
                continue;
            }
            match item.split_once('~') {
                Some((a, b)) => {
                    // `0.0~1.5` steps by the last place; `2~16` by one.
                    let places = a.split_once('.').map(|(_, f)| f.len()).unwrap_or(0);
                    let step = 10f64.powi(-(places as i32));
                    let (Ok(from), Ok(to)) = (a.parse::<f64>(), b.parse::<f64>()) else {
                        out.push(a.to_string());
                        continue;
                    };
                    let mut x = from;
                    let mut taken = 0;
                    while x <= to + step / 2.0 && taken < 12 {
                        out.push(format!("{x:.places$}"));
                        x += step;
                        taken += 1;
                    }
                    out.push(b.to_string());
                }
                None => out.push(item.to_string()),
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_rules_pick_what_cldr_says_for_every_sample() {
        let mut wrong = Vec::new();
        let mut checked = 0;
        for (locale, cats) in RULES {
            for (cat, _) in *cats {
                for s in samples(locale, cat) {
                    let Some(x) = Operands::of(&s) else {
                        wrong.push(format!("{locale}: {s} does not read"));
                        continue;
                    };
                    checked += 1;
                    let got = select(locale, &x);
                    if got != *cat {
                        wrong.push(format!("{locale}: {s} is {got}, CLDR says {cat}"));
                    }
                }
            }
        }
        assert!(
            wrong.is_empty(),
            "{} wrong of {checked}:\n{}",
            wrong.len(),
            wrong
                .iter()
                .take(30)
                .cloned()
                .collect::<Vec<_>>()
                .join("\n")
        );
        assert!(checked > 5_000, "{checked} samples");
    }

    #[test]
    fn the_forms_each_language_uses() {
        assert_eq!(categories("en"), ["one", "other"]);
        assert_eq!(
            categories("ar"),
            ["zero", "one", "two", "few", "many", "other"]
        );
        assert_eq!(categories("ja"), ["other"]);
        assert_eq!(categories("pt-PT"), ["one", "many", "other"]);
        assert_eq!(
            categories("ar-EG"),
            categories("ar"),
            "a region falls back to its language"
        );
        assert_eq!(
            categories("xx"),
            ["other"],
            "an unknown language has only `other`"
        );
        assert_eq!(select_number("en", 1.0), "one");
        assert_eq!(select_number("en", 0.0), "other");
        assert_eq!(select("en", &Operands::of("1.0").unwrap()), "other");
        assert_eq!(select_number("ar", 0.0), "zero");
        assert_eq!(select_number("ar", 2.0), "two");
        assert_eq!(select_number("ar", 105.0), "few");
        assert_eq!(select_number("ar", 111.0), "many");
        assert_eq!(select_number("ar", 100.0), "other");
        assert_eq!(select_number("pl", 22.0), "few");
        assert_eq!(select_number("pl", 25.0), "many");
        assert_eq!(select_number("ru", 21.0), "one");
        assert_eq!(select_number("fr", 0.0), "one");
        assert_eq!(select_number("fr", 1_000_000.0), "many");
    }
}
