# Animate route changes

> Animate every change of page: give the Router a transition: of .fade or .slide and a duration:, played through the View Transitions API.

The page: https://webfluent.monzeromer.dev/recipes/route-transitions

Fade or slide from one page to the next, on every route change, by giving
the `Router` a `transition:` and a `duration:` in `app`:

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

## How it works

- `transition:` is `.fade`, `.slide` or `.none` (the next page replaces the
  last at once). `duration:` is how long the change takes.
- On each route change the old page plays out and the new one in; then
  focus and scroll settle and the new title is announced.
- Where the browser has the View Transitions API, the change plays through
  it: the browser's own crossfade, or the slide the engine's stylesheet
  defines for `[data-wf-transition="slide"]`. Your own
  `::view-transition-*` rules in a project stylesheet may restyle either.
  In other browsers, a class-based animation plays instead.

A reader who asked for reduced motion gets no route transition: the next
page replaces the last at once
([Respect reduced motion](https://webfluent.monzeromer.dev/recipes/motion-defaults.md#respect-reduced-motion)).

## Variations

### Slide instead of fade

`.slide` slides the old page out and the new one in:

```wf
app {
    Navbar {
        Navbar.Brand { Link("Motion", to: "/") }
        Navbar.Links { Link("Home", to: "/")  Link("About", to: "/about") }
    }
    Router(transition: .slide, duration: "250ms")
}

page Home(path: "/", title: "Home", description: "The front page.") { Heading("Home").h1 }
page About(path: "/about", title: "About", description: "Who we are.") { Heading("About").h1 }
```
