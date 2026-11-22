# Agent benchmarks: what it costs an agent to build the same app

One of the benchmarks [`bench/README.md`](../README.md) lists: what Claude
Code spends to build the same app in WebFluent.

The agents run in Nolan Lawson's
[agent-framework-benchmark](https://github.com/nolanlawson/agent-framework-benchmark),
at the commit [`upstream.json`](upstream.json) pins, with one patch
([`patches/0001-webfluent.patch`](patches/0001-webfluent.patch)) on top:

- **WebFluent as a framework.** It gets the label `WebFluent`. Its tooling is
  the official skill (`skills/writing-webfluent`), installed up front, the same
  way his harness installs the other frameworks' skills. The `wf` release named
  by `--wf-version` is installed in the base image, checked against the release's
  `SHA256SUMS`, so the tests and the measurement can build the app.
- **One line in `SPEC.md`'s Environment section:** "The WebFluent compiler,
  `wf`, is installed." This is the only change to the spec, and every framework
  sees it.
- **`.wf` and `.wfx` count as source lines**, and the `wf` version is recorded
  beside the npm versions.
- **Subscription only:** the harness refuses `ANTHROPIC_API_KEY` and takes
  `CLAUDE_CODE_OAUTH_TOKEN`.
- **`--engine=docker`** besides Podman, and **`--images-only`**, which builds
  the containers without running an agent.

Neither the harness nor its runs are committed: `setup.sh` fetches the harness
into `.upstream/` and Lawson's published results into `.lawson-results/`.
Batches go in `runs/`.

## Running a batch

Once:

```bash
bench/agents/setup.sh                         # the harness, patched, and Lawson's results
claude setup-token                            # prints a token tied to your subscription
# keep it in CLAUDE_CODE_OAUTH_TOKEN, or in ~/.config/webfluent-bench/oauth-token (chmod 600); never ANTHROPIC_API_KEY
```

Needs Podman 4.3 or later (rootless, as Lawson ran it) or Docker usable
without `sudo`, Node 20+, and about 15 GB of disk (the Pokémon data is about
2 GB, in the base image).

Then:

```bash
just bench-agents-plan dry webfluent,astro 2  # a batch: frameworks × runs, shuffled
just bench-agents-images dry                  # build the containers
just bench-agents-run dry                     # run it; safe to stop and start again
just bench-agents-measure dry                 # Lighthouse and lines of code, passing apps only
just bench-agents-summary dry                 # runs/dry/results.json
```

`queue.mjs` runs one session at a time, in the order fixed when the batch was
planned. A session that the usage limit stopped is **not a result**: it moves
to `runs/<batch>/_lost/`, the queue waits for the limit to reset (or 30 minutes
when the message doesn't say when), and the session runs again. A session that
ends on its own counts, pass or fail. So does one the harness stopped at its
time limit, which is how Lawson counted those. A refused token stops the batch.
Every start, finish and limit is a line in `runs/<batch>/events.jsonl`, and
that file is where "how many sessions fit in a usage window" is read from.

`summarize.mjs` writes `results.json`: the machine, the versions, every
session, and for each framework the median, the IQR and a 95% bootstrap interval
of tokens (split: input, cache writes, cache reads, output), API-equivalent cost,
time, turns, tool calls, lines and Lighthouse. It also gives the pass rate with
its Wilson interval, and Lawson's published medians beside ours. He published
medians, not individual runs, so a baseline is compared median to median.

The defaults are Lawson's published setup: `claude-opus-5-5`, effort `high`,
Claude Code 2.1.310.
