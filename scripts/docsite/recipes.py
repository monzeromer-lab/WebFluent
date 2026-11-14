#!/usr/bin/env python3
"""Write the recipes from `md-docs/recipes/`.

- `site/src/pages/recipes/<Name>.wf`: a page per recipe, framed by
  `RecipeShell`. A recipe is written: its title, a paragraph that says what
  it does, the complete code (its first fenced block, before any `##`), then
  `## How it works` and, when it has them, `## Variations`. Other sections
  follow how it works, under their own headings.
- `site/src/pages/recipes/Index.wf`: `/recipes`, every recipe by group, with
  the groups' own lines from `md-docs/recipes/index.md`, in the order that
  file lists them (every recipe must be listed there, under its `group:`);
  and `RECIPE_INDEX`, the recipes' sidebar.
- A recipe's `**Needs:**` paragraph, under its intro, is drawn above its
  code rather than folded into the summary.
- `site/src/pages/recipes/Moved.wf`: a page at each old chapter address the
  recipes came from, sending the reader on; and `site/src/old-links-map.js`,
  each old `#section` address and the recipe it became, which
  `site/src/old-links.js` reads before anything draws.

`WF_RECIPES_OUT` writes into that directory instead (for
`cargo xtask site --check`). Rerun with `cargo xtask site`.
"""
import importlib.util
import json
import os
import re
import subprocess
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
RECIPES = ROOT / "md-docs" / "recipes"
_spec = importlib.util.spec_from_file_location("guide", HERE / "guide.py")
guide = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(guide)

OUT = Path(os.environ["WF_RECIPES_OUT"]) if os.environ.get("WF_RECIPES_OUT") else None
wf = Path(os.environ["WF_BIN"]) if os.environ.get("WF_BIN") else ROOT / "target" / "debug" / "wf"

GROUPS = {
    "Interaction": "interaction", "Forms and data": "data", "Real time": "realtime",
    "Content": "content", "Media and themes": "media", "Motion": "motion",
    "Reach": "reach", "Testing": "testing", "Ship": "ship",
    "Desktop and phone apps": "desktop", "Interop": "interop",
    "Server rendering": "server", "Projects": "projects",
}


# A group's key, as front matter writes it, and its name, as the index and
# the sidebar show it.
NAMES = {key: name for name, key in GROUPS.items()}

# A recipe's "Needs" line: what it takes besides the page (a setting, a
# file, another recipe). It is written under the intro, and drawn above the
# code rather than folded into the summary.
NEEDS = re.compile(r"(?ms)^\*\*Needs:\*\*.*?(?=\n\s*\n|\Z)")


def index_sections():
    """`md-docs/recipes/index.md`'s groups, in order: each one's heading,
    the line that says what it holds, and its recipes' files, in the order
    the index lists them."""
    meta, body = guide.front_matter(RECIPES / "index.md")
    lead, *groups = re.split(r"(?m)^## ", re.sub(r"^# .*\n", "", body.lstrip("\n"), count=1))
    out = []
    for g in groups:
        heading, _, text = g.partition("\n")
        heading = heading.strip()
        if heading not in GROUPS:
            raise SystemExit(f"md-docs/recipes/index.md: `## {heading}` is none of {tuple(GROUPS)}")
        blurb = "\n".join(l for l in text.split("\n") if not l.lstrip().startswith("- ")).strip()
        files = re.findall(r"(?m)^\s*- \[[^\]]*\]\(([^)#]+\.md)\)", text)
        out.append((heading, blurb, files))
    return meta, lead, out


def q(text: str) -> str:
    return '"' + guide.wf_string(text) + '"'


def pascal(slug: str) -> str:
    return "".join(w.capitalize() for w in re.split(r"[^a-z0-9]+", slug) if w)


