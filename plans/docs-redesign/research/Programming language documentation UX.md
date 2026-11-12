# Verified examples beat clever docs-site chrome

The strongest programming-language documentation is not distinguished by its visual design. It is distinguished by three structural decisions: it **separates the reader's modes** (learning, doing, looking up, understanding), it **makes every example real**, meaning complete, runnable and checked by the toolchain, and it **serves two kinds of reader at once**, the opportunistic forager who arrives from search with a task and the systematic reader who wants one linear path. The empirical record agrees. Developers spend about half of a task inside documentation, and most of that time goes to reference and recipe pages rather than concept chapters ([Meng et al. 2019](https://sigdoc.acm.org/cdq/how-developers-use-api-documentation-an-observation-study/)). The failures they rate most severe are content failures (ambiguity, incompleteness, incorrectness), not presentation ([Uddin & Robillard 2015](https://www.cs.mcgill.ca/~martin/papers/ieeesw2015.pdf)). Learners of a new language carry wrong assumptions from the old one in 61% of sampled cross-language questions ([Shrestha et al. 2020](https://par.nsf.gov//servlets/purl/10139116)). The leading sites (react.dev, Go, Python, TypeScript, Rust, Gleam, Svelte) converge on a Learn / Reference split, a zero-install interactive on-ramp and a three-zone page shell. For WebFluent this is unusually good news: the language already owns the hard parts most projects lack. Those are a machine-readable component registry, structured diagnostics with codes and `wf explain`, a test runner that asserts visible text, and a compiler that targets the browser. The redesign question is therefore less "which framework" and more "which of these assets becomes the spine of the site." The ten directions at the end of this report are ten different answers to that question.

## Diátaxis decides a page's job, not the site's skeleton

