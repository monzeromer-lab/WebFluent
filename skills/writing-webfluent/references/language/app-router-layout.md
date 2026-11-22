### App (Router + Layout)

`app` is the root of the site: what wraps every page, and a bare `Router`
where the current page renders. The Router can sit at any depth.

```wf
// Simple: Router at top level
app {
    Navbar {
        Navbar.Brand { Link("Ledger", to: "/") }
        Navbar.Links { Link("Home", to: "/") }
    }
    Router
}
```

```wf
// Sidebar layout: Router nested inside Row > Container
app {
    Row {
        style { min-height: 100vh }

        Sidebar {
            style {
                width: 220px
                position: fixed
                top: 0
                left: 0
                bottom: 0
                background: #1A1A19
            }
            Sidebar.Header { Text("My App").bold }
            Sidebar.Item(to: "/", icon: "home") { Text("Home") }
            Sidebar.Item(to: "/about", icon: "info") { Text("About") }
        }

        Container {
            style {
                margin-left: 220px
                flex: 1
            }

            Router
        }
    }
}
```

`Router(transition: .fade, duration: "200ms")` plays the old page out and the
new one in on a route change (`.slide` slides them). A page that names a
`layout:` is framed by that component instead of, or as well as, the app's
own chrome.

<!-- core -->
