; The outline panel and breadcrumbs: declarations, and the state, derived
; values and actions inside them.

(page_declaration
  "Page" @context
  name: (_) @name) @item

(component_declaration
  "Component" @context
  name: (_) @name) @item

(store_declaration
  "Store" @context
  name: (_) @name) @item

(theme_declaration
  "Theme" @context
  name: (_) @name) @item

(app_declaration
  "App" @context) @item

(state_declaration
  "state" @context
  name: (_) @name) @item

(derived_declaration
  "derived" @context
  name: (_) @name) @item

(action_declaration
  "action" @context
  name: (_) @name
  parameters: (parameter_list) @context.extra) @item

(effect_declaration
  "effect" @context) @item

(fetch_statement
  "fetch" @context
  name: (_) @name) @item

(token_declaration
  "token" @context
  name: (property_name) @name) @item
