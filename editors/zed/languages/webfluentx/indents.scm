; A block's lines are indented one level under the line that opens it,
; whether the block is braced or not; a written brace, paren or bracket
; pair indents its contents and its closing delimiter returns to the level
; of the line that opened it.

(block) @indent
(theme_body) @indent
(style_block) @indent
(transition_block) @indent
(nested_rule) @indent
(match_statement) @indent
(type_declaration) @indent
(enum_declaration) @indent

(_ "{" "}" @end) @indent
(_ "(" ")" @end) @indent
(_ "[" "]" @end) @indent
