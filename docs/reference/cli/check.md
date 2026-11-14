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

The [full entry for wf check](https://webfluent.monzeromer.dev/reference/cli.md#wf-check) has its examples and the details this page leaves out.
