# Tree

> Nested items — files and folders, an outline — as a tree a keyboard can work: Up and Down through what shows, Right to open and go in, Left to close and go out (the other way round right to left), Home and End, a letter. row(item) { … } draws each; bind: holds the chosen item's key, or a list of keys with .multiple.

The page: https://webfluent.monzeromer.dev/reference/tree

Nested items — files and folders, an outline — as a tree a keyboard can work: Up and Down through what shows, Right to open and go in, Left to close and go out (the other way round right to left), Home and End, a letter. row(item) { … } draws each; bind: holds the chosen item's key, or a list of keys with .multiple.

```wf
Tree(items: [Item], children: (Item) -> [Item]?, key: (Item) -> String or (Item) -> Number, …)
```

## Example

```wf
state file = "src/pages/Home.wf"
Tree(items: [
    { path: "src", name: "src", children: [
        { path: "src/App.wf", name: "App.wf" },
        { path: "src/pages", name: "pages", children: [
            { path: "src/pages/Home.wf", name: "Home.wf" },
            { path: "src/pages/About.wf", name: "About.wf" },
        ] },
    ] },
    { path: "webfluent.app.json", name: "webfluent.app.json" },
], children: f => f.children ?? [], key: f => f.path, bind: file, open: ["src", "src/pages"], label: "Files") {
    row(f) {
        Icon(if f.children != null { "folder" } else { "file" })
        Text(f.name)
    }
}
Text("Open: {file}").muted
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `items:` | `[Item] (required)` | The items at the top level |
| `children:` | `(Item) -> [Item]? (required)` | An item's children: `f => f.children`; null or an empty list for a leaf |
| `key:` | `(Item) -> String or (Item) -> Number (required)` | What tells two items apart: `f => f.path`; what `bind:` and `open:` hold |
| `bind:` | `State` | The state holding the chosen item's key, written on Enter, Space or a press; with `.multiple`, a list of keys |
| `multiple:` | `Bool` | Several chosen at once: Space, Shift with an arrow, Ctrl or ⌘ with A; `bind:` holds a list. Written as a flag, `.multiple`, or `multiple: true`. |
| `open:` | `[String] or [Number]` | The keys of the items open when it is first drawn |
| `label:` | `String (required)` | What the tree is, its accessible name |

## Flags

`.multiple`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take

Since WebFluent 6.2.0.
