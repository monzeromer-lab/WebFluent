# wf translations

> The translations as a table: each locale's plural forms and what it lacks, each key's messages, and where a `t()` names it

The page: https://webfluent.monzeromer.dev/reference/cli/translations

The translations as a table: each locale's plural forms and what it lacks, each key's messages, and where a `t()` names it

```bash
wf translations [options]
```

## Options

| Option | Meaning |
|---|---|
| `--dir, -d DIR` | Project directory (default: current directory) |
| `--missing MISSING` | Only the keys this locale has no message for |
| `--unused` | Only the keys nothing names |
| `--plurals` | CLDR's plural rules instead: every locale's forms, and the form each of a run of counts takes |
| `--json` | Print JSON, for tools |

Shows the project's translations as a table.

```bash
wf translations
wf translations --missing ar
wf translations --unused
wf translations --json
wf translations --plurals
```

For each locale it prints its file, whether it is the default and whether it is right to left, the plural forms CLDR's rules give it, how many keys have no message in it, how many lack a form it needs, and how many messages have placeholders the default locale's do not.

A key is unused when nothing names it: no `t()`, no key built at run time that may be it, and no `validate` rule that shows it.

`--json` prints the whole table: every key, plural keys with each form, each message in each locale with its placeholders and line, every place a `t()` names the key, and the calls that name a key no locale has (`I01`).

`--plurals` prints each locale's forms; with `--json`, also the form each of a run of counts takes.

The library has it as `webfluent::i18n::table` (with the `internals` feature, outside semver), whose `set_message` writes one message back and keeps every other byte of the file.

See also: [Translate a site](https://webfluent.monzeromer.dev/recipes/translate-a-site.md).
