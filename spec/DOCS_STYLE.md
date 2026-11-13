# How the WebFluent documentation is written

> **Status: the house style of `md-docs/`, and the study it rests on.** Every
> chapter of the guide follows it; a change to the guide is reviewed against
> it. The site's chapter pages are generated from `md-docs/` by
> `scripts/docsite/guide.py`, so following this file is how the site is
> written too.

This file has three parts:

1. [What the study found](#1-what-the-study-found) — the practices the
   best developer-tool documentation shares, and where each comes from.
2. [The shape of the guide](#2-the-shape-of-the-guide) — how those
   practices map onto WebFluent's chapters, and a template for each kind of
   page.
3. [The style checklist](#3-the-style-checklist) — the rules a sentence, a
   heading or a code sample is held to.

---

## 1. What the study found

The study read the frameworks that the field holds up as models — the
Diátaxis framework, the Google and Microsoft developer style guides, Write
the Docs, and the documentation of Rust, React, Svelte, Vue, Django, Elm,
Go, Stripe, MDN and Astro — and kept what they agree on. Sources are listed
at the end of this part.

### 1.1 There are four kinds of documentation, and they must not be mixed

Diátaxis (Daniele Procida, drawn from the Django and Divio documentation)
separates documentation by the reader's need, on two axes: *acquiring*
skill versus *applying* it, and *practical* steps versus *theoretical*
knowledge.

| Kind | The reader is… | It is… | It must not… |
|---|---|---|---|
| **Tutorial** | learning, at study | a lesson: a guaranteed path to a working result | explain at length, offer choices, or stop to cover alternatives |
| **How-to guide** | working, with a goal | a recipe: the steps to one real-world result | teach the basics, or be a list of everything possible |
| **Reference** | working, needs a fact | a description of the machinery: complete, exact, in a predictable order | explain why, or instruct |
| **Explanation** | studying, wants to understand | a discussion: context, reasons, trade-offs, alternatives | give step-by-step instructions, or list every option |

Diátaxis calls the tutorial/how-to conflation "the single most common
conflation" in software documentation, and warns that the difference is not
basic versus advanced: a tutorial may be advanced, and a how-to guide may be
simple. The other usual failures are explanation creeping into a tutorial
("a tutorial is not the place for explanation" — give one clause and a
link) and reference that teaches (reference must "describe and only
describe", in a consistent pattern whose structure "mirrors the structure of
the product"). The cure is not to delete the material but to move it to a
page of its own kind, and link.

What each kind promises its reader, in Diátaxis's own terms: a tutorial
delivers "a visible result early and often", "every step the learner
follows should produce a comprehensible result", it shows the exact
expected output and flags the likely failure ("if you see …, you probably
forgot …"), and it "aspires to perfect reliability". A how-to guide is
written as conditional imperatives — "if you want x, do y" — and sends the
reader to reference for the full list of options. Explanation is the page
you could put "About" in front of: context, design decisions, trade-offs
and alternatives, and opinion is allowed.

Django's documentation states the same four parts on its front page
(tutorials, topic guides, reference, how-to guides) and tells its writers
that "reference guides aren't the place for general explanation". react.dev
splits *Learn* — where "pages only rely on concepts introduced by the earlier
pages", each opening with *You will learn*, ending with a *Recap*, and
setting pitfalls apart in typed *Pitfall* and *Deep dive* asides — from *API
Reference*, where each entry has a formal *Reference* section (parameters,
returns, caveats), then *Usage* scenarios, then *Troubleshooting* titled by
the symptom the reader sees. The Rust project keeps *The Book*, *Rust by
Example* and *The Reference* as separate works.

### 1.2 Voice: talk to the reader, in the present, actively

Google's and Microsoft's style guides agree on the voice of technical
writing:

- **Second person.** "You declare a page" — not "the user declares", not
  "we declare". The reader is the subject.
- **Active voice, present tense.** "The compiler reports `T05`" — not
  "`T05` will be reported". Say who does what.
- **Plain words, short sentences.** One idea per sentence. Prefer the
  common word ("use", not "utilise"). Avoid idiom, wordplay and words a
  translator or a non-native reader would stumble on.
- **No false comfort.** Drop *simply*, *just*, *easy*, *obviously* — they
  tell a reader who is stuck that the fault is theirs.
- **Conditions before instructions.** "To keep a value across visits, use
  `persist`" — the reader learns whether the sentence is for them before
  being told what to do.
- **Sentence-case headings**, and headings that say what the section is
  for. In a task, a verb: *Validate a field*. In reference, the thing's
  name. A heading is an address — readers scan them, search engines index
  them, and other pages link to them.
- **Descriptive link text.** "See [Forms](…)" names the destination; "click
  [here](…)" does not.

### 1.3 Every page answers the reader's first three questions at once

Documentation is the resource developers lean on most: in Stack Overflow's
2024 survey, 84% of developers learning to code used technical
documentation, more than any other resource. And it is where projects most
often fail them: in GitHub's 2017 Open Source Survey, incomplete or outdated
documentation was the problem most respondents (93%) had met, while 60% of
contributors rarely or never contribute to documentation; nearly a quarter
of respondents read English less than very well, which is the case for
plain language. Readers scan first and read second (Nielsen Norman Group's
eye-tracking studies), so
pages the reader can trust share one anatomy:

1. **What this is, in one or two sentences** at the very top — enough to
   decide whether to stay.
2. **What you will learn or do**, and **what you need first**
   (prerequisites), for tutorials and how-to guides.
3. **The smallest working example**, early. The reader sees the shape
   before the rules.
4. **Then the detail**, in sections a reader can jump to (the site's
   "On this page" rail is built from the `##` headings).
5. **Where to go next**: the related reference, the next task.

### 1.4 Code samples are the documentation

Stripe, Go and the Rust Book treat examples as the part a developer reads
first and copies. What makes one good:

- **Complete and runnable** — a reader who pastes it gets what the page
  says, with no hidden set-up. Where a fragment is clearer, it says that it
  is one.
- **Tested in CI.** Rust's documentation tests and Go's examples are run by
  the test suite, so they cannot rot. (WebFluent's guide does the same: see
  [2.5](#25-what-the-test-suite-holds-the-guide-to).)
- **Minimal** — the one idea, with no unrelated props, styling or state.
- **Realistic names** — `invoice`, `cart`, `draft`, never `foo`/`bar`.
- **The result shown** — what the page displays, what the command prints,
  the diagnostic the mistake draws.
- **Shell commands without a prompt character**, one per line, so they
  paste (MDN's rule; Google uses a prompt it makes unselectable — the point
  is to pick one convention). Output is shown apart from the command.
- **Introduced by a sentence** that says what the sample shows, and
  followed by the explanation of the lines that matter.

### 1.5 Errors are documentation too

Elm's compiler messages and Rust's error index (`rustc --explain E0308`)
set the standard: every diagnostic has a stable code; the message names
what went wrong *in the reader's program*, points at the place, and
suggests the fix; and the code links to a longer entry that explains the
cause, shows a program that triggers it, and shows the corrected program.
Rust fixes the order of a long explanation (RFC 1567): a one-sentence
description, an example of erroneous code, *why* it is an error, and the
corrected code — and its dev guide insists the entry explain the cause,
because "it is not helpful for users to copy-paste a quick fix". MDN's
JavaScript error pages run: the message, *What went wrong?*, examples of
bad and good code, *See also*. WebFluent's `wf explain CODE` prints exactly
the guide's entry for the code, so the entry *is* the help text.

### 1.6 Getting started is measured in minutes

A quick start succeeds when a new reader reaches a working result fast:
install, verify the install, make the smallest project, see it run, make
one change and see the change. Every prerequisite is stated up front, every
command is copyable, and every step says what the reader should now see.
Anything optional is pushed later.

### 1.7 The documentation is maintained like code

Write the Docs' *docs as code* — the docs live in the repository, are
reviewed in pull requests, are built and tested in CI, and are published
from the same commit as the code. Its principles for a documentation set:
**ARID** (accept some repetition: a page must stand alone), **skimmable**,
**exemplary** (examples for the common cases), **consistent**,
**current** (wrong documentation is worse than none), **discoverable**,
**addressable** (every section has a stable link), **cumulative** (each page
builds on the ones before it) and **complete**.

Two consequences matter most here: **URLs and anchors are stable** (a
renamed chapter keeps a redirect; the site has one for every move), and
**reference that can be generated is generated** (WebFluent's components,
configuration defaults, design tokens and diagnostics are read from the
compiler).

### 1.8 Asides, used sparingly

Admonitions — note, tip, warning — work when they are rare. Google's guide
says to minimise them and never stack them ("if you're unsure, write it as
plain text first"); React types its asides by role (*Pitfall*, *Deep
dive*) rather than by alarm. A page that is half callouts has none.

### Sources

- Diátaxis — <https://diataxis.fr/> (the four kinds; *tutorials*, *how-to
  guides*, *reference*, *explanation*; the compass)
- Django documentation, "How the documentation is organized" —
  <https://docs.djangoproject.com/en/stable/#how-the-documentation-is-organized>
- Google developer documentation style guide —
  <https://developers.google.com/style> (voice and tone, second person,
  active voice, present tense, headings, procedures, code samples, link
  text, notices)
- Microsoft Writing Style Guide — <https://learn.microsoft.com/style-guide/>
  (brand voice, scannable content, procedures, developer content)
- Write the Docs, "Docs as Code" and "Documentation principles" —
  <https://www.writethedocs.org/guide/docs-as-code/>,
  <https://www.writethedocs.org/guide/writing/docs-principles/>
- The Rust Programming Language — <https://doc.rust-lang.org/book/>;
  The Rust Reference — <https://doc.rust-lang.org/reference/>;
  rustdoc documentation tests —
  <https://doc.rust-lang.org/rustdoc/write-documentation/documentation-tests.html>;
  the Rust error index — <https://doc.rust-lang.org/error_codes/>
- react.dev — <https://react.dev/learn> and <https://react.dev/reference/react>
  (Learn versus Reference; *Pitfall* and *Deep dive* asides)
- Svelte tutorial — <https://svelte.dev/tutorial>
- Vue guide — <https://vuejs.org/guide/introduction.html>
- Elm guide and "Compiler errors for humans" —
  <https://guide.elm-lang.org/>, <https://elm-lang.org/news/compiler-errors-for-humans>
- Go documentation and testable examples — <https://go.dev/doc/>,
  <https://go.dev/blog/examples>
- Stripe API reference — <https://docs.stripe.com/api>
- MDN page templates — <https://developer.mozilla.org/en-US/docs/MDN/Writing_guidelines/Page_structures/Page_types>
- GitHub Open Source Survey (2017) — <https://opensourcesurvey.org/2017/>
- Stack Overflow Developer Survey 2024 —
  <https://survey.stackoverflow.co/2024/developer-profile/>
- RFC 1567, long error explanations —
  <https://github.com/rust-lang/rfcs/blob/master/text/1567-long-error-codes-explanation-normalization.md>;
  rustc dev guide on error codes —
  <https://rustc-dev-guide.rust-lang.org/diagnostics/error-codes.html>
- MDN code style guide — <https://developer.mozilla.org/en-US/docs/MDN/Writing_guidelines/Code_style_guide>
- Next.js contribution guide — <https://nextjs.org/docs/community/contribution-guide>
- Google Cloud, Effective Samples — <https://googlecloudplatform.github.io/samples-style-guide/>
- Nielsen Norman Group, "F-shaped pattern of reading on the web" —
  <https://www.nngroup.com/articles/f-shaped-pattern-reading-web-content/>
- Tim Berners-Lee, "Cool URIs don't change" — <https://www.w3.org/Provider/Style/URI>

---

## 2. The shape of the guide

### 2.1 The sections, and the kind each is

The guide is organised by what the reader is here to do. Each section is
one kind of page, and the sidebar shows sections, not chapter numbers. A
chapter's URL never changes when it moves between sections: released
compilers link to `/docs/guide/diagnostics#…`, and a cool URI does not
change.

| Section (`group:`) | Kind | Reader |
|---|---|---|
| Get started (`start`) | explanation, quick start, tutorial | new to WebFluent |
| Tutorials and recipes (`tutorials`) | tutorial; recipes are how-to | practising on a whole project |
| Concepts (`concepts`) | explanation, with a quick-reference table at the end | learning the language |
| Guides: interface, data, content, quality, shipping (`ui`, `data`, `content`, `quality`, `ship`) | how-to | building a feature, or putting it in front of people |
| Other outputs (`outputs`) | how-to | making a PDF, a deck, a server-rendered template, or working with JavaScript |
| Reference (`reference`) | reference | looking something up |
| Help (`help`) | troubleshooting, upgrade notes | stuck, or upgrading |

A chapter's `chapter:` in its front matter orders the guide, whatever
folder it is in; to move a chapter, renumber it and update the links to
it. The sections' names live in `GROUP_NAMES` in `scripts/docsite/guide.py`.

The documentation's landing page (`/docs`, `site/src/pages/Docs.wf`) routes
a reader by need — *Learn*, *Understand*, *Build*, *Look up*, *Fix* —
rather than by chapter number.

### 2.2 What every chapter has

```text
# N. Title                         ← numbered; sentence case

<!--                               ← metadata the generator and search read
route: guide/slug                  ← never changes once published
group: start|tutorials|concepts|ui|data|content|quality|ship|outputs|reference|help
blurb: one sentence under the title on the site
description: ≤ 158 characters, for the search result
-->

Opening: one or two sentences — what this is and why the reader cares.

(tutorials and how-to) What you will learn / do, and what you need first.

## Sections …

## Next                            ← the reader's next stops; the site draws
                                     its own previous/next cards instead
```

### 2.3 Templates

**Explanation chapter** (the concepts, and the explanatory openings of
others):

1. Opening: the idea in one or two sentences, and the one thing to remember.
2. *In this chapter* — a short list of what it covers.
3. The smallest example of the idea, then what each part of it does.
4. One section per concept, each: what it is → an example → how it behaves
   → the rules and limits → a `[!WARNING]` for the mistake people make,
   when there is one.
5. *Summary* or *Quick reference*: a table of the syntax the chapter
   introduced, linking to the reference.
6. *Next*.

**How-to chapter** (the guides and other outputs):

1. Opening: what the reader will be able to do.
2. *Before you start*, when there are prerequisites.
3. Sections titled with the task, in the imperative: *Validate a field*,
   *Deploy to GitHub Pages*. Each: one sentence of goal, the code or the
   numbered steps, what the reader sees as a result, then variations.
4. A checklist or *Troubleshooting* section for the job as a whole.
5. *Next*.

**Tutorial** (chapter 3 and the three project tutorials):

1. *What you will build* — the result, and how long it takes.
2. *Before you start* — what must be installed or known.
3. Numbered steps, `## Step N: verb phrase`. Each step: what and why in a
   sentence, the code, and **Check it:** what the reader sees now — and,
   where people go wrong, "if you see …, you probably …".
4. No detours: a link to the explanation chapter replaces the explanation.
5. *What you built* — a recap of the ideas used, each linked.

**Reference entry** (CLI, configuration, built-ins, runtime, tokens):
the name as the heading → one sentence of what it is → syntax or
signature → a table of its options with type, default and meaning → one
example → *See also*. Entries in alphabetical or grouped order, never in
the order they were added.

**Diagnostic entry** (the diagnostics reference): `### CODE — Title` (the heading is the
anchor the compiler links to; it must equal the code's title in
`src/diagnostics/codes.rs`) → what it means → a program that draws it
(`wf expect=CODE`) → the message the compiler prints (a `text` block the
test suite writes) → the fix, as a corrected program.

### 2.4 Asides

Asides are GitHub alerts; GitHub renders them, and the site generator turns
each into a `Callout` (`site/src/components/Callout.wf`):

```markdown
> [!TIP]
> `wf explain T05` prints this entry in your terminal.
```

| Kind | Use it for |
|---|---|
| `NOTE` | context the reader may want but can skip |
| `TIP` | a faster or better way to do what the page describes |
| `IMPORTANT` | something to know before going on, or the step fails |
| `WARNING` | a mistake that ships a bug or loses work |
| `CAUTION` | a mistake that loses data or exposes a secret |

At most one per section, never two in a row, never a code fence inside
one. If the paragraph would be fine as a paragraph, it is a paragraph.

### 2.5 What the test suite holds the guide to

The guide is tested like code (`tests/docs_parse.rs`,
`tests/guide_runs.rs`, `tests/tutorials.rs`, `tests/agent.rs`):

- Every ` ```wf ` block parses. A block of whole declarations is also
  type-checked and linted, with warnings counted as failures; one that
  leaves something out says so with a `…`.
- Every block that is a whole program with a `page` is **built and run**
  against a fake DOM: every button is clicked and every field typed into,
  and nothing may throw.
- A ` ```wf expect=CODE ` block must draw exactly `CODE`.
- Every `env.NAME` an example reads begins `PUBLIC_`.
- Every link to a chapter lands on the chapter and on a heading it has.
- The metadata is complete; the description fits a search result.
- The chapters that list everything — the declarations (language
  basics), keywords (grammar), built-ins, configuration keys and defaults,
  commands and flags (CLI), icons (media), tokens and diagnostics — are held to the
  compiler.
- The site's generated pages, sidebar and search index match what the
  scripts write from `md-docs/`.

---

## 3. The style checklist

**Words**

- [ ] Second person, present tense, active voice.
- [ ] No *simply*, *just*, *easy*, *quick*, *straightforward*, *merely*,
      *obviously*, *of course*, *note that*.
- [ ] *This* is followed by the noun it means ("this prop", not "this").
- [ ] One term per idea, used every time (see the
      [glossary](../md-docs/reference/glossary.md)): *page*, *element*,
      *component*, *prop*, *flag*, *case*, *block*, *state*, *derived
      value*, *action*, *store*, *pre-rendered page* (the *static paint*).
- [ ] A term the reader has not met is defined, or linked, the first time.
- [ ] *For example*, not *e.g.*; *that is*, not *i.e.*.
- [ ] Code, file names, commands, flags, keys and values in `backticks`;
      UI labels in **bold**.

**Headings**

- [ ] Sentence case. No heading without text under it.
- [ ] How-to sections start with a bare verb (*Validate a field*, not
      *Validating fields*); concept and reference sections are a noun
      phrase or the thing's name. No links in headings.
- [ ] Renaming a heading changes its anchor: update every link to it.

**Paragraphs and lists**

- [ ] The point first. A paragraph of more than five sentences is two.
- [ ] A list for three or more parallel items; a table when they share
      columns; numbered steps for a sequence the reader performs.
- [ ] One action per numbered step; the result after it.

**Code**

- [ ] Introduced by a sentence that says what it shows.
- [ ] Complete (`page Name(path: …, title: …, description: …)` with an
      `h1`, so it builds clean) unless a fragment is clearer — then `…`.
- [ ] Realistic names; no `foo`, `bar`, `test1`.
- [ ] Comments only for what the prose does not say.
- [ ] The output, page or diagnostic it produces, when that is the point.
- [ ] Shell blocks: commands only, no `$` prompt.

**Links**

- [ ] Descriptive text; never *here* or *this*.
- [ ] To the chapter that covers the idea, at its first mention in a
      section — not every mention.
- [ ] Relative links between chapters (`24-data.md#resource`), so they
      work on GitHub and the generator rewrites them for the site.
