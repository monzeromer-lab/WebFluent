"""The documentation site's own pages, read as data.

The site's pages under `site/src/pages/` are the documentation's source:
each is a WebFluent `page` whose body is a few kinds of element — a
`Heading`, a `Markdown(…)` of prose, a `DocsCodeBlock`, a `DocsCallout`, a
`Table`, a `Terminal`, a `Quiz` — inside a shell (`DocsShell`,
`RecipeShell`, `ComingFromShell`, `ReferenceItemShell`). This module reads
those files without the compiler: `parse(text)` gives every `page`,
`component` and `const` a file declares, with each element call as a
`Call`, and `markdown(items)` writes an element tree back as the Markdown it
shows. The search index (`data.py`) and the Markdown twins (`twins.py`) are
made from what it reads; `tests/common/mod.rs` reads the same pages through
the compiler's own parser for the test suite.

It also holds the small text helpers the generators share: a WebFluent
string literal (`wf_string`, `wf_raw`), a heading's anchor (`slug`), a
heading without its marks (`plain`) and a description cut to a search
result's length (`snippet`).
"""
from __future__ import annotations

import re
import unicodedata
from dataclasses import dataclass, field
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
PAGES = ROOT / "site" / "src" / "pages"
SITE_SRC = ROOT / "site" / "src"
REPO = "https://github.com/monzeromer-lab/WebFluent"


# ─── Text helpers ─────────────────────────────────────────────────────────


def wf_string(text: str) -> str:
    """`text` as the inside of a WebFluent string literal."""
    return (
        text.replace("\\", "\\\\")
        .replace('"', '\\"')
        .replace("{", "\\{")
        .replace("}", "\\}")
        .replace("\t", "    ")
        .replace("\n", "\\n")
    )


def wf_raw(text: str) -> str:
    """`text` as a whole WebFluent raw string: `#"…"#`, with as many hashes
    as it takes for the text not to close it; a text ending in a quote, which
    no raw string can end in, as an escaped string."""
    text = text.replace("\t", "    ")
    hashes = 1
    while '"' + "#" * hashes in text:
        hashes += 1
    fence = "#" * hashes
    if text.endswith('"'):
        return '"' + wf_string(text) + '"'
    return f'{fence}"{text}"{fence}'


def slug(text: str) -> str:
    """A heading's anchor: its letters and digits, in any script (an Arabic
    heading keeps its words), lower-cased, the rest a dash — as the
    compiler's `context::anchor` makes it."""
    text = re.sub(r"`", "", text)
    text = "".join(c for c in unicodedata.normalize("NFC", text) if not unicodedata.category(c).startswith("M"))
    text = "".join(c if c.isalnum() else "-" for c in text.lower())
    text = re.sub(r"-+", "-", text).strip("-")
    return text or "section"


def plain(text: str) -> str:
    """Heading text without its Markdown marks."""
    parts = re.split(r"(`[^`]*`)", text)
    return "".join(
        p.strip("`") if p.startswith("`") else re.sub(r"\*|(?<!\w)_|_(?!\w)", "", p)
        for p in parts
    ).strip()


def snippet(text: str, limit: int = 160) -> str:
    """A description as a search result and a link preview show it: no
    Markdown marks, and no longer than a result shows."""
    text = re.sub(r"\[([^\]]*)\]\([^)]*\)", r"\1", text)
    text = re.sub(r"`([^`]*)`", r"\1", text)
    text = re.sub(r"\*\*([^*]*)\*\*|\*([^*]*)\*", lambda m: m.group(1) or m.group(2), text)
    text = " ".join(text.split())
    if len(text) <= limit:
        return text
    cut = text[:limit]
    ends = [m.end() for m in re.finditer(r"[.!?](?=\s)", cut)]
    if ends and ends[-1] >= 60:
        return cut[: ends[-1]]
    return cut[: cut.rfind(" ")].rstrip(",;—- ") + "…"


# ─── Reading a `.wf` file ─────────────────────────────────────────────────