def split(body: str):
    """A recipe's body as (intro, code, how, variations, others)."""
    body = re.sub(r"^# .*\n", "", body.lstrip("\n"), count=1).strip("\n")
    head, *rest = re.split(r"(?m)^(?=## )", body)
    code = ""
    # The recipe's code is its first fenced block, in whatever language
    # the task is written in: a page, a workflow file, a server's config.
    # When it takes several files (a config and the page), the code is all
    # of them, with the lines that join them, up to the first heading.
    needs = NEEDS.search(head)
    if needs:
        head = head[: needs.start()] + head[needs.end():]
        needs = needs.group(0).strip()
    blocks = list(re.finditer(r"(?ms)^```[a-z][^\n]*\n.*?^```[ \t]*$", head))
    if blocks:
        code = head[blocks[0].start():blocks[-1].end()]
        intro = head[: blocks[0].start()].strip()
        after = head[blocks[-1].end():].strip()
    else:
        intro, after = head.strip(), ""
    if needs:
        if code:
            code = needs + "\n\n" + code
        else:
            after = (needs + "\n\n" + after).strip()
    how, variations, others = after, "", []
    has_how = False
    for sec in rest:
        heading, _, text = sec.partition("\n")
        name = heading[3:].strip()
        if name == "How it works":
            how = (how + "\n\n" + text.strip()).strip()
            has_how = True
        elif name == "Variations":
            variations = text.strip()
        else:
            others.append(sec.strip())
    return intro, code, how, variations, others, has_how


def recipes():
    out = []
    for path in sorted(RECIPES.glob("*.md")):
        if path.name == "index.md":
            continue
        meta, body = guide.front_matter(path)
        title = next(l[2:].strip() for l in body.splitlines() if l.startswith("# "))
        out.append((path, meta, title, body))
    return out


def related_recipes(route, group, body, index_entries, count=3):
    """The recipes a page's "Related" row names: the next one in its group,
    so a group reads as a path (in/out, then a list, then a sequence…), then
    the recipes its text links to, then the rest of its group."""
    by_route = {e[1]: e for e in index_entries}
    same = [e for e in index_entries if e[2] == group]
    at = next((i for i, e in enumerate(same) if e[1] == route), len(same))
    linked = [by_route[r] for r in
              (f"/recipes/{stem}" for stem in re.findall(r"\]\(([\w-]+)\.md", body))
              if r in by_route]
    out = []
    for e in same[at + 1:at + 2] + linked + same[at + 1:] + same[:at]:
        if e[1] != route and e not in out:
            out.append(e)
    return out[:count]


CONFIG_KEYS = {"name", "version", "build", "meta", "theme", "i18n", "offline", "env", "public_env", "lints", "dev", "motion"}


def project_files(code_md: str, stem: str) -> dict:
    """A recipe's code as a project to download or open: each fenced block
    becomes a file. A block takes the path the line above it names
    (`` `src/celebrate.js`: ``) or its own first-line comment
    (`// src/stores/cart.wf`); otherwise the one with `app { … }` is
    `src/App.wf`, other WebFluent is the recipe's page, and a JSON object of
    settings joins `webfluent.app.json`. What is neither a file nor a setting
    (a shell command, a server's config) stays on the page."""
    files, config, pages = {}, {"name": stem}, []
    blocks = re.findall(r"(?:^|\n)([^\n]*)\n*```([^\n]*)\n(.*?)\n```", code_md, flags=re.S)
    for before, info, text in blocks:
        lang = (info.split() or [""])[0]
        # A fragment the tests do not compile is not a file of the project.
        if "no-check" in info.split():
            continue
        # A line that introduces the block names its file: it ends in a colon
        # ("…in `src/App.wf`:"). A path mentioned in passing is not one.
        named = [] if not before.rstrip().endswith(":") else re.findall(r"`((?:src|public|tests|\.github)/[\w./-]+|webfluent\.app\.json|[\w-]+\.(?:js|mjs|css|json|wf|wfx))`", before)
        first = re.match(r"\s*(?://|#)\s*((?:src|public|tests)/[\w./-]+)\s*$", text.split("\n", 1)[0])
        path = first.group(1) if first else (named[-1] if named else "")
        if lang == "json":
            try:
                value = json.loads(text)
            except ValueError:
                continue
            if path and path != "webfluent.app.json":
                files[path] = text.rstrip() + "\n"
            elif isinstance(value, dict) and set(value) & CONFIG_KEYS:
                for k, v in value.items():
                    config[k] = {**config[k], **v} if isinstance(v, dict) and isinstance(config.get(k), dict) else v
            continue
        if lang in ("wf", "wfx"):
            if path.endswith((".wf", ".wfx")):
                files[path] = text.rstrip() + "\n"
            elif re.search(r"^app\s*\{", text, flags=re.M) and "src/App.wf" not in files:
                files["src/App.wf"] = text.rstrip() + "\n"
            else:
                pages.append(text.rstrip())
            continue
        if lang in ("js", "javascript", "css") and path:
            files[path] = text.rstrip() + "\n"
    if pages:
        files[f"src/pages/{pascal(stem)}.wf"] = "\n\n".join(pages) + "\n"
    if not any(p.endswith((".wf", ".wfx")) for p in files):
        return {}
    files["webfluent.app.json"] = json.dumps(config, indent=2) + "\n"
    return files

