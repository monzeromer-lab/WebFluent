; The outline panel and breadcrumbs: declarations, and the state, derived
; values, resources, actions, events and slots inside them.

(page_declaration
  "page" @context
  name: (_) @name) @item

(component_declaration
  "component" @context
  name: (_) @name) @item

(store_declaration
  "store" @context
  name: (_) @name) @item

(theme_declaration
  "theme" @context
  name: (_) @name) @item

(type_declaration
  "type" @context
  name: (_) @name) @item

(enum_declaration
  "enum" @context
  name: (_) @name) @item

(app_declaration
  "app" @context) @item

(state_declaration
  "state" @context
  name: (_) @name) @item

(derived_declaration
  "derived" @context
  name: (_) @name) @item

(resource_declaration
  "resource" @context
  name: (_) @name) @item

(action_declaration
  "action" @context
  name: (_) @name
  parameters: (parameter_list) @context.extra) @item

(event_declaration
  "event" @context
  name: (_) @name) @item

(slot_declaration
  "slot" @context
  name: (identifier) @name) @item

(effect_declaration
  "effect" @context) @item

(token_declaration
  name: (property_name) @name) @item
