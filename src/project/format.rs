//! `format(value, style, option)` and `ago(date)` at build time.
//!
//! The browser formats with `Intl`; the static paint and a template have no
//! `Intl`, so this is the same surface over tables read off it:
//! `format_locales.rs`, which `tests/js/format-tables.mjs` writes from the
//! ICU that Node carries — each locale's digits, grouping, decimal mark,
//! minus sign, percent and currency patterns, currency symbols, compact
//! notation, and its date and time patterns with their month, weekday and
//! day-period names. `tests/build_and_runtime_agree.rs` holds every style
//! to the runtime's `Intl`, locale by locale.
//!
//! What still differs from the live page, and why:
//!
//! - A tag the table does not know is written as its language is (`de-LU`
//!   as `de`), and a language it does not know as English; the page
//!   corrects itself once it is live.
//! - A date is the moment as written: the static paint knows no reader's
//!   time zone, so `DateTime`s show in UTC (and `.time` in the `long` and
//!   `full` styles names UTC) where the browser shows the reader's zone.
//! - Only the Gregorian calendar is reckoned: a locale whose default
//!   calendar is another (`fa`, `th`) is not in the table.
//! - `ago(date)` and `.relative` are spelled in English whatever the
//!   locale; the page speaks the reader's language once it runs.
//! - The tables are one ICU's (see the head of `format_locales.rs`); a
//!   browser on another release may spell a rare case differently.

/// A value to format: a number, or a date read from its text.
#[derive(Debug, Clone, PartialEq)]
pub enum Input {
    Number(f64),
    Text(String),
}

/// A calendar moment, in the writer's terms: the static paint has no zone,
/// so a date is taken as written.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Moment {
    pub year: i64,
    pub month: u32,
    pub day: u32,
    pub hour: u32,
    pub minute: u32,
    pub second: u32,
}

/// How a locale writes numbers. In a pattern, `#` is the number, `¤` the
/// currency and `-` the minus sign; anything else is written as it is.
struct Numbers {
    /// The digit zero; the other nine follow it.
    zero: char,
    group: &'static str,
    decimal: &'static str,
    minus: &'static str,
    /// What goes before a negative number: the sign, and the marks that
    /// keep it on the number's side in right-to-left text.
    negative: &'static str,
    /// Groups of two above the first three: `12,34,567`.
    indian: bool,
    /// How many digits the first group must have before any is set off:
    /// 2 writes `1234` and `12.345`.
    min_grouping: u8,
    percent: &'static str,
    percent_negative: &'static str,
    currency: &'static str,
    currency_negative: &'static str,
    /// The marks an amount of money is grouped and divided with, where they
    /// are not the number's.
    currency_group: &'static str,
    currency_decimal: &'static str,
    /// Each currency the locale writes other than by its code.
    symbols: &'static [(&'static str, &'static str)],
    /// The compact notation, from the smallest magnitude it shortens.
    compact: &'static [Compact],
}

/// The compact notation at one magnitude (`10^magnitude` and up): the
/// power it divides by, and the pattern of each plural form.
struct Compact {
    magnitude: u8,
    scale: u8,
    other: &'static str,
    forms: &'static [(&'static str, &'static str)],
}