@dataclass
class Call:
    """An element, a component or a record written as a call:
    `DocsCodeBlock(code, title: "wf")`, `Anchor(id: "x", label: "X")`."""
    name: str
    args: list = field(default_factory=list)  # (name or None, value)
    flags: list = field(default_factory=list)
    body: list | None = None  # the items of its block
    fills: dict = field(default_factory=dict)  # slot name → items
    line: int = 0

    def arg(self, name, default=None):
        for k, v in self.args:
            if k == name:
                return v
        return default

    def pos(self, i=0, default=None):
        found = [v for k, v in self.args if k is None]
        return found[i] if i < len(found) else default

    def children(self) -> list:
        return self.body or []


@dataclass
class Enum:
    """`.case`."""
    name: str


@dataclass
class Name:
    """A name, or an expression the reader keeps only as its text."""
    text: str


@dataclass
class Block:
    """A statement with a block the twin reads through — `if`, `for`,
    `else` — or a slot fill: its items."""
    keyword: str
    items: list
    head: str = ""


@dataclass
class Decl:
    kind: str  # "page", "component" or "const"
    name: str
    args: list = field(default_factory=list)
    body: list = field(default_factory=list)
    value: object = None
    line: int = 0
    file: Path | None = None

    def arg(self, name, default=None):
        for k, v in self.args:
            if k == name:
                return v
        return default


