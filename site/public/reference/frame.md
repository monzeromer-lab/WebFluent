# Frame

> Another page, shown inside this one: an iframe — the page at src:, or the HTML srcdoc: gives it, sandboxed as sandbox: says. on message(m) hears what that page posts to this one (m.data), and nothing any other page posts. Without title: it draws an A17 warning: a frame is announced by its title.

The page: https://webfluent.monzeromer.dev/reference/frame

Another page, shown inside this one: an iframe — the page at src:, or the HTML srcdoc: gives it, sandboxed as sandbox: says. on message(m) hears what that page posts to this one (m.data), and nothing any other page posts. Without title: it draws an A17 warning: a frame is announced by its title.

```wf
Frame(src: String, srcdoc: String, title: String, …)
```

## Example

```wf
state version = 2
state heard = "nothing yet"
Stack(gap: .sm) {
    Row(gap: .sm) {
        Button("Version 1") { on click { version = 1 } }
        Button("Version 2") { on click { version = 2 } }
    }
    Frame(srcdoc: "<p style='font-family:sans-serif'>Release notes, version {version}</p>",
          title: "Release notes preview", sandbox: "") {
        style { width: 100%; height: 120px; border: 1px solid $color-border; border-radius: 8px }
        on load { heard = "version {version} loaded" }
    }
    Text("Frame says: {heard}").muted.sm
}
```

## Props

| Prop | Type | Meaning |
|---|---|---|
| `src:` | `String` | The address of the page it shows |
| `srcdoc:` | `String` | The HTML it draws in place of an address, sandboxed as `sandbox:` says; a value that reads state draws the new HTML as it changes |
| `title:` | `String` | What it shows, in a few words: its accessible name |
| `sandbox:` | `String` | What the framed page may do: `"allow-scripts allow-forms"`; empty, it may do nothing |
| `allow:` | `String` | The features it may use: `"fullscreen; clipboard-write"` |
| `loading:` | `.lazy .eager` | Unset, at once |

## Flags

`.lazy` `.eager`

## Events

- `message`
- `load`

Every element also takes the props, events and attribute families listed [on every element](https://webfluent.monzeromer.dev/reference.md#on-every-element).

## What it can raise

- [E103](https://webfluent.monzeromer.dev/errors/E103.md): A flag, case, part, event or slot the component does not have
- [E109](https://webfluent.monzeromer.dev/errors/E109.md): An element this output cannot draw
- [C04](https://webfluent.monzeromer.dev/errors/C04.md): An attribute a built-in does not declare
- [C05](https://webfluent.monzeromer.dev/errors/C05.md): A positional argument a built-in does not take
- [A17](https://webfluent.monzeromer.dev/errors/A17.md): A frame with no title
- [V03](https://webfluent.monzeromer.dev/errors/V03.md): Markup put in as markup

Since WebFluent 5.9.0.
