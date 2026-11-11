# Recipes

These are complete, idiomatic pieces to adapt. Together they are one small
app, a team's task tracker, and the whole file builds with no warnings. Copy
the shape, then rename for the project you're in.

## Contents

- [The model](#the-model)
- [The shell: navigation, a layout, a 404](#the-shell-navigation-a-layout-a-404)
- [A store that keeps its data](#a-store-that-keeps-its-data)
- [A list with a filter in the address and an empty state](#a-list-with-a-filter-in-the-address-and-an-empty-state)
- [A form with validation](#a-form-with-validation)
- [A form that sends to a server](#a-form-that-sends-to-a-server)
- [A service and a resource](#a-service-and-a-resource)
- [A detail page with a route parameter](#a-detail-page-with-a-route-parameter)
- [Confirm before deleting](#confirm-before-deleting)
- [Settings: theme, language, kept across visits](#settings-theme-language-kept-across-visits)
- [A dashboard that reflows](#a-dashboard-that-reflows)
- [Tabs](#tabs)
- [Keyboard shortcuts from one table](#keyboard-shortcuts-from-one-table)
- [Reorder by dragging](#reorder-by-dragging)
- [A long list](#a-long-list)
- [Tests for all of it](#tests-for-all-of-it)

## The model

```wf
/// Where a task is in its life. Drives the badge's word and colour.
enum Stage { todo, doing, done }

/// One task on the board.
type Task {
    id: Uuid
    title: String
    stage: Stage = .todo
    due: Date? = null
    owner: String = ""
}
```

## The shell: navigation, a layout, a 404

`app` holds what every page shares, and `Router` is where the page draws.
A layout component frames the pages that name it, so its chrome isn't repeated
in each one.

```wf
app {
    Navbar {
        Navbar.Brand { Link("Board", to: "/") }
        Navbar.Links {
            Link("Tasks", to: "/")
            Link("Dashboard", to: "/dashboard")
            Link("Settings", to: "/settings")
        }
    }
    Router
}

/// The frame of the board's pages: a width, and a place for the page.
component BoardShell {
    slot
    Container(class: "board") {
        children
    }
}

page NotFound(path: "*", title: "Not found", description: "There is no page at this address.", noindex: true) {
    Heading("Nothing here").h1
    Text("The page may have moved.").muted
    Link("Back to the board", to: "/")
}
```

## A store that keeps its data

The store keeps the domain data. Actions replace values rather than change
them in place, and work that can be derived is a `derived`.

```wf
/// The board's tasks, kept in this browser between visits.
store Board {
    persist tasks: [Task] = [] { version: 1 }

    derived open = tasks.filter(task => task.stage != .done)
    derived counts = {
        todo: tasks.filter(task => task.stage == .todo).length,
        doing: tasks.filter(task => task.stage == .doing).length,
        done: tasks.filter(task => task.stage == .done).length
    }

    action add(title: String, owner: String) {
        let text = title.trim()
        if text == "" { return }
        tasks = [...tasks, Task(id: uuid(), title: text, owner: owner.trim())]
    }

    action move(id: Uuid, stage: Stage) {
        tasks = tasks.map(task => if task.id == id { { ...task, stage: stage } } else { task })
    }

    action remove(id: Uuid) {
        tasks = tasks.filter(task => task.id != id)
    }

    action reorder(id: Uuid, near: Uuid, position: String) {
        let moving = tasks.find(task => task.id == id)
        if moving == null { return }
        let rest = tasks.filter(task => task.id != id)
        let at = rest.findIndex(task => task.id == near)
        let i = if position == "after" { at + 1 } else { at }
        tasks = [...rest.slice(0, i), moving, ...rest.slice(i)]
    }

    /// The task with this id, or null when it is gone.
    action find(id: String) {
        return tasks.find(task => task.id == id)
    }
}
```

## A list with a filter in the address and an empty state

The filter lives in `?stage=`, so a reload or a shared link keeps it. Every
row is keyed. The empty state tells the reader what to do next.

```wf
/// A task's stage, as a word, never colour alone. CSS colours it by
/// the `data-stage` the enum prop writes on the root.
component StageBadge(stage: Stage = .todo) {
    derived word = match stage { .todo { "To do" } .doing { "Doing" } .done { "Done" } }
    Badge(word, class: "stage-badge").pill
}

/// One task. A value down, events up: the row never changes the store.
component TaskRow(_ task: Task) {
    event advance(id: Uuid)
    event remove(id: Uuid)
    Row(class: ["task-row", if task.stage == .done { "is-done" }], align: .center, gap: .sm) {
        StageBadge(stage: task.stage)
        Link(task.title, to: "/tasks/{task.id}")
        if task.owner != "" { Text(task.owner).muted.sm }
        Spacer
        if task.stage != .done {
            Button("Advance", aria-label: "Advance {task.title}").sm { on click { emit advance(task.id) } }
        }
        IconButton(icon: "trash", label: "Remove {task.title}").sm { on click { emit remove(task.id) } }
    }
}

page Tasks(path: "/", title: "Tasks", description: "Every task on the board.", layout: BoardShell) {
    use Board
    state asking: Task? = null

    derived stage: Stage? = if query.stage == "todo" { .todo } else if query.stage == "doing" { .doing } else if query.stage == "done" { .done } else { null }
    derived shown = if let s = stage { Board.tasks.filter(task => task.stage == s) } else { Board.tasks }

    action advance(id: Uuid) {
        if let task = Board.tasks.find(x => x.id == id) {
            Board.move(id, if task.stage == .todo { .doing } else { .done })
        }
    }

    Heading("Tasks").h1
    Text(if Board.open.length == 1 { "1 open task" } else { "{Board.open.length} open tasks" }).muted
    NewTask
    Row(gap: .sm, role: "group", aria-label: "Show") {
        Link("All", to: "/")
        Link("To do", to: "/?stage=todo")
        Link("Doing", to: "/?stage=doing")
        Link("Done", to: "/?stage=done")
    }
    if shown.length == 0 {
        Text("Nothing here yet. Add the first task above.").muted
    } else {
        Stack(gap: .xs) {
            for task in shown by task.id {
                TaskRow(task, animate: .fadeIn, exit: .fadeOut) {
                    on advance(id) { advance(id) }
                    on remove(id) { asking = Board.tasks.find(x => x.id == id) }
                }
            }
        }
    }
    ConfirmRemove(task: asking) {
        on done { asking = null }
    }
}
```

A `Link` with a query matches only when the address has that value, so the
current filter is marked `aria-current` for you.

## A form with validation

`validate` sits beside the state it guards. The control bound to that state
shows the messages, with `aria-invalid` and `aria-describedby`. A submit that
fails focuses the first field that has a message.

```wf
/// The row that adds a task. Its own state; it calls the store's action.
component NewTask {
    use Board
    state title = ""
    state owner = ""

    validate title {
        required "Give the task a name"
        maxLength(120)
    }

    Form(bind: form, show: .onSubmit) {
        on submit {
            Board.add(title, owner)
            title = ""
            owner = ""
            form.reset()
        }
        Grid(columns: { base: 1, md: 3 }, gap: .md, align: .end) {
            Input(bind: title, label: "Task").text
            Input(bind: owner, label: "Owner", hint: "Optional").text
            Button("Add task", type: .submit).primary
        }
    }
}
```

The available rules are `required`, `email`, `url`, `minLength(n)`,
`maxLength(n)`, `min(v)`, `max(v)`, `pattern(/…/)`, `matches(other)`,
`oneOf([…])`, `custom "…" { expr }` and `async "…" { await … }`. A refined
type (`Number(1..=30)`) validates itself.

## A form that sends to a server

Describe the endpoint with `api`, type the error body, and hand the 422's
messages back to the form. `join.pending` is true while the action runs.

```wf
api Accounts(base: "/api") {
    timeout: 10.seconds
    credentials: .sameOrigin
    post invite(body: Map) -> Map
        errors { 422 -> Map }
}

page Invite(path: "/invite", title: "Invite", description: "Ask someone to join the board.", layout: BoardShell) {
    state email = ""
    state sent = false

    validate email { required  email }

    action invite() {
        try {
            await Accounts.invite(body: { email: email })
            sent = true
        } catch e {
            form.apply(e.body.errors)
        }
    }

    Heading("Invite someone").h1
    if sent { Alert("Invitation sent.").success }
    Form(bind: form) {
        on submit { invite() }
        Input(bind: email, name: "email", label: "Email").email
        Button("Send invitation", type: .submit, disabled: !form.valid || invite.pending).primary
    }
}
```

## A service and a resource

A `resource` over an endpoint is typed, cached, retried and cancelled for you.
Match every arm.
- A refresh keeps the data on screen and sets `.refreshing`.
- A failed refresh keeps the data and sets `.error`.

```wf
/// One release, as the server sends it.
type Release { tag: String, title: String, published: DateTime }

api Releases(base: "/api") {
    timeout: 10.seconds
    retry: .backoff(times: 3, on: [.network, .timeout, .status5xx])
    get list(page: Number = 1) -> [Release]
}

page Changelog(path: "/changelog", title: "Changelog", description: "What changed, newest first.", layout: BoardShell) {
    resource releases = Releases.list(cache: .swr(60.seconds), on: .focus)

    Heading("Changelog").h1
    match releases {
        loading { Skeleton(height: "20px") }
        error(e) {
            match e {
                .offline { Alert("You are offline.").warning }
                .timeout { Alert("The server took too long.").warning }
                else { Alert(e.message).danger }
            }
            Button("Try again") { on click { releases.reload() } }
        }
        ready(list) {
            if releases.error != null { Alert("Could not refresh; this is what was there.").warning }
            if releases.refreshing { Spinner.sm }
            if list.length == 0 { Text("Nothing released yet.").muted }
            for r in list by r.tag {
                Row(gap: .sm, align: .baseline) {
                    Text(r.tag).bold
                    Text(r.title)
                    Text(ago(r.published)).muted.sm
                }
            }
        }
    }
}
```

## A detail page with a route parameter

The parameter is a typed parameter of the page. The record may be gone, so
`if let` narrows it.

```wf
page TaskDetail(path: "/tasks/:id", title: "Task", description: "One task and its history.", id: String, layout: BoardShell) {
    use Board
    derived task = Board.tasks.find(x => x.id == id)

    if let task = task {
        Breadcrumb {
            Breadcrumb.Item(to: "/") { Text("Tasks") }
            Breadcrumb.Item { Text(task.title) }
        }
        Heading(task.title).h1
        StageBadge(stage: task.stage)
        if let due = task.due {
            Text("Due {due:.date(long)}")
        }
    } else {
        Heading("No such task").h1
        Link("Back to the tasks", to: "/")
    }
}
```

A static build renders a `:param` route for the values its page names in
`paths:` (`paths: posts.map(p => p.slug)`), each listed in the sitemap.

## Confirm before deleting

A dialog opened by state. Escape and the close button both run `on close`, and
the reader can always tell which row is going.

```wf
/// Asks before a task is removed. Open while `task` is not null.
component ConfirmRemove(task: Task? = null) {
    use Board
    event done()
    Modal(visible: task != null, title: "Remove this task?") {
        on close { emit done() }
        if let task = task {
            Text("“{task.title}” will be removed from the board.")
        }
        Modal.Footer {
            Button("Keep it") { on click { emit done() } }
            Button("Remove").danger {
                on click {
                    if let task = task { Board.remove(task.id) }
                    emit done()
                }
            }
        }
    }
}
```

## Settings: theme, language, kept across visits

`persist` keeps the choice, and an `effect` hands it to the world outside.
`setTheme("system")` follows the operating system.

```wf
/// What the reader chose, kept across visits.
store Prefs {
    persist appearance = "system"
    persist compact = false
}

page Settings(path: "/settings", title: "Settings", description: "How the board looks.", layout: BoardShell) {
    use Prefs
    effect { setTheme(Prefs.appearance) }

    Heading("Settings").h1
    Select(bind: Prefs.appearance, label: "Appearance") {
        Select.Option("Same as the system", value: "system")
        Select.Option("Light", value: "light")
        Select.Option("Dark", value: "dark")
    }
    Switch(bind: Prefs.compact, label: "Compact rows")
}
```

In a translated project, add `persist language = "en"` and
`effect { setLocale(Prefs.language) }`. Arabic then flips the page right to
left.

## A dashboard that reflows

Responsive values compile to CSS, so the first paint has the right layout. A
number counts to its new value with `count:`.

```wf
/// A figure and what it counts.
component Figure(_ label: String, value: Number, stage: Stage = .todo) {
    Card(class: "figure") {
        Text(label).muted.sm
        Text("{value}", count: "600ms").bold.lg
        StageBadge(stage: stage)
    }
}

page Dashboard(path: "/dashboard", title: "Dashboard", description: "The board at a glance.", layout: BoardShell) {
    use Board
    Heading("Dashboard").h1
    Grid(columns: { base: 1, sm: 3 }, gap: { base: .sm, lg: .lg }) {
        Figure("To do", value: Board.counts.todo)
        Figure("Doing", value: Board.counts.doing, stage: .doing)
        Figure("Done", value: Board.counts.done, stage: .done)
    }
}
```

## Tabs

```wf
page About(path: "/about", title: "About", description: "How the board works.", layout: BoardShell) {
    Heading("About the board").h1
    Tabs {
        Tabs.Page("Stages") { Text("A task goes from to do, to doing, to done.") }
        Tabs.Page("Keys") { Text("Press N for a new task.") }
    }
}
```

## Keyboard shortcuts from one table

Keep what the app can do in one table. The keys, the help and the menu all
read it, so they can't disagree. `keyIs` matches by key position, so `mod+k`
works on any layout. `keyLabel` writes the keys as this system shows them
(`⌘K` on a Mac).

```wf
/// Something the app can do, and the keys that do it.
type Shortcut { id: String, label: String, keys: String }

const SHORTCUTS: [Shortcut] = [
    Shortcut(id: "go.tasks", label: "Go to the tasks", keys: "mod+1"),
    Shortcut(id: "go.dashboard", label: "Go to the dashboard", keys: "mod+2")
]

/// Runs a shortcut by its id.
store Keys {
    action run(id: String) {
        if id == "go.tasks" { navigate("/") }
        if id == "go.dashboard" { navigate("/dashboard") }
    }

    action pressed(e: Any) {
        if let hit = SHORTCUTS.find(s => keyIs(e, s.keys)) {
            e.preventDefault()
            run(hit.id)
        }
    }
}

page Help(path: "/help", title: "Help", description: "Every keyboard shortcut.", layout: BoardShell) {
    use Keys
    on keydown(e) { Keys.pressed(e) }

    Heading("Keyboard shortcuts").h1
    Table(caption: "Keyboard shortcuts") {
        Table.Head { Table.Row { Table.Cell("Action")  Table.Cell("Keys") } }
        Table.Body {
            for s in SHORTCUTS by s.id {
                Table.Row { Table.Cell(s.label)  Table.Cell(keyLabel(s.keys)) }
            }
        }
    }
}
```

For a single shortcut, use `on key("mod+k") { … }` at the top of a page or
`app`. On an element, it listens only while focus is inside.

## Reorder by dragging

`drag:` picks a row up and `on receive(d)` takes the drop, with `d.position`
`"before"` or `"after"`. Keyboard users need another way, here the menu of
each row.

```wf
page Order(path: "/order", title: "Order", description: "Put the tasks in order.", layout: BoardShell) {
    use Board
    Heading("Put the tasks in order").h1
    Stack(gap: .xs) {
        for task in Board.tasks by task.id {
            Card(drag: task.id, dragGroup: "tasks", dropValue: task.id) {
                on receive(d) { Board.reorder(d.value, task.id, d.position) }
                Text(task.title)
            }
        }
    }
}
```

## A long list

A `VirtualList` draws only the rows in view. Its body is one keyed `for`, and
it needs a height its rows don't set.

```wf
page Archive(path: "/archive", title: "Archive", description: "Every task ever done.", layout: BoardShell) {
    use Board
    Heading("Archive").h1
    VirtualList(rowHeight: 32, label: "Done tasks") {
        style { height: 480px }
        for task in Board.tasks.filter(x => x.stage == .done) by task.id {
            Text(task.title)
        }
    }
}
```

## Tests for all of it

Tests say what the reader sees, and find things by the names a reader sees.
See [testing.md](testing.md).

```wf
test "a blank task is not added" {
    Tasks
    click "Add task"
    expect "Give the task a name"
    expect "Nothing here yet"
}

test "an added task is listed and counted" {
    Tasks
    type "Write the release notes" into "Task"
    click "Add task"
    expect "Write the release notes"
    expect "1 open task"
}

test "a done task offers no advance"(data: { task: { id: "1", title: "Ship it", stage: "done" } }) {
    TaskRow(task)
    expect "Done"
    expect not "Advance"
}
```
