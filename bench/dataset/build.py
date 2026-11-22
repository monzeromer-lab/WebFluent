#!/usr/bin/env python3
"""Writes the WebFluent code dataset (for Hugging Face) to target/hf-dataset/.

Two tables:

  data/projects.jsonl  one record per whole project: every source file of it,
                       and whether `wf check` passes on it with the release
                       named below. The repository's examples/, and the
                       Pokédex apps agents built in bench/agents/ (each
                       passed the benchmark's 54 hidden tests).
  data/examples.jsonl  one record per `wf` block of the guide, the reference,
                       the skill and AGENTS.md: the examples the repository's
                       tests parse and check.

Nothing that isn't WebFluent source goes in: no build output, no
dependencies, no benchmark spec, tests, design or Pokémon data.

Usage: python3 bench/dataset/build.py WF_BINARY
"""

import hashlib
import json
import os
import pathlib
import re
import shutil
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[2]
CODE = ROOT.parent
# Every place WebFluent was written: (the source's name, its folder, what it is).
SOURCES = [
    ("webfluent/examples", ROOT / "examples", "the WebFluent repository's examples"),
    ("webfluent/site", ROOT / "site", "WebFluent's documentation site, written in WebFluent"),
    ("webfluent/starters", ROOT / "src" / "tools" / "init", "the projects `wf init` starts from"),
    ("webfluent/compiler-tests", ROOT / "tests", "the WebFluent compiler's test projects (some fail on purpose)"),
    ("halyard", CODE / "halyard", "Halyard, a project written in WebFluent"),
    ("zero-state", CODE / "zero-state", "Zero State, a project written in WebFluent"),
    ("studio-testing", CODE / "studio-testing", "a project made with WebFluent Studio"),
    ("studio/app", CODE / "webfluent-mobile-editor" / "app", "WebFluent Studio's own interface, written in WebFluent"),
    ("studio/tests", CODE / "webfluent-mobile-editor" / "tests", "WebFluent Studio's test projects, fixtures and agent evaluations"),
    ("webfluent-apps", CODE / "webfluent-apps", "an app written in WebFluent from a brief"),
]
# Where the generated tables come from (pairs.py).
EVALS = CODE / "webfluent-mobile-editor" / "tests" / "agent-evals" / "tasks"
FIXTURES = CODE / "webfluent-mobile-editor" / "tests" / "fixtures"
APPS = CODE / "webfluent-apps"
# The WebFluent a source was written for, when it is not the current one. Every source
# here is migrated to the current release, so none.
WRITTEN_FOR = {}
# Prose with `wf` examples beside the reference. WebFluent Studio's older notes
# (wf-studio) are left out: their examples are written for WebFluent before 7.
EXAMPLE_SOURCES = []
# What a person's own details become in the published copy.
REDACT = [
    ("monzer.a.omer@gmail.com", "owner@example.com"),
    # Halyard's demo settings page holds a made-up key shaped like a live Stripe one;
    # the published copy says it is an example, so no secret scanner takes it for one.
    ("sk_live_9f21c0b4d8e34Kd9", "sk_live_EXAMPLE0000004Kd9"),
]
OUT = ROOT / "target" / "hf-dataset"
WF = sys.argv[1] if len(sys.argv) > 1 else "wf"
SOURCE = re.compile(r"\.(wf|wfx|css|js|md)$")
SKIP_DIRS = {"node_modules", "build", "dist", ".wf-cache", ".git", ".claude", ".agents", ".wf", "__snapshots__", "public", "target", "src-tauri", "worktrees", "android", ".svelte-kit", "docs"}
SKIP_FILES = {"AGENTS.md", "package-lock.json"}


def wf_version():
    return subprocess.run([WF, "--version"], capture_output=True, text=True).stdout.strip().replace("wf ", "")


def files_of(project):
    out = []
    for p in sorted(project.rglob("*")):
        rel = p.relative_to(project)
        if not p.is_file() or any(part in SKIP_DIRS for part in rel.parts) or p.name in SKIP_FILES:
            continue
        keep = SOURCE.search(p.name) or p.name == "webfluent.app.json" or (
            "translations" in rel.parts and p.suffix == ".json"
        )
        if not keep:
            continue
        text = redact(p.read_text(encoding="utf-8", errors="replace"))
        out.append({"path": str(rel), "text": text})
    return out


