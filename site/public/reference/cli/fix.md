# wf fix

> Fix every finding that has a fix — or every one of a code — as `wf check` and the editor offer it, and say what is left

The page: https://webfluent.monzeromer.dev/reference/cli/fix

Fix every finding that has a fix — or every one of a code — as `wf check` and the editor offer it, and say what is left

```bash
wf fix [options]
```

## Options

| Option | Meaning |
|---|---|
| `--dir, -d DIR` | Project directory (default: current directory) |
| `--code CODE` | Only this code's findings: `V01`, `A01` |
| `--dry-run` | Write nothing; say what would change |

**Writes:** the project's source files

**Preview:** `--dry-run` says what it would do without doing it.

The [full entry for wf fix](https://webfluent.monzeromer.dev/reference/cli.md#wf-fix) has its examples and the details this page leaves out.
