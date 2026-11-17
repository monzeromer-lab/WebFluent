# wf check

> Check the project — every finding a build would report — and write nothing

The page: https://webfluent.monzeromer.dev/reference/cli/check

Check the project — every finding a build would report — and write nothing

```bash
wf check [options]
```

## Options

| Option | Meaning |
|---|---|
| `--dir, -d DIR` | Project directory (default: current directory) |
| `--format FORMAT` | How findings are written: `human` (default), `json`, `sarif` (code scanning) or `github` (annotations on a pull request) |
| `--deny-warnings` | Fail on a warning too |
| `--allow-output-outside` | Accept a `build.output` outside the project, as `wf build --allow-output-outside` does; without it the check reports what the build would refuse |

Runs every check a build runs — the parser, the structure, the types, the
lints and the config — and writes nothing.

```bash
wf check
wf check --format sarif > wf.sarif
wf check --deny-warnings
```

`wf check` is fast enough for a pre-commit hook. It exits `1` on a finding
that would stop a build and `0` otherwise, and prints `No problems in NAME.`
when there are none. A check that needs the written output — the security
policy read back over the pages — runs only in `wf build`.

See also: [Diagnostics](../errors/diagnostics.md).
