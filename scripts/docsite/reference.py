#!/usr/bin/env python3
"""Write the reference section, from the compiler's own answers.

Run in `site/`, so `wf theme --json` reads the site's theme. It writes the
pages into `site/src/pages/reference/`, each one's words written into the
page itself, so they travel in that page's chunk (a `data` file would be
inlined into every page's `app.js`):

- `Builtins.wf` — `wf registry --json`: `/reference`, every built-in by
  family, and a page per built-in (`/reference/<slug>`, parts and the
  original spellings included) with its Stage, props, flags, events, slots,
  parts and example, the release it arrived in (`since_of`, from
  `RELEASE_NOTES.md`) and the codes it can raise (`codes_of`, from
  `builtin_codes.json` and `wf explain --json`); `BUILTIN_INDEX` and
  `REFERENCE_PAGES`, the sidebars' lists, which those pages share.
- `Cli.wf` — `wf commands --json`: `/reference/cli`, with the CLI chapter's
  prose under its list, and a page per command.
- `Config.wf` — `wf config schema --json`: `/reference/config`, every key
  by section, with the configuration chapter's prose.
- `Tokens.wf` — `wf theme --json`: `/reference/tokens`, the baseline
  theme's tokens by group, with the design tokens chapter's prose.

The same data goes to `site/src/data/{components,commands,config,tokens}.json`
for tools. Rerun after the registry, a command, a setting, the theme or one
of those chapters changes: `cargo xtask site`.
"""
import importlib.util
import json
import os
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts" / "docsite"))
from examples import EXAMPLES  # noqa: E402

# `WF_REFERENCE_OUT` writes into a scratch directory (`cargo xtask site
# --check`); `WF_BIN` names the `wf` to ask.
OUT = Path(os.environ["WF_REFERENCE_OUT"]) if os.environ.get("WF_REFERENCE_OUT") else None
wf = Path(os.environ["WF_BIN"]) if os.environ.get("WF_BIN") else ROOT / "target" / "debug" / "wf"
if not wf.exists():
    wf = "wf"

# The families the index groups the built-ins under, in order: the
# registry's groups, with its parts under their owner's and the original
# spellings last.
FAMILIES = ["Layout", "Navigation", "Routing", "Data display", "Form", "Feedback", "Actions",
            "Media", "Typography", "PDF", "Slides"]
LEGACY = "Original spellings"
# The controls a Stage draws, at most.
CONTROLS = 6
# What a built-in that arrived before the notes' 5.0 release says it is since.
OLDER = "5.0"
NOTES = ROOT / "RELEASE_NOTES.md"
# Built-ins the notes first name where they were reworked, not where they
# arrived: v4.3.3 already has each of them.
REWORKED = {"Menu", "Menu.Item", "Document", "Header", "Footer", "PageBreak", "Presentation"}
# The notes' sections that change what is there rather than add to it.
NOT_ADDED = ("Fix", "Upgrad", "Migrat", "Breaking", "For tools", "Tooling", "Editors", "The editor",
             "Security", "Behaviour", "Change")
CODES = ROOT / "scripts" / "docsite" / "builtin_codes.json"


def ask(*args) -> dict:
    return json.loads(subprocess.check_output([str(wf), *args, "--json"], cwd=ROOT / "site"))


def slug_of(c: dict) -> str:
    return c["qualified"].lower().replace(".", "-")


def legacy(c: dict) -> bool:
    s = c["summary"]
    return "original spelling" in s or "original grammar" in s


def prop_row(p: dict, positional: bool = False) -> dict:
    if p.get("cases"):
        ty = " ".join(f".{c['name']}" for c in p["cases"])
    else:
        ty = p["type"]
    if positional:
        ty += "  (positional)"
    elif p.get("required"):
        ty += "  (required)"
    return {"name": p["name"] if positional else f"{p['name']}:", "type": ty, "meaning": p["summary"]}


def signature(c: dict) -> str:
    pos = c.get("positional")
    args = ([f"_ {pos['name']}: {pos['type']}"] if pos else []) + [
        f"{p['name']}: {'Enum' if p.get('cases') else p['type']}" for p in c["props"][:3]
    ]
    if len(c["props"]) > 3:
        args.append("…")
    sig = c["qualified"] + ("(" + ", ".join(args) + ")" if args else "")
    if c["children"] == "elements":
        sig += " { … }"
    return sig


def control(p: dict) -> dict | None:
    """The control a Stage draws for a prop: a segmented choice for an enum,
    a switch for a `Bool`, a field for a `String`; none for anything else."""
    if p.get("cases"):
        cases = [c["name"] for c in p["cases"]]
        chosen = next((c["name"] for c in p["cases"] if c.get("default")), "")
        return {"name": p["name"], "kind": "enum", "options": cases[:5], "value": chosen}
    if p["type"] == "Bool":
        return {"name": p["name"], "kind": "bool", "options": [], "value": ""}
    if p["type"] == "String":
        return {"name": p["name"], "kind": "string", "options": [], "value": ""}
    return None


