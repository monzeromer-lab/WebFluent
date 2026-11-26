### Publishing custom elements

The `elements` output type compiles chosen components into custom elements other sites can use.

```json
{ "build": { "output_type": "elements", "elements": ["PriceTag", "Rating"] } }
```

`PriceTag` becomes `<price-tag>`; a one-word component is prefixed so its
tag has the hyphen a custom element needs (`<wf-rating>`). The build writes
`elements.js`, `styles.css` and a page listing the tags. An attribute is a
prop — a `Number` read as a number, a record, a dictionary or a list as JSON, a `Bool` true
when present and false when `="false"` — read again whenever it changes; an
event the component declares is a bubbling `CustomEvent`; what the
component made goes when the element leaves the document. React, Vue,
Svelte, Rails, WordPress and a plain page can all place a tag, which is why
this is one output rather than an adapter per framework.

