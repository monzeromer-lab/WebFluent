# wf fmt

> Format a project's source files; with `--to`, rewrite them in the other layout: `.wfx` (blocks by indentation) or `.wf` (blocks in braces)

The page: https://webfluent.monzeromer.dev/reference/cli/fmt

Format a project's source files; with `--to`, rewrite them in the other layout: `.wfx` (blocks by indentation) or `.wf` (blocks in braces)

```bash
wf fmt [PATH] [options]
```

## Options

| Option | Meaning |
|---|---|
| `[PATH]` | Project directory (default: current directory), or one file |
| `--to TO` | Change the layout instead: `wfx` or `wf` |
| `--check` | Write nothing; fail when a file is not formatted |
| `--stdout` | Print the formatted text to stdout instead of writing it |

**Writes:** the project's source files