def outputs_of(c: dict) -> list[str]:
    return {"PDF": ["pdf"], "Slides": ["slides"]}.get(c["group"], [])


def releases() -> list:
    """Each release in `RELEASE_NOTES.md`, oldest first: its version and
    its sections, as (heading, text)."""
    parts = re.split(r"^# WebFluent v(\S+) Release Notes", NOTES.read_text(), flags=re.M)
    out = []
    for version, body in zip(parts[1::2], parts[2::2]):
        sections = [(s.split("\n", 1)[0].strip(), s) for s in re.split(r"^## ", body, flags=re.M)]
        out.append((version, sections))
    return out[::-1]


def version_key(v: str) -> tuple:
    return tuple(int(n) for n in v.split("."))


def mentions(text: str, name: str) -> bool:
    """Whether `text` names the built-in in code: `Name`, `Name(…)`,
    `Name { … }` or `Name.flag`."""
    return any(s == name or s.startswith((name + "(", name + " ", name + "."))
               for s in re.findall(r"`([^`]+)`", text))


def since_of(name: str, notes: list) -> str:
    """The first release whose notes add the built-in: the first that names
    it, when that is in its Additions. One the notes name before 5.0, or
    first name where it changes (a fix, an upgrade note: it was there
    already), is older than the notes say: `OLDER`."""
    if name in REWORKED:
        return OLDER
    for version, sections in notes:
        hits = [h for h, text in sections if mentions(text, name)]
        if not hits:
            continue
        if version_key(version) < version_key(OLDER) or all(h.startswith(NOT_ADDED) for h in hits):
            return OLDER
        return version
    return OLDER


def codes_of(c: dict, table: dict, titles: dict) -> list:
    """The codes a built-in can raise, in the compiler's order, as links."""
    mine = set(table["every"])
    if c["owner"]:
        mine |= set(table["parts"])
    if not c.get("positional"):
        mine |= set(table["named_only"])
    mine |= {code for code, names in table["codes"].items() if c["qualified"] in names}
    return [{"kind": code, "title": titles[code], "to": f"/errors/{code}"} for code in titles if code in mine]


def components(reg: dict) -> dict:
    comps = reg["components"]
    by_name = {c["qualified"]: c for c in comps}
    notes = releases()
    table = json.loads(CODES.read_text())
    titles = {e["code"]: e["title"].replace("`", "") for e in json.loads(
        subprocess.check_output([str(wf), "explain", "--json"], cwd=ROOT / "site"))}
    since = {c["qualified"]: since_of(c["qualified"], notes) for c in comps}
    # A part is never older than its owner.
    for c in comps:
        if c["owner"] and version_key(since[c["owner"]]) > version_key(since[c["qualified"]]):
            since[c["qualified"]] = since[c["owner"]]
    items = []
    for c in comps:
        owner = by_name.get(c["owner"]) if c["owner"] else None
        family = LEGACY if legacy(c) else (owner["group"] if owner else c["group"])
        pos = c.get("positional")
        props = ([prop_row(pos, True)] if pos else []) + [prop_row(p) for p in c["props"]]
        controls = [k for k in (control(p) for p in ([pos] if pos else []) + c["props"]) if k][:CONTROLS]
        parts = [by_name[f"{c['qualified']}.{p}"] for p in c["parts"] if f"{c['qualified']}.{p}" in by_name]
        example = EXAMPLES.get(c["qualified"]) or (EXAMPLES.get(owner["qualified"], "") if owner else "")
        items.append({
            "name": c["qualified"],
            "slug": slug_of(c),
            "family": family,
            "owner": owner["qualified"] if owner else "",
            "ownerSlug": slug_of(owner) if owner else "",
            "summary": c["summary"],
            "plain": c["summary"].replace("`", ""),
            "signature": signature(c),
            "block": c["children"] == "elements",
            "legacy": legacy(c),
            "props": props,
            "controls": controls,
            "flags": [f".{f['name']}" for f in c["flags"] if f["prop"] not in ("animate", "speed")],
            "events": [{"name": e, "kind": "", "detail": "", "fallback": ""} for e in c["events"]],
            "slots": [{"name": s, "kind": "", "detail": "", "fallback": ""} for s in c["slots"]],
            "parts": [{"name": p["qualified"], "kind": slug_of(p), "detail": p["summary"].replace("`", ""), "fallback": ""}
                      for p in parts],
            "attributes": c.get("attributes") or [],
            "cssClass": c.get("class") or "",
            "example": example,
            "preview": c["qualified"] in PREVIEWS or bool(owner and owner["qualified"] in PREVIEWS),
            "outputs": outputs_of(owner or c),
            "since": since[c["qualified"]],
            "codes": codes_of(c, table, titles),
        })
    order = {f: i for i, f in enumerate(FAMILIES + [LEGACY])}
    unknown = sorted({i["family"] for i in items} - set(order))
    if unknown:
        raise SystemExit(f"reference.py: no family order for {unknown}; add them to FAMILIES")
    # Each family's built-ins, then their parts beside them, in the
    # registry's order.
    rank = {c["qualified"]: n for n, c in enumerate(comps)}
    items.sort(key=lambda i: (order[i["family"]], rank[i["owner"] or i["name"]], i["owner"] != "", rank[i["name"]]))
    families = [f for f in FAMILIES + [LEGACY] if any(i["family"] == f for i in items)]
    universal = {
        "props": [prop_row(p) for p in reg["universal"]["props"]],
        "events": [{"name": e["name"], "kind": "", "detail": e["summary"], "fallback": ""} for e in reg["universal"]["events"]],
    }
    return {"count": len(items), "families": families, "items": items, "universal": universal, "icons": reg["icons"]}


