# Reorder a list by dragging

> Let the reader put a list in order by dragging its rows, with buttons that do the same from the keyboard.

The page: https://webfluent.monzeromer.dev/recipes/reorder-by-dragging

To let the reader put a list in their own order, make each row something
they can pick up with `drag:`, and something another row can be dropped on
with `on receive`. This page lists today's jobs; drag a row above or below
another to move it, or use its arrow buttons:

**Needs:** nothing besides the page. Paste it into a file under
`src/pages/` and open `/order`.

```wf
type Job { id: String, title: String }

page Order(path: "/order", title: "Today's order", description: "Drag the jobs into the order you will do them.") {
    state jobs: [Job] = [
        Job(id: "brief", title: "Write the brief"),
        Job(id: "call", title: "Call the printer"),
        Job(id: "proofs", title: "Check the proofs"),
    ]

    action move(id: String, target: String, position: String) {
        let moving = jobs.find(j => j.id == id)
        if moving == null { return }
        let rest = jobs.filter(j => j.id != id)
        let at = rest.findIndex(j => j.id == target)
        let index = if position == "after" { at + 1 } else { at }
        jobs = [...rest.slice(0, index), moving, ...rest.slice(index)]
    }

    action step(id: String, offset: Number) {
        let from = jobs.findIndex(j => j.id == id)
        let to = from + offset
        if to < 0 || to >= jobs.length { return }
        move(id, jobs[to].id, if offset > 0 { "after" } else { "before" })
    }

    Heading("Today's order").h1
    Stack(gap: .sm, role: "list") {
        for job, i in jobs by job.id {
            Card(drag: job.id, dropValue: job.id, role: "listitem") {
                on receive(d) { move(d.value, job.id, d.position) }
                Row(gap: .sm, align: .center) {
                    Icon("grid")
                    Text("{i + 1}. {job.title}")
                    Spacer
                    IconButton(icon: "chevron-up", label: "Move {job.title} up", disabled: i == 0).sm {
                        on click { step(job.id, -1) }
                    }
                    IconButton(icon: "chevron-down", label: "Move {job.title} down", disabled: i == jobs.length - 1).sm {
                        on click { step(job.id, 1) }
                    }
                }
            }
        }
    }
}
```

## How it works

- `drag: job.id` makes each `Card` something the reader can pick up: with a
  mouse, by moving it a few pixels; with a finger or a pen, by holding it
  still for a moment. While it moves, the runtime draws a ghost of the row
  and a line where it would land, and Escape puts it back.
- `on receive(d)` on the same `Card` makes every row a place to drop one.
  `d.value` is the dropped row's `drag:` value (the job's `id`), and
  `d.position` is `"before"` or `"after"`: which half of this row the
  pointer was over.
- `move` takes the job out of the list and puts it back next to the row it
  was dropped on. It builds a new list rather than changing the old one in
  place, so everything that reads `jobs` draws again. The `for … by job.id`
  keeps each row's nodes and moves them, so nothing is redrawn from
  scratch.
- Dragging needs a pointer. The arrow buttons do the same move from the
  keyboard, and their labels name the job (`Move Write the brief up`), so a
  screen reader says which row each one moves. The first row's up button
  and the last row's down button are disabled.
- `role: "list"` and `role: "listitem"` tell a screen reader the cards are
  a list, and how many items it has.

The new order lasts as long as the page is open. To keep it across visits,
make the list a `persist` instead of a `state`:
`persist jobs: [Job] = [ … ]`.

## Variations

- **Drop onto a row, not between rows**: add `dropInside: true` to the
  `Card`. A drop over the row's middle then has `d.position == "inside"`,
  which is how a file tree moves a file into a folder.
- **A strip across the page**, such as tabs: give each item
  `dropAxis: .x`, so `before` and `after` are its left and right halves.
- **Two lists that must not mix**: give the rows of each its own
  `dragGroup: "todo"` and `dragGroup: "done"`. A row is dropped only on a
  row of its group.
- **Show where it will land as it moves**: `dropValue: job.id`, already on
  each row, names it. Add `on dragmove(d)` to the `Card`: the dragged row
  hears `d.over`, the `dropValue` of the row under the pointer (`null` over
  nothing), and can show it.

To check the arrow buttons, write a test that clicks one
([Test a page](https://webfluent.monzeromer.dev/recipes/test-a-page.md)):

```wf no-check
test "a job moves down" {
    Order
    expect "1. Write the brief"
    click "Move Write the brief down"
    expect "1. Call the printer"
    expect "2. Write the brief"
}
```

That test needs the page above in the same project, so it is not compiled
on its own here.
