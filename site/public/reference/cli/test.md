# wf test

> Run the project's `test "…" { … }` declarations under tests/

The page: https://webfluent.monzeromer.dev/reference/cli/test

Run the project's `test "…" { … }` declarations under tests/

```bash
wf test [PATH] [options]
```

## Options

| Option | Meaning |
|---|---|
| `[PATH]` | Project directory (default: current directory), or one test file |
| `--update` | Rewrite every snapshot with the current render |

**Writes:** the snapshots, with `--update`