def option_row(o: dict) -> dict:
    if o["positional"]:
        name = o["value_name"] if o["required"] else f"[{o['value_name']}]"
    else:
        name = f"--{o['long']}" + (f", -{o['short']}" if o["short"] else "")
        if not o["flag"]:
            name += f" {o['value_name']}"
    if o["values"]:
        ty = " | ".join(o["values"])
    else:
        ty = "flag" if o["takes"] == "nothing" else o["takes"]
    if o["repeats"]:
        ty += ", repeats"
    meaning = o["help"] or ("The project's folder" if o["name"] == "dir" else "")
    if o["default"] not in (None, "") and "default" not in meaning.lower():
        meaning = f"{meaning} (default `{o['default']}`)".strip()
    return {"name": name, "type": ty, "meaning": meaning}


def usage(c: dict, parent: str = "") -> str:
    parts = ["wf"] + ([parent] if parent else []) + [c["name"]]
    if c.get("subcommands"):
        parts.append("<" + "|".join(s["name"] for s in c["subcommands"]) + ">")
    for o in c["options"]:
        if o["positional"]:
            parts.append(o["value_name"] if o["required"] else f"[{o['value_name']}]")
    if any(not o["positional"] for o in c["options"]):
        parts.append("[options]")
    return " ".join(parts)


def commands(data: dict) -> dict:
    items = []
    for c in data["commands"]:
        items.append({
            "name": c["name"],
            "slug": c["name"],
            "summary": c["summary"],
            "writes": c.get("writes") or "",
            "preview": c.get("preview") or "",
            "usage": usage(c),
            "options": [option_row(o) for o in c["options"]],
            # `wf config get|set|…`: each its own usage and options.
            "subcommands": [{
                "name": sc["name"],
                "summary": sc["summary"],
                "usage": usage(sc, c["name"]),
                "options": [option_row(o) for o in sc["options"]],
            } for sc in c.get("subcommands") or []],
        })
    return {"count": len(items), "items": items}


def subsections(c: dict) -> str:
    """The on-this-page list of a command's page: its options, or one
    entry per subcommand."""
    if c["subcommands"]:
        return anchors([(sc["name"], q("wf " + c["name"] + " " + sc["name"])) for sc in c["subcommands"]])
    return '[Anchor(id: "options", label: t("cli.options"))]'



def shown(value) -> str:
    if isinstance(value, str):
        return value
    return json.dumps(value, ensure_ascii=False)


def config(data: list) -> dict:
    sections = []
    for k in data:
        if k["section"] not in sections:
            sections.append(k["section"])
    items = []
    for k in data:
        ty = k["type"]
        if k.get("values"):
            ty = " | ".join(json.dumps(v) for v in k["values"])
        if "min" in k or "max" in k:
            ty += f" ({k.get('min', '')}–{k.get('max', '')})"
        items.append({
            "path": k["path"],
            "slug": k["path"].replace(".", "-").replace("*", "any"),
            "section": k["section"],
            "type": ty,
            "default": shown(k["default"]),
            "summary": k["summary"],
        })
    return {"count": len(items), "sections": sections, "items": items}


def tokens(data: dict) -> dict:
    # The baseline theme's tokens: what every project starts with. A token
    # only the site declares is the site's, not the language's.
    items = []
    for t in data["tokens"]:
        if t["baseline"] is None:
            continue
        items.append({
            "name": t["name"],
            "group": t["group"],
            "value": t["baseline"],
            "color": t["group"] == "color",
            "drawnBy": ", ".join(t["drawn_by"][:6]),
        })
    groups = []
    for t in items:
        if t["group"] not in groups:
            groups.append(t["group"])
    return {"count": len(items), "groups": groups, "items": items}


