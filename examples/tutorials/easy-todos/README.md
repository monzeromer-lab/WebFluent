# Todos — the easy tutorial

A list of things to do, kept in the reader's browser. One page, one store,
one component; no server.

It is the finished project of the first tutorial in the guide's
[Tutorials chapter](../../../md-docs/35-tutorials.md#easy-todos), which
builds it step by step.

What it teaches:

- a `type` and an `enum` (`src/types.wf`)
- a store whose list survives a reload with `persist` (`src/stores/Todos.wf`)
- a component with state of its own, rename in place (`src/components/TodoRow.wf`)
- a keyed `for … by todo.id`, `derived` values, and a filter in the address (`/?show=open`)
- `on key("Enter")`, `mount:` to focus an input, a class map
- a theme with a dark variant, and the project's own stylesheet
- tests that look and tests that click (`tests/todos.wf`)

```bash
wf serve      # http://localhost:3000, rebuilt on save
wf test       # six tests; four click, in a headless Chrome
wf build      # build/, ready for any static host
```
