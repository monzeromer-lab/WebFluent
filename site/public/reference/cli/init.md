# wf init

> Create a new WebFluent project

The page: https://webfluent.monzeromer.dev/reference/cli/init

Create a new WebFluent project

```bash
wf init NAME [options]
```

## Options

| Option | Meaning |
|---|---|
| `NAME` | Project name |
| `--template, -t TEMPLATE` | Template: "spa" (default), "static", "pdf", or "slides" |
| `--locales, -l LOCALES` | Start it translated: the locales, comma-separated, the default first ("en,ar"). Every visible string goes through t(), and src/translations/ holds a file for each. spa and static only |

**Writes:** a new project's folder

Creates a new project directory with a config, a `.gitignore`, an `AGENTS.md`, the `writing-webfluent` agent skill (in `.agents/skills/` and `.claude/skills/`) and a small working application.

```bash
wf init my-site -t static
wf init my-app --locales en,ar
```

With `--locales`, every visible string of the starter goes through `t("…")`, the config's `i18n` names the locales (and `meta.lang` the default), the language switch has one entry for each locale, and `src/translations/` holds a file per locale:

- the default locale's file holds the starter's English, the text its pages and tests are written in;
- another locale's file holds the starter's own translation where it has one (Arabic), and the same English otherwise, for you to translate.

A locale is a language tag — `en`, `ar`, `pt-BR`, `zh-Hant` — written in any case and stored in the usual one. A tag named twice, an empty list, or a code that is not a tag (`pt_BR`) is refused with what to write instead. A `pdf` document or a `slides` deck is made in one language and refuses `--locales`.

`AGENTS.md` is the language reference — the file a coding agent reads before it changes a project. With it, an assistant asked to add a page writes the WebFluent of the `wf` that made the project, flags and all, instead of guessing. Its first line names the version it describes. The skill points an agent at `AGENTS.md` when the work is WebFluent, in the folders agents look in for one. A newer `wf` writes a newer one, and an older project can take the current one from the [repository](https://github.com/monzeromer-lab/WebFluent/blob/master/AGENTS.md).

`wf init` writes a shorter config than the full set of defaults, and turns `build.csp` on.

See also: [Translate a site](https://webfluent.monzeromer.dev/recipes/translate-a-site.md), [Configuration](https://webfluent.monzeromer.dev/reference/config.md).
