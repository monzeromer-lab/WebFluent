### Events

```wf
Button("Click") {
    on click { doSomething() }
}
Input(bind: term).text {
    on input { search(term) }
}
Form {
    on submit(event) { event.preventDefault()  saveData() }
}
Input(bind: q) {
    on keydown(e) { if e.key == "Escape" { q = "" } }
}
```

DOM events: `click`, `input`, `change`, `submit`, `focus`, `blur`,
`keydown`, `keyup`, `keypress`, `mouseenter`, `mouseleave`, the gestures
the runtime recognises — `longpress(p)` (held half a second; its click is not
a click), `swipe(s)` (a finger or pen, `s.direction` `"left"`/`"right"`/
`"up"`/`"down"`), `receive(d)` and `dragmove(d)` (below) — and every other
standard one — `dblclick`, `contextmenu`, `pointerdown`, `pointermove`,
`touchstart`, `wheel`, `scroll`, `paste`, `drop`, `dragover`, `ended`,
`animationend`, … (`wf registry --json` lists them). The handler's
parameter names the DOM event; without one, `event` is in scope — unless
the program has an `event` of its own (a loop's item, a state, an action),
which wins, as every name a program declares does. A DOM
event sends that one value and nothing beside it: `on click(e, x)` is T22
— read what it carries from the event, `e.clientX`. Only a component's
declared event sends several (Components, Events). An
element's block is ordered `style` → `transition` → `on …` → slot fills →
children.

How often a handler runs goes in its parentheses: `on input(debounce:
300.ms) { q = typed }` runs once the events stop for that long, with the
last; `on scroll(e, throttle: 100.ms) { … }` at once, then at most once in
that long, and once more where they stopped. A wait still pending when its
page, branch or item leaves is dropped. `on visible { feed.loadMore() }`
runs each time the element comes into view.

Drag and drop: `drag: value` on any element picks it up (a mouse moved a
few pixels, a finger held still) and `on receive(d)` on another takes it:
`d.value` is what was dropped, `d.position` `"before"`, `"after"` or, with
`dropInside: true`, `"inside"`; `dropAxis: .x` for a list across the page (a
tab strip). `dropValue: v` names the target, as `d.over`; the dragged
element's `on dragmove(d)` hears where it would land as it moves (`d.over`
null when nowhere). `dragGroup: "tabs"` on both keeps drags apart. The
runtime draws a ghost and the drop line, scrolls a box near its edge, and
Escape cancels. On your component's call (`TreeRow(name, drag: path,
dropInside: true) { on receive(d) { … } }`) these, and `on longpress` and
`on swipe`, are the one element it draws, as an attribute or a DOM event's
handler written there is — unless it declares a prop or an event of that
name. A component whose body is an `if`, a `for` or several elements has no
one element for them: `C07`.

`on key("ctrl+k") { … }` answers to one key: modifiers `mod` (⌘ on a Mac,
Ctrl elsewhere — the one to use for an app's shortcuts), `ctrl`, `shift`,
`alt`, `meta`/`cmd`, then the key as the browser names it (`k`, `Enter`,
`Escape`/`esc`, `ArrowDown`/`down`, `space`, `F2`; `inline-start` and
`inline-end` are the arrow toward where a line starts or ends, ArrowLeft
and ArrowRight in a left-to-right box and the other way round in a
right-to-left one). A chord is its steps
with a space between: `on key("mod+k mod+s")`, the second within 1.5 s; the
second step alone then does not fire its own shortcut. On an element it
listens there; at the top of a page, a component or `app` it listens on the
document for as long as that shows (on `app`, always); `on key("Escape", e)`
names the event too. Escape closes the newest open `Popover` or `Menu`
first, and nothing behind it hears that key; a `Tooltip` keeps it only
while its control has focus. A page, a component or `app` may also handle
`keydown`, `keyup`, and the window's `resize`, `scroll`, `hashchange`,
`popstate`, `online`, `offline`, `message`, `storage`,
`visibilitychange`, `blur` and `focus` (the window's) itself, outside any
element. `keyIs(e, "mod+s")` is
whether a key event is a combination — for a table of commands held as
data — and `keyLabel("mod+shift+p")` how this system writes it (`⇧⌘P`,
`Ctrl+Shift+P`); a `Menu.Item`'s or `Tooltip`'s `shortcut:` is written
that way. `trapFocus: open` on any element keeps Tab inside it while
`open` holds, moving focus in and back out.

`roving: .x` (or `.y`, `.both`) on any element makes it one stop for Tab,
the way a tab strip or a toolbar is: Tab enters at the chosen item (the one
marked `aria-selected`, `aria-checked` or `aria-pressed`, else the first)
and leaves the group; inside it the arrow keys move between its items —
the other way round across a right-to-left box — Home and End go to the
ends, a letter goes to the next item whose name starts with it, a disabled
item is passed over, and a key held with Alt, Ctrl or ⌘ is left to the
browser. Its items are its children, or the first focusable thing in each:

```wf
Row(roving: .x, role: "toolbar", aria-label: "Format") {
    Button("Bold")  Button("Italic")  Button("Underline")
}
```

`Form(bind: form)` hands a handle on the form: `form.valid`, `form.errors`
by field name, `form.touched`, `form.pending` (an async rule is still
asking), `form.values` by field `name:`, `form.reset()`, `form.submit()`,
and `form.apply(serverErrors)` for what a 422 said.

