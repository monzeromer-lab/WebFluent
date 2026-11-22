# Draft: a note to Nolan Lawson (not sent)

To send as a GitHub issue or discussion on `nolanlawson/agent-framework-benchmark`, or by
email, once you have read it and changed what you want.

---

**Subject:** Running agent-framework-benchmark on WebFluent — permission, and an offer

Hi Nolan,

I make WebFluent, a typed language that compiles to static sites, single-page apps and
PDFs. Your Pokédex benchmark is the fairest test I've seen of what a framework costs an
agent, so I'd like to run WebFluent through it rather than write a benchmark of my own.

What I've done so far, in a private checkout pinned at `b95ada0`, without publishing
anything:

- added `webfluent: 'WebFluent'` to `frameworks.mjs`, with our official skill in
  `setup-tooling.sh` installed the same way you install the others;
- installed the `wf` binary in the base image (a pinned release, checked against its
  SHA256SUMS), since there is no npm package to install;
- added one line to SPEC.md's Environment section saying `wf` is installed. Every
  framework sees that line;
- counted `.wf`/`.wfx` in `codeStats`;
- added a `--engine=docker` option, and made it refuse `ANTHROPIC_API_KEY`, because I'm
  running on a Max subscription as you did.

The whole change is one patch, about 200 lines, attached / linked.

Two questions:

1. **Licence.** The root `package.json` says Apache-2.0, but the repository has no
   LICENSE file. May I publish the patch and our results (transcripts included) under
   Apache-2.0, crediting you and linking your post?
2. **Would you rather run it yourself?** WebFluent in your own table, run by you, would
   mean more than any number I publish. I'm glad to send a PR with the change above and
   answer any question about the toolchain. If you'd rather not, I'll publish our run with
   your medians beside ours, labelled as yours and dated.

Whatever the result, I'll publish it, including a bad one.

Thanks for the benchmark, and for writing the method down so carefully.

— Monzer