The dominant framework is Diátaxis. It sorts content along two axes: action against cognition, and acquisition (study) against application (work). That gives four modes: tutorials (doing, while learning), how-to guides (doing, while working), reference (knowing, while working) and explanation (knowing, while learning) ([Diátaxis compass](https://diataxis.fr/compass/)). It descends from the Divio "documentation system", which made the same claim that documentation "isn't one thing, it's four" ([Divio](https://docs.divio.com/documentation-system/)). Each mode has its own writing rules. **A tutorial teaches by doing, delivers visible results early and keeps a "narrative of the expected"**: sample output, what to notice, ruthless omission of options ([Tutorials](https://diataxis.fr/tutorials/)). A how-to guide addresses one real goal for a competent reader, titled "How to…", with practical usefulness ahead of completeness ([How-to guides](https://diataxis.fr/how-to-guides/)). **Reference must "describe and only describe"**, in standard patterns whose structure mirrors the product's ([Reference](https://diataxis.fr/reference/)). Explanation is bounded discussion of design decisions and history, ideally titled "About X" ([Explanation](https://diataxis.fr/explanation/)).

Two points matter most for a redesign. First, the most common failure is mixing tutorials with how-to guides. A tutorial is a safe path the teacher controls; a how-to is real work with real branches ([Tutorials and how-to guides](https://diataxis.fr/tutorials-how-to/)). The second most common is letting "why" digressions leak into reference ([Reference and explanation](https://diataxis.fr/reference-explanation/)). Second, Diátaxis's authors explicitly warn against starting by creating four empty top-level boxes, which they call "horrible". The framework "changes the structure of your documentation from the inside", one page at a time ([Workflow](https://diataxis.fr/how-to-use-diataxis/)). Canonical, which adopted it across its product docs in 2021, reports that the docs look worse before they improve, because mixed modes become visible ([Canonical](https://canonical.com/blog/diataxis-a-new-foundation-for-canonical-documentation)). Django's index already uses the four-way split under its own names: tutorials, topic guides, reference, how-to ([Django docs](https://docs.djangoproject.com/en/stable/)). One caveat: no empirical evaluation of Diátaxis itself was found. Its authority comes from adoption, not from measured outcomes.

Diátaxis decides what *kind* of page something is. Mark Baker's **Every Page is Page One** decides how that page must behave: self-contained, of one specific purpose and level, establishing its own context, and richly linked, because readers arrive from search rather than from a table of contents ([EPPO](https://everypageispageone.com/the-book/)). DITA adds the two types a language needs and Diátaxis leaves generic: **troubleshooting** (corrective action) and **glossary entries** ([OASIS DITA 1.3](https://docs.oasis-open.org/dita/dita/v1.3/os/part2-tech-content/archSpec/technicalContent/dita-technicalContent-InformationTypes.html)). Carroll's minimalism adds action-first wording and explicit support for error recognition and recovery ([instructionaldesign.org](https://www.instructionaldesign.org/theories/minimalism/)). Nielsen Norman Group's progressive disclosure adds a layering rule: defer the rare to a clearly labelled second layer, and stop at two levels ([NN/g](https://www.nngroup.com/articles/progressive-disclosure/)). Write the Docs supplies the operating principles. Its "Current" principle states that **"incorrect documentation [is] worse than missing documentation"**. Its "Addressable", "Skimmable" and "Cumulative" principles ask for deep links, front-loaded headings and prerequisites before what depends on them ([Write the Docs](https://www.writethedocs.org/guide/writing/docs-principles/)).

A language needs more document types than a library. The mature sites show a recognisable set: a book-length tutorial and a short tour; a language reference; a standard-library or API reference; a by-example collection; an error-code index; What's New and migration guides; a glossary; "coming from X" on-ramps; and an idioms guide. Rust groups these by reader stage (Learning, Using, Mastering) ([doc.rust-lang.org](https://doc.rust-lang.org/)). Python lists What's New, Tutorial, three references, HOWTOs, FAQs, Deprecations and a Glossary ([docs.python.org](https://docs.python.org/3/)). Mapped onto WebFluent, the evidence points to a tutorial (one guided build from `wf init` to `wf build`); recipes seeded by the "Ten idioms"; generated references for components (`wf registry --json`), CLI (`wf commands --json`), config (`wf config schema --json`) and the `WF.*` runtime; a per-code diagnostics index; "About" pages on reactivity, store lifetime, the security model, hydration and "a PDF is the page, printed"; migration guides from 2 through 5; a glossary; and on-ramps for React, Vue and Svelte users, HTML/CSS authors and LaTeX/Typst users.

## Ten language sites converge on learn, look up and run

Across the surveyed sites, the same skeleton recurs. There is a guided learning path, a reference, and task material, with a short tour or quick start in front of the long book.

| Site | Top-level shape | Newcomer's first stop | Distinctive move |
|---|---|---|---|
| react.dev | Learn / API Reference | Quick Start, then a Tic-Tac-Toe tutorial, then "Thinking in React" | Over 600 editable sandboxes; challenges with "Show solution"; each API page split into Reference and Usage ([React](https://react.dev/blog/2023/03/16/introducing-react-dev)) |
| Go | Getting Started / Using and understanding / References | Install, short tutorial, A Tour of Go | pkg.go.dev runnable examples with "Output:", Run/Format/Share, and a "Jump to" outline ([go.dev/doc](https://go.dev/doc/); [pkg.go.dev](https://pkg.go.dev/strings)) |
| Python | Grid: What's New, Tutorial, three references, HOWTOs, FAQs, Glossary | Tutorial ("Start here") | Version switcher from 3.16 dev back to 2.6, general and module indexes ([docs.python.org](https://docs.python.org/3/)) |
| TypeScript | Get Started / Handbook / Reference / TSConfig app | Five on-ramps by background (new programmer, JS, Java/C#, functional, tooling) | A separate reference app for configuration ([TS docs](https://www.typescriptlang.org/docs/)) |
| Rust | Learning / Using / Mastering / Specialized | The Book, Rust By Example, Rustlings | Error index with one page per code, reachable through `rustc --explain` ([Rust docs](https://doc.rust-lang.org/); [E0382](https://doc.rust-lang.org/error_codes/E0382.html)) |
| Gleam | Language Tour as primary path | tour.gleam.run | Compiler runs in the browser; output and compile errors appear as you type; an "everything" one-page view ([Gleam tour](https://tour.gleam.run/)) |
| Svelte | Docs / Tutorial / Playground | Tutorial in four parts | File tree, editor and live preview, with a `solve` button it asks you not to use ([Svelte tutorial](https://svelte.dev/tutorial)) |
| Kotlin | Tour with Beginner and Intermediate tracks | Hello world in the browser | Runs "entirely in the browser, with no installation" ([Kotlin tour](https://kotlinlang.org/docs/kotlin-tour-welcome.html)) |
| Zig | Docs in two tracks (master, stable) / Guides / Introduction | Overviews by background | Stable and next side by side; honest about thin guides ([Zig learn](https://ziglang.org/learn/)) |
| Elixir (ExDoc) | Getting Started guides + per-library reference | Introduction | Version label, scoped search, "View Source" at the tag, "Copy Markdown", llms.txt, ePub ([hexdocs](https://elixir.hexdocs.pm/introduction.html)) |

The survey could not verify every site. Swift's book and Kotlin's docs home returned only page shells, so their structures are not asserted here. Elm's contribution is to error messages rather than site structure: "Compiler Errors for Humans" (2015) and "Compilers as Assistants" pushed for messages that are self-sufficient, show expected and actual types, and suggest spellings. A Rust internals thread cites it as inspiration for Rust's diagnostics ([Elm](https://elm-lang.org/news/compiler-errors-for-humans); [Elm 0.16](https://elm-lang.org/blog/compilers-as-assistants); [Rust internals](https://internals.rust-lang.org/t/compiler-errors-for-humans/2310)).

Three moves stand out as most transferable. The first is **react.dev's rewrite**, which fixed old docs that made "new readers… learn React twice" by teaching the modern model first. It archived the old site with redirects "to avoid 'breaking the web'", and ordered Learn pages so each relies "only on concepts introduced by the earlier pages" ([React](https://react.dev/blog/2023/03/16/introducing-react-dev)). The second is **Rust's error page shape**: a one-line meaning, an erroneous example, several fixes each with a rationale, and a link to the Book chapter on the governing concept ([E0382](https://doc.rust-lang.org/error_codes/E0382.html)). WebFluent's diagnostics already have codes, hints, fixes and `docs:` URLs, which is closer to Rust than almost any young language gets. The third is **Gleam's and Svelte's zero-install tours**, where a compiler built for JavaScript runs in the page. WebFluent is in exactly that position, with one advantage over both: its output pane is not console text but the rendered page itself.

The most research-backed site in the survey is Brown University's interactive Rust Book. It adds retryable quizzes, highlighting and annotation, and Aquascope diagrams. It also rewrites hard spots based on where quiz answers show misconceptions, work that earned an OOPSLA 2024 Distinguished Paper ([Brown Rust Book](https://rust-book.cs.brown.edu/)).

## Developers forage for examples, and half of them skip the prose

The best observational study of documentation use watched 11 developers through five API tasks with eye-tracking. **They spent about 49% of their time in the docs.** By page type, the shares were **API reference 18.35%, recipes 14.99%, concepts 7.91%, samples 5.69% and the welcome page 1.33%** ([Meng et al. 2019](https://sigdoc.acm.org/cdq/how-developers-use-api-documentation-an-observation-study/)). Concept-page use was bimodal: five participants barely touched concepts, and six spent 12.5–43% of their time there. The authors identified opportunistic readers (task-driven, risk-taking) and systematic ones (understand first). Two of the four systematic developers failed to finish in 70 minutes. Their recommendations are blunt. Organise by task, not content type. Weave concepts into the tasks that need them. Make code copy-ready with no placeholders pointing elsewhere. **Repeat critical concepts as code comments, because opportunistic readers skip prose.** Enable fast try-outs. The sample is small, so the percentages are indicative, but the direction matches a larger field study.

That larger study, Robillard and DeLine's survey of more than 440 Microsoft developers, found learning resources were the largest obstacle category, named by 50 of 80 respondents in its first phase ([Robillard & DeLine 2011](https://www.cs.mcgill.ca/~martin/papers/ese2011.pdf)). Its implications sharpen the example question. **Small examples that show a pattern of several calls beat single-call examples, and long examples lose value as concerns tangle.** Readers treat examples as the recommended way. Navigating to a page is an implicit query, so boilerplate wastes time. And some developers want a coherent linear overview, because fragmented collections of hyperlinked articles overwhelm them. That last point is the counterweight to EPPO: a site of standalone pages still needs a spine.

Reading behaviour on the page reinforces front-loading. NN/g's eye-tracking found F-shaped scanning in 2006 (232 users) and again in 2017 on desktop and mobile. **The pattern mirrors in right-to-left languages**, which matters for a docs site WebFluent should ship in Arabic. The remedy is meaning-bearing headings and the important content in the first lines ([NN/g 2017](https://nngroup.com/articles/f-shaped-pattern-reading-web-content/)). Code itself is not read linearly by experts: novices read code about 70% linearly, experts about 60% ([Busjahn et al. 2015](https://huang.isis.vanderbilt.edu/cs8395-f23/paper/linear-order-eye.pdf)). Annotations that link tokens to reference therefore suit how experts actually move through code.

Learning a new language adds a specific hazard. In 450 Stack Overflow posts across 18 languages, **61% contained incorrect assumptions carried over from a previous language**. Professionals learned new languages opportunistically, mapped to what they knew, and asked for cheat sheets: "If you can compare them side by side … you're more than halfway there." They also named a vocabulary barrier: they could not search for terms they did not yet know ([Shrestha et al. 2020](https://par.nsf.gov//servlets/purl/10139116)). For WebFluent, whose readers arrive from React, Vue, Svelte or plain HTML, the predictable interference is already catalogued in its own "mistakes made most": `onClick=` attributes (E106), commas between statements (E002), flags before the closing parenthesis.

The Rust Book telemetry shows where a linear guide loses people. In 62,526 readers and 1.14 million answers over 13 months, most casual readers stopped in chapter 1. The committed "triers" mostly stopped in chapters 3–4, at ownership, and **only 2% reached chapter 19**. Twelve targeted rewrites of hard spots produced significant gains in ten cases, **averaging +20% on the targeted questions**. The authoring cost was about 50 person-hours and the hosting about $15 a month ([Crichton & Krishnamurthi 2024](https://arxiv.org/html/2401.01257v1)). The lesson for a 50-chapter guide is that the first three chapters carry most of the audience, and the hardest concept (for WebFluent, probably reactivity and stores, or the registry and flags model) deserves measured, iterated treatment.

Surveys agree on priorities. Technical documentation was the top learning resource in Stack Overflow's 2024 survey, at 83.9% ([SO 2024](https://survey.stackoverflow.co/2024/developer-profile/)). In 2025, **46% of developers distrusted AI accuracy against 33% who trusted it, and 66% found AI answers "almost right, but not quite"** ([TechRepublic](https://www.techrepublic.com/article/news-developer-trust-in-ai-declines/)). That raises the value of an authoritative, verified source. Practitioners rank ambiguity, incompleteness and incorrectness as the severest documentation problems. Six problems were "blockers" that made them switch APIs, among them obsoleteness and **unexplained examples**. No respondent chose tangled information or excess structure as a top problem ([Uddin & Robillard 2015](https://www.cs.mcgill.ca/~martin/papers/ieeesw2015.pdf)). In the 2017 GitHub open-source survey, 93% reported incomplete or outdated docs ([GitHub 2017](https://opensourcesurvey.org/2017/)). Evidence on enhanced compiler messages is mixed. Messages written to readability guidelines did shorten novices' debugging ([Denny et al. 2020](https://scispace.com/papers/error-message-readability-and-novice-debugging-performance-742o5sde3a)), which argues for applying the same readability discipline to the diagnostics index entries.

## The page shell is settled, and the differentiators sit inside it

The standard shell has three zones: a section sidebar on the left, content in the middle, and an "on this page" rail with scroll-spy on the right. Above them sit a handful of top tabs, one per audience or mode. Practitioner guidance caps the top level at 5–7 items with labels under about four words. It asks for groups by user journey rather than product feature, verbs for task sections and nouns for reference, and no more than two levels of depth for critical content ([Mintlify](https://mintlify.com/docs/guides/navigation)). MongoDB's card sorts and tree tests pushed it to task-centric navigation for the same reason ([Fern](https://buildwithfern.com/post/information-architecture-best-practices-documentation)). Navigation must complement search, not be replaced by it: NN/g notes that search-first sites force users to "invest cognitive effort to create queries," while navigation teaches them the search space ([NN/g](https://www.nngroup.com/videos/search-box-vs-navigation/)). mdBook and pkg.go.dev show the keyboard layer power users expect: arrows between chapters, `/` or `S` to search, `?` for help, and `f` to jump to a symbol ([Rust Book](https://rust-book.cs.brown.edu/); [pkg.go.dev](https://pkg.go.dev/strings)). A "whole thing on one page" view (Gleam's "everything", mdBook's print) serves Ctrl-F and machine ingestion alike ([Gleam tour](https://tour.gleam.run/)).

**Search** as a Cmd/Ctrl-K modal is table stakes. What distinguishes one from another is indexing code examples, ranking pages where the term is the main subject, and scoping. The Vercel/Next.js palette is praised for the first two and criticised for mixing App Router and Pages Router results ([Docsio](https://docsio.co/blog/vercel-docs-teardown)). WebFluent's vocabulary is unusually exact (component names, codes like `T05`, commands like `wf build`), so search can act as navigation and jump straight to an entry. Shrestha's vocabulary barrier argues for synonyms: "useState" should find `state`, "computed" should find `derived`. AI "ask the docs" assistants are contested. The same teardown calls Next.js's "mostly theater", because it adds a step and drops code and edge cases ([Docsio](https://docsio.co/blog/vercel-docs-teardown)). Algolia advises scoping retrieval with facet filters ([Algolia](https://docsearch.algolia.com/docs/v4/askai)). The cheaper, widely adopted machine affordances are `llms.txt`, a `.md` version of every page and a "Copy page as Markdown" menu ([Mintlify](https://mintlify.com/docs/settings/llms); [PostHog](https://posthog.com/docs/ai-engineering/markdown-llms-txt); [Cloudflare](https://developers.cloudflare.com/style-guide/how-we-docs/ai-consumability/index.md)). WebFluent already has their substance in AGENTS.md and `wf context`.

**Code presentation** has a baseline. It includes a copy button; line, substring and diff markers; file-name and terminal frames; and synced tabs ([Starlight](https://starlight.astro.build/guides/authoring-content)). Google's style rules add an introductory sentence ending in a colon, omissions marked as comments in the language rather than `...`, and **no copy button on a block that omits code** ([Google](https://developers.google.com/style/code-samples)). The high end is interactive. MDN's "Try it" has three layouts, each a code editor beside a result: JS beside a console, HTML/CSS tabs beside rendered output, and a table of CSS variants beside one shared preview. Each layout's code loads only when it is on the page ([MDN](https://developer.mozilla.org/en-US/blog/mdn-front-end-deep-dive)). Stripe's Markdoc docs personalise samples with the reader's own test key and planned hover overlays showing reference inside code ([Stripe](https://stripe.dev/blog/markdoc)). Rust's doctests compile and run every documentation block, using hidden `#` lines for setup and `compile_fail` for intentionally broken samples ([rustdoc](https://doc.rust-lang.org/rustdoc/write-documentation/documentation-tests.html)). That mechanism, applied to `.wf` blocks through `wf check` and `wf test`'s `expect` steps, is the single most evidence-backed improvement available: it directly attacks incorrectness and obsolescence, the failures practitioners rate as blockers.

**Reference pages** at the best sites are signature-first, then prose, then examples. pkg.go.dev runs Overview → Index → Examples → functions and types, with a link to the exact source line and an "added in goX" note on each item ([pkg.go.dev](https://pkg.go.dev/strings)). react.dev splits every API page into **Reference** (formal signature) and **Usage** (recipes), and sometimes adds Troubleshooting ([React](https://react.dev/blog/2023/03/16/introducing-react-dev)). In April 2026 shadcn/ui added explicit **Composition** trees, showing what wraps what. The stated reason is that "LLMs and coding agents compose elements more reliably when they can see the full structure", and a CLI prints the docs for agents ([shadcn](https://ui.shadcn.com/docs/changelog/2026-04-component-composition)). WebFluent's parts (`Card.Header`, `Table.Row`) and slots map directly onto that tree, and `wf registry --json` already holds every prop, case, flag, event, slot and part.

**Versioning** fails when it is invisible. Next.js's switcher defaults to one version, lacks per-page warnings, and keeps a parallel Pages Router tree "often 2 to 3 months behind" ([Docsio](https://docsio.co/blog/vercel-docs-teardown)). Python's switcher spans every live and dead version ([docs.python.org](https://docs.python.org/3/)), and Zig shows stable and master tracks side by side ([Zig](https://ziglang.org/learn/)). MDN's Baseline badge shows how a status chip at the top of a reference page carries compatibility at a glance ([MDN](https://developer.mozilla.org/en-US/blog/mdn-front-end-deep-dive)). WebFluent's equivalents are "Since 4.2" or "Changed in 5.0" chips and output-support badges (Web, PDF, Slides, Template).

**Dark mode, mobile and accessibility** are expected rather than distinctive. MDN's 2026 rebuild is the reference implementation: a progressively enhanced theme switcher, light and dark variants of images, components stable before JavaScript, `<dialog>` for modals and `aria-expanded` on the nav toggle ([MDN](https://developer.mozilla.org/en-US/blog/mdn-front-end-deep-dive)). On mobile the sidebar becomes a drawer, the right rail a dropdown, and search stays one tap away. No study of developer-docs reading on mobile was found, so mobile priority rests on general practice. Feedback widgets ("Was this page helpful?"), "Edit this page" and "Last updated" are ubiquitous ([dbt](https://docs.getdbt.com/navigation-options)), but no study of their effect was found either.

## Seven pitfalls, ranked by how much they cost readers

The evidence ranks the pitfalls. First and most severe is **wrong or stale content**: incorrectness is rare but rated a blocker, and 93% of open-source respondents see incomplete or outdated docs ([Uddin & Robillard 2015](https://www.cs.mcgill.ca/~martin/papers/ieeesw2015.pdf); [GitHub 2017](https://opensourcesurvey.org/2017/)). For WebFluent's AGENTS.md-derived prose this risk is acute, because the language changes fast (4.2 removed `external`; 5.0 refuses what 4 compiled). Second is **unexplained or badly sized examples**: single-call examples are too thin and long ones tangle ([Robillard & DeLine 2011](https://www.cs.mcgill.ca/~martin/papers/ese2011.pdf)). Third is **organising by content type and separating concepts from the code that needs them** ([Meng et al. 2019](https://sigdoc.acm.org/cdq/how-developers-use-api-documentation-an-observation-study/)). Taken literally, Diátaxis's bucketing can cause exactly this, which is why its authors insist on working page by page. Fourth is **fragmentation without a spine**, which overwhelms systematic readers. Fifth is **assumed knowledge and no bridge from prior languages**, with its 61% interference rate. Sixth is **vocabulary-blind search**. Seventh is **burying the point** below the fold or mid-paragraph, which F-pattern readers miss in both reading directions ([NN/g](https://nngroup.com/articles/f-shaped-pattern-reading-web-content/)).

Two process pitfalls sit underneath. Documentation decays when writing it is optional: 60% of open-source contributors rarely or never write docs ([GitHub 2017](https://opensourcesurvey.org/2017/)). Docs-as-code, meaning docs in the repository, reviewed and tested like code, is the structural answer ([Write the Docs](https://www.writethedocs.org/guide/docs-as-code/)). And AI-generated or AI-mediated documentation risks reproducing the "almost right" content developers now distrust. An AI layer should therefore sit on top of verified content and cite it, never replace it. Finally, a scoping caveat: everything above is evidence about reading behaviour and stated pain. **No study was found that measures whether in-browser playgrounds, persona routing on a home page, or Cmd-K palettes improve outcomes.** Those choices rest on convergent industry practice, not measurement.

## Ten UX directions for the WebFluent docs

Each direction below is a coherent bet, not a feature list. All of them should share a common floor that the evidence makes non-negotiable: every `.wf` sample compiled in CI, with `compile_fail`-style markers for diagnostics; reference generated from the registry, CLI and config schema; per-code diagnostic pages linked from the compiler's `docs:` URLs; Cmd-K search with exact-match jumps; a visible version and output-type indicator; `.md` and `llms.txt` per page; and RTL and dark mode from the first paint. They differ in what they make the spine.

| # | Direction | Spine | Best for |
|---|---|---|---|
| 1 | Four Rooms | Diátaxis modes as top tabs | A large, growing team that needs content rules |
| 2 | The Live Tour | Split-pane, in-browser tutorial | Driving adoption and first impressions |
| 3 | Pick Your Output | Output type (site, app, PDF, deck, elements) | WebFluent's breadth as its pitch |
| 4 | Coming From | Reader's prior framework | React, Vue and Svelte switchers |
| 5 | The Registry Atlas | Generated component and API reference | Daily users, agents, design-system teams |
| 6 | Compiler as Guide | Diagnostics index as an entry point | Users who learn by fixing errors |
| 7 | One Long Book | A linear, printable book with a measured spine | Systematic readers; a stable language |
| 8 | Recipe-First | Task cookbook backed by the showcase | Opportunistic, working developers |
| 9 | Source and Output | Every example shown as `.wf` beside HTML, CSS, JS and PDF | Developers who trust what they can inspect |
| 10 | Agent-Native Docs | Machine-readable docs as a first-class surface | AI-assisted development |

### 1. Four Rooms

**Concept:** a Diátaxis site whose top navigation names the reader's need: Learn, How to, Reference, Understand.

**Layout:** four top tabs plus Showcase, each owning its own sidebar. Learn holds one tutorial with prev/next and a progress marker. How to holds "How to…" recipes. Reference holds the components, language, CLI, config, runtime and diagnostics. Understand holds "About…" essays. The home page is a Python-style grid of the four rooms, with search above it. Every page carries a small mode label.

**Draws on:** Diátaxis and Divio; the four-way indexes of Canonical and Django; Mintlify's 5–7 top-level guidance; Write the Docs' "Cumulative" principle.

**Strengths:** gives writers an unambiguous rule for every new page. It keeps 77 components out of the guide sidebar, and it is a proven, well-known pattern.

**Risks:** the "empty boxes first" trap that Diátaxis warns against. It separates concepts from tasks, which Meng's data argues against, and it needs cross-link discipline (the Next.js lesson). Readers must learn which room holds what.

**Suits:** a project expecting many contributors and long-term growth; maintainers who want governance more than flair.

### 2. The Live Tour

**Concept:** the docs open into a Gleam/Svelte-style split pane where the reader edits `.wf` and watches the real page render.

**Layout:** the home hero is an editable snippet with a live preview. The Tour is a three-pane app: lesson text on the left, a file tree and editor in the middle, and a rendered page plus a diagnostics panel on the right. It has a `solve` button, a contents list and an "everything" one-page view. The ~50-chapter guide sits behind it as the long-form follow-up.

**Draws on:** Gleam's in-browser compiler; Svelte's file-tree tutorial; Kotlin's no-install tour; react.dev's 600 sandboxes; Meng's "fast try-outs"; the Rust Book data that chapters 1–3 hold most readers.

**Strengths:** the strongest first impression available to a language that compiles to the browser. Its output pane is a real UI rather than console text, and every lesson is verified by construction.

**Risks:** requires the compiler in WASM, with bundle weight (mitigate with MDN-style load-on-click and a static first paint). No study shows playgrounds improve learning, and tour content can fork from the guide.

**Suits:** newcomers and evaluators; a launch or growth phase.

### 3. Pick Your Output

**Concept:** "What are you making?" as the first question, because WebFluent's real personas are defined by what it compiles to.

**Layout:** the home page shows six cards: a website, an app, a PDF report, a slide deck, custom elements, and a desktop or phone app. Each opens a tailored quickstart (`wf init -t …`) and a filtered path through the guide. A persistent output switcher in the sidebar filters the guide and greys out what does not apply, so a PDF reader sees E109 notes where controls are refused. Component pages carry Web, PDF, Slides and Template badges.

**Draws on:** Mintlify's "tabs when audiences have fundamentally different journeys"; Vercel's audience-split docs; Next.js's router switcher, including its failure modes; MDN Baseline badges; task-based IA research (MongoDB, Fern).

**Strengths:** turns WebFluent's breadth into the pitch, cuts each reader's surface area sharply, and makes "what works where" visible.

**Risks:** content parity across variants (Next.js's Pages Router tree lagged "2 to 3 months"). Most concepts are shared, so heavy filtering can hide what applies everywhere, and the switcher must be visible on every page.

**Suits:** a product whose users mostly need one output; document and report generators as much as web developers.

### 4. Coming From

**Concept:** on-ramps keyed to what the reader already knows, built around side-by-side mappings and interference warnings.

**Layout:** Get Started offers entries in the TypeScript style: "for React developers", "for Vue/Svelte", "for HTML/CSS authors", "for LaTeX/Typst users (PDF)", and "new to programming". Each is a two-column cheat sheet (their idiom beside the WebFluent idiom), followed by a "things that work differently" page fed by real diagnostics (E106, E002, flag placement). Search maps foreign vocabulary to WebFluent terms. The rest of the site is a conventional guide and reference.

**Draws on:** Shrestha et al.'s 61% interference rate and the cheat-sheet finding; Robillard's "bridge" docs; TypeScript's five on-ramps and Zig's comparisons by background; the vocabulary barrier.

**Strengths:** the most directly evidence-backed answer to the most predictable newcomer errors. It is cheap to build and highly searchable ("svelte vs webfluent").

**Risks:** comparison pages rot as other frameworks change, and they invite flame-war framing. They can also anchor readers in the old mental model instead of the new one.

**Suits:** a launch aimed at existing front-end developers; the largest likely audience.

### 5. The Registry Atlas

**Concept:** a reference-first site generated from the compiler's own registry, where every built-in is a rich, uniform, live page.

**Layout:** the component index is a filterable thumbnail grid by category. Each component page runs: name and one-line description; a live preview; a flag and variant gallery; a Storybook-style controls panel that toggles flags and enums and rewrites the snippet; a composition tree of parts and slots; a props table; events; the A-codes it can trigger; related diagnostics; and "Since" chips. A Stripe-style sticky preview column sits at the right on wide screens. Language, CLI, config and runtime references follow the same template. The guide is secondary.

**Draws on:** reference's 18% share of reading time (Meng); pkg.go.dev's Jump-to outline and "added in" notes; react.dev's Reference/Usage split; shadcn's composition trees for agents; Diátaxis's "mirror the machinery"; Write the Docs' "Complete" principle.

**Strengths:** can never drift from the compiler, scales to future components at no writing cost, and serves both humans and agents.

**Risks:** generated reference "is not enough on its own" (Diátaxis), and boilerplate wastes readers' time (Robillard). It underserves learners, and the controls panel can overshadow prose.

**Suits:** daily users, teams building on the component set, coding agents.

### 6. Compiler as Guide

**Concept:** the diagnostics index becomes a front door. Every error the reader hits opens a page that teaches the concept behind it.

**Layout:** a Rust-style index of every code, grouped by family (E, T, C, R, X, F, I, D, A, S, P, U, V). Each page gives a one-line meaning, the broken code with the real rendered compiler output, one or more fixes rendered live, a "why this rule exists" paragraph, and links to the guide chapter and the component it concerns. Typing `T05` in search lands there. The compiler's `docs:` links, `wf explain` and the editor's hover all point here. The guide's chapters link back to "the errors you'll see here".

**Draws on:** Rust's E0382 page shape and `rustc --explain`; Elm's "compilers as assistants"; Denny et al. on readable messages; Carroll's minimalism (support error recognition and recovery); DITA troubleshooting topics; Brown's practice of fixing explanations where learners fail.

**Strengths:** meets readers at their moment of highest motivation. It uses an asset WebFluent already has and few young languages do, and doctested `compile_fail` examples keep it honest.

**Risks:** an error-led path teaches by failure and is no substitute for a coherent overview. Over 100 pages need the same care, and the codes must stay stable.

**Suits:** working developers mid-task; a project whose compiler is its strongest feature.

### 7. One Long Book

**Concept:** an mdBook-style linear book, reorganised so its order follows difficulty, with comprehension checks at the hard parts.

**Layout:** a single sidebar of numbered chapters with prev/next and arrow-key navigation, a one-page or print view, and ePub/PDF export (which WebFluent can produce itself). Short concept quizzes sit at the end of the hard chapters (reactivity, stores, flags and the registry), with cookie-based analytics and no accounts. The reference and showcase are separate, linked appendices.

**Draws on:** Robillard's systematic readers who want a coherent linear overview; the Rust Book and Brown's interactive edition (+20% from targeted rewrites, ~50 hours and ~$15 a month); Gleam's "everything" page; mdBook's keyboard model.

**Strengths:** simple, fast, durable and printable. It gives the systematic half of the audience what it needs, and the quiz data shows exactly where to rewrite.

**Risks:** 98% of Rust Book readers never reached chapter 19. It serves opportunistic readers poorly unless every chapter also stands alone (EPPO), and a long book discourages deep linking into the middle.

**Suits:** a stabilising language; teams that want to measure and improve the guide over time.

### 8. Recipe-First

**Concept:** the site is organised around tasks ("How to fetch and show a list", "How to ship a PDF invoice"), each a complete, runnable recipe that links to its concepts and reference.

**Layout:** the home page is a task finder: search plus categories phrased as verbs (Build, Fetch, Style, Ship, Test, Translate). Each recipe page runs: goal; complete code with comments carrying the critical rules; a live result; "what's happening" in two short paragraphs; and "used here" chips linking to every component and keyword. The showcase becomes a gallery of complete apps, each decomposed into the recipes it uses. Concept material is inlined where tasks need it.

**Draws on:** Meng's task-organised, concepts-inline and comments-in-code recommendations; recipes' 15% share of reading time; Diátaxis how-to rules; Robillard on multi-call examples; MongoDB and Fern task-centric IA; the "Ten idioms" seed.

**Strengths:** matches how most developers actually read, makes the showcase earn its place, and every page answers a search query directly.

**Risks:** fragmentation without a spine (Robillard's Implication 7), and duplication across recipes. Concepts scattered inline are hard to study whole, so it needs a light tutorial and glossary alongside.

**Suits:** working developers, product teams shipping features.

### 9. Source and Output

**Concept:** the docs show what the compiler makes, rendering every example beside its compiled HTML, CSS and JS (and PDF pages) so the reader sees through the abstraction.

**Layout:** every example is a block with tabs: `.wf`, `.wfx`, Preview, HTML, CSS, JS, and for document examples a PDF page thumbnail. The tabs sync site-wide in localStorage. Hovering a token (`Button`, `.primary`, `$surface`) shows its registry entry and highlights the generated markup it produced. An "under the hood" explanation track (runtime modules, hydration, the CSP the build ships, bundle weight from `--stats`) runs alongside the guide.

**Draws on:** Robillard's call to explain the abstract execution model; Shrestha's SAS complaint about the execution model left a mystery; Stripe's planned hover overlays; Starlight's synced tabs; MDN's editor-and-result layout; Busjahn on experts' non-linear code reading.

**Strengths:** builds trust (no magic) and suits experienced web developers. It turns WebFluent's honest output and its security guarantees into a teaching surface, and it scales with no extra writing.

**Risks:** compiled output is noisy and can intimidate beginners. It invites readers to depend on implementation details that will change, and it is heavier to render.

**Suits:** senior front-end developers evaluating adoption; contributors; the curious.

### 10. Agent-Native Docs

**Concept:** treat the coding agent as a first-class reader. Every page is both a human page and a clean machine document, and the human site shows what the agent sees.

**Layout:** a conventional human shell (three zones, Cmd-K). Every page has a "Copy as Markdown / Open in an assistant" menu, a `.md` twin and an entry in `llms.txt` and `llms-full.txt`. Component pages lead with composition trees. The site publishes `wf context`, `wf registry --json` and AGENTS.md as browsable, versioned documents, plus an MCP endpoint. An optional "ask the docs" assistant answers only from version-scoped, cited chunks, with code blocks preserved.

**Draws on:** ExDoc's llms.txt and Copy Markdown; Mintlify, PostHog, Cloudflare and dbt affordances; shadcn's composition-for-agents rationale; MDN's MCP server; Stack Overflow 2025's "almost right" distrust; Algolia's scoping advice; the Next.js AI chat critique.

**Strengths:** WebFluent already ships most of the machinery. It makes agents right rather than almost right, which is a differentiator in 2026.

**Risks:** an AI assistant can become "theater" and answer for the wrong version. It is unmeasured (no usage data found), it can drift attention away from human prose quality, and it pulls in third-party origins that the strict CSP posture discourages.

**Suits:** teams that build with AI agents; WebFluent's own AGENTS.md-centred workflow.

## Conclusion

The research reframes the redesign. The biggest gains will not come from navigation chrome. They come from closing the gap between the docs and the compiler. Most languages must bolt verification onto prose. WebFluent can generate its reference from the registry, doctest every `.wf` block, render every example live and route every diagnostic to a page that teaches. That attacks the failure modes readers rate as blockers (wrong, stale and unexplained) at their root, and few competitors can match it cheaply. The ten directions are best read as choices of spine on top of that shared verified core, not as mutually exclusive designs. The evidence favours combining a task-and-example surface for the opportunistic majority (Directions 5, 6 and 8) with one coherent linear path for the systematic minority (2 or 7), entered through the reader's real starting point (3 or 4).

Two implications are less obvious. First, because readers abandon guides early and stumble on one hard concept, the most valuable instrument is not a feedback widget but a handful of concept questions placed at WebFluent's equivalent of Rust's ownership chapter. They cost tens of hours, and the Rust data shows they reveal exactly which paragraphs to rewrite. Second, the rising distrust of "almost right" AI answers makes WebFluent's machine-readable surfaces a quality lever, not a side feature. The same verified registry that powers the human reference is what keeps agents from guessing, so the human docs and the agent docs should be one pipeline, tested once.
