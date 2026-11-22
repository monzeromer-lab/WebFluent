## i18n (Internationalization)

### Translation files

```json
// src/translations/en.json
{ "nav.home": "Home", "greeting": "Hello, {name}!" }

// src/translations/ar.json
{ "nav.home": "الرئيسية", "greeting": "أهلاً، {name}!" }
```

### Config

```json
{
    "i18n": {
        "default_locale": "en",
        "locales": ["en", "ar"],
        "dir": "src/translations"
    }
}
```

### Starting translated

`wf init my-app --locales en,ar` (spa or static; the first is the
default) writes the `i18n` config above with `meta.lang` the default, every
visible string of the starter through `t("…")`, a language switch with one
entry per locale, and `src/translations/<locale>.json` for each: the
default's holds the starter's English (the text its pages and tests are
written in), another the starter's translation into it where it has one
(Arabic), otherwise the same English to translate. A tag is `en`, `ar`,
`pt-BR`, `zh-Hant`; one named twice, an empty list or a code that is not
a tag is refused. `pdf` and `slides` take none. The library:
`webfluent::init::create_with(dir, name, &Options { template, locales })`.

### Usage

```wf
Text(t("nav.home"))                    // Translated text
Text(t("greeting", { name: user.name }))          // With interpolation
Text(t("items", { count: n }))                    // A plural form, picked by `count`
Button("EN") { on click { setLocale("en") } }     // Switch locale
Button("AR") { on click { setLocale("ar") } }     // Auto-RTL for Arabic
```

A message with plural forms is several keys: `"items.one": "{count}
item"`, `"items.other": "{count} items"` — and `items.zero`, `items.two`,
`items.few`, `items.many` where the locale's CLDR rules call for them;
`t("items", { count: n })` picks the form, exactly none `items.zero` when
the file has it in any language, falling back to `items.other` and then
`items` — the static paint and the live page by the same rules. A key's
forms come from one language. Under `i18n.literals`, text with one number
in it (`Text("{files} files")`) picks its form by that number:
`"{files} files.one"`, `"{files} files.other"` … in each table. Under
literals a text is keyed by the names it splices (`{item?.name ?? ""}` is
`{name}`); a `derived`, `state` or action-written value is looked up only
when its name says it is words (`countText`, `emptyLabel`, `saidNote`);
I05 warns of shown words that are neither. `format` and `ago` speak the locale too.
`setLocale("ar-u-nu-arab")` (or `-u-nu-latn`) reads the `ar` table and
names the digits `format`, `ago` and counts are written in.
`wf translations` is the table: what each locale lacks, what nothing names.

RTL locales (automatic `dir="rtl"`, from the first paint when the default
locale is one): `ar`, `he`, `fa`, `ur`, `ps`, `sd`, `ckb`, `ug`, `yi`, `dv`,
`ks`, `syr`, `arc`, `nqo` — with any region (`ar-EG`)

A Latin code or figure inside Arabic text (`2026-0417`) can come out with
its parts reordered; give it its own left-to-right run:
`Text("2026-0417", class: "ltr")` with `.ltr { direction: ltr;
unicode-bidi: isolate; }` in the project's CSS.

