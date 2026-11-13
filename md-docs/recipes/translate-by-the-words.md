---
kind: "recipe"
route: "/recipes/translate-by-the-words"
group: "reach"
description: "To translate a project that was written in one language from the start, without inventing keys, set `\"literals\": true` in its `i18n` config."
from: ["/docs/guide/i18n#translate-by-the-words-as-written"]
---
# Translate by the words as written

To translate a project that was written in one language from the start,
without inventing keys, set `"literals": true` in its `i18n` config. Every
string a reader sees is then looked up in the reader's table by its text as
written — an element's text, and props such as `label:`, `title:`,
`placeholder:`, `aria-label:` and `alt:`. The Arabic table maps the English
words to their translation:

```json
{ "Problems": "المشاكل", "{count} missing": "{count} مفقودة" }
```

The page keeps its English words:

```wf
Heading("Problems").h2          // المشاكل, for a reader in ar
Text("{l.count} missing")       // one key, `{count} missing`
```

How the text becomes a key:

- **A splice of a plain path is a placeholder.** It is named for the
  path's last part, so `"{l.count} missing"` is the key `{count} missing`.
  A splice of anything more, such as `"{n + 1} more"`, keeps the text as
  written.
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

### Count by the number in the text

A literal with one number spliced into it picks its plural form by that
number. The key's plural forms are keys too, as for `count`:

```json
{
  "{files} files.zero": "لا توجد ملفات",
  "{files} files.one": "ملف واحد",
  "{files} files.two": "ملفان",
  "{files} files.few": "{files} ملفات",
  "{files} files.many": "{files} ملفًا",
  "{files} files.other": "{files} ملف"
}
```

```wf
Text("{files} files")           // 11 ملفًا, for a reader in ar
```

The default locale gives its forms the same way — `"{files} files.one":
"{files} file"` — where its words differ by count. With two numbers in the
text, neither is taken as the count: the message is looked up as written.