class Reader:
    def __init__(self, text: str):
        self.s = text
        self.i = 0

    # ── characters ──

    def line(self, at=None) -> int:
        return self.s.count("\n", 0, self.i if at is None else at) + 1

    def peek(self, n=0) -> str:
        j = self.i + n
        return self.s[j] if j < len(self.s) else ""

    def ws(self) -> bool:
        """Skip blanks and comments; whether a newline was among them."""
        nl = False
        while self.i < len(self.s):
            c = self.s[self.i]
            if c == "\n":
                nl = True
                self.i += 1
            elif c in " \t\r;":
                self.i += 1
            elif self.s.startswith("//", self.i):
                end = self.s.find("\n", self.i)
                self.i = len(self.s) if end < 0 else end
            elif self.s.startswith("/*", self.i):
                end = self.s.find("*/", self.i)
                self.i = len(self.s) if end < 0 else end + 2
            else:
                break
        return nl

    def ident(self) -> str:
        m = re.compile(r"[^\W\d][\w]*(?:-[^\W\d][\w]*)*").match(self.s, self.i)
        if not m:
            return ""
        self.i = m.end()
        return m.group(0)

    def peek_ident(self) -> str:
        m = re.compile(r"[^\W\d][\w]*").match(self.s, self.i)
        return m.group(0) if m else ""

    def eat(self, c: str) -> bool:
        if self.s.startswith(c, self.i):
            self.i += len(c)
            return True
        return False

    # ── strings ──

    def at_string(self) -> bool:
        if self.peek() == '"':
            return True
        j = self.i
        while j < len(self.s) and self.s[j] == "#":
            j += 1
        return j > self.i and j < len(self.s) and self.s[j] == '"'

    def string(self) -> str:
        if self.peek() == "#":
            hashes = 0
            while self.peek() == "#":
                hashes += 1
                self.i += 1
            self.i += 1  # the quote
            close = '"' + "#" * hashes
            end = self.s.find(close, self.i)
            text = self.s[self.i:end]
            self.i = end + len(close)
            return text
        if self.s.startswith('"""', self.i):
            self.i += 3
            end = self.s.find('"""', self.i)
            raw = self.s[self.i:end]
            self.i = end + 3
            lines = raw.split("\n")
            if lines and not lines[0].strip():
                lines = lines[1:]
            indent = len(lines[-1]) if lines and not lines[-1].strip() else 0
            if lines and not lines[-1].strip():
                lines = lines[:-1]
            return self.unescape("\n".join(l[indent:] for l in lines))
        self.i += 1
        out = []
        while self.i < len(self.s):
            c = self.s[self.i]
            if c == "\\":
                out.append(self.s[self.i:self.i + 2])
                self.i += 2
                continue
            if c == '"':
                self.i += 1
                break
            if c == "{":
                # A splice, kept as written.
                start = self.i
                depth = 0
                while self.i < len(self.s):
                    ch = self.s[self.i]
                    if ch == '"':
                        self.string()
                        continue
                    if ch == "{":
                        depth += 1
                    elif ch == "}":
                        depth -= 1
                        if depth == 0:
                            self.i += 1
                            break
                    self.i += 1
                out.append(self.s[start:self.i])
                continue
            out.append(c)
            self.i += 1
        return self.unescape("".join(out))

    @staticmethod
    def unescape(text: str) -> str:
        table = {"n": "\n", "t": "\t", "r": "\r", "\\": "\\", '"': '"', "{": "{", "}": "}"}
        return re.sub(r"\\(.)", lambda m: table.get(m.group(1), m.group(0)), text, flags=re.S)

    # ── skipping ──

    def skip_balanced(self, raw: bool = False):
        """From an opening bracket to its closing one. `raw` is CSS: no
        string or comment is read, only the braces counted."""
        opening = self.peek()
        closing = {"{": "}", "(": ")", "[": "]"}[opening]
        depth = 0
        while self.i < len(self.s):
            c = self.s[self.i]
            if not raw and (c == '"' or (c == "#" and self.at_string())):
                self.string()
                continue
            if not raw and self.s.startswith("//", self.i):
                self.i = self.s.find("\n", self.i)
                continue
            if c == opening:
                depth += 1
            elif c == closing:
                depth -= 1
                if depth == 0:
                    self.i += 1
                    return
            self.i += 1

    def skip_statement(self):
        """To the end of a statement this reader keeps nothing of: through
        its blocks, to the next line at the same depth."""
        while self.i < len(self.s):
            c = self.peek()
            if c in "}" or c == "":
                return
            if c in "{([":
                self.skip_balanced()
                if self.ws():
                    if self.peek_ident() != "else":
                        return
                continue
            if c == '"' or (c == "#" and self.at_string()):
                self.string()
            else:
                self.i += 1
            nl = self.ws()
            if nl and self.peek() not in ".)]}" and not self.s.startswith("=>", self.i) and self.peek_ident() != "else":
                # A line that ends an expression ends the statement.
                prev = self.s[:self.i].rstrip()[-1:]
                if prev not in "(,[{=+-*/&|?:<>!":
                    return

    # ── values ──

    def value(self):
        """One argument's value. What is not a literal, a list, a map, a
        call or a case is kept as its text."""
        self.ws()
        start = self.i
        v = self.primary()
        save = self.i
        if self.ws() and self.peek() not in ",)]}":
            # A value at the end of its line, a `const`'s.
            self.i = save
            return v
        # An expression that goes on — `a + b`, `x => …`, `a ?? b` — is text.
        if self.peek() and self.peek() not in ",)]}":
            while self.i < len(self.s) and self.peek() not in ",)]}":
                c = self.peek()
                if c in "{([":
                    self.skip_balanced()
                elif c == '"' or (c == "#" and self.at_string()):
                    self.string()
                else:
                    self.i += 1
                self.ws()
            return Name(self.s[start:self.i].strip())
        return v

    def primary(self):
        c = self.peek()
        if self.at_string():
            return self.string()
        if c == "[":
            self.i += 1
            items = []
            while True:
                self.ws()
                if self.eat("]"):
                    return items
                items.append(self.value())
                self.ws()
                self.eat(",")
        if c == "{":
            self.i += 1
            out = {}
            while True:
                self.ws()
                if self.eat("}"):
                    return out
                key = self.string() if self.at_string() else self.ident()
                self.ws()
                self.eat(":")
                out[key] = self.value()
                self.ws()
                self.eat(",")
        if c == "." and self.peek(1).isalpha():
            self.i += 1
            return Enum(self.ident())
        if c == "-" or c.isdigit():
            m = re.compile(r"-?\d+(\.\d+)?").match(self.s, self.i)
            if m:
                self.i = m.end()
                return float(m.group(0)) if "." in m.group(0) else int(m.group(0))
        name = self.ident()
        if not name:
            self.i += 1
            return Name(c)
        if name in ("true", "false"):
            return name == "true"
        if name == "null":
            return None
        return self.call_rest(name, allow_block=False)

    def call_rest(self, name: str, allow_block: bool = True):
        """What follows a name: `.Part`, `(args)`, `.flags`, `{ block }`."""
        line = self.line()
        while self.peek() == "." and self.peek(1).isupper():
            self.i += 1
            name += "." + self.ident()
        call = Call(name, line=line)
        if self.eat("("):
            while True:
                self.ws()
                if self.eat(")"):
                    break
                key = None
                save = self.i
                ident = self.ident()
                if ident and self.peek() == ":" and self.peek(1) != ":":
                    self.i += 1
                    key = ident
                else:
                    self.i = save
                call.args.append((key, self.value()))
                self.ws()
                self.eat(",")
        elif not allow_block and not name[:1].isupper():
            # A plain name read as a value: `RECIPE_INDEX`, `open`.
            return Name(name)
        while self.peek() == "." and self.peek(1).isalpha():
            self.i += 1
            call.flags.append(self.ident())
        if allow_block:
            save = self.i
            nl = self.ws()
            if not nl and self.peek() == "{":
                call.body, call.fills = self.block()
            else:
                self.i = save
        return call

    # ── blocks ──

    # Statements whose blocks hold code, not elements.
    CODE = {"on", "effect", "action", "state", "derived", "persist", "let", "every", "after", "use",
            "resource", "validate", "socket", "stream", "channel", "peer", "worker", "event", "slot",
            "part", "head", "transition", "emit", "return", "cleanup", "match", "sequence", "step"}

    def block(self):
        """`{ … }`: its items, and its slot fills by name."""
        self.eat("{")
        items, fills = [], {}
        while True:
            self.ws()
            c = self.peek()
            if c == "" or self.eat("}"):
                return items, fills
            word = self.peek_ident()
            if word == "style" or word == "transition":
                self.ident()
                self.ws()
                if self.peek() == "{":
                    self.skip_balanced(raw=True)
                continue
            if word and word[0].isupper():
                self.ident_full()
                items.append(self.call_rest(self.last_ident))
                continue
            if word in ("if", "for", "else", "show"):
                self.ident()
                start = self.i
                while self.i < len(self.s) and self.peek() != "{":
                    if self.at_string():
                        self.string()
                    elif self.peek() in "([":
                        self.skip_balanced()
                    else:
                        self.i += 1
                head = self.s[start:self.i].strip()
                inner, _ = self.block()
                items.append(Block(word, inner, head))
                continue
            if word and word not in self.CODE:
                save = self.i
                self.ident()
                self.ws()
                if self.peek() == "{":
                    inner, _ = self.block()
                    fills[word] = inner
                    continue
                self.i = save
            self.skip_statement()

    def ident_full(self):
        self.last_ident = self.ident()

    # ── the file ──

    def file(self) -> list:
        out = []
        while True:
            self.ws()
            if self.i >= len(self.s):
                return out
            line = self.line()
            word = self.ident()
            if word == "page" or word == "component":
                self.ws()
                name = self.ident()
                self.ws()
                args = []
                if word == "page" and self.eat("("):
                    while True:
                        self.ws()
                        if self.eat(")"):
                            break
                        key = self.ident()
                        self.ws()
                        self.eat(":")
                        args.append((key, self.value()))
                        self.ws()
                        self.eat(",")
                elif self.peek() == "(":
                    self.skip_balanced()
                self.ws()
                body, _ = self.block() if self.peek() == "{" else ([], {})
                out.append(Decl(word, name, args, body, line=line))
            elif word == "const":
                self.ws()
                name = self.ident()
                self.ws()
                if self.eat(":"):
                    self.ws()
                    while self.peek() not in "=":
                        if self.peek() in "[(":
                            self.skip_balanced()
                        else:
                            self.i += 1
                self.ws()
                self.eat("=")
                out.append(Decl("const", name, value=self.value(), line=line))
            elif not word:
                self.i += 1
            else:
                self.skip_statement()
                self.ws()
                if self.peek() == "{":
                    self.skip_balanced()


