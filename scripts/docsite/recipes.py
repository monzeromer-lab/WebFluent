#!/usr/bin/env python3
"""Write the recipes from `md-docs/recipes/`.

- `site/src/pages/recipes/<Name>.wf`: a page per recipe, framed by
  `RecipeShell`. A recipe is written: its title, a paragraph that says what
  it does, the complete code (its first fenced block, before any `##`), then
  `## How it works` and, when it has them, `## Variations`. Other sections
  follow how it works, under their own headings.
- `site/src/pages/recipes/Index.wf`: `/recipes`, every recipe by group, with
  the groups' own lines from `md-docs/recipes/index.md`; and
  `RECIPE_INDEX`, the recipes' sidebar.
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
    "Interaction": "interaction", "Data": "data", "Real time": "realtime",
    "Content": "content", "Look and motion": "look", "Reach": "reach",
    "Ship": "ship", "Interop": "interop", "Projects": "projects",
}


# A group's key, as front matter writes it, and its name, as the index and
# the sidebar show it.
NAMES = {key: name for name, key in GROUPS.items()}


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
    blocks = list(re.finditer(r"(?ms)^```[a-z][^\n]*\n.*?^```[ \t]*$", head))
    if blocks:
        code = head[blocks[0].start():blocks[-1].end()]
        intro = head[: blocks[0].start()].strip()
        after = head[blocks[-1].end():].strip()
    else:
        intro, after = head.strip(), ""
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
    related = [e for e in index_entries if e[2] == NAMES.get(group) and e[1] != meta["route"]][:3]
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
        f"hasVariations: {'true' if variations else 'false'}) {{",
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


def index_page(entries):
    meta, body = guide.front_matter(RECIPES / "index.md")
    guide.CURRENT["dir"] = RECIPES
    lead, *groups = re.split(r"(?m)^## ", re.sub(r"^# .*\n", "", body.lstrip("\n"), count=1))
    out = [
        "// The recipes, by group: written from md-docs/recipes/ by",
        "// scripts/docsite/recipes.py; edit the recipes, not this file.",
        "",
        "/// Every recipe, by group: the sidebar of each recipe's page.",
        "const RECIPE_INDEX: [IndexEntry] = [",
        ",\n".join(f"    IndexEntry(label: {q(t)}, to: {q(r)}, group: {q(g)})" for t, r, g in entries),
        "]",
        "",
        f"page Recipes(path: \"/recipes\", title: \"Recipes — WebFluent\", description: {q(meta['description'])}) {{",
        "    state text = \"\"",
        "    derived shown = RECIPE_INDEX.filter(e => text == \"\" || e.label.toLowerCase().includes(text.toLowerCase()))",
        "    SiteHeader(section: \"/recipes\")",
        "    Stack(gap: .lg, role: \"main\", id: \"bp-main\", tabindex: \"-1\", class: \"bp-main bp-wrap\") {",
        "        style { gap: 28px; padding-top: 40px; padding-bottom: 72px; @md { padding-top: 56px } }",
        "        Stack(gap: .sm) {",
        "            style { gap: 14px; max-width: 760px }",
        "            Row(class: \"bp-labels\") { KindLabel(.howto) }",
        "            Heading(t(\"nav.recipes\")).h1 { style { margin: 0; font-size: clamp(30px, 5vw, 44px); line-height: 1.08; font-weight: 650; letter-spacing: -0.025em } }",
        f"            Markdown({q(' '.join(lead.split()))}, class: \"bp-lede\")",
        "        }",
        "        Input(bind: text, placeholder: t(\"recipes.filter\"), aria-label: t(\"recipes.filter\")).search { style { max-width: 420px } }",
    ]
    for g in groups:
        heading, _, text = g.partition("\n")
        heading = heading.strip()
        key = GROUPS[heading]
        out += [
            f"        if shown.some(e => e.group == {q(heading)}) {{",
            "            Stack(gap: .md) {",
            "                style { gap: 14px }",
            f"                Heading({q(heading)}, id: \"{key}\").h2 {{ style {{ margin: 0; font-size: 22px; font-weight: 650 }} }}",
            f"                Text({q(' '.join(text.split()))}, class: \"bp-small\")",
            "                Grid(columns: { base: 1, sm: 2, lg: 3 }, gap: .md) {",
            f"                    for e in shown.filter(e => e.group == {q(heading)}) by e.to {{",
            "                        RecipeCard(e.label, to: e.to)",
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
    entries = sorted(
        ((title, meta["route"], NAMES.get(meta.get("group", ""), "")) for _, meta, title, _ in all_recipes),
        key=lambda e: (list(GROUPS).index(e[2]) if e[2] in GROUPS else 99, e[0]),
    )
    files = {}
    for path, meta, title, body in all_recipes:
        name, text = render(path, meta, title, body, entries)
        files[f"{name}.wf"] = text
    files["Index.wf"] = index_page(entries)
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
