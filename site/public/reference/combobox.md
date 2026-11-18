# Combobox

> A text field that offers what matches as the reader types: from options:, or search: asked of a server. The arrow keys go through the offers, Enter takes one, Escape closes them; .multiple keeps several, each a chip Backspace takes back. option(o) { … } draws an offer; empty { … } shows when nothing matches.

The page: https://webfluent.monzeromer.dev/reference/combobox

A text field that offers what matches as the reader types: from options:, or search: asked of a server. The arrow keys go through the offers, Enter takes one, Escape closes them; .multiple keeps several, each a chip Backspace takes back. option(o) { … } draws an offer; empty { … } shows when nothing matches.

```wf
Combobox(bind: State, options: [Item], search: (String) -> [Item] or (String) -> [Item]!, …)
```

## Example

```wf
state city = "lis"
Combobox(bind: city, options: [
    { id: "ams", name: "Amsterdam" },
    { id: "ber", name: "Berlin" },
    { id: "lis", name: "Lisbon" },
    { id: "mad", name: "Madrid" },
    { id: "par", name: "Paris" },
], text: c => c.name, key: c => c.id, label: "City", placeholder: "Type a city")
Text("Chosen: {city}").muted
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `bind:` | `State (required)` | The state it reads and writes: the chosen option's key (the option itself without `key:`), or with `.multiple` a list of keys in the order chosen |
| `options:` | `[Item]` | Every option, filtered as the reader types to those whose words contain what is typed, case ignored; use this or `search:` |
| `search:` | `(String) -> [Item] or (String) -> [Item]!` | Options asked for as the reader types, each time the typing rests for `debounce:`: a function of the text giving a list, or an endpoint's call (`q => Geo.cities(q: q)`), whose `.ok` list is shown and whose failure shows none |
| `text:` | `(Item) -> String (required)` | An option's words, shown in the list and in the field once chosen: `p => p.name` |
| `key:` | `(Item) -> String or (Item) -> Number` | What tells two options apart, and what `bind:` holds: `p => p.id` |
| `debounce:` | `Number or Duration` | How long the typing must rest before `search:` is asked: a `Duration` (`250.ms`) or milliseconds |
| `multiple:` | `Bool` | Several at once, each a chip with a button to take it out; Backspace in the empty field takes the last back. Written as a flag, `.multiple`, or `multiple: true`. |
| `label:` | `String (required)` | Its name for a screen reader: the field's and the list's (`aria-label`) |
| `placeholder:` | `String` | Shown in the field while it is empty |

## Flags

`.multiple`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
