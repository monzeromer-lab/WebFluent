#!/usr/bin/env python3
"""Write a Markdown twin of every page the site publishes, the maps an
agent reads first, and the list of twins a tool reads.

- `site/public/<route>.md` for each page (`/` is `index.md`): the page's
  own words as Markdown, read from its `.wf` source under
  `site/src/pages/` by `sitepages.py` — its headings, prose, code, asides,
  tables and quizzes, in the order the page shows them — with every link
  between pages made absolute and pointed at the other page's twin. The
  error pages are written from `wf explain --json`, and the reference's
  built-ins and commands from the compiler's own data.
- `site/public/llms.txt`: the site's map, as https://llmstxt.org lays one
  out — a line per page, by part.
- `site/public/llms-full.txt`: the book, the essays, the recipes, the cheat
  sheets and the reference's prose, whole, in one file.
- `site/public/twins.json`: every twin, for a tool that reads the
  documentation as Markdown (the studio does): `[{ route, twin, title,
  description, part, lang }]`, by route. `twin` is the twin's path on the
  site (`/learn/07-state-and-reactivity.md`); its text starts with
  `# Title`, then `> description`, then `The page: <url>`, then the page's
  Markdown. The list and the format are kept as they are; a page added to
  the site is a row added here.

`WF_TWINS_OUT` writes into that directory instead (for
`cargo xtask site --check`). Rerun with `cargo xtask site`.
"""
import json
import os
import re
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
sys.path.insert(0, str(HERE))
import components as components_md  # noqa: E402
import errors  # noqa: E402
import reference  # noqa: E402
import sitepages  # noqa: E402
from sitepages import Block, Call, markdown, text_of  # noqa: E402

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
        self.pages = {}  # route → (title, description, markdown, part, lang)

    def add(self, route, title, description, body, part, lang="en"):
        self.pages[route] = (title, description, body.strip() + "\n", part, lang)

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

        for route, (title, description, body, _, _) in sorted(self.pages.items()):
            # Code is kept as written; links are pointed at twins outside it.
            parts = re.split(r"(?ms)(^```.*?^```[ \t]*$)", body)
            text = "".join(p if p.startswith("```") else re.sub(r"\[([^\]]+)\]\(([^)\s]+)\)", point, p) for p in parts)
            head = f"# {title}\n\n"
            if description:
                head += f"> {description}\n\n"
            head += f"The page: {SITE}{route}\n\n"
            path = OUT / twin_path(route).lstrip("/")
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(head + text)


# ─── The site's own pages ────────────────────────────────────────────────

PARTS = {"learn": "Learn", "about": "About", "reference": "Reference", "errors": "Errors",
         "recipes": "Recipes", "coming-from": "Coming from"}


def part_of(route: str) -> str:
    """The part of the site a page is in, for the map: its first segment's.
    A translation (`/ar/…`) is the map's Reference, which llms.txt leaves out."""
    first = route.strip("/").split("/")[0]
    return PARTS.get(first, "Reference")


class Expand:
    """What a page places that is drawn from data rather than written in
    it, each as the Markdown it shows: none, now — the words are in the
    pages."""

    def __call__(self, call: Call):
        return None


def words(key: str) -> list:
    """A part of words of `Cli.wf` (`index`, `build`), as elements."""
    body = reference.words().get(key, "")
    return sitepages.Reader("{" + body + "}").block()[0] if body.strip() else []


def shell_body(page, s: Call) -> list:
    """A page's items in reading order, with what its shell draws itself —
    a recipe's "How it works", a cheat sheet's map — as items of its own."""
    tr = lambda key: Call("t", [(None, key)])  # noqa: E731
    head = lambda key: Call("Heading", [(None, tr(key))])  # noqa: E731
    if s.name == "RecipeShell":
        items = [Call("Markdown", [(None, text_of(s.arg("summary", "")))])]
        items += s.fills.get("code", [])
        if s.arg("how", True) is not False:
            items.append(head("recipe.how"))
        items += s.children()
        if s.fills.get("variations"):
            items += [head("recipe.variations")] + s.fills["variations"]
        return items
    if s.name == "ComingFromShell":
        rows = [r for r in s.children() if isinstance(r, Call) and r.name == "CheatRow"]
        table = ["| " + text_of(tr("coming.concept")) + " | " + text_of(s.arg("from", "")) + " | WebFluent |", "|---|---|---|"]
        cell = lambda v: f"`{v}`".replace("\n", " ").replace("|", "\\|") if v else ""  # noqa: E731
        table += [f"| {text_of(r.pos(0))} | {cell(text_of(r.arg('theirs', '')))} | {cell(text_of(r.arg('ours', '')))} |" for r in rows]
        items = [Call("Markdown", [(None, text_of(s.arg("summary", "")))]), head("coming.map"), Call("Markdown", [(None, "\n".join(table))])]
        if s.fills.get("differences"):
            items += [head("coming.differences")] + s.fills["differences"]
        if s.fills.get("port"):
            items += [head("coming.port")] + s.fills["port"]
        return items
    if s.name == "ReferenceItemShell":
        return s.fills.get("notes", []) + s.children()
    return sitepages.body_items(page)


