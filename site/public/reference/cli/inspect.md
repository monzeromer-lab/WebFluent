# wf inspect

> One page as a reader meets it, as text: landmarks, headings, links, buttons, fields, tables, and anything that failed

The page: https://webfluent.monzeromer.dev/reference/cli/inspect

One page as a reader meets it, as text: landmarks, headings, links, buttons, fields, tables, and anything that failed

```bash
wf inspect TARGET [options]
```

## Options

| Option | Meaning |
|---|---|
| `TARGET` | A route of the project's build (`/pokemon/pikachu`), or a URL of a server already running |
| `--dir, -d DIR` | Project directory (default: current directory) |
| `--proxy PREFIX=URL` | Forward requests under a path to another server, as `wf start` does |
| `--width WIDTH` | The window's width, in CSS pixels (default `1280`) |
| `--height HEIGHT` | The window's height, in CSS pixels (default `800`) |
| `--mobile` | A phone: 390×844, laid out as a phone lays it out |
| `--screenshot FILE` | Also save a PNG of the page (to FILE, or a file it names) |
| `--full-page` | The whole page in the screenshot, not only the window |
| `--wait MS` | The most milliseconds the page is given to settle (default `5000`) |
| `--lines LINES` | The most lines of outline it prints (default `400`) |
| `--do STEP` | A step to play before the page is read, as a test writes it: `click "Name"`, `type "pika" into "Search"`, `press "Enter"`, `expect "Saved"` (repeatable, in order) |
| `--fakes FILE` | Build the site with these fakes answering its endpoints, as `wf verify --fakes` does, and open that |
| `--json` | Print JSON, for tools |
| `--allow-output-outside` | Let `build.output` name a directory outside the project |

**Writes:** a screenshot, with `--screenshot`

Opens one page in a headless Chrome and prints it as a reader meets it, in text.

```bash
wf inspect /
wf inspect /pokemon/pikachu --mobile --screenshot
wf inspect http://localhost:3000/settings
wf inspect /tasks --do 'type "Milk" into "New task"' --do 'click "Add"'
```

```text
/ — "Dashboard" (en) at 1280×800
  link "Skip to content" → #wf-main
  navigation "Main"
    link "Dashboard" → / [current=page]
    link "Tasks" → /tasks
    button "Log in"
  main
    h1 Dashboard
    "Welcome to your dashboard."
    checkbox "Learn WebFluent"
no problems: nothing threw, logged an error or failed to arrive
```

A route is the project's build, served for the moment; a URL is a server already running (`wf serve`, `wf start`, anything). Each line is one thing a reader meets, indented under the landmark or list it is in: a heading (`h1 …`), a link and where it goes, a button or other control with its name and states (`[pressed]`, `[expanded]`, `[disabled]`, `[current=page]`), a field with its label and value, a table's caption and first rows, a list's first items, a picture's `alt` (or `(no alt)`, or `[broken]`), and text in quotes. What is hidden — `display: none`, `hidden`, `inert`, a closed dialog — is left out, as a reader never meets it.

With `--do`, the steps are played first, in order, finding what they name as a test does (a button's text, a field's label), and the outline is the page as it stands after them. A step that cannot be played — nothing called that, an `expect` whose text is not there — is the first problem, and the steps after it are not played.

Last come the problems: what threw, what was logged as an error, and what failed to arrive. It exits `0` either way: it reports, `wf verify` judges.
