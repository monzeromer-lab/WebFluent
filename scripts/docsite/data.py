#!/usr/bin/env python3
"""Write the documentation site's search index.

`site/public/search-index.json`: every chapter, section, component and
diagnostic code, with the terms each section is about, for the header's
search. It is a file the site fetches the first time the search box is
used, not part of every page's bundle. The reference's own data is
`reference.py`'s.

Each page's words are its Markdown twin's (`twins.py`, read from the site's
own pages); the codes and the built-ins are the compiler's. Rerun after the
registry or a page changes: `cargo xtask site`.
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

import sys
sys.path.insert(0, str(ROOT / "scripts" / "docsite"))
import reference  # noqa: E402
import sitepages  # noqa: E402
import twins as twins_md  # noqa: E402


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


def arabic_terms(text: str, cap: int = 24) -> str:
    """[`terms`], and the Arabic words of three letters or more, without
    their diacritics, each once."""
    body = re.sub(r"```.*?```", " ", text, flags=re.S)
    body = re.sub(r"[\u064B-\u0652\u0670]", "", body)
    seen = []
    for w in re.findall(r"[\u0621-\u064A]{3,}", body):
        if w not in seen:
            seen.append(w)
    return (terms(text, cap) + " " + " ".join(seen[:cap])).strip()


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
# The reference's pages of words, which are chapters of the guide as the
# pages under `pages/guide/` are (their number is their place in it).
REFERENCE_CHAPTERS = {"/reference/cli": 33, "/reference/config": 34, "/reference/tokens": 36}


def chapters() -> list:
    """The guide's chapters, in order: `(number, route)` — each page under
    `site/src/pages/guide/` numbered by its file (`07-state….wf`), but its
    translations, and the reference's pages of words."""
    import re as _re
    found = dict((r, n) for r, n in REFERENCE_CHAPTERS.items())
    for page in sitepages.pages(sitepages.PAGES / "guide"):
        m = _re.match(r"(\d\d)-", page.file.name)
        if m and not page.arg("lang"):
            found[sitepages.text_of(page.arg("path"))] = int(m.group(1))
    return sorted(((n, r) for r, n in found.items()))


def sections(md: str):
    """A page's `##` and `###` sections: `(heading, text)`."""
    parts = re.split(r"^(#{2,3}) (.+)$", md, flags=re.M)
    for k in range(1, len(parts), 3):
        yield sitepages.plain(parts[k + 1]), parts[k + 2]


# The ids `TokenBaseline` gives its groups' headings: `tokens-color`.
TOKEN_IDS = {reference.GROUP_TITLES.get(g, g.capitalize()): f"tokens-{g}"
             for g in reference.tokens(twins_md.ask_in_site("theme"))["groups"]}


def search_index():
    pages = twins_md.build().pages
    rows = []
    for n, route in chapters():
        title, description, md, _, _ = pages[route]
        num = f"{n:02d}"
        kind = KIND_OF_FOLDER.get(route.strip("/").split("/")[0], "learn")
        # Only the book's chapters show a number beside a result.
        if not route.startswith("/learn/"):
            num = ""
        rows.append({"chapter": num, "title": title, "section": "", "route": route, "kind": kind, "words": terms(description)})
        # Every `##` and `###` section, with what its text is about.
        for text, body in sections(md):
            # A group of the baseline's tokens has an id of its own.
            anchor = TOKEN_IDS.get(text, sitepages.slug(text)) if route == "/reference/tokens" else sitepages.slug(text)
            rows.append({"chapter": num, "title": title, "section": text,
                         "route": f"{route}#{anchor}", "kind": kind, "words": terms(body)})
    # Every recipe, by its title and what its text is about.
    for route, (title, description, md, part, lang) in sorted(pages.items()):
        if part == "Recipes" and route != "/recipes":
            rows.append({"chapter": "", "title": "Recipes", "section": title, "route": route, "kind": "recipes",
                         "words": terms(description + " " + md)})
    # Every cheat sheet, by the framework it starts from.
    for page in sitepages.pages(sitepages.PAGES / "comingfrom"):
        route = sitepages.text_of(page.arg("path"))
        if page.arg("lang"):
            continue
        title, description, md, _, _ = pages[route]
        rows.append({"chapter": "", "title": "Coming from", "section": title, "route": route, "kind": "learn",
                     "also": sitepages.text_of(sitepages.shell(page).arg("from", "")), "words": terms(description + " " + md)})
    # The Arabic pages, so a search written in Arabic finds them: each page
    # and its sections, with the Arabic words their text uses.
    for route, (title, description, md, part, lang) in sorted(pages.items()):
        if lang != "ar" or route in ("/ar", "/ar/playground", "/ar/benchmarks"):
            continue
        rows.append({"chapter": "", "title": "العربية", "section": title, "route": route, "kind": "learn",
                     "words": arabic_terms(description + " " + md)})
        # A cheat sheet is drawn by its shell, its sections under ids of
        # their own: the page alone is found.
        if "coming-from" in route:
            continue
        for text, body in sections(md):
            rows.append({"chapter": "", "title": title, "section": text, "route": f"{route}#{sitepages.slug(text)}",
                         "kind": "learn", "words": arabic_terms(body)})
    # The benchmarks: one row, found by what people ask of a tool.
    rows.append({"chapter": "", "title": "Benchmarks", "section": "", "route": "/benchmarks", "kind": "learn",
                 "words": "benchmark benchmarks performance lighthouse speed fast build time sitemap canonical json-ld seo "
                          "accessibility csp security runtime pdf puppeteer agent tokens compare next.js astro sveltekit nuxt react"})
    # Every diagnostic, by its code and what it says: `T05` jumps to its page.
    for e in json.loads(subprocess.check_output([str(wf), "explain", "--json"])):
        rows.append({"chapter": "", "title": "Errors", "section": f"{e['code']} — {e['title']}", "route": f"/errors/{e['code']}",
                     "kind": "errors", "words": terms(e.get("description") or e["summary"])})
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
