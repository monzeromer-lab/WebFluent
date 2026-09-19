; Every brace, paren and bracket pair indents its contents by one level; the
; closing delimiter returns to the level of the line that opened it.

(_ "{" "}" @end) @indent
(_ "(" ")" @end) @indent
(_ "[" "]" @end) @indent
