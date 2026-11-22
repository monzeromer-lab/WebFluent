# wf commands

> Every `wf` command, its options and what it writes, as data

The page: https://webfluent.monzeromer.dev/reference/cli/commands

Every `wf` command, its options and what it writes, as data

```bash
wf commands [options]
```

## Options

| Option | Meaning |
|---|---|
| `--json` | Print JSON, for tools |

Describes every `wf` command as data.

```bash
wf commands
wf commands --json
```

For each command it gives what the command does, each option — its flag, what it takes (`nothing` for a flag, a `word` from the values it lists, a `number`, a `path`, `text`) and its default — what the command writes, and the flag that shows it first and writes nothing (`fmt --check`, `migrate --check`, `fix --dry-run`). Build a tool page, a command palette or a script from this output instead of from a copy of this chapter. The library has the same as `webfluent::commands` (with the `internals` feature, outside semver).
