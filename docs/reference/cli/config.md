# wf config

> Read and change `webfluent.app.json`: every setting, what the project sets, and an edit that keeps the file's shape

The page: https://webfluent.monzeromer.dev/reference/cli/config

Read and change `webfluent.app.json`: every setting, what the project sets, and an edit that keeps the file's shape

```bash
wf config <schema|get|set|unset>
```

## wf config schema

Every setting: its type, its default and what it does

```bash
wf config schema [options]
```

| Option | Meaning |
|---|---|
| `--json` | As JSON, for a tool |

## wf config get

What the project sets at PATH, or the default

```bash
wf config get PATH [options]
```

| Option | Meaning |
|---|---|
| `PATH` | A dotted path: `build.base_path`, `build.budget.app.js` |
| `--dir, -d DIR` | The project's folder (default `.`) |

## wf config set

Set PATH to VALUE — JSON, or a word as a string — keeping the file's shape

```bash
wf config set PATH VALUE [options]
```

| Option | Meaning |
|---|---|
| `PATH` | A dotted path: `build.base_path`, `meta.site_url` |
| `VALUE` | The value: JSON (`true`, `3`, `["en", "ar"]`), or a word, which is taken as a string |
| `--dir, -d DIR` | The project's folder (default `.`) |

## wf config unset

Remove PATH, so its default applies

```bash
wf config unset PATH [options]
```

| Option | Meaning |
|---|---|
| `PATH` | A dotted path: `build.base_path`, `meta.site_url` |
| `--dir, -d DIR` | The project's folder (default `.`) |

The [full entry for wf config](https://webfluent.monzeromer.dev/reference/cli.md#wf-config) has its examples and the details this page leaves out.
