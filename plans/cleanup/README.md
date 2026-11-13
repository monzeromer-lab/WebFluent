# Cleaning up: what is unused, and what is safe to remove

Surveyed 2026-11-13, after the 6.0.0 release. Nothing here has been removed
yet. Recheck each item before acting, since the sizes and branch states
below are from that day.

## 1. Tracked files

Every tracked folder, script, test file, fixture and example was checked
against what references it: the justfile, CI and release workflows,
`xtask`, other scripts, `tests/it.rs`, the code and the docs. Almost
nothing is dead.

- **Scripts:** every one in `scripts/` is used by the justfile, CI, the
  release, `xtask`, or regenerates a checked-in file.
- **Tests:** every `tests/*.rs` file is in `tests/it.rs` (`autotests = false`).
- **Fixtures, examples and test data:** every fixture and example is built
  by a test. `tests/eval-corpus` is read by `src/template/corpus_tests.rs`.
- **`bench/pdf`:** the wf-vs-Puppeteer PDF benchmark, linked from the
  README and the justfile. Keep it.
- **`docs/`:** the built site that GitHub Pages serves. Keep it.

Worth a decision:

- **`spec/` (16 files).** Seven are referenced:
  - `SPEC.md`, from the README;
  - `DOCS_STYLE.md`, from the contributing guide;
  - `DIAGNOSTICS_PLAN.md`, `PDF_PLAN.md`, `PROJECT_SCRIPTS.md`,
    `SYNTAX_V2.md` and `TYPES.md`, from tests or code comments.

  Nothing references these nine, and most predate 5.0:
  - `ROADMAP_4.md` and `STUDIO_INTEGRATION_PLAN.md`;
  - `DESIGN.md` and `ANIMATION_SPEC.md`;
  - `ACCESSIBILITY_SPEC.md`, `I18N_SPEC.md`, `SLIDES_SPEC.md`,
    `SSG_SPEC.md` and `TEMPLATE_ENGINE_SPEC.md`.

  The guide (`md-docs/`) and AGENTS.md now cover what they describe. Skim
  each for intent the guide does not record, then delete it or move it to
  `spec/archive/`. `md-docs/52-contributing.md` describes `spec/` as
  "design notes and plans"; update that line if the folder changes.
- **`plans/refactoring/`:** the refactor is finished, so this is a record
  now. Keep it, or move it to `plans/archive/`. Its
  `security-and-memory.md` is confidential and kept out of git through
  `.git/info/exclude`; it exists only on disk.
- **`scripts/site-art.sh`:** only the release notes mention it, but it is
  the manual tool that regenerates the site's icons and link card from
  `site/art/`. Keep it.

## 2. On disk, not in git (about 240 GB)

| What | Size | Notes |
|---|---|---|
| `target/` | 60 GB | `cargo clean`; it rebuilds |
| `~/Code/wf-targets/` | 28 GB | Scratch build folders the agents used. `integ` is 19 GB; `prof`, `bench-prof`, `perf`, `bench`, `codegen-master`, `api`, `semver`, `tools` and `fuzz` are the rest |
| `.claude/worktrees/` | 50 GB | Six worktrees from other sessions; every branch is in master (see 3) |
| `~/Code/wf-worktrees/` | 38 GB | `layers`, `markdown`, `portal` and `testfix`, all merged |
| `~/Code/WebFluent-page-lang` | 8 GB | Merged; its 50 uncommitted edits are an earlier draft of what master has (see 3) |
| `~/Code/WebFluent-store-resource` | 5.5 GB | Master has both of its commits, rebased |
| `fuzz/target/` | 2 GB | Rebuilds |
| `build/` folders in fixtures and examples, `scripts/__pycache__`, `crates/wf-wasm/pkg*`, `.idea/` | ~20 MB | Regenerated or editor state |

Keep `fuzz/corpus/` (46 MB): it is what the fuzzers have learned.

The four `launch/*` worktrees on the external drive (`/run/media/.../Work/`:
`wf-bench`, `wf-launch`, `wf-showcase`, `wf-videos`) are merged too, but
they are on another disk. Decide those separately.

## 3. Git housekeeping

- **Worktrees.** Remove each with `git worktree remove <path>`, which
  refuses one with uncommitted changes; use `--force` only after checking
  them.
  - All of these are merged into master:
    - `claude/competent-lamport-0fae2d`, `claude/fervent-heisenberg-d60a70`,
      `claude/happy-mestorf-77d90f`, `claude/suspicious-chatelet-0d15ad`;
    - `claude/adoring-jepsen-07e74d`: every commit is in master, by patch;
    - the detached `exciting-goldwasser-3a2c7f`;
    - `wt/layers`, `wt/markdown`, `wt/portal`, `wt/testfix`;
    - `page-lang-translations`.
  - `store-resource`'s two commits ("A store owns its resource", "A first
    load drops the last answer…") are in master as `b27b724f` and
    `3ba4bc0c`.
  - `WebFluent-page-lang` has 50 uncommitted edits. Sampled, they are in
    master. The one thing master lacks is an old "5.14" upgrading heading,
    for the section that is now "6.0", so nothing is lost.
- **Branches:** 46 local branches are merged into master; delete them with
  `git branch -d` (which refuses an unmerged one). Delete each worktree's
  branch after its worktree.
- **Stash:** one entry, from another session before the refactor:
  `WIP on master: 72213418 Don't compress an editor's preview…`. Look
  at it with `git stash show -p stash@{0}`, then drop it or keep what it
  holds.
- **Master's working tree:** `plans/docs-redesign/README.md` is modified,
  and not by the session that wrote this. Ask before touching it.

## Order, when it is done

1. Remove the merged worktrees, then their branches. Recheck each with
   `git status` and `git cherry master <branch>` first.
2. Delete the other merged branches, and decide about the stash.
3. `cargo clean`, `rm -rf ~/Code/wf-targets`, `fuzz/target`, and the
   regenerable `build/` folders.
4. Decide `spec/` and `plans/refactoring/`, in a commit of their own.
5. The external drive's worktrees, if wanted.