def parse(text: str, file: Path | None = None) -> list:
    """Every `page`, `component` and `const` a file declares."""
    decls = Reader(text).file()
    for d in decls:
        d.file = file
    return decls


def declarations(root: Path = SITE_SRC) -> list:
    """Every declaration of every `.wf` file under the site's `src/`."""
    out = []
    for path in sorted(root.rglob("*.wf")):
        out += parse(path.read_text(), path)
    return out


def pages(root: Path = PAGES) -> list:
    return [d for d in declarations(root) if d.kind == "page"]


def consts() -> dict:
    return {d.name: d.value for d in declarations() if d.kind == "const"}


# ─── Writing it back as Markdown ──────────────────────────────────────────

# A code block's title, as the language of its fence.
FENCE_OF_TITLE = {"markdown": "md"}


# The language `t("key")` is read in: the page's being written.
LANG = ["en"]


def text_of(v) -> str:
    """A value as the text it shows: a string as it is, `t("key")` in the
    language of the page being written (`LANG`), else in English."""
    if isinstance(v, str):
        return v
    if isinstance(v, Call) and v.name == "t":
        key = v.pos(0) or ""
        return translations(LANG[0]).get(key) or translations("en").get(key, key)
    if isinstance(v, (int, float)):
        return str(v)
    return ""


