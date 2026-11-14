# wf render

> Render a .wf template with JSON data

The page: https://webfluent.monzeromer.dev/reference/cli/render

Render a .wf template with JSON data

```bash
wf render TEMPLATE [options]
```

## Options

| Option | Meaning |
|---|---|
| `TEMPLATE` | Template file (.wf or .wfx), or a directory: every template under it as one, components shared |
| `--data DATA` | JSON data file (reads stdin if omitted) |
| `--format, -f FORMAT` | Output format: "html" (default), "html-fragment", "pdf", or "slides" |
| `--output, -o OUTPUT` | Output file (stdout if omitted) |
| `--theme THEME` | Which `Theme` declared in the template to render with. Only needed when the template declares more than one |
| `--token NAME=VALUE` | A design token over the theme's, `NAME=VALUE`; may be given more than once |
| `--page PAGE` | The page to render, by name, when the template has several |
| `--lang LANG` | The document's language, `<html lang>` (default: the page's `lang:`, else `en`) |

**Writes:** the file `--output` names

The [full entry for wf render](https://webfluent.monzeromer.dev/reference/cli.md#wf-render) has its examples and the details this page leaves out.
