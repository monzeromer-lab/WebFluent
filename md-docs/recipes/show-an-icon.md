---
kind: "recipe"
route: "/recipes/show-an-icon"
group: "media"
description: "Draw one of the 83 built-in icons inline with Icon(\"name\"), size and colour it with flags, and use one on a button or a sidebar item."
from: ["/docs/guide/media#show-an-icon"]
outputs: ["website", "app"]
---
# Show an icon

Draw an icon with `Icon("name")`: one of 83 built-in SVG icons, drawn inline
in the current text colour, so it follows the theme. Use it beside text, on
an icon-only button or in a sidebar.

This page shows icons on their own, on an icon-only button, and in a
sidebar:

**Needs:** nothing besides the page. It claims `/`: paste it into a new
project, or replace the starter's `src/pages/Home.wf` with it.

```wf
page Inbox(path: "/", title: "Inbox", description: "Messages, with icons.") {
    Heading("Inbox").h1
    Row(gap: .sm) {
        Icon("mail")
        Icon("bell").lg.primary
        IconButton(icon: "close", label: "Close the inbox")
    }
    Sidebar {
        Sidebar.Item(to: "/", icon: "home") { Text("Home") }
    }
}
```

## How it works

- `Icon("mail")` draws the icon named `mail`. The `.sm` and `.lg` flags
  size it, and a tone flag such as `.primary` colours it.
- `IconButton` is a button that shows only an icon. Its `label:` is its
  accessible name for screen readers, and also its tooltip unless you give
  it a `title:`.
- `Sidebar.Item` takes an `icon:` too.

## The built-in icons

These are the 83 built-in icons:

- **General**: `home`, `menu`, `search`, `close`, `user`, `settings`,
  `check`, `plus`, `minus`, `edit`, `trash`, `star`, `heart`, `mail`,
  `bell`, `download`, `upload`, `eye`, `link`, `calendar`, `filter`, `info`,
  `warning`, `arrow-left`, `arrow-right`, `chevron-down`, `chevron-right`,
  `chevron-left`, `logout`, `copy`, `sun`, `moon`.
- **For an editor or a tool**: `folder`, `folder-plus`, `folder-off`,
  `file`, `file-plus`, `refresh`, `alert-circle`, `clock`, `dot`, `diff`,
  `book`, `git-pull-request`, `pin`, `git-branch`, `more-horizontal`,
  `window-minimize`, `window-maximize`, `outline`, `help-circle`,
  `collapse-all`, `locate`, `chevron-up`, `globe`, `external-link`,
  `panel-down`, `panel-up`, `code`, `monitor`, `git-commit`, `play`,
  `layers`, `insert`, `grid`, `list`, `square`, `pill`, `image`, `sparkle`,
  `lock`, `type`, `pointer`, `reach`, `frame`, `line`,
  `pipette`, `comment`, `hand`, `zoom`, `ruler`, `eye-off`, `unlock`.

A name that is not in the list draws as the word itself, and the compiler
warns with `V08`.

> [!IMPORTANT]
> The build keeps only the glyphs your program names. If an icon's name
> arrives at run time, set `"build": { "runtime": "full" }` so every glyph
> ships.

## Use an icon that is not built in

For an icon that is not built in, put an SVG file in `public/` and show it
with `Image(src: "/icons/rocket.svg", alt: "")`.

## Troubleshooting

| What you see | Why, and what to do |
|---|---|
| An icon shows as a word | The name is not one of the built-in icons (`V08`), or it arrives at run time and the build left its glyph out. Use a listed name, or set `build.runtime` to `"full"` |
