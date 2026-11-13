---
kind: "recipe"
route: "/recipes/responsive-navigation"
group: "interaction"
description: "To show links on a wide screen and a menu button on a narrow one, branch on `viewport`:"
from: ["/docs/guide/recipes#make-navigation-responsive"]
---
# Make navigation responsive

To show links on a wide screen and a menu button on a narrow one, branch on
`viewport`:

```wf
page Nav(path: "/", title: "Home", description: "Navigation that fits the screen.") {
    state open = false
    Heading("Home").h1
    if viewport.md {
        Row(gap: .md) {
            Link("Home", to: "/")
            Link("Docs", to: "/docs")
            Link("Pricing", to: "/pricing")
        }
    } else {
        IconButton(icon: "menu", label: "Open navigation") { on click { open = !open } }
        show open {
            Stack(gap: .sm, animate: .slideDown) {
                Link("Home", to: "/")
                Link("Docs", to: "/docs")
                Link("Pricing", to: "/pricing")
            }
        }
    }
}
```
