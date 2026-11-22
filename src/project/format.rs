//! `format(value, style, option)` and `ago(date)` at build time.
//!
//! The browser formats with `Intl`; the static paint and a template have no
//! `Intl`, so this is the same surface over a small table: digit grouping
//! and the decimal mark of the common locales, the symbols of the common
//! currencies, English and Arabic month and weekday names, and the digits
//! a locale writes by default (`ar-EG` and `ar-u-nu-arab` in Arabic-Indic,
//! `ar` and the Maghreb in Latin, as `Intl` writes them). What the table
//! lacks falls back to the English spelling, and the page corrects itself
//! once it is live.

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

const MONTHS: [&str; 12] = [
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
];
const WEEKDAYS: [&str; 7] = [
    "Sunday",
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
];

const MONTHS_AR: [&str; 12] = [
    "يناير",
    "فبراير",
    "مارس",
    "أبريل",
    "مايو",
    "يونيو",
    "يوليو",
    "أغسطس",
    "سبتمبر",
    "أكتوبر",
    "نوفمبر",
    "ديسمبر",
];
/// The Levant's and Iraq's month names, which `Intl` gives `ar-SY`,
/// `ar-LB`, `ar-JO`, `ar-PS` and `ar-IQ`.
const MONTHS_AR_LEVANT: [&str; 12] = [
    "كانون الثاني",
    "شباط",
    "آذار",
    "نيسان",
    "أيار",
    "حزيران",
    "تموز",
    "آب",
    "أيلول",
    "تشرين الأول",
    "تشرين الثاني",
    "كانون الأول",
];
const WEEKDAYS_AR: [&str; 7] = [
    "الأحد",
    "الاثنين",
    "الثلاثاء",
    "الأربعاء",
    "الخميس",
    "الجمعة",
    "السبت",
];

/// The language of a locale tag, lower-cased: `ar-EG` is `ar`.
fn language(locale: &str) -> String {
    locale
        .split(['-', '_'])
        .next()
        .unwrap_or("en")
        .to_ascii_lowercase()
}

/// The region of a locale tag, upper-cased, when it names one: `ar-EG` is
/// `EG`; `ar-u-nu-arab` names none.
fn region(locale: &str) -> Option<String> {
    let mut parts = locale.split(['-', '_']).skip(1);
    let next = parts.next()?;
    // A script subtag (`zh-Hant-TW`) comes before the region.
    let candidate = if next.len() == 4 { parts.next()? } else { next };
    ((candidate.len() == 2 && candidate.chars().all(|c| c.is_ascii_alphabetic()))
        || (candidate.len() == 3 && candidate.chars().all(|c| c.is_ascii_digit())))
    .then(|| candidate.to_ascii_uppercase())
}

/// Whether `locale` writes its digits as Arabic-Indic ones (`١٢٣`), as
/// `Intl` does: an `-u-nu-` extension says so outright; otherwise Arabic
/// with a region outside the Maghreb and the Emirates does, and Arabic
/// without one does not.
pub fn arabic_digits(locale: &str) -> bool {
    let lower = locale.to_ascii_lowercase();
    if lower.contains("-u-nu-arab") {
        return true;
    }
    if lower.contains("-u-nu-") || language(locale) != "ar" {
        return false;
    }
    region(locale)
        .is_some_and(|r| !matches!(r.as_str(), "001" | "AE" | "EH" | "MA" | "LY" | "TN" | "DZ"))
}

/// `text` with its ASCII digits written as the locale writes them.
fn localize_digits(text: String, locale: &str) -> String {
    if !arabic_digits(locale) {
        return text;
    }
    text.chars()
        .map(|c| match c.to_digit(10) {
            Some(d) => char::from_u32(0x0660 + d).unwrap_or(c),
            None => c,
        })
        .collect()
}

/// The digit-group and decimal marks of a locale.
fn separators(locale: &str) -> (&'static str, &'static str) {
    let lang = language(locale);
    if arabic_digits(locale) {
        return ("\u{66c}", "\u{66b}");
    }
    if lang == "ar"
        && region(locale).is_some_and(|r| matches!(r.as_str(), "MA" | "LY" | "TN" | "DZ"))
    {
        return (".", ",");
    }
    match lang.as_str() {
        "de" | "es" | "it" | "pt" | "nl" | "tr" | "id" | "da" | "el" | "ro" | "hr" | "sl" => {
            (".", ",")
        }
        "fr" | "ru" | "pl" | "cs" | "sv" | "nb" | "no" | "fi" | "uk" | "hu" | "sk" | "bg" => {
            ("\u{202f}", ",")
        }
        "ch" => ("’", "."),
        _ => (",", "."),
    }
}