_TABLES = {}


def translations(lang: str) -> dict:
    """The site's translations in one language, `{}` for one it has none of."""
    if lang not in _TABLES:
        import json
        path = SITE_SRC / "translations" / f"{lang}.json"
        _TABLES[lang] = json.loads(path.read_text()) if path.exists() else {}
    return _TABLES[lang]


def code_fence(code: str, lang: str, info: str = "") -> str:
    ticks = "````" if "```" in code else "```"
    return f"{ticks}{lang}{(' ' + info) if info else ''}\n{code}\n{ticks}"


def code_block(c: Call, with_info: bool = False) -> str:
    """A `DocsCodeBlock` as a fence. `with_info` adds what the block says of
    itself — `expect=T04`, `no-check` — as the guide's fences said it."""
    title = text_of(c.arg("title", "wf")) or ""
    lang = FENCE_OF_TITLE.get(title, title)
    if "." in lang and not lang.startswith("."):
        lang = lang.rsplit(".", 1)[1]
    info = []
    if with_info:
        if c.arg("expect"):
            info.append(f"expect={c.arg('expect')}")
        if c.arg("check") is False:
            info.append("no-check")
    return code_fence(text_of(c.pos(0)), lang, " ".join(info))


def cell_text(cell: Call, expand=None) -> str:
    text = text_of(cell.pos(0)) if cell.pos(0) is not None else markdown(cell.children(), inline=True, expand=expand)
    return text.replace("\n", " ").replace("|", "\\|")


def table(c: Call, expand=None) -> str:
    rows = []
    for part in c.children():
        if not isinstance(part, Call):
            continue
        for row in part.children():
            if isinstance(row, Call) and row.name == "Table.Row":
                rows.append([cell_text(x, expand) for x in row.children() if isinstance(x, Call)])
    if not rows:
        return ""
    width = max(len(r) for r in rows)
    lines = ["| " + " | ".join(rows[0]) + " |", "|" + "---|" * width]
    lines += ["| " + " | ".join(r) + " |" for r in rows[1:]]
    return "\n".join(lines)


def quiz(c: Call) -> str:
    lines = [text_of(c.pos(0))]
    for choice in c.arg("choices") or []:
        mark = "+" if choice.arg("right") else "-"
        why = text_of(choice.arg("why", ""))
        lines.append(f"{mark} {text_of(choice.arg('text'))}" + (f" -- {why}" if why else ""))
    return code_fence("\n".join(lines), "quiz")


