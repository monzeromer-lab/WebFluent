/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

// Tree-sitter grammar for WebFluent.
//
// This mirrors the hand-written parser in `src/parser/parser.rs`; when the
// language grows, that file is the reference and this one follows it. A few
// things the compiler decides with look-ahead or a vocabulary are decided here
// by token shape or left to the editor queries:
//
// - A capitalised word is a `component_identifier`, a lower-case one an
//   `identifier`. The compiler treats `Foo(…)` in statement position as an
//   element and `foo(…)` as a call by looking at the first letter; splitting
//   the token does the same without ambiguity.
// - Modifiers (`primary`, `bold`, `h1`, …) are ordinary identifiers in
//   argument position. The compiler consults a vocabulary and the names in
//   scope; the highlight query consults the same vocabulary with `#any-of?`.
// - Keywords are extracted from the `word` token, so `success`, `error`,
//   `action`, `token`, `from` and the rest are keywords only where the
//   parser expects them and plain names everywhere else — a prop called
//   `error`, a map key called `action`, `Array.from`.

const PREC = {
  lambda: -1,
  or: 1,
  and: 2,
  equality: 3,
  comparison: 4,
  additive: 5,
  multiplicative: 6,
  unary: 7,
  postfix: 8,
  element: 2,
};

// Every built-in the compiler lexes as a component keyword
// (`lexer::token::component_name`). `List` doubles as a type, and `Map` is a
// type only, so those live in `builtin_type` as well / instead.
const BUILTIN_COMPONENTS = [
  // Layout
  "Container", "Row", "Column", "Grid", "Stack", "Spacer", "Divider",
  // Navigation
  "Navbar", "Sidebar", "Breadcrumb", "Link", "Menu", "Tabs", "TabPage",
  "Router", "Route",
  // Data display
  "Card", "Table", "Thead", "Tbody", "Trow", "Tcell", "List", "Badge",
  "Avatar", "Tooltip", "Tag",
  // Data input
  "Input", "Select", "Option", "Checkbox", "Radio", "Switch", "Slider",
  "DatePicker", "FileUpload", "Form",
  // Feedback
  "Alert", "Toast", "Modal", "Dialog", "Spinner", "Progress", "Skeleton",
  // Actions
  "Button", "IconButton", "ButtonGroup", "Dropdown",
  // Media
  "Image", "Video", "Icon", "Carousel",
  // Typography
  "Text", "Heading", "Code", "Blockquote",
  // Document (PDF)
  "Document", "Section", "Paragraph", "PageBreak", "Header", "Footer",
  // Slides
  "Presentation", "Slide", "TitleSlide", "SectionSlide", "TwoColumn",
  "ImageSlide",
];

/**
 * @param {RuleOrLiteral} rule
 * @param {RuleOrLiteral} separator
 */
function sepBy1(rule, separator) {
  return seq(rule, repeat(seq(separator, rule)));
}

/**
 * @param {RuleOrLiteral} rule
 * @param {RuleOrLiteral} separator
 */
function sepBy(rule, separator) {
  return optional(sepBy1(rule, separator));
}