def page_twins(twins: Twins, expand: Expand) -> None:
    for page in sitepages.pages():
        route = text_of(page.arg("path"))
        if page.arg("redirect") or page.arg("noindex") or page.arg("guard") is not None or ":" in route:
            continue
        s = sitepages.shell(page)
        if s is None or s.name not in ("DocsShell", "RecipeShell", "ComingFromShell", "ReferenceItemShell"):
            continue
        # The reference's built-ins and commands are written from the
        # compiler's data, below; Config and Tokens are pages of words.
        if s.name == "ReferenceItemShell" and route not in ("/reference/config", "/reference/tokens"):
            continue
        lang = text_of(page.arg("lang", "")) or "en"
        sitepages.LANG[0] = lang
        title = text_of(s.pos(0))
        body = markdown(shell_body(page, s), with_info=True, expand=expand)
        twins.add(route, title, text_of(page.arg("description", "")), body, part_of(route), lang)
    sitepages.LANG[0] = "en"


def recipes_index(twins: Twins) -> None:
    """`/recipes`: its words, then every recipe by group — each group's
    heading and line as the page has them, its recipes from
    `RECIPE_INDEX`."""
    page = next(p for p in sitepages.pages() if text_of(p.arg("path")) == "/recipes")
    entries = sitepages.consts()["RECIPE_INDEX"]
    out = []

    def walk(items):
        for it in items:
            if isinstance(it, Block):
                walk(it.items)
            elif isinstance(it, Call):
                kids = [k for k in it.children() if isinstance(k, Call)]
                heading = next((k for k in kids if k.name == "Heading" and "h2" in k.flags), None)
                if it.name == "Markdown":
                    out.append(text_of(it.pos(0)))
                elif heading:
                    group = text_of(heading.pos(0))
                    out.append(f"## {group}")
                    out.extend(text_of(k.pos(0)) for k in kids if k.name == "Text")
                    out.append("\n".join(f"- [{text_of(e.arg('label'))}]({text_of(e.arg('to'))})"
                                         for e in entries if text_of(e.arg("group")) == group))
                else:
                    walk(it.children())

    walk(page.body)
    twins.add("/recipes", "Recipes", text_of(page.arg("description", "")), "\n\n".join(out), "Recipes")


