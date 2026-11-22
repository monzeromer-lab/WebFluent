### Toasts, questions and announcements

```wf
toast("Conversation archived", action: "Undo", onAction: restore)   // tone:, duration:, closable:
toast.promise(Profile.save(), loading: "Saving…", done: "Saved", failed: "Not saved")
if await ask("Delete the project?", detail: "This cannot be undone.", yes: "Delete", danger: true) { … }
announce("{results.length} results")                                 // politely
announce("Connection lost", .assertive)                              // at once
```

A toast is drawn above everything, a modal included, and said to a screen
reader; it waits while the pointer is on it or the focus is in it, has a
close button, and no more than three show at once (a fourth closes the
oldest). `ask` draws an alert dialog, the focus on its no, and is `true` for
yes and `false` for no or Escape (`confirm` is the browser's). A program's
own `toast`, `ask` or `announce` — a state, an action — keeps its name; the
function is then out of reach there.

