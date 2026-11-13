---
kind: "recipe"
route: "/recipes/show-an-icon"
group: "look"
description: "To draw an icon, write `Icon(\"name\")`."
from: ["/docs/guide/media#show-an-icon"]
---
# Show an icon

To draw an icon, write `Icon("name")`. The icon is an SVG drawn inline in
the current text colour, so it follows the theme. The `.sm` and `.lg` flags
size it, and a tone flag such as `.primary` colours it.

This page shows icons on their own, on an icon-only button, and in a
sidebar:

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

`IconButton` is a button that shows only an icon. Its `label:` is its
accessible name for screen readers, and also its tooltip unless you give it
a `title:`. `Sidebar.Item` takes an `icon:` too.

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

For an icon that is not built in, put an SVG file in `public/` and show it
with `Image(src: "/icons/rocket.svg", alt: "")`.