def checks_clean(files: dict) -> bool:
    """Whether `files`, as a project, draws no error in `wf check`: a
    download is offered only of a project that works as it lands."""
    import tempfile
    with tempfile.TemporaryDirectory() as d:
        for path, text in files.items():
            (Path(d) / path).parent.mkdir(parents=True, exist_ok=True)
            (Path(d) / path).write_text(text)
        done = subprocess.run([str(wf), "check", "-d", d, "--format", "json"], capture_output=True, text=True)
    try:
        return not any(f["severity"] == "error" for f in json.loads(done.stdout)["diagnostics"])
    except (ValueError, KeyError):
        return False


def project_map(code_md: str, stem: str) -> str:
    """[`project_files`] as a WebFluent map literal: `{}` for none, or for a
    project that would not check clean (a template rendered with data, a
    page that links to one it does not show)."""
    files = project_files(code_md or "", stem)
    if not files or not checks_clean(files):
        return "{}"
    return "{ " + ", ".join(f"{q(path)}: {guide.wf_raw(text)}" for path, text in files.items()) + " }"


def render(path, meta, title, body, index_entries):
    guide.CURRENT["dir"] = path.parent
    intro, code, how, variations, others, has_how = split(body)
    lines, sections = [], []
    code_lines = []
    if code:
        code_lines, _ = guide.body(code, title, "                ")
    rest_md = how
    if others:
        rest_md = (rest_md + "\n\n" + "\n\n".join(others)).strip()
    body_lines, body_sections = guide.body(rest_md, title, "        ") if rest_md else ([], [])
    var_lines, _ = guide.body(variations, title, "                ") if variations else ([], [])
    how_heading = bool(how) and (has_how or not others)
    if how_heading:
        sections.append(("how-it-works", "How it works"))
    sections += body_sections
    if variations:
        sections.append(("variations", "Variations"))
    group = meta.get("group", "")
    related = related_recipes(meta["route"], NAMES.get(group), body, index_entries)
    rel = ", ".join(f'RelatedLink(kind: "Recipe", title: {q(t)}, to: {q(r)})' for t, r, _ in related)
    anchors = ", ".join(f'Anchor(id: "{a}", label: {q(l)})' for a, l in sections)
    summary = guide.rewrite_links(" ".join(guide.reflow(intro.split("\n")))) if intro else meta.get("description", "")
    name = "Recipe" + pascal(path.stem)
    out = [
        f"// A recipe, written from md-docs/recipes/{path.name} by scripts/docsite/recipes.py;",
        "// edit the recipe, not this file.",
        f"page {name}(path: {q(meta['route'])}, title: {q(title + ' — Recipes')}, description: {q(meta.get('description', title))}) {{",
        f"    RecipeShell({q(title)}, summary: {q(summary)}, outputs: {json.dumps(meta.get('outputs', []))}, "
        f"since: {q(meta.get('since', ''))}, recipes: RECIPE_INDEX, sections: [{anchors}], related: [{rel}], "
        f"source: \"md-docs/recipes/{path.name}\", how: {'true' if how_heading else 'false'}, "
        f"hasVariations: {'true' if variations else 'false'}, project: {project_map(code, path.stem)}) {{",
    ]
    if code_lines:
        out.append("        code {")
        out += [l for l in code_lines if l.strip()]
        out.append("        }")
    if var_lines:
        out.append("        variations {")
        out += var_lines
        out.append("        }")
    out += body_lines
    out.append("    }")
    out.append("}")
    return name, "\n".join(out).rstrip() + "\n"


