# Developer reading research: how developers read and use documentation, and the known pitfalls

Scope: empirical evidence (academic studies, primary surveys, usability research) that should constrain the design of the WebFluent docs site (a guide of about 50 chapters, a component reference, a showcase and a home page). Researched 2026-11-11. Each finding is dated by its source year.

## How do developers read docs? (opportunistic vs systematic, scanning, code-reading)

### Takeaway
Developers move between docs and code, roughly half and half. Most read opportunistically: they come with a task, look for an example and skip prose. A large minority read systematically, looking for concepts and a linear path. Reference pages and recipe or sample pages get the most time; standalone concept pages get the least. On the web, people scan in an F-pattern, and experts read code even less linearly than novices. Docs therefore have to serve both strategies: task-organised, example-first pages with concepts woven in, plus a coherent linear path for those who want one.

### Cited Findings
- **Meng, Steinhardt & Schubert, "How Developers Use API Documentation: An Observation Study" (SIGDOC Communication Design Quarterly, 2019).**
  - Method: 11 developers worked through 5 API tasks, recorded with screencasts, eye-tracking and questionnaires.
  - Participants spent about 49% of their time in the documentation (range 31–68%) and about 51% in the editor or client.
  - Share of total time by page type: API reference 18.35%, recipes 14.99%, concepts 7.91%, samples 5.69%, welcome page 1.33%.
  - Concept-page use was bimodal. Five participants spent 0–5.6% of their time there; the other six spent 12.5–42.9%.
  - Results: 8 of 11 completed all tasks. Mean time per task was 695 s (SD 282).
  - Two strategies emerged. Opportunistic developers were exploratory and task-driven and took risks. Systematic developers took time to understand the API first.
  - Their opportunistic/systematic classification did not predict concept-page use. Of the systematic developers, 2 of 4 did not finish within 70 minutes.
  - Sample is small and the study exploratory, so treat the numbers as indicative only.
  — [Meng et al. 2019, SIGDOC CDQ](https://sigdoc.acm.org/cdq/how-developers-use-api-documentation-an-observation-study/); [PDF](https://mangold-international.com/resources/meng-et-al-how-developers-use-api-documentation-2019.pdf)
- **Meng et al.'s design recommendations (2019):**
  - Organise content by functionality or task, not by content type.
  - Integrate concepts into the tasks that need them instead of a separate section.
  - Keep navigation consistent and searchable.
  - Make code copy-ready, with no placeholders that point to other examples.
  - Repeat critical concepts as code comments, because opportunistic readers skip prose.
  - Visibly link text to the code elements it describes.
  - Enable fast try-outs (runnable samples).
  — [Meng et al. 2019](https://sigdoc.acm.org/cdq/how-developers-use-api-documentation-an-observation-study/)
- The personas go back to Clarke (2007): systematic, opportunistic and pragmatic developers. A commentary argues that authors tend to write for systematic readers even though opportunistic behaviour dominates. — [SIGDOC summary](https://sigdoc.acm.org/cdq/how-developers-use-api-documentation-an-observation-study/); [I'd Rather Be Writing, 2019](https://idratherbewriting.com/2019/03/07/how-to-design-api-documentation-for-opportunistic-behavior/)
- A follow-up paper, "Optimizing API documentation" (Meng et al., 2020), exists but was not read for these notes. — [PDF](https://mangold-international.com/resources/meng-et-al-optimizing-api-documentation-2020.pdf)
- **Robillard & DeLine, "A field study of API learning obstacles" (Empirical Software Engineering 16(6), 2011).**
  - Method: Microsoft developers across 3 phases, more than 440 in total.
  - Phase I: 80 respondents gave 165 free-form obstacles. Learning resources (documentation, examples) were the largest obstacle category, named by 50 of the 80 respondents. API structure came next (36), then background (17), technical environment (15) and process (13).
  - Phase II was 28 interviews. Phase III was a follow-up survey of 334 developers (17.3% response rate), in which documentation-related obstacles were rated the most severe.
  — [Robillard & DeLine 2011 PDF](https://www.cs.mcgill.ca/~martin/papers/ese2011.pdf)
- **Robillard & DeLine's implications for documentation design:**
  1. Document intent only where usage is not self-evident.
  2. Small examples that show a usage pattern of more than one call beat single-call examples. Participants found that very long examples lose teaching value because their concerns get tangled.
  3. Examples are read as recommended best practice ("this is how you *should* use it").
  4. Map scenarios to API elements, including "bridge" docs for people coming from other APIs.
  5. Explain performance, error handling and the API's abstract execution model.
  6. Navigating to a page is an implicit query, and boilerplate reference text wastes the reader's time.
  7. Some developers want a coherent, linear presentation, especially for overviews. Fragmented collections of hyperlinked articles overwhelm them.

  More than 25 participants commented on the role of examples. One participant said that good documentation "solves 99% of your problems." — [Robillard & DeLine 2011](https://www.cs.mcgill.ca/~martin/papers/ese2011.pdf)
- The precursor article, "What Makes APIs Hard to Learn? Answers from Developers" (IEEE Software, 2009), reported the preliminary survey of about 80 Microsoft developers. — [MSR page](https://www.microsoft.com/en-us/research/?p=354776); [Never Work in Theory summary](https://neverworkintheory.org/2011/11/08/a-field-study-of-api-learning-obstacles.html)
- **NN/g, F-shaped scanning.**
  - The original 2006 eye-tracking study recorded 232 users on thousands of pages and found consistent F-shaped scanning.
  - The 2017 follow-up found the pattern still present on both desktop and mobile. It is mirrored in right-to-left languages, which matters for WebFluent's Arabic support.
  - NN/g calls the pattern bad for users, because they miss content based only on its position in the column. Its remedy is to put the most important information in the first paragraphs, with meaningful headings and front-loaded subheads.
  — [NN/g, F-shaped pattern (2017)](https://nngroup.com/articles/f-shaped-pattern-reading-web-content/); [NN/g original (2006)](https://nngroup.com/articles/f-shaped-pattern-reading-web-content-discovered/); [NN/g text-scanning patterns](https://nngroup.com/articles/text-scanning-patterns-eyetracking/)
- **Busjahn et al., "Eye Movements in Code Reading: Relaxing the Linear Order" (ICPC 2015).**
  - Novices' eye movements were about 80% linear on prose and 70% on code.
  - Experts' were only about 60% linear on code; they jump between lines.
  - A 2020 ICPC follow-up found that how linear the code itself is drives reading order more than experience does.
  — [Busjahn et al. 2015 PDF](https://huang.isis.vanderbilt.edu/cs8395-f23/paper/linear-order-eye.pdf); [Peitek et al., ICPC 2020](https://www.se.cs.uni-saarland.de/publications/docs/PSA20.pdf)

### Inferences
- Reference and recipe pages carry most of the reading time, and concept chapters the least. The ~50-chapter guide should therefore lead each chapter with a working example and a task framing. Concept explanation should sit inline, next to the code it explains, not in separate "theory" chapters.
- A bimodal audience means both a linear path (the guide read in order, "next chapter" links) and random access (reference, search, deep links) are first-class. Robillard's Implication 7 and Meng's opportunistic readers point the same way.
- Because readers skip prose, the critical rules (for example: flags go after the parentheses; statements are separated by newlines, not commas) should also appear as comments inside the code examples.
- Headings and the first sentence of each section must carry the meaning (F-pattern). On mobile the same F-scan holds, so the first screen of each page matters even more.

### Gaps
- I did not retrieve per-group quantitative contrasts beyond Meng's participant table. No large-N study gives the population split between opportunistic and systematic readers.
- NN/g's oft-quoted 1997 figure that "79% of users scan" was not verified here, so it is not cited.
- No study found here measures doc-reading behaviour specifically on mobile devices for developer docs.

## What do developers say they want? (surveys)

### Takeaway
Across a decade of surveys, documentation is developers' most-used learning resource and its absence or poor quality is their top complaint. The specific complaints are content problems: it is incomplete, outdated, ambiguous or wrong. Presentation problems rank lower. Distrust of AI-generated answers is rising, which raises the value of authoritative, correct docs.

### Cited Findings
- **Stack Overflow Developer Survey 2024** (more than 65,000 respondents, May 2024):
  - Technical documentation was the top online resource for learning to code, at 83.9%, ahead of Stack Overflow at 80.3%.
  - 82% of developers learn with online resources.
  - API/SDK documents were reported as the documentation source of choice for 90%.
  — [SO Survey 2024, developer profile](https://survey.stackoverflow.co/2024/developer-profile/); [SO blog summary](https://stackoverflow.blog/2024/07/24/developers-want-more-more-more-the-2024-results-from-stack-overflow-s-annual-developer-survey)
- **Stack Overflow Developer Survey 2025** (more than 49,000 responses from 177 countries):
  - More developers distrust the accuracy of AI tools (46%) than trust it (33%). Only 3% highly trust it.
  - 66% say AI solutions are "almost right, but not quite." 45% say debugging AI-generated code takes more time.
  - Secondary sources disagree on the 2024 trust baseline (40% or 43%), and one gives 64% instead of 66% for "almost right."
  — [TechRepublic](https://www.techrepublic.com/article/news-developer-trust-in-ai-declines/); [VentureBeat](https://venturebeat.com/ai/stack-overflow-data-reveals-the-hidden-productivity-tax-of-almost-right-ai-code); [ADMIN Magazine](https://www.admin-magazine.com/News/Stack-Overflow-Survey-66-of-Developers-Frustrated-by-AI-Inaccuracy)
- **GitHub Open Source Survey 2017** (about 5,500 randomly sampled respondents from more than 3,800 projects):
  - 93% reported incomplete or outdated documentation as a pervasive problem. Some outlets paraphrase it as "incomplete or confusing."
  - 60% of contributors rarely or never contribute documentation.
  — [opensourcesurvey.org/2017](https://opensourcesurvey.org/2017/); [SD Times](https://sdtimes.com/github-releases-2017-open-source-survey); [InfoQ](https://www.infoq.com/news/2017/06/github-survey-open-source)
- **Postman State of the API:**
  - 2020: lack of documentation was the top obstacle to consuming APIs, at 54.3%, by a wide margin. Lack of knowledge, complexity and lack of time were each around one third.
  - 2022: a bare majority again named lack of documentation as the number-one obstacle, with discovery at 38% and lack of knowledge at 30%.
  - 2024: 39% named inconsistent docs as the biggest roadblock (internal collaboration framing).
  - 2025: documentation and discovery are still described as major headaches; no percentage was found.
  — [Nordic APIs on 2020](https://nordicapis.com/breaking-down-postmans-2020-state-of-the-api-report/); [HackerNoon](https://hackernoon.com/54percent-of-developers-cite-lack-of-documentation-as-the-top-obstacle-to-consuming-apis-2v4w3e30); [The New Stack on 2022](https://thenewstack.io/state-of-the-api-microservices-gone-macro-and-zombie-apis/); [BusinessWire 2024](https://www.businesswire.com/news/home/20241015081863/en/Postmans-2024-State-of-the-API-Report-Finds-API-First-Approach-Yields-Tangible-Results); [Nordic APIs on 2025](https://nordicapis.com/a-deep-dive-into-the-state-of-the-api-2025/)
- **Uddin & Robillard, "How API Documentation Fails" (IEEE Software 32(4), 2015).**
  - Method: two surveys of 323 professional developers in total, plus analysis of 179 documentation units.
  - The three severest problems were ambiguity, incompleteness and incorrectness.
  - Respondents ranked ambiguity as a "Top" priority 51.9% of the time. For ambiguity, 26.5% had seen it "Frequently" in the past three months.
  - Six problems were rated "Blocker", meaning respondents switched to another API: incompleteness, ambiguity, obsoleteness, incorrectness, inconsistency and unexplained examples.
  - Incorrect content was seen rarely but rated severe when it was seen.
  - Respondents prioritised five content problems over any presentation problem. Bloat, fragmentation, tangled information and excess structural information ranked lower; no one chose "tangled information" or "excessive structural information" as a top problem.
  - No significant difference was found between novices and experienced respondents.
  — [Uddin & Robillard 2015 PDF](https://www.cs.mcgill.ca/~martin/papers/ieeesw2015.pdf)
- **Aghajani et al.**
  - "Software Documentation Issues Unveiled" (ICSE 2019) mined 878 documentation-related artifacts (mailing lists, Stack Overflow, issues, pull requests) into a taxonomy. Its top-level groups are information content (what), information content (how it is written), process-related and tool-related issues.
  - "Software Documentation: The Practitioners' Perspective" (ICSE 2020) validated it with 146 practitioners (125 from one multinational).
  — [ICSE 2019 PDF](https://emadpres.github.io/pdfs/icse2019.pdf); [Never Work in Theory summary](https://neverworkintheory.org/2021/10/06/software-documentation-issues-unveiled.html); [ICSE 2020 page](https://2020.icse-conferences.org/details/icse-2020-papers/28/Software-Documentation-The-Practitioners-Perspective)

### Inferences
- For WebFluent, correctness and completeness come before polish. A wrong example is a "blocker" for a reader. Because WebFluent ships a compiler, every code sample in the guide and reference should be compiled and tested in CI so that docs and compiler cannot drift. Obsoleteness and incorrectness are the classic failure modes.
- Rising distrust of AI answers ("almost right") is an opportunity. An authoritative reference whose examples are verified by the compiler is exactly what developers fall back on. Machine-readable outputs such as AGENTS.md and `wf registry --json` also help AI tools to be right rather than almost right.
- Presentation fixes such as reducing bloat should not displace content fixes such as filling gaps.

### Gaps
- I could not retrieve the 2020 Aghajani ranking percentages (the PDF returned 403), so the relative ranking of issue types by practitioners is not reported here.
- The State of JS and State of CSS surveys have documentation-related questions, but none were retrieved.
- Stack Overflow 2025's own page on documentation and learning resources was not fetched; the 2025 figures come from secondary press coverage.

## How people learn a new programming language specifically

### Takeaway
Experienced developers learn a new language on their own and opportunistically, by mapping it to languages they know. That causes interference: 61% of cross-language Stack Overflow questions sampled contained wrong assumptions carried over from another language. Learners want cheat sheets and side-by-side comparisons. Data from the Rust Book experiment shows steep early drop-off: most readers leave in the first chapters, and only 2% reach chapter 19. Targeted fixes to hard spots raised quiz scores by about 20%.

### Cited Findings
- **Shrestha, Botta, Barik & Parnin, "Here We Go Again: Why Is It Difficult for Developers to Learn Another Programming Language?" (ICSE 2020, ACM SIGSOFT Distinguished Paper; republished in CACM 65(3), 2022).**
  - Method: 450 Stack Overflow posts across 18 languages, and 16 interviews with professionals who had recently switched language.
  - 276 posts (61%) contained incorrect assumptions due to interference from a previous language; 39% contained correct assumptions.
  - Professionals learned new languages on their own with no formal training, using an opportunistic, just-in-time strategy that related the new language to ones they knew. One interviewee: "I didn't learn typescript step-by-step."
  - Cheat sheets were preferred for transitions; an interviewee said "If you can compare them side by side … you're more than halfway there."
  - The interference themes were: old habits die hard; mindshifts when switching paradigms; little or no mapping to previous languages; finding the right terms and documentation is hard (a vocabulary barrier to even searching); and retooling (IDE and build) is a hard first step.
  - Docs were found incomplete or shallow. Of SAS documentation, one interviewee said it only shows how to "copy-paste and run a simple program," leaving the execution model a mystery.
  - Design Implication I: documentation should reduce interference and support knowledge transfer, for example cheat sheets that map a familiar language to the new one.
  — [Shrestha et al. 2020 (NSF PAR copy)](https://par.nsf.gov//servlets/purl/10139116); [MSR page](https://www.microsoft.com/en-us/research/publication/here-we-go-again/); [CACM 2022](https://cacm.acm.org/?p=124415)
- The same group's earlier work, "It's Like Python But: Towards Supporting Transfer of Programming Language Knowledge" (VL/HCC 2018), proposes tooling for mapping from one language to another. Norvig's "Python for Lisp Programmers" is cited as a model resource. — [arXiv/ar5iv](https://ar5iv.arxiv.org/html/1808.09008); [Shrestha et al. 2020](https://par.nsf.gov//servlets/purl/10139116)
- **Crichton & Krishnamurthi, "Profiling Programming Language Learning" (OOPSLA 2024, Distinguished Paper)**, on an interactive Rust Book with embedded quizzes:
  - 62,526 readers answered 1,140,202 times over 13 months (Sept 2022 to Oct 2023), with 88 quizzes and about 213 questions.
  - Drop-off:
    - Readers below the median of 6 answers ("dabblers") mostly stopped in chapter 1.
    - Those with 6 or more ("triers", 33,013 readers) mostly stopped in chapters 3–4, and chapter 4 (ownership) was the main drop point.
    - After that, drop-off was roughly uniform.
    - Only 1,220 readers (2%) answered a question in chapter 19.
    - Even the most active readers answered fewer than 100 questions, under half the book.
  - Question quality:
    - Concept questions ("why doesn't this compile?") discriminated ability best. Plain "does it compile?" type-error questions were among the least discriminating.
    - Discrimination ranged from r = 0.06 to 0.40.
    - A question with under about 20–30% correct likely signals a problem with the question or the explanation.
  - Interventions: 12 interventions rewrote the text at hard spots. 10 of 12 had a significant effect, averaging +20% on the targeted questions.
  - Engagement: readers retried a missed question 55% of the time when offered, and filed 2,026 bug reports that were used to fix the text.
  - Cost: authoring took about 50 person-hours and hosting about $15 a month. Cookie UUIDs were used with no accounts.
  — [arXiv 2401.01257](https://arxiv.org/html/2401.01257v1); [Brown CEL page](https://cel.cs.brown.edu/paper/profiling-pl-learning/); [Interactive Rust Book](https://rust-book.cs.brown.edu/)
- The authors argue that the technique generalises to languages with smaller user bases, based on simulations of their statistical inferences on smaller samples. — [Brown CS](https://cs.brown.edu/~sk/Publications/Papers/Published/ck-prof-prog-lang-learn/)

### Inferences
- WebFluent's audience is mostly people who already know HTML, CSS, JS, React, Vue or Svelte, so interference is the main risk. Examples: expecting JSX-style attributes, `onClick=` (refused, E106), commas between statements (E002), or `.flag` placed before the closing parenthesis. A "WebFluent for React/Vue/Svelte developers" cheat sheet with side-by-side mappings, plus a "things that work differently" page, is directly supported by the evidence. The existing "mistakes made most" list in AGENTS.md is the seed for it.
- Expect most guide readers to leave in the first few chapters. The home page and chapters 1–3 must deliver a working, rewarding result fast. The hardest concept (likely reactivity and stores, or the registry and flags model) should be positioned and explained with extra care, since that is where the Rust Book lost "triers".
- Embedded comprehension checks are cheap (about 50 hours, about $15 a month) and give data on where explanations fail. Concept questions beat syntax questions.
- The vocabulary barrier to search argues for synonym-aware search (for example "useState" finding `state`, "computed" finding `derived`, "props" finding component props).

### Gaps
- No peer-reviewed data was retrieved on the Go Tour or Gleam's language tour (completion rates, effectiveness). They are well-known onboarding patterns, but I found no empirical numbers.
- The specific content of Crichton's 12 interventions was commented out of the arXiv HTML version I read.

## Effects of interactive examples, runnable code and error messages

### Takeaway
Runnable, copy-ready examples are recommended by observation studies and interactive quizzes measurably help. The evidence that "enhanced" compiler error messages help is mixed. The best evidence (Denny, Prather, Becker et al. 2020) shows that messages written to readability guidelines shorten novices' debugging time.

### Cited Findings
- Meng et al. recommend fast try-outs (runnable samples, in-page requests) and complete, copy-ready code without cross-referenced placeholders. — [Meng et al. 2019](https://sigdoc.acm.org/cdq/how-developers-use-api-documentation-an-observation-study/)
- In the Rust Book experiment, interactive quizzes gave immediate feedback, readers retried 55% of missed questions, and revisions guided by the data improved targeted scores by about 20%. — [Crichton & Krishnamurthi 2024](https://arxiv.org/html/2401.01257v1)
- **Error messages, evidence mixed.**
  - Becker et al. (2016, Computer Science Education 26(2-3)) studied enhanced compiler error messages for novices.
  - Prather and colleagues note that the published experiments on enhanced messages reach conflicting conclusions. Their possible explanations include that students do not read messages and that effects are hard to measure.
  - One controlled study found no significant effect from enhanced messages with concrete examples.
  - Denny, Prather, Becker et al. (ITiCSE 2020, "Error Message Readability and Novice Debugging Performance") found that messages following readability guidelines gave significantly shorter debugging times and higher perceived usefulness for students very early in learning a new language.
  — [Becker et al. 2016](https://eprints.teachingandlearning.ie/id/eprint/5118/); [Denny et al. 2020](https://scispace.com/papers/error-message-readability-and-novice-debugging-performance-742o5sde3a); ["Compiler Error Messages Considered Unhelpful" (Becker et al., ITiCSE WG 2019)](https://bohrium.dp.tech/paper/arxiv/5eda197c9fced0a24b932b00)
- Uddin & Robillard list "unexplained examples" (code given without explanation) among the six problems practitioners rated as blockers. — [Uddin & Robillard 2015](https://www.cs.mcgill.ca/~martin/papers/ieeesw2015.pdf)

### Inferences
- WebFluent already has structured diagnostics with codes, hints, `wf explain` and doc links. Each code's docs entry (meaning, example, fix) makes the error message itself part of the documentation, and the readability guidelines from Denny et al. should govern how those entries are written.
- Runnable examples (a playground, or "copy, then `wf serve`") should be complete programs, not fragments that rely on context set up elsewhere.

### Gaps
- Elm's "compiler errors for humans" and Rust's error-message design are widely cited as models, but I retrieved no controlled study of their effect, only design essays (not fetched here).
- No study was found that directly measures the learning effect of an in-browser playground in programming-language docs (as opposed to quizzes).

## Known pitfalls

### Takeaway
The documented pitfalls, in order of severity, are content problems: incompleteness, ambiguity, incorrectness and obsolescence. After those come unexplained examples, fragmentation, assuming knowledge (no mapping from prior languages, no execution model), search and vocabulary failures, and docs organised by content type rather than task. Bloat and over-structure matter less but still count.

### Cited Findings
- **Incomplete, outdated, ambiguous or incorrect docs:**
  - 93% of open-source respondents report incomplete or outdated docs (GitHub 2017).
  - Ambiguity, incompleteness and incorrectness are the severest problems, and six problem types are "blockers" (Uddin & Robillard 2015).
  — [GitHub OSS Survey 2017](https://opensourcesurvey.org/2017/); [Uddin & Robillard 2015](https://www.cs.mcgill.ca/~martin/papers/ieeesw2015.pdf)
- **Fragmentation:** fragmented collections of hyperlinked articles overwhelm developers who want a linear overview (Robillard & DeLine, Implication 7). — [Robillard & DeLine 2011](https://www.cs.mcgill.ca/~martin/papers/ese2011.pdf)
- **Boilerplate reference:** navigating to a page is an implicit query, and boilerplate member-level text wastes time (Implication 6). — [Robillard & DeLine 2011](https://www.cs.mcgill.ca/~martin/papers/ese2011.pdf)
- **Organising by content type rather than task, and separating concepts from the code that needs them** (Meng et al. 2019). — [Meng et al. 2019](https://sigdoc.acm.org/cdq/how-developers-use-api-documentation-an-observation-study/)
- **Assuming knowledge, and no mapping from prior languages:** 61% interference rate in cross-language posts. Shallow docs that do not explain the execution model leave readers "completely mystified" (SAS example). — [Shrestha et al. 2020](https://par.nsf.gov//servlets/purl/10139116)
- **Search and vocabulary failures:** newcomers do not know the language's terms, so they cannot search for them; "you don't even know what exists, what to even look for." — [Shrestha et al. 2020](https://par.nsf.gov//servlets/purl/10139116)
- **Examples too long or too trivial:** an example's teaching value drops as it grows and tangles concerns, but single-call examples are too simple. — [Robillard & DeLine 2011](https://www.cs.mcgill.ca/~martin/papers/ese2011.pdf)
- **Important content buried below the fold or mid-paragraph:** F-pattern scanning misses it, on desktop and mobile alike. — [NN/g 2017](https://nngroup.com/articles/f-shaped-pattern-reading-web-content/)
- **Early drop-off at a hard concept:** the Rust Book lost most "triers" in chapters 3–4 (ownership). — [Crichton & Krishnamurthi 2024](https://arxiv.org/html/2401.01257v1)
- **AI-generated or AI-mediated answers:** 66% say AI answers are "almost right," and distrust (46%) exceeds trust (33%) (Stack Overflow 2025). — [TechRepublic](https://www.techrepublic.com/article/news-developer-trust-in-ai-declines/)
- **Few people write docs:** 60% of OSS contributors rarely or never write documentation, so docs decay unless writing them is built into the process. — [ADTmag on GitHub 2017](https://adtmag.com/Articles/2017/06/05/open-source-survey.aspx?m=1&p=1)

### Inferences
- These are design constraints for the WebFluent site:
  1. Compile-test every snippet, which defeats incorrectness and obsolescence.
  2. Make each reference entry answer its implicit query in the first lines: what it is, a minimal example, then props and flags.
  3. Give the guide an explicit linear spine with "next" navigation, and keep it random-access too.
  4. Write a migration and cheat-sheet chapter for React, Vue and Svelte users.
  5. Use search that knows other frameworks' vocabulary.
  6. Keep pages task-scoped, so long reference material is split by component rather than one giant page. AGENTS.md as a single file serves agents, not human scanners.
  7. Make the first three chapters short and rewarding, and put the hardest concept (reactivity) where readers are invested, with checks for understanding.
- AI-generated docs are a specific risk: they would reproduce "almost right" content, which is exactly the ambiguity and incorrectness practitioners rate most severe.

### Gaps
- No peer-reviewed measurement of page length against comprehension or abandonment for developer docs was found. "Too-long pages" is supported only indirectly (Robillard on long examples, NN/g on scanning).
- No empirical data was found on developer docs' mobile usage share or mobile-specific failures.
- No empirical study was retrieved on the quality of AI-generated documentation itself, as opposed to AI coding answers.
- The Diátaxis framework (which warns against mixing tutorials, how-to guides, reference and explanation) is widely adopted practice, but I found no empirical evaluation of it, so it is not cited as evidence.
