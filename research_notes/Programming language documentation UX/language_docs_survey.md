# Programming language documentation UX — a comparative survey (as of Nov 2026)

Scope note: pages were fetched live in Nov 2026 where possible (react.dev launch post, rust-book.cs.brown.edu, Rust error index E0382, tour.gleam.run, svelte.dev/tutorial, go.dev/doc, pkg.go.dev/strings, docs.python.org/3, typescriptlang.org/docs, elixir.hexdocs.pm, kotlinlang.org tour, ziglang.org/learn). Several fetches returned only page shells (Kotlin docs home, Swift book, Elm errors post), so claims about those sites are placed under Inferences (flagged "background knowledge, not re-verified") or Gaps. Not fetched at all: Rust by Example, rustdoc std pages, Go by Example, Tour of Go internals, Kotlin Koans, Elm guide, Astro/Vue docs, Livebook — see Gaps.

## 1. Top-level sections and the newcomer's first stop

### Takeaway
Almost every mature language site splits docs into (a) a guided learning path, (b) a reference, and (c) task/how-to material, and points newcomers at a short "tour"/"quick start" before the long book. Several (TypeScript, Zig) offer multiple entry points by the reader's background rather than one "start here".

### Cited Findings
- **React (react.dev):** Two halves — *Learn* ("a self-paced course that teaches React from scratch", chapters Describing the UI / Adding Interactivity / Managing State / Escape Hatches) and *API Reference* ("the details and usage examples for every React API"). Newcomers go to Quick Start, then the Tic-Tac-Toe tutorial if they "learn by doing", and Thinking in React for the mental model. Learn pages "only rely on concepts introduced by the earlier pages." — [react.dev launch post](https://react.dev/blog/2023/03/16/introducing-react-dev)
- **React's rationale for restructuring:** old docs made "New readers ... learn React twice" (classes then Hooks); the new docs teach Hooks from the start; old docs archived at legacy.reactjs.org with redirects "to avoid 'breaking the web'." — [react.dev launch post](https://react.dev/blog/2023/03/16/introducing-react-dev)
- **Go (go.dev/doc):** Sections: Getting Started (Installing Go → Tutorial: Getting started → A Tour of Go), Using and understanding Go (Effective Go, FAQ, diagnostics, GC guide, fuzzing, PGO…), References (package docs, command docs, Language Specification, Modules Reference, Memory Model, Release History), Accessing databases, Developing modules, Talks, Codewalks, Wiki, Non-English docs. Effective Go is called "a must read for any new Go programmer", to be read after the tour and spec. — [go.dev/doc](https://go.dev/doc/)
- **Python (docs.python.org/3):** Landing page is a grid of sections: What's new, Tutorial ("Start here"), Built-ins reference, Library reference, Language reference, Python setup and usage, HOWTOs ("In-depth topic manuals"), Installing modules, Extending and embedding, C API, FAQs, Deprecations; plus Global module index, General index, Glossary, Complete TOC. (Live site at fetch time showed 3.15 stable, 3.16 in development.) — [docs.python.org/3](https://docs.python.org/3/)
- **TypeScript:** "Get Started" offers five audience-specific entries: TS for the New Programmer, for JS Programmers ("in 5 minutes"), for Java/C# Programmers, for Functional Programmers, and Tooling in 5 minutes. Then the Handbook (The Basics, Everyday Types, Narrowing, More on Functions, Object Types, Classes, Modules; Type Manipulation subsection), Reference (Utility Types, Decorators, Enums, JSX…), Modules Reference, Tutorials, Declaration Files, JavaScript (JSDoc reference), Project Configuration (incl. the separate TSConfig Reference at /tsconfig/), downloadable Cheat Sheets, and a "Popular Documentation Pages" list. — [typescriptlang.org/docs](https://www.typescriptlang.org/docs/)
- **Zig:** Learn page groups: Documentation (two tracks: master and latest stable 0.17.0, each with Language Reference, Std Lib docs, Platform Support), Guides (only "Zig Build System", to "eventually [be] bundled with Zig's standard library documentation"), Introduction (overviews by background, incl. a comparison for C++/D/Rust programmers, Code Examples), Getting started, and links to external learning (zig.guide, Ziglings, Exercism). — [ziglang.org/learn](https://ziglang.org/learn/)
- **Kotlin:** a "Kotlin tour" with a Beginner track ("Grasp the fundamentals", starting at Hello world) and an Intermediate track (starting with extension functions); tours run "entirely in the browser, with no installation". — [Kotlin tour welcome](https://kotlinlang.org/docs/kotlin-tour-welcome.html)
- **Svelte (svelte.dev):** Top-level split Docs (/docs: Svelte, SvelteKit, CLI, AI sections) / Tutorial (/tutorial: Basic Svelte, Advanced Svelte, Basic SvelteKit, Advanced SvelteKit) / Playground (/playground, saved apps at /apps) / Packages / Blog. — [svelte.dev/tutorial](https://svelte.dev/tutorial)
- **Elixir (hexdocs):** Guides sequenced as Getting Started (Changelog → Introduction → Basic types …) with a separate "Mix and OTP guide"; search spans Elixir, EEx, ExUnit, IEx, Logger, Mix. — [elixir.hexdocs.pm/introduction](https://elixir.hexdocs.pm/introduction.html)
- **Gleam:** The Language Tour is the primary newcomer path, aimed at readers "with some programming experience", starting with basics/"Hello world". — [tour.gleam.run](https://tour.gleam.run/)

### Inferences
- The recurring tripartite model (tutorial / how-to / reference / explanation) mirrors the Diátaxis framework; Python's Tutorial/HOWTOs/Reference split is the oldest example; React and TypeScript make the Learn↔Reference split the primary nav.
- "Entry points by background" (TypeScript, Zig) is a strong fit for WebFluent, whose readers arrive as HTML/CSS authors, React/Svelte developers, or people wanting PDFs/slides.
- Background knowledge, not re-verified: The Swift Programming Language book is Welcome to Swift / A Swift Tour / Language Guide / Language Reference / Revision History, built with DocC; Rust's docs hub (doc.rust-lang.org) lists The Book, Rust by Example, Rustlings, std, the Reference, the Rustonomicon, Edition Guide, Cargo book, error index.

### Gaps
- Kotlin docs home and Swift book fetches returned only page shells; their full top-level structure was not verified.
- Astro and Vue docs were not fetched (both are known for a "Get started / Guides / Reference" sidebar and, for Vue, an Options/Composition API toggle — unverified).

## 2. Navigation patterns

### Takeaway
The dominant pattern is left sidebar + in-page TOC + prev/next + search with keyboard shortcuts (`/` or `S`), plus a light/dark/system theme switcher. Version switchers matter for languages with long-lived versions (Python, Zig, Elixir, Go package pages).

### Cited Findings
- **mdBook (Rust Book, Rust error index):** keyboard nav — ←/→ between chapters, `S` or `/` to search, `?` for shortcut help; prev/next links at page bottom; "Print this book" (whole book on one page); themes Auto, Light, Rust, Coal, Navy, Ayu. — [rust-book.cs.brown.edu](https://rust-book.cs.brown.edu/); [Rust error index E0382](https://doc.rust-lang.org/error_codes/E0382.html)
- **pkg.go.dev:** breadcrumb (Discover Packages > Standard library > strings), version badge with "Go to latest" warning, tabs Main / Versions / Licenses / Imports / Imported By, a collapsible left "Jump to" outline of every function/type/example, keyboard modal (`?` help, `/` search, `f` jump-to, `y` canonical URL), system/dark/light theme. — [pkg.go.dev/strings](https://pkg.go.dev/strings)
- **go.dev/doc:** header dropdowns (Why Go, Docs, Community), skip-to-content link, quick-links bar (Get Started, Playground, Tour, Stack Overflow, Help), theme switcher. — [go.dev/doc](https://go.dev/doc/)
- **Python:** "Docs by version" switcher from 3.16 (dev) down to 2.6 (EOL), Auto/Light/Dark theme, breadcrumb, search page, and indices (module index, general index, glossary). — [docs.python.org/3](https://docs.python.org/3/)
- **Gleam tour:** Back/Next buttons, a Contents page to jump to any lesson, and an "everything" page showing the whole tour on one page. — [tour.gleam.run](https://tour.gleam.run/)
- **Svelte tutorial:** sidebar menu listing every lesson grouped into four parts; previous/next; "Edit this page on GitHub". — [svelte.dev/tutorial](https://svelte.dev/tutorial)
- **ExDoc/hexdocs:** version label in header (v1.20.5), search with a "Search only this project" scope, prev/next, "View Source" link to the Markdown on GitHub at the tag, "Copy Markdown", "View llms.txt", ePub download, Settings. — [elixir.hexdocs.pm](https://elixir.hexdocs.pm/introduction.html)
- **Zig:** stable vs master documentation tracks side by side. — [ziglang.org/learn](https://ziglang.org/learn/)

### Inferences
- A "whole thing on one page" option (Gleam "everything", mdBook print, Zig's language reference — the last is background knowledge) is a recurring power-user feature that also aids Ctrl-F and LLM ingestion.
- ExDoc's "Copy Markdown" and llms.txt show that machine-readable docs are now a first-class nav affordance (relevant since WebFluent already ships AGENTS.md / `wf context`).

### Gaps
- Did not verify react.dev's sidebar/TOC chrome directly, nor Kotlin/Swift DocC sidebars (DocC has a filterable sidebar — background knowledge).

## 3. Interactive features

### Takeaway
The leaders converge on "learn in the browser, no install": split-pane tours with a live editor (Gleam, Svelte, Go Tour, Kotlin tour), hundreds of embedded editable sandboxes (react.dev), runnable reference examples (pkg.go.dev), and — most novel — quizzes plus misconception-driven edits (Brown's Rust Book).

### Cited Findings
- **react.dev:** "over 600" editable sandboxes with a "Fork" button; most Learn pages end with Challenges ("You don't have to solve them right now! Unless you *really* want to.") with "Show solution"; diagrams/illustrations added "when we couldn't figure out how to explain something with code and words alone". — [react.dev launch post](https://react.dev/blog/2023/03/16/introducing-react-dev)
- **Brown Rust Book experiment:** quizzes retryable until 100% (but not after viewing answers); text highlighting and annotation (stored in a cookie); a rewritten Ownership chapter with Aquascope diagrams of compile-time and run-time behaviour; "targeted edits" addressing misconceptions seen in quiz responses. Backed by research papers "Profiling Programming Language Learning" (OOPSLA 2024, Distinguished Paper) and "A Grounded Conceptual Model for Ownership Types in Rust" (OOPSLA 2023). — [rust-book.cs.brown.edu](https://rust-book.cs.brown.edu/)
- **Gleam tour:** editable code that "compiles and evaluates" as you type; output, compile errors and warnings appear in a panel; Gleam compiled to JavaScript and run "entirely within your browser window". — [tour.gleam.run](https://tour.gleam.run/)
- **Svelte tutorial:** file tree + editor (with Vim-mode toggle) + preview; "show text"/"show editor" toggles; a `solve` button fills in the answer (disabled on text-only lessons; the page recommends typing it yourself). — [svelte.dev/tutorial](https://svelte.dev/tutorial)
- **pkg.go.dev:** each example shows source plus expected "Output:" and Run / Format / Share buttons executing in the Go playground. — [pkg.go.dev/strings](https://pkg.go.dev/strings)
- **Go Tour:** interactive introduction, runnable online or locally via `go install golang.org/x/website/tour@latest`. — [go.dev/doc](https://go.dev/doc/)
- **Kotlin tour:** runs in the browser with no installation. — [Kotlin tour](https://kotlinlang.org/docs/kotlin-tour-welcome.html)
- **TypeScript:** Playground in top nav, plus Code Samples at /play/#show-examples. — [typescriptlang.org/docs](https://www.typescriptlang.org/docs/)
- **Zig:** points learners to Ziglings ("fixing broken programs") and Exercism rather than an official in-browser tour. — [ziglang.org/learn](https://ziglang.org/learn/)

### Inferences
- WebFluent compiles to HTML/CSS/JS, so it is in Gleam's position: a WASM/JS build of the compiler could power a live tour where the preview pane *is* the rendered page — stronger than text-output tours. Svelte's tutorial (file tree + editor + live preview + solve) is the closest model for a UI language.
- The Brown Rust Book shows quizzes are useful as instrumentation, not just engagement: misconception data drove edits.
- Background knowledge, not re-verified: Kotlin docs embed runnable Kotlin Playground snippets; TypeScript handbook code uses Twoslash (hover types, compiler errors inline) with "Try" links into the playground; Rust by Example and std docs have "Run" buttons to play.rust-lang.org; Elixir ships Livebook notebooks with "Run in Livebook" badges.

### Gaps
- No live verification of Kotlin Koans, Rust by Example, Go by Example (known as a static annotated two-column code/comment layout — unverified), or Livebook integration.
- No usage data found on how many readers complete tours/quizzes.

## 4. API / reference presentation

### Takeaway
Best-in-class reference pages are signature-first, then prose, then examples, with source links and a jump-to outline; react.dev additionally separates "Reference" (formal) from "Usage" (recipes) on every API page, plus Troubleshooting.

### Cited Findings
- **react.dev API pages:** one page per API, each split into *Reference* ("describes the formal API signature by listing its arguments and return values", what not how) and *Usage* ("why and how you would use this API in practice", color-coded snippets, recipes); some add *Troubleshooting* and *Alternatives*. — [react.dev launch post](https://react.dev/blog/2023/03/16/introducing-react-dev)
- **pkg.go.dev package anatomy:** header metadata (version, publish date, license, imports, importers count ~2.68M for `strings`, "Details" checks: valid go.mod, redistributable license, tagged version, stable v1); sections Overview → Index (alphabetical signatures, deprecated flagged) → Examples → Constants → Variables → Functions → Types (struct decl, constructor, methods) → Source Files. Each function: heading linking to the exact source line, "added in goX" note, full signature with linked types, prose, example. — [pkg.go.dev/strings](https://pkg.go.dev/strings)
- **pkg.go.dev history:** open-sourced June 2020 with a stated aim of a "more cohesive search and navigation experience" and godoc.org feature parity tracked in an issue. — [Go blog: pkgsite open source](https://go.dev/blog/pkgsite)
- **TypeScript TSConfig Reference** is a dedicated reference app at /tsconfig/ separate from the handbook. — [typescriptlang.org/docs](https://www.typescriptlang.org/docs/)
- **ExDoc:** "View Source" links per page at the release tag; scoped search. — [elixir.hexdocs.pm](https://elixir.hexdocs.pm/introduction.html)

### Inferences
- WebFluent's 77-component reference maps neatly onto react.dev's Reference/Usage split: a props/cases/flags/events/slots/parts table (already machine-generated by `wf registry --json`) as "Reference", then live-rendered Usage recipes. pkg.go.dev's "Jump to" outline and "added in vX" badges are cheap wins.
- A TSConfig-style dedicated config reference fits `webfluent.app.json` (`wf config schema` already has the data).
- Background knowledge, not re-verified: rustdoc puts the signature at top, a `[src]`/"Source" link, collapsible impl blocks, a search with type-signature search (`-> Vec`), and doctests that guarantee examples compile.

### Gaps
- rustdoc std pages and Swift DocC symbol pages were not fetched this session.

## 5. How compiler errors are documented

### Takeaway
Rust has the canonical model: a stable code (E0382) per error, a one-line summary, explanation, erroneous example, several fixed examples, a link to the relevant book chapter, all reachable from the terminal via `rustc --explain`. Elm's contribution is the message itself being friendly and self-sufficient. WebFluent already has codes, `wf explain`, and docs links — closest to Rust.

### Cited Findings
- **Rust error index E0382:** page heading "Error code E0382"; one-line summary ("A variable was used after its contents have been moved elsewhere"); prose explanation; an erroneous example; multiple fixes each as its own block (borrow, clone, derive Copy, Rc<RefCell>); link to Book ch. 4 "Understanding Ownership"; prev/next to E0381/E0383; mdBook search. The fetched page showed no run buttons. — [doc.rust-lang.org/error_codes/E0382](https://doc.rust-lang.org/error_codes/E0382.html)
- **Elm:** Evan Czaplicki's "Compiler Errors for Humans" (June 2015, Elm 0.15.1) targeted the complaint that compiler error messages are hard to work with; follow-ups (0.16, "Compilers as Assistants") added expected-vs-actual type diffs, more context, elimination of cascading errors, hints linking to design docs, and spelling suggestions. A Rust internals thread cites Elm's post as inspiration for Rust's current diagnostics. — [Elm: Compiler Errors for Humans](https://elm-lang.org/news/compiler-errors-for-humans); [Elm: Compilers as Assistants](https://elm-lang.org/blog/compilers-as-assistants); [Rust internals thread](https://internals.rust-lang.org/t/compiler-errors-for-humans/2310); [InfoQ on Elm syntax errors](https://www.infoq.com/news/2020/01/elm-learn-syntax-error-message)
- Elm also published "The Syntax Cliff" on syntax-error messages for learners. — [elm-lang.org/news/the-syntax-cliff](https://elm-lang.org/news/the-syntax-cliff)

### Inferences
- A WebFluent diagnostics index could beat Rust's by making the erroneous/fixed examples live (render the fixed version, show the actual compiler output for the broken one) and by linking each code back to the guide chapter and component reference page it concerns.
- The search results only showed summaries of Elm's posts; principles above are second-hand (see Gaps).

### Gaps
- The Elm post body could not be fetched; its exact principles/quotes are not verified.
- No verification of whether `rustc --explain` text is identical to the online index (widely believed; not confirmed this session).

## 6. Notable design choices praised or criticized

### Takeaway
Praised: react.dev's teach-modern-first rewrite with sandboxes and challenges; Gleam's and Svelte's zero-install split-pane tours; pkg.go.dev's runnable examples and metadata; Brown's research-driven Rust Book. Criticism found in sources was mostly about the predecessors (old React docs teaching twice; godoc.org parity gaps).

### Cited Findings
- React's redesign explicitly fixed "learn React twice" and committed to free self-paced learning. — [react.dev launch post](https://react.dev/blog/2023/03/16/introducing-react-dev)
- pkg.go.dev launched with missing godoc.org features users wanted (tracked in Go issue #39144) and promised search improvements (issue #37810). — [Go blog: pkgsite](https://go.dev/blog/pkgsite)
- Brown's Rust Book won an OOPSLA 2024 Distinguished Paper; it explicitly edits content based on observed misconceptions. — [rust-book.cs.brown.edu](https://rust-book.cs.brown.edu/)
- Svelte nudges learners away from the `solve` button, recommending typing code themselves. — [svelte.dev/tutorial](https://svelte.dev/tutorial)
- Zig admits its guides are thin ("eventually going to be bundled with Zig's standard library documentation") and leans on community resources. — [ziglang.org/learn](https://ziglang.org/learn/)

### Inferences
- Ten plausible direction archetypes for WebFluent drawn from this survey: (1) react.dev Learn/Reference split with sandboxes + challenges; (2) Gleam/Svelte split-pane tour as the home; (3) pkg.go.dev-style generated reference with Jump-to and runnable examples; (4) Python/Diátaxis quadrant landing grid; (5) TypeScript audience-specific entry points; (6) mdBook-style single long book with keyboard nav and print/one-page; (7) Rust error-index-style diagnostics hub cross-linked into guide; (8) Brown-style quizzes + annotations for retention; (9) ExDoc-style machine-readable layer (Copy Markdown, llms.txt) for agents; (10) Zig/Python version-track switcher with stable vs next.

### Gaps
- No HN/Reddit threads were read this session, so community praise/criticism beyond official sources is unverified.
- Astro, Vue, Elm guide, Rust by Example, Go by Example were not examined.