def markdown(items: list, with_info: bool = False, inline: bool = False, expand=None) -> str:
    """An element tree as the Markdown it shows: a heading for a `Heading`,
    the text of a `Markdown`, a fence for a `DocsCodeBlock` (and the shell
    commands of a `Terminal`), a GitHub alert for a `DocsCallout`, a pipe
    table for a `Table`, a ```` ```quiz ```` block for a `Quiz`, and the
    text of anything else, through its children. `expand(call)` may give
    the Markdown of a call drawn from data (the generators' components),
    or `None` to read it as written."""
    out = []
    for it in items:
        if isinstance(it, Block):
            out.append(markdown(it.items, with_info, inline, expand))
            continue
        if not isinstance(it, Call):
            continue
        n = it.name
        done = expand(it) if expand else None
        if done is not None:
            out.append(done)
        elif n == "Heading":
            level = next((int(f[1]) for f in it.flags if re.fullmatch(r"h[1-6]", f)), 2)
            out.append("#" * level + " " + text_of(it.pos(0)))
        elif n == "Markdown":
            out.append(text_of(it.pos(0)))
        elif n == "DocsCodeBlock":
            out.append(code_block(it, with_info))
        elif n == "Code" and "block" in it.flags:
            out.append(code_fence(text_of(it.pos(0)), text_of(it.arg("language", ""))))
        elif n == "Code" and it.pos(0) is not None:
            out.append(f"`{text_of(it.pos(0))}`")
        elif n == "Terminal":
            cmds = [text_of(l.arg("command")) for l in it.arg("lines") or [] if isinstance(l, Call) and l.arg("command") is not None]
            out.append(code_fence("\n".join(cmds), "bash"))
        elif n == "DocsCallout":
            kind = it.arg("kind")
            kind = kind.name if isinstance(kind, Enum) else "note"
            body = text_of(it.pos(0)).split("\n")
            out.append("\n".join([f"> [!{kind.upper()}]"] + [("> " + l).rstrip() for l in body]))
        elif n == "Table":
            out.append(table(it, expand))
        elif n == "Quiz":
            out.append(quiz(it))
        elif n in ("Text", "Paragraph") and it.pos(0) is not None:
            out.append(text_of(it.pos(0)))
        elif n == "Link" and it.pos(0) is not None:
            out.append(f"[{text_of(it.pos(0))}]({text_of(it.arg('to', it.arg('href', '')))})")
        else:
            inner = markdown(it.children(), with_info, inline, expand)
            if inner:
                out.append(inner)
    out = [o for o in out if o.strip()]
    return (" " if inline else "\n\n").join(out)


def headings(items: list) -> list:
    """Every heading an element tree shows, in order, as `(level, text)`:
    a `Heading`, and a `##`–`####` line of a `Markdown`'s text."""
    found = []
    for it in items:
        if isinstance(it, Block):
            found += headings(it.items)
        elif isinstance(it, Call):
            if it.name == "Heading":
                level = next((int(f[1]) for f in it.flags if re.fullmatch(r"h[1-6]", f)), 2)
                found.append((level, text_of(it.pos(0))))
            elif it.name == "Markdown":
                for m in re.finditer(r"^(#{2,4}) (.+)$", text_of(it.pos(0)), flags=re.M):
                    found.append((len(m.group(1)), m.group(2).strip()))
            else:
                found += headings(it.children())
                for fill in it.fills.values():
                    found += headings(fill)
    return found


def shell(page: Decl) -> Call | None:
    """The shell a page is drawn in: its `layout:`, or the first element of
    its body (a recipe's `RecipeShell`)."""
    layout = page.arg("layout")
    if isinstance(layout, Call):
        return layout
    for it in page.body:
        if isinstance(it, Call) and it.name.endswith("Shell"):
            return it
    return None


def body_items(page: Decl) -> list:
    """What a page shows inside its shell, in reading order: a shell's slot
    fills before its block, as the shells draw them."""
    s = shell(page)
    if s is None or isinstance(page.arg("layout"), Call):
        items = page.body
        # A `DocsShell` page draws everything in one `Stack`.
        if len(items) == 1 and isinstance(items[0], Call) and items[0].name == "Stack":
            return items[0].children()
        return items
    out = []
    for name in ("differences", "port", "code", "notes", "broken", "fixed", "note"):
        if name in s.fills:
            out.append(Block(name, s.fills[name]))
    rest = s.children()
    return out + rest
