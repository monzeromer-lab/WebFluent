"""What a code fence in the guide says about its block.

The words after the language — `page`, `live`, `no-check`, `expect=CODE`,
`break="from:to"`, `files=a,b` — mean what `Fence` in
`tests/common/mod.rs` says; the two readers take the same words.
"""
import shlex
from dataclasses import dataclass, field


@dataclass
class Fence:
    lang: str
    page: bool = False
    live: bool = False
    no_check: bool = False
    expect: str | None = None
    swap: tuple[str, str] | None = None
    files: list[str] = field(default_factory=list)

    @property
    def is_sample(self) -> bool:
        return self.expect is None and not self.no_check


def parse(info: str) -> Fence:
    """The fence an info string (what follows the backticks) describes."""
    words = shlex.split(info)
    fence = Fence(lang=words[0] if words else "")
    for w in words[1:]:
        key, sep, value = w.partition("=")
        if not sep and w == "page":
            fence.page = True
        elif not sep and w == "live":
            fence.live = True
        elif not sep and w == "no-check":
            fence.no_check = True
        elif key == "expect" and value:
            fence.expect = value
        elif key == "break" and ":" in value and not value.startswith(":"):
            a, _, b = value.partition(":")
            fence.swap = (a, b)
        elif key == "files" and value:
            fence.files = value.split(",")
        else:
            raise ValueError(f"`{w}` is no word a fence takes")
    return fence
