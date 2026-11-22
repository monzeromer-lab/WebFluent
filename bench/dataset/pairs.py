"""The dataset's generated tables, each checked by the compiler it teaches.

  fixes   a file broken by one small, realistic mistake, what `wf check` says about
          it, and the file as it was: an error → fix pair. Only mistakes the
          compiler reports as an error are kept, and the fixed file is the one a
          project that passes `wf check` already had.
  tasks   a request and the change that answers it: WebFluent Studio's agent
          evaluations (a fixture, a prompt, the files the reference changes) and
          the apps written from a brief (BRIEF.md → the app).
  wfx     a project that passes `wf check`, rewritten in the indented `.wfx`
          layout by `wf fmt --to wfx`, and checked again.
"""

import concurrent.futures
import hashlib
import json
import pathlib
import random
import re
import shutil
import subprocess
import tempfile

# ---------------------------------------------------------------- mistakes

# Each mistake changes one place in one line, the way a person or a model gets it
# wrong: a misspelt name, a type left out, a splice written as `+`, a brace lost.
def _misspell(word, rng):
    if len(word) < 4:
        return None
    i = rng.randrange(1, len(word) - 1)
    how = rng.choice(["swap", "drop", "double"])
    if how == "swap" and word[i] != word[i + 1]:
        return word[:i] + word[i + 1] + word[i] + word[i + 2:]
    if how == "drop":
        return word[:i] + word[i + 1:]
    return word[:i] + word[i] + word[i:]


MISTAKES = []


def mistake(kind, pattern, flags=0):
    def wrap(fn):
        MISTAKES.append((kind, re.compile(pattern, flags), fn))
        return fn
    return wrap


@mistake("misspelt store member", r"\b([A-Z][A-Za-z0-9]+)\.([a-z][A-Za-z0-9]{3,})\b")
def _store_member(m, rng):
    w = _misspell(m.group(2), rng)
    return w and f"{m.group(1)}.{w}"


@mistake("misspelt field", r"\b([a-z][A-Za-z0-9]*)\.([a-z][A-Za-z0-9]{3,})\b(?!\s*\()")
def _field(m, rng):
    w = _misspell(m.group(2), rng)
    return w and f"{m.group(1)}.{w}"


@mistake("misspelt component", r"^(\s*)([A-Z][A-Za-z0-9]{3,})(?=\s*[({.])", re.M)
def _component(m, rng):
    w = _misspell(m.group(2), rng)
    return w and f"{m.group(1)}{w}"


@mistake("misspelt flag", r"(\)|[A-Za-z])\.(primary|secondary|danger|success|warning|muted|bold|outlined|elevated|center|h1|h2|h3|sm|lg|pill|full)\b")
def _flag(m, rng):
    w = _misspell(m.group(2), rng) if len(m.group(2)) >= 4 else m.group(2) + "x"
    return w and f"{m.group(1)}.{w}"


@mistake("state without its type", r"\b(state|persist) ([A-Za-z_][A-Za-z0-9_]*): \[[^\]=]+\] = \[\]")
def _state_type(m, rng):
    return f"{m.group(1)} {m.group(2)} = []"


@mistake("nullable state without its type", r"\b(state) ([A-Za-z_][A-Za-z0-9_]*): [A-Z][A-Za-z0-9]*\? = null")
def _nullable_type(m, rng):
    return f"{m.group(1)} {m.group(2)} = null"


@mistake("lambda parameter without its type", r"\(([a-z][A-Za-z0-9]*): [A-Z][A-Za-z0-9\[\]?]*\) =>")
def _lambda_type(m, rng):
    return f"({m.group(1)}) =>"


@mistake("action parameter without its type", r"\baction ([a-z][A-Za-z0-9]*)\(([a-z][A-Za-z0-9]*): [A-Z][A-Za-z0-9\[\]?:, ]*\)")
def _param_type(m, rng):
    return f"action {m.group(1)}({m.group(2)})"


@mistake("text joined with `+`", r'"([^"{}\n]*?) \{([a-z][A-Za-z0-9]*)\}"')
def _plus(m, rng):
    return f'"{m.group(1)} " + {m.group(2)}'


