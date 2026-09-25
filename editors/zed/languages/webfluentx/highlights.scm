; Syntax highlighting for WebFluent.
;
; Later patterns win over earlier ones, so the generic captures come first and
; the specific ones after. Flags, cases, tokens and parts are nodes of the
; grammar, so no vocabulary is listed here; the compiler's registry is what
; decides whether a flag is real, and the language server reports it.

; ─── Generic ────────────────────────────────────────────────────────────

(identifier) @variable
(component_identifier) @type
(builtin_type) @type.builtin
(builtin_component) @type.builtin
(named_type (component_identifier) @type)

(comment) @comment
(doc_comment) @comment.doc

; ─── Declarations ───────────────────────────────────────────────────────

[
  "page"
  "component"
  "store"
  "app"
  "theme"
  "type"
  "enum"
  "const"
  "image"
  "api"
  "external"
  "element"
  "from"
  "fn"
  "prop"
  "integrity"
] @keyword

(page_declaration name: (_) @type.definition)
(component_declaration name: (_) @type.definition)
(store_declaration name: (_) @type.definition)
(theme_declaration name: (_) @type.definition)
(type_declaration name: (_) @type.definition)
(enum_declaration name: (_) @type.definition)
(api_declaration name: (_) @type.definition)
(animation_declaration name: (_) @type.definition)
(type_declaration base: (_) @type)
(const_declaration name: (_) @constant)
(data_declaration name: (_) @constant)

(parameter name: (_) @variable.parameter)
(parameter "_" @punctuation.special)
(field_declaration name: (_) @property)
(enum_case_declaration name: (identifier) @constant)
(route_parameter name: (identifier) @variable.parameter)

(token_declaration name: (property_name) @property)

; ─── State, actions and stores ──────────────────────────────────────────

[
  "state"
  "derived"
  "effect"
  "action"
  "use"
  "resource"
  "let"
  "return"
  "event"
  "slot"
  "part"
  "persist"
  "animation"
  "every"
  "after"
  "cleanup"
  "head"
  "test"
  "data"
  "expect"
  "emit"
  "await"
] @keyword

(children) @keyword

(state_declaration name: (_) @variable)
(derived_declaration name: (_) @variable)
(resource_declaration name: (_) @variable)
(let_declaration name: (_) @variable)
(action_declaration name: (_) @function.definition)
(external_declaration name: (_) @type.definition)
(external_fn name: (_) @function.definition)
(external_type name: (_) @type.definition)
(external_member name: (_) @function.definition)
(event_declaration name: (_) @function.definition)
(slot_declaration name: (identifier) @label)
(emit_statement event: (_) @function)
(use_declaration store: (_) @type)

; ─── Services, connections and validation ───────────────────────────────

[
  "headers"
  "errors"
  "at"
  "get"
  "post"
  "put"
  "patch"
  "delete"
  "options"
  "validate"
  "socket"
  "stream"
  "channel"
  "send"
  "receive"
  "try"
  "catch"
] @keyword

(endpoint name: (_) @function.definition)
(api_setting name: (identifier) @property)
(endpoint_setting name: (identifier) @property)
(header_name (_) @property)
(connection_declaration name: (_) @variable)
(validate_declaration name: (_) @variable)
(validation_rule rule: (identifier) @function.builtin)

; ─── Control flow ───────────────────────────────────────────────────────

[
  "if"
  "else"
  "for"
  "in"
  "by"
  "show"
  "sequence"
  "step"
  "migrate"
  "click"
  "press"
  "into"
  "match"
] @keyword

(if_let name: (_) @variable)
(for_statement item: (_) @variable)
(for_statement index: (_) @variable)
(error_pattern name: (_) @variable)
(ready_pattern name: (_) @variable)
(state_pattern name: (_) @variable)

[
  (loading_pattern)
  (else_pattern)
] @keyword
(error_pattern "error" @keyword)
[
  "connecting"
  "open"
  "closed"
] @keyword
(ready_pattern "ready" @keyword)

; Built-in statements that read like calls.
[
  "navigate"
  "log"
] @function.special

; ─── Elements ───────────────────────────────────────────────────────────

(element name: (component_identifier) @type)

(part
  owner: (builtin_component) @type.builtin
  name: (_) @type.builtin)

; `.primary`, `.lg` — a flag; `.info` — a case; `$surface` — a token.
(flag) @attribute
(enum_case) @constant
(case_value name: (_) @constant)
(case_pattern binding: (_) @variable.parameter)
(design_token) @variable.special

(named_argument name: (attribute_name) @property)
(attribute_name (identifier) @property)

(call_statement
  object: (component_identifier) @type
  method: (method) @function.method)
(call_statement property: (property) @property)
(store_member object: (component_identifier) @type)
(store_member property: (property) @property)

; ─── Events and slots ───────────────────────────────────────────────────

"on" @keyword
(event_handler event: (identifier) @label)
(event_handler parameter: (_) @variable.parameter)
(slot_fill name: (identifier) @label)

; ─── Style and transitions ──────────────────────────────────────────────

[
  "style"
  "transition"
] @keyword

(style_property name: (property_name) @property)
(style_property name: (custom_property_name) @property)
(property_name (identifier) @property)
(style_text) @string.special
(selector) @keyword

; ─── Expressions ────────────────────────────────────────────────────────

(member_expression property: (identifier) @property)
(member_expression property: (component_identifier) @property)

(call_expression function: (identifier) @function)
(call_expression
  function: (member_expression property: (_) @function.method))
(call_expression function: (builtin_type) @type.builtin)
(call_expression function: (component_identifier) @type)

(lambda parameters: (identifier) @variable.parameter)
(lambda_parameters (identifier) @variable.parameter)

(pair key: (identifier) @property)
(pair key: (component_identifier) @property)

; ─── Literals ───────────────────────────────────────────────────────────

(string) @string
(raw_string) @string
(block_string) @string
(escape_sequence) @string.escape
(format_spec) @string.special
(number) @number
(temporal) @number
(money) @number
(duration) @number
(color) @constant

; The types the language brings with it, which a program names like its own.
((named_type (component_identifier) @type.builtin)
  (#match? @type.builtin "^(Date|Time|DateTime|Duration|Money|Url|Email|Color|Uuid|File|Secret)$"))
(boolean) @boolean
(null) @constant.builtin

(interpolation
  "{" @punctuation.special
  "}" @punctuation.special)

; ─── Operators and punctuation ──────────────────────────────────────────

[
  "+"
  "-"
  "*"
  "/"
  "%"
  "=="
  "!="
  "!=="
  "<"
  ">"
  "<="
  ">="
  "&&"
  "||"
  "??"
  "!"
  "="
  "=>"
] @operator

[
  "("
  ")"
  "["
  "]"
  "{"
  "}"
] @punctuation.bracket

[
  ","
  "."
  ":"
  ";"
  "?"
] @punctuation.delimiter
(keyframe_selector) @constant
