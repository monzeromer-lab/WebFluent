# wf docs

> Document the project's components as a site, each `example` drawn live

The page: https://webfluent.monzeromer.dev/reference/cli/docs

Document the project's components as a site, each `example` drawn live

```bash
wf docs [options]
```

## Options

| Option | Meaning |
|---|---|
| `--dir, -d DIR` | Project directory (default: current directory) |
| `--out, -o OUT` | Where to write the site (default: docs/) |

**Writes:** a gallery, in the folder `--out` names

Documents the project's components as a site, in the design of this
reference.

```bash
wf docs
wf docs -o component-docs
```

A page a component: its `///` comment and header, every
[`example`](../learn/10-components.md#document-it) of it drawn live beside
its source — in a frame of the project's own build, with its theme,
stylesheets and scripts — then its props (type, what it is when left out,
each prop's `///`), events, slots, parts and what places it. A component
with no example is drawn from its props. The types, stores and pages each
have a page.

The project is checked first, and an error stops it, as a build does.
Serve the folder to read it. A second run clears what the first wrote; a
folder it did not write is left alone.
