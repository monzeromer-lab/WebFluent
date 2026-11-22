### Icon System

WebFluent includes 83 built-in SVG icons, rendered inline. Available icons:

`home`, `menu`, `search`, `close`, `user`, `settings`, `check`, `plus`, `minus`, `edit`, `trash`, `star`, `heart`, `mail`, `bell`, `download`, `upload`, `eye`, `link`, `calendar`, `filter`, `info`, `warning`, `arrow-left`, `arrow-right`, `chevron-down`, `chevron-right`, `chevron-left`, `logout`, `copy`, `sun`, `moon`; for an editor or a tool: `folder`, `folder-plus`, `folder-off`, `file`, `file-plus`, `refresh`, `alert-circle`, `clock`, `dot`, `diff`, `book`, `git-pull-request`, `pin`, `git-branch`, `more-horizontal`, `window-minimize`, `window-maximize`, `outline`, `help-circle`, `collapse-all`, `locate`, `chevron-up`, `globe`, `external-link`, `panel-down`, `panel-up`, `code`, `monitor`, `git-commit`, `play`, `layers`, `insert`, `grid`, `list`, `square`, `pill`, `image`, `sparkle`, `lock`, `type`, `pointer`, `reach`, `frame`, `line`, `pipette`, `comment`, `hand`, `zoom`, `ruler`, `eye-off`, `unlock`

Usage:

```wf
Icon("home")
Icon("search").lg.primary
IconButton(icon: "close", label: "Close")
Sidebar.Item(to: "/", icon: "home") { Text("Home") }
```