/// How a locale writes dates and times: the patterns of each style
/// (`short`, `medium`, `long`, `full`), quoted where they hold letters,
/// and the names they use. The `standalone_` names are the ones a pattern
/// of the program's own (`MMMM yyyy`) takes, as `Intl` gives a month or a
/// weekday alone.
struct Dates {
    date: [&'static str; 4],
    time: [&'static str; 4],
    datetime: [&'static str; 4],
    months: [[&'static str; 12]; 2],
    weekdays: [&'static str; 7],
    /// The day period of each hour (`AM`, `PM`; `凌晨`, `上午`, `中午`…).
    periods: [&'static str; 24],
    standalone_months: [[&'static str; 12]; 2],
    standalone_weekdays: [[&'static str; 7]; 2],
    standalone_periods: [&'static str; 24],
}

include!("format_locales.rs");

/// A locale tag, resolved against the table.
struct Locale {
    language: String,
    numbers: &'static Numbers,
    dates: &'static Dates,
    /// The digit zero the tag's `-u-nu-` names, when it names one.
    explicit_zero: Option<char>,
}

impl Locale {
    fn zero(&self) -> char {
        self.explicit_zero.unwrap_or(self.numbers.zero)
    }
}

/// The digit zero of a numbering system an `-u-nu-` extension names.
fn numbering_zero(system: &str) -> Option<char> {
    Some(match system {
        "latn" => '0',
        "arab" => '\u{660}',
        "arabext" => '\u{6f0}',
        "deva" => '\u{966}',
        "beng" => '\u{9e6}',
        "thai" => '\u{e50}',
        "fullwide" => '\u{ff10}',
        _ => return None,
    })
}

/// `locale` read against the table: `lang-Script-REGION`, then
/// `lang-Script`, `lang-REGION` and `lang`, then English.
fn resolve(locale: &str) -> Locale {
    let lower = locale.replace('_', "-").to_ascii_lowercase();
    let (base, extension) = match lower.find("-u-") {
        Some(at) => (&lower[..at], Some(&lower[at + 3..])),
        None => (lower.as_str(), None),
    };
    let system = extension.and_then(|ext| {
        let keys: Vec<&str> = ext.split('-').collect();
        keys.windows(2)
            .find(|w| w[0] == "nu")
            .map(|w| w[1].to_string())
    });
    let mut parts = base.split('-');
    let language = parts.next().unwrap_or("en").to_string();
    let mut script = None;
    let mut region = None;
    for part in parts {
        if part.len() == 4 && script.is_none() && region.is_none() {
            let mut s = part.to_string();
            s[..1].make_ascii_uppercase();
            script = Some(s);
        } else if (part.len() == 2 && part.chars().all(|c| c.is_ascii_alphabetic()))
            || (part.len() == 3 && part.chars().all(|c| c.is_ascii_digit()))
        {
            region = Some(part.to_ascii_uppercase());
            break;
        }
    }
    let mut candidates = Vec::new();
    if let (Some(s), Some(r)) = (&script, &region) {
        candidates.push(format!("{language}-{s}-{r}"));
    }
    if let Some(s) = &script {
        candidates.push(format!("{language}-{s}"));
    }
    if let Some(r) = &region {
        candidates.push(format!("{language}-{r}"));
    }
    candidates.push(language.clone());
    let found = candidates
        .iter()
        .find_map(|c| LOCALES.iter().find(|(tag, _, _)| tag == c));
    let (language, numbers, dates) = if let Some((_, n, d)) = found {
        (language, *n, *d)
    } else {
        let (_, n, d) = LOCALES[0];
        ("en".to_string(), n, d)
    };
    let explicit_zero = system.as_deref().and_then(numbering_zero);
    // A numbering system the language writes its own symbols for (Arabic's
    // `٬` and `٫` with its digits, `,` and `.` with Latin ones).
    let numbers = system
        .as_deref()
        .and_then(|s| {
            let key = format!("{language}-u-nu-{s}");
            NUMBERING.iter().find(|(tag, _)| *tag == key)
        })
        .map_or(numbers, |(_, n)| *n);
    Locale {
        language,
        numbers,
        dates,
        explicit_zero,
    }
}

/// `text` with its ASCII digits written from `zero`.
fn digits_from(text: &str, zero: char) -> String {
    if zero == '0' {
        return text.to_string();
    }
    text.chars()
        .map(|c| match c.to_digit(10) {
            Some(d) if c.is_ascii_digit() => char::from_u32(zero as u32 + d).unwrap_or(c),
            _ => c,
        })
        .collect()
}

/// A number as the decimal digits it is written with — the shortest that
/// reads back as the same `f64`, which is the decimal `Intl` rounds — with
/// the point after `point` of them.
#[derive(Debug, Clone)]
struct Decimal {
    digits: Vec<u8>,
    point: i32,
}

impl Decimal {
    /// The magnitude of a finite number, without its sign.
    fn of(value: f64) -> Decimal {
        // `Display` for an `f64` is the shortest round-trip text and never
        // an exponent.
        let text = format!("{}", value.abs());
        let (whole, frac) = text.split_once('.').unwrap_or((&text, ""));
        let mut digits: Vec<u8> = whole
            .bytes()
            .chain(frac.bytes())
            .map(|b| b - b'0')
            .collect();
        let mut point = whole.len() as i32;
        while digits.first() == Some(&0) {
            digits.remove(0);
            point -= 1;
        }
        while digits.last() == Some(&0) {
            digits.pop();
        }
        if digits.is_empty() {
            point = 0;
        }
        Decimal { digits, point }
    }

    fn is_zero(&self) -> bool {
        self.digits.is_empty()
    }

    /// Ten to the `by` times the number.
    fn shifted(&self, by: i32) -> Decimal {
        if self.is_zero() {
            return self.clone();
        }
        Decimal {
            digits: self.digits.clone(),
            point: self.point + by,
        }
    }

    /// Kept to `keep` digits from the first, a half rounded away from zero
    /// as `Intl` rounds it.
    fn keep(&self, keep: i32) -> Decimal {
        let len = self.digits.len() as i32;
        if keep >= len {
            return self.clone();
        }
        if keep < 0 {
            return Decimal {
                digits: Vec::new(),
                point: 0,
            };
        }
        let up = self.digits[keep as usize] >= 5;
        let mut digits = self.digits[..keep as usize].to_vec();
        let mut point = self.point;
        if up {
            let mut i = digits.len();
            loop {
                if i == 0 {
                    digits.insert(0, 1);
                    point += 1;
                    break;
                }
                i -= 1;
                if digits[i] == 9 {
                    digits[i] = 0;
                } else {
                    digits[i] += 1;
                    break;
                }
            }
        }
        while digits.last() == Some(&0) {
            digits.pop();
        }
        if digits.is_empty() {
            point = 0;
        }
        Decimal { digits, point }
    }

    /// Rounded to `places` decimals.
    fn places(&self, places: usize) -> Decimal {
        if self.is_zero() {
            return self.clone();
        }
        self.keep(self.point + places as i32)
    }

    /// Rounded to `count` significant digits.
    fn significant(&self, count: i32) -> Decimal {
        self.keep(count)
    }

    /// The whole part's digits and the fraction's, at least `min` places.
    fn split(&self, min: usize) -> (String, String) {
        let digit = |i: i32| -> char {
            if i >= 0 && (i as usize) < self.digits.len() {
                (b'0' + self.digits[i as usize]) as char
            } else {
                '0'
            }
        };
        let whole: String = if self.point <= 0 {
            "0".to_string()
        } else {
            (0..self.point).map(digit).collect()
        };
        let frac_len = (self.digits.len() as i32 - self.point).max(0) as usize;
        let frac: String = (0..frac_len.max(min) as i32)
            .map(|i| digit(self.point + i))
            .collect();
        (whole, frac)
    }
}

impl Numbers {
    /// `digits` (ASCII) grouped, with the decimal mark and the locale's
    /// digits.
    fn write(&self, value: &Decimal, min_places: usize, min_grouping: u8, zero: char) -> String {
        self.write_with(
            value,
            min_places,
            min_grouping,
            zero,
            (self.group, self.decimal),
        )
    }

    fn write_with(
        &self,
        value: &Decimal,
        min_places: usize,
        min_grouping: u8,
        zero: char,
        (group, decimal): (&str, &str),
    ) -> String {
        let (whole, frac) = value.split(min_places);
        let len = whole.len();
        let mut grouped = String::new();
        let grouping = len >= 3 + usize::from(min_grouping);
        for (i, c) in whole.chars().enumerate() {
            let left = len - i;
            if i > 0 && grouping {
                let boundary = if self.indian {
                    left == 3 || (left > 3 && (left - 3) % 2 == 0)
                } else {
                    left % 3 == 0
                };
                if boundary {
                    grouped.push_str(group);
                }
            }
            grouped.push(c);
        }
        let mut out = digits_from(&grouped, zero);
        if !frac.is_empty() {
            out.push_str(decimal);
            out.push_str(&digits_from(&frac, zero));
        }
        out
    }

    /// A pattern with its number, currency and sign put in.
    fn fill(&self, pattern: &str, number: &str, symbol: &str) -> String {
        let mut out = String::new();
        let chars: Vec<char> = pattern.chars().collect();
        for (i, c) in chars.iter().enumerate() {
            match c {
                '#' => out.push_str(number),
                '-' => out.push_str(self.minus),
                '¤' => {
                    // A symbol of letters touching the digits is set off
                    // from them, as ICU's currency spacing does: `SAR 12.00`.
                    let before = i > 0 && chars[i - 1] == '#';
                    let after = chars.get(i + 1) == Some(&'#');
                    if before && symbol.chars().next().is_some_and(char::is_alphabetic) {
                        out.push('\u{a0}');
                    }
                    out.push_str(symbol);
                    if after && symbol.chars().last().is_some_and(char::is_alphabetic) {
                        out.push('\u{a0}');
                    }
                }
                c => out.push(*c),
            }
        }
        out
    }
}

/// What is written for a number that is not finite.
fn not_finite(value: f64, locale: &Locale) -> String {
    if value.is_nan() {
        "NaN".to_string()
    } else if value > 0.0 {
        "∞".to_string()
    } else {
        format!("{}∞", locale.numbers.negative)
    }
}

/// `1234.5` with `places` decimals (up to three, trailing zeros dropped,
/// when `None`), grouped for the locale.
pub fn number(value: f64, places: Option<usize>, locale: &str) -> String {
    let locale = resolve(locale);
    write_number(value, places, places.unwrap_or(0), &locale)
}

fn write_number(value: f64, max: Option<usize>, min: usize, locale: &Locale) -> String {
    if !value.is_finite() {
        return not_finite(value, locale);
    }
    // `Intl.NumberFormat` takes at most 100 fraction digits; a number of
    // places from the data is held to that.
    let max = max.unwrap_or(3).min(100);
    let min = min.min(max);
    let n = locale.numbers;
    let text = n.write(
        &Decimal::of(value).places(max),
        min,
        n.min_grouping,
        locale.zero(),
    );
    if value.is_sign_negative() {
        format!("{}{text}", n.negative)
    } else {
        text
    }
}

fn percent(value: f64, places: usize, locale: &Locale) -> String {
    if !value.is_finite() {
        return not_finite(value, locale);
    }
    let places = places.min(100);
    let n = locale.numbers;
    let digits = n.write(
        &Decimal::of(value).shifted(2).places(places),
        places,
        n.min_grouping,
        locale.zero(),
    );
    let pattern = if value.is_sign_negative() {
        n.percent_negative
    } else {
        n.percent
    };
    n.fill(pattern, &digits, "")
}

/// The places a currency's amounts are written with.
fn currency_places(code: &str) -> usize {
    CURRENCY_DIGITS
        .iter()
        .find(|(c, _)| *c == code)
        .map_or(2, |(_, d)| *d)
}

fn currency(value: f64, code: &str, locale: &Locale) -> String {
    if !value.is_finite() {
        return not_finite(value, locale);
    }
    let n = locale.numbers;
    let places = currency_places(code);
    let digits = n.write_with(
        &Decimal::of(value).places(places),
        places,
        n.min_grouping,
        locale.zero(),
        (n.currency_group, n.currency_decimal),
    );
    let symbol = n
        .symbols
        .iter()
        .find(|(c, _)| *c == code)
        .map_or(code, |(_, s)| *s);
    let pattern = if value.is_sign_negative() {
        n.currency_negative
    } else {
        n.currency
    };
    n.fill(pattern, &digits, symbol)
}

/// The plural form `Intl.PluralRules` gives a number the compact notation
/// shows, for the languages whose short words change with it (Arabic's
/// `3 آلاف` against `11 ألف`).
fn plural(language: &str, value: &Decimal) -> &'static str {
    if language != "ar" {
        return "other";
    }
    let (whole, frac) = value.split(0);
    if !frac.is_empty() {
        return "other";
    }
    let n: u64 = whole.parse().unwrap_or(0);
    match n % 100 {
        _ if n == 0 => "zero",
        _ if n == 1 => "one",
        _ if n == 2 => "two",
        3..=10 => "few",
        11..=99 => "many",
        _ => "other",
    }
}

/// `1.2K`, `3.4M`: the compact notation.
fn compact(value: f64, locale: &Locale) -> String {
    if !value.is_finite() {
        return not_finite(value, locale);
    }
    let n = locale.numbers;
    let mut decimal = Decimal::of(value);
    let mut entry: Option<&Compact>;
    let mut shown;
    // Once more when rounding carries into the next magnitude: 999,950 is
    // `1M`, not `1000K`.
    let mut tries = 0;
    loop {
        let magnitude = decimal.point - 1;
        entry = n
            .compact
            .iter()
            .rev()
            .find(|c| i32::from(c.magnitude) <= magnitude);
        let scale = entry.map_or(0, |c| i32::from(c.scale));
        let scaled = decimal.shifted(-scale);
        // Two significant digits below ten, a whole number above.
        shown = if scaled.point >= 2 {
            scaled.places(0)
        } else {
            scaled.significant(2)
        };
        tries += 1;
        if tries < 2 && !shown.is_zero() && shown.point - 1 + scale > magnitude {
            decimal = shown.shifted(scale);
            continue;
        }
        break;
    }
    let digits = n.write(&shown, 0, n.min_grouping.max(2), locale.zero());
    let pattern = entry.map_or("#", |c| {
        let form = plural(&locale.language, &shown);
        c.forms
            .iter()
            .find(|(f, _)| *f == form)
            .map_or(c.other, |(_, p)| *p)
    });
    let text = n.fill(pattern, &digits, "");
    if value.is_sign_negative() {
        format!("{}{text}", n.negative)
    } else {
        text
    }
}

/// Whether `locale` writes its digits as Arabic-Indic ones (`١٢٣`), as
/// `Intl` does.
pub fn arabic_digits(locale: &str) -> bool {
    resolve(locale).zero() == '\u{660}'
}

/// `2024-03-05`, `2024-03-05T14:30`, `2024-03-05T14:30:00Z`, or a Unix time
/// in milliseconds.
pub fn parse_moment(input: &Input) -> Option<Moment> {
    match input {
        Input::Number(ms) => Some(from_unix((*ms / 1000.0).floor() as i64)),
        Input::Text(text) => {
            let text = text.trim();
            let (date, time) = match text.split_once(['T', ' ']) {
                Some((d, t)) => (d, Some(t)),
                None => (text, None),
            };
            let mut parts = date.splitn(3, '-');
            let year: i64 = parts.next()?.parse().ok()?;
            let month: u32 = parts.next()?.parse().ok()?;
            let day: u32 = parts.next()?.parse().ok()?;
            if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
                return None;
            }
            let (hour, minute, second) = match time {
                Some(t) => {
                    let t = t.trim_end_matches('Z');
                    let t = t.split(['+', '.']).next().unwrap_or(t);
                    // A `-` after the time is a zone offset; the date's own
                    // dashes were split off already.
                    let t = t.split('-').next().unwrap_or(t);
                    let mut hms = t.split(':');
                    let h: u32 = hms.next()?.trim().parse().ok()?;
                    let m: u32 = hms.next().unwrap_or("0").parse().ok()?;
                    let s: u32 = hms.next().unwrap_or("0").parse().ok()?;
                    (h, m, s)
                }
                None => (0, 0, 0),
            };
            Some(Moment {
                year,
                month,
                day,
                hour,
                minute,
                second,
            })
        }
    }
}

/// Days since 1970-01-01 of a civil date (Howard Hinnant's algorithm).
fn days_from_civil(y: i64, m: u32, d: u32) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = (i64::from(m) + 9) % 12;
    let doy = (153 * mp + 2) / 5 + i64::from(d) - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

fn from_unix(seconds: i64) -> Moment {
    let days = seconds.div_euclid(86400);
    let rem = seconds.rem_euclid(86400);
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    Moment {
        year: if m <= 2 { y + 1 } else { y },
        month: m,
        day: d,
        hour: (rem / 3600) as u32,
        minute: ((rem % 3600) / 60) as u32,
        second: (rem % 60) as u32,
    }
}

/// `seconds` since the epoch as the ISO 8601 text a `DateTime` carries:
/// `2026-11-19T09:30:00Z`.
pub fn iso(seconds: i64) -> String {
    let m = from_unix(seconds);
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        m.year, m.month, m.day, m.hour, m.minute, m.second
    )
}

/// A field a date pattern names, read as the runtime's `formatPattern`
/// reads one: the longest of these at each place, anything else as it is.
const FIELDS: &[&str] = &[
    "yyyy", "yy", "MMMM", "MMM", "MM", "M", "dd", "d", "EEEE", "EEE", "HH", "H", "hh", "h", "mm",
    "ss", "a",
];

/// The fields of the locale's own patterns, which also write a minute or
/// a second unpadded (`12시 0분 0초`).
const STYLE_FIELDS: &[&str] = &[
    "yyyy", "yy", "MMMM", "MMM", "MM", "M", "dd", "d", "EEEE", "EEE", "HH", "H", "hh", "h", "mm",
    "m", "ss", "s", "a",
];

/// The names a pattern's fields are written with.
struct Names<'a> {
    fields: &'static [&'static str],
    months: [&'a [&'static str; 12]; 2],
    weekdays: [&'a [&'static str; 7]; 2],
    periods: &'a [&'static str; 24],
    /// The digit zero figures are written from.
    zero: char,
}

impl Moment {
    /// Seconds since the epoch, the moment taken as UTC.
    pub fn unix(&self) -> i64 {
        days_from_civil(self.year, self.month, self.day) * 86400
            + i64::from(self.hour) * 3600
            + i64::from(self.minute) * 60
            + i64::from(self.second)
    }

    /// 0 = Sunday.
    fn weekday(&self) -> usize {
        let days = days_from_civil(self.year, self.month, self.day);
        ((days + 4).rem_euclid(7)) as usize
    }

    /// The moment written to a pattern: `yyyy-MM-dd`, `d MMM yyyy`,
    /// `EEEE, HH:mm`, `h:mm a` — in English.
    pub fn pattern(&self, pattern: &str) -> String {
        self.pattern_in(pattern, "en")
    }

    /// The moment written to a pattern of the program's own, as the
    /// runtime writes it: its month and weekday names and its AM and PM
    /// those `Intl` gives the locale, and its figures in the digits an
    /// `-u-nu-` extension names (0–9 otherwise).
    pub fn pattern_in(&self, pattern: &str, locale: &str) -> String {
        let locale = resolve(locale);
        let d = locale.dates;
        let names = Names {
            fields: FIELDS,
            months: [&d.standalone_months[0], &d.standalone_months[1]],
            weekdays: [&d.standalone_weekdays[0], &d.standalone_weekdays[1]],
            periods: &d.standalone_periods,
            zero: locale.explicit_zero.unwrap_or('0'),
        };
        self.fields(pattern, &names)
    }

    /// The moment written to one of the locale's own patterns, whose
    /// literal text is quoted (`d' de 'MMMM`), with the names a date of
    /// that style takes and the locale's digits.
    fn styled(&self, pattern: &str, locale: &Locale) -> String {
        let d = locale.dates;
        let names = Names {
            fields: STYLE_FIELDS,
            months: [&d.months[0], &d.months[1]],
            weekdays: [&d.weekdays, &d.standalone_weekdays[1]],
            periods: &d.periods,
            zero: locale.zero(),
        };
        let mut out = String::new();
        let mut rest = pattern;
        while let Some(at) = rest.find('\'') {
            out.push_str(&self.fields(&rest[..at], &names));
            let quoted = &rest[at + 1..];
            let end = quoted.find('\'').unwrap_or(quoted.len());
            if end == 0 {
                out.push('\'');
            } else {
                out.push_str(&quoted[..end]);
            }
            rest = quoted.get(end + 1..).unwrap_or("");
        }
        out.push_str(&self.fields(rest, &names));
        out
    }

    fn fields(&self, pattern: &str, names: &Names<'_>) -> String {
        let h12 = match self.hour % 12 {
            0 => 12,
            h => h,
        };
        let figures = |n: String| digits_from(&n, names.zero);
        let mut out = String::new();
        let mut rest = pattern;
        while let Some(c) = rest.chars().next() {
            let Some(field) = names.fields.iter().find(|f| rest.starts_with(**f)) else {
                out.push(c);
                rest = &rest[c.len_utf8()..];
                continue;
            };
            out.push_str(&match *field {
                "yyyy" => figures(self.year.to_string()),
                "yy" => figures(format!("{:02}", self.year.rem_euclid(100))),
                "MMMM" => names.months[0][self.month as usize - 1].to_string(),
                "MMM" => names.months[1][self.month as usize - 1].to_string(),
                "MM" => figures(format!("{:02}", self.month)),
                "M" => figures(self.month.to_string()),
                "dd" => figures(format!("{:02}", self.day)),
                "d" => figures(self.day.to_string()),
                "EEEE" => names.weekdays[0][self.weekday()].to_string(),
                "EEE" => names.weekdays[1][self.weekday()].to_string(),
                "HH" => figures(format!("{:02}", self.hour)),
                "H" => figures(self.hour.to_string()),
                "hh" => figures(format!("{h12:02}")),
                "h" => figures(h12.to_string()),
                "mm" => figures(format!("{:02}", self.minute)),
                "m" => figures(self.minute.to_string()),
                "ss" => figures(format!("{:02}", self.second)),
                "s" => figures(self.second.to_string()),
                _ => names.periods[self.hour as usize % 24].to_string(),
            });
            rest = &rest[field.len()..];
        }
        out
    }
}

/// The styles `format(value, .style, option)` takes, each with what it
/// shows — for a tool to offer; a test holds them to [`format()`].
pub const STYLES: &[(&str, &str)] = &[
    ("number", "a number in the locale's way: 1,234.5"),
    ("integer", "a whole number: 1,235"),
    ("decimal", "a fixed number of places (option: places, 2)"),
    ("currency", "money (option: the currency code, USD)"),
    ("percent", "a fraction as a percentage (option: places, 0)"),
    ("compact", "a short number: 1.2K"),
    ("date", "a date (option: short, medium, long, full)"),
    ("time", "a time of day (option: short, medium, long, full)"),
    (
        "datetime",
        "a date and a time (option: short, medium, long, full)",
    ),
    ("relative", "how long ago or from now: 3 minutes ago"),
];

/// `format(value, style, option)`: `None` when the value cannot be read as
/// what the style needs.
pub fn format(
    value: &Input,
    style: Option<&str>,
    option: Option<&Input>,
    locale: &str,
) -> Option<String> {
    let option_text = |o: Option<&Input>| match o {
        Some(Input::Text(t)) => Some(t.clone()),
        Some(Input::Number(n)) => Some(n.to_string()),
        None => None,
    };
    let option_places = |o: Option<&Input>, default: usize| match o {
        Some(Input::Number(n)) => *n as usize,
        Some(Input::Text(t)) => t.parse().unwrap_or(default),
        None => default,
    };
    let as_number = |v: &Input| match v {
        Input::Number(n) => Some(*n),
        Input::Text(t) => t.trim().parse::<f64>().ok(),
    };
    // The index of a `dateStyle` or `timeStyle`.
    let style_index = |o: Option<&Input>, default: usize| match option_text(o).as_deref() {
        Some("short") => 0,
        Some("medium") => 1,
        Some("long") => 2,
        Some("full") => 3,
        _ => default,
    };
    let tag = locale;
    let resolved = resolve(tag);
    let locale = &resolved;
    match style {
        // Text that is not a number is shown as it is: `format(label)` of
        // what a page already wrote out, `Stat(value: "$1,234.50")`.
        None if matches!(value, Input::Text(t) if as_number(value).is_none() && !t.trim().is_empty()) => {
            Some(match value {
                Input::Text(t) => t.clone(),
                Input::Number(n) => n.to_string(),
            })
        }
        None | Some("number") => Some(write_number(as_number(value)?, None, 0, locale)),
        Some("integer") => Some(write_number(as_number(value)?, Some(0), 0, locale)),
        Some("decimal") => {
            let places = option_places(option, 2);
            Some(write_number(
                as_number(value)?,
                Some(places),
                places,
                locale,
            ))
        }
        Some("currency") => {
            let code = option_text(option).unwrap_or_else(|| "USD".to_string());
            Some(currency(
                as_number(value)?,
                &code.to_ascii_uppercase(),
                locale,
            ))
        }
        Some("percent") => Some(percent(as_number(value)?, option_places(option, 0), locale)),
        Some("compact") => Some(compact(as_number(value)?, locale)),
        Some("date") => {
            let m = parse_moment(value)?;
            Some(m.styled(locale.dates.date[style_index(option, 1)], locale))
        }
        Some("time") => {
            let m = parse_moment(value)?;
            Some(m.styled(locale.dates.time[style_index(option, 0)], locale))
        }
        Some("datetime") => {
            let m = parse_moment(value)?;
            Some(m.styled(locale.dates.datetime[style_index(option, 1)], locale))
        }
        Some("relative") => ago(value, None),
        // Anything else is a date pattern.
        Some(pattern) => Some(parse_moment(value)?.pattern_in(pattern, tag)),
    }
}

/// `ago(date)`: "3 minutes ago", "yesterday", "in 2 weeks", against `now`
/// (the build's moment when none is given).
pub fn ago(value: &Input, now: Option<&Input>) -> Option<String> {
    let then = parse_moment(value)?.unix();
    let now = match now {
        Some(n) => parse_moment(n)?.unix(),
        None => crate::base::platform::unix_now(),
    };
    Some(relative(then - now))
}

/// The English spelling `Intl.RelativeTimeFormat("en", { numeric: "auto" })`
/// gives a difference of `seconds`.
pub fn relative(seconds: i64) -> String {
    const UNITS: [(&str, i64, &str, &str); 6] = [
        ("year", 31_536_000, "last year", "next year"),
        ("month", 2_592_000, "last month", "next month"),
        ("week", 604_800, "last week", "next week"),
        ("day", 86_400, "yesterday", "tomorrow"),
        ("hour", 3_600, "1 hour ago", "in 1 hour"),
        ("minute", 60, "1 minute ago", "in 1 minute"),
    ];
    for (unit, size, one_ago, one_ahead) in UNITS {
        if seconds.abs() >= size {
            let n = seconds / size;
            return match n {
                -1 => one_ago.to_string(),
                1 => one_ahead.to_string(),
                n if n < 0 => format!("{} {unit}s ago", -n),
                n => format!("in {n} {unit}s"),
            };
        }
    }
    if seconds.abs() < 45 {
        return "now".to_string();
    }
    if seconds < 0 {
        format!("{} seconds ago", -seconds)
    } else {
        format!("in {seconds} seconds")
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn every_style_a_tool_offers_is_one_format_shows() {
        for (style, _) in STYLES {
            let value = if matches!(*style, "date" | "time" | "datetime" | "relative") {
                t("2026-03-14T09:30:00Z")
            } else {
                n(1234.5)
            };
            assert!(
                format(&value, Some(style), None, "en").is_some(),
                "`.{style}` is offered and shows nothing"
            );
        }
    }

    use super::*;

    fn n(v: f64) -> Input {
        Input::Number(v)
    }
    fn t(v: &str) -> Input {
        Input::Text(v.to_string())
    }

    /// The static paint of an Arabic page wrote its dates in English
    /// ("November 12, 2026") while the live page wrote them in Arabic.
    #[test]
    fn arabic_dates_and_digits_are_what_intl_writes() {
        let day = t("2026-11-12T14:05:00Z");
        let date = |style: &str, locale: &str| {
            format(&day, Some("date"), Some(&t(style)), locale).unwrap()
        };
        assert_eq!(date("long", "ar"), "12 نوفمبر 2026");
        assert_eq!(date("full", "ar"), "الخميس، 12 نوفمبر 2026");
        assert_eq!(date("short", "ar"), "12\u{200f}/11\u{200f}/2026");
        assert_eq!(date("long", "ar-EG"), "١٢ نوفمبر ٢٠٢٦");
        assert_eq!(date("long", "ar-u-nu-arab"), "١٢ نوفمبر ٢٠٢٦");
        assert_eq!(date("long", "ar-SY"), "١٢ تشرين الثاني ٢٠٢٦");
        assert_eq!(date("long", "en"), "November 12, 2026");
        assert_eq!(format(&day, Some("time"), None, "ar").unwrap(), "2:05 م");
        assert_eq!(
            format(&day, Some("datetime"), Some(&t("long")), "ar").unwrap(),
            "12 نوفمبر 2026 في 2:05 م"
        );
        assert_eq!(format(&n(1234.5), None, None, "ar").unwrap(), "1,234.5");
        assert_eq!(format(&n(1234.5), None, None, "ar-EG").unwrap(), "١٬٢٣٤٫٥");
        assert_eq!(format(&n(1234.5), None, None, "ar-MA").unwrap(), "1.234,5");
        assert_eq!(
            format(&n(2000.0), Some("compact"), None, "ar-EG").unwrap(),
            "٢\u{a0}ألف"
        );
        assert_eq!(
            format(&n(3000.0), Some("compact"), None, "ar").unwrap(),
            "3\u{a0}آلاف"
        );
    }

    /// The build wrote every currency before its amount, so a German price
    /// was painted `€1.234,50` and became `1.234,50 €` when the page ran;
    /// it took `ch` (Chamorro) for Swiss German, and grouped four Spanish
    /// digits that `Intl` leaves alone.
    #[test]
    fn money_and_grouping_follow_the_locale() {
        let eur =
            |locale: &str| format(&n(1234.5), Some("currency"), Some(&t("EUR")), locale).unwrap();
        assert_eq!(eur("de"), "1.234,50\u{a0}€");
        assert_eq!(eur("fr"), "1\u{202f}234,50\u{a0}€");
        assert_eq!(eur("nl"), "€\u{a0}1.234,50");
        assert_eq!(eur("de-CH"), "EUR\u{a0}1'234.50");
        assert_eq!(eur("en"), "€1,234.50");
        assert_eq!(format(&n(1234.0), None, None, "es").unwrap(), "1234");
        assert_eq!(format(&n(12345.0), None, None, "es").unwrap(), "12.345");
        assert_eq!(
            format(&n(-9_876_543.21), None, None, "hi").unwrap(),
            "-98,76,543.21"
        );
        assert_eq!(
            format(&n(0.25), Some("percent"), None, "tr").unwrap(),
            "%25"
        );
        assert_eq!(
            format(&n(1.005), Some("decimal"), None, "en").unwrap(),
            "1.01"
        );
    }

    #[test]
    fn numbers_group_and_round_for_the_locale() {
        assert_eq!(
            format(&n(1_234_567.891), None, None, "en").unwrap(),
            "1,234,567.891"
        );
        assert_eq!(
            format(&n(1234.5), Some("integer"), None, "en").unwrap(),
            "1,235"
        );
        assert_eq!(
            format(&n(1234.5), Some("decimal"), None, "en").unwrap(),
            "1,234.50"
        );
        assert_eq!(
            format(&n(1234.5), Some("decimal"), Some(&n(0.0)), "en").unwrap(),
            "1,235"
        );
        assert_eq!(
            format(&n(1234.5), Some("decimal"), None, "de-DE").unwrap(),
            "1.234,50"
        );
        assert_eq!(
            format(&n(1234.5), Some("decimal"), None, "fr").unwrap(),
            "1\u{202f}234,50"
        );
        assert_eq!(
            format(&n(-0.001), Some("integer"), None, "en").unwrap(),
            "-0"
        );
    }

    #[test]
    fn currency_percent_and_compact() {
        assert_eq!(
            format(&n(1234.5), Some("currency"), None, "en").unwrap(),
            "$1,234.50"
        );
        assert_eq!(
            format(&n(-9.99), Some("currency"), Some(&t("EUR")), "en").unwrap(),
            "-€9.99"
        );
        assert_eq!(
            format(&n(1500.0), Some("currency"), Some(&t("JPY")), "en").unwrap(),
            "¥1,500"
        );
        assert_eq!(
            format(&n(12.0), Some("currency"), Some(&t("SAR")), "en").unwrap(),
            "SAR\u{a0}12.00"
        );
        assert_eq!(
            format(&n(0.256), Some("percent"), None, "en").unwrap(),
            "26%"
        );
        assert_eq!(
            format(&n(0.256), Some("percent"), Some(&n(1.0)), "en").unwrap(),
            "25.6%"
        );
        assert_eq!(
            format(&n(1234.0), Some("compact"), None, "en").unwrap(),
            "1.2K"
        );
        assert_eq!(
            format(&n(12345.0), Some("compact"), None, "en").unwrap(),
            "12K"
        );
        assert_eq!(
            format(&n(2_000_000.0), Some("compact"), None, "en").unwrap(),
            "2M"
        );
        assert_eq!(
            format(&n(999.0), Some("compact"), None, "en").unwrap(),
            "999"
        );
    }

    #[test]
    fn dates_follow_a_style_or_a_pattern() {
        let d = t("2024-03-05T14:07:09Z");
        assert_eq!(format(&d, Some("date"), None, "en").unwrap(), "Mar 5, 2024");
        assert_eq!(
            format(&d, Some("date"), Some(&t("long")), "en").unwrap(),
            "March 5, 2024"
        );
        assert_eq!(
            format(&d, Some("date"), Some(&t("full")), "en").unwrap(),
            "Tuesday, March 5, 2024"
        );
        assert_eq!(format(&d, Some("time"), None, "en").unwrap(), "2:07 PM");
        assert_eq!(
            format(&d, Some("datetime"), None, "en").unwrap(),
            "Mar 5, 2024, 2:07 PM"
        );
        assert_eq!(
            format(&d, Some("yyyy-MM-dd HH:mm:ss"), None, "en").unwrap(),
            "2024-03-05 14:07:09"
        );
        assert_eq!(
            format(&d, Some("EEE d MMM yy, h a"), None, "en").unwrap(),
            "Tue 5 Mar 24, 2 PM"
        );
        assert_eq!(
            format(&t("2024-03-05"), Some("EEEE"), None, "en").unwrap(),
            "Tuesday"
        );
        assert_eq!(
            format(&n(0.0), Some("yyyy-MM-dd"), None, "en").unwrap(),
            "1970-01-01"
        );
        assert_eq!(format(&t("not a date"), Some("date"), None, "en"), None);
    }

    #[test]
    fn relative_time_is_spelled_as_intl_spells_it() {
        assert_eq!(relative(0), "now");
        assert_eq!(relative(-30), "now");
        assert_eq!(relative(-50), "50 seconds ago");
        assert_eq!(relative(-90), "1 minute ago");
        assert_eq!(relative(-3 * 60), "3 minutes ago");
        assert_eq!(relative(3 * 3600), "in 3 hours");
        assert_eq!(relative(-86_400), "yesterday");
        assert_eq!(relative(2 * 86_400), "in 2 days");
        assert_eq!(relative(-8 * 86_400), "last week");
        assert_eq!(relative(-40 * 86_400), "last month");
        assert_eq!(relative(400 * 86_400), "next year");
        assert_eq!(
            ago(&t("2024-03-05T14:00:00Z"), Some(&t("2024-03-05T14:03:00Z"))).unwrap(),
            "3 minutes ago"
        );
    }
}
