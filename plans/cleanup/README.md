# Cleaning up: what was unused, and what was removed

Surveyed 2026-11-13, after the 6.0.0 release; carried out 2026-11-14.

## Done

- **Worktrees.** Every worktree in `.claude/worktrees/` and
  `~/Code/wf-worktrees/`, plus `WebFluent-page-lang` and
  `WebFluent-store-resource`, was removed. Before removal, each one's
  commits were checked against master by content, not just by hash. Four
  branches had commits that had been rebased or re-landed under other
  hashes:
  - `store-resource`, landed as `b27b724f` and `3ba4bc0c`;
  - the book rewrite, landed as `b7a98b0d` and `b6b98b11`;
  - the generated reference, landed as `39cbfadd`.

  The 50 uncommitted edits in `WebFluent-page-lang` were an earlier draft of
  what master has. The sessions that owned the `.claude` worktrees were
  archived first.
- **Branches.** The 52 merged branches were deleted, along with the five
  whose commits master has by content. Only `master` and the four `launch/*`
  branches remain.
- **Stash.** It was dropped. It held the reference's layout-example fixes
  and their test, both already in master.
- **Build caches.** `target/`, `~/Code/wf-targets/` and `fuzz/target/` were
  cleared; they rebuild. `fuzz/corpus/` was kept.
- **`spec/`.** The nine specs nothing referenced were deleted (they remain
  in git history): `ROADMAP_4`, `STUDIO_INTEGRATION_PLAN`, `DESIGN`,
  `ANIMATION_SPEC`, `ACCESSIBILITY_SPEC`, `I18N_SPEC`, `SLIDES_SPEC`,
  `SSG_SPEC` and `TEMPLATE_ENGINE_SPEC`. The guide and AGENTS.md cover what
  they described. The seven referenced specs stay.
- **`plans/refactoring/`** moved to `plans/archive/refactoring/`. Its
  `security-and-memory.md` is still confidential and still kept out of git
  through `.git/info/exclude`, under the new path.

## Kept, on purpose

- Every script, test, fixture and example: each one is used (see the
  survey in this file's history).
- `bench/pdf`, `docs/` (the built site) and `scripts/site-art.sh`.

- **The external drive's worktrees.** The four `launch/*` worktrees
  (`wf-bench`, `wf-launch`, `wf-showcase`, `wf-videos`) were removed on
  2026-11-14, along with their branches. Their links still pointed at the
  repository's old place on that drive, so `git worktree repair` came first.
  All four were merged with nothing uncommitted; only build output,
  dependencies and the PDF benchmark's last results (`bench/pdf/out/`) went
  with them.

The repository now has `master` and no other branch or worktree.
