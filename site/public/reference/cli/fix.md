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

Applies every fix the compiler is certain of — the fix the editor offers first — and writes the files.

```bash
wf fix
wf fix --code V07
wf fix --dry-run
```

A fix that overlaps another, or that the first fixes make way for, is taken on a second pass, and passes repeat until nothing more changes. `wf fix` then lists what is left: findings with no certain fix, which a person decides. `wf check` lists them too.

See also: [Diagnostics](https://webfluent.monzeromer.dev/errors.md).