def build() -> "Twins":
    """Every twin, in memory: what `main` writes, and what the search
    index (`data.py`) reads each page's words from."""
    twins = Twins()
    explained = ask("explain")
    expand = Expand()

    page_twins(twins, expand)
    recipes_index(twins)
    # Each code's page, from the compiler's explanation of it.
    for e in explained:
        twins.add(f"/errors/{e['code']}", f"{e['code']} — {e['title']}", e.get("description") or e["summary"],
                  errors.link_paths(e["section"]), "Errors")
    # The book's and the essays' contents pages.
    for part, route, title, lede in (
        ("learn", "/learn", "Learn WebFluent", "The book, in order. The first three chapters are the tour."),
        ("about", "/about", "About WebFluent", "Essays on why WebFluent works as it does."),
    ):
        rows = sorted((r, v[0]) for r, v in twins.pages.items() if r.startswith(route + "/"))
        body = lede + "\n\n" + "\n".join(f"- [{t}]({r})" for r, t in rows)
        twins.add(route, title, "", body, "Learn" if part == "learn" else "About")
    # The reference's built-ins and commands, from the compiler's data.
    # A legacy name has a page of its own too (it says what replaced it).
    reg = ask("registry")
    twins.add("/reference", "Components reference",
              "The components reference: every built-in element, its props, flags, events, slots, parts, attribute families and an example.",
              components_md.markdown(reg), "Reference")
    items = reference.components(reg)["items"]
    by_slug = {c["slug"]: c for c in items}
    for c in items:
        if c["owner"]:
            o = by_slug[c["ownerSlug"]]
            body = moved(c, f"/reference/{o['slug']}#{c['slug']}", o["name"], False)
        elif c["legacy"]:
            body = moved(c, c["modern"][1] or "/reference#older-spellings", c["modern"][0] or "the reference", True)
        else:
            body = component(c, [p for p in items if p["owner"] == c["name"]])
        twins.add(f"/reference/{c['slug']}", c["name"], c["plain"], body, "Reference")
    cmds = reference.commands(ask("commands"))["items"]
    group_of = reference.cli_groups(reference.words())
    for c in cmds:
        twins.add(f"/reference/cli/{c['slug']}", f"wf {c['name']}", c["summary"], command(c, expand), "Reference")
    # `/reference/cli`: its words, then every command by group, each with
    # what its own page says.
    cli = [markdown(words("index"), with_info=True, expand=expand), "## Commands"]
    groups = []
    for c in cmds:
        g = group_of.get(c["name"], "") or "Other commands"
        if g not in groups:
            groups.append(g)
    for g in groups:
        cli.append(f"### {g}")
        for c in cmds:
            if (group_of.get(c["name"], "") or "Other commands") != g:
                continue
            cli.append(f"#### `wf {c['name']}`\n\n{c['summary']}: [its page](/reference/cli/{c['slug']}).\n\n```bash\n{c['usage']}\n```")
            if words(c["slug"]):
                cli.append(markdown(words(c["slug"]), with_info=True, expand=expand))
    twins.add("/reference/cli", "Command line",
              "Every wf command and its options, from init, build and serve to render, migrate and verify, and the language server in each editor.",
              "\n\n".join(cli), "Reference")
    # The pages written as pages rather than as documents.
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
    bench = benchmarks_markdown()
    if bench:
        twins.add("/benchmarks", "Benchmarks", "WebFluent against Next.js, Astro, SvelteKit, Nuxt and React, measured.",
                  bench, "Start")
        twins.add("/ar/benchmarks", "Benchmarks", "The benchmarks, in Arabic.",
                  f"The Arabic edition of [the benchmarks]({SITE}/benchmarks.md): the same numbers.", "Start", "ar")
    twins.add("/changelog", "Changelog", "What each release of WebFluent changed.",
              re.sub(r"^# .*\n", "", (ROOT / "RELEASE_NOTES.md").read_text(), count=1), "Start")
    twins.add("/ar", "WebFluent", "The front page, in Arabic.",
              f"The Arabic edition of [the front page]({SITE}/index.md). Translated so far: "
              "[Getting started](/ar/learn/01-getting-started), [Tutorial: Shelf](/ar/learn/02-tutorial-shelf), "
              "[Tutorial: Todos](/ar/learn/03-tutorial-todos) and [Coming from React](/ar/coming-from/react).", "Start", "ar")
    twins.add("/ar/playground", "Playground", "The compiler, in the browser (Arabic interface).",
              f"The Arabic interface of [the Playground]({SITE}/playground.md).", "Start", "ar")
    return twins


def capabilities(caps, words):
    """The capability table: each cell its level, linked to its source."""
    head = "| " + " | ".join([words["bench.measure"]] + caps["columns"]) + " |"
    rule = "|---" * (len(caps["columns"]) + 1) + "|"
    rows = ["| " + " | ".join([words[r["key"]]] + [f"[{words['bench.level.' + c['level']]}]({c['source']})" for c in r["cells"]]) + " |"
            for r in caps["rows"]]
    rows.append("| " + " | ".join([words["bench.check.count"]] + [f"{n} / 12" for n in caps["counts"]]) + " |")
    return f"## {words['bench.defaults.caption']}\n\n" + "\n".join([head, rule] + rows)