def index_page(entries, outputs):
    """The index: every recipe under its group, each card with the outputs it
    works in, and a filter by task and by output."""
    meta, lead, groups = index_sections()
    guide.CURRENT["dir"] = RECIPES
    out = [
        "// The recipes, by group: written from md-docs/recipes/ by",
        "// scripts/docsite/recipes.py; edit the recipes, not this file.",
        "",
        "/// Every recipe, by group: the sidebar of each recipe's page.",
        "const RECIPE_INDEX: [IndexEntry] = [",
        ",\n".join(f"    IndexEntry(label: {q(t)}, to: {q(r)}, group: {q(g)}, outputs: [{', '.join(q(o) for o in outputs.get(r, []))}])"
                  for t, r, g in entries),
        "]",
        "",
        f"page Recipes(path: \"/recipes\", title: \"Recipes — WebFluent\", description: {q(meta['description'])}) {{",
        "    state text = \"\"",
        "    state output = \"\"",
        "    // A recipe that names no outputs works in all of them.",
        "    derived shown = RECIPE_INDEX.filter(e => (text == \"\" || e.label.toLowerCase().includes(text.toLowerCase())) && (output == \"\" || e.outputs.length == 0 || e.outputs.includes(output)))",
        "    SiteHeader(section: \"/recipes\")",
        "    Stack(gap: .lg, role: \"main\", id: \"bp-main\", tabindex: \"-1\", class: \"bp-main bp-wrap\") {",
        "        style { gap: 28px; padding-top: 40px; padding-bottom: 72px; @md { padding-top: 56px } }",
        "        Stack(gap: .sm) {",
        "            style { gap: 14px; max-width: 760px }",
        "            Row(class: \"bp-labels\") { KindLabel(.howto) }",
        "            Heading(t(\"nav.recipes\")).h1 { style { margin: 0; font-size: clamp(30px, 5vw, 44px); line-height: 1.08; font-weight: 650; letter-spacing: -0.025em } }",
        f"            Markdown({q(' '.join(lead.split()))}, class: \"bp-lede\")",
        "        }",
        "        RecipeFilter { on change(words, picked) { text = words  output = picked } }",
        "        if shown.length == 0 {",
        "            Text(t(\"recipes.none\"), class: \"bp-small\")",
        "        }",
    ]
    for heading, text, _ in groups:
        key = GROUPS[heading]
        out += [
            f"        if shown.some(e => e.group == {q(heading)}) {{",
            "            Stack(gap: .md) {",
            "                style { gap: 14px }",
            f"                Heading({q(heading)}, id: \"{key}\").h2 {{ style {{ margin: 0; font-size: 22px; font-weight: 650 }} }}",
            f"                Text({q(' '.join(text.split()))}, class: \"bp-small\")",
            "                Grid(columns: { base: 1, sm: 2, lg: 3 }, gap: .md) {",
            f"                    for e in shown.filter(e => e.group == {q(heading)}) by e.to {{",
            "                        RecipeCard(e.label, to: e.to, outputs: e.outputs)",
            "                    }",
            "                }",
            "            }",
            "        }",
        ]
    out += ["    }", "}", ""]
    return "\n".join(out)


