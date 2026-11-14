#!/usr/bin/env python3
"""Write the guide's components reference from `wf registry --json`.

The registry is the compiler's own description of every built-in, so the
reference can never drift from what the compiler accepts. Rerun after the
registry changes: `cargo xtask site`.
"""
import json
import os
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts" / "docsite"))
from examples import EXAMPLES  # noqa: E402
# The page is `md-docs/reference/components.md`; its front matter is its own,
# kept as it is.
TARGET = ROOT / "md-docs" / "reference" / "components.md"
# `WF_COMPONENTS_OUT` writes the page elsewhere, for `cargo xtask site --check`.
OUT_PATH = Path(os.environ["WF_COMPONENTS_OUT"]) if os.environ.get("WF_COMPONENTS_OUT") else TARGET

wf = Path(os.environ["WF_BIN"]) if os.environ.get("WF_BIN") else ROOT / "target" / "debug" / "wf"
if not wf.exists():
    wf = "wf"
data = json.loads(subprocess.check_output([str(wf), "registry", "--json"]))
components = data["components"]
SITE = json.loads((ROOT / "site" / "webfluent.app.json").read_text())["meta"]["site_url"].rstrip("/")
# The props every element takes. A flag of one (`.fadeIn`, `.trapFocus`) is
# listed once, under "On every element", not on each component.
UNIVERSAL = {p["name"] for p in data["universal"]["props"]}


def own_flags(c):
    mine = {p["name"] for p in c["props"]}
    return [f"`.{f['name']}`" for f in c["flags"] if f["prop"] in mine or f["prop"] not in UNIVERSAL]


def legacy(c):
    s = c["summary"]
    return "original spelling" in s or "original grammar" in s

def esc(t):
    return t.replace("|", "\\|")

def prop_line(p):
    ty = p["type"]
    if p.get("cases"):
        ty = " ".join(f"`.{c['name']}`" for c in p["cases"])
    else:
        ty = f"`{ty}`"
    required = " (required)" if p.get("required") else ""
    return f"| `{p['name']}:`{required} | {ty} | {esc(p['summary'])} |"

out = []
out.append("# Components reference\n")
FRONT = TARGET.read_text().split("\n---\n", 1)[0] + "\n---"
out.insert(0, FRONT)
out.append("Generated from the compiler's registry (`wf registry --json`) by `scripts/docsite/components.py`, with an example of each from `scripts/docsite/examples.py`; edit those, not this file.\n")
out.append("Every component is written `Name(positional, prop: value).flag { block }`. A **prop** is passed by name; the one **positional** prop, where there is one, comes first and unnamed. A **flag** is written after the parentheses with a dot: a `Bool` prop, or a case of one of the element's enum props (`Button(\"x\").primary.lg`). An **enum prop** takes a case written `.case` (`Row(gap: .md)`). Every element also takes the universal props and events listed at the end, and the HTML attribute families named on it (`aria-*`, `data-*`, and the global attributes such as `id`, `class`, `title`, `role`, `tabindex`).\n")

groups = []
for c in components:
    if c["owner"] is None and not legacy(c) and c["group"] not in groups:
        groups.append(c["group"])

out.append("## Contents\n")
for g in groups:
    names = [c["name"] for c in components if c["owner"] is None and not legacy(c) and c["group"] == g]
    # Each entry is the built-in's own page, by its whole address: this file
    # is published as the Markdown twin of `/reference`, whose HTML has no
    # section per built-in, and a guide page links another site page only
    # by its URL.
    out.append(f"- **{g}**: " + ", ".join(f"[`{n}`]({SITE}/reference/{n.lower()})" for n in names))
out.append("")

for g in groups:
    out.append(f"## {g}\n")
    for c in components:
        if c["owner"] is not None or legacy(c) or c["group"] != g:
            continue
        name = c["name"]
        out.append(f"### {name}\n")
        out.append(esc(c["summary"]) + "\n")
        pos = c.get("positional")
        sig = name
        args = []
        if pos:
            args.append(pos["name"])
        args += [f"{p['name']}: …" for p in c["props"][:3]]
        if args:
            sig += "(" + ", ".join(args) + ")"
        if c["children"] == "elements":
            sig += " { … }"
        out.append(f"```\n{sig}\n```\n")
        example = EXAMPLES.get(name)
        if example is None:
            sys.exit(f"scripts/docsite/examples.py has no example of {name}")
        out.append(f"```wf\n{example}\n```\n")
        if c.get("class"):
            out.append(f"Renders with the class `{c['class']}`.\n")
        rows = []
        if pos:
            rows.append(f"| `{pos['name']}` (positional) | `{pos['type']}` | {esc(pos['summary'])} |")
        for p in c["props"]:
            rows.append(prop_line(p))
        if rows:
            out.append("| Prop | Type | Meaning |\n|---|---|---|")
            out.extend(rows)
            out.append("")
        # The flags every element takes are listed once, at the end.
        flags = own_flags(c)
        if flags:
            out.append("**Flags:** " + " ".join(flags) + "\n")
        if c["events"]:
            out.append("**Events of its own:** " + ", ".join(f"`on {e}`" for e in c["events"]) + "\n")
        parts = [p for p in components if p["owner"] == name]
        if parts:
            out.append("**Parts:**\n")
            for p in parts:
                psig = f"{name}.{p['name']}"
                ppos = p.get("positional")
                pargs = ([ppos["name"]] if ppos else []) + [f"{q['name']}: …" for q in p["props"]]
                if pargs:
                    psig += "(" + ", ".join(pargs) + ")"
                if p["children"] == "elements":
                    psig += " { … }"
                out.append(f"- `{psig}` — {esc(p['summary'])}")
                for q in ([ppos] if ppos else []) + p["props"]:
                    ty = " ".join(f"`.{cs['name']}`" for cs in q["cases"]) if q.get("cases") else f"`{q['type']}`"
                    out.append(f"  - `{q['name']}:` {ty} — {esc(q['summary'])}")
                pflags = own_flags(p)
                if pflags:
                    out.append("  - flags: " + " ".join(pflags))
            out.append("")
        if c["children"] == "elements":
            out.append("Takes a block of children.\n")
        attrs = c.get("attributes") or []
        if attrs:
            out.append("Attribute families: " + ", ".join(f"`{a}`" for a in attrs) + ".\n")

out.append("## On every element\n")
out.append("| Prop | Type | Meaning |\n|---|---|---|")
for p in data["universal"]["props"]:
    out.append(prop_line(p))
out.append("\nThe cases of `animate:` and `speed:` are also flags on every element: `.fadeIn` … `.spin`, `.fast` `.normal` `.slow`; "
           "so are the `Bool` props above, `.dropInside` and `.trapFocus`.\n")
out.append("\nEvents every element accepts in an `on … { }` handler:\n")
for e in data["universal"]["events"]:
    out.append(f"- `on {e['name']}` — {esc(e['summary'])}")
out.append("")
out.append("## Icons\n")
out.append("The names `Icon(…)`, `IconButton(icon:)` and `Sidebar.Item(icon:)` draw; any other name shows as the word, and the compiler warns.\n")
out.append(" ".join(f"`{i}`" for i in data["icons"]) + "\n")

out.append("## Next\n")
out.append("[CLI](cli.md).\n")
OUT_PATH.write_text("\n".join(out))
print(f"wrote {OUT_PATH}")
