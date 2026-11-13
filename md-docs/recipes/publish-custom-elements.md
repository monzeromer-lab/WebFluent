---
kind: "recipe"
route: "/recipes/publish-custom-elements"
group: "interop"
description: "To use your components from another framework or a plain page, build them as custom elements."
from: ["/docs/guide/interop#publish-your-components-as-custom-elements"]
---
# Publish components as custom elements

To use your components from another framework or a plain page, build them
as custom elements. Name the components in `build.elements`:

```json
{ "build": { "output_type": "elements", "elements": ["PriceTag", "Rating"] } }
```

Each name becomes a standard custom element. `PriceTag` is `<price-tag>`.
A one-word component gets a prefix, so its tag has the hyphen a custom
element needs: `Rating` is `<wf-rating>`. The build writes `elements.js`,
`styles.css`, and a page that lists the tags it published.

This HTML places a published component on any page:

```html
<script src="/elements.js" defer></script>
<link rel="stylesheet" href="/styles.css">

<price-tag label="Pro" amount="29" sale></price-tag>
```

How a published element behaves:

- **An attribute is a prop.** It is read when the element is connected, and
  again whenever it changes. A `Number` prop is read as a number, and a
  `Map` or a list as JSON.
- **A `Bool` prop is true when the attribute is present**, as HTML reads
  `disabled`, and false when it is `="false"`, so a framework that writes
  the string still works.
- **An event the component declares** is dispatched as a `CustomEvent`
  that bubbles.
- **What the component created goes** when the element leaves the
  document.

React, Vue, Svelte, Angular, Rails, WordPress and a plain page can all
place a tag. That is why WebFluent publishes custom elements rather than
an adapter per framework: the interface every framework shares is the web
platform itself.
