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

The [full entry for wf init](https://webfluent.monzeromer.dev/reference/cli.md#wf-init) has its examples and the details this page leaves out.
