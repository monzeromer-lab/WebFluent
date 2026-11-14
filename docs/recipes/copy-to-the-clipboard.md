# Copy to the clipboard

> Copy text to the reader's clipboard from a button: an action that awaits the browser's clipboard, then shows a toast.

The page: https://webfluent.monzeromer.dev/recipes/copy-to-the-clipboard

A **Copy** button saves the reader selecting a link by hand. To copy text,
call the browser's clipboard from an action and say when it worked:

```wf
page Share(path: "/share", title: "Share", description: "Copy the link.") {
    state copied = false
    state failed = false
    action copy(text: String) {
        copied = false
        failed = false
        try {
            await navigator.clipboard.writeText(text)
            copied = true
        } catch e {
            failed = true
        }
    }
    Heading("Share").h1
    Button("Copy the link") { on click { copy(location.href) } }
    if copied { Toast("Copied").success }
    if failed { Toast("Could not copy. Select the address and copy it by hand.").danger }
}
```

## How it works

- `navigator.clipboard` and `location` are browser globals, used as written
  with no prefix.
- `writeText` returns a promise, so the
  [action](https://webfluent.monzeromer.dev/learn/07-state-and-reactivity.md#actions-and-locals) awaits
  it. `await` makes the action async, and `copied` is set only once the
  text is on the clipboard.
- The browser may refuse: the page is not focused, it is not served over
  HTTPS, or the reader denied the permission. The promise then rejects, the
  `catch` sets `failed`, and the reader is told to copy by hand instead of
  seeing nothing.
- The button's click handler calls `copy` with the page's own address.
- A `Toast` shows once, when its branch is drawn. The action sets `copied`
  and `failed` back to `false` before it starts, so the branch is drawn again
  on every click and a second copy shows a second toast. Without that,
  `copied` would stay `true` and the second click would show nothing.