# What the Stage draws for the built-ins that need nothing around them, and
# the states it binds. A part is drawn by its owner's. Each is a fragment of
# the page, so it uses nothing the site does not have (no picture from
# `public/`, no route the site lacks, no tone whose label fails the theme's
# contrast check). The rest show their example's code until the compiler
# runs in the browser.
PREVIEWS = {
    "Container": ('Container { Text("Centred, at most the content width") }', []),
    "Row": ('Row(gap: .md, align: .center) { Badge("One")  Badge("Two")  Badge("Three") }', []),
    "Stack": ('Stack(gap: .sm) { Badge("First")  Badge("Second") }', []),
    "Grid": ('Grid(columns: 3, gap: .sm) { Card { Text("One") }  Card { Text("Two") }  Card { Text("Three") } }', []),
    "Divider": ('Stack(gap: .sm) { Text("Above")  Divider  Text("Below") }', []),
    "Breadcrumb": ('Breadcrumb { Breadcrumb.Item(to: "/reference") { Text("Reference") }  Breadcrumb.Item { Text("Breadcrumb") } }', []),
    "Link": ('Link("Back to the reference", to: "/reference")', []),
    "Tabs": ('Tabs { Tabs.Page("Profile") { Text("Your details") }  Tabs.Page("Billing") { Text("Your plan") } }', []),
    "Card": ('Card.elevated { Card.Header { Text("Pro").bold }  Card.Body { Text("Everything in Free, and more.") }  Card.Footer { Button("Choose Pro").primary } }', []),
    "Table": ('Table(caption: "Deploys") { Table.Head { Table.Row { Table.Cell("Region")  Table.Cell("Status") } }  Table.Body { Table.Row { Table.Cell("fra1")  Table.Cell("Healthy") } } }', []),
    "List": ('List { List.Item { Text("Milk") }  List.Item { Text("Eggs") } }', []),
    "Badge": ('Row(gap: .sm) { Badge("Beta").info  Badge("3 new").primary.pill }', []),
    "Tag": ('Tag("design")', []),
    "Avatar": ('Avatar(initials: "AL").primary.lg', []),
    "Tooltip": ('Tooltip("Copies the link") { Button("Copy") }', []),
    "Input": ('Input(bind: text, label: "Email", hint: "We never share it").email', ["text"]),
    "Select": ('Select(bind: plan, label: "Plan") { Select.Option("Free", value: "free")  Select.Option("Team", value: "team") }', ["plan"]),
    "Checkbox": ('Checkbox(bind: checked, label: "I agree to the terms")', ["checked"]),
    "Radio": ('Stack(gap: .xs) { Radio(bind: size, value: "s", label: "Small")  Radio(bind: size, value: "m", label: "Medium") }', ["size"]),
    "Switch": ('Switch(bind: checked, label: "Email me about replies")', ["checked"]),
    "Slider": ('Slider(bind: level, min: 0, max: 100, step: 5, label: "Volume")', ["level"]),
    "Textarea": ('Textarea(bind: text, label: "Notes", rows: 3, maxLength: 200)', ["text"]),
    "Alert": ('Alert("Your changes were saved.").info', []),
    "Spinner": ('Spinner.lg', []),
    "Progress": ('Progress(value: 40, max: 100)', []),
    "Skeleton": ('Skeleton(height: "20px", width: "200px")', []),
    "Button": ('Button("Save changes").primary', []),
    "IconButton": ('IconButton(icon: "edit", label: "Edit the title").primary', []),
    "ButtonGroup": ('ButtonGroup { Button("Day")  Button("Week")  Button("Month") }', []),
    "Icon": ('Row(gap: .md) { Icon("search")  Icon("check").lg }', []),
    "Text": ('Text("Hello, reader").bold', []),
    "Heading": ('Heading("A heading").h2', []),
    "Code": ('Code("const x = 1")', []),
    "Blockquote": ('Blockquote { Text("Simple is not the same as easy.") }', []),
    "Markdown": ('Markdown("Some **bold** words and `code`.")', []),
    "QrCode": ('QrCode("https://webfluent.monzeromer.dev")', []),
}
STATES = {"text": '""', "checked": "true", "level": "50", "plan": '"free"', "size": '"m"'}

_spec = importlib.util.spec_from_file_location("guide", ROOT / "scripts" / "docsite" / "guide.py")
guide = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(guide)
S = guide.wf_string


def q(text: str) -> str:
    """`text` as a WebFluent string literal."""
    return f'"{S(text)}"'


def pascal(text: str) -> str:
    return "".join(w[:1].upper() + w[1:] for w in re.split(r"[^A-Za-z0-9]+", text) if w)


def anchors(pairs) -> str:
    """`(id, label)` pairs as a list of `Anchor`s; a label already escaped
    (from guide.prose) is passed as it is."""
    return "[" + ", ".join(f'Anchor(id: "{i}", label: {label if label.startswith(("t(", chr(34))) else q(label)})'
                           for i, label in pairs) + "]"


def describe(text: str) -> str:
    """A search result's snippet: at most about 155 characters, at a word."""
    text = text.replace("`", "").strip()
    if len(text) <= 155:
        return text
    return text[:152].rsplit(" ", 1)[0].rstrip(",;:") + "…"


