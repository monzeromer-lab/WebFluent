//! The methods each kind of value has, and the places a secret may not go.

use super::*;

/// Whether a CSS property takes a length, which a bare number is not.
pub(super) fn takes_length(name: &str) -> bool {
    matches!(
        name,
        "width"
            | "height"
            | "min-width"
            | "max-width"
            | "min-height"
            | "max-height"
            | "top"
            | "left"
            | "right"
            | "bottom"
            | "inset"
            | "gap"
            | "row-gap"
            | "column-gap"
            | "font-size"
            | "border-radius"
            | "radius"
            | "border-width"
            | "outline-width"
            | "outline-offset"
            | "letter-spacing"
            | "word-spacing"
            | "text-indent"
            | "flex-basis"
            | "inline-size"
            | "block-size"
    ) || name.starts_with("margin")
        || name.starts_with("padding")
}

/// The browser's own objects a secret must not be handed to.
pub(super) const SECRET_SINKS: &[&str] = &[
    "console",
    "localStorage",
    "sessionStorage",
    "document",
    "window",
    "navigator",
    "history",
];

/// Every method a list has: the browser's own, and the helpers the
/// runtime adds (`sortBy`, `groupBy`, `unique`, `take`, `first`, `last`,
/// `sum`, `remove`, `contains`).
pub(super) const LIST_METHODS: &[&str] = &[
    "map",
    "filter",
    "find",
    "findIndex",
    "findLast",
    "findLastIndex",
    "some",
    "every",
    "includes",
    "indexOf",
    "lastIndexOf",
    "join",
    "slice",
    "concat",
    "reverse",
    "sort",
    "push",
    "pop",
    "shift",
    "unshift",
    "splice",
    "reduce",
    "reduceRight",
    "forEach",
    "flat",
    "flatMap",
    "fill",
    "copyWithin",
    "entries",
    "keys",
    "values",
    "at",
    "with",
    "toReversed",
    "toSorted",
    "toSpliced",
    "toString",
    "toLocaleString",
    "sortBy",
    "groupBy",
    "unique",
    "take",
    "first",
    "last",
    "sum",
    "remove",
    "contains",
];

/// Every method a string has: the browser's own and the runtime's
/// (`capitalize`, `truncate`, `dedent`, `lines`, `words`).
pub(super) const STRING_METHODS: &[&str] = &[
    "toLowerCase",
    "toUpperCase",
    "toLocaleLowerCase",
    "toLocaleUpperCase",
    "trim",
    "trimStart",
    "trimEnd",
    "replace",
    "replaceAll",
    "slice",
    "substring",
    "substr",
    "charAt",
    "charCodeAt",
    "codePointAt",
    "at",
    "toString",
    "valueOf",
    "padStart",
    "padEnd",
    "repeat",
    "normalize",
    "concat",
    "indexOf",
    "lastIndexOf",
    "search",
    "localeCompare",
    "includes",
    "startsWith",
    "endsWith",
    "split",
    "match",
    "matchAll",
    "isWellFormed",
    "toWellFormed",
    "capitalize",
    "truncate",
    "dedent",
    "lines",
    "words",
    "contains",
    "toUpper",
    "toLower",
];

/// Every method a number has.
pub(super) const NUMBER_METHODS: &[&str] = &[
    "toFixed",
    "toString",
    "toPrecision",
    "toExponential",
    "toLocaleString",
    "valueOf",
];

/// The type a control's `bind:` state must hold: an `Input` by its type
/// flag — a number field a number, any other its text.
pub(super) fn bound_type(component: &str, modifiers: &[String]) -> Option<Type> {
    match component {
        "Checkbox" | "Switch" => Some(Type::Bool),
        "Slider" => Some(Type::Number),
        // A date picker picks a date, which is a `Date` — not a string
        // that happens to look like one. Empty, it holds nothing.
        "DatePicker" => Some(Type::optional(Type::Scalar(Scalar::Date))),
        "Input" if modifiers.iter().any(|m| m == "number") => Some(Type::Number),
        "Input" | "Textarea" => Some(Type::String),
        _ => None,
    }
}

/// What in `ty` the browser's storage cannot keep as JSON, said for a
/// finding.
pub(super) fn not_serialisable(ty: &Type) -> Option<&'static str> {
    match ty {
        Type::Func(..) => Some("a function"),
        Type::Scalar(Scalar::File) => Some("a `File`"),
        Type::Promise(_) => Some("a promise"),
        Type::Resource(_) => Some("a resource"),
        Type::Optional(inner) | Type::List(inner) => not_serialisable(inner),
        Type::Shape(fields) => fields.iter().find_map(|(_, t)| not_serialisable(t)),
        _ => None,
    }
}

