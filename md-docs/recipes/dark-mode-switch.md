---
kind: "recipe"
route: "/recipes/dark-mode-switch"
group: "look"
description: "To let the reader pick light, dark or the system's choice, call `setTheme` and read `theme`:"
from: ["/docs/guide/recipes#add-a-dark-mode-switch"]
---
# Add a dark-mode switch

To let the reader pick light, dark or the system's choice, call `setTheme`
and read `theme`:

```wf
page Settings(path: "/settings", title: "Settings", description: "How the site looks.") {
    Heading("Settings").h1
    Row(gap: .sm) {
        Button("Light", outlined: theme != "light") { on click { setTheme("light") } }
        Button("Dark", outlined: theme != "dark") { on click { setTheme("dark") } }
        Button("Match the system", outlined: theme != "system") { on click { setTheme("system") } }
    }
}
```

The switch needs a dark theme named in the config
([Styling](../learn/13-styling.md#dark-mode)).
