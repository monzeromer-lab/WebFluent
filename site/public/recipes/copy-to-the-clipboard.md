# Copy to the clipboard

> Copy text to the reader's clipboard from a button: an action that awaits the browser's clipboard, then shows a toast.

The page: https://webfluent.monzeromer.dev/recipes/copy-to-the-clipboard

A **Copy** button saves the reader selecting a link by hand. To copy text,
call the browser's clipboard from an action and say when it worked:

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

## How it works

- `navigator.clipboard` and `location` are browser globals, used as written
  with no prefix.
- `writeText` returns a promise, so the
  [action](https://webfluent.monzeromer.dev/learn/07-state-and-reactivity.md#actions-and-locals) awaits
  it. `await` makes the action async, and `copied` is set only once the
  text is on the clipboard.
- The button's click handler calls `copy` with the page's own address.
- When `copied` turns true, the `Toast` appears to confirm it.