@mistake("missing await", r"\b(let [a-z][A-Za-z0-9]*(?:: [^=]+)? = )await ")
def _await(m, rng):
    return m.group(1)


@mistake("6.x read arm", r"\.loaded\(")
def _ready(m, rng):
    return "ready("


@mistake("unknown store", r"^(\s*)use ([A-Z][A-Za-z0-9]{3,})$", re.M)
def _use(m, rng):
    w = _misspell(m.group(2), rng)
    return w and f"{m.group(1)}use {w}"


@mistake("handler naming more than its event sends", r"\bon click \{")
def _handler_args(m, rng):
    return "on click(e, x) {"


@mistake("misspelt enum case", r"(\b(?:align|justify|gap|placement|tone|size): )\.([a-z]{4,})\b")
def _case(m, rng):
    w = _misspell(m.group(2), rng)
    return w and f"{m.group(1)}.{w}"


def _mutants(text, rng, per_file):
    """Up to `per_file` single-place mistakes in `text`, each of a different kind,
    the kinds taken round-robin so no one kind of mistake crowds out the rest."""
    by_kind = {}
    for kind, rx, fn in MISTAKES:
        ms = list(rx.finditer(text))
        if ms:
            by_kind[kind] = (ms, fn)
    kinds = list(by_kind)
    rng.shuffle(kinds)
    out = []
    for kind in kinds:
        ms, fn = by_kind[kind]
        m = rng.choice(ms)
        new = fn(m, rng)
        if not new or new == m.group(0):
            continue
        broken = text[: m.start()] + new + text[m.end():]
        line = text.count("\n", 0, m.start()) + 1
        out.append((kind, line, broken))
        if len(out) >= per_file:
            break
    return out


def _check_json(wf, project):
    run = subprocess.run(["nice", "-n", "10", wf, "check", "--format", "json", "-d", str(project)],
                         capture_output=True, text=True)
    try:
        return json.loads(run.stdout).get("diagnostics", [])
    except json.JSONDecodeError:
        return None


def _write(root, files):
    for f in files:
        p = root / f["path"]
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_text(f["text"])


def _one_fix(wf, rec, path, kind, line, broken):
    with tempfile.TemporaryDirectory() as tmp:
        root = pathlib.Path(tmp)
        _write(root, rec["files"])
        (root / path).write_text(broken)
        diags = _check_json(wf, root)
    if diags is None:
        return None
    errors = [d for d in diags if d.get("severity") == "error"]
    here = [d for d in errors if str(d.get("file", "")).endswith(path)]
    # One mistake, reported where it was made: a pair that teaches one thing.
    if not here or len(errors) != len(here):
        return None
    original = next(f["text"] for f in rec["files"] if f["path"] == path)
    return {
        "mistake": kind,
        "line": line,
        "broken": broken,
        "fixed": original,
        "diagnostics": [
            {k: d.get(k) for k in ("code", "line", "column", "message", "hint")}
            for d in here
        ],
        "compiler_fix": [f for d in here for f in (d.get("fixes") or [])],
    }


