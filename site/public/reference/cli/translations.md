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