def rows(items: list) -> str:
    return "[" + ", ".join(f"{{ name: {q(r['name'])}, type: {q(r['type'])}, meaning: {q(r['meaning'])} }}" for r in items) + "]"


def prop_rows(items: list) -> str:
    return "[" + ", ".join(f"PropRow(name: {q(r['name'])}, kind: {q(r['kind'])}, detail: {q(r['detail'])})" for r in items) + "]"


def wf_map(v) -> str:
    if isinstance(v, list):
        return "[" + ", ".join(wf_map(x) for x in v) + "]"
    if isinstance(v, dict):
        return "{ " + ", ".join(f"{k}: {wf_map(x)}" for k, x in v.items()) + " }"
    return q(str(v))


HEAD = """// {what}: written by scripts/docsite/reference.py from `{source}`;
// edit the compiler or the script, not this file.
"""


def entries_const(name: str, entries: list) -> str:
    body = ",\n".join(f"    IndexEntry(label: {q(l)}, to: {q(t)}, group: {q(g)})" for l, t, g in entries)
    return f"const {name}: [IndexEntry] = [\n{body}\n]\n"


REFERENCE_PAGES = [
    ("Expressions", "/reference/expressions", "Language"),
    ("Built-in functions and values", "/reference/built-ins", "Language"),
    ("Grammar", "/reference/grammar", "Language"),
    ("Runtime API", "/reference/runtime", "Language"),
    ("Command line", "/reference/cli", "Tools"),
    ("Configuration", "/reference/config", "Tools"),
    ("Design tokens", "/reference/tokens", "Tools"),
    ("Project structure", "/reference/project", "Tools"),
    ("Browser support", "/reference/browser-support", "Tools"),
    ("Glossary", "/reference/glossary", "Tools"),
]


def builtin_page(c: dict, by_slug: dict) -> list:
    out = [f'page Builtin{pascal(c["slug"])}(path: "/reference/{c["slug"]}", title: {q(c["name"] + " — Reference")}, '
           f'description: {q(describe(c["name"] + ": " + c["plain"]))}) {{']
    preview = PREVIEWS.get(c["name"]) or PREVIEWS.get(c["owner"])
    if preview:
        for st in preview[1]:
            out.append(f"    state {st} = {STATES[st]}")
    sections = anchors([("props", 't("ref.tab.props")'), ("flags", 't("ref.tab.flags")'), ("events", 't("ref.tab.events")'),
                        ("slots", 't("ref.slotsparts")'), ("example", 't("ref.example")')])
    outputs = "[" + ", ".join(q(o) for o in c["outputs"]) + "]"
    codes = "[" + ", ".join(
        f'RelatedLink(kind: {q(k["kind"])}, title: {q(k["title"])}, to: {q(k["to"])})' for k in c["codes"]) + "]"
    since = "" if c["since"] == OLDER else f'since: {q(c["since"])}, '
    older = "older: true, " if c["since"] == OLDER else ""
    out.append(f'    ReferenceItemShell({q(c["name"])}, summary: {q(c["plain"])}, signature: {q(c["signature"])}, '
               f'{since}{older}outputs: {outputs}, group: {q(c["family"])}, entries: BUILTIN_INDEX, sections: {sections}, '
               f'codes: {codes}, source: "src/registry/builtins.rs") {{')
    out.append("        stage {")
    out.append(f'            Stage(label: {q(c["name"])}) {{')
    out.append("                controls {")
    for k in c["controls"]:
        out.append(f"                    StageControl({wf_map(k)})")
    if not c["controls"]:
        out.append('                    Text(t("stage.nocontrols"), class: "bp-small")')
    out.append("                }")
    if preview:
        out.append(f"                {preview[0]}")
    else:
        out.append('                Text(t("stage.later"), class: "bp-small") { style { max-width: 420px; text-align: center } }')
    out.append("            }")
    out.append("        }")
    out.append('        notes { Text(t("stage.static"), class: "bp-small") }')
    if c["owner"]:
        out.append(f'        Markdown(t("ref.partof", {{ owner: {q(c["owner"])}, to: "/reference/{c["ownerSlug"]}" }}), class: "bp-prose")')
    out.append('        Heading(t("ref.tab.props"), id: "props").h2')
    if c["props"]:
        out.append(f'        PropsTable(rows: {rows(c["props"])}, caption: {q("Props of " + c["name"])})')
    else:
        out.append('        Text(t("ref.noprops"), class: "bp-small")')
    out.append(f'        Markdown(t("ref.attrs", {{ families: {q(", ".join(c["attributes"]))} }}), class: "bp-prose")')
    out.append('        Heading(t("ref.tab.flags"), id: "flags").h2')
    out.append('        Markdown(t("ref.flags.body"), class: "bp-prose")')
    if c["flags"]:
        out.append("        Row(gap: .sm) {")
        out.append("            style { gap: 8px; flex-wrap: wrap }")
        for f in c["flags"]:
            out.append(f'            Text({q(f)}, class: "bp-label bp-mono")')
        out.append("        }")
    else:
        out.append('        Text(t("ref.noflags"), class: "bp-small")')
    out.append('        Heading(t("ref.tab.events"), id: "events").h2')
    if c["events"]:
        out.append(f'        EventsTable(rows: {prop_rows(c["events"])}, caption: {q("Events of " + c["name"])})')
    out.append('        Markdown(t("ref.universal.link"), class: "bp-prose")')
    out.append('        Heading(t("ref.slotsparts"), id: "slots").h2')
    out.append(f'        SlotsAndParts(slots: {prop_rows(c["slots"])}, parts: {prop_rows(c["parts"])})')
    if c["parts"]:
        out.append("        Row(gap: .sm) {")
        out.append("            style { gap: 8px; flex-wrap: wrap }")
        for p in c["parts"]:
            out.append(f'            Link({q(p["name"])}, to: "/reference/{p["kind"]}", class: "bp-button")')
        out.append("        }")
    out.append('        Heading(t("ref.example"), id: "example").h2')
    if c["example"]:
        out.append(f'        CodeBlock({guide.wf_raw(c["example"])}, title: "wf")')
    else:
        out.append('        Text(t("ref.noexample"), class: "bp-small")')
    if c["cssClass"]:
        out.append(f'        Markdown(t("ref.renders", {{ cls: {q(c["cssClass"])} }}), class: "bp-prose")')
    out.append("    }")
    out.append("}")
    return out


