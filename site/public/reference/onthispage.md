# OnThisPage

> The page's headings as links, the one being read marked as the reader scrolls: a table of contents beside a long page.

The page: https://webfluent.monzeromer.dev/reference/onthispage

The page's headings as links, the one being read marked as the reader scrolls: a table of contents beside a long page.

```wf
OnThisPage(levels: Number, label: String)
```

## Example

```wf
Row(gap: .lg, align: .start) {
    Stack {
        Heading("Deploying").h1
        Heading("Build the site").h2
        Text("Run wf build; the output is in build/.")
        Heading("Choose a host").h2
        Heading("Static hosts").h3
        Text("Any host that serves files.")
        Heading("Set the base path").h2
        Text("Under a folder, name it in the config.")
    }
    OnThisPage(levels: 2, label: "On this page")
}
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `levels:` | `Number` | How many heading levels it lists, from `h2`: `1` is `h2` alone, `2` adds `h3`, up to 5 |
| `label:` | `String` | Its title, and its accessible name (`aria-label` on its `<nav>`) |

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## Drawn for it

What `OnThisPage` draws itself, by its class; not written.

- `OnThisPage.Title` (`.`): Its title.

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
