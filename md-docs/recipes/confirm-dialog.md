---
kind: "recipe"
route: "/recipes/confirm-dialog"
group: "interaction"
description: "Ask before a destructive action: an AlertDialog in a component that fires confirm or cancel, or one line, await ask(…)."
outputs: ["website", "app"]
from: ["/docs/guide/recipes#build-a-confirm-dialog-component"]
---
# Build a confirm dialog

Before a reader deletes something, ask them. This component wraps an
`AlertDialog` with **Cancel** and **Confirm** buttons and tells the page which
one was pressed, so every destructive action on the site asks the same way:

**Needs:** nothing besides the page. It claims `/`: paste it into a new
project, or replace the starter's `src/pages/Home.wf` with it.

```wf
component Confirm(_ title: String, visible: Bool, danger: Bool = false) {
    event confirm()
    event cancel()
    slot
    AlertDialog(visible: visible, title: title) {
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
- It declares two [events](../learn/10-components.md#events), `confirm` and
  `cancel`, and each button fires one with `emit`. The component decides
  nothing; the page does.
- `slot` declares a default [slot](../learn/10-components.md#slots), and
  `children` places it inside the dialog. The caller's block fills it, so
  the question's detail is the caller's to write.
- The page owns `asking`. **Delete account** sets it, and both handlers
  clear it, which closes the
  [dialog](../learn/08-events.md#overlays-modals-dialogs-toasts-and-menus).
- An `AlertDialog` is one to a screen reader, read with its body. Escape
  closes it as **Cancel** would, and a press beside it does not: an answer
  is a button.

## One question, from a handler

When the question is asked once, from one handler, `await ask(…)` draws the
alert dialog itself and is the answer:

```wf
page Account(path: "/", title: "Account", description: "Close your account.") {
    state gone = false
    Heading("Account").h1
    Button("Delete account").danger {
        on click {
            if await ask("Delete your account?", detail: "Everything goes. This cannot be undone.", yes: "Delete", danger: true) {
                gone = true
            }
        }
    }
    if gone { Alert("Account deleted.").info }
}
```

The focus starts on **Cancel**, the answer that loses nothing, and goes
back to the button once it is answered.