def builtins_file(comps: dict) -> str:
    items = comps["items"]
    by_slug = {i["slug"]: i for i in items}
    out = [HEAD.format(what="The reference's index and a page per built-in", source="wf registry --json")]
    out.append("/// The reference's pages that are not one built-in, in the order the index\n/// and the sidebars list them.")
    out.append(entries_const("REFERENCE_PAGES", REFERENCE_PAGES))
    out.append("/// Every built-in, by family: the sidebar of each built-in's page.")
    out.append(entries_const("BUILTIN_INDEX", [(i["name"], f"/reference/{i['slug']}", i["family"]) for i in items]))
    fam_id = lambda f: f.lower().replace(" ", "-")
    sections = anchors([("pages", 't("ref.pages")')] + [(fam_id(f), f) for f in comps["families"]]
                       + [("on-every-element", 't("ref.universal.title")')])
    index = [
        'page ReferenceIndexPage(path: "/reference", title: "Reference", description: "Every built-in element, command, setting and design token, read from the compiler itself, with the language\'s reference pages.") {',
        f'    ReferenceItemShell(t("nav.reference"), summary: t("ref.lede", {{ count: {comps["count"]} }}), entries: REFERENCE_PAGES, sections: {sections}, source: "scripts/docsite/reference.py") {{',
        '        Heading(t("ref.pages"), id: "pages").h2',
        "        Grid(columns: { base: 1, sm: 2, xl: 3 }, gap: .sm) {",
    ]
    for label, to, group in REFERENCE_PAGES:
        index.append(f'            Link(to: "{to}", class: "bp-card") {{ Text({q(label)}) {{ style {{ font-weight: 600 }} }}  Text({q(group)}, class: "bp-small") }}')
    index.append("        }")
    for fam in comps["families"]:
        index.append(f'        Heading({q(fam)}, id: "{fam_id(fam)}").h2')
        index.append("        Grid(columns: { base: 1, sm: 2, xl: 3 }, gap: .sm) {")
        for c in items:
            if c["family"] == fam:
                index.append(f'            Link(to: "/reference/{c["slug"]}", class: "bp-card") {{')
                index.append(f'                Text({q(c["name"])}, class: "bp-mono") {{ style {{ font-size: 14px; color: $link }} }}')
                index.append(f'                Text({q(c["plain"])}, class: "bp-small") {{ style {{ overflow: hidden; display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical }} }}')
                index.append("            }")
        index.append("        }")
    uni = comps["universal"]
    index += [
        '        Heading(t("ref.universal.title"), id: "on-every-element").h2',
        '        Markdown(t("ref.universal.body"), class: "bp-prose")',
        f'        PropsTable(rows: {rows(uni["props"])}, caption: t("ref.universal.title"))',
        f'        EventsTable(rows: {prop_rows(uni["events"])}, caption: t("ref.events"))',
        '        Heading(t("ref.icons")).h3 { style { margin: 8px 0 0; font-size: 18px; font-weight: 600 } }',
        '        Text(t("ref.icons.body"), class: "bp-small")',
        "        Row(gap: .sm) {",
        "            style { gap: 8px; flex-wrap: wrap }",
    ]
    for icon in comps["icons"]:
        index.append(f'            Row(align: .center, gap: .xs, class: "bp-label") {{ Icon({q(icon)})  Text({q(icon)}) }}')
    index += ["        }", "    }", "}"]
    out.append("\n".join(index) + "\n")
    for c in items:
        out.append("\n".join(builtin_page(c, by_slug)) + "\n")
    return "\n".join(out)


