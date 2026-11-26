<!-- The core of WebFluent {{version}}, written by `wf init`. The whole reference, a file a section: .agents/skills/writing-webfluent/references/language.md. A newer wf brings a newer one. -->

# WebFluent

This project is written in WebFluent, a typed language that compiles `.wf`
files to a static site, a single-page app, a PDF or a slide deck. This file
is the short version: the grammar, ten idioms, the five most common
mistakes and the commands to run. The full reference is in the skill under
`.agents/skills/writing-webfluent/` (also in `.claude/skills/`). Pages go in
`src/pages/`, components in `src/components/`, stores in `src/stores/`,
tests in `tests/`, and static files in `public/`.

## Dev loop

```bash
wf serve              # Dev server on localhost:3000, live reload, error overlay
wf check              # Report all diagnostics; write nothing
wf explain T06        # Explain a diagnostic code: meaning, example, fix
wf fix --dry-run      # Show the fixes the compiler can apply
wf fmt                # Format sources
wf test               # Run the tests under tests/
wf build              # Compile to build/
wf inspect /route     # Print a page as text: headings, links, controls, errors
wf verify             # Load every page in headless Chrome; fail on errors
wf components Button  # A component's props, events, slots and parts
```

After each change: `wf check`, then `wf test`. Before saying a page works,
`wf build` and `wf verify`. Fix a finding rather than deleting the code it
is about.

{{grammar}}

{{idioms}}

{{mistakes}}

## Where to read more

Read only the file the task needs; each stays in your context.

| Task | File under `.agents/skills/writing-webfluent/` |
|---|---|
| Pages, components, state, stores, data, styling: the core | [references/core.md](.agents/skills/writing-webfluent/references/core.md) |
| How good WebFluent is written; review checklist | [references/code-quality.md](.agents/skills/writing-webfluent/references/code-quality.md) |
| Lists, forms, services, routes, dialogs, settings, shortcuts | [references/recipes.md](.agents/skills/writing-webfluent/references/recipes.md) |
| Themes, dark mode, `style { }`, CSS files, responsive layout, RTL | [references/styling.md](.agents/skills/writing-webfluent/references/styling.md) |
| Tests, fakes, `wf verify`, CI | [references/testing.md](.agents/skills/writing-webfluent/references/testing.md) |
| A diagnostic you don't understand | [references/diagnostics.md](.agents/skills/writing-webfluent/references/diagnostics.md) |
| PDFs, slide decks, server rendering | [references/documents.md](.agents/skills/writing-webfluent/references/documents.md) |
| Desktop and mobile apps (Tauri, Electron) | [references/tauri-apps.md](.agents/skills/writing-webfluent/references/tauri-apps.md) |
| Any language section: the index | [references/language.md](.agents/skills/writing-webfluent/references/language.md) |

The compiler answers too: `wf components Name`, `wf explain CODE`,
`wf scope FILE:LINE`, `wf config schema`.