/// The symbol a currency is written with, or its code.
fn currency_symbol(code: &str) -> String {
    match code {
        "USD" => "$".to_string(),
        "EUR" => "€".to_string(),
        "GBP" => "£".to_string(),
        "JPY" => "¥".to_string(),
        "CNY" => "CN¥".to_string(),
        "INR" => "₹".to_string(),
        "KRW" => "₩".to_string(),
        "CAD" => "CA$".to_string(),
        "AUD" => "A$".to_string(),
        "BRL" => "R$".to_string(),
        "ILS" => "₪".to_string(),
        "NGN" => "₦".to_string(),
        "PHP" => "₱".to_string(),
        "VND" => "₫".to_string(),
        other => format!("{other}\u{a0}"),
    }
}

/// `1234.5` with `places` decimals, grouped for the locale.
pub fn number(value: f64, places: Option<usize>, locale: &str) -> String {
    if !value.is_finite() {
        return if value.is_nan() {
            "NaN".to_string()
        } else if value > 0.0 {
            "∞".to_string()
        } else {
            "-∞".to_string()
        };
    }
    // `Intl.NumberFormat` takes at most 100 fraction digits; a number of
    // places from the data is held to that, never handed to `format!`,
    // whose precision panics past 65,535.
    let places = places.map(|p| p.min(100));
    let (group, point) = separators(locale);
    let negative = value < 0.0;
    // Halves round away from zero, as `Intl.NumberFormat` rounds them.
    let rounded = |p: usize| {
        let scale = 10f64.powi(p as i32);
        format!("{:.*}", p, (value.abs() * scale).round() / scale)
    };
    let text = if let Some(p) = places {
        rounded(p)
    } else {
        // Up to three decimals, as `Intl.NumberFormat` shows by default.
        let t = rounded(3);
        let t = t.trim_end_matches('0').trim_end_matches('.');
        t.to_string()
    };
    let (whole, frac) = match text.split_once('.') {
        Some((w, f)) => (w.to_string(), Some(f.to_string())),
        None => (text, None),
    };
    let mut grouped = String::new();
    for (i, c) in whole.chars().enumerate() {
        if i > 0 && (whole.len() - i) % 3 == 0 {
            grouped.push_str(group);
        }
        grouped.push(c);
    }
    let mut out = String::new();
    if negative && (grouped != "0" || frac.as_deref().is_some_and(|f| f.chars().any(|c| c != '0')))
    {
        out.push('-');
    }
    out.push_str(&grouped);
    if let Some(f) = frac {
        out.push_str(point);
        out.push_str(&f);
    }
    localize_digits(out, locale)
}