def moved_pages(all_recipes):
    """A page at each old chapter address — of a recipe or a cheat sheet —
    and the old sections' new homes."""
    chapters, anchors = {}, {}
    # The cheat sheets answer for the chapter they came from, too.
    sheets = []
    for path in sorted((ROOT / "md-docs" / "coming-from").glob("*.md")):
        meta, body = guide.front_matter(path)
        sheets.append((path, meta, "", body))
    for path, meta, title, body in all_recipes + sheets:
        for old in meta.get("from", []):
            page, _, anchor = old.partition("#")
            if anchor:
                anchors[old] = meta["route"]
                chapters.setdefault(page, "/recipes#" + meta.get("group", ""))
            else:
                chapters[page] = meta["route"]
    out = [
        "// The guide's chapters that became recipes: each old address sends the",
        "// reader to the recipes (an old `#section` link goes to its recipe, by",
        "// site/src/old-links.js). Written by scripts/docsite/recipes.py.",
        "",
    ]
    for page, to in sorted(chapters.items()):
        name = "Moved" + pascal(page)
        out += [
            f"page {name}(path: {q(page)}, title: \"Moved\", description: \"This chapter became recipes.\", guard: false, redirect: {q(to)}, noindex: true) {{",
            "    Heading(t(\"nav.recipes\")).h1",
            f"    Link(t(\"nav.recipes\"), to: {q(to)})",
            "}",
            "",
        ]
    js = (
        "// Each old `#section` address of a chapter that became recipes, and the\n"
        "// recipe it became: written by scripts/docsite/recipes.py, read by\n"
        "// old-links.js.\n"
        f"const OLD_LINKS = {json.dumps(dict(sorted(anchors.items())), indent=1)};\n"
    )
    return "\n".join(out), js


def main():
    all_recipes = recipes()
    # The index lists every recipe under its group, in the order a reader
    # takes them (install before deploy), and the sidebar follows it.
    _, _, sections = index_sections()
    listed = {f: (heading, i) for heading, _, files in sections for i, f in enumerate(files)}
    problems = []
    for path, meta, _, _ in all_recipes:
        group = NAMES.get(meta.get("group", ""))
        if group is None:
            problems.append(f"{path.name}: `group: {meta.get('group')}` is none of {tuple(NAMES)}")
        elif path.name not in listed:
            problems.append(f"{path.name}: md-docs/recipes/index.md does not list it under `## {group}`")
        elif listed[path.name][0] != group:
            problems.append(f"{path.name}: listed under `## {listed[path.name][0]}`, but its group is `{group}`")
    if problems:
        raise SystemExit("\n".join(problems))
    order = {path.name: listed[path.name][1] for path, *_ in all_recipes}
    entries = [
        e for _, e in sorted(
            (((list(GROUPS).index(NAMES[meta["group"]]), order[path.name]),
              (title, meta["route"], NAMES[meta["group"]])) for path, meta, title, _ in all_recipes),
        )
    ]
    files = {}
    for path, meta, title, body in all_recipes:
        name, text = render(path, meta, title, body, entries)
        files[f"{name}.wf"] = text
    files["Index.wf"] = index_page(entries, {meta["route"]: meta.get("outputs", []) for _, meta, _, _ in all_recipes})
    moved, js = moved_pages(all_recipes)
    files["Moved.wf"] = moved

    target = OUT or ROOT / "site" / "src" / "pages" / "recipes"
    target.mkdir(parents=True, exist_ok=True)
    for old in target.glob("*.wf"):
        old.unlink()
    for name, text in files.items():
        (target / name).write_text(text)
    (OUT or ROOT / "site" / "src").joinpath("old-links-map.js").write_text(js)
    subprocess.run([str(wf) if wf.exists() else "wf", "fmt", str(target)], check=False, capture_output=True)
    print(f"wrote {len(all_recipes)} recipes")


if __name__ == "__main__":
    main()
