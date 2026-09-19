; Vim-mode text objects and motions.
;
; A "function" is anything with a body that runs: an action, an effect, an
; event handler, a lambda. A "class" is a top-level declaration.

(action_declaration
  body: (block
    "{"
    (_)* @function.inside
    "}")) @function.around

(effect_declaration
  body: (block
    "{"
    (_)* @function.inside
    "}")) @function.around

(event_handler
  body: (block
    "{"
    (_)* @function.inside
    "}")) @function.around

(lambda
  body: (_) @function.inside) @function.around

(page_declaration
  body: (block
    "{"
    (_)* @class.inside
    "}")) @class.around

(component_declaration
  body: (block
    "{"
    (_)* @class.inside
    "}")) @class.around

(store_declaration
  body: (block
    "{"
    (_)* @class.inside
    "}")) @class.around

(app_declaration
  body: (block
    "{"
    (_)* @class.inside
    "}")) @class.around

(theme_declaration
  body: (theme_body
    "{"
    (_)* @class.inside
    "}")) @class.around

(comment)+ @comment.around
