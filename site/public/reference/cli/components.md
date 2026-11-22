# wf components

> Every component a call can name — the built-ins and the project's — with what each takes: its props and their kinds, events, slots, parts and children

The page: https://webfluent.monzeromer.dev/reference/cli/components

Every component a call can name — the built-ins and the project's — with what each takes: its props and their kinds, events, slots, parts and children

```bash
wf components [options]
```

## Options

| Option | Meaning |
|---|---|
| `--dir, -d DIR` | Project directory (default: current directory) |
| `--json` | Print JSON, for tools |

Lists every component a call can name — the built-ins first, then the project's own — in one vocabulary.

```bash
wf components
wf components --json
```

For each prop it gives the kind of value it takes (`text`, `number`, `enum` and its cases, `file` and what kind, `state`, `path`, a declared `record`, a `list`, `date`, `color`, `money` …), whether it is the positional one, its default as written, a range where its type declares one (`Number(0..=100)`), and whether `.name` alone sets it. Then it gives the component's events, its slots and what each hands its fill, its parts and their owner, the attribute families its root takes, and whether it takes children.

This is the table the checker reads a call against. An inspector draws a control per prop from it, and an Insert panel knows what may go where. The library has it as `webfluent::registry::schema::of_project_via` (with the `internals` feature, outside semver).
