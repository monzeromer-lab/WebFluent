---
kind: "recipe"
route: "/recipes/copy-to-the-clipboard"
group: "interaction"
description: "To copy text, call the browser's clipboard from an action:"
from: ["/docs/guide/recipes#copy-to-the-clipboard"]
---
# Copy to the clipboard

To copy text, call the browser's clipboard from an action:

```wf
page Share(path: "/share", title: "Share", description: "Copy the link.") {
    state copied = false
    action copy(text: String) {
        await navigator.clipboard.writeText(text)
        copied = true
    }
    Heading("Share").h1
    Button("Copy the link") { on click { copy(location.href) } }
    if copied { Toast("Copied").success }
}
```
