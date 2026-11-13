---
kind: "recipe"
route: "/recipes/confirm-dialog"
group: "interaction"
description: "To ask before a destructive action, wrap a `Dialog` in a component that fires `confirm` or `cancel`:"
from: ["/docs/guide/recipes#build-a-confirm-dialog-component"]
---
# Build a confirm dialog

To ask before a destructive action, wrap a `Dialog` in a component that
fires `confirm` or `cancel`:

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

The caller's block fills the default slot, so the question's detail is the
caller's to write.
