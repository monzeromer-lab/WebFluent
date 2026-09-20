; Scopes the language config can override settings in (see config.toml), and
; that bracket pairs can be disabled in with `not_in`.

(string) @string

(comment) @comment.inclusive
(doc_comment) @comment.inclusive

(style_block) @style
(nested_rule) @style
(transition_block) @style
(theme_body) @style

(argument_list) @arguments
(page_attributes) @arguments
("(" @open ")" @close)
("[" @open "]" @close)
("{" @open "}" @close)

(("\"" @open "\"" @close) (#set! rainbow.exclude))
