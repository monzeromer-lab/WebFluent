### Browser Globals

The browser's names are used without a prefix, compile to themselves, and
are typed by the language's browser table — so a misspelt member is found
where it is written:

```wf
localStorage.setItem("token", tok)            // (key: String, value: String)
let saved = localStorage.getItem("token")     // String?
window.open("/docs", "_blank", "noopener")
document.title = "New Title"
console.log("saved", count)                   // anything but a Secret
let raw = JSON.parse(text)                    // Json: match it, or decode it
let text = JSON.stringify({ a: 1 })           // any plain data
setTimeout(() => close(), 1000)
await navigator.clipboard.writeText(url)
let wide = Math.max(1, viewport.width / 2)
```

The table types `window`, `document`, `console`, `localStorage`,
`sessionStorage`, `JSON`, `Math`, `Date.now()`, `Object.keys`, `Number.*`,
`navigator` (and its clipboard), `location`, `history`, `performance`,
`screen`, the timers, `parseInt`, `parseFloat`, `encodeURIComponent` and the
rest of the plain functions. A global or a member it does not list —
`navigator.bluetooth`, `crypto` — is `T30`, with the nearest name: reach it
from a project script whose JSDoc says its types (see [Your own
JavaScript](#your-own-javascript)).

A handler's event is the browser's: `on keydown(e)` is a `KeyboardEvent`
(`key`, `code`, `shiftKey`, `ctrlKey`, `altKey`, `metaKey`, `repeat`,
`preventDefault()`), `on click(e)` a `MouseEvent` (`clientX`, `clientY`,
`button`), and so `PointerEvent`, `InputEvent`, `SubmitEvent`,
`FocusEvent`, `WheelEvent`, `DragEvent`, `ClipboardEvent`, `TouchEvent`,
`MessageEvent`; `target` and `currentTarget` are an `Element`. The
runtime's gestures hand over what they say: `on swipe(s)` `s.direction`,
`s.dx`, `s.dy`; `on longpress(p)` `p.x`, `p.y`; `on receive(d)` `d.value`
(`Json`), `d.over`, `d.position`. A `ref:` handle is an `Element` too —
`value`, `checked`, `focus()`, `blur()`, `select()`, `scrollIntoView()`,
`getBoundingClientRect()`, `dataset`, `closest(…)`, `contains(…)`.

`catch e` binds an `Error` — `e.name`, `e.message`, `e.stack` — whatever
was thrown: the runtime wraps a string, a number or a plain object a script
throws in one, its text the message.

<!-- core -->
