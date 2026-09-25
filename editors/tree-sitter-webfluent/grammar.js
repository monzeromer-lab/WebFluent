/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

// Tree-sitter grammar for WebFluent 3.
//
// This mirrors the hand-written parser in `src/parser/v2.rs`; when the
// language grows, that file is the reference and this one follows it. A few
// things the compiler decides with look-ahead are decided here by token
// shape or left to the editor queries:
//
// - A capitalised word is a `component_identifier`, a lower-case one an
//   `identifier`. The compiler treats `Foo(…)` in statement position as an
//   element and `foo(…)` as a call by looking at the first letter; splitting
//   the token does the same without ambiguity.
// - Keywords are extracted from the `word` token, so `type`, `error`,
//   `action`, `on` and the rest are keywords only where the parser expects
//   them and plain names everywhere else — a prop called `type`, a map key
//   called `action`, `Array.from`.
// - A style value is raw CSS to the end of its line or a `;`, with
//   `$token` and `{expr}` splices inside it. The end of a line means
//   something there and nowhere else — except in the indented layout: a
//   `.wfx` file writes a block as the lines indented under the line that
//   opens it, and the same external scanner (`src/scanner.c`) turns that
//   into the `_indent` and `_dedent` tokens a block may be delimited by
//   instead of braces. One grammar reads both layouts, as the compiler's
//   lexer does (`src/lexer/v2.rs`, `Layout`).

const PREC = {
  range: -1,
  coalesce: 0,
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

// Every built-in the compiler knows (`lexer::token::ALL_COMPONENT_NAMES`).
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
  "DatePicker", "FileUpload", "Form", "Textarea",
  // Feedback
  "Alert", "Toast", "Modal", "Dialog", "Spinner", "Progress", "Skeleton",
  // Actions
  "Button", "IconButton", "ButtonGroup", "Dropdown",
  // Media
  "Image", "Video", "Audio", "Icon", "Carousel",
  // Typography
  "Text", "Heading", "Code", "Blockquote", "Markdown", "Unsafe",
  // A node handed to somebody else's code.
  "Host",
  // Document (PDF)
  "Document", "Section", "Paragraph", "PageBreak", "Header", "Footer",
  // Slides
  "Presentation", "Slide", "TitleSlide", "SectionSlide", "TwoColumn",
  "ImageSlide",
];

// One grammar file, two grammars: `tree-sitter-webfluent` reads `.wf`,
// where a block is braced; generated with `WFX=1` — as
// `tree-sitter-webfluentx` is — it reads `.wfx`, where a block may also be
// the lines indented under the line that opens it. A braced file can
// indent a continuation line deeper than the line before it, so the
// indent tokens are only offered to the grammar that needs them.
const OFFSIDE = process.env.WFX === "1";

/**
 * A block's body, between braces or — in the indented layout — between the
 * indent that follows the line opening it and the dedent that ends it.
 *
 * @param {GrammarSymbols<any>} $
 * @param {RuleOrLiteral} body
 */