def redact(text):
    for a, b in REDACT:
        text = text.replace(a, b)
    return text


def walk_files(root):
    for dirpath, dirnames, filenames in os.walk(root):
        dirnames[:] = sorted(d for d in dirnames if d not in SKIP_DIRS)
        for name in sorted(filenames):
            yield pathlib.Path(dirpath) / name


def check_copy(files):
    import tempfile
    with tempfile.TemporaryDirectory() as tmp:
        for f in files:
            p = pathlib.Path(tmp) / f["path"]
            p.parent.mkdir(parents=True, exist_ok=True)
            p.write_text(f["text"])
        return check(pathlib.Path(tmp))


def check(project):
    run = subprocess.run([WF, "check", "-d", str(project)], capture_output=True, text=True)
    return run.returncode == 0


def projects():
    records = []
    for key, root, what in SOURCES:
        if not root.is_dir():
            continue
        configs = sorted(p.parent for p in walk_files(root) if p.name == "webfluent.app.json")
        for project in configs:
            records.append({
                "id": f"{key}/{project.relative_to(root)}".rstrip("/."),
                "key": key,
                "source": what,
                "kind": "project",
                "project": project,
                "check": True,
                "meta": {},
            })
        # `.wf` files under no project of their own: a folder of them is one record.
        loose = {}
        for f in walk_files(root):
            if f.suffix in (".wf", ".wfx") and not any(c in f.parents for c in configs):
                loose.setdefault(f.parent, []).append(f)
        for folder in sorted(loose):
            records.append({
                "id": f"{key}/{folder.relative_to(root)}".rstrip("/."),
                "key": key,
                "source": what,
                "kind": "files",
                "project": folder,
                "only": loose[folder],
                "check": False,
                "meta": {},
            })
    runs = ROOT / "bench" / "agents" / "runs"
    for results in sorted(runs.glob("*/results.json")):
        batch = results.parent.name
        data = json.loads(results.read_text())
        for s in data.get("sessions", []):
            app = results.parent / s["dir"] / "repo" / "app"
            if not s.get("passed") or not app.is_dir():
                continue
            records.append({
                "id": f"agent-pokedex/{batch}/{s['dir']}",
                "source": "an agent's build of the Pokédex app of nolanlawson/agent-framework-benchmark",
                "kind": "agent-built app",
                "project": app,
                "check": True,
                "meta": {
                    "agent": "Claude Code " + (s.get("claudeCode") or "").split(" ")[0],
                    "model": data["config"]["model"],
                    "effort": data["config"]["effort"],
                    "arm": batch,
                    "passed_hidden_tests": "54/54",
                    "tokens": (s.get("tokens") or {}).get("total"),
                },
            })
    out = []
    seen = set()
    for r in records:
        if "only" in r:
            files = [{"path": f.name, "text": redact(f.read_text(encoding="utf-8", errors="replace"))} for f in r["only"]]
        else:
            files = files_of(r["project"])
        if not any(f["path"].endswith((".wf", ".wfx")) for f in files):
            continue
        # A starter names its project `{{name}}`, which `wf init` fills in; so is it here.
        starter = r.get("key") == "webfluent/starters"
        if starter:
            for f in files:
                f["text"] = f["text"].replace("{{name}}", "my-app")
        # The same sources twice (a copy, a test session counted again) are one record.
        digest = hashlib.sha256(json.dumps(sorted((f["path"], f["text"]) for f in files)).encode()).hexdigest()
        if digest in seen:
            continue
        seen.add(digest)
        out.append({
            "id": r["id"],
            "source": r["source"],
            "kind": r["kind"],
            "wf_version": VERSION,
            "written_for": WRITTEN_FOR.get(r.get("key"), VERSION),
            "layout": "wfx" if any(f["path"].endswith(".wfx") for f in files) else "wf",
            "checks_clean": (check_copy(files) if starter else check(r["project"])) if r["check"] else None,
            "files": files,
            **({"run": r["meta"]} if r["meta"] else {}),
        })
    return out


FENCE = re.compile(r"^```(wf|wfx)([^\n]*)\n(.*?)^```", re.S | re.M)


