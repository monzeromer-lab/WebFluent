---
kind: "recipe"
route: "/recipes/route-transitions"
group: "look"
description: "To animate every change of page, give the `Router` a `transition:` and a `duration:`:"
from: ["/docs/guide/motion#animate-route-changes"]
---
# Animate route changes

To animate every change of page, give the `Router` a `transition:` and a
`duration:`:

```wf
app {
    Navbar {
        Navbar.Brand { Link("Motion", to: "/") }
        Navbar.Links { Link("Home", to: "/")  Link("About", to: "/about") }
    }
    Router(transition: .fade, duration: "200ms")
}

page Home(path: "/", title: "Home", description: "The front page.") { Heading("Home").h1 }
page About(path: "/about", title: "About", description: "Who we are.") { Heading("About").h1 }
```

`transition:` is `.fade`, `.slide` or `.none` (the next page replaces the
last at once). On each route change the old page plays out and the new one
in; then focus and scroll settle and the new title is announced.

Where the browser has the View Transitions API, the change plays through
it: the browser's own crossfade, or the slide the engine's stylesheet
defines for `[data-wf-transition="slide"]`. Your own `::view-transition-*`
rules in a project stylesheet may restyle either. In other browsers, a
class-based animation plays instead.