function braced($, body) {
  const withBraces = seq("{", body, "}");
  return OFFSIDE ? choice(withBraces, seq($._indent, body, $._dedent)) : withBraces;
}

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
  name: OFFSIDE ? "webfluentx" : "webfluent",

  word: ($) => $.identifier,

  // `_layout` is the scanner's empty mark: a line at the same level, or
  // blanks it skipped and found nothing after.
  extras: ($) => [/\s/, $.doc_comment, $.comment, $._layout],

  // The layout tokens are declared in both grammars so one scanner serves
  // both; the braced grammar never asks for an indent or a dedent. The
  // braces go through the scanner too, counted: inside them the layout is
  // free.
  externals: ($) => [$.style_text, $._indent, $._dedent, $._layout, "{", "}"],

  conflicts: ($) => [
    [$.state_declaration],
    // `Button("x", aria-pressed: on)` versus `Text(a - b)`: after the first
    // word both an attribute name and a subtraction are open until the `:`.
    [$.attribute_name, $._primary_expression],
    // `(a, b) => a - b` versus `(a)`: a parenthesised lambda parameter list
    // and a parenthesised expression look the same until the `=>`.
    [$.lambda_parameters, $._primary_expression],
    // `path: "/"` versus `id: String` in a page header: an attribute or a
    // route parameter, until what follows the colon.
    [$.route_parameter, $.attribute_name],
    // `if a { go() }`: the call is the statement of the block, or the value
    // of an `if` expression on which a member or call could follow;
    // `a.b()`: `a` is a statement or the object of a member. Both readings
    // run until the next token settles it, and the statement wins a tie.
    [$.expression_statement, $._primary_expression],
    [$.expression_statement, $._expression],
  ],

  // Inlined so that `element`'s precedence applies to the name itself: a
  // capitalised word at the start of a statement is an element, not an
  // expression, and the choice is made on that token.
  inline: ($) => [$._element_name, $._name, $._declaration_name],

  supertypes: ($) => [$._declaration, $._statement, $._expression],

  rules: {
    source_file: ($) => repeat($._declaration),

    // ─── Declarations ───────────────────────────────────────────────────

    _declaration: ($) =>
      choice(
        $.page_declaration,
        $.component_declaration,
        $.store_declaration,
        $.external_declaration,
        $.theme_declaration,
        $.app_declaration,
        $.type_declaration,
        $.enum_declaration,
        $.const_declaration,
        $.data_declaration,
        $.animation_declaration,
        $.test_declaration,
      ),

    // `const API = "/api"`
    const_declaration: ($) =>
      seq(
        "const",
        field("name", $._name),
        optional(seq(":", field("type", $._type))),
        "=",
        field("value", $._expression),
      ),

    // `data posts = "posts.json"` — a file's JSON, a constant at build time.
    data_declaration: ($) =>
      seq(
        "data",
        field("name", $._name),
        optional(seq(":", field("type", $._type))),
        "=",
        field("file", $.string),
      ),

    // `page Home(path: "/", title: "Home", id: String, layout: Shell) { }`
    page_declaration: ($) =>
      seq(
        "page",
        field("name", $._declaration_name),
        optional(field("attributes", $.page_attributes)),
        field("body", $.block),
      ),

    page_attributes: ($) => seq("(", sepBy(choice($.route_parameter, $.named_argument), ","), ")"),

    // `id: String` — a route parameter, told from an attribute by its type;
    // `String` is also a name an expression may be, so the parameter wins
    // where both readings survive.
    route_parameter: ($) =>
      prec.dynamic(1, seq(field("name", $.identifier), ":", field("type", $._type))),

    // `component Name(_ label: String, tone: Tone = .calm) { }`
    component_declaration: ($) =>
      seq(
        "component",
        field("name", $._declaration_name),
        optional(field("parameters", $.parameter_list)),
        field("body", $.block),
      ),

    // `part Header(_ text: String) { … }` — a component of its owner's,
    // called `Owner.Header`.
    part_declaration: ($) =>
      seq(
        "part",
        field("name", $._declaration_name),
        optional(field("parameters", $.parameter_list)),
        field("body", $.block),
      ),

    // `external Chart from "chart.js" { fn … type … }`, and
    // `external element Stripe("stripe-pricing-table") { prop … event … }`.
    external_declaration: ($) =>
      seq(
        "external",
        optional("element"),
        field("name", $._name),
        choice(
          seq("from", field("from", $.string)),
          seq("(", field("tag", $.string), ")"),
        ),
        optional(field("body", $.external_body)),
      ),

    external_body: ($) =>
      braced(
        $,
        repeat(
          choice(
            $.external_fn,
            $.external_type,
            $.external_prop,
            $.event_declaration,
            seq("integrity", ":", $.string),
          ),
        ),
      ),

    external_fn: ($) =>
      seq(
        "fn",
        field("name", $._name),
        optional(field("parameters", $.parameter_list)),
        optional(seq("-", ">", field("returns", $._type))),
      ),

    external_type: ($) =>
      seq(
        "type",
        field("name", $._name),
        braced($, repeat(seq(choice($.external_member, $.parameter), optional(",")))),
      ),

    external_member: ($) => seq(field("name", $._name), field("parameters", $.parameter_list)),

    external_prop: ($) => seq("prop", $.parameter),

    store_declaration: ($) =>
      seq(
        "store",
        field("name", $._name),
        // `store Cart(scope: .route, eager: true)`
        optional(field("options", $.arguments)),
        field("body", $.block),
      ),

    theme_declaration: ($) =>
      seq("theme", field("name", $._name), field("body", $.theme_body)),

    theme_body: ($) => braced($, repeat($.token_declaration)),

    // `animation Pulse { from { opacity: 1 } 50% { opacity: 0.4 } to { opacity: 1 } }`
    animation_declaration: ($) =>
      seq("animation", field("name", $._declaration_name), braced($, repeat($.keyframe))),

    keyframe: ($) =>
      seq(field("selector", $.keyframe_selector), braced($, repeat($.style_property))),

    keyframe_selector: (_) => token(prec(1, /(from|to|[0-9]+(\.[0-9]+)?%(\s*,\s*[0-9]+(\.[0-9]+)?%)*)/)),

    // `test "name"(data: { … }) { elements  expect "text" }`
    test_declaration: ($) =>
      seq(
        "test",
        field("name", $.string),
        optional(field("arguments", $.argument_list)),
        field("body", $.block),
      ),

    // `expect "text"`, `expect not "text"` — inside a test.
    expect_statement: ($) => seq("expect", optional("not"), field("text", $._expression)),

    // What a test does, as against what it then expects: `click "Save"`,
    // `type "Ada" into "Name"`, `press "Escape" in "Search"`.
    act_statement: ($) =>
      choice(
        seq("click", field("target", $._expression)),
        seq("type", field("text", $.string), "into", field("into", $._expression)),
        seq("press", field("key", $._expression), optional(seq("in", field("target", $._expression)))),
      ),

    // `color-primary: #6366F1` — a raw CSS value.
    token_declaration: ($) =>
      seq(field("name", $.property_name), ":", field("value", $.style_value), optional(";")),

    app_declaration: ($) => seq("app", field("body", $.block)),

    // `type Todo { id: String, done: Bool = false }`
    type_declaration: ($) =>
      seq(
        "type",
        field("name", $._name),
        braced($, seq(sepBy($.field_declaration, optional(",")), optional(","))),
      ),

    field_declaration: ($) =>
      seq(
        field("name", $._name),
        ":",
        field("type", $._type),
        optional(seq("=", field("default", $._expression))),
      ),

    // `enum Tone { calm, loud }`; a case may carry a payload:
    // `enum Status { idle, failed(reason: String) }`.
    enum_declaration: ($) =>
      seq(
        "enum",
        field("name", $._name),
        braced($, seq(sepBy($.enum_case_declaration, optional(",")), optional(","))),
      ),

    enum_case_declaration: ($) =>
      seq(
        field("name", $.identifier),
        optional(seq("(", sepBy1($.field_declaration, ","), optional(","), ")")),
      ),

    // A page or component may be named after a built-in (`page Menu`).
    _declaration_name: ($) => choice($._name, $.builtin_component),

    _name: ($) => choice($.identifier, $.component_identifier),

    // ─── Parameters and types ───────────────────────────────────────────

    parameter_list: ($) => seq("(", sepBy($.parameter, ","), ")"),

    // `_ label: String` marks the one positional prop.
    parameter: ($) =>
      seq(
        optional(field("positional", "_")),
        field("name", $._name),
        ":",
        field("type", $._type),
        optional(seq("=", field("default", $._expression))),
      ),

    _type: ($) => choice($.builtin_type, $.list_type, $.optional_type, $.named_type),

    builtin_type: (_) => choice("String", "Number", "Bool", "Map", "Any"),

    list_type: ($) => seq("[", $._type, "]"),

    optional_type: ($) => prec.left(seq($._type, "?")),

    named_type: ($) => $.component_identifier,

    // ─── Statements ─────────────────────────────────────────────────────

    block: ($) => braced($, repeat($._statement)),

    _statement: ($) =>
      choice(
        $.state_declaration,
        $.derived_declaration,
        $.effect_declaration,
        $.action_declaration,
        $.use_declaration,
        $.resource_declaration,
        $.event_declaration,
        $.slot_declaration,
        $.part_declaration,
        $.timer_statement,
        $.cleanup_block,
        $.head_block,
        $.expect_statement,
        $.act_statement,
        $.let_declaration,
        $.try_statement,
        $.if_statement,
        $.for_statement,
        $.show_statement,
        $.sequence_statement,
        $.match_statement,
        $.navigate_statement,
        $.log_statement,
        $.return_statement,
        $.emit_statement,
        $.children,
        $.event_handler,
        $.style_block,
        $.transition_block,
        $.slot_fill,
        $.call_statement,
        $.element,
        $.assignment,
        $.expression_statement,
      ),

    // `state n = 0`; `persist theme = "light"` is kept across visits, and
    // may say where, at what version, and how an older value is brought
    // forward.
    state_declaration: ($) =>
      seq(
        choice("state", "persist"),
        field("name", $._name),
        optional(seq(":", field("type", $._type))),
        "=",
        field("value", $._expression),
        // Only a `persist` carries one, and the parser knows it from the
        // keyword; here it is the brace that follows the value.
        optional(field("policy", $.persist_policy)),
      ),

    persist_policy: ($) =>
      braced($, repeat(choice($.persist_option, $.migration))),

    persist_option: ($) =>
      seq(field("name", $.identifier), ":", field("value", $._expression)),

    // `migrate 1 -> 2 { old.map(…) }`
    migration: ($) =>
      seq(
        "migrate",
        field("from", $.number),
        "-",
        ">",
        field("to", $.number),
        braced($, $._expression),
      ),

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

    // `resource rows = fetch(url, method: "POST")`
    resource_declaration: ($) =>
      seq(
        "resource",
        field("name", $._name),
        optional(seq(":", field("type", $._type))),
        "=",
        field("value", $._expression),
      ),

    // `event toggle(id: String)`
    event_declaration: ($) =>
      prec.right(seq("event", field("name", $._name), optional(field("parameters", $.parameter_list)))),

    // `slot trailing`, or a bare `slot` for the default one; a scoped slot
    // names what it hands its fill: `slot row(item: Todo)`.
    slot_declaration: ($) =>
      prec.right(
        seq(
          "slot",
          optional(seq(field("name", $.identifier), optional(field("parameters", $.parameter_list)))),
        ),
      ),

    let_declaration: ($) =>
      seq(
        "let",
        choice(
          seq(field("name", $._name), optional(seq(":", field("type", $._type)))),
          // `let { a, b } = m`, `let [x, y] = l`
          field("pattern", $.destructuring_pattern),
        ),
        "=",
        field("value", $._expression),
      ),

    destructuring_pattern: ($) =>
      choice(
        seq("{", sepBy($._name, ","), optional(","), "}"),
        seq("[", sepBy($._name, ","), optional(","), "]"),
      ),

    // `try { … } catch e { … }`
    try_statement: ($) =>
      seq(
        "try",
        field("body", $.block),
        "catch",
        optional(field("error", $.identifier)),
        field("handler", $.block),
      ),

    navigate_statement: ($) => seq("navigate", "(", $._expression, ")"),

    log_statement: ($) => seq("log", "(", $._expression, ")"),

    // `return` alone at the end of a block, or `return expr`.
    return_statement: ($) => prec.right(seq("return", optional($._expression))),

    // `emit toggle(todo.id)`
    emit_statement: ($) =>
      prec.right(seq("emit", field("event", $._name), optional(field("arguments", $.arguments)))),

    children: (_) => "children",

    // ─── Control flow ───────────────────────────────────────────────────

    if_statement: ($) =>
      prec.right(
        seq(
          "if",
          choice(
            field("condition", $._expression),
            field("binding", $.if_let),
          ),
          field("consequence", $.block),
          optional(field("alternative", $.else_clause)),
        ),
      ),

    // `if let hint = expr { }`, or `if let hint { }` for a name in scope.
    if_let: ($) =>
      prec.right(seq("let", field("name", $._name), optional(seq("=", field("value", $._expression))))),

    else_clause: ($) => seq("else", choice($.if_statement, $.block)),

    // `for item, i in items by item.id { }`
    for_statement: ($) =>
      seq(
        "for",
        field("item", $._name),
        optional(seq(",", field("index", $._name))),
        "in",
        field("iterable", $._expression),
        optional(seq("by", field("key", $._expression))),
        field("body", $.block),
      ),

    show_statement: ($) =>
      seq("show", field("condition", $._expression), field("body", $.block)),

    // `sequence { step { … } step(after: "120ms") { … } }` — orchestration,
    // which the compiler lowers to a `delay:` on each step's elements.
    sequence_statement: ($) => seq("sequence", braced($, repeat($.step))),

    step: ($) =>
      seq(
        "step",
        optional(seq("(", "after", ":", field("after", $.string), ")")),
        field("body", $.block),
      ),

    // `match users { loading { } error(e) { } ready(list) { } else { } }`
    match_statement: ($) =>
      seq("match", field("value", $._expression), braced($, repeat($.match_arm))),

    match_arm: ($) => seq(field("pattern", $._arm_pattern), field("body", $.block)),

    _arm_pattern: ($) =>
      choice($.loading_pattern, $.error_pattern, $.ready_pattern, $.case_pattern, $.else_pattern),

    loading_pattern: (_) => "loading",

    error_pattern: ($) => seq("error", optional(seq("(", field("name", $._name), ")"))),

    ready_pattern: ($) => seq("ready", optional(seq("(", field("name", $._name), ")"))),

    // `.failed(reason)` binds the case's payload, one name per part.
    case_pattern: ($) =>
      seq($.enum_case, optional(seq("(", sepBy1(field("binding", $._name), ","), ")"))),

    else_pattern: (_) => "else",

    // ─── Elements ───────────────────────────────────────────────────────

    // `Card.elevated { … }`, `Button("Save").primary.lg { on click { } }`,
    // `Table.Row { }`, `UserCard(name: "x")`, `Router`.
    element: ($) =>
      prec.right(
        PREC.element,
        seq(
          field("name", $._element_name),
          optional(field("arguments", $.argument_list)),
          repeat(field("flag", $.flag)),
          optional(field("body", $.block)),
        ),
      ),

    _element_name: ($) => choice($.part, $.builtin_component, $.component_identifier),

    // `Table.Row`, `Card.Header`: a part of a built-in, or `Panel.Header`,
    // a part a component of the project declares — capitalised after the dot.
    part: ($) =>
      prec(
        PREC.element,
        seq(
          field("owner", choice($.builtin_component, $.component_identifier)),
          ".",
          field("name", choice($.builtin_component, $.component_identifier)),
        ),
      ),

    // `.primary`, `.lg`, `.required` — a boolean prop or an enum case,
    // written tight against the element: `Button("x").primary.lg`. The
    // absence of a space is what tells it from a case on the next line.
    flag: ($) => $._dot_word,

    _dot_word: (_) => token.immediate(seq(".", /[a-z_][a-zA-Z0-9_]*/)),

    // `List` is also a type; in element and expression position it is the
    // component, and a type is only ever read where one is expected.
    builtin_component: (_) => prec(1, choice(...BUILTIN_COMPONENTS)),

    // ─── Arguments ──────────────────────────────────────────────────────

    // Element arguments: one positional value first, then `name: value`
    // pairs. The precedence settles `Foo(x)` at the start of a statement as
    // an element rather than a call.
    argument_list: ($) => prec(1, seq("(", sepBy($._argument, ","), ")")),

    _argument: ($) => choice($.named_argument, $._expression),

    named_argument: ($) =>
      seq(field("name", $.attribute_name), ":", field("value", $._expression)),

    // `bind`, `aria-pressed`, `data-tone`: hyphenated because an HTML
    // attribute may be.
    attribute_name: ($) => seq($.identifier, repeat(seq("-", $.identifier))),

    // Call arguments; `name: value` is accepted so `t("greeting", name:
    // user.name)`, `fetch(url, method: "POST")` and `Todo(id: 1)` read as
    // one call each.
    arguments: ($) => seq("(", sepBy($._argument, ","), ")"),

    // ─── Events and slots ───────────────────────────────────────────────

    // `on click { }`, `on keydown(event) { }`, `on toggle(id) { }`.
    // `on click(e) { }`; `on key("ctrl+k") { }` names the key it answers to.
    event_handler: ($) =>
      seq(
        "on",
        field("event", $.identifier),
        optional(
          seq(
            "(",
            choice(
              seq(field("key", $.string), optional(seq(",", field("parameter", $._name)))),
              field("parameter", $._name),
            ),
            ")",
          ),
        ),
        field("body", $.block),
      ),

    // `every(1000) { tick() }`, `after(500) { hide() }`.
    timer_statement: ($) =>
      seq(
        field("kind", choice("every", "after")),
        "(",
        field("interval", $._expression),
        ")",
        field("body", $.block),
      ),

    // `cleanup { … }` closes an effect's body.
    cleanup_block: ($) => seq("cleanup", field("body", $.block)),

    // `head { meta(name: "x", content: y) link(…) script(…) }` — a page's own head tags.
    head_block: ($) => seq("head", braced($, repeat($.head_tag))),

    head_tag: ($) =>
      seq(field("tag", alias(choice("meta", "link", "script"), $.identifier)), field("arguments", $.arguments)),

    // `trailing { … }`; a scoped slot's fill names its values: `row(item) { … }`.
    slot_fill: ($) =>
      prec(
        1,
        seq(
          field("name", $.identifier),
          optional(seq("(", sepBy1(field("parameter", $.identifier), ","), ")")),
          field("body", $.block),
        ),
      ),

    // ─── Style ──────────────────────────────────────────────────────────

    style_block: ($) =>
      seq("style", braced($, repeat(choice($.style_property, $.nested_rule)))),

    // `padding: 6px 0`, `--accent: {color}`, `background: $surface;`
    style_property: ($) =>
      seq(
        field(
          "name",
          choice($.property_name, $.vendor_property_name, $.custom_property_name),
        ),
        ":",
        field("value", $.style_value),
        optional(";"),
      ),

    // Raw CSS to the end of the line or a `;`, with tokens and splices.
    style_value: ($) => repeat1(choice($.style_text, $.design_token, $.interpolation)),

    // `&:hover { }`, `&[aria-current="page"] { }`, `@media (max-width: 768px) { }`.
    nested_rule: ($) =>
      seq(
        field("selector", $.selector),
        braced($, repeat(choice($.style_property, $.nested_rule))),
      ),

    selector: (_) => /[&@.:\[>*+~][^{}\n]*/,

    // `border-radius`, `color-text-muted`, `viz-1`, `radius-2xl`.
    property_name: ($) =>
      seq(
        $.identifier,
        repeat(seq("-", choice($.identifier, seq($.number, optional($.identifier))))),
      ),

    // `-webkit-line-clamp`, `-moz-osx-font-smoothing` — a vendor prefix is
    // a property name that opens with a `-`, which is otherwise a
    // separator between two words of one.
    vendor_property_name: (_) => token(seq("-", /[a-zA-Z][a-zA-Z0-9_-]*/)),

    // `--hover-bg: hoverColor` — a custom property a nested rule can read.
    custom_property_name: (_) => token(seq("--", /[a-zA-Z_][a-zA-Z0-9_-]*/)),

    // ─── Transition ─────────────────────────────────────────────────────

    // `transition { background: 200ms ease-out }` — raw values, like a style.
    transition_block: ($) =>
      seq("transition", braced($, repeat($.style_property))),

    // ─── Assignment & expression statements ─────────────────────────────

    // `Store.load(id)` — a call on a store or component name, which at the
    // top of a page is set-up code. It outranks the element `Store` with a
    // flag `.load`, which could not take the parenthesis.
    call_statement: ($) =>
      prec(
        PREC.element + 1,
        seq(
          field("object", $.component_identifier),
          field("method", alias($._dot_word, $.method)),
          field("arguments", $.arguments),
        ),
      ),

    assignment: ($) =>
      seq(
        field("target", choice($._name, $.member_expression, $.index_expression)),
        "=",
        field("value", $._expression),
      ),

    // A statement that is an expression does something: a call, a member
    // call, an `await`, or a bare name (a slot's content). A value — a
    // literal, an `if` or `match` value — is never a statement on its own,
    // which is what keeps `if a { go() }` the statement and `.lg { }`
    // after an element its flag and block.
    expression_statement: ($) =>
      prec.dynamic(
        1,
        choice(
          $.call_expression,
          $.member_expression,
          $.index_expression,
          $.unary_expression,
          $.identifier,
          $.component_identifier,
          $.builtin_component,
        ),
      ),

    // ─── Expressions ────────────────────────────────────────────────────

    _expression: ($) =>
      choice(
        $.range_expression,
        $.binary_expression,
        $.unary_expression,
        $.lambda,
        $.if_expression,
        $.match_expression,
        $._primary_expression,
      ),

    _primary_expression: ($) =>
      choice(
        $.member_expression,
        $.index_expression,
        $.call_expression,
        $.parenthesized_expression,
        $.string,
        $.regex,
        $.number,
        $.boolean,
        $.null,
        $.array,
        $.object,
        $.case_value,
        $.enum_case,
        $.design_token,
        $.identifier,
        $.component_identifier,
        $.builtin_type,
        $.builtin_component,
      ),

    binary_expression: ($) => {
      const table = [
        [PREC.coalesce, "??"],
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
      prec(
        PREC.unary,
        seq(field("operator", choice("!", "-", "await")), field("operand", $._expression)),
      ),

    // `event` is a keyword where a statement starts, so a handler's
    // `event.preventDefault()` names it by its keyword; everywhere else the
    // word lexes as a plain name.
    member_expression: ($) =>
      prec.left(
        PREC.postfix,
        seq(
          field("object", choice($._expression, alias("event", $.identifier))),
          // `?.` reads null through null.
          choice(".", "?."),
          // Any word may follow a dot — `item.action`, `Array.from`.
          field("property", $._name),
        ),
      ),

    index_expression: ($) =>
      prec.left(
        PREC.postfix,
        seq(
          field("object", $._expression),
          optional("?."),
          "[",
          field("index", $._expression),
          "]",
        ),
      ),

    // Only a name, a member, an index or another call is callable.
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

    // `if cond { a } else { b }` as a value, chained with `else if`. The
    // low precedence keeps `if a { go() }` at the start of a statement the
    // statement, whose block holds statements, not a value.
    if_expression: ($) =>
      prec(
        -2,
        seq(
          "if",
          choice(
            // `if let x = e { … }` binds the non-null value.
            seq("let", field("binding", $.identifier), "=", field("value", $._expression)),
            field("condition", $._expression),
          ),
          "{",
          field("consequence", $._expression),
          "}",
          "else",
          choice($.if_expression, seq("{", field("alternative", $._expression), "}")),
        ),
      ),

    // `match tone { .calm { 1 } else { 2 } }` as a value.
    match_expression: ($) =>
      prec(
        -2,
        seq("match", field("value", $._expression), "{", repeat($.match_expression_arm), "}"),
      ),

    match_expression_arm: ($) =>
      seq(field("pattern", $._arm_pattern), "{", field("value", $._expression), "}"),

    // `.primary`, `.calm` — a case of an enum, named by the prop or type.
    enum_case: ($) => seq(".", field("name", $.identifier)),

    // `.failed("boom")` — a case with its payload.
    case_value: ($) =>
      prec(PREC.postfix, seq(".", field("name", $.identifier), field("arguments", $.arguments))),

    // `$surface`, `$spacing-xl` — a design token.
    design_token: (_) => token(seq("$", /[a-zA-Z_][a-zA-Z0-9_-]*/)),

    // ─── Literals ───────────────────────────────────────────────────────

    string: ($) => choice($._quoted_string, $.raw_string, $.block_string),

    _quoted_string: ($) =>
      seq(
        '"',
        repeat(choice($.string_content, $.escape_sequence, $.interpolation)),
        choice('"', alias(token.immediate(/\{"/), '"')),
      ),

    // `#"…"#`, with as many `#` as the text needs: no escapes, no splices,
    // so a sample of code goes in whole. One token, since what ends it is
    // the run of hashes it opened with.
    raw_string: (_) =>
      token(
        choice(
          seq('#"', repeat(choice(/[^"]/, /"[^#]/)), '"#'),
          seq('##"', repeat(choice(/[^"]/, /"[^#]/, /"#[^#]/)), '"##'),
        ),
      ),

    // `"""…"""` — escapes and splices, over as many lines as it likes.
    block_string: ($) =>
      seq(
        '"""',
        repeat(choice($.block_string_content, $.escape_sequence, $.interpolation)),
        '"""',
      ),

    block_string_content: (_) =>
      choice(
        token.immediate(prec(1, /([^"\\{]|"[^"]|""[^"])+/)),
        token.immediate(/\{[^a-zA-Z_"\\{\[(]/),
      ),

    // A `{` opens an interpolation only when a name, a `[` or a `(` follows
    // it; `"{ a: 1 }"` and `"{"` are text, as they are for the compiler. A
    // `{` right before the closing quote is text too.
    string_content: (_) =>
      choice(
        token.immediate(prec(1, /[^"\\{]+/)),
        token.immediate(/\{[^a-zA-Z_"\\{\[(]/),
      ),

    escape_sequence: (_) => token.immediate(/\\./),

    // Its braces are the grammar's own tokens, not the scanner's: inside a
    // string or a style value they are not the writer's block braces.
    interpolation: ($) =>
      seq(alias(/\{/, "{"), $._expression, optional($.format_spec), alias(/\}/, "}")),

    // `{total:.currency}`, `{when:.date(long)}` — how to show the value,
    // written where it is read.
    format_spec: ($) =>
      seq(
        ":",
        ".",
        field("style", $.identifier),
        optional(seq("(", field("option", choice($.identifier, $.number, $.string)), ")")),
      ),

    number: (_) => /\d+(\.\d+)?/,

    // `/pattern/flags`, where a value may start; a `/` after a value is the
    // operator, which the contextual lexer tells apart.
    regex: (_) =>
      token(
        seq(
          "/",
          // The first character is never `*` or `/`: those start a comment.
          /[^/\\\n\[*]|\\.|\[[^\]\n]*\]/,
          /([^/\\\n\[]|\\.|\[[^\]\n]*\])*/,
          "/",
          /[a-z]*/,
        ),
      ),

    boolean: (_) => choice("true", "false"),

    null: (_) => "null",

    array: ($) =>
      seq("[", sepBy(choice($._expression, $.spread_element), ","), optional(","), "]"),

    object: ($) => seq("{", sepBy(choice($.pair, $.spread_element), ","), optional(","), "}"),

    // `...items` inside a list or a map.
    spread_element: ($) => seq("...", $._expression),

    pair: ($) =>
      seq(
        field("key", choice($._name, $.string)),
        ":",
        field("value", $._expression),
      ),

    // `a..b`, `a..=b`
    range_expression: ($) =>
      prec.left(PREC.range, seq($._expression, choice("..", "..="), $._expression)),

    // ─── Names ──────────────────────────────────────────────────────────

    identifier: (_) => /[a-z_][a-zA-Z0-9_]*/,

    component_identifier: (_) => /[A-Z][a-zA-Z0-9_]*/,

    // ─── Comments ───────────────────────────────────────────────────────

    // `/// …` documents the declaration, prop or event that follows.
    doc_comment: (_) => token(prec(1, seq("///", /.*/))),

    comment: (_) =>
      token(
        choice(
          seq("//", /.*/),
          seq("/*", /[^*]*\*+([^/*][^*]*\*+)*/, "/"),
        ),
      ),
  },
});