def examples():
    out = []
    sources = [ROOT / "AGENTS.md"]
    # The site's pages, as the Markdown twin each is published with, and the
    # compiler's explanation of each diagnostic.
    sources += sorted((ROOT / "site" / "public").rglob("*.md"))
    sources += sorted((ROOT / "src" / "diagnostics" / "explain").glob("*.md"))
    sources += sorted((ROOT / "skills" / "writing-webfluent").glob("**/*.md"))
    for folder, _what in EXAMPLE_SOURCES:
        if folder.is_dir():
            sources += sorted(p for p in folder.rglob("*.md") if not any(part in SKIP_DIRS - {"docs"} for part in p.parts))
    for md in sources:
        rel = md.relative_to(ROOT) if md.is_relative_to(ROOT) else pathlib.Path("wf-studio") / md.relative_to(CODE / "wf-studio")
        # The skill's reference sections are AGENTS.md again.
        if "references/language" in str(rel) or rel.name == "language.md":
            continue
        text = redact(md.read_text(encoding="utf-8"))
        heading = ""
        last = 0
        for i, m in enumerate(FENCE.finditer(text)):
            for h in re.findall(r"^#{1,4} (.+)$", text[last:m.start()], re.M):
                heading = h.strip()
            last = m.end()
            if "no-check" in m.group(2):
                continue
            out.append({
                "id": f"{rel}#{i}",
                "source": str(rel),
                "section": heading,
                "layout": m.group(1),
                "wf_version": VERSION,
                "text": m.group(3),
            })
    return out


def card(projs, exs, fixes, tasks):
    n_files = sum(len(p["files"]) for p in projs)
    n_wf = sum(1 for p in projs for f in p["files"] if f["path"].endswith((".wf", ".wfx")))
    by = {}
    for p in projs:
        b = by.setdefault(p["source"], [0, 0, 0, 0, p["written_for"]])
        b[0] += 1
        b[1] += len(p["files"])
        b[2] += 1 if p["checks_clean"] is not None else 0
        b[3] += 1 if p["checks_clean"] else 0
    rows = "\n".join(f"| {src} | {wf} | {n} | {f} | {c} of {k} |" for src, (n, f, k, c, wf) in by.items())
    ex_by = {}
    for e in exs:
        k = "WebFluent Studio's prompt cards and notes" if e["source"].startswith("wf-studio") else "WebFluent's site, diagnostics, skill and AGENTS.md"
        ex_by[k] = ex_by.get(k, 0) + 1
    ex_rows = "\n".join(f"| {k} | {n} |" for k, n in ex_by.items())
    fx_by = {}
    for r in fixes:
        b = fx_by.setdefault(r["mistake"], [0, set()])
        b[0] += 1
        b[1].update(d["code"] for d in r["diagnostics"])
    fx_rows = "\n".join(f"| {k} | {n} | {', '.join(sorted(c))} |" for k, (n, c) in sorted(fx_by.items(), key=lambda kv: -kv[1][0]))
    tk_by = {}
    for t in tasks:
        tk_by[t["source"]] = tk_by.get(t["source"], 0) + 1
    tk_rows = "\n".join(f"| {k} | {n} |" for k, n in tk_by.items())
    n_wfx = sum(1 for p in projs if p.get("layout") == "wfx")
    return f"""---
license: mpl-2.0
language:
- en
- ar
pretty_name: WebFluent code
tags:
- code
- webfluent
- programming-language
- web
size_categories:
- 1K<n<10K
configs:
- config_name: projects
  data_files: data/projects.jsonl
- config_name: examples
  data_files: data/examples.jsonl
- config_name: fixes
  data_files: data/fixes.jsonl
- config_name: tasks
  data_files: data/tasks.jsonl
---

# WebFluent code

Source code in [WebFluent](https://webfluent.monzeromer.dev), a typed language
that compiles to static sites, single-page apps, PDFs and slide decks: every
WebFluent project its author has written, and the examples the language
teaches with. Read with WebFluent {VERSION}.

## `projects`: {len(projs)} records, {n_files} files ({n_wf} of them `.wf`)

A record is one project — its `webfluent.app.json`, `.wf` sources,
stylesheets, scripts, translations and tests — or, where `kind` is `files`, a
folder of `.wf` files with no project of their own (test fixtures, agent
evaluations). `checks_clean` says whether `wf check` passes on a project with
WebFluent {VERSION} (`null` for a folder of files), and `written_for` the
WebFluent it was written for. Some fail on purpose: the compiler's test
projects include programs it must refuse, and `examples/videos/typo` shows it
catching a misspelt field; the studio's fixtures include a parse error and a
misspelt store member for its agent to fix, and an evaluation's expected answer
is laid over a fixture rather than built alone. Every source is written for the
current language: WebFluent Studio, first written for 6.x, is migrated to
{VERSION} like the rest.

| Source | Written for | Records | Files | `wf check` clean, of those checked |
|---|---|---:|---:|---:|
{rows}

The agent-built apps were written by a coding agent (Claude Code, Claude Opus
5.5) from the spec of
[agent-framework-benchmark](https://github.com/nolanlawson/agent-framework-benchmark);
each passed that benchmark's 54 hidden end-to-end tests, and `run` says which
arm of the experiment it came from and what it cost. The benchmark's spec,
tests, design and data are not included.

A project that passes `wf check` is there twice: as written, and rewritten in
the indented `.wfx` layout by `wf fmt --to wfx` and checked again (`layout:
"wfx"`, `derived_from` the original; {n_wfx} records). The two layouts are one
grammar, so the pair shows the same program both ways.

## `fixes`: {len(fixes)} error → fix pairs

A file of a project that passes `wf check`, broken by one small mistake of the
kind people and models make, what `wf check` says about it (`diagnostics`: code,
place, message, hint; `compiler_fix`: the edit the compiler offers, when it has
one), and the file as it was (`fixed`). A pair is kept only when the compiler
reports the mistake as an error, and only there.

| Mistake | Pairs | Codes reported |
|---|---:|---|
{fx_rows}

## `tasks`: {len(tasks)} requests and their answers

A request in words (`prompt`) and the files that answer it (`answer`), with the
files of the project as they were beforehand where the request changes a
project (`context`, and every file's path in `project_files`).
`answer_checks_clean` says whether the project with the answer passes `wf
check`.

| Source | Tasks |
|---|---:|
{tk_rows}

## `examples`: {len(exs)} records

Every `wf` block of these, with the file and section it is from:

| Source | Blocks |
|---|---:|
{ex_rows}

## What is left out

Build output, dependencies, vendored third-party libraries, native (Rust,
Android) code, and anything that is not WebFluent source or a project's own
stylesheets, scripts and translations. A personal email address is replaced
by `owner@example.com`.

## Licence

MPL-2.0, published by the author of WebFluent and of every project here.
Some projects name Pokémon and read the PokéAPI; Pokémon and Pokémon
character names are trademarks of Nintendo.
"""


