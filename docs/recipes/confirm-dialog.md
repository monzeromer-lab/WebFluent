# Build a confirm dialog

> Ask before a destructive action: a component that wraps a Dialog and fires confirm or cancel, with the question's detail written by the caller.

The page: https://webfluent.monzeromer.dev/recipes/confirm-dialog

Before a reader deletes something, ask them. This component wraps a
`Dialog` with **Cancel** and **Confirm** buttons and tells the page which
one was pressed, so every destructive action on the site asks the same way:

**Needs:** nothing besides the page. It claims `/`: paste it into a new
project, or replace the starter's `src/pages/Home.wf` with it.

```wf
component Confirm(_ title: String, visible: Bool, danger: Bool = false) {
    event confirm()
    event cancel()
    slot
    Dialog(visible: visible, title: title) {
        children
        Row(gap: .sm, justify: .end) {
            Button("Cancel") { on click { emit cancel() } }
            Button("Confirm", tone: if danger { .danger } else { .primary }) { on click { emit confirm() } }
        }
    }
}

page Account(path: "/", title: "Account", description: "Close your account.") {
    state asking = false
    state gone = false
    Heading("Account").h1
    Button("Delete account").danger { on click { asking = true } }
    Confirm("Delete your account?", visible: asking, danger: true) {
        on confirm { gone = true  asking = false }
        on cancel { asking = false }
        Text("Everything goes. This cannot be undone.")
    }
    if gone { Alert("Account deleted.").info }
}
```

## How it works

- `Confirm` takes the question as its positional `title`, whether it shows
  as `visible`, and a `danger` prop that turns the confirm button red.
  `tone:` picks the button's case from a condition instead of a fixed flag.
- It declares two [events](https://webfluent.monzeromer.dev/learn/10-components.md#events), `confirm` and
  `cancel`, and each button fires one with `emit`. The component decides
  nothing; the page does.
- `slot` declares a default [slot](https://webfluent.monzeromer.dev/learn/10-components.md#slots), and
  `children` places it inside the dialog. The caller's block fills it, so
  the question's detail is the caller's to write.
- The page owns `asking`. **Delete account** sets it, and both handlers
  clear it, which closes the
  [dialog](https://webfluent.monzeromer.dev/learn/08-events.md#overlays-modals-dialogs-toasts-and-menus).
