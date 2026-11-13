#!/usr/bin/env python3
"""Write a Markdown twin of every page the site publishes, and the maps an
agent reads first.

- `site/public/<route>.md` for each page (`/` is `index.md`): its source in
  `md-docs/` with the front matter off and every link between pages made
  absolute and pointed at the other page's twin; the reference's built-ins
  and commands are written from the compiler's own data.
- `site/public/llms.txt`: the site's map, as https://llmstxt.org lays one
  out — a line per page, by part.
- `site/public/llms-full.txt`: the book, the essays, the recipes, the cheat
  sheets and the reference's prose, whole, in one file.

`WF_TWINS_OUT` writes into that directory instead (for
`cargo xtask site --check`). Rerun with `cargo xtask site`.
"""
import importlib.util
import json
import os
import re
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
DOCS = ROOT / "md-docs"
_spec = importlib.util.spec_from_file_location("guide", HERE / "guide.py")
guide = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(guide)
sys.path.insert(0, str(HERE))
import reference  # noqa: E402

OUT = Path(os.environ["WF_TWINS_OUT"]) if os.environ.get("WF_TWINS_OUT") else ROOT / "site" / "public"
wf = Path(os.environ["WF_BIN"]) if os.environ.get("WF_BIN") else ROOT / "target" / "debug" / "wf"
if not wf.exists():
    wf = "wf"
CONFIG = json.loads((ROOT / "site" / "webfluent.app.json").read_text())
SITE = CONFIG["meta"]["site_url"].rstrip("/")


def ask(*args):
    return json.loads(subprocess.check_output([str(wf), *args, "--json"]))


def twin_path(route: str) -> str:
    """Where a page's twin is served: `/learn/07-state` → `/learn/07-state.md`."""
    route = route.rstrip("/")
    return (route or "/index") + ".md"


class Twins:
    def __init__(self):
        self.pages = {}  # route → (title, description, markdown, part)

    def add(self, route, title, description, body, part):
        self.pages[route] = (title, description, body.strip() + "\n", part)

    def write(self):
        routes = set(self.pages)

        def point(m):
            label, target = m.group(1), m.group(2)
            path, _, anchor = target.partition("#")
            if path.rstrip("/") in routes or (path == "/" and "/" in routes):
                return f"[{label}]({SITE}{twin_path(path)}{'#' + anchor if anchor else ''})"
            if path.startswith("/"):
                return f"[{label}]({SITE}{target})"
            return m.group(0)

        for route, (title, description, body, _) in sorted(self.pages.items()):
            text = re.sub(r"\[([^\]]+)\]\(([^)\s]+)\)", point, body)
            head = f"# {title}\n\n"
            if description:
                head += f"> {description}\n\n"
            head += f"The page: {SITE}{route}\n\n"
            path = OUT / twin_path(route).lstrip("/")
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(head + text)


def source(path: Path):
    """A page of `md-docs/`: its front matter, title and body, with its
    links between pages made site routes."""
    meta, body = guide.front_matter(path)
    guide.CURRENT["dir"] = path.parent
    lines = body.lstrip("\n").split("\n")
    title = re.sub(r"^# (\d+\.\s*)?", "", lines[0]).strip() if lines[0].startswith("# ") else meta.get("title", "")
    rest = "\n".join(lines[1:] if lines[0].startswith("# ") else lines)
    # Code is kept as written; links are rewritten outside it only.
    parts = re.split(r"(?ms)(^```.*?^```[ \t]*$)", rest)
    rest = "".join(p if p.startswith("```") else guide.rewrite_links(p) for p in parts)
    # A heading numbered for the book keeps its number in the title only.
    return meta, title, rest


