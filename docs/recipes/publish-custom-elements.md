# Publish components as custom elements

> Build WebFluent components as standard custom elements that React, Vue, Svelte, WordPress or a plain HTML page can place by their tag.

The page: https://webfluent.monzeromer.dev/recipes/publish-custom-elements

To use your components from another framework or a plain page, build them
as custom elements. These two components, a price tag and a star rating,
are what this project publishes; the page places them so you can see them
in the project itself:

```wf
component PriceTag(label: String, amount: Number, sale: Bool = false) {
    Row(gap: .sm, align: .center) {
        Text(label).bold
        Text("${amount}")
        if sale { Badge("Sale").success }
    }
}

component Rating(value: Number = 0) {
    event rate(stars: Number)
    Row(gap: .xs) {
        for n in 1..=5 by n {
            Button("{n}", outlined: n > value) {
                on click { emit rate(n) }
            }
        }
    }
}

page Preview(path: "/", title: "Preview", description: "The components this project publishes.") {
    state stars = 3
    Heading("Preview").h1
    PriceTag(label: "Pro", amount: 29).sale
    Rating(value: stars) { on rate(n) { stars = n } }
}
```

Name the components in `build.elements`, in `webfluent.app.json`:

```json
{ "build": { "output_type": "elements", "elements": ["PriceTag", "Rating"] } }
```

This HTML places the published components on any page, and listens for a
rating:

```html
<script src="/elements.js" defer></script>
<link rel="stylesheet" href="/styles.css">

<price-tag label="Pro" amount="29" sale></price-tag>
<wf-rating value="3"></wf-rating>

<script>
  document.addEventListener("rate", (e) => console.log("rated", e.detail));
</script>
```

## How it works

Each name becomes a standard custom element. `PriceTag` is `<price-tag>`.
A one-word component gets a prefix, so its tag has the hyphen a custom
element needs: `Rating` is `<wf-rating>`. The build writes `elements.js`,
`styles.css`, and a page that lists the tags it published.

A published element behaves this way:

- **An attribute is a prop.** It is read when the element is connected, and
  again whenever it changes. A `Number` prop is read as a number, and a
  `Map` or a list as JSON. A prop with a capital in its name is written
  with a hyphen: `publishableKey` is the attribute `publishable-key`.
- **A `Bool` prop is true when the attribute is present**, as HTML reads
  `disabled`, and false when it is `="false"`, so a framework that writes
  the string still works.
- **An event the component declares** is dispatched as a `CustomEvent`
  that bubbles. Its `detail` is the value the event sends, or a list of
  them when it sends several.
- **What the component created goes** when the element leaves the
  document.

React, Vue, Svelte, Angular, Rails, WordPress and a plain page can all
place a tag. That is why WebFluent publishes custom elements rather than
an adapter per framework: the interface every framework shares is the web
platform itself.

## Troubleshooting

| What you see | What to do |
|---|---|
| The build says `output_type: "elements"` publishes nothing | Name the components in `build.elements` |
| The build names a component the project does not declare | Check the spelling in `build.elements` against the component's name |
| The tag renders as nothing | Load `elements.js` before the tag is used, or let the browser upgrade it when the script arrives (`defer` does) |
