---
chapter: 18
kind: "about"
route: "/about/why"
from: ["/docs/guide/introduction"]
group: "about"
blurb: "Why the language exists, and the choices it makes: one meaning per word, a compiler that refuses what would break in the browser, and one source for the web and for paper."
description: "Why WebFluent exists: one meaning per word, a compiler that refuses what would fail in the browser, and one source for sites, PDFs and slides."
---
# Why WebFluent

This essay argues for WebFluent: why a language for websites, PDFs and
slide decks should exist when JavaScript frameworks already do the job, and
what it gives up to be what it is.

The argument in one line: a website is a program whose mistakes are found
by its readers, and most of those mistakes can be found by a compiler
first, if the language is designed so that the compiler can see them.

## What breaks a website

Here is a page with a counter, the smallest program that has every part a
site has: an address, text, state, and something the reader does.

```wf
page Home(path: "/", title: "Hello", description: "A first WebFluent page.") {
    state count = 0
    Container {
        Heading("Hello, world").h1
        Text("You have clicked {count} times.")
        Button("Click me").primary { on click { count = count + 1 } }
    }
}
```

Now count the ways the same page goes wrong in a typical stack. A class
name misspelled in the markup does nothing. A prop a component does not
take is ignored. A field read from a record that does not have it is
`undefined`, and the page shows `undefined`. A value that may be `null`
crashes the page only for the reader whose data has a gap. A link to a
route that was renamed is a 404 that nobody sees until a reader follows
it. None of these is a hard problem. Each one is a problem the toolchain
had all the information to report, and did not.

WebFluent starts from the other end. Every word in a program resolves to
one declaration the compiler knows: a built-in in the registry, or a
component, store, type or page the project declares. A word that resolves
to nothing is an error, not a no-op.

```wf expect=E103
page Home(path: "/", title: "Hello", description: "A first WebFluent page.") {
    Button("Save").primry
}
```

The build stops on that line, says that `Button` has no flag `primry`, and
lists the ones it takes. The browser never sees the page.

## One lexeme, one meaning

The compiler can only check what it can read, so the grammar is built for
reading. Each kind of word means one thing:

- A lowercase keyword declares: `page`, `component`, `store`, `state`,
  `derived`, `action`, `type`, `enum`, `api`.
- A capitalised name is an element or a type: `Button`, `Todo`.
- `.primary` after a call sets a `Bool` prop or picks the case of an enum
  prop, and nothing else.
- `name: value` passes a prop. A block is the element's children.
  `on click { }` is a handler. `style { }` is CSS.

There is no second spelling for any of these, no attribute that is
sometimes a prop and sometimes markup, no string that is sometimes a
template. That is a constraint on the author, and it is the reason the
compiler, the linters, the formatter and the language server can all read
one registry and agree. When the editor offers you the flags of a `Button`
after a dot, it is reading the same table the build refuses `primry` from.

The cost is real: you cannot invent syntax, and a library cannot extend
the grammar. A component you write is called exactly as a built-in is, and
that is the extension point the language offers.

## Refuse what would fail in the browser

A compiler that stopped at spelling would be a linter. WebFluent's goes
further, because most of what fails in a browser is a type error or a
reactivity error in disguise:

- A value that may be `null`, read as if it were not, is `T04`. An
  `if let`, a `??` or a `!= null` check narrows it.
- A `state` that reads `now` once and so never ticks is `X06`; the fix the
  compiler offers is `derived`.
- An effect that writes what it reads, and so never settles, is `X02`.
- A link to a path no page answers is `R01`.
- A `Secret` shown, logged, spliced into a URL or kept with `persist` is
  `T12`.
- A page the build's own Content-Security-Policy would block is `E902`;
  the build reads its output back and holds it to the policy it ships.

Each finding has a code, a place, a hint and, where there is one right
answer, a fix the editor applies. The bet is that a project with no
findings is a project whose pages load, and `wf verify` exists to test the
bet: it opens every route in a real browser and fails on an uncaught error,
a failed request or a page that drew nothing.

Types make most of this possible, and they are optional. A value the
checker cannot work out is `Any`, which agrees with everything, so a
program that declares no types checks as it always would. Every annotation
you add narrows what the checker can say.

## Secure by construction, not by advice

The same idea applies to security. Advice such as "escape your output" or
"never put a token in `localStorage`" is agreed to in every review and hard to
follow on every line. WebFluent turns the advice into properties of the
output: every string an element shows is a text node; markup goes in only
through `Unsafe.Html`, which is named so a review finds it; an `href:`
whose scheme a browser would run is refused where it is written; an `env`
name that is not public cannot be read from a page. The
[security essay](security.md) lists each guarantee and what is still yours.

## One source for the web and for paper

A team that publishes a site usually publishes documents too: an invoice,
a report, the slides for the quarter. These tend to live in a second
toolchain, with a second set of styles that drifts from the first.

WebFluent lays a PDF out from the same HTML and CSS the page would get, by
its own paged engine, with no browser and no external tool. A `Card` in a
PDF looks like the `Card` on the site, a theme's tokens reach both, and a
`Presentation` makes a deck from the same components. The same templates
render with JSON data from Rust or Node on a server. One language, one
look, one set of checks.

## Fine-grained, with no virtual DOM

A `state` is a signal. What reads it is what redraws: the text node that
splices `{count}`, an attribute, a branch. Nothing re-renders a component
to find out what changed, so there is no virtual DOM to diff and no memo
to forget. [How reactivity works](how-reactivity-works.md) follows a click
from the handler to the text node it changes.

## One binary

`wf` is the compiler, the dev server, the formatter, the test runner and
the template renderer; its sibling `wf-lsp` is the language server, built
from the same compiler. There is no `node_modules`, no
bundler, and no configuration to write before the first page. A build
writes only the runtime modules the program reaches and only the component
rules it uses, so a page of text ships a few kilobytes of script. The
[essay on what a build ships](performance.md) shows the numbers.

## What it gives up

A language with opinions is the wrong tool for some work:

- **A framework's ecosystem.** There is no package of WebFluent
  components to install from npm. Any browser library can be used from a
  plain script under `src/` or a `meta.scripts` entry, but you write the
  binding yourself.
- **Rendering per request.** A WebFluent site is static files and a
  browser runtime; your server stays your server. The template engine
  renders HTML and PDFs with data on a server, but it runs no handlers
  after the render.
- **Freedom of syntax.** One spelling for each thing means none of your
  own.

If those costs are acceptable, what you get in return is a site whose
mistakes the compiler finds before your readers do.

## Where to go from here

- To learn the language, start with
  [Your first page](../learn/01-getting-started.md).
- Coming from a framework, read
  [Coming from React](../coming-from/react.md) (or Vue, or Svelte) first.
- To look a fact up, use the [reference](../reference/components.md).
- This guide describes WebFluent 6. [Upgrading](upgrading.md) lists what
  each release changed.