module.exports = grammar({
  name: "webfluent",

  word: ($) => $.identifier,

  extras: ($) => [/\s/, $.comment],

  conflicts: ($) => [
    // `Button("x", aria-pressed: on)` versus `Text(a - b)`: after the first
    // word both an attribute name and a subtraction are open until the `:`.
    [$.attribute_name, $._primary_expression],
    // `(a, b) => a - b` versus `(a)`: a parenthesised lambda parameter list
    // and a parenthesised expression look the same until the `=>`.
    [$.lambda_parameters, $._primary_expression],
  ],

  // Inlined so that `element`'s precedence applies to the name itself: a
  // capitalised word at the start of a statement is an element, not an
  // expression, and the choice is made on that token.
  inline: ($) => [$._component_name, $._name, $._declaration_name],

  supertypes: ($) => [$._declaration, $._statement, $._expression],

  rules: {
    source_file: ($) => repeat($._declaration),

    // ─── Declarations ───────────────────────────────────────────────────

    _declaration: ($) =>
      choice(
        $.page_declaration,
        $.component_declaration,
        $.store_declaration,
        $.theme_declaration,
        $.app_declaration,
      ),

    page_declaration: ($) =>
      seq(
        "Page",
        field("name", $._declaration_name),
        field("attributes", $.argument_list),
        field("body", $.block),
      ),

    component_declaration: ($) =>
      seq(
        "Component",
        field("name", $._declaration_name),
        field("parameters", $.parameter_list),
        field("body", $.block),
      ),

    store_declaration: ($) =>
      seq("Store", field("name", $._name), field("body", $.block)),

    theme_declaration: ($) =>
      seq(
        "Theme",
        field("name", $._name),
        field("body", $.theme_body),
      ),

    theme_body: ($) => seq("{", repeat($.token_declaration), "}"),

    token_declaration: ($) =>
      seq(
        "token",
        field("name", $.property_name),
        ":",
        field("value", $._expression),
      ),

    app_declaration: ($) => seq("App", field("body", $.block)),

    // A page or component may be named after a built-in (`Page Menu`).
    _declaration_name: ($) => choice($._name, $.builtin_component),

    _name: ($) => choice($.identifier, $.component_identifier),

    // ─── Parameters ─────────────────────────────────────────────────────

    parameter_list: ($) => seq("(", sepBy($.parameter, ","), ")"),

    parameter: ($) =>
      seq(
        field("name", $._name),
        optional(field("optional", "?")),
        ":",
        field("type", $.builtin_type),
        optional(seq("=", field("default", $._expression))),
      ),

    builtin_type: (_) => choice("String", "Number", "Bool", "List", "Map"),

    // ─── Statements ─────────────────────────────────────────────────────

    block: ($) => seq("{", repeat($._statement), "}"),

    _statement: ($) =>
      choice(
        $.state_declaration,
        $.derived_declaration,
        $.effect_declaration,
        $.action_declaration,
        $.use_declaration,
        $.if_statement,
        $.for_statement,
        $.show_statement,
        $.fetch_statement,
        $.navigate_statement,
        $.log_statement,
        $.return_statement,
        $.animate_statement,
        $.children,
        $.event_handler,
        $.style_block,
        $.transition_block,
        $.element,
        $.assignment,
        $.expression_statement,
      ),

    state_declaration: ($) =>
      seq("state", field("name", $._name), "=", field("value", $._expression)),

    derived_declaration: ($) =>
      seq("derived", field("name", $._name), "=", field("value", $._expression)),

    effect_declaration: ($) => seq("effect", field("body", $.block)),

    action_declaration: ($) =>
      seq(
        "action",
        field("name", $._name),
        field("parameters", $.parameter_list),
        field("body", $.block),
      ),

    use_declaration: ($) => seq("use", field("store", $._name)),

    navigate_statement: ($) => seq("navigate", "(", $._expression, ")"),

    log_statement: ($) => seq("log", "(", $._expression, ")"),

    // `return` alone at the end of a block, or `return expr`.
    return_statement: ($) => prec.right(seq("return", optional($._expression))),

    // `animate(target, fadeIn, "300ms")` — replay an animation on an element.
    animate_statement: ($) =>
      seq(
        "animate",
        "(",
        field("target", $._expression),
        ",",
        field("animation", $.identifier),
        optional(seq(",", field("duration", $.string))),
        ")",
      ),

    children: (_) => "children",

    // ─── Control flow ───────────────────────────────────────────────────

    if_statement: ($) =>
      prec.right(
        seq(
          "if",
          field("condition", $._expression),
          optional(seq(",", $.animate_clause)),
          field("consequence", $.block),
          optional(field("alternative", $.else_clause)),
        ),
      ),

    else_clause: ($) => seq("else", choice($.if_statement, $.block)),

    for_statement: ($) =>
      seq(
        "for",
        field("item", $._name),
        optional(seq(",", field("index", $._name))),
        "in",
        field("iterable", $._expression),
        optional(seq(",", $.animate_clause)),
        field("body", $.block),
      ),

    show_statement: ($) =>
      seq(
        "show",
        field("condition", $._expression),
        optional(seq(",", $.animate_clause)),
        field("body", $.block),
      ),

    // `animate(enter, exit, duration: "300ms", stagger: "50ms")`
    animate_clause: ($) => seq("animate", $.argument_list),

    // ─── Fetch ──────────────────────────────────────────────────────────

    fetch_statement: ($) =>
      seq(
        "fetch",
        field("name", $._name),
        "from",
        field("url", $._expression),
        optional(field("options", $.fetch_options)),
        field("body", $.fetch_body),
      ),

    // Options open with a named argument, which is how the compiler tells
    // `from url (method: "POST")` apart from a call `from build(1)`.
    fetch_options: ($) =>
      seq("(", $.named_argument, repeat(seq(",", $._argument)), ")"),

    fetch_body: ($) =>
      seq(
        "{",
        repeat(choice($.loading_block, $.error_block, $.success_block)),
        "}",
      ),

    loading_block: ($) => seq("loading", field("body", $.block)),

    error_block: ($) =>
      seq(
        "error",
        optional(seq("(", field("name", $._name), ")")),
        field("body", $.block),
      ),

    success_block: ($) => seq("success", field("body", $.block)),

    // ─── Elements ───────────────────────────────────────────────────────

    // `Card(elevated) { … }`, `UserCard(name: "x")`, `Footer`, `Card.Header { }`.
    // The precedence makes `Text("a")` an element with arguments rather than
    // a bare element followed by a parenthesised expression, and a bare
    // capitalised word an element rather than an expression statement.
    element: ($) =>
      prec.right(
        PREC.element,
        seq(
          field("name", $._component_name),
          optional(field("arguments", $.argument_list)),
          optional(field("body", $.block)),
        ),
      ),

    _component_name: ($) =>
      choice($.sub_component, $.builtin_component, $.component_identifier),

    // Only built-ins have sub-components: `Card.Header`, `Sidebar.Item`.
    // `CartStore.clear()` is a member expression.
    sub_component: ($) =>
      prec(
        PREC.element,
        seq(
        field("parent", $.builtin_component),
        ".",
        field("child", choice($.builtin_component, $.component_identifier, $.identifier)),
        ),
      ),

    // `List` is also a type; in element and expression position it is the
    // component, and a type is only ever read where one is expected.
    builtin_component: (_) => prec(1, choice(...BUILTIN_COMPONENTS)),

    // ─── Arguments ──────────────────────────────────────────────────────

    // Element arguments: positional values, modifiers (bare identifiers) and
    // `name: value` pairs, in any order. The precedence settles `Foo(x)` at
    // the start of a statement as an element rather than a call.
    argument_list: ($) => prec(1, seq("(", sepBy($._argument, ","), ")")),

    _argument: ($) => choice($.named_argument, $._expression),

    named_argument: ($) =>
      seq(field("name", $.attribute_name), ":", field("value", $._expression)),

    // `bind`, `aria-pressed`, `data-tone`: hyphenated because an HTML
    // attribute may be.
    attribute_name: ($) => seq($.identifier, repeat(seq("-", $.identifier))),

    // Call arguments. The compiler takes positional values only; `name: value`
    // is accepted here so `t("greeting", name: user.name)` and
    // `fetch x from url (method: "POST")` still read as one call each.
    arguments: ($) => seq("(", sepBy($._argument, ","), ")"),

    // ─── Events ─────────────────────────────────────────────────────────

    event_handler: ($) => seq(field("event", $.event_name), field("body", $.block)),

    event_name: (_) => token(seq("on:", /[a-zA-Z_][a-zA-Z0-9_]*/)),

    // ─── Style ──────────────────────────────────────────────────────────

    style_block: ($) =>
      seq(
        "style",
        "{",
        repeat(choice($.style_property, $.pseudo_block, $.at_rule)),
        "}",
      ),

    style_property: ($) =>
      seq(
        field("name", choice($.property_name, $.custom_property_name)),
        ":",
        field("value", $._expression),
      ),

    // `hover { … }`, `focus-within { … }`, `placeholder { … }`.
    pseudo_block: ($) =>
      seq(
        field("state", $.pseudo_state),
        "{",
        repeat($.style_property),
        "}",
      ),

    pseudo_state: ($) => seq($.identifier, optional(seq("-", $.identifier))),

    // `@media (max-width: 768px) { … }` — the condition runs to the brace.
    at_rule: ($) =>
      seq(
        field("name", $.at_keyword),
        field("condition", $.at_rule_condition),
        "{",
        repeat($.style_property),
        "}",
      ),

    at_keyword: (_) => /@[a-zA-Z_][a-zA-Z0-9_-]*/,

    at_rule_condition: (_) => /[^{}\s][^{}\n]*/,

    // `border-radius`, `color-text-muted`, `viz-1`, `radius-2xl`.
    property_name: ($) =>
      seq(
        $.identifier,
        repeat(seq("-", choice($.identifier, seq($.number, optional($.identifier))))),
      ),

    // `--hover-bg: hoverColor` — a custom property a static rule can read.
    custom_property_name: (_) => token(seq("--", /[a-zA-Z_][a-zA-Z0-9_-]*/)),

    // ─── Transition ─────────────────────────────────────────────────────

    transition_block: ($) =>
      seq("transition", "{", repeat($.transition_property), "}"),

    // `background 200ms ease`, `transform fast spring`, `opacity "150ms"`.
    transition_property: ($) =>
      seq(
        field("property", $.property_name),
        field("duration", choice($.duration, $.string, $.number, $.duration_keyword)),
        optional(field("easing", $.easing)),
      ),

    duration: (_) => /\d+(\.\d+)?(ms|s)/,

    duration_keyword: (_) => choice("fast", "normal", "slow"),

    easing: (_) =>
      choice("ease", "linear", "easeIn", "easeOut", "easeInOut", "spring", "bouncy", "smooth"),

    // ─── Assignment & expression statements ─────────────────────────────

    assignment: ($) =>
      seq(
        field("target", choice($._name, $.member_expression, $.index_expression)),
        "=",
        field("value", $._expression),
      ),

    // An `if` at the start of a statement is always the statement, never
    // the value form; the value form still nests inside larger expressions.
    // The low precedence keeps `foo(x)` one call rather than a name followed
    // by a parenthesised statement.
    expression_statement: ($) =>
      prec(
        -1,
        choice($.binary_expression, $.unary_expression, $.lambda, $._primary_expression),
      ),

    // ─── Expressions ────────────────────────────────────────────────────

    _expression: ($) =>
      choice(
        $.binary_expression,
        $.unary_expression,
        $.lambda,
        $.if_expression,
        $._primary_expression,
      ),

    _primary_expression: ($) =>
      choice(
        $.member_expression,
        $.index_expression,
        $.call_expression,
        $.parenthesized_expression,
        $.string,
        $.number,
        $.boolean,
        $.null,
        $.array,
        $.object,
        $.identifier,
        $.component_identifier,
        $.builtin_type,
        $.builtin_component,
      ),

    binary_expression: ($) => {
      const table = [
        [PREC.or, "||"],
        [PREC.and, "&&"],
        [PREC.equality, choice("==", "!=", "!==")],
        [PREC.comparison, choice("<", ">", "<=", ">=")],
        [PREC.additive, choice("+", "-")],
        [PREC.multiplicative, choice("*", "/", "%")],
      ];
      return choice(
        ...table.map(([precedence, operator]) =>
          prec.left(
            // @ts-ignore
            precedence,
            seq(
              field("left", $._expression),
              // @ts-ignore
              field("operator", operator),
              field("right", $._expression),
            ),
          ),
        ),
      );
    },

    unary_expression: ($) =>
      prec(PREC.unary, seq(field("operator", choice("!", "-")), field("operand", $._expression))),

    member_expression: ($) =>
      prec.left(
        PREC.postfix,
        seq(
          field("object", $._expression),
          ".",
          // Any word may follow a dot — `item.action`, `Array.from`.
          field("property", $._name),
        ),
      ),

    index_expression: ($) =>
      prec.left(
        PREC.postfix,
        seq(field("object", $._expression), "[", field("index", $._expression), "]"),
      ),

    // Only a name, a member, an index or another call is callable. A string
    // never is, which is what lets `fetch x from base + "/api" (method: …)`
    // read the parenthesised group as fetch options.
    call_expression: ($) =>
      prec.left(
        PREC.postfix,
        seq(
          field(
            "function",
            choice(
              $.identifier,
              $.component_identifier,
              $.builtin_type,
              $.member_expression,
              $.index_expression,
              $.call_expression,
            ),
          ),
          field("arguments", $.arguments),
        ),
      ),

    parenthesized_expression: ($) => seq("(", $._expression, ")"),

    // `x => expr` and `(a, b) => expr`.
    lambda: ($) =>
      prec.right(
        PREC.lambda,
        seq(
          field("parameters", choice($.identifier, $.lambda_parameters)),
          "=>",
          field("body", $._expression),
        ),
      ),

    lambda_parameters: ($) => seq("(", sepBy($.identifier, ","), ")"),

    // `if cond { a } else { b }` as a value, chained with `else if`.
    if_expression: ($) =>
      seq(
        "if",
        field("condition", $._expression),
        "{",
        field("consequence", $._expression),
        "}",
        "else",
        choice($.if_expression, seq("{", field("alternative", $._expression), "}")),
      ),

    // ─── Literals ───────────────────────────────────────────────────────

    string: ($) =>
      seq(
        '"',
        repeat(choice($.string_content, $.escape_sequence, $.interpolation)),
        '"',
      ),

    // A `{` opens an interpolation only when a name follows it; `"{ a: 1 }"`
    // and `"{"` are text, as they are for the compiler.
    string_content: (_) =>
      choice(
        token.immediate(prec(1, /[^"\\{]+/)),
        token.immediate(/\{[^a-zA-Z_"\\{]/),
      ),

    escape_sequence: (_) => token.immediate(/\\./),

    interpolation: ($) => seq("{", $._expression, "}"),

    number: (_) => /\d+(\.\d+)?/,

    boolean: (_) => choice("true", "false"),

    null: (_) => "null",

    array: ($) => seq("[", sepBy($._expression, ","), optional(","), "]"),

    object: ($) => seq("{", sepBy($.pair, ","), optional(","), "}"),

    pair: ($) =>
      seq(
        field("key", choice($._name, $.string)),
        ":",
        field("value", $._expression),
      ),

    // ─── Names ──────────────────────────────────────────────────────────

    identifier: (_) => /[a-z_][a-zA-Z0-9_]*/,

    component_identifier: (_) => /[A-Z][a-zA-Z0-9_]*/,

    // ─── Comments ───────────────────────────────────────────────────────

    comment: (_) =>
      token(
        choice(
          seq("//", /.*/),
          seq("/*", /[^*]*\*+([^/*][^*]*\*+)*/, "/"),
        ),
      ),
  },
});
