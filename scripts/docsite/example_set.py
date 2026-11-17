"""The reference's examples, one entry per built-in: the main example — drawn
live on the built-in's page and shown as its code — any variations, and an
example for a part its owner's does not show.

`examples.py` holds the first, short ones (a string each); a file of
`examples_by_family/` holds a family's richer ones and wins. An entry is

    "Card": {
        "code": '…',                         # the example; its leading
                                             # `state …` lines are the page's
        "variations": [("Outlined", '…')],   # shown below it, as code
        "parts": {"Card.Header": '…'},       # a part's own, when needed
        "preview": False,                    # only to draw no preview
    }

Every one is a fragment of a page. Its preview is drawn on the docs site, so
it names only what the site has: a route of the site (`/reference`,
`/learn`), a picture of its `public/` (`/og.png`, `/icon-512.png`).
"""

import importlib.util
from pathlib import Path

from examples import EXAMPLES as FIRST

HERE = Path(__file__).resolve().parent


def _load() -> dict:
    # The first, short examples are shown as code; only a family's entry,
    # written to be drawn on the site, is drawn too.
    out = {name: {"code": code, "preview": False} for name, code in FIRST.items()}
    for f in sorted((HERE / "examples_by_family").glob("*.py")):
        spec = importlib.util.spec_from_file_location(f"examples_{f.stem}", f)
        mod = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(mod)
        for name, e in mod.EXAMPLES.items():
            out[name] = {"code": e} if isinstance(e, str) else dict(e)
    return out


ALL = _load()


def entry(name: str) -> dict:
    """What `name` is shown with: `code`, `variations`, `parts`, `preview`."""
    e = ALL.get(name) or {}
    return {
        "code": (e.get("code") or "").strip("\n"),
        "variations": [(t, c.strip("\n")) for t, c in e.get("variations", [])],
        "parts": {k: v.strip("\n") for k, v in e.get("parts", {}).items()},
        "preview": e.get("preview", True),
    }


DECLARES = ("state ", "persist ", "derived ", "validate ")


def _depth(line: str) -> int:
    """How many brackets a line opens and leaves open, strings aside."""
    depth, quoted, escaped = 0, False, False
    for ch in line:
        if escaped:
            escaped = False
        elif ch == "\\":
            escaped = True
        elif ch == '"':
            quoted = not quoted
        elif not quoted and ch in "([{":
            depth += 1
        elif not quoted and ch in ")]}":
            depth -= 1
    return depth


def split_states(code: str) -> tuple[list, str]:
    """An example's leading declarations — `state`, `persist`, `derived`,
    `validate`, each over as many lines as its brackets take — which the
    page declares, and the rest, which the stage draws."""
    lines = code.strip("\n").split("\n")
    states, i = [], 0
    while i < len(lines) and lines[i].lstrip().startswith(DECLARES):
        block, depth = [lines[i].strip()], _depth(lines[i])
        i += 1
        while depth > 0 and i < len(lines):
            block.append(lines[i].rstrip())
            depth += _depth(lines[i])
            i += 1
        states.append("\n".join(block))
    return states, "\n".join(lines[i:]).strip("\n")
