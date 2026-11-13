# Hand an element to a script

> Give an element to a script or a library and take it back when the element goes, with the mount:, update: and cleanup: props, or a bare node from Host.

The page: https://webfluent.monzeromer.dev/recipes/hand-an-element-to-a-script

A library that draws into an element (a chart, a map, an editor, an
effect) needs the element when it is made, new values while it lives, and
a chance to stop when it goes. The three lifetime props give it each, on
any element or component call. This page hands a paragraph to a script
that types a greeting into it, retypes it when the name changes, and stops
its timer when the paragraph leaves:

```wf
page Welcome(path: "/welcome", title: "Welcome", description: "A greeting a script types out, letter by letter.") {
    state name = "reader"
    state typing = true

    Heading("Welcome").h1
    Input(bind: name, label: "Your name")
    Switch(bind: typing, label: "Type the greeting")
    if typing {
        Host(tag: "p",
             mount: (node) => startTypewriter(node, "Hello, {name}."),
             update: (writer) => writer.retype("Hello, {name}."),
             cleanup: (writer) => writer.stop())
    }
}
```

The script is a plain file under `src/`
([Call your own JavaScript](https://webfluent.monzeromer.dev/recipes/call-your-own-script.md)):

```js
// src/typewriter.js
/**
 * Type `text` into `node`, one letter every 60 ms.
 * @param {HTMLElement} node
 * @param {string} text
 */
function startTypewriter(node, text) {
  let shown = 0;
  const timer = setInterval(() => {
    if (shown < text.length) node.textContent = text.slice(0, ++shown);
  }, 60);
  return {
    retype(next) { text = next; shown = 0; node.textContent = ""; },
    stop() { clearInterval(timer); },
  };
}
```

## How it works

- `mount: (node) => …` runs each time the element is made: the first
  paint, a route change, an `if` branch arriving, a list item added. What
  it returns is the *handle* the other two receive. Here the handle is the
  object with `retype` and `stop`.
- `update: (handle) => …` runs again whenever the state it reads changes.
  It reads `name`, so typing in the field retypes the greeting.
- `cleanup: (handle) => …` runs when the element leaves (its page, its
  branch or its list item), so nothing a library made outlives it. Turning
  the switch off removes the branch, and `stop` clears the timer.

A `mount:` that throws removes itself and logs the error; the page stays
up.

To get a bare node for a library, use `Host`, as this page does. It has
the same lifetime: `Host(tag: "canvas", …)` makes the element the library
wants. It makes a `div` by default, or a `span`, `canvas`, `svg`,
`section`, `figure`, `pre`, `p`, `ul` or `table`.

## Variations

The three props work on every element, not only `Host`:
`Card(mount: (n) => initTilt(n))` hands the card itself to a script. On a
component call, they belong to the component's root element, unless the
component declares a prop of that name itself.

## Troubleshooting

| What you see | What to do |
|---|---|
| `mount failed` in the console, and the element is empty | The script threw on mount; the message after it is the script's own error |
| `V10` on a `Host` | Use one of the tags `Host` makes; any other is a `div` |
| A timer or listener keeps running after the page changes | Stop it in `cleanup:`; the handle `mount:` returned is what `cleanup:` receives |