def benchmarks_markdown():
    """The benchmarks page as Markdown: each table, with the words the page
    gives its rows, from the file bench/report.mjs writes."""
    path = ROOT / "site/src/benchmarks.json"
    if not path.exists():
        return ""
    data = json.loads(path.read_text())
    words = json.loads((ROOT / "site/src/translations/en.json").read_text())
    clean = lambda v: str(v).replace("\u2066", "").replace("\u2069", "")

    def table(t, title):
        head = "| " + " | ".join([words["bench.measure"]] + t["columns"]) + " |"
        rule = "|---" * (len(t["columns"]) + 1) + "|"
        rows = ["| " + " | ".join([words.get(r["key"], r["key"])] + r["cells"]) + " |" for r in t["rows"]]
        return f"## {title}\n\n" + "\n".join([head, rule] + rows)

    h = data["headline"]
    out = [
        "Every number comes from a script under [`bench/`](https://github.com/monzeromer-lab/WebFluent/tree/master/bench), "
        "run on one machine; the raw results sit beside each script.",
        f"- Lighthouse for a new `wf init -t static` site: {clean(h['lighthouse'])} in every category\n"
        f"- Built in with no package: {clean(h['defaults'])} ({clean(h['defaultsUntouched'])} with nothing done, three more with `meta.site_url`); the most any other framework builds in: {clean(h['defaultsBest'])}\n"
        f"- Production build: {clean(h['build'])}, no node_modules (the others: {clean(h['buildOthers'])})\n"
        f"- PDF, cold: {clean(h['pdf'])} faster than Puppeteer and Chrome\n"
        f"- Runtime, geometric mean against vanilla JS: {clean(h['runtime'])}\n"
        f"- Tokens for an agent to build the Pokédex app: {clean(h['agentTokens'])} (the twelve others: {clean(h['agentTokensOthers'])})",
        table(data["lighthouse"], words["bench.lighthouse.caption"]),
        capabilities(data["capabilities"], words),
        table(data["toolchain"], words["bench.toolchain.caption"]),
        table(data["runtime"], words["bench.runtime.caption"]),
        table(data["pdf"], words["bench.pdf.caption"]),
    ]
    agents = ["| " + " | ".join(words[k] for k in ["bench.framework", "bench.ag.tokens", "bench.ag.cost", "bench.ag.wall", "bench.ag.turns", "bench.ag.loc", "bench.ag.passed"]) + " |",
              "|---" * 7 + "|"]
    agents += [f"| {r['label']} | {r['tokens']} | {r['cost']} | {r['wall']} | {r['turns']} | {r['loc']} | {r['passed']} |" for r in data["agentsTable"]]
    out.append(f"## {words['bench.agents.table']}\n\n" + "\n".join(agents))
    out.append("Why the gap: today's models were trained on years of React, Svelte and Next.js code and on none of "
               "WebFluent, so an agent learns the language as it goes — reading the skill and checking with the "
               "compiler. Every app passed every test; the extra tokens are those turns, not failed builds. Better "
               f"docs and tools cut the cost from 35M to {clean(h['agentTokens'])} tokens; the rest closes as models are trained on "
               "WebFluent, which is why its code is published as an open dataset: "
               "https://huggingface.co/datasets/monzeromer-lab/webfluent-code")
    return "\n\n".join(out)


def main():
    twins = build()
    twins.write()

    # twins.json: every twin, for a tool.
    rows = [{"route": r, "twin": twin_path(r), "title": t, "description": d, "part": p, "lang": lang}
            for r, (t, d, b, p, lang) in sorted(twins.pages.items())]
    (OUT / "twins.json").write_text(json.dumps(rows, indent=1, ensure_ascii=False) + "\n")

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
        rows = [(r, v[0], v[1]) for r, v in twins.pages.items() if v[3] == part and not r.startswith("/ar/")]
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
        for r, (t, d, b, p, _) in sorted(twins.pages.items()):
            if p == part:
                full += [f"\n\n---\n\n# {t}\n", f"{SITE}{r}\n", b]
    for r in ("/reference/expressions", "/reference/grammar", "/reference/project", "/reference/built-ins",
              "/reference/runtime", "/reference/cli", "/reference/config", "/reference/tokens", "/errors"):
        if r in twins.pages:
            t, d, b, p, _ = twins.pages[r]
            full += [f"\n\n---\n\n# {t}\n", f"{SITE}{r}\n", b]
    (OUT / "llms-full.txt").write_text("\n".join(full) + "\n")
    print(f"wrote {len(twins.pages)} twins, twins.json, llms.txt and llms-full.txt")


def ask_in_site(*args):
    """`wf theme --json` reads the site's theme, so it runs in `site/`."""
    return json.loads(subprocess.check_output([str(Path(wf).resolve()) if Path(str(wf)).exists() else str(wf), *args, "--json"], cwd=ROOT / "site"))


# ─── The compiler's data, as Markdown ─────────────────────────────────────


