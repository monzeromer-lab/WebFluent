# wf generate

> Generate a new page, component, or store — or a block, a whole page to own: sign-in, pricing, dashboard, settings

The page: https://webfluent.monzeromer.dev/reference/cli/generate

Generate a new page, component, or store — or a block, a whole page to own: sign-in, pricing, dashboard, settings

```bash
wf generate KIND NAME [options]
```

## Options

| Option | Meaning |
|---|---|
| `KIND` | What to generate: page, component, store, or block |
| `NAME` | Name of the item to generate; for a block, which: sign-in, pricing, dashboard or settings |
| `--dir, -d DIR` | Project directory (default: current directory) |

**Writes:** a new source file

Writes a starter page, component or store.

```bash
wf generate page Pricing
wf generate component PriceCard
wf generate store Cart
```

A page gets a path from its name (`/pricing`), a title, a description and an `h1`. The file is `.wfx` when the project is written in that layout.

`wf generate block NAME` writes a whole page instead — `sign-in` (a form that checks what is typed), `pricing` (three plans, monthly or yearly), `dashboard` (figures, a trend and a table) or `settings` (a profile, notifications and the account, in tabs) — into `src/pages/`. It is ordinary source, the project's own to change; it builds with nothing to report. `wf generate` never overwrites a file: when one exists, it says so and writes nothing.
