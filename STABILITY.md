# Stability: the 8.x long-term line

WebFluent 8 is a long-term line. A project that builds on 8.0 builds on
every later 8.x and means the same thing there. This file is the whole
promise: what it covers, what it does not cover, and how long it lasts.

## What does not change in 8.x

- **The language.** The syntax of `.wf` and `.wfx` files, and what a
  program means: types, reactivity, control flow, scoping, and the rules
  the checker enforces.
- **The built-ins.** Every built-in component's name, its props and their
  types, flags, enum cases, events, slots and parts. The browser values
  (`viewport`, `now`, `network`, …) and the built-in functions (`format`,
  `t`, `navigate`, `toast`, …).
- **Design tokens.** The names of the baseline theme's tokens. Their values
  may change, for example for contrast.
- **Configuration.** Every key of `webfluent.app.json` and what it does.
- **The CLI.** Every command, option and exit code. Output meant for
  tools (`--json`, `--format json|sarif|github`) keeps its fields; new
  fields may appear.
- **Diagnostics.** A code keeps its meaning. A warning does not become an
  error.
- **The public runtime.** The `WF.*` functions the guide documents, and
  the `wf:render` event.
- **The Rust and Node APIs.** `webfluent::Template`, `webfluent::memory`
  and the `webfluent` npm package follow semantic versioning. CI checks
  the Rust public API against 8.0.0.

## What a minor release may do

- **Add**: a component, a prop, a flag, a function, a config key, a command
  or an option.
- **Add a diagnostic, as a warning.** A new error is allowed only for code
  that already fails in the browser (a script error, a request that cannot
  work), and it comes with its fix through `wf fix`.
- **Fix a bug** so that the output does what the guide says it does, even
  if a program relied on the bug. Such a fix is named in the release notes.
- **Change any internal**: the parser, the checker, the code generator, the
  runtime's modules and how they are split, and the generated HTML, CSS and
  JavaScript.

## What is not covered

- **The exact bytes of the output** and its sizes. The generated markup,
  class names beginning `wf-`, the runtime's internal functions and its
  module list may change in any release. Style your own elements, not the
  built-ins' internal classes (`V04` warns about that).
- **Anything marked experimental** in the guide, and the Studio protocol
  (`wf build --studio`, `nodes.json`).
- **Diagnostic wording.** Messages and help text may improve. Match on the
  code, not the text.
- **Pre-release versions** (`8.1.0-rc.1`).

## How long 8.x lasts

- 9.0 ships no sooner than nine months after 8.0.
- 8.x gets bug fixes for at least twelve months after 9.0 ships, and
  security fixes for as long as it gets bug fixes.
- Fixes land on the newest 8.x minor; there are no parallel 8.x branches.

## How something leaves

Something is removed only in a major release. Before that it is
deprecated in an 8.x minor: it keeps working, a warning names its
replacement, and `wf migrate` rewrites it. A deprecation lasts at least
one minor release and at least three months before the next major.

8.0 itself deprecates six built-ins that were near-copies of another, so
the line freezes without them: `Dialog` (write `Modal`), `Dropdown`
(`Menu(trigger: …)`), `Tag` (`Chip`), the `Toast` element (the `toast(…)`
function), `Code(…).block` (`CodeBlock(…)`) and `CircularProgress`
(`Progress(…).circular`). Each draws as it did through every 8.x release,
is warned of as `V13`, is rewritten by `wf fix --code V13` and by `wf
migrate`'s 7 → 8 step where the rewrite is exact, and is removed in 9.0.

## Upgrading between majors

`wf migrate` takes a project from one major to the next and reports
whatever it could not decide. WebFluent 8 migrates projects written for
WebFluent 3 and later. A project in the WebFluent 2 grammar is migrated with
WebFluent 7 first (`cargo install webfluent --version ^7`, then
`wf migrate`), then with 8.