def prose_block(route: str) -> tuple[list, list]:
    """The chapter at `route`, as the lines of a `notes { }` fill, and its
    sections."""
    sections, body = guide.prose(route)
    lines = ["        notes {", "            Stack(gap: .lg) {"] + body + ["            }", "        }"]
    return sections, lines


def cli_file(cmds: dict) -> str:
    out = [HEAD.format(what="The command line's reference: every command, then a page per command", source="wf commands --json")]
    out.append("/// Every command: the sidebar of each command's page.")
    out.append(entries_const("CLI_INDEX", [(f"wf {c['name']}", f"/reference/cli/{c['slug']}", "Commands") for c in cmds["items"]]))
    prose_sections, prose = prose_block("/reference/cli")
    page = [
        'page CliIndexPage(path: "/reference/cli", title: "Command line — Reference", description: "Every wf command and its options, from init, build and serve to render, migrate and verify, and the language server in each editor.") {',
        f'    ReferenceItemShell(t("cli.title"), summary: t("cli.lede", {{ count: {cmds["count"]} }}), signature: "wf <command> [options]", group: t("cli.title"), '
        f'entries: REFERENCE_PAGES.concat(CLI_INDEX), sections: {anchors([("commands", "t(\"cli.commands\")")] + prose_sections)}, source: "md-docs/reference/cli.md") {{',
        *prose,
        '        Heading(t("cli.commands"), id: "commands").h2',
        '        Stack(class: "bp-sheet") {',
        "            style { gap: 0 }",
    ]
    for c in cmds["items"]:
        page += [
            f'            Link(to: "/reference/cli/{c["slug"]}") {{',
            "                style { display: flex; align-items: baseline; gap: 16px; min-height: 44px; padding: 8px 14px; border-bottom: 1px solid $line; color: $ink; text-decoration: none; &:last-child { border-bottom: 0 } &:hover { background: $raised; text-decoration: none } & p { margin: 0 } }",
            f'                Text({q("wf " + c["name"])}, class: "bp-mono") {{ style {{ flex-shrink: 0; width: 140px; font-size: 13px; color: $link }} }}',
            f'                Text({q(c["summary"])}, class: "bp-small")',
            "            }",
        ]
    page += ["        }", "    }", "}"]
    out.append("\n".join(page) + "\n")
    for c in cmds["items"]:
        p = [
            f'page Cli{pascal(c["slug"])}(path: "/reference/cli/{c["slug"]}", title: {q("wf " + c["name"] + " — Reference")}, '
            f'description: {q(describe("wf " + c["name"] + ": " + c["summary"] + ". What it writes, and every option it takes."))}) {{',
            f'    ReferenceItemShell({q("wf " + c["name"])}, summary: {q(c["summary"])}, signature: {q(c["usage"])}, group: t("cli.title"), '
            f'entries: CLI_INDEX, sections: {subsections(c)}, source: "src/cli/app.rs") {{',
        ]
        if c["writes"]:
            p.append(f'        Markdown(t("cli.writes", {{ what: {q(c["writes"])} }}), class: "bp-prose")')
        if c["preview"]:
            p.append(f'        Markdown(t("cli.preview", {{ flag: {q(c["preview"])} }}), class: "bp-prose")')
        for sc in c["subcommands"]:
            p.append(f'        Heading({q("wf " + c["name"] + " " + sc["name"])}, id: {q(sc["name"])}).h2')
            p.append(f'        Text({q(sc["summary"])})')
            p.append(f'        Code({q(sc["usage"])}).block')
            if sc["options"]:
                p.append(f'        PropsTable(rows: {rows(sc["options"])}, caption: {q("Options of wf " + c["name"] + " " + sc["name"])}, nameLabel: t("cli.option"))')
        if not c["subcommands"]:
            p.append('        Heading(t("cli.options"), id: "options").h2')
            if c["options"]:
                p.append(f'        PropsTable(rows: {rows(c["options"])}, caption: {q("Options of wf " + c["name"])}, nameLabel: t("cli.option"))')
            else:
                p.append('        Text(t("cli.noopts"), class: "bp-small")')
        p += ['        Link(t("cli.all"), to: "/reference/cli")', "    }", "}"]
        out.append("\n".join(p) + "\n")
    return "\n".join(out)


def table(caption: str, heads: list, body: list) -> list:
    out = [f'        Table(caption: {q(caption)}, class: "bp-table") {{', "            Table.Head {", "                Table.Row {"]
    out += [f"                    Table.Cell({h}).header" for h in heads]
    out += ["                }", "            }", "            Table.Body {"]
    for cells in body:
        out.append("                Table.Row {")
        out += [f"                    {cell}" for cell in cells]
        out.append("                }")
    out += ["            }", "        }"]
    return out


