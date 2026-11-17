# InputGroup

> An input drawn as one box with what goes beside it: an icon, InputGroup.Addon("https://"), a button, or InputGroup.Clear, which empties the input and gives it the focus back.

The page: https://webfluent.monzeromer.dev/reference/inputgroup

An input drawn as one box with what goes beside it: an icon, InputGroup.Addon("https://"), a button, or InputGroup.Clear, which empties the input and gives it the focus back.

```wf
InputGroup { … }
```

## Example

```wf
state search = ""
state domain = "acme"
Stack(gap: .md) {
    InputGroup {
        Icon("search")
        Input(bind: search, placeholder: "Search the docs", aria-label: "Search the docs")
        InputGroup.Clear
    }
    InputGroup {
        InputGroup.Addon("https://")
        Input(bind: domain, aria-label: "Subdomain")
        InputGroup.Addon(".example.com")
    }
    Text("Searching for “{search}” on https://{domain}.example.com").muted
}
```

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Slots

- `children`

## Parts

Written under its name, inside `InputGroup`.

### InputGroup.Addon

Text beside the input: https://, .com, kg.

```wf
InputGroup.Addon(_ text: String) { … }
```

| Prop | Type | Meaning |
|---|---|---|
| `text` | `String  (positional)` | The text |

### InputGroup.Clear

A button that empties the input and puts the focus back in it; hidden while it is empty.

```wf
InputGroup.Clear
```

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
