# Contributing to WebFluent

Thank you for helping. This file says how the repository is laid out, how
to run the checks quickly, and what a change needs before it is merged.

WebFluent 8 is a long-term line: read [STABILITY.md](STABILITY.md) before
changing anything a program can see. In an 8.x release a change may add,
fix and improve internals; it may not remove or rename anything, and a new
diagnostic is a warning.

## Getting started

You need a stable Rust toolchain (see `rust-version` in `Cargo.toml`),
Node.js 20 or later for the runtime and parity tests, and, for the browser
tests, Chrome or Chromium (or `WF_CHROME` pointing at one). `just` runs the
recipes below; each recipe is a short command you can also run by hand.

```bash
git clone --recurse-submodules https://github.com/monzeromer-lab/WebFluent
cd WebFluent
cargo build
cargo run -- init demo -t spa && cargo run -- build -d demo
```

## Where things are

The [Contributing](https://webfluent.monzeromer.dev/about/contributing)
page of the guide is the full map: the compiler's layers, the path a source
file takes, and the steps for adding a built-in, changing the grammar,
the runtime or a diagnostic, and writing the guide. This file is the
short version.

## The fast loop

Run only the tests your change touches while you work, then the whole
suite once.

```bash
cargo nextest run --test it paged          # one area of the integration tests
cargo nextest run --lib codegen::          # unit tests by module path
node --test tests/js/*.test.mjs             # the runtime's own tests
```

```bash
cargo nextest run --workspace              # everything (about four minutes)
```

```bash
scripts/check.sh                           # formatting, clippy, build and docs, as CI runs them
```

Don't edit sources while a cargo command runs: it rebuilds and the results
get confusing.

## When a test says a file is out of date

Several tests compare the repository against what it generates. Each says
which command writes it again; the usual ones:

| What changed | Run |
|---|---|
| Compiled output of the reference sites | `UPDATE_GOLDEN=1 cargo test --release --test it -- golden::` |
| `AGENTS.md` (the skill is generated from it) | `WF_BLESS_SKILL=1 cargo test --test it the_writing_webfluent_skill_is_true_of_this_version` |
| A command or option | `cargo test --lib commands_json` (twice) |
| The guide's examples (the editor grammar's corpus) | `WF_WRITE_GUIDE_CORPUS=1 cargo test --test it -- docs_parse::the_editor_grammar_corpus` |
| The site's data: search, config, components | `cargo build --release && cargo xtask site` |
| The public Rust API | `WF_API_TOOLCHAIN=nightly-2026-09-30 cargo xtask api --update` |

Look at the diff before you commit it: a regenerated file is part of the
review.

## What a change needs

- **A test** that fails without it. A bug fix's test names the bug's case.
- **Both sides of a twin.** Formatting, sanitizing, highlighting and
  Markdown run at build time in Rust and in the browser in JavaScript; a
  parity test holds them to the same cases. Change both.
- **The docs**, when a program can see the change: `AGENTS.md`, the guide
  chapter, and a line in `RELEASE_NOTES.md` under the next version.
- **A diagnostic with a code**, when the compiler refuses something: a
  message that names the place, a help line that says what to write, and a
  fix where there is one right answer. `wf explain CODE` reads the same
  registry (`src/diagnostics/codes.rs`).
- **Clean checks**: `cargo fmt`, clippy with `-D warnings`, and
  `cargo doc` with `-D warnings`.

Commit messages say what changed and why, in the present tense
("Keep a theme's breakpoints in the output"). One change a commit.

## Reporting a bug

Open an issue with the smallest `.wf` file that shows it, the command you
ran, what you expected, and what `wf --version` prints. A wrong diagnostic
is a bug too.

## Licence

WebFluent is under the [MPL-2.0](LICENSE). By contributing you agree that
your contribution is under the same licence.
