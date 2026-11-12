# Frameworks and writing principles for programming-language documentation

Scope: information-architecture frameworks and writing rules from the strongest developer docs, as concrete rules a docs team can apply. Written for a redesign of the WebFluent docs site, which today has a guide of about 50 chapters, a reference for 77 built-in components, a showcase and a home page. Researched November 2026, from primary sources where they could be reached.

## Diátaxis in depth: the four modes, the axes, how to write each, mixing mistakes, adoption

### Takeaway
Diátaxis sorts documentation by the reader's need along two axes. The first is action against cognition (doing against knowing). The second is acquisition against application (study against work). Each of the four modes has its own writing rules. The most common failure is mixing tutorials with how-to guides, and explanation with reference. Diátaxis is meant to be applied a page at a time, from the inside out. Its authors say not to start by creating four empty boxes.

### Cited Findings
**Purpose and name**
- Diátaxis covers three things: content (what to write), style (how to write it) and architecture (how to organise it). It identifies "four distinct needs, and four corresponding forms of documentation": tutorials, how-to guides, technical reference and explanation. The name comes from the Greek *dia* ("across") and *taxis* ("arrangement"). — [diataxis.fr](https://diataxis.fr/)

**The compass**
- Two questions place any piece of content: "action or cognition?" and "acquisition or application?". In plain terms, action means practical steps, cognition means theoretical knowledge, acquisition means study and application means work. — [The compass](https://diataxis.fr/compass/)
- The answers map to the modes like this:
  - Guides doing, for learning: a tutorial.
  - Guides doing, for applying: a how-to guide.
  - Informs thinking, for applying: reference.
  - Informs thinking, for learning: explanation.
  — [The compass](https://diataxis.fr/compass/)
- The compass works at any scale, from a single sentence to a whole document. Use it on what you plan to write, on existing text and on what the user needs. — [The compass](https://diataxis.fr/compass/)

**Tutorials (learning-oriented)**
- Don't teach by telling. Give the learner things to do, and let the learning come from the doing. — [Tutorials](https://diataxis.fr/tutorials/)
- The rules:
  - Say at the start what will be built, and avoid the phrase "you will learn".
  - Deliver visible results early and often.
  - Keep up a "narrative of the expected": show sample output and flag likely mistakes.
  - Point out what to notice.
  - Allow steps to be repeated.
  - Ruthlessly minimise explanation, and link to fuller explanation elsewhere.
  - Stay concrete.
  - Ignore options and alternatives.
  - Aim for perfect reliability, tested with real users.

  — [Tutorials](https://diataxis.fr/tutorials/)
- Typical phrasing in a tutorial — [Tutorials](https://diataxis.fr/tutorials/):
  - "We …"
  - "In this tutorial, we will …"
  - "First, do x. Now, do y."
  - "The output should look something like…"
  - "Notice that…"
  - "You have built…"

**How-to guides (task-oriented)**
- Address one real goal (for example, "how to configure reconnection back-off policies"), not an open-ended subject. Write from the user's project, not the tool's operations. Assume the reader is competent. Leave out explanation and exhaustive option lists and link to them instead. Put practical usefulness ahead of completeness. A guide may branch and may have several ways in and out. — [How-to guides](https://diataxis.fr/how-to-guides/)
- Titles: "How to integrate application performance monitoring" is good. "Integrating application performance monitoring" is ambiguous. A bare "Application performance monitoring" is the weakest. Open with "This guide shows you how to…". Use conditional imperatives ("If you want X, do Y"). — [How-to guides](https://diataxis.fr/how-to-guides/)
- The recipe is the model: it answers one specific question, assumes basic competence and leaves out teaching. — [How-to guides](https://diataxis.fr/how-to-guides/)

**Reference (information-oriented)**
- "Neutral description is the key imperative of technical reference." Describe and only describe: don't instruct, explain or opine. Use standard, consistent patterns. "The structure of the documentation should mirror the structure of the product." Give brief usage examples. The tone is austere, because readers consult reference rather than read it. — [Reference](https://diataxis.fr/reference/)
- Auto-generated reference is a strong way to stay accurate, but it is not enough on its own, and it must meet the same standards. — [Reference](https://diataxis.fr/reference/)

**Explanation (understanding-oriented)**
- Make connections, give context (design decisions, history, constraints), talk *about* the subject, admit opinion and alternatives, and keep it closely bounded with no instructions or specs creeping in. Title it "About X". Other names for the section are Discussion, Background, Conceptual guides and Topics. — [Explanation](https://diataxis.fr/explanation/)
- Explanation is neglected because it feels less urgent, gets scattered across other sections and is open-ended. A "why" question can serve as its boundary. — [Explanation](https://diataxis.fr/explanation/)

**Common mixing mistakes**
- Confusing tutorials with how-to guides is "the most common mistake". Tutorials serve a user "at study", on one safe path managed by the teacher. How-to guides serve a user "at work", who carries the responsibility and meets real-world branches. — [Tutorials and how-to guides](https://diataxis.fr/tutorials-how-to/)
- Basic against advanced is not the distinction: a how-to can cover something basic, and a tutorial can teach something advanced. The analogy is a trainee learning to suture versus a surgeon following a clinical manual. — [Tutorials and how-to guides](https://diataxis.fr/tutorials-how-to/)
- The test between reference and explanation: would the reader consult it mid-task (reference) or after stepping away to think (explanation)? — [Reference and explanation](https://diataxis.fr/reference-explanation/)
  - Examples tempt writers into digressions about "why" and "what if". That clutters the reference, and it also starves the explanation of a page of its own.
  - A tide chart is reference; an article on why tides happen is explanation.

**Workflow**
- Don't build the structure first. Creating empty tutorial, how-to, reference and explanation sections is called "horrible". "Diátaxis changes the structure of your documentation from the inside." — [Workflow](https://diataxis.fr/how-to-use-diataxis/)
- The cycle is choose → assess → decide → do. Publish each small improvement at once, aim for "complete, not finished", and allow "well-formed organic growth". — [Workflow](https://diataxis.fr/how-to-use-diataxis/)

**Adoption**
- Canonical announced Diátaxis on 15 December 2021, written by Daniele Procida, as the foundation of all its product documentation. — [Canonical blog](https://canonical.com/blog/diataxis-a-new-foundation-for-canonical-documentation)
  - Each piece of content belongs to exactly one type, and the four-type split is visible at the top of each product's docs (Anbox Cloud, Charmed Kubeflow, Ubuntu Core and others).
  - Their lesson: applying the framework makes the docs look worse before they improve, because mixed modes become visible.
- Ubuntu's Multipass team overhauled its docs in 2022 as an early adopter. — [Ubuntu blog](https://ubuntu.com/blog/multipass-documentation-proudly-a-work-in-progress?format=md)
- Testimonials on diataxis.fr — [diataxis.fr](https://diataxis.fr/):
  - Cloudflare: "Diátaxis became our north star for information architecture."
  - Gatsby: "The four quadrants helped us prioritize the user's goal for each type of documentation."
- Django's docs index already uses the four-way split under its own names — [Django docs](https://docs.djangoproject.com/en/stable/):
  - Tutorials (`intro/`): "take you by the hand".
  - Topic guides (`topics/`): explanation, "fairly high level… background".
  - Reference guides (`ref/`): "assume that you have a basic understanding of key concepts".
  - How-to guides (`howto/`): "recipes".
- Procida contributed to Django's documentation, and the framework's ideas first crystallised at Divio (per search-result summaries of Canonical and DjangoCon pages). — [Canonical](https://canonical.com/blog/the-future-of-documentation-at-canonical); [DjangoCon EU 2022 speaker page](https://2022.djangocon.eu/pretalx.evolutio.pt/djangocon-europe-2022/speaker/FDRCZ3/index.html)

### Inferences
- For WebFluent, the ~50-chapter guide almost certainly mixes modes. AGENTS.md-style prose such as "Data Fetching" moves from explanation to option lists to examples within one section. Running the compass over each chapter, one at a time, will show which parts are tutorials, how-tos, reference (component props, config keys, diagnostic codes) and explanation (reactivity model, why stores build on first read, security guarantees).
- The 77-component reference should "mirror the machinery". It should be generated from the registry the compiler already reads (`wf registry --json`), so its structure (props, cases, flags, events, slots, parts) is uniform, and its examples should stay brief.
- Following Diátaxis's own workflow, the redesign should not create four empty top-level buckets first. It should improve chapters one by one and let the navigation follow, even if the final top-level navigation does show the four types, as Django's and Canonical's do.

### Gaps
- The "complex hierarchies" page (two-dimensional structures, landing pages) returned 404 at both URLs tried, so I could not confirm its current guidance.
- NumPy's adoption (NEP 44, "Restructuring the NumPy documentation") could not be fetched. NumPy appears only on a self-reported adopters list per search snippets, so its adoption is unverified here.
- I found no primary source confirming that Django formally "adopted Diátaxis". Its structure matches, but the order of causation is not documented in what I read.

## The Divio documentation system and its relation to Diátaxis

### Takeaway
The Divio "documentation system" is the earlier form of the same idea: "documentation is not one thing but four". It has the same four functions. Diátaxis is Procida's later, independent and more developed version, which adds the compass, the theory and the workflow.

### Cited Findings
- Divio's page opens by saying there is a secret to good software documentation: it "isn't one thing, it's four". It splits docs into tutorials, how-to guides, technical reference and explanation, each needing its own approach, and credits the quote "The Grand Unified Theory of Documentation" to David Laing. — [Divio documentation system](https://docs.divio.com/documentation-system/)
- The key concepts behind Diátaxis "first crystallised while working at Divio" (Procida, per search-result summary of Canonical's pages). — [Canonical](https://canonical.com/blog/the-future-of-documentation-at-canonical)

### Inferences
- Treat Divio as the historical snapshot and diataxis.fr as the canonical, current text to cite in the WebFluent docs style guide.

### Gaps
- The fetched Divio page did not state outright that Diátaxis supersedes it, and the fetch did not include the cooking analogy, so I have no primary quote on the relationship.

## Other frameworks and style guidance

### Takeaway
The style guides agree on short, front-loaded, second-person and scannable text, with uniform code-sample formatting. Write the Docs adds structural principles: ARID, skimmable, addressable, cumulative, and "incorrect worse than missing". Every Page is Page One and DITA argue for self-contained, typed topics, because readers land from search. Minimalism and progressive disclosure argue for action first, error recovery, and deferring the rare.

### Cited Findings
**Google developer documentation style guide (code samples)**
- Indent with spaces (usually two) per the language's style guide, and wrap at 80 characters. — [Google: code samples](https://developers.google.com/style/code-samples)
- Mark omitted code with a comment in the sample's own language, never `...` or an ellipsis. Don't make a block click-to-copy if it omits code. — [Google: code samples](https://developers.google.com/style/code-samples)
- Put an introductory sentence before most samples. End it with a colon when the sample follows immediately, and with a period when something else comes between. — [Google: code samples](https://developers.google.com/style/code-samples)
- Follow the language's public style guide, or the project's own style guide where one overrides it. — [Google: code samples](https://developers.google.com/style/code-samples)

**Microsoft Writing Style Guide: top 10 tips (page dated July 2026)** — [Microsoft: top 10 tips](https://learn.microsoft.com/en-us/style-guide/top-10-tips-style-voice)
- Use bigger ideas and fewer words ("Shorter is always better").
- Write like you speak, and read the text aloud.
- Project friendliness with contractions.
- Get to the point fast: lead with what's most important and front-load keywords for scanning.
- Be brief and prune every excess word.
- Use sentence-style capitalisation.
- Put no end punctuation on headings.
- Use the serial comma.
- Use one space after punctuation and none around em dashes.
- Revise weak writing: start statements with a verb, cut an unnecessary "you can", and avoid "there is/are".
- An example of the guide's error-message advice: replace "Invalid ID" with "You need an ID that looks like this: someone@example.com".

**Write the Docs principles** — [Write the Docs: principles](https://www.writethedocs.org/guide/writing/docs-principles/)
- **Precursory:** document before you code.
- **Participatory:** include everyone, and let readers give feedback.
- **ARID:** "Accept (some) Repetition In Documentation".
- **Skimmable:** descriptive headings, meaningful link text, key concepts first.
- **Exemplary:** examples for common cases, but not too many, and kept separate from dense reference.
- **Consistent.**
- **Current:** "Consider incorrect documentation to be worse than missing documentation".
- **Nearby:** docs live next to the code.
- **Unique:** sources do not overlap.
- **Discoverable.**
- **Addressable:** links to granular sections.
- **Cumulative:** prerequisites first, tutorials before reference.
- **Complete:** cover a concept fully or not at all, and say so up front if coverage is partial.
- **Beautiful:** even `--help` text.
- **Comprehensive:** the whole body of docs answers the likely questions.

**Docs as code**
- Write docs with the same tools and workflows as the software: issue trackers, Git, plain-text markup, code review and automated tests. Developers often draft the docs, and teams can require docs before merging a feature. — [Write the Docs: Docs as Code](https://www.writethedocs.org/guide/docs-as-code/)

**Every Page is Page One (Mark Baker)** — [EPPO: the book](https://everypageispageone.com/the-book/)
- Readers arrive at pages from search and links, and they forage for good-enough answers.
- An EPPO topic has seven characteristics:
  1. Self-contained ("function alone").
  2. A specific and limited purpose.
  3. Conforms to a type that is specific to the subject matter, not a generic concept or task.
  4. Establishes its own context.
  5. Assumes a qualified reader, while giving others links to become qualified.
  6. Stays on one level, never jumping between overview and detail.
  7. Links richly along lines of subject affinity.
- The architecture is bottom-up, with each page a hub, rather than a book-style table of contents.

**DITA (OASIS)**
- The core topic types answer distinct questions: concept ("What is…?"), task ("How do I…?") and reference (facts). DITA 1.3 adds troubleshooting (corrective action) and glossary entries (one sense of one term per entry, a specialisation of concept). — [OASIS DITA 1.3 information types](https://docs.oasis-open.org/dita/dita/v1.3/os/part2-tech-content/archSpec/technicalContent/dita-technicalContent-InformationTypes.html); summary via search results

**Minimalism (John Carroll)** (secondary sources only)
- Carroll's minimal manual rests on four principles: an action-oriented approach, text optimisation, support for error recognition and recovery, and modularity. The action principle prefers wording that invites activity, such as commands and questions, over declarative statements. — [van der Meij & Lazonder 1993, via search summary](https://meilu.jpshuntong.com/url-68747470733a2f2f7269732e757477656e74652e6e6c/ws/files/6853553/Meij93assessment.pdf); [instructionaldesign.org](https://www.instructionaldesign.org/theories/minimalism/)
- Error information should support detection, diagnosis and recovery. Give it where errors are costly or frequent, and present general recovery methods before specific ones. — [instructionaldesign.org](https://www.instructionaldesign.org/theories/minimalism/); [EduTech wiki](https://edutechwiki.unige.ch/en/Minimalist_instruction)
- JoAnn Hackos's later formulation: action-oriented; anchor the tool in the task domain; support error recognition and recovery; support reading to do, to study and to locate. — [EPPO: What is minimalism?](https://everypageispageone.com/2013/07/02/what-is-minimalism)

**Progressive disclosure (Nielsen Norman Group)** — [NN/g: progressive disclosure](https://www.nngroup.com/articles/progressive-disclosure/)
- Defer "advanced or rarely used features to a secondary screen".
- Get the split between the initial and secondary layers right, and make the path to the secondary layer obvious, with an accurate label.
- Keep to two levels where possible, since three or more confuse users.
- Staged disclosure, as in a wizard, is linear and suits tasks with independent steps.

### Inferences
- EPPO and Diátaxis fit together. Diátaxis decides what *kind* of page something is. EPPO says each page must stand alone, because most WebFluent readers will arrive from search or from a diagnostic's `docs:` link.
- Applied to reference pages, progressive disclosure means:
  - Lead each component page with the common call and its most-used props.
  - Put the full props, cases, flags, events, slots and parts table below it.
  - Push edge behaviour (hydration, pre-render, the template engine) to a clearly labelled lower section.
- The "Complete" principle and "incorrect is worse than missing" argue for generating reference from the compiler's registry, and for testing every sample (docs as code).

### Gaps
- I did not fetch Google's voice, tone or placeholder pages, or Microsoft's pages on code examples and global readiness. Only the code-sample page and the top-10 page were read.
- The van der Meij & Carroll 1998 chapter "Principles and heuristics for designing minimalist instruction" was not read directly, so the exact wording of the four principles is unconfirmed.

## What a language needs beyond a library, and which frameworks address it

### Takeaway
Mature language docs (Rust, Python, Go, TypeScript) offer a recognisable set of documents:
- an interactive or book-length tutorial;
- a language reference or spec;
- a standard-library or API reference;
- a code-heavy "by example" collection or cookbook;
- an error-code index (Rust: `rustc --explain`);
- version and migration documents (What's New, edition guides, migration guides);
- a glossary;
- "coming from X" on-ramps (TypeScript);
- a style or idioms guide (Effective Go, the Rust Style Guide).

Diátaxis covers these only generically. DITA's troubleshooting and glossary types, and EPPO's subject-specific types, fit the error index and glossary more naturally.

### Cited Findings
**Rust** — [doc.rust-lang.org](https://doc.rust-lang.org/)
- The docs index groups resources by reader stage: *Learning Rust*, *Using Rust*, *Mastering Rust* and *Specialized Rust*.
  - Learning: the Book ("from first principles", with projects), Rust By Example ("a lot of code with few words", with exercises), Rustlings (interactive exercises) and the Playground.
  - Using: the std API docs, whose examples have a "Run" button into the Playground; release notes; the Edition Guide; the rustc, Cargo, Rustdoc and Clippy books; and the Extended Error Listing, also available through `rustc --explain`.
  - Mastering: the Reference ("not a formal spec"), the Style Guide, the Rustonomicon and the Unstable Book.
- Each error-code page follows the same shape. E0382 is an example — [E0382](https://doc.rust-lang.org/error_codes/E0382.html):
  - a one-line meaning ("A variable was used after its contents have been moved elsewhere");
  - an "Erroneous code example";
  - an explanation tied to the governing rule;
  - several fixed alternatives, each with a short rationale;
  - a link to the Book chapter that explains the concept.

**Python** — [docs.python.org](https://docs.python.org/3/)
- The docs index lists:
  - What's New (every version since 2.0);
  - a Tutorial;
  - a Built-ins reference, a Library reference and a Language reference ("Syntax and language elements");
  - Setup and usage;
  - HOWTOs ("in-depth topic manuals");
  - FAQs and Deprecations;
  - a Glossary, a general index and a module index.

**Go** — [go.dev/doc](https://go.dev/doc/)
- Getting Started: install, several short single-goal tutorials, and A Tour of Go (interactive, four sections, each ending with exercises).
- Using and Understanding Go: Effective Go ("a must read for any new Go programmer", after the tour and the spec), the FAQ, guides, and a migration guide (`encoding/json/v2`).
- References: package docs, command docs, the Language Specification and the Memory Model.
- Also listed: Talks, Codewalks and blog articles.

**TypeScript** — [typescriptlang.org/docs](https://www.typescriptlang.org/docs/)
- Get Started offers on-ramps by background: TS for the New Programmer, for JavaScript Programmers, for Java/C# Programmers, for Functional Programmers, and a 5-minute tooling tour.
- Then: the Handbook (daily reading), Reference (deep dives), cheat sheets, declaration files and project configuration.

**How the frameworks cover these**
- Diátaxis's four modes are generic. Its reference rule ("mirror the machinery") covers the language reference, the std reference and an error index. "How-to as recipe" covers cookbook and by-example material. — [Reference](https://diataxis.fr/reference/); [How-to guides](https://diataxis.fr/how-to-guides/)
- DITA has explicit troubleshooting and glossary-entry types. — [OASIS DITA 1.3](https://docs.oasis-open.org/dita/dita/v1.3/os/part2-tech-content/archSpec/technicalContent/dita-technicalContent-InformationTypes.html)
- EPPO says topic types should be specific to the subject matter. — [EPPO](https://everypageispageone.com/the-book/)

### Inferences
- **Proposed map for WebFluent.** WebFluent already has many of the pieces of a language's docs. In Diátaxis terms:
  - **Tutorial:** one guided build, `wf init`, then a page, a store, a test and `wf build`, with expected output at each step.
  - **How-to/cookbook:** short recipes titled "How to…", for example: how to fetch and show a list, how to add dark mode, how to ship a PDF invoice, how to persist a setting, how to add a third-party script. The "Ten idioms" in AGENTS.md are recipe seeds.
  - **Reference:**
    - a language reference (syntax, declarations, types, expressions, strings);
    - a component reference for the 77 built-ins, generated from the registry;
    - a CLI reference, generated from `wf commands --json`;
    - a config reference, generated from `wf config schema --json`;
    - a runtime (`WF.*`) reference;
    - a diagnostics index with one page per code, like Rust's E0382 (meaning, erroneous example, fixed example, link to the concept). `wf explain` already produces entry text, and diagnostics already carry `docs:` URLs.
  - **Explanation:** "About" pages on reactivity and signals, store lifetime, the security model, how a page is pre-rendered and hydrated, and why a PDF is "the page, printed".
  - **Version docs:** What's New per release, plus migration guides (2→3, 3→4, 4→4.2, 4→5).
  - **Glossary:** state, derived, signal, flag, case, part, slot, resource and similar terms.
  - **Coming from X:** on-ramps for readers who know React, Vue or Svelte, plain HTML/CSS, or LaTeX/Typst for PDFs.
- Rust's grouping by reader stage (Learn, Use, Master) is an alternative top-level navigation that still sits on top of Diátaxis types. It may suit a language home page better than four bare mode names.
- The showcase is neither a tutorial nor reference. It works best as a gallery of complete, runnable examples, close to Rust By Example and Go's codewalks, with each example linking to the recipes and reference it uses.

### Gaps
- I did not find a framework document that treats "coming from X" guides, error indexes or migration guides as first-class types. The evidence for them is how leading language sites are organised, not theory.
- I found no usage data comparing the effectiveness of these components (for example, whether readers use error indexes heavily).

## Rules for code examples: completeness, runnable, copy-pasteable, output, length

### Takeaway
The best practice combines several sources:
- Each sample gets an introductory sentence (Google).
- Each sample is complete enough to run, with omissions shown as comments in the language, never as `...` (Google).
- Tutorials show the expected output (Diátaxis).
- Reference examples stay brief (Diátaxis).
- Samples are executed by the build so they cannot drift (Rust doctests), with hidden set-up lines so the shown part stays short.

### Cited Findings
- Tutorials must "maintain a narrative of the expected": "The output should look something like…" and "Notice that…". They aim at "perfect reliability", so every step works for every user. — [Tutorials](https://diataxis.fr/tutorials/)
- Reference gives brief usage examples that illustrate an item in context without turning into a tutorial. — [Reference](https://diataxis.fr/reference/)
- Write the Docs, "Exemplary": include examples for common use cases, "but not too many", kept separate from dense reference. — [Write the Docs](https://www.writethedocs.org/guide/writing/docs-principles/)
- Google's rules for samples — [Google: code samples](https://developers.google.com/style/code-samples):
  - an introductory sentence ending in a colon;
  - omissions as comments in the language, never `...`;
  - no click-to-copy on a block that omits code;
  - 80-column wrap;
  - the language's indentation conventions.
- Rust doctests — [rustdoc: documentation tests](https://doc.rust-lang.org/rustdoc/write-documentation/documentation-tests.html):
  - Every code block in the docs is compiled and run by `cargo test --doc`, "to make sure examples in your documentation stay up to date and working".
  - A sample passes if it compiles and doesn't panic, so `assert_eq!` doubles as shown output.
  - Lines starting with `#` are compiled but hidden, so the reader sees a focused excerpt of a complete program.
  - Attributes mark intent: `should_panic`, `no_run` (compiles but is not run, for network calls), `compile_fail` (must fail to compile) and `ignore`. A `text` block is preferred for content that is not code.
- Rust's std reference examples have a "Run" button that opens them in the Playground. — [doc.rust-lang.org](https://doc.rust-lang.org/)
- Rust's error pages pair an erroneous example with fixed examples. — [E0382](https://doc.rust-lang.org/error_codes/E0382.html)
- Microsoft: be brief, and lead with what matters. — [Microsoft: top 10 tips](https://learn.microsoft.com/en-us/style-guide/top-10-tips-style-voice)

### Inferences
Rules a WebFluent docs team could adopt:
1. Every ` ```wf ` block in the docs compiles in CI, through `wf check` on an extracted snippet or through `wf test`.
   - Snippets that are not whole files get an implied wrapper (as rustdoc wraps a sample in `fn main`) or hidden lines.
   - Intentionally broken samples carry a marker such as `compile_fail` with the expected diagnostic code. Every diagnostic-index page would use this for its erroneous example, so the code stays honest.
2. Show the output.
   - For WebFluent, "output" is rendered UI, so place a live render or screenshot beside the code. The existing `wf test` `expect "…"` steps can assert the visible text, as `assert_eq!` does in Rust.
   - For CLI commands, show the terminal output, as AGENTS.md already does for `wf verify` and `--stats`.
3. Use a copy button only on complete blocks. Mark omissions with `// …` comments, never bare `...`.
4. Length by mode:
   - Reference: the smallest call that shows the item, then one realistic variant.
   - How-to: the full working change.
   - Tutorial: incremental steps, each with the whole file state reachable.
5. Precede every sample with one sentence that says what it shows, ending in a colon.
6. Offer a "run in playground" link where possible. WebFluent compiles to JS, so an in-browser playground is feasible.

### Gaps
- I found no authoritative empirical study of ideal sample length. The guidance here is framework-level ("brief" in reference, "not too many" examples), not numeric.
- I did not reach a primary Google or Microsoft page on showing command output or on placeholder conventions in samples.
