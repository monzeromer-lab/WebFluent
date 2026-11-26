### JavaScript interop

Plain `.js` files under `src/` are linked on every page, and their top-level functions can be called from `.wf` code.

A `.js` file under `src/` is part of the project, as a `.css` file is: a
**plain browser script** — no `export`, no `import`, no build step — copied
to `build/js/` byte for byte and linked with `<script defer>` on every page,
before the compiled code. What it declares at the top level (`function`,
`class`, `const`, `let`, `var`, and anything assigned to `window.x`) is in
scope by name.

```js
// src/tilt.js
/**
 * Tilt an element toward the pointer.
 * @param {HTMLElement} node
 * @param {number} [max] degrees
 */
function initTilt(node, max = 15) {
  return { setMax(m) { max = m; }, destroy() {} };
}
```

```wf
page Gallery(path: "/gallery", title: "Gallery", description: "Cards that lean.") {
    state max = 15
    Heading("Gallery").h1
    Card(class: "tilt",
         mount: (n) => initTilt(n, max),      // each time the element is made
         update: (tilt) => tilt.setMax(max),  // again when what it reads changes
         cleanup: (tilt) => tilt.destroy()) { // when its page, branch or item leaves
        Text("Lean on me.")
    }
}
```

- **A call is checked**: the number of arguments (`T10`, defaults and
  `...rest` allowed for), and each argument's type from the JSDoc (`T01`) —
  `string`/`number`/`boolean` → `String`/`Number`/`Bool`, `T[]` → `[T]`,
  `?T`/`[name]` → optional, `(v: T) => R` → a function (an action may be
  passed), `{*}`/`{object}` → `Json`, `{Object<string, T>}` → `[String:
  T]`, `{HTMLElement}` → `Element`, a `@typedef` or a class of the
  project's → that type; `@returns` types the result.
- **The JSDoc is the signature**: a function called from `.wf` with none
  is `T31`, and its fix writes one from the call (`@param {number} max`);
  a union (`{string|number}`) or an `{Array}` that does not say what it
  holds is `T31` too. A function with JSDoc and no `@returns` gives nothing
  back.
- **What it returns is checked, in every build**: a call of a function
  whose `@returns` names a type is held to it as it returns (or settles),
  and a mismatch throws a `DecodeError` naming the function and the place —
  so a value typed `Row` is a `Row`, under `wf serve` and deployed alike.
- **`@typedef` is a type**: `/** @typedef {{ id: string, done?: boolean }}
  Row */` (or `@typedef {Object} Row` with a `@property` a field) is a
  record the program names — `state rows: [Row] = []`.
- **A class is constructed**: a call to one compiles to `new C(…)`, and is
  an instance whose members are its JSDoc'd methods (`c.next()`; another
  name is `T05`).
- **One name, one meaning**: two scripts declaring one name, or a script
  name that is also a component, store, `const`, `type` or a name the
  language or browser owns (`format`, `log`, `fetch`, `WF`, `env`), is an
  error naming both.
- **A module is refused**: a top-level `import`/`export` stops the build; a
  file the scanner cannot read is still linked, with a warning, and its names
  are not in scope; a `public/` file at the same address is an error.
- **A script reports, WebFluent owns the class**: hand the script an action
  (`mount: (n) => watchVisible(n, setVisible)`) and keep the look in
  `class: { "is-visible": visible }`.
- **`wf:render`** is dispatched on `document` once per drawing — the first
  mount, the takeover of a pre-rendered page, every route — with
  `{ route, params }`, for a script that runs its own `querySelectorAll`.

`mount:`, `update:` and `cleanup:` are on every element and component call
(a component's root, unless it declares the prop); `Host(tag: "canvas", …)`
is the same lifetime on a bare node (`div` · `span` · `canvas` · `svg` ·
`section` · `figure` · `pre` · `p` · `ul` · `table`).