def component(c) -> str:
    out = [c["plain"], "", "```wf", c["signature"], "```", ""]
    if c["props"]:
        out += ["## Props", "", "| Prop | Type | Meaning |", "|---|---|---|"]
        out += [f"| `{p['name']}` | `{p['type']}` | {p['meaning']} |" for p in c["props"]]
        out.append("")
    if c["flags"]:
        out += ["## Flags", "", " ".join(f"`{f}`" for f in c["flags"]), ""]
    if c["events"]:
        out += ["## Events", ""] + [f"- `{e['name']}`{': ' + e['detail'] if e.get('detail') else ''}" for e in c["events"]] + [""]
    if c["slots"]:
        out += ["## Slots", ""] + [f"- `{s['name']}`{': ' + s['detail'] if s.get('detail') else ''}" for s in c["slots"]] + [""]
    if c["parts"]:
        out += ["## Parts", ""] + [f"- `{c['name']}.{p['name'] if isinstance(p, dict) else p}`" for p in c["parts"]] + [""]
    if c.get("example"):
        out += ["## Example", "", "```wf", c["example"], "```", ""]
    if c["codes"]:
        out += ["## What it can raise", ""] + [f"- [{k['kind']}]({k['to']}): {k['title']}" for k in c["codes"]] + [""]
    if c.get("since"):
        out += [f"Since WebFluent {c['since']}.", ""]
    return "\n".join(out)


def command(c) -> str:
    out = [c["summary"], "", "```bash", c["usage"], "```", ""]
    if c["options"]:
        out += ["## Options", "", "| Option | Meaning |", "|---|---|"]
        for o in c["options"]:
            out.append(f"| `{o.get('flag') or o.get('name', '')}` | {o.get('meaning') or o.get('help', '')} |")
        out.append("")
    if c.get("writes"):
        out += [f"**Writes:** {c['writes']}", ""]
    return "\n".join(out)


