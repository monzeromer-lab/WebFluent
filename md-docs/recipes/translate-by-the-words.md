---
kind: "recipe"
route: "/recipes/translate-by-the-words"
group: "reach"
description: "Translate a project written in one language without inventing keys: with literals on, each string is looked up by its own words."
from: ["/docs/guide/i18n#translate-by-the-words-as-written"]
outputs: ["website", "app", "desktop"]
---
# Translate by the words as written

Translate a project that was written in one language from the start,
without inventing keys or wrapping every string in `t()`: each string a
reader sees is looked up in the reader's table by its own words. Turn it on
with `"literals": true` in the `i18n` config:

**Needs:** the `i18n` setting below, and the translation files [How it
works](#how-it-works) shows. It claims `/`: paste it into a new project,
or replace the starter's `src/pages/Home.wf` with it.

```json
{ "i18n": { "default_locale": "en", "locales": ["en", "ar"], "literals": true } }
```

This page keeps its English words, and a reader in Arabic sees them
translated:

```wf
page Problems(path: "/", title: "Problems", description: "What the checks found.") {
    state report = { count: 3, files: 11 }
    Heading("Problems").h1              // المشاكل, for a reader in ar
    Text("{report.count} missing")      // one key, `{count} missing`
    Text("{report.files} files")        // 11 ملفًا, for a reader in ar
}
```

## How it works

Every string a reader sees is looked up by its text as written: an
element's text, and props such as `label:`, `title:`, `placeholder:`,
`aria-label:` and `alt:`. The Arabic table, `src/translations/ar.json`,
maps the English words to their translation:

```json
{
  "Problems": "المشاكل",
  "{count} missing": "{count} مفقودة",
  "{files} files.zero": "لا توجد ملفات",
  "{files} files.one": "ملف واحد",
  "{files} files.two": "ملفان",
  "{files} files.few": "{files} ملفات",
  "{files} files.many": "{files} ملفًا",
  "{files} files.other": "{files} ملف"
}
```

How the text becomes a key:

- **A splice of a plain path is a placeholder.** It is named for the
  path's last part, so `"{report.count} missing"` is the key
  `{count} missing`. A splice of anything more, such as `"{n + 1} more"`,
  keeps the text as written.
- **Each branch of an `if` is its own key** when the text chooses between
  them.
- **A value shown by its name** — a `derived` or `state` — is looked up
  only when its name says it is words, such as `countText` or
  `emptyLabel`. Shown words that neither rule covers draw an `I05` warning.
- **What is not words is left alone:** an icon's name, `Code(…)`, and a
  component named `Kbd`.
- **A list of words is translated word by word.** A `labels:` prop, which
  holds one word for each option of a control whose options are values,
  is looked up item by item: `Segmented(options: ["2", "4"], labels: ["2
  spaces", "4 spaces"])` on your own `Segmented` component.

The default locale's messages are the words themselves, so its file may be
empty. `wf translations` counts each string as a key in use, and lists what
another locale has not translated as missing.

With an empty `en.json`, `wf check` still warns `` I03: `en.json` has no
`{files} files.other` ``. The warning is wrong: an English reader sees
`11 files`. To silence it, and to get "1 file" right as well, give
`en.json` its two forms:

```json
{ "{files} files.one": "{files} file", "{files} files.other": "{files} files" }
```

### Count by the number in the text

A literal with one number spliced into it picks its plural form by that
number, as `"{report.files} files"` does. The key's plural forms are keys
too, as for `count` with `t()`: `{files} files.one`, `{files} files.other`
and the rest, as in the table above.

The default locale gives its forms the same way — `"{files} files.one":
"{files} file"` — where its words differ by count. With two numbers in the
text, neither is taken as the count: the message is looked up as written.