def part(p, owner) -> list:
    """A part, as a section of its owner's twin."""
    out = [f"### {p['name']}", "", p["plain"], "", "```wf", p["signature"], "```", ""]
    if p["props"]:
        out += ["| Prop | Type | Meaning |", "|---|---|---|"]
        out += [f"| `{r['name']}` | `{r['type']}` | {r['meaning']} |" for r in p["props"]] + [""]
    if p["flags"]:
        out += ["Flags: " + " ".join(f"`{f}`" for f in p["flags"]), ""]
    if p["events"]:
        out += ["Events: " + ", ".join(f"`{e['name']}`" for e in p["events"]), ""]
    if p.get("example") and p["example"].strip() != (owner.get("example") or "").strip():
        out += ["```wf", p["example"], "```", ""]
    return out


def moved(c, to: str, label: str, older: bool) -> str:
    """The twin of a page that sends the reader on: a part's, or an older
    spelling's."""
    if older:
        return f"`{c['name']}` is an older spelling of [`{label}`]({to}), which new code uses.\n"
    return f"`{c['name']}` is a part of [`{label}`]({to}), and is described on its page.\n"


def component(c, parts=()) -> str:
    out = [c["plain"], "", "```wf", c["signature"], "```", ""]
    if c.get("example"):
        out += ["## Example", "", "```wf", c["example"], "```", ""]
    if c.get("modern", ("", ""))[0]:
        modern, to = c["modern"]
        out += [f"Use [`{modern}`]({to}) in new code: it is the current spelling, and its page has the example.", ""]
    if c["props"]:
        out += ["## Props", "", "| Prop | Type | Meaning |", "|---|---|---|"]
        out += [f"| `{p['name']}` | `{p['type']}` | {p['meaning']} |" for p in c["props"]]
        out.append("")
    if c["flags"]:
        out += ["## Flags", "", " ".join(f"`{f}`" for f in c["flags"]), ""]
    if c["events"]:
        out += ["## Events", ""] + [f"- `{e['name']}`{': ' + e['detail'] if e.get('detail') else ''}" for e in c["events"]] + [""]
    if c.get("domEvents"):
        out += ["## DOM events it answers", "", "Any other DOM event works too, in an `on … { }` handler.", ""]
        out += [f"- `on {e['name']}`{': ' + e['detail'] if e.get('detail') else ''}" for e in c["domEvents"]] + [""]
    out += ["Every element also takes the props, events and attribute families listed "
            "[on every element](/reference#on-every-element).", ""]
    if c["slots"]:
        out += ["## Slots", ""] + [f"- `{s['name']}`{': ' + s['detail'] if s.get('detail') else ''}" for s in c["slots"]] + [""]
    drawn = [p for p in parts if p["plain"].rstrip().endswith("Drawn for it.")]
    written = [p for p in parts if p not in drawn]
    if written:
        out += ["## Parts", "", f"Written under its name, inside `{c['name']}`.", ""]
        for p in written:
            out += part(p, c)
    if drawn:
        out += ["## Drawn for it", "", f"What `{c['name']}` draws itself, by its class; not written.", ""]
        out += [f"- `{p['name']}` (`.{p['cssClass']}`): {p['plain'].replace(' Drawn for it.', '')}" for p in drawn] + [""]
    if c["codes"]:
        out += ["## What it can raise", ""] + [f"- [{k['kind']}]({k['to']}): {k['title']}" for k in c["codes"]] + [""]
    if c.get("since") == reference.OLDER:
        out += [f"Before WebFluent {reference.OLDER}.", ""]
    elif c.get("since"):
        out += [f"Since WebFluent {c['since']}.", ""]
    return "\n".join(out)


def command(c, expand) -> str:
    out = [c["summary"], "", "```bash", c["usage"], "```", ""]
    if c["options"]:
        out += ["## Options", "", "| Option | Meaning |", "|---|---|"]
        for o in c["options"]:
            out.append(f"| `{o.get('flag') or o.get('name', '')}` | {o.get('meaning') or o.get('help', '')} |")
        out.append("")
    for sc in c.get("subcommands") or []:
        out += [f"## wf {c['name']} {sc['name']}", "", sc["summary"], "", "```bash", sc["usage"], "```", ""]
        if sc["options"]:
            out += ["| Option | Meaning |", "|---|---|"]
            out += [f"| `{o.get('name', '')}` | {o.get('meaning', '')} |" for o in sc["options"]]
            out.append("")
    if c.get("writes"):
        out += [f"**Writes:** {c['writes']}", ""]
    if c.get("preview"):
        out += [f"**Preview:** `{c['preview']}` says what it would do without doing it.", ""]
    if words(c["slug"]):
        out += [markdown(words(c["slug"]), with_info=True, expand=expand), ""]
    return "\n".join(out)


if __name__ == "__main__":
    main()
