## Key Rules

1. **Every page needs a path**: `page Name(path: "/route") { ... }`
2. **State is reactive**: any UI referencing a state variable auto-updates
3. **Flags are written with a dot, after the parentheses**: `Button("Label").primary.lg` — order doesn't matter. A flag is a Bool prop or an enum case the component declares; a word nothing declares is an error. A prop, state or derived name is an expression, so a component with a `text` prop writes `Text(text)` and means the prop
4. **One positional argument, then named ones**: `Input(bind: myVar, placeholder: "...").text` — an enum-valued prop takes a case: `Row(gap: .md, align: .center)`
5. **Braces for children/body**: `Card { Card.Body { Text("content") } }`
6. **Parts use dot, after the owner**: `Card.Header`, `Card.Body`, `Card.Footer`, `Navbar.Brand`, `Table.Row`, `Select.Option`
7. **Event handlers**: `on click { ... }` inside an element's block, `on click(e) { ... }` when the body reads the event. A `Button`'s block holds what it shows; `on click` holds what it does
8. **String interpolation is reactive**: `Text("Count: {count}")` updates when `count` changes
9. **Imports via `use`**: `use StoreName` to access shared stores
10. **No semicolons needed**: statements are newline-separated; `;` separates two on one line
11. **`return` in actions**: `return expr` returns a value from store actions
12. **Style values are CSS**: `padding: 1rem 2rem`, `background: $surface`, `width: {pct}%` — no quotes; `$name` is a design token, `{expr}` a live value; two on one line are separated by `;`, never `,` (`E002`)
13. **`@media` and nested rules inside style blocks**: `@media (max-width: 768px) { display: none }` and `&:hover { … }` compile to stylesheet rules scoped to the element
14. **Router nests anywhere**: `Router` can be inside `Row`, `Container`, `Stack`, or any layout wrapper at any depth; pages own their routes
15. **Browser globals are not prefixed**: `localStorage`, `window`, `console`, `JSON`, `Math`, `setTimeout`, etc. compile as-is, typed; one the table lacks (`crypto`, `window.fetch`) is `T30`
16. **Both `!=` and `!==`**: both inequality operators are supported (both compile to `!==` in JS); against a `null` literal, `==` and `!=` compile to JavaScript's loose `== null`, so an absent value — a query key the address lacks — counts as null
17. **Quoted map keys**: `{ "Content-Type": "application/json" }` — use for HTTP headers and hyphenated keys
18. **Reserved words as map keys, not as names**: `{ action: "approve", token: tok }` — all keywords work as map keys, fields and enum cases; a name the language has is never a program's own (`E118`): `env`, `params`, `locale`, `dir`, `now`, `viewport`, `query`, `hash`, `theme`, `network`, `update`, the built-in functions (`t`, `format`, `ago`, `log`, `navigate`, `fetch`, `uuid`, `every`, `after`, …) and the words a statement is written with (`state`, `action`, `let`, `if`, `show`, `match`, `style`, `on`, …) cannot be a state, prop, parameter, loop item, lambda parameter or `const`. `event`, `e`, `value`, `key` and the browser's globals can
19. **`public/` copies to build root**: files in `public/` land at the root of the output directory, not nested
20. **Slides need a Presentation wrapper**: `page Deck(path: "/") { Presentation { Slide { ... } } }` — slide elements outside `Presentation` are a compile error
21. **One Slide = one PDF page**: slides do not flow across pages; overflow is clipped with a note
22. **A PDF is the page, printed**: the same CSS lays it out — style a document as you would a web page, and use `break-inside: avoid`, `PageBreak` and `Header`/`Footer` for what paper needs
