# wf migrate

> Carry a project forward: WebFluent 2 → 3, 3 → 4, 4 → 4.2 and 6 → 7, each step that applies

The page: https://webfluent.monzeromer.dev/reference/cli/migrate

Carry a project forward: WebFluent 2 → 3, 3 → 4, 4 → 4.2 and 6 → 7, each step that applies

```bash
wf migrate [PATH] [options]
```

## Options

| Option | Meaning |
|---|---|
| `[PATH]` | Project directory (default: current directory), or one .wf file |
| `--check` | Report what would change without writing anything |
| `--stdout` | Print the migrated text of one file to stdout instead of writing it |
| `--wfx` | Write the migrated files in the indented layout, as `.wfx` |

**Writes:** the project's source files

**Preview:** `--check` says what it would do without doing it.

The [full entry for wf migrate](https://webfluent.monzeromer.dev/reference/cli.md#wf-migrate) has its examples and the details this page leaves out.
