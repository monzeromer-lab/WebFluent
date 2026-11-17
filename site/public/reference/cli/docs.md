# wf docs

> Write a component gallery: every built-in, and what the project declares

The page: https://webfluent.monzeromer.dev/reference/cli/docs

Write a component gallery: every built-in, and what the project declares

```bash
wf docs [options]
```

## Options

| Option | Meaning |
|---|---|
| `--dir, -d DIR` | Project directory (default: current directory) |
| `--out, -o OUT` | Where to write index.html (default: docs/) |

**Writes:** a gallery, in the folder `--out` names

Writes a self-contained component gallery.

```bash
wf docs
wf docs -o gallery
```

The gallery shows every built-in — props, cases, flags, events, slots,
parts — and every component, enum, type, store and page the project
declares, with their `///` documentation.