def main():
    twins = Twins()
    parts = {"learn": "Learn", "about": "About", "reference": "Reference", "errors": "Errors",
             "recipes": "Recipes", "coming-from": "Coming from"}
    # Every page written from md-docs/.
    for path in sorted(DOCS.rglob("*.md")):
        if path.parent == DOCS:
            continue
        meta = guide.front_matter(path)[0]
        folder = path.relative_to(DOCS).parts[0]
        if path.name == "index.md" and folder == "recipes":
            route = "/recipes"
        elif "route" in meta:
            route = guide.href(meta["route"])
        else:
            continue
        _, title, body = source(path)
        twins.add(route, title, meta.get("description", ""), body, parts.get(folder, "Reference"))
    # The book's and the essays' contents pages.
    for part, route, title, lede in (
        ("learn", "/learn", "Learn WebFluent", "The book, in order. The first three chapters are the tour."),
        ("about", "/about", "About WebFluent", "Essays on why WebFluent works as it does."),
    ):
        rows = sorted((r, t) for r, (t, d, b, p) in twins.pages.items() if r.startswith(route + "/"))
        body = lede + "\n\n" + "\n".join(f"- [{t}]({r})" for r, t in rows)
        twins.add(route, title, "", body, "Learn" if part == "learn" else "About")
    # The reference's built-ins and commands, from the compiler's data.
    # A legacy name has a page of its own too (it says what replaced it).
    for c in reference.components(ask("registry"))["items"]:
        twins.add(f"/reference/{c['slug']}", c["name"], c["plain"], component(c), "Reference")
    for c in reference.commands(ask("commands"))["items"]:
        twins.add(f"/reference/cli/{c['slug']}", f"wf {c['name']}", c["summary"], command(c), "Reference")
    # The pages written in WebFluent rather than Markdown.
    twins.add("/", "WebFluent", CONFIG["meta"].get("description", ""),
              "WebFluent compiles one typed `.wf` file into a static site, a web app, a PDF or a slide "
              "deck. Start with [the book](/learn), find a task in [the recipes](/recipes), look a "
              "built-in up in [the reference](/reference), or find a compiler message in "
              "[the errors](/errors). The map for agents is [llms.txt](/llms.txt).\n\n"
              "```bash\ncurl -sSL https://raw.githubusercontent.com/monzeromer-lab/WebFluent/master/install.sh | bash\n"
              "wf init my-site -t static\ncd my-site && wf serve\n```", "Start")
    twins.add("/playground", "Playground", "The compiler, in the browser.",
              "The Playground runs the same compiler as `wf build`, compiled to WebAssembly, in the "
              "reader's browser: write a page, see it built, its findings listed, and its PDF. Nothing "
              "written there is sent to a server.", "Start")
    twins.add("/showcase", "Showcase", "Sites, documents and decks made with WebFluent.",
              "Real outputs of WebFluent projects: websites, PDFs and slide decks, each from the "
              "`.wf` source in the repository's `examples/`.", "Start")
    twins.add("/docs", "Documentation", "", "The documentation's parts: [Learn](/learn), "
              "[Recipes](/recipes), [Reference](/reference), [Errors](/errors) and [About](/about).", "Start")
    twins.add("/changelog", "Changelog", "What each release of WebFluent changed.",
              re.sub(r"^# .*\n", "", (ROOT / "RELEASE_NOTES.md").read_text(), count=1), "Start")
    twins.add("/ar", "WebFluent", "The front page, in Arabic.",
              f"The Arabic edition of [the front page]({SITE}/index.md). Translated so far: "
              "[Getting started](/ar/learn/01-getting-started), [Tutorial: Shelf](/ar/learn/02-tutorial-shelf), "
              "[Tutorial: Todos](/ar/learn/03-tutorial-todos) and [Coming from React](/ar/coming-from/react).", "Start")
    twins.add("/ar/playground", "Playground", "The compiler, in the browser (Arabic interface).",
              f"The Arabic interface of [the Playground]({SITE}/playground.md).", "Start")
    twins.write()

    # llms.txt: the map, a line per page, by part.
    order = ["Start", "Learn", "Recipes", "Coming from", "Reference", "Errors", "About"]
    lines = [
        "# WebFluent",
        "",
        "> A web-first language: one typed `.wf` source compiles to a static site, a web app, a PDF or a "
        "slide deck. The compiler refuses, with a code and a fix, what would fail in the browser.",
        "",
        "Every page below is Markdown. The language's whole reference in one file is "
        f"[AGENTS.md](https://github.com/monzeromer-lab/WebFluent/blob/master/AGENTS.md), which `wf init` "
        f"writes into every project; [llms-full.txt]({SITE}/llms-full.txt) is the book, the recipes and the "
        "reference's prose in one file.",
    ]
    for part in order:
        # The Arabic interface's pages are the English ones', in the map.
        rows = [(r, t, d) for r, (t, d, b, p) in twins.pages.items() if p == part and not r.startswith("/ar/")]
        if not rows:
            continue
        lines += ["", f"## {part}", ""]
        for r, t, d in sorted(rows, key=lambda x: x[0]):
            lines.append(f"- [{t}]({SITE}{twin_path(r)})" + (f": {d}" if d else ""))
    (OUT / "llms.txt").write_text("\n".join(lines) + "\n")

    # llms-full.txt: the prose, whole — not each code's or each built-in's
    # page, which the map lists one by one.
    full = ["# WebFluent — the documentation in one file", ""]
    for part in ["Learn", "Recipes", "Coming from", "About"]:
        for r, (t, d, b, p) in sorted(twins.pages.items()):
            if p == part:
                full += [f"\n\n---\n\n# {t}\n", f"{SITE}{r}\n", b]
    for r in ("/reference/expressions", "/reference/grammar", "/reference/project", "/reference/built-ins",
              "/reference/runtime", "/reference/cli", "/reference/config", "/reference/tokens", "/errors"):
        if r in twins.pages:
            t, d, b, p = twins.pages[r]
            full += [f"\n\n---\n\n# {t}\n", f"{SITE}{r}\n", b]
    (OUT / "llms-full.txt").write_text("\n".join(full) + "\n")
    print(f"wrote {len(twins.pages)} twins, llms.txt and llms-full.txt")


if __name__ == "__main__":
    main()
