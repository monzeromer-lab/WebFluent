/// <reference types="tree-sitter-cli/dsl" />

module.exports = grammar({
  name: "webfluent",

  extras: ($) => [/\s/, $.comment],

  conflicts: ($) => [
    [$.component_name, $._expression],
    [$.sub_component, $._expression],
  ],

  rules: {
    source_file: ($) => repeat($._declaration),

    _declaration: ($) =>
      choice(
        $.page_declaration,
        $.component_declaration,
        $.store_declaration,
        $.app_declaration
      ),

    // ─── Top-level declarations ──────────────────────────

    page_declaration: ($) =>
      seq("Page", $.identifier, "(", optional($.argument_list), ")", $.block),

    component_declaration: ($) =>
      seq(
        "Component",
        $.identifier,
        "(",
        optional($.parameter_list),
        ")",
        $.block
      ),

    store_declaration: ($) => seq("Store", $.identifier, $.block),

    app_declaration: ($) => seq("App", $.block),

    // ─── Parameters & Arguments ──────────────────────────

    parameter_list: ($) => seq($.parameter, repeat(seq(",", $.parameter))),

    parameter: ($) =>
      seq(
        $.identifier,
        optional("?"),
        optional(seq(":", $.type_annotation)),
        optional(seq("=", $._expression))
      ),

    argument_list: ($) => seq($._argument, repeat(seq(",", $._argument))),

    _argument: ($) => choice($.named_argument, $._expression, $.modifier),

    named_argument: ($) => seq($.identifier, ":", $._expression),

    modifier: ($) =>
      token(
        choice(
          "primary",
          "secondary",
          "success",
          "danger",
          "warning",
          "info",
          "small",
          "medium",
          "large",
          "bold",
          "italic",
          "underline",
          "uppercase",
          "lowercase",
          "center",
          "right",
          "muted",
          "heading",
          "elevated",
          "outlined",
          "flat",
          "full",
          "fit",
          "rounded",
          "pill",
          "square",
          "block",
          "ordered",
          "dismissible",
          "fadeIn",
          "fadeOut",
          "slideUp",
          "slideDown",
          "slideLeft",
          "slideRight",
          "scaleIn",
          "scaleOut",
          "bounce",
          "shake",
          "pulse",
          "spin",
          "fast",
          "slow",
          "h1",
          "h2",
          "h3",
          "h4",
          "h5",
          "h6",
          "sm",
          "md",
          "lg",
          "xl"
        )
      ),

    type_annotation: ($) =>
      token(choice("String", "Number", "Bool", "List", "Map")),

    // ─── Statements ──────────────────────────────────────

    block: ($) => seq("{", repeat($._statement), "}"),

    _statement: ($) =>
      choice(
        $.state_declaration,
        $.derived_declaration,
        $.effect_declaration,
        $.action_declaration,
        $.use_declaration,
        $.ui_element,
        $.if_statement,
        $.for_statement,
        $.show_statement,
        $.fetch_statement,
        $.assignment,
        $.event_handler,
        $.style_block,
        $.transition_block,
        $.navigate_call,
        $.expression_statement
      ),

    state_declaration: ($) => seq("state", $.identifier, "=", $._expression),

    derived_declaration: ($) =>
      seq("derived", $.identifier, "=", $._expression),

    effect_declaration: ($) => seq("effect", $.block),

    action_declaration: ($) =>
      seq("action", $.identifier, "(", optional($.parameter_list), ")", $.block),

    use_declaration: ($) => seq("use", $.identifier),

    navigate_call: ($) => seq("navigate", "(", $._expression, ")"),

    assignment: ($) => seq($.identifier, "=", $._expression),

    expression_statement: ($) => $._expression,

    // ─── UI Elements ─────────────────────────────────────

    ui_element: ($) =>
      prec.left(
        seq(
          $.component_name,
          optional(seq("(", optional($.argument_list), ")")),
          optional($.block)
        )
      ),

    component_name: ($) =>
      choice($.sub_component, $.builtin_component, $.identifier),

    sub_component: ($) =>
      seq(
        field("parent", $.identifier),
        ".",
        field("child", $.identifier)
      ),

    builtin_component: ($) =>
      token(
        choice(
          "Container",
          "Row",
          "Column",
          "Grid",
          "Stack",
          "Spacer",
          "Divider",
          "Navbar",
          "Sidebar",
          "Link",
          "Tabs",
          "TabPage",
          "Breadcrumb",
          "Menu",
          "Card",
          "Table",
          "Thead",
          "Tbody",
          "Trow",
          "Tcell",
          "List",
          "Badge",
          "Avatar",
          "Tooltip",
          "Tag",
          "Input",
          "Select",
          "Option",
          "Checkbox",
          "Radio",
          "Switch",
          "Slider",
          "DatePicker",
          "FileUpload",
          "Form",
          "Alert",
          "Toast",
          "Modal",
          "Dialog",
          "Spinner",
          "Progress",
          "Skeleton",
          "Button",
          "IconButton",
          "ButtonGroup",
          "Dropdown",
          "Image",
          "Video",
          "Icon",
          "Carousel",
          "Text",
          "Heading",
          "Code",
          "Blockquote",
          "Document",
          "Section",
          "Paragraph",
          "Header",
          "Footer",
          "PageBreak",
          "Router",
          "Route"
        )
      ),

    // ─── Control flow ────────────────────────────────────

    if_statement: ($) =>
      seq(
        "if",
        $._expression,
        optional(seq(",", $.animate_clause)),
        $.block,
        optional(seq("else", choice($.if_statement, $.block)))
      ),

    for_statement: ($) =>
      seq(
        "for",
        $.identifier,
        optional(seq(",", $.identifier)),
        "in",
        $._expression,
        optional(seq(",", $.animate_clause)),
        $.block
      ),

    show_statement: ($) =>
      seq(
        "show",
        $._expression,
        optional(seq(",", $.animate_clause)),
        $.block
      ),

    animate_clause: ($) =>
      seq("animate", "(", optional($.argument_list), ")"),

    // ─── Fetch ───────────────────────────────────────────

    fetch_statement: ($) =>
      seq(
        "fetch",
        $.identifier,
        "from",
        $._expression,
        optional(seq("(", optional($.argument_list), ")")),
        $.fetch_block
      ),

    fetch_block: ($) =>
      seq(
        "{",
        repeat(
          choice($.loading_block, $.error_block, $.success_block, $._statement)
        ),
        "}"
      ),

    loading_block: ($) => seq("loading", $.block),
    error_block: ($) => seq("error", optional(seq("(", $.identifier, ")")), $.block),
    success_block: ($) => seq("success", $.block),

    // ─── Event handlers ──────────────────────────────────

    event_handler: ($) => seq($.event_name, $.block),

    event_name: ($) =>
      token(
        seq(
          "on:",
          choice(
            "click",
            "input",
            "change",
            "submit",
            "focus",
            "blur",
            "keydown",
            "keyup",
            "keypress",
            "mouseenter",
            "mouseleave"
          )
        )
      ),

    // ─── Style & Transition ──────────────────────────────

    style_block: ($) =>
      seq("style", "{", repeat(choice($.style_property, $.media_query)), "}"),

    style_property: ($) => seq($.property_name, ":", $._expression),

    property_name: ($) =>
      seq($.identifier, repeat(seq("-", $.identifier))),

    media_query: ($) =>
      seq(
        token("@media"),
        /[^{]+/,
        "{",
        repeat($.style_property),
        "}"
      ),

    transition_block: ($) =>
      seq("transition", "{", repeat($.transition_property), "}"),

    transition_property: ($) =>
      prec.left(seq($.identifier, $._expression, optional($.identifier))),

    // ─── Expressions ─────────────────────────────────────

    _expression: ($) =>
      choice(
        $.binary_expression,
        $.unary_expression,
        $.member_expression,
        $.call_expression,
        $.string,
        $.number,
        $.boolean,
        $.null,
        $.array,
        $.object,
        $.identifier,
        $.t_call,
        seq("(", $._expression, ")")
      ),

    binary_expression: ($) =>
      prec.left(
        1,
        seq(
          $._expression,
          choice(
            "+", "-", "*", "/", "%",
            "==", "!=", "<", ">", "<=", ">=",
            "&&", "||"
          ),
          $._expression
        )
      ),

    unary_expression: ($) => prec(2, seq("!", $._expression)),

    member_expression: ($) =>
      prec.left(3, seq($._expression, ".", $.identifier)),

    call_expression: ($) =>
      prec.left(
        3,
        seq($._expression, "(", optional($.argument_list), ")")
      ),

    t_call: ($) => seq("t", "(", $.string, optional(seq(",", $.argument_list)), ")"),

    // ─── Literals ────────────────────────────────────────

    string: ($) =>
      seq(
        '"',
        repeat(choice($.interpolation, $.escape_sequence, $.string_content)),
        '"'
      ),

    string_content: ($) => /[^"\\{]+/,

    interpolation: ($) => seq("{", $._expression, "}"),

    escape_sequence: ($) => /\\./,

    number: ($) => /\d+(\.\d+)?/,

    boolean: ($) => choice("true", "false"),

    null: ($) => "null",

    array: ($) => seq("[", optional(seq($._expression, repeat(seq(",", $._expression)))), "]"),

    object: ($) =>
      seq(
        "{",
        optional(
          seq($.object_entry, repeat(seq(",", $.object_entry)))
        ),
        "}"
      ),

    object_entry: ($) => seq($.identifier, ":", $._expression),

    identifier: ($) => /[a-zA-Z_][a-zA-Z0-9_]*/,

    // ─── Comments ────────────────────────────────────────

    comment: ($) =>
      choice(
        seq("//", /.*/),
        seq("/*", /[^*]*\*+([^/*][^*]*\*+)*/, "/")
      ),
  },
});