/// `.failed(reason: String)`: a case with its payload, for a hint.
pub(super) fn case_signature(case: &str, fields: &[(String, Type)]) -> String {
    let parts: Vec<String> = fields.iter().map(|(n, t)| format!("{n}: {t}")).collect();
    format!(".{case}({})", parts.join(", "))
}

/// What a method of one of the language's own types gives back, and
/// whether it has that method at all.
pub(super) fn scalar_method(scalar: Scalar, method: &str) -> Option<Type> {
    use Scalar::*;
    let moment = matches!(scalar, Date | Time | DateTime);
    Some(match method {
        // Reading a moment apart.
        "year" | "month" | "day" | "weekday" | "hour" | "minute" | "second" if moment => {
            Type::Number
        }
        "native" if moment => Type::Any,
        "date" if scalar == DateTime => Type::Scalar(Date),
        "time" if scalar == DateTime => Type::Scalar(Time),
        "inZone" if moment => Type::Scalar(DateTime),
        "startOfDay" | "startOfWeek" | "startOfMonth" | "endOfDay" if moment => {
            Type::Scalar(scalar)
        }
        "isBefore" | "isAfter" | "isSame" if moment => Type::Bool,
        "until" if moment => Type::Scalar(Duration),
        // Arithmetic: a moment and money both take `plus` and `minus`, and
        // each gives back what it was.
        "plus" | "minus" if moment || matches!(scalar, Money | Duration) => Type::Scalar(scalar),
        // A length of time, read in a unit.
        "days" | "hours" | "minutes" | "seconds" | "ms" if scalar == Duration => Type::Number,
        // Money.
        "times" if scalar == Money => Type::Scalar(Money),
        "convert" if scalar == Money => Type::Scalar(Money),
        // A web address.
        "host" | "path" if scalar == Url => Type::String,
        "query" if scalar == Url => Type::Map,
        "with" if scalar == Url => Type::Scalar(Url),
        "domain" if scalar == Email => Type::String,
        // Colour.
        "mix" | "lighten" | "darken" if scalar == Color => Type::Scalar(Color),
        "alpha" if scalar == Color => Type::String,
        "contrast" if scalar == Color => Type::Number,
        // A file the reader chose.
        "preview" if scalar == File => Type::String,
        // The browser's own: what the file holds, read when awaited.
        "text" | "arrayBuffer" | "slice" | "stream" if scalar == File => Type::Any,
        // Everything a string can do, a string-carried scalar can do.
        _ if scalar.is_text() => return string_method(method),
        _ => return None,
    })
}

/// What a `String`'s method gives back, `None` when it has none.
pub(super) fn string_method(method: &str) -> Option<Type> {
    Some(match method {
        "toLowerCase" | "toUpperCase" | "trim" | "trimStart" | "trimEnd" | "replace"
        | "replaceAll" | "slice" | "substring" | "charAt" | "at" | "toString" | "padStart"
        | "padEnd" | "repeat" | "normalize" | "capitalize" | "truncate" | "dedent" => Type::String,
        "indexOf" | "lastIndexOf" | "length" | "charCodeAt" | "search" | "localeCompare" => {
            Type::Number
        }
        "includes" | "startsWith" | "endsWith" => Type::Bool,
        "split" | "lines" | "words" => Type::list(Type::String),
        "match" => Type::optional(Type::list(Type::String)),
        _ => return None,
    })
}

/// The methods a scalar has, for the message when one is misspelled.
pub(super) fn scalar_methods_hint(scalar: Scalar) -> String {
    use Scalar::*;
    let listed: &str = match scalar {
        Date | Time | DateTime => {
            "`.year()`, `.month()`, `.day()`, `.weekday()`, `.hour()`, `.minute()`,              `.plus(days: 1)`, `.minus(…)`, `.isBefore(d)`, `.isAfter(d)`, `.until(d)`,              `.startOfDay()`, `.startOfWeek()`, `.startOfMonth()`, `.endOfDay()`,              `.inZone(\"Europe/Berlin\")`, `.native()`"
        }
        Duration => "`.days()`, `.hours()`, `.minutes()`, `.seconds()`, `.ms()`, `.plus(d)`",
        Money => {
            "`.plus(m)`, `.minus(m)`, `.times(n)`, `.convert(rate, \"USD\")`, and the fields `.amount` and `.currency`"
        }
        Url => "`.host()`, `.path()`, `.query()`, `.with(query: { page: 2 })`",
        Email => "`.domain()`",
        Color => {
            "`.mix(other, 0.2)`, `.lighten(0.1)`, `.darken(0.1)`, `.alpha(0.5)`, `.contrast(other)`"
        }
        File => "`.preview()`, and the fields `.name`, `.size` and `.type`",
        Uuid | Secret => "the methods of a string",
    };
    format!("It has {listed}")
}