if __name__ == "__main__":
    VERSION = wf_version()
    shutil.rmtree(OUT, ignore_errors=True)
    (OUT / "data").mkdir(parents=True)
    import pairs
    projs = projects()
    projs += pairs.wfx(WF, projs, check)
    exs = examples()
    fixes = pairs.fixes(WF, projs)
    tasks = pairs.tasks(WF, EVALS, FIXTURES, APPS, files_of, check)
    for name, rows in (("fixes", fixes), ("tasks", tasks)):
        with open(OUT / "data" / f"{name}.jsonl", "w") as f:
            for r in rows:
                f.write(json.dumps(r, ensure_ascii=False) + "\n")
    with open(OUT / "data" / "projects.jsonl", "w") as f:
        for r in projs:
            f.write(json.dumps(r, ensure_ascii=False) + "\n")
    with open(OUT / "data" / "examples.jsonl", "w") as f:
        for r in exs:
            f.write(json.dumps(r, ensure_ascii=False) + "\n")
    n_files = sum(len(p["files"]) for p in projs)
    n_agent = sum(1 for p in projs if p["kind"] == "agent-built app")
    (OUT / "README.md").write_text(card(projs, exs, fixes, tasks))
    clean = sum(1 for p in projs if p["checks_clean"])
    n_wfx = sum(1 for p in projs if p.get("layout") == "wfx")
    print(f"{len(projs)} projects ({n_agent} agent-built, {n_wfx} in .wfx), {n_files} files, {clean} pass wf check; "
          f"{len(exs)} examples; {len(fixes)} error-fix pairs; {len(tasks)} tasks -> {OUT}")
