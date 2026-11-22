## Core Syntax

One lexeme, one meaning. Declarations are lowercase keywords; elements are
capitalised; a `.flag` sets a boolean prop or picks a case; `name: value`
passes a prop; a block is the element's children; `on click { }` is a
handler; `style { }` is raw CSS. What a component takes — its props, cases,
events, slots and parts — comes from one registry the compiler, the
linters and the language server all read, and anything that resolves to
nothing is an error, never a silent no-op.

A program's own names may be in any script, Arabic included: a letter,
then letters, digits, `_` and the marks of its letters (UAX #31), one name
however it was typed (NFC) — `state عددٌ = 3`, `store المهام`. A hidden
direction control in the source is E119 (Trojan Source).

<!-- core -->
