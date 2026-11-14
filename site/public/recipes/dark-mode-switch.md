# Add a dark-mode switch

> Let the reader pick light, dark or the system's choice with setTheme, and mark the current choice by reading theme.

The page: https://webfluent.monzeromer.dev/recipes/dark-mode-switch

Let the reader pick a light look, a dark one, or whatever their system
prefers, and keep that choice across visits. The switch needs a dark theme
named in `webfluent.app.json`:

**Needs:** the `theme` setting below in `webfluent.app.json`. The starter
already declares `theme Night` in `src/theme.wf` and a `ThemeToggle`
component that does this; replace `src/theme.wf` with this code's themes,
and its `Settings` page replaces the starter's `src/pages/Settings.wf`.

```json
{ "theme": { "name": "Light", "dark": "Night" } }
```

This program declares both themes and a settings page with three buttons,
the current choice drawn solid and the others outlined:

```wf
theme Light {
    color-background: #FFFFFF
    color-text: #0F172A
}

theme Night {
    color-background: #0B1220
    color-surface: #111827
    color-text: #E5E7EB
    color-text-muted: #9CA3AF
    color-border: #1F2937
}

page Settings(path: "/settings", title: "Settings", description: "How the site looks.") {
    Heading("Settings").h1
    Row(gap: .sm) {
        Button("Light", outlined: theme != "light") { on click { setTheme("light") } }
        Button("Dark", outlined: theme != "dark") { on click { setTheme("dark") } }
        Button("Match the system", outlined: theme != "system") { on click { setTheme("system") } }
    }
}
```

## How it works

- **Two themes.** `Night` names only the tokens that change in the dark;
  every token it leaves out keeps its value. The config's `theme.dark` says
  which theme is the dark one ([Styling](https://webfluent.monzeromer.dev/learn/13-styling.md#dark-mode)).
- **`setTheme`** takes `"light"`, `"dark"` or `"system"`, and the choice is
  kept across visits. With `"system"`, the dark theme applies whenever the
  reader's system prefers dark (`prefers-color-scheme: dark`).
- **`theme`** reads the choice, and stays current. A `Bool` prop given a
  condition follows it, so `outlined: theme != "light"` outlines every
  button but the chosen one.
