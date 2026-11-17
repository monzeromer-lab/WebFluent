# wf types

> Describe what a project declares: enums, types, components, stores, pages

The page: https://webfluent.monzeromer.dev/reference/cli/types

Describe what a project declares: enums, types, components, stores, pages

```bash
wf types [PATH] [options]
```

## Options

| Option | Meaning |
|---|---|
| `[PATH]` | Project directory (default: current directory) |
| `--json` | Print JSON, for tools |
| `--holes` | List every place a value's type is unknown, and why |

Describes what a project declares: enums, types, components, stores and
pages.

```bash
wf types
wf types --json
wf types --holes
```

It includes the names the project's scripts under `src/` declare, each with
the parameters and return type its JSDoc gives it.

With `--holes` it lists every place the checker cannot say what a value is,
each with its file, line and kind, then the count of each kind. A hole is
counted where the unknown value is made, not at every place it goes
afterwards. Since 7.0 a program that checks has none: `Any` and `Map` are
not types (`T23`, `T24`), every `state`, parameter and return says what it
holds, and the browser, events and built-ins are typed. The list is for a
project on its way there — one `wf migrate` has not finished — and for a
tool that wants to prove it. `--json` gives `total`, the count of each
kind, and the list.

```text
  0 place(s) where a type is unknown, in 0 file(s)
```
