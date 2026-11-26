### Indented syntax (`.wfx`)

The same grammar, with blocks written by indentation instead of braces. A
file named `.wfx` is read that way: a line whose next line is indented
deeper opens a block, and a dedent closes every block it leaves. Nothing
else changes — the parser, the checks, the output and the editor support
are the same, and a project may mix `.wf` and `.wfx` files.

```wfx
type Todo { id: String, title: String }

store Todos
    state items: [Todo] = []
    action add(title: String)
        items = items.concat([Todo(id: uuid(), title: title)])

component TodoRow(_ label: String, todo: Todo)
    Row
        Text(label)

page Home(path: "/", title: "Todos")
    use Todos
    state draft = ""
    Row(align: .center, gap: .sm)
        style
            padding: 6px 0
            &:hover
                background: $surface
        on click
            draft = ""
        Text("Todos").bold
    if Todos.items.length == 0
        Text("Nothing yet").muted
    else
        for todo in Todos.items by todo.id
            TodoRow(todo.title, todo: todo)
    Button("Add").primary { on click { Todos.add(draft) } }
```

- Inside parentheses, brackets, and braces you write yourself, layout is
  free, as it is in a `.wf` file: a multi-line argument list, a map
  literal, a one-line `{ on click { save() } }` all read the same.
- An empty block is written `{ }`.
- A comment line counts for nothing; blank lines count for nothing.
- Indentation must be consistent: a line that lines up with no enclosing
  block is an error.

`wf fmt --to wfx` rewrites a project's `.wf` files as `.wfx` (and `--to
wf` the other way); it is a change of layout and nothing else — the build
is byte-identical — and it refuses a file whose indentation does not
already follow its braces rather than change what it says. `wf migrate
--wfx` migrates a WebFluent 2 project straight into the indented layout.
`wf generate` writes `.wfx` in a project that has `.wfx` files and no
`.wf`.

`wf fmt` (no `--to`) formats a project's sources in place: four spaces a
level, a line that closes a block one level out, trailing blanks gone,
tabs made spaces, runs of blank lines folded to one, a brace one space
from what it opens (`Row{` → `Row {`, `}else` → `} else`). A value carried
over a line by a trailing comma is left as written; comments stay where
they are. The result is held to the file's tokens, so formatting never
changes what a file says; `wf fmt --check` writes nothing and fails when a
file would change; a `.wfx` file is normalised through its braced spelling.

