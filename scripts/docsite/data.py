#!/usr/bin/env python3
"""Write the documentation site's search index.

`site/public/search-index.json`: every chapter, section, component and
diagnostic code, with the terms each section is about, for the header's
search. It is a file the site fetches the first time the search box is
used, not part of every page's bundle. The reference's own data is
`reference.py`'s.

Rerun after the registry or the guide changes: `cargo xtask site`.
"""
import json
import os
import re
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
# `WF_SITE_DATA_OUT` and `WF_BIN` let `tests/docs_parse.rs` write these into a
# scratch directory with the binary it just built, and hold the committed
# files to the result.
SITE = Path(os.environ["WF_SITE_DATA_OUT"]) if os.environ.get("WF_SITE_DATA_OUT") else ROOT / "site" / "src"
wf = Path(os.environ["WF_BIN"]) if os.environ.get("WF_BIN") else ROOT / "target" / "debug" / "wf"
if not wf.exists():
    wf = "wf"

import importlib.util
import sys
spec = importlib.util.spec_from_file_location("guide", ROOT / "scripts" / "docsite" / "guide.py")
guide = importlib.util.module_from_spec(spec)
spec.loader.exec_module(guide)
sys.path.insert(0, str(ROOT / "scripts" / "docsite"))
import reference  # noqa: E402


def builtins():
    """Every built-in, as the reference pages list it (`reference.py`)."""
    reg = json.loads(subprocess.check_output([str(wf), "registry", "--json"]))
    return reference.components(reg)["items"]


STOP = set("""about after again also always another because before being between both cannot could does doesn during each either every first from have here into itself just like made make many more most much must need never only other over same should since some such than that their them then there these they this those through under until very what when where which while with within without would your yours""".split())


def terms(text: str, cap: int = 24) -> str:
    """The words in a section a reader might search for: what is written as
    code, capitalised names, and longer words — each once, in order, up to
    `cap`. The index is part of the site's bundle, so it stays short."""
    body = re.sub(r"```.*?```", " ", text, flags=re.S)
    # An alert's marker (`> [!TIP]`) is layout, not something to search for.
    body = re.sub(r"\[!(NOTE|TIP|IMPORTANT|WARNING|CAUTION)\]", " ", body)
    found = []
    for code in re.findall(r"`([^`]+)`", body):
        for w in re.findall(r"[A-Za-z_$@.][\w.$-]{1,}", code):
            found.append(w.strip(".").lower())
    for w in re.findall(r"[A-Za-z][A-Za-z0-9-]{2,}", re.sub(r"`[^`]*`", " ", body)):
        if (w[0].isupper() or len(w) >= 7) and w.lower() not in STOP:
            found.append(w.lower())
    seen = []
    for w in found:
        if w and w not in seen:
            seen.append(w)
    return " ".join(seen[:cap])


# What a reader coming from another framework, or from LaTeX, types, and
# the section that teaches the WebFluent way. A row holding a synonym is
# ranked with the rows whose names match, so `useState` finds `state`
# first. Each target must be a section the index has.
SYNONYMS = {
    "/learn/07-state-and-reactivity#state": ["useState", "ref", "reactive", "$state", "signal", "data()", "setState"],
    "/learn/07-state-and-reactivity#derived": ["useMemo", "computed", "$derived", "memo", "selector", "getter"],
    "/learn/07-state-and-reactivity#effect": ["useEffect", "watch", "watchEffect", "$effect", "onMount", "componentDidMount", "$:"],
    "/learn/07-state-and-reactivity#persist": ["localStorage", "sessionStorage", "useLocalStorage"],
    "/learn/09-control-flow#for": ["v-for", "{#each}", "#each", ".map(", "map", "ngFor", "key"],
    "/learn/09-control-flow#if-else-if-else": ["v-if", "v-else", "{#if}", "#if", "ngIf", "ternary", "&&"],
    "/learn/09-control-flow#show": ["v-show", "display none", "hidden"],
    "/learn/09-control-flow#match": ["switch", "case", "{#await}", "pattern matching"],
    "/learn/10-components#props": ["props", "defineProps", "export let", "$props", "propTypes", "@Input"],
    "/learn/10-components#slots": ["slot", "children", "props.children", "<slot>", "v-slot", "render props", "ng-content"],
    "/learn/10-components#events": ["emit", "$emit", "defineEmits", "createEventDispatcher", "callback prop", "@Output", "onChange prop"],
    "/learn/08-events#handlers": ["onClick", "@click", "on:click", "v-on", "addEventListener", "onclick", "(click)"],
    "/learn/13-styling#add-a-class": ["className", ":class", "class:", "clsx", "classnames", "ngClass"],
    "/learn/13-styling#style-one-element": ["style={{}}", ":style", "inline style", "css-in-js", "styled-components", "emotion"],
    "/learn/13-styling#dark-mode": ["prefers-color-scheme", "dark theme", "color scheme"],
    "/learn/15-data#show-a-request-with-resource": ["fetch", "useSWR", "useQuery", "axios", "tanstack query", "react query", "loader", "{#await}"],
    "/learn/15-data#describe-a-service-with-api": ["axios instance", "api client", "openapi", "swagger", "trpc"],
    "/learn/11-stores#a-first-store": ["context", "useContext", "provide", "inject", "redux", "zustand", "pinia", "vuex", "writable", "global state"],
    "/learn/14-forms#bind-a-control-to-state": ["v-model", "bind:value", "controlled input", "onChange", "two-way binding", "ngModel"],
    "/learn/14-forms#validate-a-field": ["yup", "zod", "formik", "react-hook-form", "vee-validate", "validation"],
    "/learn/05-pages-and-routing#declaring-a-page": ["react-router", "vue-router", "sveltekit", "next.js", "route", "routes", "<Route>"],
    "/learn/05-pages-and-routing#links-and-navigation": ["<a href>", "router-link", "useNavigate", "router.push", "goto"],
    "/learn/05-pages-and-routing#per-page-head-tags": ["<head>", "helmet", "useHead", "svelte:head", "meta tags"],
    "/learn/17-pdf-and-slides#make-a-pdf-document": ["\\documentclass", "\\section", "#set page", "latex", "typst", "pdf"],
    "/learn/17-pdf-and-slides#control-where-pages-break": ["\\newpage", "#pagebreak", "page break"],
    "/learn/17-pdf-and-slides#set-the-paper-headers-and-footers": ["fancyhdr", "header footer", "page number"],
    "/learn/17-pdf-and-slides#make-a-slide-deck": ["beamer", "powerpoint", "keynote", "reveal.js", "slides"],
    "/learn/16-testing#before-you-start": ["jest", "vitest", "testing library", "cypress", "playwright", "getByText"],
}

