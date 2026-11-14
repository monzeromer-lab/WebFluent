---
kind: "recipe"
route: "/recipes/call-your-own-script"
group: "interop"
description: "Call a function of your own JavaScript from a WebFluent page: a plain script under src/, typed with JSDoc, linked on every page, its names in scope."
outputs: ["website", "app", "desktop"]
from: ["/docs/guide/interop#before-you-start", "/docs/guide/interop#call-your-own-script"]
---
# Call your own JavaScript

Some things are easier to write in JavaScript: a pointer effect, a
measurement, glue around a library. Put the function in a `.js` file under
`src/`, and your pages call it by name. You need a WebFluent project
([Getting started](../learn/01-getting-started.md)); the script is a plain
browser script, with no `import`, no `export` and no build step.

**Needs:** the script `src/tilt.js`, shown below.

```wf
page Gallery(path: "/gallery", title: "Gallery", description: "Cards that lean.") {
    state max = 15

    Heading("Gallery").h1
    Card(class: "tilt",
         mount: (n) => initTilt(n, max),
         update: (tilt) => tilt.setMax(max),
         cleanup: (tilt) => tilt.destroy()) {
        Text("Move the pointer over me.")
    }
    Slider(bind: max, min: 0, max: 45, label: "How far")
}
```

This script tilts an element toward the pointer, and returns a handle the
page can update and destroy. Write it the way you would write any file a
`<script src>` loads:

```js
// src/tilt.js
/**
 * Tilt an element toward the pointer.
 * @param {HTMLElement} node
 * @param {number} [max] how far, in degrees
 */
function initTilt(node, max = 15) {
  const move = (e) => {
    const r = node.getBoundingClientRect();
    const x = (e.clientX - r.left) / r.width - 0.5;
    node.style.transform = `rotateY(${x * max}deg)`;
  };
  node.addEventListener("pointermove", move);
  return {
    setMax(m) { max = m; },
    destroy() { node.removeEventListener("pointermove", move); },
  };
}
```

## How it works

The page hands the card to `initTilt` with `mount:`, changes how far it
tilts with `update:` as the reader moves the slider, and removes the
listener with `cleanup:` when the page leaves
([Hand an element to a script](hand-an-element-to-a-script.md)). The build
treats the script this way:

- **The file runs as written.** The build copies it to `build/js/tilt.js`
  byte for byte and links it with `<script defer>` on every page, before the
  compiled code. A stack trace points at your own line.
- **Its names are in scope.** Every top-level `function`, `class`, `const`,
  `let` and `var`, and anything assigned to `window.x`, is a name your `.wf`
  code can call directly. What a function or an IIFE keeps inside stays
  private, as it does in the browser.
- **A call is checked.** `initTilt(n, 1, 2)` is a `T10` error where it is
  written, because the function takes one or two arguments. Its JSDoc types
  the arguments ([Type a script with JSDoc](#type-a-script-with-jsdoc)). A
  name the file does not declare is still a `T13`.
- **A class is constructed.** WebFluent has no `new`. A call to a class a
  script declares compiles to `new`.
- **One name has one meaning.** Two scripts that declare the same name, or
  a script name that is also a component, a store, a `const`, or a name the
  language or the browser owns (`format`, `log`, `fetch`), is an error that
  names both.
- **An ES module is refused.** A file with a top-level `import` or `export`
  cannot run from a `<script src>`, so the build stops and says so. Load a
  published module from the config instead
  ([Load a library from a CDN](library-from-a-cdn.md)).

The tools know your scripts too: `wf serve` rebuilds when a script changes,
`wf audit` lists what each one declares, `wf types` prints each function's
signature, and the editor completes, documents and jumps to them.

## Type a script with JSDoc

To have calls checked against a function's types, describe the function in
a JSDoc comment. JSDoc is a comment, so the file stays exactly what the
browser runs. The checker reads it this way:

| JSDoc | WebFluent |
|---|---|
| `string` · `number` · `boolean` | `String` · `Number` · `Bool` |
| `object` · `{ a: T }` | `Map`, or a shape of the fields written |
| `T[]` · `Array<T>` | `[T]` |
| `?T` · `T \| null` · `[name]` | `T?`, or a parameter a call may leave out |
| `Promise<T>` | `T`, as an action that awaits it reads it |
| `(v: T) => R` | a function: you may pass an action or a lambda |
| `HTMLElement`, anything else | `Any` |

With no comment at all, every parameter is `Any`, so adding a script never
breaks a build. Each tag only narrows what a call may pass. `@returns` types
the result.

## Let a script report, and keep the class in WebFluent

When the way an element looks depends on something only a script can see
(an intersection, a scroll position, a library's event), hand the script an
action. An action is a plain function at run time. The script calls it, and
the class stays in a `class:` the compiler can read.

This page sets `is-visible` on the card while it is on screen:

```wf
page Story(path: "/story", title: "Story", description: "It fades in.") {
    state visible = false
    action setVisible(v: Bool) { visible = v }

    Heading("Story").h1
    Card(class: { "is-visible": visible },
         mount: (n) => watchVisible(n, setVisible),
         cleanup: (w) => w.destroy()) {
        Text("Here I am.")
    }
}
```

This script reports whether an element is on screen:

```js
// src/visible.js
/**
 * Say whether `node` is on screen, each time that changes.
 * @param {HTMLElement} node
 * @param {(v: boolean) => void} onChange
 */
function watchVisible(node, onChange) {
  const io = new IntersectionObserver(([e]) => onChange(e.isIntersecting));
  io.observe(node);
  return { destroy: () => io.disconnect() };
}
```

Each class has one owner. A class that `class:` names belongs to
WebFluent, and a class a script adds with `classList.add` belongs to the
script: `class:` only ever takes off what it put on.

## Troubleshooting

| What you see | What to do |
|---|---|
| `T13`: nothing declares the function | Check that the file is under `src/` and declares the name at its top level, not inside another function |
| `T10` on a call that looks right | Count the parameters the function declares; a JSDoc `[name]` or a default value lets a call leave one out |
| The build stops on a top-level `import` or `export` | Remove them and declare the names at the top level; a published module loads from `meta.scripts` instead ([Load a library from a CDN](library-from-a-cdn.md)) |
| An error that names your script and a component, store or `const` | Rename one of the two: a name has one meaning in a project |
