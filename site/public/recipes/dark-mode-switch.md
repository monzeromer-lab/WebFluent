# Add a dark-mode switch

> Let the reader pick light, dark or the system's choice with setTheme, and mark the current choice by reading theme.

The page: https://webfluent.monzeromer.dev/recipes/dark-mode-switch

Let the reader pick a light look, a dark one, or whatever their system
prefers, and keep that choice across visits. The switch needs a dark theme
named in `webfluent.app.json`:

**Needs:** the `theme` setting below in `webfluent.app.json`. The `spa`
starter already has it, declares these two themes in `src/theme.wf`, and
places a `DarkModeSwitch` component in its header (its `Night` names no
`color-on-primary`; add the line below). This page replaces the starter's
`src/pages/Settings.wf`. In a new project, put the themes in
`src/theme.wf`.

```json
{ "theme": { "name": "Day", "dark": "Night" } }
```

This program declares both themes and a settings page with three buttons,
the current choice drawn solid and the others outlined:

```wf
/// The look by day. Every token not named here keeps its baseline value.
theme Day {
    color-primary: #2563EB
    color-on-primary: #FFFFFF
    color-background: #F8FAFC
    color-surface: #FFFFFF
    color-text: #0F172A
    color-text-muted: #64748B
    color-border: #E2E8F0
}

/// The same tokens after dark.
theme Night {
    color-primary: #60A5FA
    color-on-primary: #0B1120
    color-background: #0B1120
    color-surface: #131C2E
    color-text: #E2E8F0
    color-text-muted: #94A3B8
    color-border: #1E293B
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

- **Two themes.** The config's `theme.name` is the one the site starts
  with and `theme.dark` the one for the dark
  ([Styling](https://webfluent.monzeromer.dev/learn/13-styling.md#dark-mode)). A token the dark theme
  leaves out keeps the light theme's value, so name the same tokens in both.
- **Every built-in draws from these tokens**, so the buttons, the cards and
  the fields change with the theme and nothing else needs a dark rule. A
  raw colour in a `style { }` block does not change; use a token there
  (`background: $color-surface`).
- **`color-on-primary`** is the colour of a label written on the primary
  colour: a primary button's text, a selected day, a pressed toggle. The
  dark primary is light, so its label is dark. `wf theme` prints each pair
  and its contrast, and `A13` names a pair below the WCAG AA ratio.
- **`setTheme`** takes `"light"`, `"dark"` or `"system"`, and the choice is
  kept across visits. With `"system"`, the dark theme applies whenever the
  reader's system prefers dark (`prefers-color-scheme: dark`).
- **`theme`** reads the choice, and stays current. A `Bool` prop given a
  condition follows it, so `outlined: theme != "light"` outlines every
  button but the chosen one.

## Variations

### One button

The built-in `ThemeToggle` is a single button that switches between the
two: a moon on a light page, a sun on a dark one, named for the theme it
switches to. Put it in a header:

```wf
app {
    Navbar {
        Navbar.Brand { Text("Ledger").bold }
        Navbar.Actions { ThemeToggle }
    }
    Router
}
```

### A labelled switch

The `spa` starter's `DarkModeSwitch` (`src/components/DarkModeSwitch.wf`)
is a `Switch` labelled "Dark mode". It is on when the reader chose dark, or
chose nothing and their system prefers dark, and it calls `setTheme` with
the other one when it is flipped.
