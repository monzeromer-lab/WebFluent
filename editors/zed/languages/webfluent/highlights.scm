; ─── Comments ─────────────────────────────────────────
(comment) @comment

; ─── Declaration keywords ─────────────────────────────
(page_declaration "Page" @keyword)
(component_declaration "Component" @keyword)
(store_declaration "Store" @keyword)
(app_declaration "App" @keyword)

; ─── Declaration names ────────────────────────────────
(page_declaration (identifier) @type)
(component_declaration (identifier) @type)
(store_declaration (identifier) @type)

; ─── State & logic keywords ──────────────────────────
(state_declaration "state" @keyword)
(derived_declaration "derived" @keyword)
(effect_declaration "effect" @keyword)
(action_declaration "action" @keyword)
(use_declaration "use" @keyword)
(navigate_call "navigate" @keyword)

; ─── State & action names ─────────────────────────────
(state_declaration (identifier) @variable)
(derived_declaration (identifier) @variable)
(action_declaration (identifier) @function)
(use_declaration (identifier) @type)

; ─── Control flow ─────────────────────────────────────
(if_statement "if" @keyword)
(if_statement "else" @keyword)
(for_statement "for" @keyword)
(for_statement "in" @keyword)
(show_statement "show" @keyword)
(animate_clause "animate" @keyword)

; ─── Fetch ────────────────────────────────────────────
(fetch_statement "fetch" @keyword)
(fetch_statement "from" @keyword)
(loading_block "loading" @keyword)
(error_block "error" @keyword)
(success_block "success" @keyword)

; ─── Built-in components ──────────────────────────────
(builtin_component) @type.builtin

; ─── Sub-components ───────────────────────────────────
(sub_component
  parent: (identifier) @type.builtin
  child: (identifier) @property)

; ─── Events ───────────────────────────────────────────
(event_name) @attribute

; ─── Modifiers ────────────────────────────────────────
(modifier) @attribute

; ─── Type annotations ─────────────────────────────────
(type_annotation) @type.builtin

; ─── Named arguments ─────────────────────────────────
(named_argument (identifier) @property)

; ─── Style blocks ─────────────────────────────────────
(style_block "style" @keyword)
(style_property (property_name) @property)

; ─── Transition blocks ────────────────────────────────
(transition_block "transition" @keyword)

; ─── Strings ──────────────────────────────────────────
(string) @string
(escape_sequence) @string.escape
(interpolation "{" @punctuation.special)
(interpolation "}" @punctuation.special)

; ─── t() function ─────────────────────────────────────
(t_call "t" @function)

; ─── Literals ─────────────────────────────────────────
(number) @number
(boolean) @boolean
(null) @constant.builtin

; ─── Operators ────────────────────────────────────────
(binary_expression "+" @operator)
(binary_expression "-" @operator)
(binary_expression "*" @operator)
(binary_expression "/" @operator)
(binary_expression "==" @operator)
(binary_expression "!=" @operator)
(binary_expression "&&" @operator)
(binary_expression "||" @operator)
(unary_expression "!" @operator)
(assignment "=" @operator)

; ─── Punctuation ──────────────────────────────────────
"{" @punctuation.bracket
"}" @punctuation.bracket
"(" @punctuation.bracket
")" @punctuation.bracket
"[" @punctuation.bracket
"]" @punctuation.bracket
"," @punctuation.delimiter
"." @punctuation.delimiter