# The search's tabs: the part of the site a row is in.
KIND_OF_FOLDER = {"learn": "learn", "about": "learn", "reference": "reference", "errors": "errors"}


def search_index():
    rows = []
    for f in guide.chapter_files():
        meta = guide.read_meta(f)
        num = f"{meta['chapter']:02d}"
        route = meta["route"]
        md = guide.front_matter(f)[1]
        title = meta["title"]
        if route == guide.REFERENCE_ROUTE:
            continue  # its entries come from the registry, below
        kind = KIND_OF_FOLDER.get(meta["path"].split("/")[0], "learn")
        # Only the book's chapters show a number beside a result.
        if kind != "learn" or meta["path"].startswith("about/"):
            num = ""
        rows.append({"chapter": num, "title": title, "section": "", "route": guide.href(route), "kind": kind, "words": terms(meta["description"])})
        # Every `##` and `###` section, with what its text is about.
        parts = re.split(r"^(#{2,3}) (.+)$", md, flags=re.M)
        for k in range(1, len(parts), 3):
            text = guide.plain(parts[k + 1])
            if text in ("Next", "Contents"):
                continue
            rows.append({
                "chapter": num,
                "title": title,
                "section": text,
                "route": f"{guide.href(route)}#{guide.slug(text)}",
                "kind": kind,
                "words": terms(parts[k + 2]),
            })
    # Every recipe, by its title and what its text is about.
    for f in sorted((ROOT / "md-docs" / "recipes").glob("*.md")):
        meta, md = guide.front_matter(f)
        if f.name == "index.md":
            continue
        title = next(l[2:].strip() for l in md.splitlines() if l.startswith("# "))
        rows.append({"chapter": "", "title": "Recipes", "section": title, "route": meta["route"], "kind": "recipes",
                     "words": terms(meta.get("description", "") + " " + md)})
    # Every cheat sheet, by the framework it starts from.
    for f in sorted((ROOT / "md-docs" / "coming-from").glob("*.md")):
        meta, md = guide.front_matter(f)
        title = next(l[2:].strip() for l in md.splitlines() if l.startswith("# "))
        rows.append({"chapter": "", "title": "Coming from", "section": title, "route": meta["route"], "kind": "learn",
                     "also": meta.get("framework", ""), "words": terms(meta.get("description", "") + " " + md)})
    # Every diagnostic, by its code and what it says: `T05` jumps to its page.
    for f in sorted((ROOT / "md-docs" / "errors").glob("*.md")):
        meta, md = guide.front_matter(f)
        if "code" not in meta:
            continue
        title = next(l[2:].strip() for l in md.splitlines() if l.startswith("# "))
        rows.append({"chapter": "", "title": "Errors", "section": title, "route": meta["route"], "kind": "errors",
                     "words": terms(meta.get("description", ""))})
    # Every command.
    for c in json.loads(subprocess.check_output([str(wf), "commands", "--json"]))["commands"]:
        rows.append({"chapter": "", "title": "Commands", "section": f"wf {c['name']}", "route": f"/reference/cli/{c['name']}",
                     "kind": "reference", "words": terms(c["summary"])})
    # Every built-in, by name, with what it takes.
    for c in builtins():
        if c["legacy"]:
            continue
        rows.append({
            "chapter": "",
            "title": "Components reference",
            "section": c["name"],
            "route": f"/reference/{c['slug']}",
            "kind": "reference",
            "words": terms(c["summary"] + " " + " ".join(f"`{p['name']}`" for p in c["props"])),
        })
    by_route = {r["route"]: r for r in rows}
    for route, also in SYNONYMS.items():
        if route not in by_route:
            raise SystemExit(f"scripts/docsite/data.py: a synonym names {route}, which the index does not have")
        by_route[route]["also"] = "|".join(a.lower() for a in also)
    return rows


if __name__ == "__main__":
    # Fetched on demand, so it is written compact, into `public/`.
    public = SITE if os.environ.get("WF_SITE_DATA_OUT") else ROOT / "site" / "public"
    public.mkdir(parents=True, exist_ok=True)
    (public / "search-index.json").write_text(json.dumps(search_index(), separators=(",", ":"), ensure_ascii=False) + "\n")
    print(f"wrote {public / 'search-index.json'}")