/// `1.2K`, `3.4M`: the compact notation.
fn compact(value: f64, locale: &str) -> String {
    let abs = value.abs();
    let (scaled, suffix) = if abs >= 1e12 {
        (value / 1e12, "T")
    } else if abs >= 1e9 {
        (value / 1e9, "B")
    } else if abs >= 1e6 {
        (value / 1e6, "M")
    } else if abs >= 1e3 {
        (value / 1e3, "K")
    } else {
        return number(value, None, locale);
    };
    // One decimal below ten, none above, as Intl rounds it.
    let places = if scaled.abs() < 10.0 {
        Some(1)
    } else {
        Some(0)
    };
    let text = number(scaled, places, locale);
    let (_, point) = separators(locale);
    let text = match text.split_once(point) {
        Some((w, f)) if f.chars().all(|c| matches!(c, '0' | '\u{660}')) => w.to_string(),
        _ => text,
    };
    format!("{text}{suffix}")
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

    /// The moment written to a pattern, its month and weekday names, its
    /// AM and PM and its digits the locale's (English where the table has
    /// none of its own).
    pub fn pattern_in(&self, pattern: &str, locale: &str) -> String {
        let arabic = language(locale) == "ar";
        let months: &[&str; 12] = if !arabic {
            &MONTHS
        } else if region(locale)
            .is_some_and(|r| matches!(r.as_str(), "SY" | "LB" | "JO" | "PS" | "IQ"))
        {
            &MONTHS_AR_LEVANT
        } else {
            &MONTHS_AR
        };
        let weekdays: &[&str; 7] = if arabic { &WEEKDAYS_AR } else { &WEEKDAYS };
        let month = months[self.month as usize - 1];
        let weekday = weekdays[self.weekday()];
        // An Arabic name is not cut short: `Intl`'s short month is the
        // long one.
        let short = |name: &str| {
            if arabic {
                name.to_string()
            } else {
                name.chars().take(3).collect::<String>()
            }
        };
        let mut out = String::new();
        let chars: Vec<char> = pattern.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            let c = chars[i];
            let mut run = 1;
            while i + run < chars.len() && chars[i + run] == c {
                run += 1;
            }
            let h12 = match self.hour % 12 {
                0 => 12,
                h => h,
            };
            let piece = match (c, run) {
                ('y', 2) => format!("{:02}", self.year.rem_euclid(100)),
                ('y', _) => format!("{}", self.year),
                ('M', 4) => month.to_string(),
                ('M', 3) => short(month),
                ('M', 2) => format!("{:02}", self.month),
                ('M', _) => format!("{}", self.month),
                ('d', 2) => format!("{:02}", self.day),
                ('d', _) => format!("{}", self.day),
                ('E', 4) => weekday.to_string(),
                ('E', _) => short(weekday),
                ('H', 2) => format!("{:02}", self.hour),
                ('H', _) => format!("{}", self.hour),
                ('h', 2) => format!("{h12:02}"),
                ('h', _) => format!("{h12}"),
                ('m', _) => format!("{:02}", self.minute),
                ('s', _) => format!("{:02}", self.second),
                ('a', _) => match (arabic, self.hour < 12) {
                    (true, true) => "ص",
                    (true, false) => "م",
                    (false, true) => "AM",
                    (false, false) => "PM",
                }
                .to_string(),
                _ => std::iter::repeat_n(c, run).collect(),
            };
            out.push_str(&piece);
            i += run;
        }
        localize_digits(out, locale)
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
    match style {
        // Text that is not a number is shown as it is: `format(label)` of
        // what a page already wrote out, `Stat(value: "$1,234.50")`.
        None if matches!(value, Input::Text(t) if as_number(value).is_none() && !t.trim().is_empty()) => {
            Some(match value {
                Input::Text(t) => t.clone(),
                Input::Number(n) => n.to_string(),
            })
        }
        None | Some("number") => Some(number(as_number(value)?, None, locale)),
        Some("integer") => Some(number(as_number(value)?.round(), Some(0), locale)),
        Some("decimal") => Some(number(
            as_number(value)?,
            Some(option_places(option, 2)),
            locale,
        )),
        Some("currency") => {
            let n = as_number(value)?;
            let code = option_text(option).unwrap_or_else(|| "USD".to_string());
            let places = if code == "JPY" || code == "KRW" { 0 } else { 2 };
            let digits = number(n.abs(), Some(places), locale);
            let symbol = currency_symbol(&code);
            Some(if n < 0.0 {
                format!("-{symbol}{digits}")
            } else {
                format!("{symbol}{digits}")
            })
        }
        Some("percent") => Some(format!(
            "{}%",
            number(
                as_number(value)? * 100.0,
                Some(option_places(option, 0)),
                locale
            )
        )),
        Some("compact") => Some(compact(as_number(value)?, locale)),
        Some("date") => {
            let m = parse_moment(value)?;
            let style = option_text(option);
            Some(m.pattern_in(date_pattern(locale, style.as_deref()), locale))
        }
        Some("time") => {
            let m = parse_moment(value)?;
            let style = option_text(option);
            Some(m.pattern_in(time_pattern(locale, style.as_deref()), locale))
        }
        Some("datetime") => {
            let m = parse_moment(value)?;
            let style = option_text(option);
            let date = m.pattern_in(date_pattern(locale, style.as_deref()), locale);
            // `Intl` gives a date and time no seconds unless asked.
            let time = m.pattern_in(time_pattern(locale, None), locale);
            let joiner = match (language(locale).as_str(), style.as_deref()) {
                ("ar", Some("long" | "full")) => " في ",
                ("ar", _) => "، ",
                _ => ", ",
            };
            Some(format!("{date}{joiner}{time}"))
        }
        Some("relative") => ago(value, None),
        // Anything else is a date pattern.
        Some(pattern) => Some(parse_moment(value)?.pattern_in(pattern, locale)),
    }
}

/// The pattern `Intl.DateTimeFormat(locale, { dateStyle })` writes a date
/// to, for the locales the table knows; English for the rest.
fn date_pattern(locale: &str, style: Option<&str>) -> &'static str {
    if language(locale) == "ar" {
        // The right-to-left marks keep the slashes between the numbers.
        return match style {
            Some("long") => "d MMMM yyyy",
            Some("full") => "EEEE، d MMMM yyyy",
            _ => "d\u{200f}/M\u{200f}/yyyy",
        };
    }
    match style {
        Some("short") => "M/d/yy",
        Some("long") => "MMMM d, yyyy",
        Some("full") => "EEEE, MMMM d, yyyy",
        _ => "MMM d, yyyy",
    }
}

/// The pattern `Intl.DateTimeFormat(locale, { timeStyle })` writes a time
/// to.
fn time_pattern(locale: &str, style: Option<&str>) -> &'static str {
    let maghreb = language(locale) == "ar"
        && region(locale).is_some_and(|r| matches!(r.as_str(), "MA" | "LY" | "TN" | "DZ"));
    match (maghreb, style) {
        (true, Some("medium" | "long" | "full")) => "HH:mm:ss",
        (true, _) => "HH:mm",
        (false, Some("medium" | "long" | "full")) => "h:mm:ss a",
        (false, _) => "h:mm a",
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
            "٢K"
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
            "0"
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
