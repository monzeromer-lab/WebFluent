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

**Preview:** `--check` says what it would do without doing it.

Formats every `.wf` and `.wfx` file under a path, or one file, in place.

```bash
wf fmt
wf fmt --check
wf fmt src/pages/Home.wf --stdout
wf fmt --to wfx
```

The formatter applies these rules:

- four spaces for each level, and a line that closes a block one level out;
- trailing spaces removed, and tabs turned into spaces;
- runs of blank lines folded to one;
- a brace one space from what it opens (`Row{` → `Row {`, `}else` → `} else`).

A line carried on by a trailing comma stays as written, and comments stay where they are. The result is checked against the file's own tokens, so formatting never changes what a file says.

`--to wfx` rewrites the project's `.wf` files in the indented layout, and `--to wf` goes the other way. It changes the layout and nothing else, and refuses a file whose indentation does not already follow its braces.

See also: [Two layouts: `.wf` and `.wfx`](https://webfluent.monzeromer.dev/learn/04-language-basics.md#two-layouts-wf-and-wfx).
