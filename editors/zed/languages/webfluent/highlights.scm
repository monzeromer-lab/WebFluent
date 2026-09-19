; Syntax highlighting for WebFluent.
;
; Later patterns win over earlier ones, so the generic captures come first and
; the specific ones after. The modifier vocabulary below is generated from
; `src/parser/vocabulary.rs` (`just zed-check-vocabulary` verifies it); the
; pseudo-state list from `Parser::PSEUDO_STATES`.

; ─── Generic ────────────────────────────────────────────────────────────

(identifier) @variable
(component_identifier) @type
(builtin_type) @type.builtin
(builtin_component) @type.builtin

(comment) @comment

; ─── Declarations ───────────────────────────────────────────────────────

[
  "Page"
  "Component"
  "Store"
  "App"
  "Theme"
] @keyword

(page_declaration name: (_) @type.definition)
(component_declaration name: (_) @type.definition)
(store_declaration name: (_) @type.definition)
(theme_declaration name: (_) @type.definition)

(parameter name: (_) @variable.parameter)

(token_declaration name: (property_name) @property)

; ─── State, actions and stores ──────────────────────────────────────────

[
  "state"
  "derived"
  "effect"
  "action"
  "use"
  "token"
  "return"
] @keyword

(children) @keyword

(state_declaration name: (_) @variable)
(derived_declaration name: (_) @variable)
(action_declaration name: (_) @function.definition)
(use_declaration store: (_) @type)

; ─── Control flow ───────────────────────────────────────────────────────

[
  "if"
  "else"
  "for"
  "in"
  "show"
  "fetch"
  "from"
  "loading"
  "error"
  "success"
] @keyword

(for_statement item: (_) @variable)
(for_statement index: (_) @variable)
(fetch_statement name: (_) @variable)
(error_block name: (_) @variable)

; Built-in statements that read like calls.
[
  "navigate"
  "log"
  "animate"
] @function.special

; ─── Elements ───────────────────────────────────────────────────────────

(element name: (component_identifier) @type)

(sub_component
  parent: (builtin_component) @type.builtin
  child: (_) @type.builtin)

; A bare word among an element's arguments is a modifier when it is one the
; compiler knows: `Button("Save", primary, large)`, `Heading("T", h1)`.
((argument_list (identifier) @attribute)
  (#any-of? @attribute
    "small" "medium" "large" "primary" "secondary" "success" "danger"
    "warning" "info" "error" "loading" "rounded" "pill" "square" "flat"
    "elevated" "outlined" "full" "fit" "fluid" "circle" "xs" "sm" "md" "lg"
    "xl" "ordered" "multiple" "header" "bold" "italic" "underline"
    "uppercase" "lowercase" "left" "center" "right" "heading" "subtitle"
    "muted" "h1" "h2" "h3" "h4" "h5" "h6" "dismissible" "block" "bordered"
    "controls" "autoplay" "text" "email" "password" "number" "search" "tel"
    "url" "date" "time" "datetime" "color" "submit" "reset" "fadeIn"
    "fadeOut" "slideUp" "slideDown" "slideLeft" "slideRight" "scaleIn"
    "scaleOut" "bounce" "shake" "pulse" "spin" "fast" "slow"))

(named_argument name: (attribute_name) @property)
(attribute_name (identifier) @property)

(event_name) @keyword @label

; ─── Style and transitions ──────────────────────────────────────────────

[
  "style"
  "transition"
] @keyword

(style_property name: (property_name) @property)
(style_property name: (custom_property_name) @property)
(property_name (identifier) @property)

; `hover { }`, `focus-within { }` — the states the compiler compiles to a
; rule. Any other word before a brace stays a plain name.
((pseudo_state) @keyword
  (#any-of? @keyword
    "hover" "focus" "active" "disabled" "placeholder" "focus-within"
    "current" "pressed" "selected" "checked" "expanded" "invalid"))
((pseudo_state (identifier) @keyword) @_state
  (#any-of? @_state
    "hover" "focus" "active" "disabled" "placeholder" "focus-within"
    "current" "pressed" "selected" "checked" "expanded" "invalid"))

(at_keyword) @keyword
(at_rule_condition) @string.special

(transition_property property: (property_name) @property)
(transition_property duration: (_) @number)
(duration_keyword) @constant.builtin
(easing) @constant.builtin

; ─── Expressions ────────────────────────────────────────────────────────

(member_expression property: (identifier) @property)
(member_expression property: (component_identifier) @property)

(call_expression function: (identifier) @function)
(call_expression
  function: (member_expression property: (_) @function.method))
(call_expression function: (builtin_type) @type.builtin)

(lambda parameters: (identifier) @variable.parameter)
(lambda_parameters (identifier) @variable.parameter)

(pair key: (identifier) @property)
(pair key: (component_identifier) @property)

(animate_statement animation: (identifier) @attribute)
(animate_clause (argument_list (identifier) @attribute))

; ─── Literals ───────────────────────────────────────────────────────────

(string) @string
(escape_sequence) @string.escape
(number) @number
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
  "?"
] @punctuation.delimiter
