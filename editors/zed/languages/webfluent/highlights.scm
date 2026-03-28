; WebFluent Tree-sitter highlight queries
; These queries follow standard Tree-sitter node naming conventions and will
; work once a Tree-sitter grammar for WebFluent is created.

; ---------------------------------------------------------------------------
; Declaration keywords
; ---------------------------------------------------------------------------

[
  "Page"
  "Component"
  "Store"
  "App"
] @keyword

; ---------------------------------------------------------------------------
; Control flow
; ---------------------------------------------------------------------------

[
  "if"
  "else"
  "for"
  "in"
  "show"
] @keyword.control

; ---------------------------------------------------------------------------
; State and reactivity keywords
; ---------------------------------------------------------------------------

[
  "state"
  "derived"
  "effect"
  "action"
] @keyword.other

; ---------------------------------------------------------------------------
; Built-in components
; ---------------------------------------------------------------------------

(component_name) @type.builtin

[
  "Container"
  "Row"
  "Column"
  "Grid"
  "Stack"
  "Spacer"
  "Divider"
  "Heading"
  "Text"
  "Button"
  "Link"
  "Image"
  "Icon"
  "IconButton"
  "Input"
  "Select"
  "Option"
  "Checkbox"
  "Radio"
  "Switch"
  "Slider"
  "DatePicker"
  "FileUpload"
  "Textarea"
  "Card"
  "Modal"
  "Alert"
  "Badge"
  "Tag"
  "Avatar"
  "Tooltip"
  "Progress"
  "Spinner"
  "Skeleton"
  "Table"
  "Thead"
  "Trow"
  "Tcell"
  "Tabs"
  "TabPage"
  "Navbar"
  "Sidebar"
  "Breadcrumb"
  "Dropdown"
  "Menu"
  "Router"
  "Route"
  "Code"
  "Blockquote"
] @type.builtin

; ---------------------------------------------------------------------------
; Sub-components (e.g. Card.Header, Sidebar.Item, Modal.Footer)
; ---------------------------------------------------------------------------

(sub_component) @type
(sub_component_name) @type

(member_expression
  object: (identifier) @type
  "." @punctuation.delimiter
  property: (identifier) @type)

; ---------------------------------------------------------------------------
; User-defined component references
; ---------------------------------------------------------------------------

(component_reference) @type

; ---------------------------------------------------------------------------
; Modifiers / style keywords (positional arguments)
; ---------------------------------------------------------------------------

(modifier) @attribute

[
  "primary"
  "secondary"
  "success"
  "danger"
  "warning"
  "info"
  "muted"
  "bold"
  "italic"
  "underline"
  "uppercase"
  "lowercase"
  "center"
  "left"
  "right"
  "small"
  "medium"
  "large"
  "xl"
  "sm"
  "md"
  "lg"
  "full"
  "rounded"
  "outlined"
  "elevated"
  "block"
  "inline"
  "heading"
  "h1"
  "h2"
  "h3"
  "h4"
  "h5"
  "h6"
  "circle"
  "multiple"
  "text"
  "fadeIn"
  "fadeOut"
  "slideUp"
  "slideDown"
  "slideLeft"
  "slideRight"
] @attribute

; ---------------------------------------------------------------------------
; Events (on:click, on:input, on:submit, etc.)
; ---------------------------------------------------------------------------

(event_handler) @attribute

(event_name
  "on" @attribute
  ":" @punctuation.delimiter
  (identifier) @attribute)

; Fallback pattern for on:event syntax
("on" ":" @punctuation.delimiter (identifier) @attribute) @attribute

; ---------------------------------------------------------------------------
; Named arguments (bind:, path:, title:, gap:, etc.)
; ---------------------------------------------------------------------------

(named_argument
  name: (identifier) @property
  ":" @punctuation.delimiter)

(keyword_argument
  key: (identifier) @property
  ":" @punctuation.delimiter)

; Standalone named-arg pattern
(argument_name) @property

[
  "bind"
  "path"
  "title"
  "to"
  "label"
  "icon"
  "value"
  "placeholder"
  "accept"
  "gap"
  "columns"
  "span"
  "align"
  "justify"
  "min"
  "max"
  "step"
  "visible"
  "trigger"
  "height"
  "width"
  "size"
  "initials"
  "guard"
  "redirect"
  "page"
  "name"
  "role"
  "active"
  "description"
] @property

; ---------------------------------------------------------------------------
; Strings
; ---------------------------------------------------------------------------

(string_literal) @string

(string
  "\"" @string) @string

; String interpolation: {variable} inside strings
(interpolation
  "{" @punctuation.special
  "}" @punctuation.special)

(string_interpolation
  "{" @punctuation.special
  (expression) @embedded
  "}" @punctuation.special)

(template_substitution
  "{" @punctuation.special
  "}" @punctuation.special)

; ---------------------------------------------------------------------------
; Numbers
; ---------------------------------------------------------------------------

(number_literal) @number
(number) @number
(integer) @number
(float) @number

; ---------------------------------------------------------------------------
; Booleans
; ---------------------------------------------------------------------------

[
  "true"
  "false"
] @constant.builtin

; ---------------------------------------------------------------------------
; Comments
; ---------------------------------------------------------------------------

(line_comment) @comment
(block_comment) @comment
(comment) @comment

; ---------------------------------------------------------------------------
; Types (in Component prop declarations)
; ---------------------------------------------------------------------------

(type_identifier) @type

[
  "String"
  "Number"
  "Bool"
  "List"
  "Map"
] @type

; ---------------------------------------------------------------------------
; Operators
; ---------------------------------------------------------------------------

(binary_expression
  operator: _ @operator)

[
  "="
  "=="
  "!="
  "<"
  ">"
  "<="
  ">="
  "+"
  "-"
  "*"
  "/"
  "%"
  "&&"
  "||"
  "!"
  "=>"
  "?"
] @operator

; ---------------------------------------------------------------------------
; Punctuation
; ---------------------------------------------------------------------------

"{" @punctuation.bracket
"}" @punctuation.bracket
"(" @punctuation.bracket
")" @punctuation.bracket
"[" @punctuation.bracket
"]" @punctuation.bracket

"," @punctuation.delimiter
"." @punctuation.delimiter
":" @punctuation.delimiter

; ---------------------------------------------------------------------------
; Identifiers
; ---------------------------------------------------------------------------

(identifier) @variable

; Function calls (built-in functions like navigate, log, setLocale, t, etc.)
(call_expression
  function: (identifier) @function)

(function_call
  name: (identifier) @function)

; Method calls (e.g. items.map, items.sum)
(method_call
  method: (identifier) @function.method)

(call_expression
  function: (member_expression
    property: (identifier) @function.method))

; Lambda / arrow function parameters
(arrow_function
  parameter: (identifier) @variable.parameter)

(parameter) @variable.parameter
(parameter_name) @variable.parameter