def fixes(wf, projects, per_project=12, seed=7):
    """Error → fix pairs from every project that passes `wf check`."""
    rng = random.Random(seed)
    jobs = []
    for rec in projects:
        if not rec.get("checks_clean") or rec.get("layout") == "wfx":
            continue
        wf_files = [f for f in rec["files"] if f["path"].endswith(".wf") and "/tests/" not in f"/{f['path']}"]
        rng.shuffle(wf_files)
        # A larger project has more to get wrong: its share grows with its files.
        budget = min(per_project * 5, per_project + len(wf_files) // 2)
        for f in wf_files:
            if budget <= 0:
                break
            for kind, line, broken in _mutants(f["text"], rng, min(3, budget)):
                jobs.append((rec, f["path"], kind, line, broken))
                budget -= 1
    out = []
    with concurrent.futures.ThreadPoolExecutor(max_workers=6) as pool:
        futs = {pool.submit(_one_fix, wf, rec, path, kind, line, broken): (rec, path)
                for rec, path, kind, line, broken in jobs}
        for fut in concurrent.futures.as_completed(futs):
            rec, path = futs[fut]
            r = fut.result()
            if r:
                out.append({"id": "", "project": rec["id"], "file": path, **r})
    out.sort(key=lambda r: (r["project"], r["file"], r["line"], r["mistake"]))
    for i, r in enumerate(out):
        r["id"] = f"{r['project']}:{r['file']}:{r['line']}:{hashlib.sha1(r['broken'].encode()).hexdigest()[:8]}"
    return out


# ---------------------------------------------------------------- tasks

def _overlay(base, layer):
    files = {f["path"]: f["text"] for f in base}
    for f in layer:
        files[f["path"]] = f["text"]
    return [{"path": p, "text": t} for p, t in sorted(files.items())]


def tasks(wf, evals_dir, fixtures_dir, apps_dir, files_of, check):
    """Requests and the changes that answer them."""
    out = []
    evals_dir, fixtures_dir = pathlib.Path(evals_dir), pathlib.Path(fixtures_dir)
    for tj in sorted(evals_dir.glob("*/task.json")):
        task = json.loads(tj.read_text())
        fixture = fixtures_dir / task["fixture"]
        if not fixture.is_dir() or not (tj.parent / "reference").is_dir():
            continue
        base = files_of(fixture)
        setup = files_of(tj.parent / "setup") if (tj.parent / "setup").is_dir() else []
        before = _overlay(base, setup)
        ref = files_of(tj.parent / "reference")
        after = _overlay(before, ref)
        old = {f["path"]: f["text"] for f in before}
        changed = [f for f in ref if old.get(f["path"]) != f["text"]]
        if not changed:
            continue
        with tempfile.TemporaryDirectory() as tmp:
            _write(pathlib.Path(tmp), after)
            clean = check(pathlib.Path(tmp))
        # An answer that no longer builds would teach the wrong thing.
        if not clean:
            continue
        out.append({
            "id": f"studio-eval/{task['id']}",
            "source": "WebFluent Studio's agent evaluations",
            "prompt": task["prompt"],
            "open": task.get("open"),
            "context": [f for f in before if f["path"] in {c["path"] for c in changed} or f["path"] == task.get("open")],
            "project_files": [f["path"] for f in before],
            "answer": changed,
            "answer_checks_clean": clean,
        })
    apps_dir = pathlib.Path(apps_dir)
    if apps_dir.is_dir():
        for brief in sorted(apps_dir.glob("*/BRIEF.md")):
            app = brief.parent
            if not (app / "webfluent.app.json").is_file():
                continue
            files = [f for f in files_of(app) if f["path"] != "BRIEF.md"]
            out.append({
                "id": f"webfluent-apps/{app.name}",
                "source": "an app written in WebFluent from a brief",
                "prompt": brief.read_text().strip(),
                "open": None,
                "context": [],
                "project_files": [],
                "answer": files,
                "answer_checks_clean": check(app),
            })
    return out


# ---------------------------------------------------------------- .wfx

def _one_wfx(wf, rec, check):
    with tempfile.TemporaryDirectory() as tmp:
        root = pathlib.Path(tmp) / "p"
        root.mkdir()
        _write(root, rec["files"])
        run = subprocess.run(["nice", "-n", "10", wf, "fmt", "--to", "wfx", str(root)], capture_output=True, text=True)
        if run.returncode != 0 or not check(root):
            return None
        files = []
        for f in rec["files"]:
            path = f["path"]
            if path.endswith(".wf") and (root / (path[:-3] + ".wfx")).is_file():
                path = path[:-3] + ".wfx"
            p = root / path
            if p.is_file():
                files.append({"path": path, "text": p.read_text()})
    if not any(f["path"].endswith(".wfx") for f in files):
        return None
    return {**rec, "id": rec["id"] + "@wfx", "layout": "wfx", "files": files, "checks_clean": True,
            "derived_from": rec["id"]}


def wfx(wf, projects, check):
    """Every clean `.wf` project again in the indented layout."""
    todo = [r for r in projects if r.get("checks_clean") and r["kind"] != "files"
            and any(f["path"].endswith(".wf") for f in r["files"])
            and not any(f["path"].endswith(".wfx") for f in r["files"])]
    out = []
    with concurrent.futures.ThreadPoolExecutor(max_workers=6) as pool:
        for r in pool.map(lambda rec: _one_wfx(wf, rec, check), todo):
            if r:
                out.append(r)
    return out