def config_file(cfg: dict) -> str:
    out = [HEAD.format(what="Every key of webfluent.app.json, by section", source="wf config schema --json")]
    prose_sections, prose = prose_block("/reference/config")
    sections = [(f"keys-{s}", s) for s in cfg["sections"]] + prose_sections
    page = [
        'page ConfigPage(path: "/reference/config", title: "Configuration — Reference", description: "Every key of webfluent.app.json, its type and default: project, build, media, PDF, slides, dev, motion, meta, theme, i18n, env, offline and lints.") {',
        f'    ReferenceItemShell(t("config.title"), summary: t("config.lede", {{ count: {cfg["count"]} }}), signature: "webfluent.app.json", group: t("config.title"), '
        f'entries: REFERENCE_PAGES, sections: {anchors(sections)}, source: "md-docs/reference/configuration.md") {{',
        *prose,
    ]
    for s in cfg["sections"]:
        page.append(f'        Heading({q(s)}, id: "keys-{s}", class: "bp-mono").h2')
        body = [[f"Table.Cell {{ Code({q(k['path'])}) }}", f"Table.Cell {{ Code({q(k['type'])}) }}",
                 f"Table.Cell {{ Code({q(k['default'] or chr(34) * 2)}) }}", f"Table.Cell({q(k['summary'])})"]
                for k in cfg["items"] if k["section"] == s]
        page += table(f"The {s} keys", ['t("config.key")', 't("ref.type")', 't("config.default")', 't("ref.meaning")'], body)
    page += ["    }", "}"]
    out.append("\n".join(page) + "\n")
    return "\n".join(out)


def tokens_file(tok: dict) -> str:
    out = [HEAD.format(what="Every token of the baseline theme, by group", source="wf theme --json")]
    prose_sections, prose = prose_block("/reference/tokens")
    sections = [(f"tokens-{g}", g) for g in tok["groups"]] + prose_sections
    page = [
        'page TokensPage(path: "/reference/tokens", title: "Design tokens — Reference", description: "Every baseline design token with its default value, how a theme overrides one, and how a style block or a stylesheet reads it, by full or short name.") {',
        f'    ReferenceItemShell(t("tokens.title"), summary: t("tokens.lede", {{ count: {tok["count"]} }}), signature: "theme Brand \\{{ color-primary: #3B82F6 \\}}", group: t("tokens.title"), '
        f'entries: REFERENCE_PAGES, sections: {anchors(sections)}, source: "md-docs/reference/design-tokens.md") {{',
        *prose,
    ]
    for g in tok["groups"]:
        page.append(f'        Heading({q(g)}, id: "tokens-{g}", class: "bp-mono").h2')
        body = []
        for t in tok["items"]:
            if t["group"] != g:
                continue
            swatch = ""
            if t["color"]:
                swatch = f'Text("", aria-hidden: "true") {{ style {{ flex-shrink: 0; width: 14px; height: 14px; border: 1px solid $line; border-radius: 3px; background: {t["value"]} }} }}  '
            body.append([f"Table.Cell {{ Code({q('$' + t['name'])}) }}",
                         f"Table.Cell {{ Row(align: .center, gap: .xs) {{ style {{ flex-wrap: nowrap; gap: 8px; & p {{ margin: 0 }} }} {swatch}Code({q(t['value'])}) }} }}",
                         f"Table.Cell({q(t['drawnBy'])})"])
        page += table(f"The {g} tokens", ['t("tokens.token")', 't("tokens.value")', 't("tokens.drawn")'], body)
    page += ["    }", "}"]
    out.append("\n".join(page) + "\n")
    return "\n".join(out)


def main() -> None:
    reg = ask("registry")
    comps = components(reg)
    cmds = commands(ask("commands"))
    cfg = config(ask("config", "schema"))
    tok = tokens(ask("theme"))
    data_dir = OUT or ROOT / "site" / "src" / "data"
    pages_dir = OUT or ROOT / "site" / "src" / "pages" / "reference"
    for d in (data_dir, pages_dir):
        d.mkdir(parents=True, exist_ok=True)
    for name, value in (("components.json", comps), ("commands.json", cmds), ("config.json", cfg), ("tokens.json", tok)):
        (data_dir / name).write_text(json.dumps(value, indent=1, ensure_ascii=False) + "\n")
    pages = {"Builtins.wf": builtins_file(comps), "Cli.wf": cli_file(cmds),
             "Config.wf": config_file(cfg), "Tokens.wf": tokens_file(tok)}
    for name, text in pages.items():
        (pages_dir / name).write_text(text)
        subprocess.run([str(wf), "fmt", str(pages_dir / name)], check=False, stdout=subprocess.DEVNULL)
    print(f"wrote the reference's pages to {pages_dir} and its data to {data_dir}")


if __name__ == "__main__":
    main()
