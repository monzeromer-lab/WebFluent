#include "tree_sitter/parser.h"

#if defined(__GNUC__) || defined(__clang__)
#pragma GCC diagnostic ignored "-Wmissing-field-initializers"
#endif

#ifdef _MSC_VER
#pragma optimize("", off)
#elif defined(__clang__)
#pragma clang optimize off
#elif defined(__GNUC__)
#pragma GCC optimize ("O0")
#endif

#define LANGUAGE_VERSION 14
#define STATE_COUNT 445
#define LARGE_STATE_COUNT 7
#define SYMBOL_COUNT 133
#define ALIAS_COUNT 0
#define TOKEN_COUNT 68
#define EXTERNAL_TOKEN_COUNT 0
#define FIELD_COUNT 2
#define MAX_ALIAS_SEQUENCE_LENGTH 9
#define PRODUCTION_ID_COUNT 2

enum ts_symbol_identifiers {
  anon_sym_Page = 1,
  anon_sym_LPAREN = 2,
  anon_sym_RPAREN = 3,
  anon_sym_Component = 4,
  anon_sym_Store = 5,
  anon_sym_App = 6,
  anon_sym_COMMA = 7,
  anon_sym_QMARK = 8,
  anon_sym_COLON = 9,
  anon_sym_EQ = 10,
  sym_modifier = 11,
  sym_type_annotation = 12,
  anon_sym_LBRACE = 13,
  anon_sym_RBRACE = 14,
  anon_sym_state = 15,
  anon_sym_derived = 16,
  anon_sym_effect = 17,
  anon_sym_action = 18,
  anon_sym_use = 19,
  anon_sym_navigate = 20,
  anon_sym_DOT = 21,
  sym_builtin_component = 22,
  anon_sym_if = 23,
  anon_sym_else = 24,
  anon_sym_for = 25,
  anon_sym_in = 26,
  anon_sym_show = 27,
  anon_sym_animate = 28,
  anon_sym_fetch = 29,
  anon_sym_from = 30,
  anon_sym_loading = 31,
  anon_sym_error = 32,
  anon_sym_success = 33,
  sym_event_name = 34,
  anon_sym_style = 35,
  anon_sym_DASH = 36,
  anon_sym_ATmedia = 37,
  aux_sym_media_query_token1 = 38,
  anon_sym_transition = 39,
  anon_sym_PLUS = 40,
  anon_sym_STAR = 41,
  anon_sym_SLASH = 42,
  anon_sym_PERCENT = 43,
  anon_sym_EQ_EQ = 44,
  anon_sym_BANG_EQ = 45,
  anon_sym_LT = 46,
  anon_sym_GT = 47,
  anon_sym_LT_EQ = 48,
  anon_sym_GT_EQ = 49,
  anon_sym_AMP_AMP = 50,
  anon_sym_PIPE_PIPE = 51,
  anon_sym_BANG = 52,
  anon_sym_t = 53,
  anon_sym_DQUOTE = 54,
  sym_string_content = 55,
  sym_escape_sequence = 56,
  sym_number = 57,
  anon_sym_true = 58,
  anon_sym_false = 59,
  sym_null = 60,
  anon_sym_LBRACK = 61,
  anon_sym_RBRACK = 62,
  sym_identifier = 63,
  anon_sym_SLASH_SLASH = 64,
  aux_sym_comment_token1 = 65,
  anon_sym_SLASH_STAR = 66,
  aux_sym_comment_token2 = 67,
  sym_source_file = 68,
  sym__declaration = 69,
  sym_page_declaration = 70,
  sym_component_declaration = 71,
  sym_store_declaration = 72,
  sym_app_declaration = 73,
  sym_parameter_list = 74,
  sym_parameter = 75,
  sym_argument_list = 76,
  sym__argument = 77,
  sym_named_argument = 78,
  sym_block = 79,
  sym__statement = 80,
  sym_state_declaration = 81,
  sym_derived_declaration = 82,
  sym_effect_declaration = 83,
  sym_action_declaration = 84,
  sym_use_declaration = 85,
  sym_navigate_call = 86,
  sym_assignment = 87,
  sym_expression_statement = 88,
  sym_ui_element = 89,
  sym_component_name = 90,
  sym_sub_component = 91,
  sym_if_statement = 92,
  sym_for_statement = 93,
  sym_show_statement = 94,
  sym_animate_clause = 95,
  sym_fetch_statement = 96,
  sym_fetch_block = 97,
  sym_loading_block = 98,
  sym_error_block = 99,
  sym_success_block = 100,
  sym_event_handler = 101,
  sym_style_block = 102,
  sym_style_property = 103,
  sym_property_name = 104,
  sym_media_query = 105,
  sym_transition_block = 106,
  sym_transition_property = 107,
  sym__expression = 108,
  sym_binary_expression = 109,
  sym_unary_expression = 110,
  sym_member_expression = 111,
  sym_call_expression = 112,
  sym_t_call = 113,
  sym_string = 114,
  sym_interpolation = 115,
  sym_boolean = 116,
  sym_array = 117,
  sym_object = 118,
  sym_object_entry = 119,
  sym_comment = 120,
  aux_sym_source_file_repeat1 = 121,
  aux_sym_parameter_list_repeat1 = 122,
  aux_sym_argument_list_repeat1 = 123,
  aux_sym_block_repeat1 = 124,
  aux_sym_fetch_block_repeat1 = 125,
  aux_sym_style_block_repeat1 = 126,
  aux_sym_property_name_repeat1 = 127,
  aux_sym_media_query_repeat1 = 128,
  aux_sym_transition_block_repeat1 = 129,
  aux_sym_string_repeat1 = 130,
  aux_sym_array_repeat1 = 131,
  aux_sym_object_repeat1 = 132,
};

static const char * const ts_symbol_names[] = {
  [ts_builtin_sym_end] = "end",
  [anon_sym_Page] = "Page",
  [anon_sym_LPAREN] = "(",
  [anon_sym_RPAREN] = ")",
  [anon_sym_Component] = "Component",
  [anon_sym_Store] = "Store",
  [anon_sym_App] = "App",
  [anon_sym_COMMA] = ",",
  [anon_sym_QMARK] = "\?",
  [anon_sym_COLON] = ":",
  [anon_sym_EQ] = "=",
  [sym_modifier] = "modifier",
  [sym_type_annotation] = "type_annotation",
  [anon_sym_LBRACE] = "{",
  [anon_sym_RBRACE] = "}",
  [anon_sym_state] = "state",
  [anon_sym_derived] = "derived",
  [anon_sym_effect] = "effect",
  [anon_sym_action] = "action",
  [anon_sym_use] = "use",
  [anon_sym_navigate] = "navigate",
  [anon_sym_DOT] = ".",
  [sym_builtin_component] = "builtin_component",
  [anon_sym_if] = "if",
  [anon_sym_else] = "else",
  [anon_sym_for] = "for",
  [anon_sym_in] = "in",
  [anon_sym_show] = "show",
  [anon_sym_animate] = "animate",
  [anon_sym_fetch] = "fetch",
  [anon_sym_from] = "from",
  [anon_sym_loading] = "loading",
  [anon_sym_error] = "error",
  [anon_sym_success] = "success",
  [sym_event_name] = "event_name",
  [anon_sym_style] = "style",
  [anon_sym_DASH] = "-",
  [anon_sym_ATmedia] = "@media",
  [aux_sym_media_query_token1] = "media_query_token1",
  [anon_sym_transition] = "transition",
  [anon_sym_PLUS] = "+",
  [anon_sym_STAR] = "*",
  [anon_sym_SLASH] = "/",
  [anon_sym_PERCENT] = "%",
  [anon_sym_EQ_EQ] = "==",
  [anon_sym_BANG_EQ] = "!=",
  [anon_sym_LT] = "<",
  [anon_sym_GT] = ">",
  [anon_sym_LT_EQ] = "<=",
  [anon_sym_GT_EQ] = ">=",
  [anon_sym_AMP_AMP] = "&&",
  [anon_sym_PIPE_PIPE] = "||",
  [anon_sym_BANG] = "!",
  [anon_sym_t] = "t",
  [anon_sym_DQUOTE] = "\"",
  [sym_string_content] = "string_content",
  [sym_escape_sequence] = "escape_sequence",
  [sym_number] = "number",
  [anon_sym_true] = "true",
  [anon_sym_false] = "false",
  [sym_null] = "null",
  [anon_sym_LBRACK] = "[",
  [anon_sym_RBRACK] = "]",
  [sym_identifier] = "identifier",
  [anon_sym_SLASH_SLASH] = "//",
  [aux_sym_comment_token1] = "comment_token1",
  [anon_sym_SLASH_STAR] = "/*",
  [aux_sym_comment_token2] = "comment_token2",
  [sym_source_file] = "source_file",
  [sym__declaration] = "_declaration",
  [sym_page_declaration] = "page_declaration",
  [sym_component_declaration] = "component_declaration",
  [sym_store_declaration] = "store_declaration",
  [sym_app_declaration] = "app_declaration",
  [sym_parameter_list] = "parameter_list",
  [sym_parameter] = "parameter",
  [sym_argument_list] = "argument_list",
  [sym__argument] = "_argument",
  [sym_named_argument] = "named_argument",
  [sym_block] = "block",
  [sym__statement] = "_statement",
  [sym_state_declaration] = "state_declaration",
  [sym_derived_declaration] = "derived_declaration",
  [sym_effect_declaration] = "effect_declaration",
  [sym_action_declaration] = "action_declaration",
  [sym_use_declaration] = "use_declaration",
  [sym_navigate_call] = "navigate_call",
  [sym_assignment] = "assignment",
  [sym_expression_statement] = "expression_statement",
  [sym_ui_element] = "ui_element",
  [sym_component_name] = "component_name",
  [sym_sub_component] = "sub_component",
  [sym_if_statement] = "if_statement",
  [sym_for_statement] = "for_statement",
  [sym_show_statement] = "show_statement",
  [sym_animate_clause] = "animate_clause",
  [sym_fetch_statement] = "fetch_statement",
  [sym_fetch_block] = "fetch_block",
  [sym_loading_block] = "loading_block",
  [sym_error_block] = "error_block",
  [sym_success_block] = "success_block",
  [sym_event_handler] = "event_handler",
  [sym_style_block] = "style_block",
  [sym_style_property] = "style_property",
  [sym_property_name] = "property_name",
  [sym_media_query] = "media_query",
  [sym_transition_block] = "transition_block",
  [sym_transition_property] = "transition_property",
  [sym__expression] = "_expression",
  [sym_binary_expression] = "binary_expression",
  [sym_unary_expression] = "unary_expression",
  [sym_member_expression] = "member_expression",
  [sym_call_expression] = "call_expression",
  [sym_t_call] = "t_call",
  [sym_string] = "string",
  [sym_interpolation] = "interpolation",
  [sym_boolean] = "boolean",
  [sym_array] = "array",
  [sym_object] = "object",
  [sym_object_entry] = "object_entry",
  [sym_comment] = "comment",
  [aux_sym_source_file_repeat1] = "source_file_repeat1",
  [aux_sym_parameter_list_repeat1] = "parameter_list_repeat1",
  [aux_sym_argument_list_repeat1] = "argument_list_repeat1",
  [aux_sym_block_repeat1] = "block_repeat1",
  [aux_sym_fetch_block_repeat1] = "fetch_block_repeat1",
  [aux_sym_style_block_repeat1] = "style_block_repeat1",
  [aux_sym_property_name_repeat1] = "property_name_repeat1",
  [aux_sym_media_query_repeat1] = "media_query_repeat1",
  [aux_sym_transition_block_repeat1] = "transition_block_repeat1",
  [aux_sym_string_repeat1] = "string_repeat1",
  [aux_sym_array_repeat1] = "array_repeat1",
  [aux_sym_object_repeat1] = "object_repeat1",
};

static const TSSymbol ts_symbol_map[] = {
  [ts_builtin_sym_end] = ts_builtin_sym_end,
  [anon_sym_Page] = anon_sym_Page,
  [anon_sym_LPAREN] = anon_sym_LPAREN,
  [anon_sym_RPAREN] = anon_sym_RPAREN,
  [anon_sym_Component] = anon_sym_Component,
  [anon_sym_Store] = anon_sym_Store,
  [anon_sym_App] = anon_sym_App,
  [anon_sym_COMMA] = anon_sym_COMMA,
  [anon_sym_QMARK] = anon_sym_QMARK,
  [anon_sym_COLON] = anon_sym_COLON,
  [anon_sym_EQ] = anon_sym_EQ,
  [sym_modifier] = sym_modifier,
  [sym_type_annotation] = sym_type_annotation,
  [anon_sym_LBRACE] = anon_sym_LBRACE,
  [anon_sym_RBRACE] = anon_sym_RBRACE,
  [anon_sym_state] = anon_sym_state,
  [anon_sym_derived] = anon_sym_derived,
  [anon_sym_effect] = anon_sym_effect,
  [anon_sym_action] = anon_sym_action,
  [anon_sym_use] = anon_sym_use,
  [anon_sym_navigate] = anon_sym_navigate,
  [anon_sym_DOT] = anon_sym_DOT,
  [sym_builtin_component] = sym_builtin_component,
  [anon_sym_if] = anon_sym_if,
  [anon_sym_else] = anon_sym_else,
  [anon_sym_for] = anon_sym_for,
  [anon_sym_in] = anon_sym_in,
  [anon_sym_show] = anon_sym_show,
  [anon_sym_animate] = anon_sym_animate,
  [anon_sym_fetch] = anon_sym_fetch,
  [anon_sym_from] = anon_sym_from,
  [anon_sym_loading] = anon_sym_loading,
  [anon_sym_error] = anon_sym_error,
  [anon_sym_success] = anon_sym_success,
  [sym_event_name] = sym_event_name,
  [anon_sym_style] = anon_sym_style,
  [anon_sym_DASH] = anon_sym_DASH,
  [anon_sym_ATmedia] = anon_sym_ATmedia,
  [aux_sym_media_query_token1] = aux_sym_media_query_token1,
  [anon_sym_transition] = anon_sym_transition,
  [anon_sym_PLUS] = anon_sym_PLUS,
  [anon_sym_STAR] = anon_sym_STAR,
  [anon_sym_SLASH] = anon_sym_SLASH,
  [anon_sym_PERCENT] = anon_sym_PERCENT,
  [anon_sym_EQ_EQ] = anon_sym_EQ_EQ,
  [anon_sym_BANG_EQ] = anon_sym_BANG_EQ,
  [anon_sym_LT] = anon_sym_LT,
  [anon_sym_GT] = anon_sym_GT,
  [anon_sym_LT_EQ] = anon_sym_LT_EQ,
  [anon_sym_GT_EQ] = anon_sym_GT_EQ,
  [anon_sym_AMP_AMP] = anon_sym_AMP_AMP,
  [anon_sym_PIPE_PIPE] = anon_sym_PIPE_PIPE,
  [anon_sym_BANG] = anon_sym_BANG,
  [anon_sym_t] = anon_sym_t,
  [anon_sym_DQUOTE] = anon_sym_DQUOTE,
  [sym_string_content] = sym_string_content,
  [sym_escape_sequence] = sym_escape_sequence,
  [sym_number] = sym_number,
  [anon_sym_true] = anon_sym_true,
  [anon_sym_false] = anon_sym_false,
  [sym_null] = sym_null,
  [anon_sym_LBRACK] = anon_sym_LBRACK,
  [anon_sym_RBRACK] = anon_sym_RBRACK,
  [sym_identifier] = sym_identifier,
  [anon_sym_SLASH_SLASH] = anon_sym_SLASH_SLASH,
  [aux_sym_comment_token1] = aux_sym_comment_token1,
  [anon_sym_SLASH_STAR] = anon_sym_SLASH_STAR,
  [aux_sym_comment_token2] = aux_sym_comment_token2,
  [sym_source_file] = sym_source_file,
  [sym__declaration] = sym__declaration,
  [sym_page_declaration] = sym_page_declaration,
  [sym_component_declaration] = sym_component_declaration,
  [sym_store_declaration] = sym_store_declaration,
  [sym_app_declaration] = sym_app_declaration,
  [sym_parameter_list] = sym_parameter_list,
  [sym_parameter] = sym_parameter,
  [sym_argument_list] = sym_argument_list,
  [sym__argument] = sym__argument,
  [sym_named_argument] = sym_named_argument,
  [sym_block] = sym_block,
  [sym__statement] = sym__statement,
  [sym_state_declaration] = sym_state_declaration,
  [sym_derived_declaration] = sym_derived_declaration,
  [sym_effect_declaration] = sym_effect_declaration,
  [sym_action_declaration] = sym_action_declaration,
  [sym_use_declaration] = sym_use_declaration,
  [sym_navigate_call] = sym_navigate_call,
  [sym_assignment] = sym_assignment,
  [sym_expression_statement] = sym_expression_statement,
  [sym_ui_element] = sym_ui_element,
  [sym_component_name] = sym_component_name,
  [sym_sub_component] = sym_sub_component,
  [sym_if_statement] = sym_if_statement,
  [sym_for_statement] = sym_for_statement,
  [sym_show_statement] = sym_show_statement,
  [sym_animate_clause] = sym_animate_clause,
  [sym_fetch_statement] = sym_fetch_statement,
  [sym_fetch_block] = sym_fetch_block,
  [sym_loading_block] = sym_loading_block,
  [sym_error_block] = sym_error_block,
  [sym_success_block] = sym_success_block,
  [sym_event_handler] = sym_event_handler,
  [sym_style_block] = sym_style_block,
  [sym_style_property] = sym_style_property,
  [sym_property_name] = sym_property_name,
  [sym_media_query] = sym_media_query,
  [sym_transition_block] = sym_transition_block,
  [sym_transition_property] = sym_transition_property,
  [sym__expression] = sym__expression,
  [sym_binary_expression] = sym_binary_expression,
  [sym_unary_expression] = sym_unary_expression,
  [sym_member_expression] = sym_member_expression,
  [sym_call_expression] = sym_call_expression,
  [sym_t_call] = sym_t_call,
  [sym_string] = sym_string,
  [sym_interpolation] = sym_interpolation,
  [sym_boolean] = sym_boolean,
  [sym_array] = sym_array,
  [sym_object] = sym_object,
  [sym_object_entry] = sym_object_entry,
  [sym_comment] = sym_comment,
  [aux_sym_source_file_repeat1] = aux_sym_source_file_repeat1,
  [aux_sym_parameter_list_repeat1] = aux_sym_parameter_list_repeat1,
  [aux_sym_argument_list_repeat1] = aux_sym_argument_list_repeat1,
  [aux_sym_block_repeat1] = aux_sym_block_repeat1,
  [aux_sym_fetch_block_repeat1] = aux_sym_fetch_block_repeat1,
  [aux_sym_style_block_repeat1] = aux_sym_style_block_repeat1,
  [aux_sym_property_name_repeat1] = aux_sym_property_name_repeat1,
  [aux_sym_media_query_repeat1] = aux_sym_media_query_repeat1,
  [aux_sym_transition_block_repeat1] = aux_sym_transition_block_repeat1,
  [aux_sym_string_repeat1] = aux_sym_string_repeat1,
  [aux_sym_array_repeat1] = aux_sym_array_repeat1,
  [aux_sym_object_repeat1] = aux_sym_object_repeat1,
};

static const TSSymbolMetadata ts_symbol_metadata[] = {
  [ts_builtin_sym_end] = {
    .visible = false,
    .named = true,
  },
  [anon_sym_Page] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_LPAREN] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_RPAREN] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_Component] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_Store] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_App] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_COMMA] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_QMARK] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_COLON] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_EQ] = {
    .visible = true,
    .named = false,
  },
  [sym_modifier] = {
    .visible = true,
    .named = true,
  },
  [sym_type_annotation] = {
    .visible = true,
    .named = true,
  },
  [anon_sym_LBRACE] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_RBRACE] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_state] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_derived] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_effect] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_action] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_use] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_navigate] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_DOT] = {
    .visible = true,
    .named = false,
  },
  [sym_builtin_component] = {
    .visible = true,
    .named = true,
  },
  [anon_sym_if] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_else] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_for] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_in] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_show] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_animate] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_fetch] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_from] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_loading] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_error] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_success] = {
    .visible = true,
    .named = false,
  },
  [sym_event_name] = {
    .visible = true,
    .named = true,
  },
  [anon_sym_style] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_DASH] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_ATmedia] = {
    .visible = true,
    .named = false,
  },
  [aux_sym_media_query_token1] = {
    .visible = false,
    .named = false,
  },
  [anon_sym_transition] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_PLUS] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_STAR] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_SLASH] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_PERCENT] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_EQ_EQ] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_BANG_EQ] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_LT] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_GT] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_LT_EQ] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_GT_EQ] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_AMP_AMP] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_PIPE_PIPE] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_BANG] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_t] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_DQUOTE] = {
    .visible = true,
    .named = false,
  },
  [sym_string_content] = {
    .visible = true,
    .named = true,
  },
  [sym_escape_sequence] = {
    .visible = true,
    .named = true,
  },
  [sym_number] = {
    .visible = true,
    .named = true,
  },
  [anon_sym_true] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_false] = {
    .visible = true,
    .named = false,
  },
  [sym_null] = {
    .visible = true,
    .named = true,
  },
  [anon_sym_LBRACK] = {
    .visible = true,
    .named = false,
  },
  [anon_sym_RBRACK] = {
    .visible = true,
    .named = false,
  },
  [sym_identifier] = {
    .visible = true,
    .named = true,
  },
  [anon_sym_SLASH_SLASH] = {
    .visible = true,
    .named = false,
  },
  [aux_sym_comment_token1] = {
    .visible = false,
    .named = false,
  },
  [anon_sym_SLASH_STAR] = {
    .visible = true,
    .named = false,
  },
  [aux_sym_comment_token2] = {
    .visible = false,
    .named = false,
  },
  [sym_source_file] = {
    .visible = true,
    .named = true,
  },
  [sym__declaration] = {
    .visible = false,
    .named = true,
  },
  [sym_page_declaration] = {
    .visible = true,
    .named = true,
  },
  [sym_component_declaration] = {
    .visible = true,
    .named = true,
  },
  [sym_store_declaration] = {
    .visible = true,
    .named = true,
  },
  [sym_app_declaration] = {
    .visible = true,
    .named = true,
  },
  [sym_parameter_list] = {
    .visible = true,
    .named = true,
  },
  [sym_parameter] = {
    .visible = true,
    .named = true,
  },
  [sym_argument_list] = {
    .visible = true,
    .named = true,
  },
  [sym__argument] = {
    .visible = false,
    .named = true,
  },
  [sym_named_argument] = {
    .visible = true,
    .named = true,
  },
  [sym_block] = {
    .visible = true,
    .named = true,
  },
  [sym__statement] = {
    .visible = false,
    .named = true,
  },
  [sym_state_declaration] = {
    .visible = true,
    .named = true,
  },
  [sym_derived_declaration] = {
    .visible = true,
    .named = true,
  },
  [sym_effect_declaration] = {
    .visible = true,
    .named = true,
  },
  [sym_action_declaration] = {
    .visible = true,
    .named = true,
  },
  [sym_use_declaration] = {
    .visible = true,
    .named = true,
  },
  [sym_navigate_call] = {
    .visible = true,
    .named = true,
  },
  [sym_assignment] = {
    .visible = true,
    .named = true,
  },
  [sym_expression_statement] = {
    .visible = true,
    .named = true,
  },
  [sym_ui_element] = {
    .visible = true,
    .named = true,
  },
  [sym_component_name] = {
    .visible = true,
    .named = true,
  },
  [sym_sub_component] = {
    .visible = true,
    .named = true,
  },
  [sym_if_statement] = {
    .visible = true,
    .named = true,
  },
  [sym_for_statement] = {
    .visible = true,
    .named = true,
  },
  [sym_show_statement] = {
    .visible = true,
    .named = true,
  },
  [sym_animate_clause] = {
    .visible = true,
    .named = true,
  },
  [sym_fetch_statement] = {
    .visible = true,
    .named = true,
  },
  [sym_fetch_block] = {
    .visible = true,
    .named = true,
  },
  [sym_loading_block] = {
    .visible = true,
    .named = true,
  },
  [sym_error_block] = {
    .visible = true,
    .named = true,
  },
  [sym_success_block] = {
    .visible = true,
    .named = true,
  },
  [sym_event_handler] = {
    .visible = true,
    .named = true,
  },
  [sym_style_block] = {
    .visible = true,
    .named = true,
  },
  [sym_style_property] = {
    .visible = true,
    .named = true,
  },
  [sym_property_name] = {
    .visible = true,
    .named = true,
  },
  [sym_media_query] = {
    .visible = true,
    .named = true,
  },
  [sym_transition_block] = {
    .visible = true,
    .named = true,
  },
  [sym_transition_property] = {
    .visible = true,
    .named = true,
  },
  [sym__expression] = {
    .visible = false,
    .named = true,
  },
  [sym_binary_expression] = {
    .visible = true,
    .named = true,
  },
  [sym_unary_expression] = {
    .visible = true,
    .named = true,
  },
  [sym_member_expression] = {
    .visible = true,
    .named = true,
  },
  [sym_call_expression] = {
    .visible = true,
    .named = true,
  },
  [sym_t_call] = {
    .visible = true,
    .named = true,
  },
  [sym_string] = {
    .visible = true,
    .named = true,
  },
  [sym_interpolation] = {
    .visible = true,
    .named = true,
  },
  [sym_boolean] = {
    .visible = true,
    .named = true,
  },
  [sym_array] = {
    .visible = true,
    .named = true,
  },
  [sym_object] = {
    .visible = true,
    .named = true,
  },
  [sym_object_entry] = {
    .visible = true,
    .named = true,
  },
  [sym_comment] = {
    .visible = true,
    .named = true,
  },
  [aux_sym_source_file_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_parameter_list_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_argument_list_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_block_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_fetch_block_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_style_block_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_property_name_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_media_query_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_transition_block_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_string_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_array_repeat1] = {
    .visible = false,
    .named = false,
  },
  [aux_sym_object_repeat1] = {
    .visible = false,
    .named = false,
  },
};

enum ts_field_identifiers {
  field_child = 1,
  field_parent = 2,
};

static const char * const ts_field_names[] = {
  [0] = NULL,
  [field_child] = "child",
  [field_parent] = "parent",
};

static const TSFieldMapSlice ts_field_map_slices[PRODUCTION_ID_COUNT] = {
  [1] = {.index = 0, .length = 2},
};

static const TSFieldMapEntry ts_field_map_entries[] = {
  [0] =
    {field_child, 2},
    {field_parent, 0},
};

static const TSSymbol ts_alias_sequences[PRODUCTION_ID_COUNT][MAX_ALIAS_SEQUENCE_LENGTH] = {
  [0] = {0},
};

static const uint16_t ts_non_terminal_alias_map[] = {
  0,
};

static const TSStateId ts_primary_state_ids[STATE_COUNT] = {
  [0] = 0,
  [1] = 1,
  [2] = 2,
  [3] = 3,
  [4] = 4,
  [5] = 4,
  [6] = 2,
  [7] = 7,
  [8] = 8,
  [9] = 9,
  [10] = 8,
  [11] = 7,
  [12] = 7,
  [13] = 8,
  [14] = 7,
  [15] = 8,
  [16] = 7,
  [17] = 8,
  [18] = 18,
  [19] = 19,
  [20] = 20,
  [21] = 21,
  [22] = 22,
  [23] = 23,
  [24] = 24,
  [25] = 25,
  [26] = 26,
  [27] = 27,
  [28] = 28,
  [29] = 29,
  [30] = 30,
  [31] = 31,
  [32] = 32,
  [33] = 33,
  [34] = 34,
  [35] = 35,
  [36] = 36,
  [37] = 37,
  [38] = 38,
  [39] = 20,
  [40] = 32,
  [41] = 19,
  [42] = 42,
  [43] = 35,
  [44] = 44,
  [45] = 21,
  [46] = 22,
  [47] = 23,
  [48] = 24,
  [49] = 25,
  [50] = 26,
  [51] = 27,
  [52] = 28,
  [53] = 29,
  [54] = 30,
  [55] = 31,
  [56] = 33,
  [57] = 42,
  [58] = 44,
  [59] = 18,
  [60] = 38,
  [61] = 36,
  [62] = 37,
  [63] = 34,
  [64] = 64,
  [65] = 65,
  [66] = 66,
  [67] = 67,
  [68] = 68,
  [69] = 69,
  [70] = 70,
  [71] = 71,
  [72] = 72,
  [73] = 73,
  [74] = 74,
  [75] = 75,
  [76] = 76,
  [77] = 77,
  [78] = 78,
  [79] = 79,
  [80] = 80,
  [81] = 81,
  [82] = 82,
  [83] = 83,
  [84] = 84,
  [85] = 85,
  [86] = 86,
  [87] = 87,
  [88] = 88,
  [89] = 89,
  [90] = 90,
  [91] = 91,
  [92] = 92,
  [93] = 93,
  [94] = 94,
  [95] = 95,
  [96] = 96,
  [97] = 97,
  [98] = 98,
  [99] = 99,
  [100] = 100,
  [101] = 70,
  [102] = 65,
  [103] = 103,
  [104] = 104,
  [105] = 105,
  [106] = 65,
  [107] = 69,
  [108] = 64,
  [109] = 70,
  [110] = 67,
  [111] = 68,
  [112] = 66,
  [113] = 113,
  [114] = 114,
  [115] = 98,
  [116] = 116,
  [117] = 79,
  [118] = 81,
  [119] = 84,
  [120] = 76,
  [121] = 99,
  [122] = 86,
  [123] = 123,
  [124] = 82,
  [125] = 88,
  [126] = 89,
  [127] = 90,
  [128] = 128,
  [129] = 77,
  [130] = 91,
  [131] = 95,
  [132] = 93,
  [133] = 94,
  [134] = 83,
  [135] = 100,
  [136] = 97,
  [137] = 70,
  [138] = 103,
  [139] = 65,
  [140] = 116,
  [141] = 96,
  [142] = 75,
  [143] = 87,
  [144] = 72,
  [145] = 116,
  [146] = 73,
  [147] = 74,
  [148] = 78,
  [149] = 128,
  [150] = 92,
  [151] = 151,
  [152] = 151,
  [153] = 151,
  [154] = 154,
  [155] = 155,
  [156] = 155,
  [157] = 155,
  [158] = 158,
  [159] = 159,
  [160] = 160,
  [161] = 161,
  [162] = 162,
  [163] = 163,
  [164] = 164,
  [165] = 165,
  [166] = 166,
  [167] = 165,
  [168] = 168,
  [169] = 169,
  [170] = 166,
  [171] = 171,
  [172] = 172,
  [173] = 173,
  [174] = 174,
  [175] = 169,
  [176] = 176,
  [177] = 177,
  [178] = 166,
  [179] = 179,
  [180] = 180,
  [181] = 181,
  [182] = 174,
  [183] = 183,
  [184] = 174,
  [185] = 185,
  [186] = 158,
  [187] = 181,
  [188] = 183,
  [189] = 163,
  [190] = 176,
  [191] = 177,
  [192] = 161,
  [193] = 158,
  [194] = 168,
  [195] = 25,
  [196] = 19,
  [197] = 42,
  [198] = 44,
  [199] = 199,
  [200] = 21,
  [201] = 22,
  [202] = 23,
  [203] = 24,
  [204] = 204,
  [205] = 26,
  [206] = 27,
  [207] = 28,
  [208] = 208,
  [209] = 29,
  [210] = 30,
  [211] = 31,
  [212] = 33,
  [213] = 213,
  [214] = 214,
  [215] = 215,
  [216] = 216,
  [217] = 199,
  [218] = 204,
  [219] = 214,
  [220] = 199,
  [221] = 215,
  [222] = 216,
  [223] = 20,
  [224] = 32,
  [225] = 35,
  [226] = 226,
  [227] = 227,
  [228] = 228,
  [229] = 229,
  [230] = 230,
  [231] = 231,
  [232] = 232,
  [233] = 228,
  [234] = 234,
  [235] = 234,
  [236] = 236,
  [237] = 237,
  [238] = 238,
  [239] = 239,
  [240] = 231,
  [241] = 241,
  [242] = 242,
  [243] = 242,
  [244] = 244,
  [245] = 241,
  [246] = 242,
  [247] = 247,
  [248] = 248,
  [249] = 249,
  [250] = 250,
  [251] = 251,
  [252] = 250,
  [253] = 251,
  [254] = 254,
  [255] = 255,
  [256] = 255,
  [257] = 257,
  [258] = 255,
  [259] = 254,
  [260] = 254,
  [261] = 261,
  [262] = 262,
  [263] = 263,
  [264] = 264,
  [265] = 265,
  [266] = 266,
  [267] = 267,
  [268] = 70,
  [269] = 269,
  [270] = 65,
  [271] = 271,
  [272] = 272,
  [273] = 273,
  [274] = 274,
  [275] = 275,
  [276] = 276,
  [277] = 277,
  [278] = 278,
  [279] = 279,
  [280] = 280,
  [281] = 281,
  [282] = 282,
  [283] = 283,
  [284] = 281,
  [285] = 283,
  [286] = 277,
  [287] = 282,
  [288] = 288,
  [289] = 280,
  [290] = 290,
  [291] = 291,
  [292] = 292,
  [293] = 293,
  [294] = 294,
  [295] = 295,
  [296] = 296,
  [297] = 297,
  [298] = 298,
  [299] = 299,
  [300] = 300,
  [301] = 301,
  [302] = 302,
  [303] = 303,
  [304] = 294,
  [305] = 305,
  [306] = 301,
  [307] = 307,
  [308] = 290,
  [309] = 309,
  [310] = 310,
  [311] = 297,
  [312] = 301,
  [313] = 294,
  [314] = 314,
  [315] = 290,
  [316] = 316,
  [317] = 297,
  [318] = 318,
  [319] = 319,
  [320] = 320,
  [321] = 321,
  [322] = 322,
  [323] = 323,
  [324] = 324,
  [325] = 325,
  [326] = 326,
  [327] = 320,
  [328] = 328,
  [329] = 329,
  [330] = 330,
  [331] = 331,
  [332] = 332,
  [333] = 333,
  [334] = 334,
  [335] = 323,
  [336] = 336,
  [337] = 337,
  [338] = 330,
  [339] = 339,
  [340] = 340,
  [341] = 341,
  [342] = 326,
  [343] = 343,
  [344] = 344,
  [345] = 332,
  [346] = 346,
  [347] = 347,
  [348] = 337,
  [349] = 341,
  [350] = 350,
  [351] = 323,
  [352] = 352,
  [353] = 353,
  [354] = 354,
  [355] = 355,
  [356] = 356,
  [357] = 339,
  [358] = 347,
  [359] = 354,
  [360] = 360,
  [361] = 343,
  [362] = 362,
  [363] = 362,
  [364] = 324,
  [365] = 347,
  [366] = 366,
  [367] = 331,
  [368] = 328,
  [369] = 369,
  [370] = 370,
  [371] = 371,
  [372] = 372,
  [373] = 373,
  [374] = 374,
  [375] = 375,
  [376] = 376,
  [377] = 377,
  [378] = 378,
  [379] = 371,
  [380] = 380,
  [381] = 381,
  [382] = 382,
  [383] = 383,
  [384] = 384,
  [385] = 385,
  [386] = 386,
  [387] = 387,
  [388] = 388,
  [389] = 389,
  [390] = 390,
  [391] = 391,
  [392] = 392,
  [393] = 393,
  [394] = 394,
  [395] = 370,
  [396] = 396,
  [397] = 376,
  [398] = 398,
  [399] = 385,
  [400] = 400,
  [401] = 371,
  [402] = 380,
  [403] = 403,
  [404] = 404,
  [405] = 405,
  [406] = 400,
  [407] = 405,
  [408] = 389,
  [409] = 409,
  [410] = 410,
  [411] = 369,
  [412] = 387,
  [413] = 413,
  [414] = 380,
  [415] = 415,
  [416] = 409,
  [417] = 417,
  [418] = 372,
  [419] = 419,
  [420] = 376,
  [421] = 382,
  [422] = 422,
  [423] = 423,
  [424] = 424,
  [425] = 425,
  [426] = 426,
  [427] = 413,
  [428] = 428,
  [429] = 429,
  [430] = 430,
  [431] = 431,
  [432] = 377,
  [433] = 388,
  [434] = 431,
  [435] = 375,
  [436] = 390,
  [437] = 377,
  [438] = 417,
  [439] = 378,
  [440] = 419,
  [441] = 423,
  [442] = 442,
  [443] = 443,
  [444] = 444,
};

static bool ts_lex(TSLexer *lexer, TSStateId state) {
  START_LEXER();
  eof = lexer->eof(lexer);
  switch (state) {
    case 0:
      if (eof) ADVANCE(181);
      ADVANCE_MAP(
        '!', 256,
        '"', 260,
        '%', 246,
        '&', 9,
        '(', 183,
        ')', 184,
        '*', 244,
        '+', 243,
        ',', 188,
        '-', 236,
        '.', 209,
        '/', 245,
        ':', 190,
        '<', 249,
        '=', 191,
        '>', 250,
        '?', 189,
        '@', 100,
        'A', 133,
        'B', 121,
        'C', 117,
        'L', 79,
        'M', 23,
        'N', 167,
        'P', 18,
        'S', 154,
        '[', 273,
        '\\', 180,
        ']', 274,
        'a', 35,
        'd', 44,
        'e', 68,
        'f', 19,
        'i', 69,
        'l', 119,
        'n', 20,
        'o', 104,
        's', 77,
        't', 257,
        'u', 146,
        '{', 195,
        '|', 178,
        '}', 196,
      );
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') SKIP(0);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(265);
      END_STATE();
    case 1:
      ADVANCE_MAP(
        '!', 256,
        '"', 260,
        '%', 246,
        '&', 9,
        '(', 183,
        ')', 184,
        '*', 244,
        '+', 243,
        ',', 188,
        '-', 236,
        '.', 209,
        '/', 245,
        '<', 249,
        '=', 191,
        '>', 250,
        'A', 489,
        'B', 282,
        'C', 289,
        'D', 291,
        'F', 450,
        'G', 592,
        'H', 396,
        'I', 336,
        'L', 451,
        'M', 395,
        'N', 283,
        'O', 571,
        'P', 284,
        'R', 295,
        'S', 378,
        'T', 285,
        'V', 472,
        '[', 273,
        ']', 274,
        'a', 351,
        'd', 414,
        'e', 430,
        'f', 292,
        'i', 426,
        'n', 294,
        'o', 516,
        's', 448,
        't', 258,
        'u', 604,
        '{', 195,
        '|', 178,
        '}', 196,
      );
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') SKIP(1);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(265);
      if (('E' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('b' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 2:
      ADVANCE_MAP(
        '!', 256,
        '"', 260,
        '%', 246,
        '&', 9,
        '(', 183,
        '*', 244,
        '+', 243,
        '-', 236,
        '.', 209,
        '/', 245,
        '<', 249,
        '=', 191,
        '>', 250,
        'A', 489,
        'B', 282,
        'C', 289,
        'D', 291,
        'F', 450,
        'G', 592,
        'H', 396,
        'I', 336,
        'L', 451,
        'M', 395,
        'N', 283,
        'O', 571,
        'P', 284,
        'R', 295,
        'S', 378,
        'T', 285,
        'V', 472,
        '[', 273,
        'a', 351,
        'd', 414,
        'e', 429,
        'f', 292,
        'i', 426,
        'l', 563,
        'n', 294,
        'o', 516,
        's', 447,
        't', 258,
        'u', 604,
        '{', 195,
        '|', 178,
        '}', 196,
      );
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') SKIP(2);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(265);
      if (('E' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('b' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 3:
      ADVANCE_MAP(
        '!', 255,
        '"', 260,
        '(', 183,
        ')', 184,
        '/', 10,
        '[', 273,
        'b', 502,
        'c', 411,
        'd', 313,
        'e', 498,
        'f', 288,
        'h', 425,
        'i', 526,
        'l', 311,
        'm', 359,
        'n', 640,
        'o', 596,
        'p', 475,
        'r', 458,
        's', 343,
        't', 259,
        'u', 541,
        'w', 312,
        'x', 488,
        '{', 195,
      );
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') SKIP(3);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(265);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 4:
      ADVANCE_MAP(
        '!', 255,
        '"', 260,
        '(', 183,
        '/', 10,
        'A', 489,
        'B', 282,
        'C', 289,
        'D', 291,
        'F', 450,
        'G', 592,
        'H', 396,
        'I', 336,
        'L', 451,
        'M', 395,
        'N', 283,
        'O', 571,
        'P', 284,
        'R', 295,
        'S', 378,
        'T', 285,
        'V', 472,
        '[', 273,
        'a', 351,
        'd', 414,
        'e', 427,
        'f', 292,
        'i', 426,
        'l', 563,
        'n', 294,
        'o', 516,
        's', 447,
        't', 258,
        'u', 604,
        '{', 195,
        '}', 196,
      );
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') SKIP(4);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(265);
      if (('E' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('b' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 5:
      ADVANCE_MAP(
        '!', 255,
        '"', 260,
        '(', 183,
        '/', 10,
        'A', 489,
        'B', 282,
        'C', 289,
        'D', 291,
        'F', 450,
        'G', 592,
        'H', 396,
        'I', 336,
        'L', 451,
        'M', 395,
        'N', 283,
        'O', 571,
        'P', 284,
        'R', 295,
        'S', 378,
        'T', 285,
        'V', 472,
        '[', 273,
        'a', 351,
        'd', 414,
        'e', 428,
        'f', 292,
        'i', 426,
        'n', 294,
        'o', 516,
        's', 448,
        't', 258,
        'u', 604,
        '{', 195,
        '}', 196,
      );
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') SKIP(5);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(265);
      if (('E' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('b' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 6:
      ADVANCE_MAP(
        '!', 255,
        '"', 260,
        '(', 183,
        '/', 10,
        '[', 273,
        ']', 274,
        'f', 293,
        'n', 640,
        't', 259,
        '{', 195,
      );
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') SKIP(6);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(265);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 7:
      ADVANCE_MAP(
        '!', 16,
        '%', 246,
        '&', 9,
        '(', 183,
        ')', 184,
        '*', 244,
        '+', 243,
        '-', 236,
        '.', 209,
        '/', 245,
        '<', 249,
        '=', 17,
        '>', 250,
        '@', 100,
        '|', 178,
        '}', 196,
      );
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') SKIP(7);
      if (('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 8:
      if (lookahead == '"') ADVANCE(260);
      if (lookahead == '/') ADVANCE(261);
      if (lookahead == '\\') ADVANCE(180);
      if (lookahead == '{') ADVANCE(195);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') ADVANCE(262);
      if (lookahead != 0) ADVANCE(263);
      END_STATE();
    case 9:
      if (lookahead == '&') ADVANCE(253);
      END_STATE();
    case 10:
      if (lookahead == '*') ADVANCE(677);
      if (lookahead == '/') ADVANCE(669);
      END_STATE();
    case 11:
      if (lookahead == '*') ADVANCE(682);
      if (lookahead == '/') ADVANCE(13);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') ADVANCE(11);
      if (lookahead != 0) ADVANCE(12);
      END_STATE();
    case 12:
      if (lookahead == '*') ADVANCE(682);
      if (lookahead != 0) ADVANCE(12);
      END_STATE();
    case 13:
      if (lookahead == '*') ADVANCE(678);
      if (lookahead == '/') ADVANCE(670);
      if (lookahead != 0) ADVANCE(12);
      END_STATE();
    case 14:
      if (lookahead == '/') ADVANCE(238);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') ADVANCE(239);
      if (lookahead != 0 &&
          lookahead != '{') ADVANCE(240);
      END_STATE();
    case 15:
      if (lookahead == ':') ADVANCE(31);
      END_STATE();
    case 16:
      if (lookahead == '=') ADVANCE(248);
      END_STATE();
    case 17:
      if (lookahead == '=') ADVANCE(247);
      END_STATE();
    case 18:
      if (lookahead == 'a') ADVANCE(73);
      END_STATE();
    case 19:
      if (lookahead == 'a') ADVANCE(97);
      if (lookahead == 'e') ADVANCE(159);
      if (lookahead == 'o') ADVANCE(137);
      if (lookahead == 'r') ADVANCE(124);
      END_STATE();
    case 20:
      if (lookahead == 'a') ADVANCE(172);
      if (lookahead == 'u') ADVANCE(95);
      END_STATE();
    case 21:
      if (lookahead == 'a') ADVANCE(237);
      END_STATE();
    case 22:
      if (lookahead == 'a') ADVANCE(108);
      if (lookahead == 'u') ADVANCE(49);
      END_STATE();
    case 23:
      if (lookahead == 'a') ADVANCE(131);
      END_STATE();
    case 24:
      if (lookahead == 'a') ADVANCE(161);
      if (lookahead == 'y') ADVANCE(96);
      END_STATE();
    case 25:
      if (lookahead == 'a') ADVANCE(115);
      END_STATE();
    case 26:
      if (lookahead == 'a') ADVANCE(43);
      END_STATE();
    case 27:
      if (lookahead == 'a') ADVANCE(174);
      END_STATE();
    case 28:
      if (lookahead == 'a') ADVANCE(162);
      END_STATE();
    case 29:
      if (lookahead == 'a') ADVANCE(163);
      END_STATE();
    case 30:
      if (lookahead == 'b') ADVANCE(103);
      END_STATE();
    case 31:
      if (lookahead == 'b') ADVANCE(94);
      if (lookahead == 'c') ADVANCE(78);
      if (lookahead == 'f') ADVANCE(127);
      if (lookahead == 'i') ADVANCE(111);
      if (lookahead == 'k') ADVANCE(52);
      if (lookahead == 'm') ADVANCE(122);
      if (lookahead == 's') ADVANCE(166);
      END_STATE();
    case 32:
      if (lookahead == 'b') ADVANCE(62);
      END_STATE();
    case 33:
      if (lookahead == 'c') ADVANCE(76);
      END_STATE();
    case 34:
      if (lookahead == 'c') ADVANCE(91);
      END_STATE();
    case 35:
      if (lookahead == 'c') ADVANCE(160);
      if (lookahead == 'n') ADVANCE(83);
      END_STATE();
    case 36:
      if (lookahead == 'c') ADVANCE(39);
      END_STATE();
    case 37:
      if (lookahead == 'c') ADVANCE(168);
      END_STATE();
    case 38:
      if (lookahead == 'c') ADVANCE(156);
      END_STATE();
    case 39:
      if (lookahead == 'c') ADVANCE(60);
      END_STATE();
    case 40:
      if (lookahead == 'd') ADVANCE(199);
      END_STATE();
    case 41:
      if (lookahead == 'd') ADVANCE(84);
      END_STATE();
    case 42:
      if (lookahead == 'd') ADVANCE(118);
      if (lookahead == 'p') ADVANCE(143);
      if (lookahead == 'u') ADVANCE(132);
      END_STATE();
    case 43:
      if (lookahead == 'd') ADVANCE(89);
      END_STATE();
    case 44:
      if (lookahead == 'e') ADVANCE(140);
      END_STATE();
    case 45:
      if (lookahead == 'e') ADVANCE(41);
      END_STATE();
    case 46:
      if (lookahead == 'e') ADVANCE(205);
      END_STATE();
    case 47:
      if (lookahead == 'e') ADVANCE(182);
      END_STATE();
    case 48:
      if (lookahead == 'e') ADVANCE(216);
      END_STATE();
    case 49:
      if (lookahead == 'e') ADVANCE(267);
      END_STATE();
    case 50:
      if (lookahead == 'e') ADVANCE(186);
      END_STATE();
    case 51:
      if (lookahead == 'e') ADVANCE(269);
      END_STATE();
    case 52:
      if (lookahead == 'e') ADVANCE(177);
      END_STATE();
    case 53:
      if (lookahead == 'e') ADVANCE(197);
      END_STATE();
    case 54:
      if (lookahead == 'e') ADVANCE(234);
      END_STATE();
    case 55:
      if (lookahead == 'e') ADVANCE(223);
      END_STATE();
    case 56:
      if (lookahead == 'e') ADVANCE(233);
      END_STATE();
    case 57:
      if (lookahead == 'e') ADVANCE(207);
      END_STATE();
    case 58:
      if (lookahead == 'e') ADVANCE(66);
      END_STATE();
    case 59:
      if (lookahead == 'e') ADVANCE(40);
      END_STATE();
    case 60:
      if (lookahead == 'e') ADVANCE(148);
      END_STATE();
    case 61:
      if (lookahead == 'e') ADVANCE(38);
      END_STATE();
    case 62:
      if (lookahead == 'e') ADVANCE(136);
      END_STATE();
    case 63:
      if (lookahead == 'e') ADVANCE(27);
      END_STATE();
    case 64:
      if (lookahead == 'e') ADVANCE(139);
      END_STATE();
    case 65:
      if (lookahead == 'e') ADVANCE(112);
      END_STATE();
    case 66:
      if (lookahead == 'e') ADVANCE(113);
      if (lookahead == 'l') ADVANCE(63);
      END_STATE();
    case 67:
      if (lookahead == 'e') ADVANCE(150);
      END_STATE();
    case 68:
      if (lookahead == 'f') ADVANCE(70);
      if (lookahead == 'l') ADVANCE(149);
      if (lookahead == 'r') ADVANCE(141);
      END_STATE();
    case 69:
      if (lookahead == 'f') ADVANCE(214);
      if (lookahead == 'n') ADVANCE(220);
      END_STATE();
    case 70:
      if (lookahead == 'f') ADVANCE(61);
      END_STATE();
    case 71:
      if (lookahead == 'g') ADVANCE(194);
      END_STATE();
    case 72:
      if (lookahead == 'g') ADVANCE(227);
      END_STATE();
    case 73:
      if (lookahead == 'g') ADVANCE(47);
      END_STATE();
    case 74:
      if (lookahead == 'g') ADVANCE(56);
      END_STATE();
    case 75:
      if (lookahead == 'g') ADVANCE(29);
      END_STATE();
    case 76:
      if (lookahead == 'h') ADVANCE(224);
      END_STATE();
    case 77:
      if (lookahead == 'h') ADVANCE(116);
      if (lookahead == 't') ADVANCE(24);
      if (lookahead == 'u') ADVANCE(36);
      END_STATE();
    case 78:
      if (lookahead == 'h') ADVANCE(25);
      if (lookahead == 'l') ADVANCE(85);
      END_STATE();
    case 79:
      if (lookahead == 'i') ADVANCE(147);
      END_STATE();
    case 80:
      if (lookahead == 'i') ADVANCE(75);
      END_STATE();
    case 81:
      if (lookahead == 'i') ADVANCE(173);
      END_STATE();
    case 82:
      if (lookahead == 'i') ADVANCE(109);
      END_STATE();
    case 83:
      if (lookahead == 'i') ADVANCE(102);
      END_STATE();
    case 84:
      if (lookahead == 'i') ADVANCE(21);
      END_STATE();
    case 85:
      if (lookahead == 'i') ADVANCE(34);
      END_STATE();
    case 86:
      if (lookahead == 'i') ADVANCE(157);
      END_STATE();
    case 87:
      if (lookahead == 'i') ADVANCE(128);
      END_STATE();
    case 88:
      if (lookahead == 'i') ADVANCE(129);
      END_STATE();
    case 89:
      if (lookahead == 'i') ADVANCE(110);
      END_STATE();
    case 90:
      if (lookahead == 'i') ADVANCE(164);
      END_STATE();
    case 91:
      if (lookahead == 'k') ADVANCE(233);
      END_STATE();
    case 92:
      if (lookahead == 'l') ADVANCE(194);
      END_STATE();
    case 93:
      if (lookahead == 'l') ADVANCE(271);
      END_STATE();
    case 94:
      if (lookahead == 'l') ADVANCE(170);
      END_STATE();
    case 95:
      if (lookahead == 'l') ADVANCE(93);
      END_STATE();
    case 96:
      if (lookahead == 'l') ADVANCE(54);
      END_STATE();
    case 97:
      if (lookahead == 'l') ADVANCE(152);
      END_STATE();
    case 98:
      if (lookahead == 'm') ADVANCE(32);
      END_STATE();
    case 99:
      if (lookahead == 'm') ADVANCE(226);
      END_STATE();
    case 100:
      if (lookahead == 'm') ADVANCE(45);
      END_STATE();
    case 101:
      if (lookahead == 'm') ADVANCE(135);
      END_STATE();
    case 102:
      if (lookahead == 'm') ADVANCE(28);
      END_STATE();
    case 103:
      if (lookahead == 'm') ADVANCE(86);
      END_STATE();
    case 104:
      if (lookahead == 'n') ADVANCE(15);
      END_STATE();
    case 105:
      if (lookahead == 'n') ADVANCE(203);
      END_STATE();
    case 106:
      if (lookahead == 'n') ADVANCE(233);
      END_STATE();
    case 107:
      if (lookahead == 'n') ADVANCE(241);
      END_STATE();
    case 108:
      if (lookahead == 'n') ADVANCE(151);
      END_STATE();
    case 109:
      if (lookahead == 'n') ADVANCE(71);
      END_STATE();
    case 110:
      if (lookahead == 'n') ADVANCE(72);
      END_STATE();
    case 111:
      if (lookahead == 'n') ADVANCE(134);
      END_STATE();
    case 112:
      if (lookahead == 'n') ADVANCE(158);
      END_STATE();
    case 113:
      if (lookahead == 'n') ADVANCE(165);
      END_STATE();
    case 114:
      if (lookahead == 'n') ADVANCE(65);
      END_STATE();
    case 115:
      if (lookahead == 'n') ADVANCE(74);
      END_STATE();
    case 116:
      if (lookahead == 'o') ADVANCE(175);
      END_STATE();
    case 117:
      if (lookahead == 'o') ADVANCE(101);
      END_STATE();
    case 118:
      if (lookahead == 'o') ADVANCE(176);
      END_STATE();
    case 119:
      if (lookahead == 'o') ADVANCE(26);
      END_STATE();
    case 120:
      if (lookahead == 'o') ADVANCE(92);
      END_STATE();
    case 121:
      if (lookahead == 'o') ADVANCE(120);
      END_STATE();
    case 122:
      if (lookahead == 'o') ADVANCE(171);
      END_STATE();
    case 123:
      if (lookahead == 'o') ADVANCE(142);
      if (lookahead == 'r') ADVANCE(82);
      END_STATE();
    case 124:
      if (lookahead == 'o') ADVANCE(99);
      END_STATE();
    case 125:
      if (lookahead == 'o') ADVANCE(114);
      END_STATE();
    case 126:
      if (lookahead == 'o') ADVANCE(138);
      END_STATE();
    case 127:
      if (lookahead == 'o') ADVANCE(37);
      END_STATE();
    case 128:
      if (lookahead == 'o') ADVANCE(105);
      END_STATE();
    case 129:
      if (lookahead == 'o') ADVANCE(107);
      END_STATE();
    case 130:
      if (lookahead == 'p') ADVANCE(187);
      END_STATE();
    case 131:
      if (lookahead == 'p') ADVANCE(194);
      END_STATE();
    case 132:
      if (lookahead == 'p') ADVANCE(233);
      END_STATE();
    case 133:
      if (lookahead == 'p') ADVANCE(130);
      END_STATE();
    case 134:
      if (lookahead == 'p') ADVANCE(169);
      END_STATE();
    case 135:
      if (lookahead == 'p') ADVANCE(125);
      END_STATE();
    case 136:
      if (lookahead == 'r') ADVANCE(194);
      END_STATE();
    case 137:
      if (lookahead == 'r') ADVANCE(218);
      END_STATE();
    case 138:
      if (lookahead == 'r') ADVANCE(229);
      END_STATE();
    case 139:
      if (lookahead == 'r') ADVANCE(233);
      END_STATE();
    case 140:
      if (lookahead == 'r') ADVANCE(81);
      END_STATE();
    case 141:
      if (lookahead == 'r') ADVANCE(126);
      END_STATE();
    case 142:
      if (lookahead == 'r') ADVANCE(50);
      END_STATE();
    case 143:
      if (lookahead == 'r') ADVANCE(67);
      END_STATE();
    case 144:
      if (lookahead == 's') ADVANCE(233);
      END_STATE();
    case 145:
      if (lookahead == 's') ADVANCE(231);
      END_STATE();
    case 146:
      if (lookahead == 's') ADVANCE(46);
      END_STATE();
    case 147:
      if (lookahead == 's') ADVANCE(155);
      END_STATE();
    case 148:
      if (lookahead == 's') ADVANCE(145);
      END_STATE();
    case 149:
      if (lookahead == 's') ADVANCE(48);
      END_STATE();
    case 150:
      if (lookahead == 's') ADVANCE(144);
      END_STATE();
    case 151:
      if (lookahead == 's') ADVANCE(90);
      END_STATE();
    case 152:
      if (lookahead == 's') ADVANCE(51);
      END_STATE();
    case 153:
      if (lookahead == 's') ADVANCE(58);
      END_STATE();
    case 154:
      if (lookahead == 't') ADVANCE(123);
      END_STATE();
    case 155:
      if (lookahead == 't') ADVANCE(194);
      END_STATE();
    case 156:
      if (lookahead == 't') ADVANCE(201);
      END_STATE();
    case 157:
      if (lookahead == 't') ADVANCE(233);
      END_STATE();
    case 158:
      if (lookahead == 't') ADVANCE(185);
      END_STATE();
    case 159:
      if (lookahead == 't') ADVANCE(33);
      END_STATE();
    case 160:
      if (lookahead == 't') ADVANCE(87);
      END_STATE();
    case 161:
      if (lookahead == 't') ADVANCE(53);
      END_STATE();
    case 162:
      if (lookahead == 't') ADVANCE(55);
      END_STATE();
    case 163:
      if (lookahead == 't') ADVANCE(57);
      END_STATE();
    case 164:
      if (lookahead == 't') ADVANCE(88);
      END_STATE();
    case 165:
      if (lookahead == 't') ADVANCE(64);
      END_STATE();
    case 166:
      if (lookahead == 'u') ADVANCE(30);
      END_STATE();
    case 167:
      if (lookahead == 'u') ADVANCE(98);
      END_STATE();
    case 168:
      if (lookahead == 'u') ADVANCE(144);
      END_STATE();
    case 169:
      if (lookahead == 'u') ADVANCE(157);
      END_STATE();
    case 170:
      if (lookahead == 'u') ADVANCE(139);
      END_STATE();
    case 171:
      if (lookahead == 'u') ADVANCE(153);
      END_STATE();
    case 172:
      if (lookahead == 'v') ADVANCE(80);
      END_STATE();
    case 173:
      if (lookahead == 'v') ADVANCE(59);
      END_STATE();
    case 174:
      if (lookahead == 'v') ADVANCE(56);
      END_STATE();
    case 175:
      if (lookahead == 'w') ADVANCE(221);
      END_STATE();
    case 176:
      if (lookahead == 'w') ADVANCE(106);
      END_STATE();
    case 177:
      if (lookahead == 'y') ADVANCE(42);
      END_STATE();
    case 178:
      if (lookahead == '|') ADVANCE(254);
      END_STATE();
    case 179:
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(266);
      END_STATE();
    case 180:
      if (lookahead != 0 &&
          lookahead != '\n') ADVANCE(264);
      END_STATE();
    case 181:
      ACCEPT_TOKEN(ts_builtin_sym_end);
      END_STATE();
    case 182:
      ACCEPT_TOKEN(anon_sym_Page);
      END_STATE();
    case 183:
      ACCEPT_TOKEN(anon_sym_LPAREN);
      END_STATE();
    case 184:
      ACCEPT_TOKEN(anon_sym_RPAREN);
      END_STATE();
    case 185:
      ACCEPT_TOKEN(anon_sym_Component);
      END_STATE();
    case 186:
      ACCEPT_TOKEN(anon_sym_Store);
      END_STATE();
    case 187:
      ACCEPT_TOKEN(anon_sym_App);
      END_STATE();
    case 188:
      ACCEPT_TOKEN(anon_sym_COMMA);
      END_STATE();
    case 189:
      ACCEPT_TOKEN(anon_sym_QMARK);
      END_STATE();
    case 190:
      ACCEPT_TOKEN(anon_sym_COLON);
      END_STATE();
    case 191:
      ACCEPT_TOKEN(anon_sym_EQ);
      if (lookahead == '=') ADVANCE(247);
      END_STATE();
    case 192:
      ACCEPT_TOKEN(sym_modifier);
      if (lookahead == 'a') ADVANCE(492);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('b' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 193:
      ACCEPT_TOKEN(sym_modifier);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 194:
      ACCEPT_TOKEN(sym_type_annotation);
      END_STATE();
    case 195:
      ACCEPT_TOKEN(anon_sym_LBRACE);
      END_STATE();
    case 196:
      ACCEPT_TOKEN(anon_sym_RBRACE);
      END_STATE();
    case 197:
      ACCEPT_TOKEN(anon_sym_state);
      END_STATE();
    case 198:
      ACCEPT_TOKEN(anon_sym_state);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 199:
      ACCEPT_TOKEN(anon_sym_derived);
      END_STATE();
    case 200:
      ACCEPT_TOKEN(anon_sym_derived);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 201:
      ACCEPT_TOKEN(anon_sym_effect);
      END_STATE();
    case 202:
      ACCEPT_TOKEN(anon_sym_effect);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 203:
      ACCEPT_TOKEN(anon_sym_action);
      END_STATE();
    case 204:
      ACCEPT_TOKEN(anon_sym_action);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 205:
      ACCEPT_TOKEN(anon_sym_use);
      END_STATE();
    case 206:
      ACCEPT_TOKEN(anon_sym_use);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 207:
      ACCEPT_TOKEN(anon_sym_navigate);
      END_STATE();
    case 208:
      ACCEPT_TOKEN(anon_sym_navigate);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 209:
      ACCEPT_TOKEN(anon_sym_DOT);
      END_STATE();
    case 210:
      ACCEPT_TOKEN(sym_builtin_component);
      if (lookahead == 'B') ADVANCE(653);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 211:
      ACCEPT_TOKEN(sym_builtin_component);
      if (lookahead == 'G') ADVANCE(587);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 212:
      ACCEPT_TOKEN(sym_builtin_component);
      if (lookahead == 'r') ADVANCE(213);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 213:
      ACCEPT_TOKEN(sym_builtin_component);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 214:
      ACCEPT_TOKEN(anon_sym_if);
      END_STATE();
    case 215:
      ACCEPT_TOKEN(anon_sym_if);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 216:
      ACCEPT_TOKEN(anon_sym_else);
      END_STATE();
    case 217:
      ACCEPT_TOKEN(anon_sym_else);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 218:
      ACCEPT_TOKEN(anon_sym_for);
      END_STATE();
    case 219:
      ACCEPT_TOKEN(anon_sym_for);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 220:
      ACCEPT_TOKEN(anon_sym_in);
      END_STATE();
    case 221:
      ACCEPT_TOKEN(anon_sym_show);
      END_STATE();
    case 222:
      ACCEPT_TOKEN(anon_sym_show);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 223:
      ACCEPT_TOKEN(anon_sym_animate);
      END_STATE();
    case 224:
      ACCEPT_TOKEN(anon_sym_fetch);
      END_STATE();
    case 225:
      ACCEPT_TOKEN(anon_sym_fetch);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 226:
      ACCEPT_TOKEN(anon_sym_from);
      END_STATE();
    case 227:
      ACCEPT_TOKEN(anon_sym_loading);
      END_STATE();
    case 228:
      ACCEPT_TOKEN(anon_sym_loading);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 229:
      ACCEPT_TOKEN(anon_sym_error);
      END_STATE();
    case 230:
      ACCEPT_TOKEN(anon_sym_error);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 231:
      ACCEPT_TOKEN(anon_sym_success);
      END_STATE();
    case 232:
      ACCEPT_TOKEN(anon_sym_success);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 233:
      ACCEPT_TOKEN(sym_event_name);
      END_STATE();
    case 234:
      ACCEPT_TOKEN(anon_sym_style);
      END_STATE();
    case 235:
      ACCEPT_TOKEN(anon_sym_style);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 236:
      ACCEPT_TOKEN(anon_sym_DASH);
      END_STATE();
    case 237:
      ACCEPT_TOKEN(anon_sym_ATmedia);
      END_STATE();
    case 238:
      ACCEPT_TOKEN(aux_sym_media_query_token1);
      if (lookahead == '*') ADVANCE(681);
      if (lookahead == '/') ADVANCE(673);
      if (lookahead != 0 &&
          lookahead != '{') ADVANCE(240);
      END_STATE();
    case 239:
      ACCEPT_TOKEN(aux_sym_media_query_token1);
      if (lookahead == '/') ADVANCE(238);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') ADVANCE(239);
      if (lookahead != 0 &&
          lookahead != '{') ADVANCE(240);
      END_STATE();
    case 240:
      ACCEPT_TOKEN(aux_sym_media_query_token1);
      if (lookahead != 0 &&
          lookahead != '{') ADVANCE(240);
      END_STATE();
    case 241:
      ACCEPT_TOKEN(anon_sym_transition);
      END_STATE();
    case 242:
      ACCEPT_TOKEN(anon_sym_transition);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 243:
      ACCEPT_TOKEN(anon_sym_PLUS);
      END_STATE();
    case 244:
      ACCEPT_TOKEN(anon_sym_STAR);
      END_STATE();
    case 245:
      ACCEPT_TOKEN(anon_sym_SLASH);
      if (lookahead == '*') ADVANCE(677);
      if (lookahead == '/') ADVANCE(669);
      END_STATE();
    case 246:
      ACCEPT_TOKEN(anon_sym_PERCENT);
      END_STATE();
    case 247:
      ACCEPT_TOKEN(anon_sym_EQ_EQ);
      END_STATE();
    case 248:
      ACCEPT_TOKEN(anon_sym_BANG_EQ);
      END_STATE();
    case 249:
      ACCEPT_TOKEN(anon_sym_LT);
      if (lookahead == '=') ADVANCE(251);
      END_STATE();
    case 250:
      ACCEPT_TOKEN(anon_sym_GT);
      if (lookahead == '=') ADVANCE(252);
      END_STATE();
    case 251:
      ACCEPT_TOKEN(anon_sym_LT_EQ);
      END_STATE();
    case 252:
      ACCEPT_TOKEN(anon_sym_GT_EQ);
      END_STATE();
    case 253:
      ACCEPT_TOKEN(anon_sym_AMP_AMP);
      END_STATE();
    case 254:
      ACCEPT_TOKEN(anon_sym_PIPE_PIPE);
      END_STATE();
    case 255:
      ACCEPT_TOKEN(anon_sym_BANG);
      END_STATE();
    case 256:
      ACCEPT_TOKEN(anon_sym_BANG);
      if (lookahead == '=') ADVANCE(248);
      END_STATE();
    case 257:
      ACCEPT_TOKEN(anon_sym_t);
      if (lookahead == 'r') ADVANCE(22);
      END_STATE();
    case 258:
      ACCEPT_TOKEN(anon_sym_t);
      if (lookahead == 'r') ADVANCE(297);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 259:
      ACCEPT_TOKEN(anon_sym_t);
      if (lookahead == 'r') ADVANCE(647);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 260:
      ACCEPT_TOKEN(anon_sym_DQUOTE);
      END_STATE();
    case 261:
      ACCEPT_TOKEN(sym_string_content);
      if (lookahead == '*') ADVANCE(679);
      if (lookahead == '/') ADVANCE(671);
      if (lookahead != 0 &&
          lookahead != '"' &&
          lookahead != '\\' &&
          lookahead != '{') ADVANCE(263);
      END_STATE();
    case 262:
      ACCEPT_TOKEN(sym_string_content);
      if (lookahead == '/') ADVANCE(261);
      if (('\t' <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') ADVANCE(262);
      if (lookahead != 0 &&
          lookahead != '"' &&
          lookahead != '\\' &&
          lookahead != '{') ADVANCE(263);
      END_STATE();
    case 263:
      ACCEPT_TOKEN(sym_string_content);
      if (lookahead != 0 &&
          lookahead != '"' &&
          lookahead != '\\' &&
          lookahead != '{') ADVANCE(263);
      END_STATE();
    case 264:
      ACCEPT_TOKEN(sym_escape_sequence);
      END_STATE();
    case 265:
      ACCEPT_TOKEN(sym_number);
      if (lookahead == '.') ADVANCE(179);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(265);
      END_STATE();
    case 266:
      ACCEPT_TOKEN(sym_number);
      if (('0' <= lookahead && lookahead <= '9')) ADVANCE(266);
      END_STATE();
    case 267:
      ACCEPT_TOKEN(anon_sym_true);
      END_STATE();
    case 268:
      ACCEPT_TOKEN(anon_sym_true);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 269:
      ACCEPT_TOKEN(anon_sym_false);
      END_STATE();
    case 270:
      ACCEPT_TOKEN(anon_sym_false);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 271:
      ACCEPT_TOKEN(sym_null);
      END_STATE();
    case 272:
      ACCEPT_TOKEN(sym_null);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 273:
      ACCEPT_TOKEN(anon_sym_LBRACK);
      END_STATE();
    case 274:
      ACCEPT_TOKEN(anon_sym_RBRACK);
      END_STATE();
    case 275:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == ':') ADVANCE(31);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 276:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'B') ADVANCE(591);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 277:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'D') ADVANCE(564);
      if (lookahead == 'L') ADVANCE(401);
      if (lookahead == 'R') ADVANCE(459);
      if (lookahead == 'U') ADVANCE(568);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 278:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'I') ADVANCE(523);
      if (lookahead == 'O') ADVANCE(646);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 279:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'P') ADVANCE(286);
      if (lookahead == 'l') ADVANCE(380);
      if (lookahead == 's') ADVANCE(213);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 280:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'P') ADVANCE(463);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 281:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'U') ADVANCE(572);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 282:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'a') ADVANCE(353);
      if (lookahead == 'l') ADVANCE(551);
      if (lookahead == 'r') ADVANCE(422);
      if (lookahead == 'u') ADVANCE(618);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('b' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 283:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'a') ADVANCE(654);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('b' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 284:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'a') ADVANCE(441);
      if (lookahead == 'r') ADVANCE(546);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('b' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 285:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'a') ADVANCE(325);
      if (lookahead == 'b') ADVANCE(557);
      if (lookahead == 'c') ADVANCE(417);
      if (lookahead == 'e') ADVANCE(665);
      if (lookahead == 'h') ADVANCE(423);
      if (lookahead == 'o') ADVANCE(290);
      if (lookahead == 'r') ADVANCE(543);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('d' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 286:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'a') ADVANCE(439);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('b' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 287:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'a') ADVANCE(480);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('b' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 288:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'a') ADVANCE(369);
      if (lookahead == 'i') ADVANCE(616);
      if (lookahead == 'l') ADVANCE(309);
      if (lookahead == 'u') ADVANCE(492);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('b' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 289:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'a') ADVANCE(576);
      if (lookahead == 'h') ADVANCE(400);
      if (lookahead == 'o') ADVANCE(363);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('b' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 290:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'a') ADVANCE(605);
      if (lookahead == 'o') ADVANCE(503);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('b' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 291:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'a') ADVANCE(622);
      if (lookahead == 'i') ADVANCE(296);
      if (lookahead == 'o') ADVANCE(348);
      if (lookahead == 'r') ADVANCE(548);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('b' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 292:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'a') ADVANCE(501);
      if (lookahead == 'e') ADVANCE(621);
      if (lookahead == 'o') ADVANCE(578);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('b' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 293:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'a') ADVANCE(501);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('b' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 294:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'a') ADVANCE(655);
      if (lookahead == 'u') ADVANCE(490);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('b' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 295:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'a') ADVANCE(373);
      if (lookahead == 'o') ADVANCE(652);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('b' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 296:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'a') ADVANCE(494);
      if (lookahead == 'v') ADVANCE(461);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('b' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 297:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'a') ADVANCE(524);
      if (lookahead == 'u') ADVANCE(385);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('b' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 298:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'a') ADVANCE(355);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('b' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 299:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'a') ADVANCE(620);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('b' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 300:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'a') ADVANCE(484);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('b' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 301:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'a') ADVANCE(570);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('b' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 302:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'a') ADVANCE(349);
      if (lookahead == 'i') ADVANCE(529);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('b' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 303:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'a') ADVANCE(356);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('b' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 304:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'a') ADVANCE(329);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('b' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 305:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'a') ADVANCE(486);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('b' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 306:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'a') ADVANCE(577);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('b' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 307:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'a') ADVANCE(457);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('b' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 308:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'a') ADVANCE(364);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('b' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 309:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'a') ADVANCE(616);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('b' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 310:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'a') ADVANCE(504);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('b' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 311:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'a') ADVANCE(594);
      if (lookahead == 'g') ADVANCE(193);
      if (lookahead == 'o') ADVANCE(663);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('b' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 312:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'a') ADVANCE(588);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('b' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 313:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'a') ADVANCE(534);
      if (lookahead == 'i') ADVANCE(602);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('b' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 314:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'a') ADVANCE(581);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('b' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 315:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'a') ADVANCE(440);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('b' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 316:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'a') ADVANCE(362);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('b' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 317:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'a') ADVANCE(590);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('b' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 318:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'a') ADVANCE(630);
      if (lookahead == 'y') ADVANCE(497);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('b' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 319:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'a') ADVANCE(375);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('b' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 320:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'a') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('b' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 321:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'a') ADVANCE(499);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('b' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 322:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'a') ADVANCE(631);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('b' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 323:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'a') ADVANCE(632);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('b' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 324:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'b') ADVANCE(213);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 325:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'b') ADVANCE(279);
      if (lookahead == 'g') ADVANCE(213);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 326:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'b') ADVANCE(306);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 327:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'b') ADVANCE(545);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 328:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'b') ADVANCE(500);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 329:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'c') ADVANCE(480);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 330:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'c') ADVANCE(446);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 331:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'c') ADVANCE(193);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 332:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'c') ADVANCE(481);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 333:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'c') ADVANCE(445);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 334:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'c') ADVANCE(617);
      if (lookahead == 'l') ADVANCE(415);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 335:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'c') ADVANCE(483);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 336:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'c') ADVANCE(549);
      if (lookahead == 'm') ADVANCE(286);
      if (lookahead == 'n') ADVANCE(569);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 337:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'c') ADVANCE(485);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 338:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'c') ADVANCE(482);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 339:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'c') ADVANCE(614);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 340:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'c') ADVANCE(615);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 341:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'c') ADVANCE(345);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 342:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'c') ADVANCE(595);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 343:
      ACCEPT_TOKEN(sym_identifier);
      ADVANCE_MAP(
        'c', 321,
        'e', 346,
        'h', 300,
        'l', 477,
        'm', 192,
        'p', 468,
        'q', 648,
        'u', 352,
      );
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 344:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'c') ADVANCE(320);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 345:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'c') ADVANCE(397);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 346:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'c') ADVANCE(561);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 347:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'c') ADVANCE(392);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 348:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'c') ADVANCE(641);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 349:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'c') ADVANCE(404);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 350:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'c') ADVANCE(424);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 351:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'c') ADVANCE(635);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 352:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'c') ADVANCE(350);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 353:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'd') ADVANCE(439);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 354:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'd') ADVANCE(213);
      if (lookahead == 'o') ADVANCE(651);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 355:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'd') ADVANCE(213);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 356:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'd') ADVANCE(403);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 357:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'd') ADVANCE(666);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 358:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'd') ADVANCE(200);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 359:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'd') ADVANCE(193);
      if (lookahead == 'e') ADVANCE(365);
      if (lookahead == 'u') ADVANCE(632);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 360:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'd') ADVANCE(193);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 361:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'd') ADVANCE(305);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 362:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'd') ADVANCE(462);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 363:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'd') ADVANCE(380);
      if (lookahead == 'l') ADVANCE(639);
      if (lookahead == 'n') ADVANCE(623);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 364:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'd') ADVANCE(342);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 365:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'd') ADVANCE(478);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 366:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'd') ADVANCE(379);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 367:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'd') ADVANCE(409);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 368:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'd') ADVANCE(550);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 369:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'd') ADVANCE(393);
      if (lookahead == 'l') ADVANCE(609);
      if (lookahead == 's') ADVANCE(616);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 370:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'd') ADVANCE(408);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 371:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'd') ADVANCE(394);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 372:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'd') ADVANCE(404);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 373:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'd') ADVANCE(465);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 374:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'd') ADVANCE(314);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 375:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'd') ADVANCE(470);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 376:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'd') ADVANCE(419);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 377:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'd') ADVANCE(412);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 378:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'e') ADVANCE(334);
      if (lookahead == 'i') ADVANCE(366);
      if (lookahead == 'k') ADVANCE(416);
      if (lookahead == 'l') ADVANCE(461);
      if (lookahead == 'p') ADVANCE(302);
      if (lookahead == 't') ADVANCE(304);
      if (lookahead == 'w') ADVANCE(473);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 379:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'e') ADVANCE(326);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 380:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'e') ADVANCE(213);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 381:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'e') ADVANCE(206);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 382:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'e') ADVANCE(280);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 383:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'e') ADVANCE(281);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 384:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'e') ADVANCE(276);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 385:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'e') ADVANCE(268);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 386:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'e') ADVANCE(212);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 387:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'e') ADVANCE(270);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 388:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'e') ADVANCE(198);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 389:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'e') ADVANCE(235);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 390:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'e') ADVANCE(208);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 391:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'e') ADVANCE(217);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 392:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'e') ADVANCE(193);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 393:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'e') ADVANCE(278);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 394:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'e') ADVANCE(277);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 395:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'e') ADVANCE(525);
      if (lookahead == 'o') ADVANCE(361);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 396:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'e') ADVANCE(303);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 397:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'e') ADVANCE(601);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 398:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'e') ADVANCE(657);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 399:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'e') ADVANCE(583);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 400:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'e') ADVANCE(335);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 401:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'e') ADVANCE(431);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 402:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'e') ADVANCE(486);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 403:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'e') ADVANCE(577);
      if (lookahead == 'i') ADVANCE(527);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 404:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'e') ADVANCE(577);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 405:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'e') ADVANCE(287);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 406:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'e') ADVANCE(358);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 407:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'e') ADVANCE(530);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 408:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'e') ADVANCE(360);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 409:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'e') ADVANCE(542);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 410:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'e') ADVANCE(586);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 411:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'e') ADVANCE(540);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 412:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'e') ADVANCE(597);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 413:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'e') ADVANCE(580);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 414:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'e') ADVANCE(582);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 415:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'e') ADVANCE(339);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 416:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'e') ADVANCE(496);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 417:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'e') ADVANCE(491);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 418:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'e') ADVANCE(603);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 419:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'e') ADVANCE(589);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 420:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'e') ADVANCE(628);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 421:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'e') ADVANCE(340);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 422:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'e') ADVANCE(308);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 423:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'e') ADVANCE(298);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 424:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'e') ADVANCE(607);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 425:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'e') ADVANCE(319);
      if (('1' <= lookahead && lookahead <= '6')) ADVANCE(193);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 426:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'f') ADVANCE(215);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 427:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'f') ADVANCE(433);
      if (lookahead == 'l') ADVANCE(611);
      if (lookahead == 'r') ADVANCE(584);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 428:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'f') ADVANCE(433);
      if (lookahead == 'l') ADVANCE(611);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 429:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'f') ADVANCE(433);
      if (lookahead == 'r') ADVANCE(584);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 430:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'f') ADVANCE(433);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 431:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'f') ADVANCE(616);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 432:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'f') ADVANCE(544);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 433:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'f') ADVANCE(421);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 434:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'g') ADVANCE(213);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 435:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'g') ADVANCE(228);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 436:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'g') ADVANCE(193);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 437:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'g') ADVANCE(449);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 438:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'g') ADVANCE(593);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 439:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'g') ADVANCE(380);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 440:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'g') ADVANCE(585);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 441:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'g') ADVANCE(384);
      if (lookahead == 'r') ADVANCE(315);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 442:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'g') ADVANCE(392);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 443:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'g') ADVANCE(322);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 444:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'g') ADVANCE(413);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 445:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'h') ADVANCE(213);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 446:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'h') ADVANCE(225);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 447:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'h') ADVANCE(547);
      if (lookahead == 't') ADVANCE(318);
      if (lookahead == 'u') ADVANCE(341);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 448:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'h') ADVANCE(547);
      if (lookahead == 't') ADVANCE(318);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 449:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'h') ADVANCE(616);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 450:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'i') ADVANCE(493);
      if (lookahead == 'o') ADVANCE(562);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 451:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'i') ADVANCE(517);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 452:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'i') ADVANCE(656);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 453:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'i') ADVANCE(443);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 454:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'i') ADVANCE(328);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 455:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'i') ADVANCE(567);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 456:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'i') ADVANCE(355);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 457:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'i') ADVANCE(536);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 458:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'i') ADVANCE(437);
      if (lookahead == 'o') ADVANCE(650);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 459:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'i') ADVANCE(437);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 460:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'i') ADVANCE(515);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 461:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'i') ADVANCE(372);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 462:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'i') ADVANCE(528);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 463:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'i') ADVANCE(337);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 464:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'i') ADVANCE(556);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 465:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'i') ADVANCE(542);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 466:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'i') ADVANCE(558);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 467:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'i') ADVANCE(331);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 468:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'i') ADVANCE(523);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 469:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'i') ADVANCE(559);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 470:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'i') ADVANCE(531);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 471:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'i') ADVANCE(532);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 472:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'i') ADVANCE(367);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 473:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'i') ADVANCE(625);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 474:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'i') ADVANCE(533);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 475:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'i') ADVANCE(492);
      if (lookahead == 'r') ADVANCE(460);
      if (lookahead == 'u') ADVANCE(506);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 476:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'i') ADVANCE(606);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 477:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'i') ADVANCE(371);
      if (lookahead == 'o') ADVANCE(660);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 478:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'i') ADVANCE(645);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 479:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'i') ADVANCE(636);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 480:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'k') ADVANCE(213);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 481:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'k') ADVANCE(575);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 482:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'k') ADVANCE(193);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 483:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'k') ADVANCE(327);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 484:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'k') ADVANCE(392);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 485:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'k') ADVANCE(404);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 486:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'l') ADVANCE(213);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 487:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'l') ADVANCE(272);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 488:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'l') ADVANCE(193);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 489:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'l') ADVANCE(399);
      if (lookahead == 'v') ADVANCE(299);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 490:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'l') ADVANCE(487);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 491:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'l') ADVANCE(486);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 492:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'l') ADVANCE(488);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 493:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'l') ADVANCE(383);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 494:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'l') ADVANCE(552);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 495:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'l') ADVANCE(360);
      if (lookahead == 'u') ADVANCE(537);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 496:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'l') ADVANCE(420);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 497:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'l') ADVANCE(389);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 498:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'l') ADVANCE(398);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 499:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'l') ADVANCE(393);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 500:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'l') ADVANCE(392);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 501:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'l') ADVANCE(609);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 502:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'l') ADVANCE(560);
      if (lookahead == 'o') ADVANCE(495);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 503:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'l') ADVANCE(624);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 504:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'l') ADVANCE(467);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 505:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'l') ADVANCE(471);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 506:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'l') ADVANCE(612);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 507:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'l') ADVANCE(474);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 508:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'l') ADVANCE(565);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 509:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'm') ADVANCE(213);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 510:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'm') ADVANCE(193);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 511:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'm') ADVANCE(324);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 512:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'm') ADVANCE(518);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 513:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'm') ADVANCE(476);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 514:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'm') ADVANCE(407);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 515:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'm') ADVANCE(314);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 516:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'n') ADVANCE(275);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 517:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'n') ADVANCE(480);
      if (lookahead == 's') ADVANCE(614);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 518:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'n') ADVANCE(213);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 519:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'n') ADVANCE(210);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 520:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'n') ADVANCE(211);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 521:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'n') ADVANCE(204);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 522:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'n') ADVANCE(242);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 523:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'n') ADVANCE(193);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 524:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'n') ADVANCE(613);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 525:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'n') ADVANCE(638);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 526:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'n') ADVANCE(432);
      if (lookahead == 't') ADVANCE(310);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 527:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'n') ADVANCE(434);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 528:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'n') ADVANCE(435);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 529:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'n') ADVANCE(536);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 530:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'n') ADVANCE(614);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 531:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'n') ADVANCE(436);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 532:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'n') ADVANCE(408);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 533:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'n') ADVANCE(392);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 534:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'n') ADVANCE(444);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 535:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'n') ADVANCE(374);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 536:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'n') ADVANCE(404);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 537:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'n') ADVANCE(347);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 538:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'n') ADVANCE(470);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 539:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'n') ADVANCE(370);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 540:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'n') ADVANCE(637);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 541:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'n') ADVANCE(377);
      if (lookahead == 'p') ADVANCE(574);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 542:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'o') ADVANCE(213);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 543:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'o') ADVANCE(658);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 544:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'o') ADVANCE(193);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 545:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'o') ADVANCE(664);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 546:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'o') ADVANCE(438);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 547:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'o') ADVANCE(659);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 548:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'o') ADVANCE(573);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 549:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'o') ADVANCE(519);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 550:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'o') ADVANCE(661);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 551:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'o') ADVANCE(332);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 552:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'o') ADVANCE(434);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 553:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'o') ADVANCE(579);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 554:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'o') ADVANCE(520);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 555:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'o') ADVANCE(643);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 556:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'o') ADVANCE(518);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 557:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'o') ADVANCE(357);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 558:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'o') ADVANCE(521);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 559:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'o') ADVANCE(522);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 560:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'o') ADVANCE(338);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 561:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'o') ADVANCE(535);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 562:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'o') ADVANCE(633);
      if (lookahead == 'r') ADVANCE(509);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 563:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'o') ADVANCE(316);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 564:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'o') ADVANCE(662);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 565:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'o') ADVANCE(298);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 566:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'o') ADVANCE(619);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 567:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'p') ADVANCE(213);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 568:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'p') ADVANCE(193);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 569:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'p') ADVANCE(644);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 570:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'p') ADVANCE(445);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 571:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'p') ADVANCE(617);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 572:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'p') ADVANCE(508);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 573:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'p') ADVANCE(368);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 574:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'p') ADVANCE(410);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 575:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'q') ADVANCE(649);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 576:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'r') ADVANCE(354);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 577:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'r') ADVANCE(213);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 578:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'r') ADVANCE(219);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 579:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'r') ADVANCE(230);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 580:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'r') ADVANCE(193);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 581:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'r') ADVANCE(667);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 582:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'r') ADVANCE(452);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 583:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'r') ADVANCE(614);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 584:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'r') ADVANCE(553);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 585:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'r') ADVANCE(301);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 586:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'r') ADVANCE(344);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 587:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'r') ADVANCE(555);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 588:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'r') ADVANCE(538);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 589:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'r') ADVANCE(408);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 590:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'r') ADVANCE(392);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 591:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'r') ADVANCE(405);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 592:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'r') ADVANCE(456);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 593:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'r') ADVANCE(418);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 594:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'r') ADVANCE(442);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 595:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'r') ADVANCE(642);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 596:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'r') ADVANCE(376);
      if (lookahead == 'u') ADVANCE(627);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 597:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'r') ADVANCE(507);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 598:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 's') ADVANCE(213);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 599:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 's') ADVANCE(232);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 600:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 's') ADVANCE(193);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 601:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 's') ADVANCE(599);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 602:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 's') ADVANCE(513);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 603:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 's') ADVANCE(598);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 604:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 's') ADVANCE(381);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 605:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 's') ADVANCE(614);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 606:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 's') ADVANCE(608);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 607:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 's') ADVANCE(600);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 608:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 's') ADVANCE(454);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 609:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 's') ADVANCE(387);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 610:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 's') ADVANCE(402);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 611:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 's') ADVANCE(391);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 612:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 's') ADVANCE(392);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 613:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 's') ADVANCE(479);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 614:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 't') ADVANCE(213);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 615:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 't') ADVANCE(202);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 616:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 't') ADVANCE(193);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 617:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 't') ADVANCE(464);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 618:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 't') ADVANCE(626);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 619:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 't') ADVANCE(380);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 620:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 't') ADVANCE(306);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 621:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 't') ADVANCE(330);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 622:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 't') ADVANCE(382);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 623:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 't') ADVANCE(307);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 624:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 't') ADVANCE(455);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 625:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 't') ADVANCE(333);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 626:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 't') ADVANCE(554);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 627:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 't') ADVANCE(505);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 628:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 't') ADVANCE(556);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 629:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 't') ADVANCE(386);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 630:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 't') ADVANCE(388);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 631:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 't') ADVANCE(390);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 632:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 't') ADVANCE(408);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 633:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 't') ADVANCE(404);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 634:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 't') ADVANCE(628);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 635:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 't') ADVANCE(466);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 636:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 't') ADVANCE(469);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 637:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 't') ADVANCE(413);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 638:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'u') ADVANCE(213);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 639:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'u') ADVANCE(512);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 640:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'u') ADVANCE(490);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 641:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'u') ADVANCE(514);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 642:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'u') ADVANCE(511);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 643:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'u') ADVANCE(567);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 644:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'u') ADVANCE(614);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 645:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'u') ADVANCE(510);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 646:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'u') ADVANCE(616);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 647:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'u') ADVANCE(385);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 648:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'u') ADVANCE(317);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 649:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'u') ADVANCE(566);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 650:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'u') ADVANCE(539);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 651:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'u') ADVANCE(610);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 652:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'u') ADVANCE(629);
      if (lookahead == 'w') ADVANCE(213);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 653:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'u') ADVANCE(634);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 654:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'v') ADVANCE(326);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 655:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'v') ADVANCE(453);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 656:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'v') ADVANCE(406);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 657:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'v') ADVANCE(323);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 658:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'w') ADVANCE(213);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 659:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'w') ADVANCE(222);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 660:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'w') ADVANCE(193);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 661:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'w') ADVANCE(518);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 662:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'w') ADVANCE(523);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 663:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'w') ADVANCE(410);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 664:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'x') ADVANCE(213);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 665:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'x') ADVANCE(614);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 666:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'y') ADVANCE(213);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 667:
      ACCEPT_TOKEN(sym_identifier);
      if (lookahead == 'y') ADVANCE(193);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 668:
      ACCEPT_TOKEN(sym_identifier);
      if (('0' <= lookahead && lookahead <= '9') ||
          ('A' <= lookahead && lookahead <= 'Z') ||
          lookahead == '_' ||
          ('a' <= lookahead && lookahead <= 'z')) ADVANCE(668);
      END_STATE();
    case 669:
      ACCEPT_TOKEN(anon_sym_SLASH_SLASH);
      END_STATE();
    case 670:
      ACCEPT_TOKEN(anon_sym_SLASH_SLASH);
      if (lookahead == '*') ADVANCE(682);
      if (lookahead != 0) ADVANCE(12);
      END_STATE();
    case 671:
      ACCEPT_TOKEN(anon_sym_SLASH_SLASH);
      if (lookahead != 0 &&
          lookahead != '"' &&
          lookahead != '\\' &&
          lookahead != '{') ADVANCE(263);
      END_STATE();
    case 672:
      ACCEPT_TOKEN(anon_sym_SLASH_SLASH);
      if (lookahead != 0 &&
          lookahead != '\n') ADVANCE(676);
      END_STATE();
    case 673:
      ACCEPT_TOKEN(anon_sym_SLASH_SLASH);
      if (lookahead != 0 &&
          lookahead != '{') ADVANCE(240);
      END_STATE();
    case 674:
      ACCEPT_TOKEN(aux_sym_comment_token1);
      if (lookahead == '*') ADVANCE(680);
      if (lookahead == '/') ADVANCE(672);
      if (lookahead != 0 &&
          lookahead != '\n') ADVANCE(676);
      END_STATE();
    case 675:
      ACCEPT_TOKEN(aux_sym_comment_token1);
      if (lookahead == '/') ADVANCE(674);
      if (lookahead == '\t' ||
          (0x0b <= lookahead && lookahead <= '\r') ||
          lookahead == ' ') ADVANCE(675);
      if (lookahead != 0 &&
          (lookahead < '\t' || '\r' < lookahead)) ADVANCE(676);
      END_STATE();
    case 676:
      ACCEPT_TOKEN(aux_sym_comment_token1);
      if (lookahead != 0 &&
          lookahead != '\n') ADVANCE(676);
      END_STATE();
    case 677:
      ACCEPT_TOKEN(anon_sym_SLASH_STAR);
      END_STATE();
    case 678:
      ACCEPT_TOKEN(anon_sym_SLASH_STAR);
      if (lookahead == '*') ADVANCE(682);
      if (lookahead != 0 &&
          lookahead != '/') ADVANCE(12);
      END_STATE();
    case 679:
      ACCEPT_TOKEN(anon_sym_SLASH_STAR);
      if (lookahead != 0 &&
          lookahead != '"' &&
          lookahead != '\\' &&
          lookahead != '{') ADVANCE(263);
      END_STATE();
    case 680:
      ACCEPT_TOKEN(anon_sym_SLASH_STAR);
      if (lookahead != 0 &&
          lookahead != '\n') ADVANCE(676);
      END_STATE();
    case 681:
      ACCEPT_TOKEN(anon_sym_SLASH_STAR);
      if (lookahead != 0 &&
          lookahead != '{') ADVANCE(240);
      END_STATE();
    case 682:
      ACCEPT_TOKEN(aux_sym_comment_token2);
      if (lookahead == '*') ADVANCE(682);
      if (lookahead != 0 &&
          lookahead != '/') ADVANCE(12);
      END_STATE();
    default:
      return false;
  }
}

static const TSLexMode ts_lex_modes[STATE_COUNT] = {
  [0] = {.lex_state = 0},
  [1] = {.lex_state = 0},
  [2] = {.lex_state = 2},
  [3] = {.lex_state = 2},
  [4] = {.lex_state = 2},
  [5] = {.lex_state = 2},
  [6] = {.lex_state = 2},
  [7] = {.lex_state = 1},
  [8] = {.lex_state = 1},
  [9] = {.lex_state = 1},
  [10] = {.lex_state = 1},
  [11] = {.lex_state = 1},
  [12] = {.lex_state = 1},
  [13] = {.lex_state = 1},
  [14] = {.lex_state = 1},
  [15] = {.lex_state = 1},
  [16] = {.lex_state = 1},
  [17] = {.lex_state = 1},
  [18] = {.lex_state = 2},
  [19] = {.lex_state = 1},
  [20] = {.lex_state = 1},
  [21] = {.lex_state = 1},
  [22] = {.lex_state = 1},
  [23] = {.lex_state = 1},
  [24] = {.lex_state = 1},
  [25] = {.lex_state = 1},
  [26] = {.lex_state = 1},
  [27] = {.lex_state = 1},
  [28] = {.lex_state = 1},
  [29] = {.lex_state = 1},
  [30] = {.lex_state = 1},
  [31] = {.lex_state = 1},
  [32] = {.lex_state = 1},
  [33] = {.lex_state = 1},
  [34] = {.lex_state = 2},
  [35] = {.lex_state = 1},
  [36] = {.lex_state = 2},
  [37] = {.lex_state = 2},
  [38] = {.lex_state = 2},
  [39] = {.lex_state = 2},
  [40] = {.lex_state = 2},
  [41] = {.lex_state = 2},
  [42] = {.lex_state = 2},
  [43] = {.lex_state = 2},
  [44] = {.lex_state = 2},
  [45] = {.lex_state = 2},
  [46] = {.lex_state = 2},
  [47] = {.lex_state = 2},
  [48] = {.lex_state = 2},
  [49] = {.lex_state = 2},
  [50] = {.lex_state = 2},
  [51] = {.lex_state = 2},
  [52] = {.lex_state = 2},
  [53] = {.lex_state = 2},
  [54] = {.lex_state = 2},
  [55] = {.lex_state = 2},
  [56] = {.lex_state = 2},
  [57] = {.lex_state = 1},
  [58] = {.lex_state = 1},
  [59] = {.lex_state = 1},
  [60] = {.lex_state = 1},
  [61] = {.lex_state = 1},
  [62] = {.lex_state = 1},
  [63] = {.lex_state = 1},
  [64] = {.lex_state = 4},
  [65] = {.lex_state = 4},
  [66] = {.lex_state = 2},
  [67] = {.lex_state = 2},
  [68] = {.lex_state = 2},
  [69] = {.lex_state = 4},
  [70] = {.lex_state = 4},
  [71] = {.lex_state = 2},
  [72] = {.lex_state = 2},
  [73] = {.lex_state = 2},
  [74] = {.lex_state = 2},
  [75] = {.lex_state = 2},
  [76] = {.lex_state = 2},
  [77] = {.lex_state = 2},
  [78] = {.lex_state = 2},
  [79] = {.lex_state = 2},
  [80] = {.lex_state = 2},
  [81] = {.lex_state = 2},
  [82] = {.lex_state = 2},
  [83] = {.lex_state = 2},
  [84] = {.lex_state = 2},
  [85] = {.lex_state = 2},
  [86] = {.lex_state = 2},
  [87] = {.lex_state = 2},
  [88] = {.lex_state = 2},
  [89] = {.lex_state = 2},
  [90] = {.lex_state = 2},
  [91] = {.lex_state = 2},
  [92] = {.lex_state = 2},
  [93] = {.lex_state = 2},
  [94] = {.lex_state = 2},
  [95] = {.lex_state = 2},
  [96] = {.lex_state = 2},
  [97] = {.lex_state = 2},
  [98] = {.lex_state = 2},
  [99] = {.lex_state = 2},
  [100] = {.lex_state = 2},
  [101] = {.lex_state = 2},
  [102] = {.lex_state = 2},
  [103] = {.lex_state = 2},
  [104] = {.lex_state = 2},
  [105] = {.lex_state = 2},
  [106] = {.lex_state = 5},
  [107] = {.lex_state = 5},
  [108] = {.lex_state = 5},
  [109] = {.lex_state = 5},
  [110] = {.lex_state = 1},
  [111] = {.lex_state = 1},
  [112] = {.lex_state = 1},
  [113] = {.lex_state = 3},
  [114] = {.lex_state = 1},
  [115] = {.lex_state = 1},
  [116] = {.lex_state = 3},
  [117] = {.lex_state = 1},
  [118] = {.lex_state = 1},
  [119] = {.lex_state = 1},
  [120] = {.lex_state = 1},
  [121] = {.lex_state = 1},
  [122] = {.lex_state = 1},
  [123] = {.lex_state = 3},
  [124] = {.lex_state = 1},
  [125] = {.lex_state = 1},
  [126] = {.lex_state = 1},
  [127] = {.lex_state = 1},
  [128] = {.lex_state = 3},
  [129] = {.lex_state = 1},
  [130] = {.lex_state = 1},
  [131] = {.lex_state = 1},
  [132] = {.lex_state = 1},
  [133] = {.lex_state = 1},
  [134] = {.lex_state = 1},
  [135] = {.lex_state = 1},
  [136] = {.lex_state = 1},
  [137] = {.lex_state = 1},
  [138] = {.lex_state = 1},
  [139] = {.lex_state = 1},
  [140] = {.lex_state = 3},
  [141] = {.lex_state = 1},
  [142] = {.lex_state = 1},
  [143] = {.lex_state = 1},
  [144] = {.lex_state = 1},
  [145] = {.lex_state = 3},
  [146] = {.lex_state = 1},
  [147] = {.lex_state = 1},
  [148] = {.lex_state = 1},
  [149] = {.lex_state = 3},
  [150] = {.lex_state = 1},
  [151] = {.lex_state = 3},
  [152] = {.lex_state = 3},
  [153] = {.lex_state = 3},
  [154] = {.lex_state = 3},
  [155] = {.lex_state = 6},
  [156] = {.lex_state = 6},
  [157] = {.lex_state = 6},
  [158] = {.lex_state = 6},
  [159] = {.lex_state = 6},
  [160] = {.lex_state = 6},
  [161] = {.lex_state = 6},
  [162] = {.lex_state = 6},
  [163] = {.lex_state = 6},
  [164] = {.lex_state = 6},
  [165] = {.lex_state = 6},
  [166] = {.lex_state = 6},
  [167] = {.lex_state = 6},
  [168] = {.lex_state = 6},
  [169] = {.lex_state = 6},
  [170] = {.lex_state = 6},
  [171] = {.lex_state = 6},
  [172] = {.lex_state = 6},
  [173] = {.lex_state = 6},
  [174] = {.lex_state = 6},
  [175] = {.lex_state = 6},
  [176] = {.lex_state = 6},
  [177] = {.lex_state = 6},
  [178] = {.lex_state = 6},
  [179] = {.lex_state = 6},
  [180] = {.lex_state = 6},
  [181] = {.lex_state = 6},
  [182] = {.lex_state = 6},
  [183] = {.lex_state = 6},
  [184] = {.lex_state = 6},
  [185] = {.lex_state = 6},
  [186] = {.lex_state = 6},
  [187] = {.lex_state = 6},
  [188] = {.lex_state = 6},
  [189] = {.lex_state = 6},
  [190] = {.lex_state = 6},
  [191] = {.lex_state = 6},
  [192] = {.lex_state = 6},
  [193] = {.lex_state = 6},
  [194] = {.lex_state = 6},
  [195] = {.lex_state = 7},
  [196] = {.lex_state = 7},
  [197] = {.lex_state = 7},
  [198] = {.lex_state = 7},
  [199] = {.lex_state = 0},
  [200] = {.lex_state = 7},
  [201] = {.lex_state = 7},
  [202] = {.lex_state = 7},
  [203] = {.lex_state = 7},
  [204] = {.lex_state = 0},
  [205] = {.lex_state = 7},
  [206] = {.lex_state = 7},
  [207] = {.lex_state = 7},
  [208] = {.lex_state = 0},
  [209] = {.lex_state = 7},
  [210] = {.lex_state = 7},
  [211] = {.lex_state = 7},
  [212] = {.lex_state = 7},
  [213] = {.lex_state = 7},
  [214] = {.lex_state = 0},
  [215] = {.lex_state = 0},
  [216] = {.lex_state = 0},
  [217] = {.lex_state = 0},
  [218] = {.lex_state = 0},
  [219] = {.lex_state = 0},
  [220] = {.lex_state = 0},
  [221] = {.lex_state = 0},
  [222] = {.lex_state = 0},
  [223] = {.lex_state = 7},
  [224] = {.lex_state = 7},
  [225] = {.lex_state = 7},
  [226] = {.lex_state = 0},
  [227] = {.lex_state = 0},
  [228] = {.lex_state = 0},
  [229] = {.lex_state = 0},
  [230] = {.lex_state = 0},
  [231] = {.lex_state = 0},
  [232] = {.lex_state = 0},
  [233] = {.lex_state = 0},
  [234] = {.lex_state = 0},
  [235] = {.lex_state = 0},
  [236] = {.lex_state = 0},
  [237] = {.lex_state = 0},
  [238] = {.lex_state = 7},
  [239] = {.lex_state = 0},
  [240] = {.lex_state = 0},
  [241] = {.lex_state = 0},
  [242] = {.lex_state = 0},
  [243] = {.lex_state = 0},
  [244] = {.lex_state = 0},
  [245] = {.lex_state = 0},
  [246] = {.lex_state = 0},
  [247] = {.lex_state = 0},
  [248] = {.lex_state = 0},
  [249] = {.lex_state = 7},
  [250] = {.lex_state = 7},
  [251] = {.lex_state = 7},
  [252] = {.lex_state = 7},
  [253] = {.lex_state = 7},
  [254] = {.lex_state = 8},
  [255] = {.lex_state = 8},
  [256] = {.lex_state = 8},
  [257] = {.lex_state = 8},
  [258] = {.lex_state = 8},
  [259] = {.lex_state = 8},
  [260] = {.lex_state = 8},
  [261] = {.lex_state = 0},
  [262] = {.lex_state = 7},
  [263] = {.lex_state = 0},
  [264] = {.lex_state = 0},
  [265] = {.lex_state = 7},
  [266] = {.lex_state = 7},
  [267] = {.lex_state = 0},
  [268] = {.lex_state = 0},
  [269] = {.lex_state = 0},
  [270] = {.lex_state = 0},
  [271] = {.lex_state = 0},
  [272] = {.lex_state = 0},
  [273] = {.lex_state = 0},
  [274] = {.lex_state = 0},
  [275] = {.lex_state = 8},
  [276] = {.lex_state = 7},
  [277] = {.lex_state = 0},
  [278] = {.lex_state = 0},
  [279] = {.lex_state = 7},
  [280] = {.lex_state = 7},
  [281] = {.lex_state = 7},
  [282] = {.lex_state = 7},
  [283] = {.lex_state = 0},
  [284] = {.lex_state = 7},
  [285] = {.lex_state = 0},
  [286] = {.lex_state = 0},
  [287] = {.lex_state = 7},
  [288] = {.lex_state = 8},
  [289] = {.lex_state = 7},
  [290] = {.lex_state = 7},
  [291] = {.lex_state = 0},
  [292] = {.lex_state = 7},
  [293] = {.lex_state = 0},
  [294] = {.lex_state = 0},
  [295] = {.lex_state = 0},
  [296] = {.lex_state = 0},
  [297] = {.lex_state = 0},
  [298] = {.lex_state = 7},
  [299] = {.lex_state = 0},
  [300] = {.lex_state = 0},
  [301] = {.lex_state = 0},
  [302] = {.lex_state = 0},
  [303] = {.lex_state = 0},
  [304] = {.lex_state = 0},
  [305] = {.lex_state = 0},
  [306] = {.lex_state = 0},
  [307] = {.lex_state = 7},
  [308] = {.lex_state = 7},
  [309] = {.lex_state = 0},
  [310] = {.lex_state = 0},
  [311] = {.lex_state = 0},
  [312] = {.lex_state = 0},
  [313] = {.lex_state = 0},
  [314] = {.lex_state = 0},
  [315] = {.lex_state = 7},
  [316] = {.lex_state = 0},
  [317] = {.lex_state = 0},
  [318] = {.lex_state = 0},
  [319] = {.lex_state = 0},
  [320] = {.lex_state = 0},
  [321] = {.lex_state = 7},
  [322] = {.lex_state = 0},
  [323] = {.lex_state = 0},
  [324] = {.lex_state = 0},
  [325] = {.lex_state = 0},
  [326] = {.lex_state = 0},
  [327] = {.lex_state = 0},
  [328] = {.lex_state = 0},
  [329] = {.lex_state = 0},
  [330] = {.lex_state = 0},
  [331] = {.lex_state = 0},
  [332] = {.lex_state = 0},
  [333] = {.lex_state = 0},
  [334] = {.lex_state = 0},
  [335] = {.lex_state = 0},
  [336] = {.lex_state = 7},
  [337] = {.lex_state = 0},
  [338] = {.lex_state = 0},
  [339] = {.lex_state = 0},
  [340] = {.lex_state = 0},
  [341] = {.lex_state = 0},
  [342] = {.lex_state = 0},
  [343] = {.lex_state = 0},
  [344] = {.lex_state = 0},
  [345] = {.lex_state = 0},
  [346] = {.lex_state = 7},
  [347] = {.lex_state = 0},
  [348] = {.lex_state = 0},
  [349] = {.lex_state = 0},
  [350] = {.lex_state = 0},
  [351] = {.lex_state = 0},
  [352] = {.lex_state = 0},
  [353] = {.lex_state = 7},
  [354] = {.lex_state = 0},
  [355] = {.lex_state = 0},
  [356] = {.lex_state = 0},
  [357] = {.lex_state = 0},
  [358] = {.lex_state = 0},
  [359] = {.lex_state = 0},
  [360] = {.lex_state = 0},
  [361] = {.lex_state = 0},
  [362] = {.lex_state = 0},
  [363] = {.lex_state = 0},
  [364] = {.lex_state = 0},
  [365] = {.lex_state = 0},
  [366] = {.lex_state = 0},
  [367] = {.lex_state = 0},
  [368] = {.lex_state = 0},
  [369] = {.lex_state = 0},
  [370] = {.lex_state = 0},
  [371] = {.lex_state = 0},
  [372] = {.lex_state = 7},
  [373] = {.lex_state = 7},
  [374] = {.lex_state = 7},
  [375] = {.lex_state = 0},
  [376] = {.lex_state = 7},
  [377] = {.lex_state = 0},
  [378] = {.lex_state = 7},
  [379] = {.lex_state = 0},
  [380] = {.lex_state = 0},
  [381] = {.lex_state = 7},
  [382] = {.lex_state = 0},
  [383] = {.lex_state = 0},
  [384] = {.lex_state = 0},
  [385] = {.lex_state = 7},
  [386] = {.lex_state = 0},
  [387] = {.lex_state = 0},
  [388] = {.lex_state = 0},
  [389] = {.lex_state = 7},
  [390] = {.lex_state = 0},
  [391] = {.lex_state = 0},
  [392] = {.lex_state = 7},
  [393] = {.lex_state = 0},
  [394] = {.lex_state = 675},
  [395] = {.lex_state = 0},
  [396] = {.lex_state = 0},
  [397] = {.lex_state = 7},
  [398] = {.lex_state = 0},
  [399] = {.lex_state = 7},
  [400] = {.lex_state = 0},
  [401] = {.lex_state = 0},
  [402] = {.lex_state = 0},
  [403] = {.lex_state = 0},
  [404] = {.lex_state = 0},
  [405] = {.lex_state = 0},
  [406] = {.lex_state = 0},
  [407] = {.lex_state = 0},
  [408] = {.lex_state = 7},
  [409] = {.lex_state = 7},
  [410] = {.lex_state = 11},
  [411] = {.lex_state = 0},
  [412] = {.lex_state = 0},
  [413] = {.lex_state = 0},
  [414] = {.lex_state = 0},
  [415] = {.lex_state = 0},
  [416] = {.lex_state = 7},
  [417] = {.lex_state = 7},
  [418] = {.lex_state = 7},
  [419] = {.lex_state = 7},
  [420] = {.lex_state = 7},
  [421] = {.lex_state = 0},
  [422] = {.lex_state = 0},
  [423] = {.lex_state = 7},
  [424] = {.lex_state = 0},
  [425] = {.lex_state = 0},
  [426] = {.lex_state = 0},
  [427] = {.lex_state = 0},
  [428] = {.lex_state = 7},
  [429] = {.lex_state = 0},
  [430] = {.lex_state = 14},
  [431] = {.lex_state = 0},
  [432] = {.lex_state = 0},
  [433] = {.lex_state = 0},
  [434] = {.lex_state = 0},
  [435] = {.lex_state = 0},
  [436] = {.lex_state = 0},
  [437] = {.lex_state = 0},
  [438] = {.lex_state = 7},
  [439] = {.lex_state = 7},
  [440] = {.lex_state = 7},
  [441] = {.lex_state = 7},
  [442] = {.lex_state = 0},
  [443] = {(TSStateId)(-1)},
  [444] = {(TSStateId)(-1)},
};

static const uint16_t ts_parse_table[LARGE_STATE_COUNT][SYMBOL_COUNT] = {
  [0] = {
    [sym_comment] = STATE(0),
    [ts_builtin_sym_end] = ACTIONS(1),
    [anon_sym_Page] = ACTIONS(1),
    [anon_sym_LPAREN] = ACTIONS(1),
    [anon_sym_RPAREN] = ACTIONS(1),
    [anon_sym_Component] = ACTIONS(1),
    [anon_sym_Store] = ACTIONS(1),
    [anon_sym_App] = ACTIONS(1),
    [anon_sym_COMMA] = ACTIONS(1),
    [anon_sym_QMARK] = ACTIONS(1),
    [anon_sym_COLON] = ACTIONS(1),
    [anon_sym_EQ] = ACTIONS(1),
    [sym_type_annotation] = ACTIONS(1),
    [anon_sym_LBRACE] = ACTIONS(1),
    [anon_sym_RBRACE] = ACTIONS(1),
    [anon_sym_state] = ACTIONS(1),
    [anon_sym_derived] = ACTIONS(1),
    [anon_sym_effect] = ACTIONS(1),
    [anon_sym_action] = ACTIONS(1),
    [anon_sym_use] = ACTIONS(1),
    [anon_sym_navigate] = ACTIONS(1),
    [anon_sym_DOT] = ACTIONS(1),
    [anon_sym_if] = ACTIONS(1),
    [anon_sym_else] = ACTIONS(1),
    [anon_sym_for] = ACTIONS(1),
    [anon_sym_in] = ACTIONS(1),
    [anon_sym_show] = ACTIONS(1),
    [anon_sym_animate] = ACTIONS(1),
    [anon_sym_fetch] = ACTIONS(1),
    [anon_sym_from] = ACTIONS(1),
    [anon_sym_loading] = ACTIONS(1),
    [anon_sym_error] = ACTIONS(1),
    [anon_sym_success] = ACTIONS(1),
    [sym_event_name] = ACTIONS(1),
    [anon_sym_style] = ACTIONS(1),
    [anon_sym_DASH] = ACTIONS(1),
    [anon_sym_ATmedia] = ACTIONS(1),
    [anon_sym_transition] = ACTIONS(1),
    [anon_sym_PLUS] = ACTIONS(1),
    [anon_sym_STAR] = ACTIONS(1),
    [anon_sym_SLASH] = ACTIONS(1),
    [anon_sym_PERCENT] = ACTIONS(1),
    [anon_sym_EQ_EQ] = ACTIONS(1),
    [anon_sym_BANG_EQ] = ACTIONS(1),
    [anon_sym_LT] = ACTIONS(1),
    [anon_sym_GT] = ACTIONS(1),
    [anon_sym_LT_EQ] = ACTIONS(1),
    [anon_sym_GT_EQ] = ACTIONS(1),
    [anon_sym_AMP_AMP] = ACTIONS(1),
    [anon_sym_PIPE_PIPE] = ACTIONS(1),
    [anon_sym_BANG] = ACTIONS(1),
    [anon_sym_t] = ACTIONS(1),
    [anon_sym_DQUOTE] = ACTIONS(1),
    [sym_escape_sequence] = ACTIONS(1),
    [sym_number] = ACTIONS(1),
    [anon_sym_true] = ACTIONS(1),
    [anon_sym_false] = ACTIONS(1),
    [sym_null] = ACTIONS(1),
    [anon_sym_LBRACK] = ACTIONS(1),
    [anon_sym_RBRACK] = ACTIONS(1),
    [anon_sym_SLASH_SLASH] = ACTIONS(3),
    [anon_sym_SLASH_STAR] = ACTIONS(5),
  },
  [1] = {
    [sym_source_file] = STATE(426),
    [sym__declaration] = STATE(274),
    [sym_page_declaration] = STATE(269),
    [sym_component_declaration] = STATE(269),
    [sym_store_declaration] = STATE(269),
    [sym_app_declaration] = STATE(269),
    [sym_comment] = STATE(1),
    [aux_sym_source_file_repeat1] = STATE(247),
    [ts_builtin_sym_end] = ACTIONS(7),
    [anon_sym_Page] = ACTIONS(9),
    [anon_sym_Component] = ACTIONS(11),
    [anon_sym_Store] = ACTIONS(13),
    [anon_sym_App] = ACTIONS(15),
    [anon_sym_SLASH_SLASH] = ACTIONS(3),
    [anon_sym_SLASH_STAR] = ACTIONS(5),
  },
  [2] = {
    [sym__statement] = STATE(71),
    [sym_state_declaration] = STATE(103),
    [sym_derived_declaration] = STATE(103),
    [sym_effect_declaration] = STATE(103),
    [sym_action_declaration] = STATE(103),
    [sym_use_declaration] = STATE(103),
    [sym_navigate_call] = STATE(103),
    [sym_assignment] = STATE(103),
    [sym_expression_statement] = STATE(103),
    [sym_ui_element] = STATE(103),
    [sym_component_name] = STATE(66),
    [sym_sub_component] = STATE(87),
    [sym_if_statement] = STATE(103),
    [sym_for_statement] = STATE(103),
    [sym_show_statement] = STATE(103),
    [sym_fetch_statement] = STATE(103),
    [sym_loading_block] = STATE(71),
    [sym_error_block] = STATE(71),
    [sym_success_block] = STATE(71),
    [sym_event_handler] = STATE(103),
    [sym_style_block] = STATE(103),
    [sym_transition_block] = STATE(103),
    [sym__expression] = STATE(34),
    [sym_binary_expression] = STATE(39),
    [sym_unary_expression] = STATE(39),
    [sym_member_expression] = STATE(39),
    [sym_call_expression] = STATE(39),
    [sym_t_call] = STATE(39),
    [sym_string] = STATE(39),
    [sym_boolean] = STATE(39),
    [sym_array] = STATE(39),
    [sym_object] = STATE(39),
    [sym_comment] = STATE(2),
    [aux_sym_fetch_block_repeat1] = STATE(4),
    [anon_sym_LPAREN] = ACTIONS(17),
    [anon_sym_LBRACE] = ACTIONS(19),
    [anon_sym_RBRACE] = ACTIONS(21),
    [anon_sym_state] = ACTIONS(23),
    [anon_sym_derived] = ACTIONS(25),
    [anon_sym_effect] = ACTIONS(27),
    [anon_sym_action] = ACTIONS(29),
    [anon_sym_use] = ACTIONS(31),
    [anon_sym_navigate] = ACTIONS(33),
    [sym_builtin_component] = ACTIONS(35),
    [anon_sym_if] = ACTIONS(37),
    [anon_sym_for] = ACTIONS(39),
    [anon_sym_show] = ACTIONS(41),
    [anon_sym_fetch] = ACTIONS(43),
    [anon_sym_loading] = ACTIONS(45),
    [anon_sym_error] = ACTIONS(47),
    [anon_sym_success] = ACTIONS(49),
    [sym_event_name] = ACTIONS(51),
    [anon_sym_style] = ACTIONS(53),
    [anon_sym_transition] = ACTIONS(55),
    [anon_sym_BANG] = ACTIONS(57),
    [anon_sym_t] = ACTIONS(59),
    [anon_sym_DQUOTE] = ACTIONS(61),
    [sym_number] = ACTIONS(63),
    [anon_sym_true] = ACTIONS(65),
    [anon_sym_false] = ACTIONS(65),
    [sym_null] = ACTIONS(67),
    [anon_sym_LBRACK] = ACTIONS(69),
    [sym_identifier] = ACTIONS(71),
    [anon_sym_SLASH_SLASH] = ACTIONS(3),
    [anon_sym_SLASH_STAR] = ACTIONS(5),
  },
  [3] = {
    [sym__statement] = STATE(71),
    [sym_state_declaration] = STATE(103),
    [sym_derived_declaration] = STATE(103),
    [sym_effect_declaration] = STATE(103),
    [sym_action_declaration] = STATE(103),
    [sym_use_declaration] = STATE(103),
    [sym_navigate_call] = STATE(103),
    [sym_assignment] = STATE(103),
    [sym_expression_statement] = STATE(103),
    [sym_ui_element] = STATE(103),
    [sym_component_name] = STATE(66),
    [sym_sub_component] = STATE(87),
    [sym_if_statement] = STATE(103),
    [sym_for_statement] = STATE(103),
    [sym_show_statement] = STATE(103),
    [sym_fetch_statement] = STATE(103),
    [sym_loading_block] = STATE(71),
    [sym_error_block] = STATE(71),
    [sym_success_block] = STATE(71),
    [sym_event_handler] = STATE(103),
    [sym_style_block] = STATE(103),
    [sym_transition_block] = STATE(103),
    [sym__expression] = STATE(34),
    [sym_binary_expression] = STATE(39),
    [sym_unary_expression] = STATE(39),
    [sym_member_expression] = STATE(39),
    [sym_call_expression] = STATE(39),
    [sym_t_call] = STATE(39),
    [sym_string] = STATE(39),
    [sym_boolean] = STATE(39),
    [sym_array] = STATE(39),
    [sym_object] = STATE(39),
    [sym_comment] = STATE(3),
    [aux_sym_fetch_block_repeat1] = STATE(3),
    [anon_sym_LPAREN] = ACTIONS(73),
    [anon_sym_LBRACE] = ACTIONS(76),
    [anon_sym_RBRACE] = ACTIONS(79),
    [anon_sym_state] = ACTIONS(81),
    [anon_sym_derived] = ACTIONS(84),
    [anon_sym_effect] = ACTIONS(87),
    [anon_sym_action] = ACTIONS(90),
    [anon_sym_use] = ACTIONS(93),
    [anon_sym_navigate] = ACTIONS(96),
    [sym_builtin_component] = ACTIONS(99),
    [anon_sym_if] = ACTIONS(102),
    [anon_sym_for] = ACTIONS(105),
    [anon_sym_show] = ACTIONS(108),
    [anon_sym_fetch] = ACTIONS(111),
    [anon_sym_loading] = ACTIONS(114),
    [anon_sym_error] = ACTIONS(117),
    [anon_sym_success] = ACTIONS(120),
    [sym_event_name] = ACTIONS(123),
    [anon_sym_style] = ACTIONS(126),
    [anon_sym_transition] = ACTIONS(129),
    [anon_sym_BANG] = ACTIONS(132),
    [anon_sym_t] = ACTIONS(135),
    [anon_sym_DQUOTE] = ACTIONS(138),
    [sym_number] = ACTIONS(141),
    [anon_sym_true] = ACTIONS(144),
    [anon_sym_false] = ACTIONS(144),
    [sym_null] = ACTIONS(147),
    [anon_sym_LBRACK] = ACTIONS(150),
    [sym_identifier] = ACTIONS(153),
    [anon_sym_SLASH_SLASH] = ACTIONS(3),
    [anon_sym_SLASH_STAR] = ACTIONS(5),
  },
  [4] = {
    [sym__statement] = STATE(71),
    [sym_state_declaration] = STATE(103),
    [sym_derived_declaration] = STATE(103),
    [sym_effect_declaration] = STATE(103),
    [sym_action_declaration] = STATE(103),
    [sym_use_declaration] = STATE(103),
    [sym_navigate_call] = STATE(103),
    [sym_assignment] = STATE(103),
    [sym_expression_statement] = STATE(103),
    [sym_ui_element] = STATE(103),
    [sym_component_name] = STATE(66),
    [sym_sub_component] = STATE(87),
    [sym_if_statement] = STATE(103),
    [sym_for_statement] = STATE(103),
    [sym_show_statement] = STATE(103),
    [sym_fetch_statement] = STATE(103),
    [sym_loading_block] = STATE(71),
    [sym_error_block] = STATE(71),
    [sym_success_block] = STATE(71),
    [sym_event_handler] = STATE(103),
    [sym_style_block] = STATE(103),
    [sym_transition_block] = STATE(103),
    [sym__expression] = STATE(34),
    [sym_binary_expression] = STATE(39),
    [sym_unary_expression] = STATE(39),
    [sym_member_expression] = STATE(39),
    [sym_call_expression] = STATE(39),
    [sym_t_call] = STATE(39),
    [sym_string] = STATE(39),
    [sym_boolean] = STATE(39),
    [sym_array] = STATE(39),
    [sym_object] = STATE(39),
    [sym_comment] = STATE(4),
    [aux_sym_fetch_block_repeat1] = STATE(3),
    [anon_sym_LPAREN] = ACTIONS(17),
    [anon_sym_LBRACE] = ACTIONS(19),
    [anon_sym_RBRACE] = ACTIONS(156),
    [anon_sym_state] = ACTIONS(23),
    [anon_sym_derived] = ACTIONS(25),
    [anon_sym_effect] = ACTIONS(27),
    [anon_sym_action] = ACTIONS(29),
    [anon_sym_use] = ACTIONS(31),
    [anon_sym_navigate] = ACTIONS(33),
    [sym_builtin_component] = ACTIONS(35),
    [anon_sym_if] = ACTIONS(37),
    [anon_sym_for] = ACTIONS(39),
    [anon_sym_show] = ACTIONS(41),
    [anon_sym_fetch] = ACTIONS(43),
    [anon_sym_loading] = ACTIONS(45),
    [anon_sym_error] = ACTIONS(47),
    [anon_sym_success] = ACTIONS(49),
    [sym_event_name] = ACTIONS(51),
    [anon_sym_style] = ACTIONS(53),
    [anon_sym_transition] = ACTIONS(55),
    [anon_sym_BANG] = ACTIONS(57),
    [anon_sym_t] = ACTIONS(59),
    [anon_sym_DQUOTE] = ACTIONS(61),
    [sym_number] = ACTIONS(63),
    [anon_sym_true] = ACTIONS(65),
    [anon_sym_false] = ACTIONS(65),
    [sym_null] = ACTIONS(67),
    [anon_sym_LBRACK] = ACTIONS(69),
    [sym_identifier] = ACTIONS(71),
    [anon_sym_SLASH_SLASH] = ACTIONS(3),
    [anon_sym_SLASH_STAR] = ACTIONS(5),
  },
  [5] = {
    [sym__statement] = STATE(71),
    [sym_state_declaration] = STATE(103),
    [sym_derived_declaration] = STATE(103),
    [sym_effect_declaration] = STATE(103),
    [sym_action_declaration] = STATE(103),
    [sym_use_declaration] = STATE(103),
    [sym_navigate_call] = STATE(103),
    [sym_assignment] = STATE(103),
    [sym_expression_statement] = STATE(103),
    [sym_ui_element] = STATE(103),
    [sym_component_name] = STATE(66),
    [sym_sub_component] = STATE(87),
    [sym_if_statement] = STATE(103),
    [sym_for_statement] = STATE(103),
    [sym_show_statement] = STATE(103),
    [sym_fetch_statement] = STATE(103),
    [sym_loading_block] = STATE(71),
    [sym_error_block] = STATE(71),
    [sym_success_block] = STATE(71),
    [sym_event_handler] = STATE(103),
    [sym_style_block] = STATE(103),
    [sym_transition_block] = STATE(103),
    [sym__expression] = STATE(34),
    [sym_binary_expression] = STATE(39),
    [sym_unary_expression] = STATE(39),
    [sym_member_expression] = STATE(39),
    [sym_call_expression] = STATE(39),
    [sym_t_call] = STATE(39),
    [sym_string] = STATE(39),
    [sym_boolean] = STATE(39),
    [sym_array] = STATE(39),
    [sym_object] = STATE(39),
    [sym_comment] = STATE(5),
    [aux_sym_fetch_block_repeat1] = STATE(3),
    [anon_sym_LPAREN] = ACTIONS(17),
    [anon_sym_LBRACE] = ACTIONS(19),
    [anon_sym_RBRACE] = ACTIONS(158),
    [anon_sym_state] = ACTIONS(23),
    [anon_sym_derived] = ACTIONS(25),
    [anon_sym_effect] = ACTIONS(27),
    [anon_sym_action] = ACTIONS(29),
    [anon_sym_use] = ACTIONS(31),
    [anon_sym_navigate] = ACTIONS(33),
    [sym_builtin_component] = ACTIONS(35),
    [anon_sym_if] = ACTIONS(37),
    [anon_sym_for] = ACTIONS(39),
    [anon_sym_show] = ACTIONS(41),
    [anon_sym_fetch] = ACTIONS(43),
    [anon_sym_loading] = ACTIONS(45),
    [anon_sym_error] = ACTIONS(47),
    [anon_sym_success] = ACTIONS(49),
    [sym_event_name] = ACTIONS(51),
    [anon_sym_style] = ACTIONS(53),
    [anon_sym_transition] = ACTIONS(55),
    [anon_sym_BANG] = ACTIONS(57),
    [anon_sym_t] = ACTIONS(59),
    [anon_sym_DQUOTE] = ACTIONS(61),
    [sym_number] = ACTIONS(63),
    [anon_sym_true] = ACTIONS(65),
    [anon_sym_false] = ACTIONS(65),
    [sym_null] = ACTIONS(67),
    [anon_sym_LBRACK] = ACTIONS(69),
    [sym_identifier] = ACTIONS(71),
    [anon_sym_SLASH_SLASH] = ACTIONS(3),
    [anon_sym_SLASH_STAR] = ACTIONS(5),
  },
  [6] = {
    [sym__statement] = STATE(71),
    [sym_state_declaration] = STATE(103),
    [sym_derived_declaration] = STATE(103),
    [sym_effect_declaration] = STATE(103),
    [sym_action_declaration] = STATE(103),
    [sym_use_declaration] = STATE(103),
    [sym_navigate_call] = STATE(103),
    [sym_assignment] = STATE(103),
    [sym_expression_statement] = STATE(103),
    [sym_ui_element] = STATE(103),
    [sym_component_name] = STATE(66),
    [sym_sub_component] = STATE(87),
    [sym_if_statement] = STATE(103),
    [sym_for_statement] = STATE(103),
    [sym_show_statement] = STATE(103),
    [sym_fetch_statement] = STATE(103),
    [sym_loading_block] = STATE(71),
    [sym_error_block] = STATE(71),
    [sym_success_block] = STATE(71),
    [sym_event_handler] = STATE(103),
    [sym_style_block] = STATE(103),
    [sym_transition_block] = STATE(103),
    [sym__expression] = STATE(34),
    [sym_binary_expression] = STATE(39),
    [sym_unary_expression] = STATE(39),
    [sym_member_expression] = STATE(39),
    [sym_call_expression] = STATE(39),
    [sym_t_call] = STATE(39),
    [sym_string] = STATE(39),
    [sym_boolean] = STATE(39),
    [sym_array] = STATE(39),
    [sym_object] = STATE(39),
    [sym_comment] = STATE(6),
    [aux_sym_fetch_block_repeat1] = STATE(5),
    [anon_sym_LPAREN] = ACTIONS(17),
    [anon_sym_LBRACE] = ACTIONS(19),
    [anon_sym_RBRACE] = ACTIONS(160),
    [anon_sym_state] = ACTIONS(23),
    [anon_sym_derived] = ACTIONS(25),
    [anon_sym_effect] = ACTIONS(27),
    [anon_sym_action] = ACTIONS(29),
    [anon_sym_use] = ACTIONS(31),
    [anon_sym_navigate] = ACTIONS(33),
    [sym_builtin_component] = ACTIONS(35),
    [anon_sym_if] = ACTIONS(37),
    [anon_sym_for] = ACTIONS(39),
    [anon_sym_show] = ACTIONS(41),
    [anon_sym_fetch] = ACTIONS(43),
    [anon_sym_loading] = ACTIONS(45),
    [anon_sym_error] = ACTIONS(47),
    [anon_sym_success] = ACTIONS(49),
    [sym_event_name] = ACTIONS(51),
    [anon_sym_style] = ACTIONS(53),
    [anon_sym_transition] = ACTIONS(55),
    [anon_sym_BANG] = ACTIONS(57),
    [anon_sym_t] = ACTIONS(59),
    [anon_sym_DQUOTE] = ACTIONS(61),
    [sym_number] = ACTIONS(63),
    [anon_sym_true] = ACTIONS(65),
    [anon_sym_false] = ACTIONS(65),
    [sym_null] = ACTIONS(67),
    [anon_sym_LBRACK] = ACTIONS(69),
    [sym_identifier] = ACTIONS(71),
    [anon_sym_SLASH_SLASH] = ACTIONS(3),
    [anon_sym_SLASH_STAR] = ACTIONS(5),
  },
};

static const uint16_t ts_small_parse_table[] = {
  [0] = 35,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(162), 1,
      anon_sym_LPAREN,
    ACTIONS(164), 1,
      anon_sym_LBRACE,
    ACTIONS(166), 1,
      anon_sym_RBRACE,
    ACTIONS(168), 1,
      anon_sym_state,
    ACTIONS(170), 1,
      anon_sym_derived,
    ACTIONS(172), 1,
      anon_sym_effect,
    ACTIONS(174), 1,
      anon_sym_action,
    ACTIONS(176), 1,
      anon_sym_use,
    ACTIONS(178), 1,
      anon_sym_navigate,
    ACTIONS(180), 1,
      sym_builtin_component,
    ACTIONS(182), 1,
      anon_sym_if,
    ACTIONS(184), 1,
      anon_sym_for,
    ACTIONS(186), 1,
      anon_sym_show,
    ACTIONS(188), 1,
      anon_sym_fetch,
    ACTIONS(190), 1,
      sym_event_name,
    ACTIONS(192), 1,
      anon_sym_style,
    ACTIONS(194), 1,
      anon_sym_transition,
    ACTIONS(196), 1,
      anon_sym_BANG,
    ACTIONS(198), 1,
      anon_sym_t,
    ACTIONS(200), 1,
      anon_sym_DQUOTE,
    ACTIONS(202), 1,
      sym_number,
    ACTIONS(206), 1,
      sym_null,
    ACTIONS(208), 1,
      anon_sym_LBRACK,
    ACTIONS(210), 1,
      sym_identifier,
    STATE(7), 1,
      sym_comment,
    STATE(8), 1,
      aux_sym_block_repeat1,
    STATE(63), 1,
      sym__expression,
    STATE(112), 1,
      sym_component_name,
    STATE(114), 1,
      sym__statement,
    STATE(143), 1,
      sym_sub_component,
    ACTIONS(204), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(20), 9,
      sym_binary_expression,
      sym_unary_expression,
      sym_member_expression,
      sym_call_expression,
      sym_t_call,
      sym_string,
      sym_boolean,
      sym_array,
      sym_object,
    STATE(138), 16,
      sym_state_declaration,
      sym_derived_declaration,
      sym_effect_declaration,
      sym_action_declaration,
      sym_use_declaration,
      sym_navigate_call,
      sym_assignment,
      sym_expression_statement,
      sym_ui_element,
      sym_if_statement,
      sym_for_statement,
      sym_show_statement,
      sym_fetch_statement,
      sym_event_handler,
      sym_style_block,
      sym_transition_block,
  [130] = 35,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(162), 1,
      anon_sym_LPAREN,
    ACTIONS(164), 1,
      anon_sym_LBRACE,
    ACTIONS(168), 1,
      anon_sym_state,
    ACTIONS(170), 1,
      anon_sym_derived,
    ACTIONS(172), 1,
      anon_sym_effect,
    ACTIONS(174), 1,
      anon_sym_action,
    ACTIONS(176), 1,
      anon_sym_use,
    ACTIONS(178), 1,
      anon_sym_navigate,
    ACTIONS(180), 1,
      sym_builtin_component,
    ACTIONS(182), 1,
      anon_sym_if,
    ACTIONS(184), 1,
      anon_sym_for,
    ACTIONS(186), 1,
      anon_sym_show,
    ACTIONS(188), 1,
      anon_sym_fetch,
    ACTIONS(190), 1,
      sym_event_name,
    ACTIONS(192), 1,
      anon_sym_style,
    ACTIONS(194), 1,
      anon_sym_transition,
    ACTIONS(196), 1,
      anon_sym_BANG,
    ACTIONS(198), 1,
      anon_sym_t,
    ACTIONS(200), 1,
      anon_sym_DQUOTE,
    ACTIONS(202), 1,
      sym_number,
    ACTIONS(206), 1,
      sym_null,
    ACTIONS(208), 1,
      anon_sym_LBRACK,
    ACTIONS(210), 1,
      sym_identifier,
    ACTIONS(212), 1,
      anon_sym_RBRACE,
    STATE(8), 1,
      sym_comment,
    STATE(9), 1,
      aux_sym_block_repeat1,
    STATE(63), 1,
      sym__expression,
    STATE(112), 1,
      sym_component_name,
    STATE(114), 1,
      sym__statement,
    STATE(143), 1,
      sym_sub_component,
    ACTIONS(204), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(20), 9,
      sym_binary_expression,
      sym_unary_expression,
      sym_member_expression,
      sym_call_expression,
      sym_t_call,
      sym_string,
      sym_boolean,
      sym_array,
      sym_object,
    STATE(138), 16,
      sym_state_declaration,
      sym_derived_declaration,
      sym_effect_declaration,
      sym_action_declaration,
      sym_use_declaration,
      sym_navigate_call,
      sym_assignment,
      sym_expression_statement,
      sym_ui_element,
      sym_if_statement,
      sym_for_statement,
      sym_show_statement,
      sym_fetch_statement,
      sym_event_handler,
      sym_style_block,
      sym_transition_block,
  [260] = 34,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(214), 1,
      anon_sym_LPAREN,
    ACTIONS(217), 1,
      anon_sym_LBRACE,
    ACTIONS(220), 1,
      anon_sym_RBRACE,
    ACTIONS(222), 1,
      anon_sym_state,
    ACTIONS(225), 1,
      anon_sym_derived,
    ACTIONS(228), 1,
      anon_sym_effect,
    ACTIONS(231), 1,
      anon_sym_action,
    ACTIONS(234), 1,
      anon_sym_use,
    ACTIONS(237), 1,
      anon_sym_navigate,
    ACTIONS(240), 1,
      sym_builtin_component,
    ACTIONS(243), 1,
      anon_sym_if,
    ACTIONS(246), 1,
      anon_sym_for,
    ACTIONS(249), 1,
      anon_sym_show,
    ACTIONS(252), 1,
      anon_sym_fetch,
    ACTIONS(255), 1,
      sym_event_name,
    ACTIONS(258), 1,
      anon_sym_style,
    ACTIONS(261), 1,
      anon_sym_transition,
    ACTIONS(264), 1,
      anon_sym_BANG,
    ACTIONS(267), 1,
      anon_sym_t,
    ACTIONS(270), 1,
      anon_sym_DQUOTE,
    ACTIONS(273), 1,
      sym_number,
    ACTIONS(279), 1,
      sym_null,
    ACTIONS(282), 1,
      anon_sym_LBRACK,
    ACTIONS(285), 1,
      sym_identifier,
    STATE(63), 1,
      sym__expression,
    STATE(112), 1,
      sym_component_name,
    STATE(114), 1,
      sym__statement,
    STATE(143), 1,
      sym_sub_component,
    ACTIONS(276), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(9), 2,
      sym_comment,
      aux_sym_block_repeat1,
    STATE(20), 9,
      sym_binary_expression,
      sym_unary_expression,
      sym_member_expression,
      sym_call_expression,
      sym_t_call,
      sym_string,
      sym_boolean,
      sym_array,
      sym_object,
    STATE(138), 16,
      sym_state_declaration,
      sym_derived_declaration,
      sym_effect_declaration,
      sym_action_declaration,
      sym_use_declaration,
      sym_navigate_call,
      sym_assignment,
      sym_expression_statement,
      sym_ui_element,
      sym_if_statement,
      sym_for_statement,
      sym_show_statement,
      sym_fetch_statement,
      sym_event_handler,
      sym_style_block,
      sym_transition_block,
  [388] = 35,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(162), 1,
      anon_sym_LPAREN,
    ACTIONS(164), 1,
      anon_sym_LBRACE,
    ACTIONS(168), 1,
      anon_sym_state,
    ACTIONS(170), 1,
      anon_sym_derived,
    ACTIONS(172), 1,
      anon_sym_effect,
    ACTIONS(174), 1,
      anon_sym_action,
    ACTIONS(176), 1,
      anon_sym_use,
    ACTIONS(178), 1,
      anon_sym_navigate,
    ACTIONS(180), 1,
      sym_builtin_component,
    ACTIONS(182), 1,
      anon_sym_if,
    ACTIONS(184), 1,
      anon_sym_for,
    ACTIONS(186), 1,
      anon_sym_show,
    ACTIONS(188), 1,
      anon_sym_fetch,
    ACTIONS(190), 1,
      sym_event_name,
    ACTIONS(192), 1,
      anon_sym_style,
    ACTIONS(194), 1,
      anon_sym_transition,
    ACTIONS(196), 1,
      anon_sym_BANG,
    ACTIONS(198), 1,
      anon_sym_t,
    ACTIONS(200), 1,
      anon_sym_DQUOTE,
    ACTIONS(202), 1,
      sym_number,
    ACTIONS(206), 1,
      sym_null,
    ACTIONS(208), 1,
      anon_sym_LBRACK,
    ACTIONS(210), 1,
      sym_identifier,
    ACTIONS(288), 1,
      anon_sym_RBRACE,
    STATE(9), 1,
      aux_sym_block_repeat1,
    STATE(10), 1,
      sym_comment,
    STATE(63), 1,
      sym__expression,
    STATE(112), 1,
      sym_component_name,
    STATE(114), 1,
      sym__statement,
    STATE(143), 1,
      sym_sub_component,
    ACTIONS(204), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(20), 9,
      sym_binary_expression,
      sym_unary_expression,
      sym_member_expression,
      sym_call_expression,
      sym_t_call,
      sym_string,
      sym_boolean,
      sym_array,
      sym_object,
    STATE(138), 16,
      sym_state_declaration,
      sym_derived_declaration,
      sym_effect_declaration,
      sym_action_declaration,
      sym_use_declaration,
      sym_navigate_call,
      sym_assignment,
      sym_expression_statement,
      sym_ui_element,
      sym_if_statement,
      sym_for_statement,
      sym_show_statement,
      sym_fetch_statement,
      sym_event_handler,
      sym_style_block,
      sym_transition_block,
  [518] = 35,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(162), 1,
      anon_sym_LPAREN,
    ACTIONS(164), 1,
      anon_sym_LBRACE,
    ACTIONS(168), 1,
      anon_sym_state,
    ACTIONS(170), 1,
      anon_sym_derived,
    ACTIONS(172), 1,
      anon_sym_effect,
    ACTIONS(174), 1,
      anon_sym_action,
    ACTIONS(176), 1,
      anon_sym_use,
    ACTIONS(178), 1,
      anon_sym_navigate,
    ACTIONS(180), 1,
      sym_builtin_component,
    ACTIONS(182), 1,
      anon_sym_if,
    ACTIONS(184), 1,
      anon_sym_for,
    ACTIONS(186), 1,
      anon_sym_show,
    ACTIONS(188), 1,
      anon_sym_fetch,
    ACTIONS(190), 1,
      sym_event_name,
    ACTIONS(192), 1,
      anon_sym_style,
    ACTIONS(194), 1,
      anon_sym_transition,
    ACTIONS(196), 1,
      anon_sym_BANG,
    ACTIONS(198), 1,
      anon_sym_t,
    ACTIONS(200), 1,
      anon_sym_DQUOTE,
    ACTIONS(202), 1,
      sym_number,
    ACTIONS(206), 1,
      sym_null,
    ACTIONS(208), 1,
      anon_sym_LBRACK,
    ACTIONS(210), 1,
      sym_identifier,
    ACTIONS(290), 1,
      anon_sym_RBRACE,
    STATE(10), 1,
      aux_sym_block_repeat1,
    STATE(11), 1,
      sym_comment,
    STATE(63), 1,
      sym__expression,
    STATE(112), 1,
      sym_component_name,
    STATE(114), 1,
      sym__statement,
    STATE(143), 1,
      sym_sub_component,
    ACTIONS(204), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(20), 9,
      sym_binary_expression,
      sym_unary_expression,
      sym_member_expression,
      sym_call_expression,
      sym_t_call,
      sym_string,
      sym_boolean,
      sym_array,
      sym_object,
    STATE(138), 16,
      sym_state_declaration,
      sym_derived_declaration,
      sym_effect_declaration,
      sym_action_declaration,
      sym_use_declaration,
      sym_navigate_call,
      sym_assignment,
      sym_expression_statement,
      sym_ui_element,
      sym_if_statement,
      sym_for_statement,
      sym_show_statement,
      sym_fetch_statement,
      sym_event_handler,
      sym_style_block,
      sym_transition_block,
  [648] = 35,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(162), 1,
      anon_sym_LPAREN,
    ACTIONS(164), 1,
      anon_sym_LBRACE,
    ACTIONS(168), 1,
      anon_sym_state,
    ACTIONS(170), 1,
      anon_sym_derived,
    ACTIONS(172), 1,
      anon_sym_effect,
    ACTIONS(174), 1,
      anon_sym_action,
    ACTIONS(176), 1,
      anon_sym_use,
    ACTIONS(178), 1,
      anon_sym_navigate,
    ACTIONS(180), 1,
      sym_builtin_component,
    ACTIONS(182), 1,
      anon_sym_if,
    ACTIONS(184), 1,
      anon_sym_for,
    ACTIONS(186), 1,
      anon_sym_show,
    ACTIONS(188), 1,
      anon_sym_fetch,
    ACTIONS(190), 1,
      sym_event_name,
    ACTIONS(192), 1,
      anon_sym_style,
    ACTIONS(194), 1,
      anon_sym_transition,
    ACTIONS(196), 1,
      anon_sym_BANG,
    ACTIONS(198), 1,
      anon_sym_t,
    ACTIONS(200), 1,
      anon_sym_DQUOTE,
    ACTIONS(202), 1,
      sym_number,
    ACTIONS(206), 1,
      sym_null,
    ACTIONS(208), 1,
      anon_sym_LBRACK,
    ACTIONS(210), 1,
      sym_identifier,
    ACTIONS(292), 1,
      anon_sym_RBRACE,
    STATE(12), 1,
      sym_comment,
    STATE(13), 1,
      aux_sym_block_repeat1,
    STATE(63), 1,
      sym__expression,
    STATE(112), 1,
      sym_component_name,
    STATE(114), 1,
      sym__statement,
    STATE(143), 1,
      sym_sub_component,
    ACTIONS(204), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(20), 9,
      sym_binary_expression,
      sym_unary_expression,
      sym_member_expression,
      sym_call_expression,
      sym_t_call,
      sym_string,
      sym_boolean,
      sym_array,
      sym_object,
    STATE(138), 16,
      sym_state_declaration,
      sym_derived_declaration,
      sym_effect_declaration,
      sym_action_declaration,
      sym_use_declaration,
      sym_navigate_call,
      sym_assignment,
      sym_expression_statement,
      sym_ui_element,
      sym_if_statement,
      sym_for_statement,
      sym_show_statement,
      sym_fetch_statement,
      sym_event_handler,
      sym_style_block,
      sym_transition_block,
  [778] = 35,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(162), 1,
      anon_sym_LPAREN,
    ACTIONS(164), 1,
      anon_sym_LBRACE,
    ACTIONS(168), 1,
      anon_sym_state,
    ACTIONS(170), 1,
      anon_sym_derived,
    ACTIONS(172), 1,
      anon_sym_effect,
    ACTIONS(174), 1,
      anon_sym_action,
    ACTIONS(176), 1,
      anon_sym_use,
    ACTIONS(178), 1,
      anon_sym_navigate,
    ACTIONS(180), 1,
      sym_builtin_component,
    ACTIONS(182), 1,
      anon_sym_if,
    ACTIONS(184), 1,
      anon_sym_for,
    ACTIONS(186), 1,
      anon_sym_show,
    ACTIONS(188), 1,
      anon_sym_fetch,
    ACTIONS(190), 1,
      sym_event_name,
    ACTIONS(192), 1,
      anon_sym_style,
    ACTIONS(194), 1,
      anon_sym_transition,
    ACTIONS(196), 1,
      anon_sym_BANG,
    ACTIONS(198), 1,
      anon_sym_t,
    ACTIONS(200), 1,
      anon_sym_DQUOTE,
    ACTIONS(202), 1,
      sym_number,
    ACTIONS(206), 1,
      sym_null,
    ACTIONS(208), 1,
      anon_sym_LBRACK,
    ACTIONS(210), 1,
      sym_identifier,
    ACTIONS(294), 1,
      anon_sym_RBRACE,
    STATE(9), 1,
      aux_sym_block_repeat1,
    STATE(13), 1,
      sym_comment,
    STATE(63), 1,
      sym__expression,
    STATE(112), 1,
      sym_component_name,
    STATE(114), 1,
      sym__statement,
    STATE(143), 1,
      sym_sub_component,
    ACTIONS(204), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(20), 9,
      sym_binary_expression,
      sym_unary_expression,
      sym_member_expression,
      sym_call_expression,
      sym_t_call,
      sym_string,
      sym_boolean,
      sym_array,
      sym_object,
    STATE(138), 16,
      sym_state_declaration,
      sym_derived_declaration,
      sym_effect_declaration,
      sym_action_declaration,
      sym_use_declaration,
      sym_navigate_call,
      sym_assignment,
      sym_expression_statement,
      sym_ui_element,
      sym_if_statement,
      sym_for_statement,
      sym_show_statement,
      sym_fetch_statement,
      sym_event_handler,
      sym_style_block,
      sym_transition_block,
  [908] = 35,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(162), 1,
      anon_sym_LPAREN,
    ACTIONS(164), 1,
      anon_sym_LBRACE,
    ACTIONS(168), 1,
      anon_sym_state,
    ACTIONS(170), 1,
      anon_sym_derived,
    ACTIONS(172), 1,
      anon_sym_effect,
    ACTIONS(174), 1,
      anon_sym_action,
    ACTIONS(176), 1,
      anon_sym_use,
    ACTIONS(178), 1,
      anon_sym_navigate,
    ACTIONS(180), 1,
      sym_builtin_component,
    ACTIONS(182), 1,
      anon_sym_if,
    ACTIONS(184), 1,
      anon_sym_for,
    ACTIONS(186), 1,
      anon_sym_show,
    ACTIONS(188), 1,
      anon_sym_fetch,
    ACTIONS(190), 1,
      sym_event_name,
    ACTIONS(192), 1,
      anon_sym_style,
    ACTIONS(194), 1,
      anon_sym_transition,
    ACTIONS(196), 1,
      anon_sym_BANG,
    ACTIONS(198), 1,
      anon_sym_t,
    ACTIONS(200), 1,
      anon_sym_DQUOTE,
    ACTIONS(202), 1,
      sym_number,
    ACTIONS(206), 1,
      sym_null,
    ACTIONS(208), 1,
      anon_sym_LBRACK,
    ACTIONS(210), 1,
      sym_identifier,
    ACTIONS(296), 1,
      anon_sym_RBRACE,
    STATE(14), 1,
      sym_comment,
    STATE(15), 1,
      aux_sym_block_repeat1,
    STATE(63), 1,
      sym__expression,
    STATE(112), 1,
      sym_component_name,
    STATE(114), 1,
      sym__statement,
    STATE(143), 1,
      sym_sub_component,
    ACTIONS(204), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(20), 9,
      sym_binary_expression,
      sym_unary_expression,
      sym_member_expression,
      sym_call_expression,
      sym_t_call,
      sym_string,
      sym_boolean,
      sym_array,
      sym_object,
    STATE(138), 16,
      sym_state_declaration,
      sym_derived_declaration,
      sym_effect_declaration,
      sym_action_declaration,
      sym_use_declaration,
      sym_navigate_call,
      sym_assignment,
      sym_expression_statement,
      sym_ui_element,
      sym_if_statement,
      sym_for_statement,
      sym_show_statement,
      sym_fetch_statement,
      sym_event_handler,
      sym_style_block,
      sym_transition_block,
  [1038] = 35,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(162), 1,
      anon_sym_LPAREN,
    ACTIONS(164), 1,
      anon_sym_LBRACE,
    ACTIONS(168), 1,
      anon_sym_state,
    ACTIONS(170), 1,
      anon_sym_derived,
    ACTIONS(172), 1,
      anon_sym_effect,
    ACTIONS(174), 1,
      anon_sym_action,
    ACTIONS(176), 1,
      anon_sym_use,
    ACTIONS(178), 1,
      anon_sym_navigate,
    ACTIONS(180), 1,
      sym_builtin_component,
    ACTIONS(182), 1,
      anon_sym_if,
    ACTIONS(184), 1,
      anon_sym_for,
    ACTIONS(186), 1,
      anon_sym_show,
    ACTIONS(188), 1,
      anon_sym_fetch,
    ACTIONS(190), 1,
      sym_event_name,
    ACTIONS(192), 1,
      anon_sym_style,
    ACTIONS(194), 1,
      anon_sym_transition,
    ACTIONS(196), 1,
      anon_sym_BANG,
    ACTIONS(198), 1,
      anon_sym_t,
    ACTIONS(200), 1,
      anon_sym_DQUOTE,
    ACTIONS(202), 1,
      sym_number,
    ACTIONS(206), 1,
      sym_null,
    ACTIONS(208), 1,
      anon_sym_LBRACK,
    ACTIONS(210), 1,
      sym_identifier,
    ACTIONS(298), 1,
      anon_sym_RBRACE,
    STATE(9), 1,
      aux_sym_block_repeat1,
    STATE(15), 1,
      sym_comment,
    STATE(63), 1,
      sym__expression,
    STATE(112), 1,
      sym_component_name,
    STATE(114), 1,
      sym__statement,
    STATE(143), 1,
      sym_sub_component,
    ACTIONS(204), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(20), 9,
      sym_binary_expression,
      sym_unary_expression,
      sym_member_expression,
      sym_call_expression,
      sym_t_call,
      sym_string,
      sym_boolean,
      sym_array,
      sym_object,
    STATE(138), 16,
      sym_state_declaration,
      sym_derived_declaration,
      sym_effect_declaration,
      sym_action_declaration,
      sym_use_declaration,
      sym_navigate_call,
      sym_assignment,
      sym_expression_statement,
      sym_ui_element,
      sym_if_statement,
      sym_for_statement,
      sym_show_statement,
      sym_fetch_statement,
      sym_event_handler,
      sym_style_block,
      sym_transition_block,
  [1168] = 35,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(162), 1,
      anon_sym_LPAREN,
    ACTIONS(164), 1,
      anon_sym_LBRACE,
    ACTIONS(168), 1,
      anon_sym_state,
    ACTIONS(170), 1,
      anon_sym_derived,
    ACTIONS(172), 1,
      anon_sym_effect,
    ACTIONS(174), 1,
      anon_sym_action,
    ACTIONS(176), 1,
      anon_sym_use,
    ACTIONS(178), 1,
      anon_sym_navigate,
    ACTIONS(180), 1,
      sym_builtin_component,
    ACTIONS(182), 1,
      anon_sym_if,
    ACTIONS(184), 1,
      anon_sym_for,
    ACTIONS(186), 1,
      anon_sym_show,
    ACTIONS(188), 1,
      anon_sym_fetch,
    ACTIONS(190), 1,
      sym_event_name,
    ACTIONS(192), 1,
      anon_sym_style,
    ACTIONS(194), 1,
      anon_sym_transition,
    ACTIONS(196), 1,
      anon_sym_BANG,
    ACTIONS(198), 1,
      anon_sym_t,
    ACTIONS(200), 1,
      anon_sym_DQUOTE,
    ACTIONS(202), 1,
      sym_number,
    ACTIONS(206), 1,
      sym_null,
    ACTIONS(208), 1,
      anon_sym_LBRACK,
    ACTIONS(210), 1,
      sym_identifier,
    ACTIONS(300), 1,
      anon_sym_RBRACE,
    STATE(16), 1,
      sym_comment,
    STATE(17), 1,
      aux_sym_block_repeat1,
    STATE(63), 1,
      sym__expression,
    STATE(112), 1,
      sym_component_name,
    STATE(114), 1,
      sym__statement,
    STATE(143), 1,
      sym_sub_component,
    ACTIONS(204), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(20), 9,
      sym_binary_expression,
      sym_unary_expression,
      sym_member_expression,
      sym_call_expression,
      sym_t_call,
      sym_string,
      sym_boolean,
      sym_array,
      sym_object,
    STATE(138), 16,
      sym_state_declaration,
      sym_derived_declaration,
      sym_effect_declaration,
      sym_action_declaration,
      sym_use_declaration,
      sym_navigate_call,
      sym_assignment,
      sym_expression_statement,
      sym_ui_element,
      sym_if_statement,
      sym_for_statement,
      sym_show_statement,
      sym_fetch_statement,
      sym_event_handler,
      sym_style_block,
      sym_transition_block,
  [1298] = 35,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(162), 1,
      anon_sym_LPAREN,
    ACTIONS(164), 1,
      anon_sym_LBRACE,
    ACTIONS(168), 1,
      anon_sym_state,
    ACTIONS(170), 1,
      anon_sym_derived,
    ACTIONS(172), 1,
      anon_sym_effect,
    ACTIONS(174), 1,
      anon_sym_action,
    ACTIONS(176), 1,
      anon_sym_use,
    ACTIONS(178), 1,
      anon_sym_navigate,
    ACTIONS(180), 1,
      sym_builtin_component,
    ACTIONS(182), 1,
      anon_sym_if,
    ACTIONS(184), 1,
      anon_sym_for,
    ACTIONS(186), 1,
      anon_sym_show,
    ACTIONS(188), 1,
      anon_sym_fetch,
    ACTIONS(190), 1,
      sym_event_name,
    ACTIONS(192), 1,
      anon_sym_style,
    ACTIONS(194), 1,
      anon_sym_transition,
    ACTIONS(196), 1,
      anon_sym_BANG,
    ACTIONS(198), 1,
      anon_sym_t,
    ACTIONS(200), 1,
      anon_sym_DQUOTE,
    ACTIONS(202), 1,
      sym_number,
    ACTIONS(206), 1,
      sym_null,
    ACTIONS(208), 1,
      anon_sym_LBRACK,
    ACTIONS(210), 1,
      sym_identifier,
    ACTIONS(302), 1,
      anon_sym_RBRACE,
    STATE(9), 1,
      aux_sym_block_repeat1,
    STATE(17), 1,
      sym_comment,
    STATE(63), 1,
      sym__expression,
    STATE(112), 1,
      sym_component_name,
    STATE(114), 1,
      sym__statement,
    STATE(143), 1,
      sym_sub_component,
    ACTIONS(204), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(20), 9,
      sym_binary_expression,
      sym_unary_expression,
      sym_member_expression,
      sym_call_expression,
      sym_t_call,
      sym_string,
      sym_boolean,
      sym_array,
      sym_object,
    STATE(138), 16,
      sym_state_declaration,
      sym_derived_declaration,
      sym_effect_declaration,
      sym_action_declaration,
      sym_use_declaration,
      sym_navigate_call,
      sym_assignment,
      sym_expression_statement,
      sym_ui_element,
      sym_if_statement,
      sym_for_statement,
      sym_show_statement,
      sym_fetch_statement,
      sym_event_handler,
      sym_style_block,
      sym_transition_block,
  [1428] = 9,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(307), 1,
      anon_sym_EQ,
    ACTIONS(312), 1,
      anon_sym_DOT,
    STATE(18), 1,
      sym_comment,
    ACTIONS(317), 3,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
    ACTIONS(304), 7,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(315), 10,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
    ACTIONS(309), 22,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_loading,
      anon_sym_error,
      anon_sym_success,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_BANG,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [1494] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(19), 1,
      sym_comment,
    ACTIONS(319), 21,
      anon_sym_LPAREN,
      anon_sym_RPAREN,
      anon_sym_COMMA,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      anon_sym_DOT,
      sym_event_name,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
      anon_sym_RBRACK,
    ACTIONS(321), 22,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
      anon_sym_BANG,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [1551] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(20), 1,
      sym_comment,
    ACTIONS(315), 21,
      anon_sym_LPAREN,
      anon_sym_RPAREN,
      anon_sym_COMMA,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      anon_sym_DOT,
      sym_event_name,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
      anon_sym_RBRACK,
    ACTIONS(317), 22,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
      anon_sym_BANG,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [1608] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(21), 1,
      sym_comment,
    ACTIONS(323), 21,
      anon_sym_LPAREN,
      anon_sym_RPAREN,
      anon_sym_COMMA,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      anon_sym_DOT,
      sym_event_name,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
      anon_sym_RBRACK,
    ACTIONS(325), 22,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
      anon_sym_BANG,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [1665] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(22), 1,
      sym_comment,
    ACTIONS(327), 21,
      anon_sym_LPAREN,
      anon_sym_RPAREN,
      anon_sym_COMMA,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      anon_sym_DOT,
      sym_event_name,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
      anon_sym_RBRACK,
    ACTIONS(329), 22,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
      anon_sym_BANG,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [1722] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(23), 1,
      sym_comment,
    ACTIONS(331), 21,
      anon_sym_LPAREN,
      anon_sym_RPAREN,
      anon_sym_COMMA,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      anon_sym_DOT,
      sym_event_name,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
      anon_sym_RBRACK,
    ACTIONS(333), 22,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
      anon_sym_BANG,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [1779] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(24), 1,
      sym_comment,
    ACTIONS(335), 21,
      anon_sym_LPAREN,
      anon_sym_RPAREN,
      anon_sym_COMMA,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      anon_sym_DOT,
      sym_event_name,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
      anon_sym_RBRACK,
    ACTIONS(337), 22,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
      anon_sym_BANG,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [1836] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(25), 1,
      sym_comment,
    ACTIONS(339), 21,
      anon_sym_LPAREN,
      anon_sym_RPAREN,
      anon_sym_COMMA,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      anon_sym_DOT,
      sym_event_name,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
      anon_sym_RBRACK,
    ACTIONS(341), 22,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
      anon_sym_BANG,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [1893] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(26), 1,
      sym_comment,
    ACTIONS(343), 21,
      anon_sym_LPAREN,
      anon_sym_RPAREN,
      anon_sym_COMMA,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      anon_sym_DOT,
      sym_event_name,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
      anon_sym_RBRACK,
    ACTIONS(345), 22,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
      anon_sym_BANG,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [1950] = 7,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(347), 1,
      anon_sym_LPAREN,
    ACTIONS(353), 1,
      anon_sym_DOT,
    STATE(27), 1,
      sym_comment,
    ACTIONS(349), 19,
      anon_sym_RPAREN,
      anon_sym_COMMA,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
      anon_sym_RBRACK,
    ACTIONS(351), 22,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
      anon_sym_BANG,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [2011] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(28), 1,
      sym_comment,
    ACTIONS(355), 21,
      anon_sym_LPAREN,
      anon_sym_RPAREN,
      anon_sym_COMMA,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      anon_sym_DOT,
      sym_event_name,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
      anon_sym_RBRACK,
    ACTIONS(357), 22,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
      anon_sym_BANG,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [2068] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(29), 1,
      sym_comment,
    ACTIONS(359), 21,
      anon_sym_LPAREN,
      anon_sym_RPAREN,
      anon_sym_COMMA,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      anon_sym_DOT,
      sym_event_name,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
      anon_sym_RBRACK,
    ACTIONS(361), 22,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
      anon_sym_BANG,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [2125] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(30), 1,
      sym_comment,
    ACTIONS(363), 21,
      anon_sym_LPAREN,
      anon_sym_RPAREN,
      anon_sym_COMMA,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      anon_sym_DOT,
      sym_event_name,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
      anon_sym_RBRACK,
    ACTIONS(365), 22,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
      anon_sym_BANG,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [2182] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(31), 1,
      sym_comment,
    ACTIONS(367), 21,
      anon_sym_LPAREN,
      anon_sym_RPAREN,
      anon_sym_COMMA,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      anon_sym_DOT,
      sym_event_name,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
      anon_sym_RBRACK,
    ACTIONS(369), 22,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
      anon_sym_BANG,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [2239] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(32), 1,
      sym_comment,
    ACTIONS(371), 21,
      anon_sym_LPAREN,
      anon_sym_RPAREN,
      anon_sym_COMMA,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      anon_sym_DOT,
      sym_event_name,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
      anon_sym_RBRACK,
    ACTIONS(373), 22,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
      anon_sym_BANG,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [2296] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(33), 1,
      sym_comment,
    ACTIONS(375), 21,
      anon_sym_LPAREN,
      anon_sym_RPAREN,
      anon_sym_COMMA,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      anon_sym_DOT,
      sym_event_name,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
      anon_sym_RBRACK,
    ACTIONS(377), 22,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
      anon_sym_BANG,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [2353] = 9,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(379), 1,
      anon_sym_LPAREN,
    ACTIONS(385), 1,
      anon_sym_DOT,
    STATE(34), 1,
      sym_comment,
    ACTIONS(389), 3,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
    ACTIONS(381), 6,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(387), 10,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
    ACTIONS(383), 22,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_loading,
      anon_sym_error,
      anon_sym_success,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_BANG,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [2418] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(35), 1,
      sym_comment,
    ACTIONS(391), 21,
      anon_sym_LPAREN,
      anon_sym_RPAREN,
      anon_sym_COMMA,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      anon_sym_DOT,
      sym_event_name,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
      anon_sym_RBRACK,
    ACTIONS(393), 22,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
      anon_sym_BANG,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [2475] = 9,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(379), 1,
      anon_sym_LPAREN,
    ACTIONS(385), 1,
      anon_sym_DOT,
    STATE(36), 1,
      sym_comment,
    ACTIONS(389), 3,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
    ACTIONS(395), 6,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(387), 10,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
    ACTIONS(397), 22,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_loading,
      anon_sym_error,
      anon_sym_success,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_BANG,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [2540] = 9,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(379), 1,
      anon_sym_LPAREN,
    ACTIONS(385), 1,
      anon_sym_DOT,
    STATE(37), 1,
      sym_comment,
    ACTIONS(389), 3,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
    ACTIONS(399), 6,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(387), 10,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
    ACTIONS(401), 22,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_loading,
      anon_sym_error,
      anon_sym_success,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_BANG,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [2605] = 9,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(379), 1,
      anon_sym_LPAREN,
    ACTIONS(385), 1,
      anon_sym_DOT,
    STATE(38), 1,
      sym_comment,
    ACTIONS(389), 3,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
    ACTIONS(403), 6,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(387), 10,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
    ACTIONS(405), 22,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_loading,
      anon_sym_error,
      anon_sym_success,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_BANG,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [2670] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(39), 1,
      sym_comment,
    ACTIONS(315), 18,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      anon_sym_DOT,
      sym_event_name,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(317), 25,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_loading,
      anon_sym_error,
      anon_sym_success,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
      anon_sym_BANG,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [2727] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(40), 1,
      sym_comment,
    ACTIONS(371), 18,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      anon_sym_DOT,
      sym_event_name,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(373), 25,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_loading,
      anon_sym_error,
      anon_sym_success,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
      anon_sym_BANG,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [2784] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(41), 1,
      sym_comment,
    ACTIONS(319), 18,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      anon_sym_DOT,
      sym_event_name,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(321), 25,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_loading,
      anon_sym_error,
      anon_sym_success,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
      anon_sym_BANG,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [2841] = 7,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(379), 1,
      anon_sym_LPAREN,
    ACTIONS(385), 1,
      anon_sym_DOT,
    STATE(42), 1,
      sym_comment,
    ACTIONS(407), 16,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(409), 25,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_loading,
      anon_sym_error,
      anon_sym_success,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
      anon_sym_BANG,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [2902] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(43), 1,
      sym_comment,
    ACTIONS(391), 18,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      anon_sym_DOT,
      sym_event_name,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(393), 25,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_loading,
      anon_sym_error,
      anon_sym_success,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
      anon_sym_BANG,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [2959] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(44), 1,
      sym_comment,
    ACTIONS(411), 18,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      anon_sym_DOT,
      sym_event_name,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(413), 25,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_loading,
      anon_sym_error,
      anon_sym_success,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
      anon_sym_BANG,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [3016] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(45), 1,
      sym_comment,
    ACTIONS(323), 18,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      anon_sym_DOT,
      sym_event_name,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(325), 25,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_loading,
      anon_sym_error,
      anon_sym_success,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
      anon_sym_BANG,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [3073] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(46), 1,
      sym_comment,
    ACTIONS(327), 18,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      anon_sym_DOT,
      sym_event_name,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(329), 25,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_loading,
      anon_sym_error,
      anon_sym_success,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
      anon_sym_BANG,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [3130] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(47), 1,
      sym_comment,
    ACTIONS(331), 18,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      anon_sym_DOT,
      sym_event_name,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(333), 25,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_loading,
      anon_sym_error,
      anon_sym_success,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
      anon_sym_BANG,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [3187] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(48), 1,
      sym_comment,
    ACTIONS(335), 18,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      anon_sym_DOT,
      sym_event_name,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(337), 25,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_loading,
      anon_sym_error,
      anon_sym_success,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
      anon_sym_BANG,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [3244] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(49), 1,
      sym_comment,
    ACTIONS(339), 18,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      anon_sym_DOT,
      sym_event_name,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(341), 25,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_loading,
      anon_sym_error,
      anon_sym_success,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
      anon_sym_BANG,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [3301] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(50), 1,
      sym_comment,
    ACTIONS(343), 18,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      anon_sym_DOT,
      sym_event_name,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(345), 25,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_loading,
      anon_sym_error,
      anon_sym_success,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
      anon_sym_BANG,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [3358] = 7,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(379), 1,
      anon_sym_LPAREN,
    ACTIONS(385), 1,
      anon_sym_DOT,
    STATE(51), 1,
      sym_comment,
    ACTIONS(349), 16,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(351), 25,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_loading,
      anon_sym_error,
      anon_sym_success,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
      anon_sym_BANG,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [3419] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(52), 1,
      sym_comment,
    ACTIONS(355), 18,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      anon_sym_DOT,
      sym_event_name,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(357), 25,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_loading,
      anon_sym_error,
      anon_sym_success,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
      anon_sym_BANG,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [3476] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(53), 1,
      sym_comment,
    ACTIONS(359), 18,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      anon_sym_DOT,
      sym_event_name,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(361), 25,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_loading,
      anon_sym_error,
      anon_sym_success,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
      anon_sym_BANG,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [3533] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(54), 1,
      sym_comment,
    ACTIONS(363), 18,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      anon_sym_DOT,
      sym_event_name,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(365), 25,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_loading,
      anon_sym_error,
      anon_sym_success,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
      anon_sym_BANG,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [3590] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(55), 1,
      sym_comment,
    ACTIONS(367), 18,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      anon_sym_DOT,
      sym_event_name,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(369), 25,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_loading,
      anon_sym_error,
      anon_sym_success,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
      anon_sym_BANG,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [3647] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(56), 1,
      sym_comment,
    ACTIONS(375), 18,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      anon_sym_DOT,
      sym_event_name,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(377), 25,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_loading,
      anon_sym_error,
      anon_sym_success,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
      anon_sym_BANG,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [3704] = 7,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(347), 1,
      anon_sym_LPAREN,
    ACTIONS(353), 1,
      anon_sym_DOT,
    STATE(57), 1,
      sym_comment,
    ACTIONS(407), 19,
      anon_sym_RPAREN,
      anon_sym_COMMA,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
      anon_sym_RBRACK,
    ACTIONS(409), 22,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
      anon_sym_BANG,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [3765] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(58), 1,
      sym_comment,
    ACTIONS(411), 21,
      anon_sym_LPAREN,
      anon_sym_RPAREN,
      anon_sym_COMMA,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      anon_sym_DOT,
      sym_event_name,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
      anon_sym_RBRACK,
    ACTIONS(413), 22,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
      anon_sym_BANG,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [3822] = 9,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(415), 1,
      anon_sym_EQ,
    ACTIONS(417), 1,
      anon_sym_DOT,
    STATE(59), 1,
      sym_comment,
    ACTIONS(317), 3,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
    ACTIONS(304), 7,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(315), 10,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
    ACTIONS(309), 19,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_BANG,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [3885] = 9,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(347), 1,
      anon_sym_LPAREN,
    ACTIONS(353), 1,
      anon_sym_DOT,
    STATE(60), 1,
      sym_comment,
    ACTIONS(422), 3,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
    ACTIONS(403), 6,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(420), 10,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
    ACTIONS(405), 19,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_BANG,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [3947] = 9,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(347), 1,
      anon_sym_LPAREN,
    ACTIONS(353), 1,
      anon_sym_DOT,
    STATE(61), 1,
      sym_comment,
    ACTIONS(422), 3,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
    ACTIONS(395), 6,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(420), 10,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
    ACTIONS(397), 19,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_BANG,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [4009] = 9,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(347), 1,
      anon_sym_LPAREN,
    ACTIONS(353), 1,
      anon_sym_DOT,
    STATE(62), 1,
      sym_comment,
    ACTIONS(422), 3,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
    ACTIONS(399), 6,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(420), 10,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
    ACTIONS(401), 19,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_BANG,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [4071] = 9,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(347), 1,
      anon_sym_LPAREN,
    ACTIONS(353), 1,
      anon_sym_DOT,
    STATE(63), 1,
      sym_comment,
    ACTIONS(422), 3,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
    ACTIONS(381), 6,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(420), 10,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
    ACTIONS(383), 19,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_BANG,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [4133] = 6,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(428), 1,
      anon_sym_else,
    STATE(64), 1,
      sym_comment,
    ACTIONS(424), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(426), 21,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_loading,
      anon_sym_error,
      anon_sym_success,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [4179] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(65), 1,
      sym_comment,
    ACTIONS(430), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(432), 22,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_else,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_loading,
      anon_sym_error,
      anon_sym_success,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [4223] = 6,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(66), 1,
      sym_comment,
    STATE(75), 1,
      sym_block,
    ACTIONS(434), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(436), 21,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_loading,
      anon_sym_error,
      anon_sym_success,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [4269] = 6,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(67), 1,
      sym_comment,
    STATE(84), 1,
      sym_block,
    ACTIONS(438), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(440), 21,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_loading,
      anon_sym_error,
      anon_sym_success,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [4315] = 6,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(68), 1,
      sym_comment,
    STATE(92), 1,
      sym_block,
    ACTIONS(442), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(444), 21,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_loading,
      anon_sym_error,
      anon_sym_success,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [4361] = 6,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(450), 1,
      anon_sym_else,
    STATE(69), 1,
      sym_comment,
    ACTIONS(446), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(448), 21,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_loading,
      anon_sym_error,
      anon_sym_success,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [4407] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(70), 1,
      sym_comment,
    ACTIONS(452), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(454), 22,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_else,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_loading,
      anon_sym_error,
      anon_sym_success,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [4451] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(71), 1,
      sym_comment,
    ACTIONS(456), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(458), 21,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_loading,
      anon_sym_error,
      anon_sym_success,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [4494] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(72), 1,
      sym_comment,
    ACTIONS(460), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(462), 21,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_loading,
      anon_sym_error,
      anon_sym_success,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [4537] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(73), 1,
      sym_comment,
    ACTIONS(464), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(466), 21,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_loading,
      anon_sym_error,
      anon_sym_success,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [4580] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(74), 1,
      sym_comment,
    ACTIONS(468), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(470), 21,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_loading,
      anon_sym_error,
      anon_sym_success,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [4623] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(75), 1,
      sym_comment,
    ACTIONS(472), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(474), 21,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_loading,
      anon_sym_error,
      anon_sym_success,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [4666] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(76), 1,
      sym_comment,
    ACTIONS(476), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(478), 21,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_loading,
      anon_sym_error,
      anon_sym_success,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [4709] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(77), 1,
      sym_comment,
    ACTIONS(480), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(482), 21,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_loading,
      anon_sym_error,
      anon_sym_success,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [4752] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(78), 1,
      sym_comment,
    ACTIONS(484), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(486), 21,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_loading,
      anon_sym_error,
      anon_sym_success,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [4795] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(79), 1,
      sym_comment,
    ACTIONS(488), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(490), 21,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_loading,
      anon_sym_error,
      anon_sym_success,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [4838] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(80), 1,
      sym_comment,
    ACTIONS(492), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(494), 21,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_loading,
      anon_sym_error,
      anon_sym_success,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [4881] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(81), 1,
      sym_comment,
    ACTIONS(496), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(498), 21,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_loading,
      anon_sym_error,
      anon_sym_success,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [4924] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(82), 1,
      sym_comment,
    ACTIONS(500), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(502), 21,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_loading,
      anon_sym_error,
      anon_sym_success,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [4967] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(83), 1,
      sym_comment,
    ACTIONS(504), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(506), 21,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_loading,
      anon_sym_error,
      anon_sym_success,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [5010] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(84), 1,
      sym_comment,
    ACTIONS(442), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(444), 21,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_loading,
      anon_sym_error,
      anon_sym_success,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [5053] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(85), 1,
      sym_comment,
    ACTIONS(508), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(510), 21,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_loading,
      anon_sym_error,
      anon_sym_success,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [5096] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(86), 1,
      sym_comment,
    ACTIONS(512), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(514), 21,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_loading,
      anon_sym_error,
      anon_sym_success,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [5139] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(87), 1,
      sym_comment,
    ACTIONS(516), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(518), 21,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_loading,
      anon_sym_error,
      anon_sym_success,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [5182] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(88), 1,
      sym_comment,
    ACTIONS(446), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(448), 21,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_loading,
      anon_sym_error,
      anon_sym_success,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [5225] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(89), 1,
      sym_comment,
    ACTIONS(520), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(522), 21,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_loading,
      anon_sym_error,
      anon_sym_success,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [5268] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(90), 1,
      sym_comment,
    ACTIONS(524), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(526), 21,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_loading,
      anon_sym_error,
      anon_sym_success,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [5311] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(91), 1,
      sym_comment,
    ACTIONS(528), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(530), 21,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_loading,
      anon_sym_error,
      anon_sym_success,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [5354] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(92), 1,
      sym_comment,
    ACTIONS(532), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(534), 21,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_loading,
      anon_sym_error,
      anon_sym_success,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [5397] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(93), 1,
      sym_comment,
    ACTIONS(536), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(538), 21,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_loading,
      anon_sym_error,
      anon_sym_success,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [5440] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(94), 1,
      sym_comment,
    ACTIONS(540), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(542), 21,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_loading,
      anon_sym_error,
      anon_sym_success,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [5483] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(95), 1,
      sym_comment,
    ACTIONS(544), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(546), 21,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_loading,
      anon_sym_error,
      anon_sym_success,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [5526] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(96), 1,
      sym_comment,
    ACTIONS(548), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(550), 21,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_loading,
      anon_sym_error,
      anon_sym_success,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [5569] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(97), 1,
      sym_comment,
    ACTIONS(552), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(554), 21,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_loading,
      anon_sym_error,
      anon_sym_success,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [5612] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(98), 1,
      sym_comment,
    ACTIONS(556), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(558), 21,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_loading,
      anon_sym_error,
      anon_sym_success,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [5655] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(99), 1,
      sym_comment,
    ACTIONS(560), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(562), 21,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_loading,
      anon_sym_error,
      anon_sym_success,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [5698] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(100), 1,
      sym_comment,
    ACTIONS(564), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(566), 21,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_loading,
      anon_sym_error,
      anon_sym_success,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [5741] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(101), 1,
      sym_comment,
    ACTIONS(452), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(454), 21,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_loading,
      anon_sym_error,
      anon_sym_success,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [5784] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(102), 1,
      sym_comment,
    ACTIONS(430), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(432), 21,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_loading,
      anon_sym_error,
      anon_sym_success,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [5827] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(103), 1,
      sym_comment,
    ACTIONS(568), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(570), 21,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_loading,
      anon_sym_error,
      anon_sym_success,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [5870] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(104), 1,
      sym_comment,
    ACTIONS(572), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(574), 21,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_loading,
      anon_sym_error,
      anon_sym_success,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [5913] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(105), 1,
      sym_comment,
    ACTIONS(576), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(578), 21,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_loading,
      anon_sym_error,
      anon_sym_success,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [5956] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(106), 1,
      sym_comment,
    ACTIONS(430), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(432), 19,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_else,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [5997] = 6,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(580), 1,
      anon_sym_else,
    STATE(107), 1,
      sym_comment,
    ACTIONS(446), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(448), 18,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [6040] = 6,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(582), 1,
      anon_sym_else,
    STATE(108), 1,
      sym_comment,
    ACTIONS(424), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(426), 18,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [6083] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(109), 1,
      sym_comment,
    ACTIONS(452), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(454), 19,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_else,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [6124] = 6,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(110), 1,
      sym_comment,
    STATE(119), 1,
      sym_block,
    ACTIONS(438), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(440), 18,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [6167] = 6,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(111), 1,
      sym_comment,
    STATE(150), 1,
      sym_block,
    ACTIONS(442), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(444), 18,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [6210] = 6,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(112), 1,
      sym_comment,
    STATE(142), 1,
      sym_block,
    ACTIONS(434), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(436), 18,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [6253] = 20,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(162), 1,
      anon_sym_LPAREN,
    ACTIONS(164), 1,
      anon_sym_LBRACE,
    ACTIONS(196), 1,
      anon_sym_BANG,
    ACTIONS(198), 1,
      anon_sym_t,
    ACTIONS(200), 1,
      anon_sym_DQUOTE,
    ACTIONS(202), 1,
      sym_number,
    ACTIONS(206), 1,
      sym_null,
    ACTIONS(208), 1,
      anon_sym_LBRACK,
    ACTIONS(584), 1,
      anon_sym_RPAREN,
    ACTIONS(586), 1,
      sym_modifier,
    ACTIONS(588), 1,
      sym_identifier,
    STATE(113), 1,
      sym_comment,
    STATE(229), 1,
      sym__expression,
    STATE(318), 1,
      sym__argument,
    STATE(356), 1,
      sym_named_argument,
    STATE(422), 1,
      sym_argument_list,
    ACTIONS(204), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(20), 9,
      sym_binary_expression,
      sym_unary_expression,
      sym_member_expression,
      sym_call_expression,
      sym_t_call,
      sym_string,
      sym_boolean,
      sym_array,
      sym_object,
  [6323] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(114), 1,
      sym_comment,
    ACTIONS(590), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(592), 18,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [6363] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(115), 1,
      sym_comment,
    ACTIONS(556), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(558), 18,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [6403] = 20,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(162), 1,
      anon_sym_LPAREN,
    ACTIONS(164), 1,
      anon_sym_LBRACE,
    ACTIONS(196), 1,
      anon_sym_BANG,
    ACTIONS(198), 1,
      anon_sym_t,
    ACTIONS(200), 1,
      anon_sym_DQUOTE,
    ACTIONS(202), 1,
      sym_number,
    ACTIONS(206), 1,
      sym_null,
    ACTIONS(208), 1,
      anon_sym_LBRACK,
    ACTIONS(586), 1,
      sym_modifier,
    ACTIONS(588), 1,
      sym_identifier,
    ACTIONS(594), 1,
      anon_sym_RPAREN,
    STATE(116), 1,
      sym_comment,
    STATE(229), 1,
      sym__expression,
    STATE(318), 1,
      sym__argument,
    STATE(356), 1,
      sym_named_argument,
    STATE(379), 1,
      sym_argument_list,
    ACTIONS(204), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(20), 9,
      sym_binary_expression,
      sym_unary_expression,
      sym_member_expression,
      sym_call_expression,
      sym_t_call,
      sym_string,
      sym_boolean,
      sym_array,
      sym_object,
  [6473] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(117), 1,
      sym_comment,
    ACTIONS(488), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(490), 18,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [6513] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(118), 1,
      sym_comment,
    ACTIONS(496), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(498), 18,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [6553] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(119), 1,
      sym_comment,
    ACTIONS(442), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(444), 18,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [6593] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(120), 1,
      sym_comment,
    ACTIONS(476), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(478), 18,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [6633] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(121), 1,
      sym_comment,
    ACTIONS(560), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(562), 18,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [6673] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(122), 1,
      sym_comment,
    ACTIONS(512), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(514), 18,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [6713] = 20,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(162), 1,
      anon_sym_LPAREN,
    ACTIONS(164), 1,
      anon_sym_LBRACE,
    ACTIONS(196), 1,
      anon_sym_BANG,
    ACTIONS(198), 1,
      anon_sym_t,
    ACTIONS(200), 1,
      anon_sym_DQUOTE,
    ACTIONS(202), 1,
      sym_number,
    ACTIONS(206), 1,
      sym_null,
    ACTIONS(208), 1,
      anon_sym_LBRACK,
    ACTIONS(586), 1,
      sym_modifier,
    ACTIONS(588), 1,
      sym_identifier,
    ACTIONS(596), 1,
      anon_sym_RPAREN,
    STATE(123), 1,
      sym_comment,
    STATE(229), 1,
      sym__expression,
    STATE(318), 1,
      sym__argument,
    STATE(356), 1,
      sym_named_argument,
    STATE(442), 1,
      sym_argument_list,
    ACTIONS(204), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(20), 9,
      sym_binary_expression,
      sym_unary_expression,
      sym_member_expression,
      sym_call_expression,
      sym_t_call,
      sym_string,
      sym_boolean,
      sym_array,
      sym_object,
  [6783] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(124), 1,
      sym_comment,
    ACTIONS(500), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(502), 18,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [6823] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(125), 1,
      sym_comment,
    ACTIONS(446), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(448), 18,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [6863] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(126), 1,
      sym_comment,
    ACTIONS(520), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(522), 18,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [6903] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(127), 1,
      sym_comment,
    ACTIONS(524), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(526), 18,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [6943] = 20,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(162), 1,
      anon_sym_LPAREN,
    ACTIONS(164), 1,
      anon_sym_LBRACE,
    ACTIONS(196), 1,
      anon_sym_BANG,
    ACTIONS(198), 1,
      anon_sym_t,
    ACTIONS(200), 1,
      anon_sym_DQUOTE,
    ACTIONS(202), 1,
      sym_number,
    ACTIONS(206), 1,
      sym_null,
    ACTIONS(208), 1,
      anon_sym_LBRACK,
    ACTIONS(586), 1,
      sym_modifier,
    ACTIONS(588), 1,
      sym_identifier,
    ACTIONS(598), 1,
      anon_sym_RPAREN,
    STATE(128), 1,
      sym_comment,
    STATE(229), 1,
      sym__expression,
    STATE(318), 1,
      sym__argument,
    STATE(356), 1,
      sym_named_argument,
    STATE(411), 1,
      sym_argument_list,
    ACTIONS(204), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(20), 9,
      sym_binary_expression,
      sym_unary_expression,
      sym_member_expression,
      sym_call_expression,
      sym_t_call,
      sym_string,
      sym_boolean,
      sym_array,
      sym_object,
  [7013] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(129), 1,
      sym_comment,
    ACTIONS(480), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(482), 18,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [7053] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(130), 1,
      sym_comment,
    ACTIONS(528), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(530), 18,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [7093] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(131), 1,
      sym_comment,
    ACTIONS(544), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(546), 18,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [7133] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(132), 1,
      sym_comment,
    ACTIONS(536), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(538), 18,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [7173] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(133), 1,
      sym_comment,
    ACTIONS(540), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(542), 18,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [7213] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(134), 1,
      sym_comment,
    ACTIONS(504), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(506), 18,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [7253] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(135), 1,
      sym_comment,
    ACTIONS(564), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(566), 18,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [7293] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(136), 1,
      sym_comment,
    ACTIONS(552), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(554), 18,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [7333] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(137), 1,
      sym_comment,
    ACTIONS(452), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(454), 18,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [7373] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(138), 1,
      sym_comment,
    ACTIONS(568), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(570), 18,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [7413] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(139), 1,
      sym_comment,
    ACTIONS(430), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(432), 18,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [7453] = 20,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(162), 1,
      anon_sym_LPAREN,
    ACTIONS(164), 1,
      anon_sym_LBRACE,
    ACTIONS(196), 1,
      anon_sym_BANG,
    ACTIONS(198), 1,
      anon_sym_t,
    ACTIONS(200), 1,
      anon_sym_DQUOTE,
    ACTIONS(202), 1,
      sym_number,
    ACTIONS(206), 1,
      sym_null,
    ACTIONS(208), 1,
      anon_sym_LBRACK,
    ACTIONS(586), 1,
      sym_modifier,
    ACTIONS(588), 1,
      sym_identifier,
    ACTIONS(600), 1,
      anon_sym_RPAREN,
    STATE(140), 1,
      sym_comment,
    STATE(229), 1,
      sym__expression,
    STATE(318), 1,
      sym__argument,
    STATE(356), 1,
      sym_named_argument,
    STATE(371), 1,
      sym_argument_list,
    ACTIONS(204), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(20), 9,
      sym_binary_expression,
      sym_unary_expression,
      sym_member_expression,
      sym_call_expression,
      sym_t_call,
      sym_string,
      sym_boolean,
      sym_array,
      sym_object,
  [7523] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(141), 1,
      sym_comment,
    ACTIONS(548), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(550), 18,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [7563] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(142), 1,
      sym_comment,
    ACTIONS(472), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(474), 18,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [7603] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(143), 1,
      sym_comment,
    ACTIONS(516), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(518), 18,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [7643] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(144), 1,
      sym_comment,
    ACTIONS(460), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(462), 18,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [7683] = 20,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(162), 1,
      anon_sym_LPAREN,
    ACTIONS(164), 1,
      anon_sym_LBRACE,
    ACTIONS(196), 1,
      anon_sym_BANG,
    ACTIONS(198), 1,
      anon_sym_t,
    ACTIONS(200), 1,
      anon_sym_DQUOTE,
    ACTIONS(202), 1,
      sym_number,
    ACTIONS(206), 1,
      sym_null,
    ACTIONS(208), 1,
      anon_sym_LBRACK,
    ACTIONS(586), 1,
      sym_modifier,
    ACTIONS(588), 1,
      sym_identifier,
    ACTIONS(602), 1,
      anon_sym_RPAREN,
    STATE(145), 1,
      sym_comment,
    STATE(229), 1,
      sym__expression,
    STATE(318), 1,
      sym__argument,
    STATE(356), 1,
      sym_named_argument,
    STATE(401), 1,
      sym_argument_list,
    ACTIONS(204), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(20), 9,
      sym_binary_expression,
      sym_unary_expression,
      sym_member_expression,
      sym_call_expression,
      sym_t_call,
      sym_string,
      sym_boolean,
      sym_array,
      sym_object,
  [7753] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(146), 1,
      sym_comment,
    ACTIONS(464), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(466), 18,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [7793] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(147), 1,
      sym_comment,
    ACTIONS(468), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(470), 18,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [7833] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(148), 1,
      sym_comment,
    ACTIONS(484), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(486), 18,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [7873] = 20,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(162), 1,
      anon_sym_LPAREN,
    ACTIONS(164), 1,
      anon_sym_LBRACE,
    ACTIONS(196), 1,
      anon_sym_BANG,
    ACTIONS(198), 1,
      anon_sym_t,
    ACTIONS(200), 1,
      anon_sym_DQUOTE,
    ACTIONS(202), 1,
      sym_number,
    ACTIONS(206), 1,
      sym_null,
    ACTIONS(208), 1,
      anon_sym_LBRACK,
    ACTIONS(586), 1,
      sym_modifier,
    ACTIONS(588), 1,
      sym_identifier,
    ACTIONS(604), 1,
      anon_sym_RPAREN,
    STATE(149), 1,
      sym_comment,
    STATE(229), 1,
      sym__expression,
    STATE(318), 1,
      sym__argument,
    STATE(356), 1,
      sym_named_argument,
    STATE(369), 1,
      sym_argument_list,
    ACTIONS(204), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(20), 9,
      sym_binary_expression,
      sym_unary_expression,
      sym_member_expression,
      sym_call_expression,
      sym_t_call,
      sym_string,
      sym_boolean,
      sym_array,
      sym_object,
  [7943] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(150), 1,
      sym_comment,
    ACTIONS(532), 8,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_RBRACE,
      sym_event_name,
      anon_sym_BANG,
      anon_sym_DQUOTE,
      sym_number,
      anon_sym_LBRACK,
    ACTIONS(534), 18,
      anon_sym_state,
      anon_sym_derived,
      anon_sym_effect,
      anon_sym_action,
      anon_sym_use,
      anon_sym_navigate,
      sym_builtin_component,
      anon_sym_if,
      anon_sym_for,
      anon_sym_show,
      anon_sym_fetch,
      anon_sym_style,
      anon_sym_transition,
      anon_sym_t,
      anon_sym_true,
      anon_sym_false,
      sym_null,
      sym_identifier,
  [7983] = 19,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(162), 1,
      anon_sym_LPAREN,
    ACTIONS(164), 1,
      anon_sym_LBRACE,
    ACTIONS(196), 1,
      anon_sym_BANG,
    ACTIONS(198), 1,
      anon_sym_t,
    ACTIONS(200), 1,
      anon_sym_DQUOTE,
    ACTIONS(202), 1,
      sym_number,
    ACTIONS(206), 1,
      sym_null,
    ACTIONS(208), 1,
      anon_sym_LBRACK,
    ACTIONS(586), 1,
      sym_modifier,
    ACTIONS(588), 1,
      sym_identifier,
    STATE(151), 1,
      sym_comment,
    STATE(229), 1,
      sym__expression,
    STATE(318), 1,
      sym__argument,
    STATE(356), 1,
      sym_named_argument,
    STATE(414), 1,
      sym_argument_list,
    ACTIONS(204), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(20), 9,
      sym_binary_expression,
      sym_unary_expression,
      sym_member_expression,
      sym_call_expression,
      sym_t_call,
      sym_string,
      sym_boolean,
      sym_array,
      sym_object,
  [8050] = 19,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(162), 1,
      anon_sym_LPAREN,
    ACTIONS(164), 1,
      anon_sym_LBRACE,
    ACTIONS(196), 1,
      anon_sym_BANG,
    ACTIONS(198), 1,
      anon_sym_t,
    ACTIONS(200), 1,
      anon_sym_DQUOTE,
    ACTIONS(202), 1,
      sym_number,
    ACTIONS(206), 1,
      sym_null,
    ACTIONS(208), 1,
      anon_sym_LBRACK,
    ACTIONS(586), 1,
      sym_modifier,
    ACTIONS(588), 1,
      sym_identifier,
    STATE(152), 1,
      sym_comment,
    STATE(229), 1,
      sym__expression,
    STATE(318), 1,
      sym__argument,
    STATE(356), 1,
      sym_named_argument,
    STATE(380), 1,
      sym_argument_list,
    ACTIONS(204), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(20), 9,
      sym_binary_expression,
      sym_unary_expression,
      sym_member_expression,
      sym_call_expression,
      sym_t_call,
      sym_string,
      sym_boolean,
      sym_array,
      sym_object,
  [8117] = 19,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(162), 1,
      anon_sym_LPAREN,
    ACTIONS(164), 1,
      anon_sym_LBRACE,
    ACTIONS(196), 1,
      anon_sym_BANG,
    ACTIONS(198), 1,
      anon_sym_t,
    ACTIONS(200), 1,
      anon_sym_DQUOTE,
    ACTIONS(202), 1,
      sym_number,
    ACTIONS(206), 1,
      sym_null,
    ACTIONS(208), 1,
      anon_sym_LBRACK,
    ACTIONS(586), 1,
      sym_modifier,
    ACTIONS(588), 1,
      sym_identifier,
    STATE(153), 1,
      sym_comment,
    STATE(229), 1,
      sym__expression,
    STATE(318), 1,
      sym__argument,
    STATE(356), 1,
      sym_named_argument,
    STATE(402), 1,
      sym_argument_list,
    ACTIONS(204), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(20), 9,
      sym_binary_expression,
      sym_unary_expression,
      sym_member_expression,
      sym_call_expression,
      sym_t_call,
      sym_string,
      sym_boolean,
      sym_array,
      sym_object,
  [8184] = 18,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(162), 1,
      anon_sym_LPAREN,
    ACTIONS(164), 1,
      anon_sym_LBRACE,
    ACTIONS(196), 1,
      anon_sym_BANG,
    ACTIONS(198), 1,
      anon_sym_t,
    ACTIONS(200), 1,
      anon_sym_DQUOTE,
    ACTIONS(202), 1,
      sym_number,
    ACTIONS(206), 1,
      sym_null,
    ACTIONS(208), 1,
      anon_sym_LBRACK,
    ACTIONS(586), 1,
      sym_modifier,
    ACTIONS(588), 1,
      sym_identifier,
    STATE(154), 1,
      sym_comment,
    STATE(229), 1,
      sym__expression,
    STATE(350), 1,
      sym__argument,
    STATE(356), 1,
      sym_named_argument,
    ACTIONS(204), 2,
      anon_sym_true,
      anon_sym_false,
    STATE(20), 9,
      sym_binary_expression,
      sym_unary_expression,
      sym_member_expression,
      sym_call_expression,
      sym_t_call,
      sym_string,
      sym_boolean,
      sym_array,
      sym_object,
  [8248] = 15,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(162), 1,
      anon_sym_LPAREN,
    ACTIONS(164), 1,
      anon_sym_LBRACE,
    ACTIONS(196), 1,
      anon_sym_BANG,
    ACTIONS(198), 1,
      anon_sym_t,
    ACTIONS(200), 1,
      anon_sym_DQUOTE,
    ACTIONS(202), 1,
      sym_number,
    ACTIONS(208), 1,
      anon_sym_LBRACK,
    ACTIONS(606), 1,
      anon_sym_RBRACK,
    STATE(155), 1,
      sym_comment,
    STATE(220), 1,
      sym__expression,
    ACTIONS(204), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(206), 2,
      sym_null,
      sym_identifier,
    STATE(20), 9,
      sym_binary_expression,
      sym_unary_expression,
      sym_member_expression,
      sym_call_expression,
      sym_t_call,
      sym_string,
      sym_boolean,
      sym_array,
      sym_object,
  [8304] = 15,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(162), 1,
      anon_sym_LPAREN,
    ACTIONS(164), 1,
      anon_sym_LBRACE,
    ACTIONS(196), 1,
      anon_sym_BANG,
    ACTIONS(198), 1,
      anon_sym_t,
    ACTIONS(200), 1,
      anon_sym_DQUOTE,
    ACTIONS(202), 1,
      sym_number,
    ACTIONS(208), 1,
      anon_sym_LBRACK,
    ACTIONS(608), 1,
      anon_sym_RBRACK,
    STATE(156), 1,
      sym_comment,
    STATE(199), 1,
      sym__expression,
    ACTIONS(204), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(206), 2,
      sym_null,
      sym_identifier,
    STATE(20), 9,
      sym_binary_expression,
      sym_unary_expression,
      sym_member_expression,
      sym_call_expression,
      sym_t_call,
      sym_string,
      sym_boolean,
      sym_array,
      sym_object,
  [8360] = 15,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(162), 1,
      anon_sym_LPAREN,
    ACTIONS(164), 1,
      anon_sym_LBRACE,
    ACTIONS(196), 1,
      anon_sym_BANG,
    ACTIONS(198), 1,
      anon_sym_t,
    ACTIONS(200), 1,
      anon_sym_DQUOTE,
    ACTIONS(202), 1,
      sym_number,
    ACTIONS(208), 1,
      anon_sym_LBRACK,
    ACTIONS(610), 1,
      anon_sym_RBRACK,
    STATE(157), 1,
      sym_comment,
    STATE(217), 1,
      sym__expression,
    ACTIONS(204), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(206), 2,
      sym_null,
      sym_identifier,
    STATE(20), 9,
      sym_binary_expression,
      sym_unary_expression,
      sym_member_expression,
      sym_call_expression,
      sym_t_call,
      sym_string,
      sym_boolean,
      sym_array,
      sym_object,
  [8416] = 14,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(162), 1,
      anon_sym_LPAREN,
    ACTIONS(164), 1,
      anon_sym_LBRACE,
    ACTIONS(196), 1,
      anon_sym_BANG,
    ACTIONS(198), 1,
      anon_sym_t,
    ACTIONS(200), 1,
      anon_sym_DQUOTE,
    ACTIONS(202), 1,
      sym_number,
    ACTIONS(208), 1,
      anon_sym_LBRACK,
    STATE(158), 1,
      sym_comment,
    STATE(242), 1,
      sym__expression,
    ACTIONS(204), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(206), 2,
      sym_null,
      sym_identifier,
    STATE(20), 9,
      sym_binary_expression,
      sym_unary_expression,
      sym_member_expression,
      sym_call_expression,
      sym_t_call,
      sym_string,
      sym_boolean,
      sym_array,
      sym_object,
  [8469] = 14,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(612), 1,
      anon_sym_LPAREN,
    ACTIONS(614), 1,
      anon_sym_LBRACE,
    ACTIONS(616), 1,
      anon_sym_BANG,
    ACTIONS(618), 1,
      anon_sym_t,
    ACTIONS(620), 1,
      anon_sym_DQUOTE,
    ACTIONS(622), 1,
      sym_number,
    ACTIONS(628), 1,
      anon_sym_LBRACK,
    STATE(159), 1,
      sym_comment,
    STATE(213), 1,
      sym__expression,
    ACTIONS(624), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(626), 2,
      sym_null,
      sym_identifier,
    STATE(223), 9,
      sym_binary_expression,
      sym_unary_expression,
      sym_member_expression,
      sym_call_expression,
      sym_t_call,
      sym_string,
      sym_boolean,
      sym_array,
      sym_object,
  [8522] = 14,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(162), 1,
      anon_sym_LPAREN,
    ACTIONS(164), 1,
      anon_sym_LBRACE,
    ACTIONS(196), 1,
      anon_sym_BANG,
    ACTIONS(198), 1,
      anon_sym_t,
    ACTIONS(200), 1,
      anon_sym_DQUOTE,
    ACTIONS(202), 1,
      sym_number,
    ACTIONS(208), 1,
      anon_sym_LBRACK,
    STATE(160), 1,
      sym_comment,
    STATE(227), 1,
      sym__expression,
    ACTIONS(204), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(206), 2,
      sym_null,
      sym_identifier,
    STATE(20), 9,
      sym_binary_expression,
      sym_unary_expression,
      sym_member_expression,
      sym_call_expression,
      sym_t_call,
      sym_string,
      sym_boolean,
      sym_array,
      sym_object,
  [8575] = 14,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(162), 1,
      anon_sym_LPAREN,
    ACTIONS(164), 1,
      anon_sym_LBRACE,
    ACTIONS(196), 1,
      anon_sym_BANG,
    ACTIONS(198), 1,
      anon_sym_t,
    ACTIONS(200), 1,
      anon_sym_DQUOTE,
    ACTIONS(202), 1,
      sym_number,
    ACTIONS(208), 1,
      anon_sym_LBRACK,
    STATE(161), 1,
      sym_comment,
    STATE(214), 1,
      sym__expression,
    ACTIONS(204), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(206), 2,
      sym_null,
      sym_identifier,
    STATE(20), 9,
      sym_binary_expression,
      sym_unary_expression,
      sym_member_expression,
      sym_call_expression,
      sym_t_call,
      sym_string,
      sym_boolean,
      sym_array,
      sym_object,
  [8628] = 14,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(162), 1,
      anon_sym_LPAREN,
    ACTIONS(164), 1,
      anon_sym_LBRACE,
    ACTIONS(196), 1,
      anon_sym_BANG,
    ACTIONS(198), 1,
      anon_sym_t,
    ACTIONS(200), 1,
      anon_sym_DQUOTE,
    ACTIONS(202), 1,
      sym_number,
    ACTIONS(208), 1,
      anon_sym_LBRACK,
    STATE(162), 1,
      sym_comment,
    STATE(230), 1,
      sym__expression,
    ACTIONS(204), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(206), 2,
      sym_null,
      sym_identifier,
    STATE(20), 9,
      sym_binary_expression,
      sym_unary_expression,
      sym_member_expression,
      sym_call_expression,
      sym_t_call,
      sym_string,
      sym_boolean,
      sym_array,
      sym_object,
  [8681] = 14,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(162), 1,
      anon_sym_LPAREN,
    ACTIONS(164), 1,
      anon_sym_LBRACE,
    ACTIONS(196), 1,
      anon_sym_BANG,
    ACTIONS(198), 1,
      anon_sym_t,
    ACTIONS(200), 1,
      anon_sym_DQUOTE,
    ACTIONS(202), 1,
      sym_number,
    ACTIONS(208), 1,
      anon_sym_LBRACK,
    STATE(163), 1,
      sym_comment,
    STATE(241), 1,
      sym__expression,
    ACTIONS(204), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(206), 2,
      sym_null,
      sym_identifier,
    STATE(20), 9,
      sym_binary_expression,
      sym_unary_expression,
      sym_member_expression,
      sym_call_expression,
      sym_t_call,
      sym_string,
      sym_boolean,
      sym_array,
      sym_object,
  [8734] = 14,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(162), 1,
      anon_sym_LPAREN,
    ACTIONS(164), 1,
      anon_sym_LBRACE,
    ACTIONS(196), 1,
      anon_sym_BANG,
    ACTIONS(198), 1,
      anon_sym_t,
    ACTIONS(200), 1,
      anon_sym_DQUOTE,
    ACTIONS(202), 1,
      sym_number,
    ACTIONS(208), 1,
      anon_sym_LBRACK,
    STATE(164), 1,
      sym_comment,
    STATE(244), 1,
      sym__expression,
    ACTIONS(204), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(206), 2,
      sym_null,
      sym_identifier,
    STATE(20), 9,
      sym_binary_expression,
      sym_unary_expression,
      sym_member_expression,
      sym_call_expression,
      sym_t_call,
      sym_string,
      sym_boolean,
      sym_array,
      sym_object,
  [8787] = 14,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(17), 1,
      anon_sym_LPAREN,
    ACTIONS(19), 1,
      anon_sym_LBRACE,
    ACTIONS(57), 1,
      anon_sym_BANG,
    ACTIONS(59), 1,
      anon_sym_t,
    ACTIONS(61), 1,
      anon_sym_DQUOTE,
    ACTIONS(63), 1,
      sym_number,
    ACTIONS(69), 1,
      anon_sym_LBRACK,
    STATE(36), 1,
      sym__expression,
    STATE(165), 1,
      sym_comment,
    ACTIONS(65), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(67), 2,
      sym_null,
      sym_identifier,
    STATE(39), 9,
      sym_binary_expression,
      sym_unary_expression,
      sym_member_expression,
      sym_call_expression,
      sym_t_call,
      sym_string,
      sym_boolean,
      sym_array,
      sym_object,
  [8840] = 14,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(612), 1,
      anon_sym_LPAREN,
    ACTIONS(614), 1,
      anon_sym_LBRACE,
    ACTIONS(616), 1,
      anon_sym_BANG,
    ACTIONS(618), 1,
      anon_sym_t,
    ACTIONS(620), 1,
      anon_sym_DQUOTE,
    ACTIONS(622), 1,
      sym_number,
    ACTIONS(628), 1,
      anon_sym_LBRACK,
    STATE(166), 1,
      sym_comment,
    STATE(206), 1,
      sym__expression,
    ACTIONS(624), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(626), 2,
      sym_null,
      sym_identifier,
    STATE(223), 9,
      sym_binary_expression,
      sym_unary_expression,
      sym_member_expression,
      sym_call_expression,
      sym_t_call,
      sym_string,
      sym_boolean,
      sym_array,
      sym_object,
  [8893] = 14,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(162), 1,
      anon_sym_LPAREN,
    ACTIONS(164), 1,
      anon_sym_LBRACE,
    ACTIONS(196), 1,
      anon_sym_BANG,
    ACTIONS(198), 1,
      anon_sym_t,
    ACTIONS(200), 1,
      anon_sym_DQUOTE,
    ACTIONS(202), 1,
      sym_number,
    ACTIONS(208), 1,
      anon_sym_LBRACK,
    STATE(61), 1,
      sym__expression,
    STATE(167), 1,
      sym_comment,
    ACTIONS(204), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(206), 2,
      sym_null,
      sym_identifier,
    STATE(20), 9,
      sym_binary_expression,
      sym_unary_expression,
      sym_member_expression,
      sym_call_expression,
      sym_t_call,
      sym_string,
      sym_boolean,
      sym_array,
      sym_object,
  [8946] = 14,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(17), 1,
      anon_sym_LPAREN,
    ACTIONS(19), 1,
      anon_sym_LBRACE,
    ACTIONS(57), 1,
      anon_sym_BANG,
    ACTIONS(59), 1,
      anon_sym_t,
    ACTIONS(61), 1,
      anon_sym_DQUOTE,
    ACTIONS(63), 1,
      sym_number,
    ACTIONS(69), 1,
      anon_sym_LBRACK,
    STATE(37), 1,
      sym__expression,
    STATE(168), 1,
      sym_comment,
    ACTIONS(65), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(67), 2,
      sym_null,
      sym_identifier,
    STATE(39), 9,
      sym_binary_expression,
      sym_unary_expression,
      sym_member_expression,
      sym_call_expression,
      sym_t_call,
      sym_string,
      sym_boolean,
      sym_array,
      sym_object,
  [8999] = 14,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(17), 1,
      anon_sym_LPAREN,
    ACTIONS(19), 1,
      anon_sym_LBRACE,
    ACTIONS(57), 1,
      anon_sym_BANG,
    ACTIONS(59), 1,
      anon_sym_t,
    ACTIONS(61), 1,
      anon_sym_DQUOTE,
    ACTIONS(63), 1,
      sym_number,
    ACTIONS(69), 1,
      anon_sym_LBRACK,
    STATE(38), 1,
      sym__expression,
    STATE(169), 1,
      sym_comment,
    ACTIONS(65), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(67), 2,
      sym_null,
      sym_identifier,
    STATE(39), 9,
      sym_binary_expression,
      sym_unary_expression,
      sym_member_expression,
      sym_call_expression,
      sym_t_call,
      sym_string,
      sym_boolean,
      sym_array,
      sym_object,
  [9052] = 14,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(162), 1,
      anon_sym_LPAREN,
    ACTIONS(164), 1,
      anon_sym_LBRACE,
    ACTIONS(196), 1,
      anon_sym_BANG,
    ACTIONS(198), 1,
      anon_sym_t,
    ACTIONS(200), 1,
      anon_sym_DQUOTE,
    ACTIONS(202), 1,
      sym_number,
    ACTIONS(208), 1,
      anon_sym_LBRACK,
    STATE(27), 1,
      sym__expression,
    STATE(170), 1,
      sym_comment,
    ACTIONS(204), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(206), 2,
      sym_null,
      sym_identifier,
    STATE(20), 9,
      sym_binary_expression,
      sym_unary_expression,
      sym_member_expression,
      sym_call_expression,
      sym_t_call,
      sym_string,
      sym_boolean,
      sym_array,
      sym_object,
  [9105] = 14,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(162), 1,
      anon_sym_LPAREN,
    ACTIONS(164), 1,
      anon_sym_LBRACE,
    ACTIONS(196), 1,
      anon_sym_BANG,
    ACTIONS(198), 1,
      anon_sym_t,
    ACTIONS(200), 1,
      anon_sym_DQUOTE,
    ACTIONS(202), 1,
      sym_number,
    ACTIONS(208), 1,
      anon_sym_LBRACK,
    STATE(171), 1,
      sym_comment,
    STATE(236), 1,
      sym__expression,
    ACTIONS(204), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(206), 2,
      sym_null,
      sym_identifier,
    STATE(20), 9,
      sym_binary_expression,
      sym_unary_expression,
      sym_member_expression,
      sym_call_expression,
      sym_t_call,
      sym_string,
      sym_boolean,
      sym_array,
      sym_object,
  [9158] = 14,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(162), 1,
      anon_sym_LPAREN,
    ACTIONS(164), 1,
      anon_sym_LBRACE,
    ACTIONS(196), 1,
      anon_sym_BANG,
    ACTIONS(198), 1,
      anon_sym_t,
    ACTIONS(200), 1,
      anon_sym_DQUOTE,
    ACTIONS(202), 1,
      sym_number,
    ACTIONS(208), 1,
      anon_sym_LBRACK,
    STATE(172), 1,
      sym_comment,
    STATE(239), 1,
      sym__expression,
    ACTIONS(204), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(206), 2,
      sym_null,
      sym_identifier,
    STATE(20), 9,
      sym_binary_expression,
      sym_unary_expression,
      sym_member_expression,
      sym_call_expression,
      sym_t_call,
      sym_string,
      sym_boolean,
      sym_array,
      sym_object,
  [9211] = 14,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(162), 1,
      anon_sym_LPAREN,
    ACTIONS(164), 1,
      anon_sym_LBRACE,
    ACTIONS(196), 1,
      anon_sym_BANG,
    ACTIONS(198), 1,
      anon_sym_t,
    ACTIONS(200), 1,
      anon_sym_DQUOTE,
    ACTIONS(202), 1,
      sym_number,
    ACTIONS(208), 1,
      anon_sym_LBRACK,
    STATE(173), 1,
      sym_comment,
    STATE(226), 1,
      sym__expression,
    ACTIONS(204), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(206), 2,
      sym_null,
      sym_identifier,
    STATE(20), 9,
      sym_binary_expression,
      sym_unary_expression,
      sym_member_expression,
      sym_call_expression,
      sym_t_call,
      sym_string,
      sym_boolean,
      sym_array,
      sym_object,
  [9264] = 14,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(162), 1,
      anon_sym_LPAREN,
    ACTIONS(164), 1,
      anon_sym_LBRACE,
    ACTIONS(196), 1,
      anon_sym_BANG,
    ACTIONS(198), 1,
      anon_sym_t,
    ACTIONS(200), 1,
      anon_sym_DQUOTE,
    ACTIONS(202), 1,
      sym_number,
    ACTIONS(208), 1,
      anon_sym_LBRACK,
    STATE(57), 1,
      sym__expression,
    STATE(174), 1,
      sym_comment,
    ACTIONS(204), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(206), 2,
      sym_null,
      sym_identifier,
    STATE(20), 9,
      sym_binary_expression,
      sym_unary_expression,
      sym_member_expression,
      sym_call_expression,
      sym_t_call,
      sym_string,
      sym_boolean,
      sym_array,
      sym_object,
  [9317] = 14,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(162), 1,
      anon_sym_LPAREN,
    ACTIONS(164), 1,
      anon_sym_LBRACE,
    ACTIONS(196), 1,
      anon_sym_BANG,
    ACTIONS(198), 1,
      anon_sym_t,
    ACTIONS(200), 1,
      anon_sym_DQUOTE,
    ACTIONS(202), 1,
      sym_number,
    ACTIONS(208), 1,
      anon_sym_LBRACK,
    STATE(60), 1,
      sym__expression,
    STATE(175), 1,
      sym_comment,
    ACTIONS(204), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(206), 2,
      sym_null,
      sym_identifier,
    STATE(20), 9,
      sym_binary_expression,
      sym_unary_expression,
      sym_member_expression,
      sym_call_expression,
      sym_t_call,
      sym_string,
      sym_boolean,
      sym_array,
      sym_object,
  [9370] = 14,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(162), 1,
      anon_sym_LPAREN,
    ACTIONS(164), 1,
      anon_sym_LBRACE,
    ACTIONS(196), 1,
      anon_sym_BANG,
    ACTIONS(198), 1,
      anon_sym_t,
    ACTIONS(200), 1,
      anon_sym_DQUOTE,
    ACTIONS(202), 1,
      sym_number,
    ACTIONS(208), 1,
      anon_sym_LBRACK,
    STATE(176), 1,
      sym_comment,
    STATE(204), 1,
      sym__expression,
    ACTIONS(204), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(206), 2,
      sym_null,
      sym_identifier,
    STATE(20), 9,
      sym_binary_expression,
      sym_unary_expression,
      sym_member_expression,
      sym_call_expression,
      sym_t_call,
      sym_string,
      sym_boolean,
      sym_array,
      sym_object,
  [9423] = 14,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(162), 1,
      anon_sym_LPAREN,
    ACTIONS(164), 1,
      anon_sym_LBRACE,
    ACTIONS(196), 1,
      anon_sym_BANG,
    ACTIONS(198), 1,
      anon_sym_t,
    ACTIONS(200), 1,
      anon_sym_DQUOTE,
    ACTIONS(202), 1,
      sym_number,
    ACTIONS(208), 1,
      anon_sym_LBRACK,
    STATE(177), 1,
      sym_comment,
    STATE(240), 1,
      sym__expression,
    ACTIONS(204), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(206), 2,
      sym_null,
      sym_identifier,
    STATE(20), 9,
      sym_binary_expression,
      sym_unary_expression,
      sym_member_expression,
      sym_call_expression,
      sym_t_call,
      sym_string,
      sym_boolean,
      sym_array,
      sym_object,
  [9476] = 14,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(17), 1,
      anon_sym_LPAREN,
    ACTIONS(19), 1,
      anon_sym_LBRACE,
    ACTIONS(57), 1,
      anon_sym_BANG,
    ACTIONS(59), 1,
      anon_sym_t,
    ACTIONS(61), 1,
      anon_sym_DQUOTE,
    ACTIONS(63), 1,
      sym_number,
    ACTIONS(69), 1,
      anon_sym_LBRACK,
    STATE(51), 1,
      sym__expression,
    STATE(178), 1,
      sym_comment,
    ACTIONS(65), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(67), 2,
      sym_null,
      sym_identifier,
    STATE(39), 9,
      sym_binary_expression,
      sym_unary_expression,
      sym_member_expression,
      sym_call_expression,
      sym_t_call,
      sym_string,
      sym_boolean,
      sym_array,
      sym_object,
  [9529] = 14,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(612), 1,
      anon_sym_LPAREN,
    ACTIONS(614), 1,
      anon_sym_LBRACE,
    ACTIONS(616), 1,
      anon_sym_BANG,
    ACTIONS(618), 1,
      anon_sym_t,
    ACTIONS(620), 1,
      anon_sym_DQUOTE,
    ACTIONS(622), 1,
      sym_number,
    ACTIONS(628), 1,
      anon_sym_LBRACK,
    STATE(179), 1,
      sym_comment,
    STATE(238), 1,
      sym__expression,
    ACTIONS(624), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(626), 2,
      sym_null,
      sym_identifier,
    STATE(223), 9,
      sym_binary_expression,
      sym_unary_expression,
      sym_member_expression,
      sym_call_expression,
      sym_t_call,
      sym_string,
      sym_boolean,
      sym_array,
      sym_object,
  [9582] = 14,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(162), 1,
      anon_sym_LPAREN,
    ACTIONS(164), 1,
      anon_sym_LBRACE,
    ACTIONS(196), 1,
      anon_sym_BANG,
    ACTIONS(198), 1,
      anon_sym_t,
    ACTIONS(200), 1,
      anon_sym_DQUOTE,
    ACTIONS(202), 1,
      sym_number,
    ACTIONS(208), 1,
      anon_sym_LBRACK,
    STATE(180), 1,
      sym_comment,
    STATE(237), 1,
      sym__expression,
    ACTIONS(204), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(206), 2,
      sym_null,
      sym_identifier,
    STATE(20), 9,
      sym_binary_expression,
      sym_unary_expression,
      sym_member_expression,
      sym_call_expression,
      sym_t_call,
      sym_string,
      sym_boolean,
      sym_array,
      sym_object,
  [9635] = 14,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(162), 1,
      anon_sym_LPAREN,
    ACTIONS(164), 1,
      anon_sym_LBRACE,
    ACTIONS(196), 1,
      anon_sym_BANG,
    ACTIONS(198), 1,
      anon_sym_t,
    ACTIONS(200), 1,
      anon_sym_DQUOTE,
    ACTIONS(202), 1,
      sym_number,
    ACTIONS(208), 1,
      anon_sym_LBRACK,
    STATE(181), 1,
      sym_comment,
    STATE(221), 1,
      sym__expression,
    ACTIONS(204), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(206), 2,
      sym_null,
      sym_identifier,
    STATE(20), 9,
      sym_binary_expression,
      sym_unary_expression,
      sym_member_expression,
      sym_call_expression,
      sym_t_call,
      sym_string,
      sym_boolean,
      sym_array,
      sym_object,
  [9688] = 14,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(612), 1,
      anon_sym_LPAREN,
    ACTIONS(614), 1,
      anon_sym_LBRACE,
    ACTIONS(616), 1,
      anon_sym_BANG,
    ACTIONS(618), 1,
      anon_sym_t,
    ACTIONS(620), 1,
      anon_sym_DQUOTE,
    ACTIONS(622), 1,
      sym_number,
    ACTIONS(628), 1,
      anon_sym_LBRACK,
    STATE(182), 1,
      sym_comment,
    STATE(197), 1,
      sym__expression,
    ACTIONS(624), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(626), 2,
      sym_null,
      sym_identifier,
    STATE(223), 9,
      sym_binary_expression,
      sym_unary_expression,
      sym_member_expression,
      sym_call_expression,
      sym_t_call,
      sym_string,
      sym_boolean,
      sym_array,
      sym_object,
  [9741] = 14,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(162), 1,
      anon_sym_LPAREN,
    ACTIONS(164), 1,
      anon_sym_LBRACE,
    ACTIONS(196), 1,
      anon_sym_BANG,
    ACTIONS(198), 1,
      anon_sym_t,
    ACTIONS(200), 1,
      anon_sym_DQUOTE,
    ACTIONS(202), 1,
      sym_number,
    ACTIONS(208), 1,
      anon_sym_LBRACK,
    STATE(183), 1,
      sym_comment,
    STATE(222), 1,
      sym__expression,
    ACTIONS(204), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(206), 2,
      sym_null,
      sym_identifier,
    STATE(20), 9,
      sym_binary_expression,
      sym_unary_expression,
      sym_member_expression,
      sym_call_expression,
      sym_t_call,
      sym_string,
      sym_boolean,
      sym_array,
      sym_object,
  [9794] = 14,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(17), 1,
      anon_sym_LPAREN,
    ACTIONS(19), 1,
      anon_sym_LBRACE,
    ACTIONS(57), 1,
      anon_sym_BANG,
    ACTIONS(59), 1,
      anon_sym_t,
    ACTIONS(61), 1,
      anon_sym_DQUOTE,
    ACTIONS(63), 1,
      sym_number,
    ACTIONS(69), 1,
      anon_sym_LBRACK,
    STATE(42), 1,
      sym__expression,
    STATE(184), 1,
      sym_comment,
    ACTIONS(65), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(67), 2,
      sym_null,
      sym_identifier,
    STATE(39), 9,
      sym_binary_expression,
      sym_unary_expression,
      sym_member_expression,
      sym_call_expression,
      sym_t_call,
      sym_string,
      sym_boolean,
      sym_array,
      sym_object,
  [9847] = 14,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(162), 1,
      anon_sym_LPAREN,
    ACTIONS(164), 1,
      anon_sym_LBRACE,
    ACTIONS(196), 1,
      anon_sym_BANG,
    ACTIONS(198), 1,
      anon_sym_t,
    ACTIONS(200), 1,
      anon_sym_DQUOTE,
    ACTIONS(202), 1,
      sym_number,
    ACTIONS(208), 1,
      anon_sym_LBRACK,
    STATE(185), 1,
      sym_comment,
    STATE(232), 1,
      sym__expression,
    ACTIONS(204), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(206), 2,
      sym_null,
      sym_identifier,
    STATE(20), 9,
      sym_binary_expression,
      sym_unary_expression,
      sym_member_expression,
      sym_call_expression,
      sym_t_call,
      sym_string,
      sym_boolean,
      sym_array,
      sym_object,
  [9900] = 14,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(162), 1,
      anon_sym_LPAREN,
    ACTIONS(164), 1,
      anon_sym_LBRACE,
    ACTIONS(196), 1,
      anon_sym_BANG,
    ACTIONS(198), 1,
      anon_sym_t,
    ACTIONS(200), 1,
      anon_sym_DQUOTE,
    ACTIONS(202), 1,
      sym_number,
    ACTIONS(208), 1,
      anon_sym_LBRACK,
    STATE(186), 1,
      sym_comment,
    STATE(243), 1,
      sym__expression,
    ACTIONS(204), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(206), 2,
      sym_null,
      sym_identifier,
    STATE(20), 9,
      sym_binary_expression,
      sym_unary_expression,
      sym_member_expression,
      sym_call_expression,
      sym_t_call,
      sym_string,
      sym_boolean,
      sym_array,
      sym_object,
  [9953] = 14,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(162), 1,
      anon_sym_LPAREN,
    ACTIONS(164), 1,
      anon_sym_LBRACE,
    ACTIONS(196), 1,
      anon_sym_BANG,
    ACTIONS(198), 1,
      anon_sym_t,
    ACTIONS(200), 1,
      anon_sym_DQUOTE,
    ACTIONS(202), 1,
      sym_number,
    ACTIONS(208), 1,
      anon_sym_LBRACK,
    STATE(187), 1,
      sym_comment,
    STATE(215), 1,
      sym__expression,
    ACTIONS(204), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(206), 2,
      sym_null,
      sym_identifier,
    STATE(20), 9,
      sym_binary_expression,
      sym_unary_expression,
      sym_member_expression,
      sym_call_expression,
      sym_t_call,
      sym_string,
      sym_boolean,
      sym_array,
      sym_object,
  [10006] = 14,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(162), 1,
      anon_sym_LPAREN,
    ACTIONS(164), 1,
      anon_sym_LBRACE,
    ACTIONS(196), 1,
      anon_sym_BANG,
    ACTIONS(198), 1,
      anon_sym_t,
    ACTIONS(200), 1,
      anon_sym_DQUOTE,
    ACTIONS(202), 1,
      sym_number,
    ACTIONS(208), 1,
      anon_sym_LBRACK,
    STATE(188), 1,
      sym_comment,
    STATE(216), 1,
      sym__expression,
    ACTIONS(204), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(206), 2,
      sym_null,
      sym_identifier,
    STATE(20), 9,
      sym_binary_expression,
      sym_unary_expression,
      sym_member_expression,
      sym_call_expression,
      sym_t_call,
      sym_string,
      sym_boolean,
      sym_array,
      sym_object,
  [10059] = 14,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(162), 1,
      anon_sym_LPAREN,
    ACTIONS(164), 1,
      anon_sym_LBRACE,
    ACTIONS(196), 1,
      anon_sym_BANG,
    ACTIONS(198), 1,
      anon_sym_t,
    ACTIONS(200), 1,
      anon_sym_DQUOTE,
    ACTIONS(202), 1,
      sym_number,
    ACTIONS(208), 1,
      anon_sym_LBRACK,
    STATE(189), 1,
      sym_comment,
    STATE(245), 1,
      sym__expression,
    ACTIONS(204), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(206), 2,
      sym_null,
      sym_identifier,
    STATE(20), 9,
      sym_binary_expression,
      sym_unary_expression,
      sym_member_expression,
      sym_call_expression,
      sym_t_call,
      sym_string,
      sym_boolean,
      sym_array,
      sym_object,
  [10112] = 14,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(162), 1,
      anon_sym_LPAREN,
    ACTIONS(164), 1,
      anon_sym_LBRACE,
    ACTIONS(196), 1,
      anon_sym_BANG,
    ACTIONS(198), 1,
      anon_sym_t,
    ACTIONS(200), 1,
      anon_sym_DQUOTE,
    ACTIONS(202), 1,
      sym_number,
    ACTIONS(208), 1,
      anon_sym_LBRACK,
    STATE(190), 1,
      sym_comment,
    STATE(218), 1,
      sym__expression,
    ACTIONS(204), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(206), 2,
      sym_null,
      sym_identifier,
    STATE(20), 9,
      sym_binary_expression,
      sym_unary_expression,
      sym_member_expression,
      sym_call_expression,
      sym_t_call,
      sym_string,
      sym_boolean,
      sym_array,
      sym_object,
  [10165] = 14,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(162), 1,
      anon_sym_LPAREN,
    ACTIONS(164), 1,
      anon_sym_LBRACE,
    ACTIONS(196), 1,
      anon_sym_BANG,
    ACTIONS(198), 1,
      anon_sym_t,
    ACTIONS(200), 1,
      anon_sym_DQUOTE,
    ACTIONS(202), 1,
      sym_number,
    ACTIONS(208), 1,
      anon_sym_LBRACK,
    STATE(191), 1,
      sym_comment,
    STATE(231), 1,
      sym__expression,
    ACTIONS(204), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(206), 2,
      sym_null,
      sym_identifier,
    STATE(20), 9,
      sym_binary_expression,
      sym_unary_expression,
      sym_member_expression,
      sym_call_expression,
      sym_t_call,
      sym_string,
      sym_boolean,
      sym_array,
      sym_object,
  [10218] = 14,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(162), 1,
      anon_sym_LPAREN,
    ACTIONS(164), 1,
      anon_sym_LBRACE,
    ACTIONS(196), 1,
      anon_sym_BANG,
    ACTIONS(198), 1,
      anon_sym_t,
    ACTIONS(200), 1,
      anon_sym_DQUOTE,
    ACTIONS(202), 1,
      sym_number,
    ACTIONS(208), 1,
      anon_sym_LBRACK,
    STATE(192), 1,
      sym_comment,
    STATE(219), 1,
      sym__expression,
    ACTIONS(204), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(206), 2,
      sym_null,
      sym_identifier,
    STATE(20), 9,
      sym_binary_expression,
      sym_unary_expression,
      sym_member_expression,
      sym_call_expression,
      sym_t_call,
      sym_string,
      sym_boolean,
      sym_array,
      sym_object,
  [10271] = 14,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(162), 1,
      anon_sym_LPAREN,
    ACTIONS(164), 1,
      anon_sym_LBRACE,
    ACTIONS(196), 1,
      anon_sym_BANG,
    ACTIONS(198), 1,
      anon_sym_t,
    ACTIONS(200), 1,
      anon_sym_DQUOTE,
    ACTIONS(202), 1,
      sym_number,
    ACTIONS(208), 1,
      anon_sym_LBRACK,
    STATE(193), 1,
      sym_comment,
    STATE(246), 1,
      sym__expression,
    ACTIONS(204), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(206), 2,
      sym_null,
      sym_identifier,
    STATE(20), 9,
      sym_binary_expression,
      sym_unary_expression,
      sym_member_expression,
      sym_call_expression,
      sym_t_call,
      sym_string,
      sym_boolean,
      sym_array,
      sym_object,
  [10324] = 14,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(162), 1,
      anon_sym_LPAREN,
    ACTIONS(164), 1,
      anon_sym_LBRACE,
    ACTIONS(196), 1,
      anon_sym_BANG,
    ACTIONS(198), 1,
      anon_sym_t,
    ACTIONS(200), 1,
      anon_sym_DQUOTE,
    ACTIONS(202), 1,
      sym_number,
    ACTIONS(208), 1,
      anon_sym_LBRACK,
    STATE(62), 1,
      sym__expression,
    STATE(194), 1,
      sym_comment,
    ACTIONS(204), 2,
      anon_sym_true,
      anon_sym_false,
    ACTIONS(206), 2,
      sym_null,
      sym_identifier,
    STATE(20), 9,
      sym_binary_expression,
      sym_unary_expression,
      sym_member_expression,
      sym_call_expression,
      sym_t_call,
      sym_string,
      sym_boolean,
      sym_array,
      sym_object,
  [10377] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(195), 1,
      sym_comment,
    ACTIONS(341), 3,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
    ACTIONS(339), 15,
      anon_sym_LPAREN,
      anon_sym_RBRACE,
      anon_sym_DOT,
      anon_sym_DASH,
      anon_sym_ATmedia,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
      sym_identifier,
  [10409] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(196), 1,
      sym_comment,
    ACTIONS(321), 3,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
    ACTIONS(319), 15,
      anon_sym_LPAREN,
      anon_sym_RBRACE,
      anon_sym_DOT,
      anon_sym_DASH,
      anon_sym_ATmedia,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
      sym_identifier,
  [10441] = 7,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(630), 1,
      anon_sym_LPAREN,
    ACTIONS(632), 1,
      anon_sym_DOT,
    STATE(197), 1,
      sym_comment,
    ACTIONS(409), 3,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
    ACTIONS(407), 13,
      anon_sym_RBRACE,
      anon_sym_DASH,
      anon_sym_ATmedia,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
      sym_identifier,
  [10477] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(198), 1,
      sym_comment,
    ACTIONS(413), 3,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
    ACTIONS(411), 15,
      anon_sym_LPAREN,
      anon_sym_RBRACE,
      anon_sym_DOT,
      anon_sym_DASH,
      anon_sym_ATmedia,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
      sym_identifier,
  [10509] = 10,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(347), 1,
      anon_sym_LPAREN,
    ACTIONS(353), 1,
      anon_sym_DOT,
    ACTIONS(634), 1,
      anon_sym_COMMA,
    ACTIONS(636), 1,
      anon_sym_RBRACK,
    STATE(199), 1,
      sym_comment,
    STATE(294), 1,
      aux_sym_array_repeat1,
    ACTIONS(422), 3,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
    ACTIONS(420), 10,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
  [10551] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(200), 1,
      sym_comment,
    ACTIONS(325), 3,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
    ACTIONS(323), 15,
      anon_sym_LPAREN,
      anon_sym_RBRACE,
      anon_sym_DOT,
      anon_sym_DASH,
      anon_sym_ATmedia,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
      sym_identifier,
  [10583] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(201), 1,
      sym_comment,
    ACTIONS(329), 3,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
    ACTIONS(327), 15,
      anon_sym_LPAREN,
      anon_sym_RBRACE,
      anon_sym_DOT,
      anon_sym_DASH,
      anon_sym_ATmedia,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
      sym_identifier,
  [10615] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(202), 1,
      sym_comment,
    ACTIONS(333), 3,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
    ACTIONS(331), 15,
      anon_sym_LPAREN,
      anon_sym_RBRACE,
      anon_sym_DOT,
      anon_sym_DASH,
      anon_sym_ATmedia,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
      sym_identifier,
  [10647] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(203), 1,
      sym_comment,
    ACTIONS(337), 3,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
    ACTIONS(335), 15,
      anon_sym_LPAREN,
      anon_sym_RBRACE,
      anon_sym_DOT,
      anon_sym_DASH,
      anon_sym_ATmedia,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
      sym_identifier,
  [10679] = 10,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(347), 1,
      anon_sym_LPAREN,
    ACTIONS(353), 1,
      anon_sym_DOT,
    ACTIONS(638), 1,
      anon_sym_COMMA,
    ACTIONS(640), 1,
      anon_sym_LBRACE,
    STATE(126), 1,
      sym_block,
    STATE(204), 1,
      sym_comment,
    ACTIONS(422), 3,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
    ACTIONS(420), 10,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
  [10721] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(205), 1,
      sym_comment,
    ACTIONS(345), 3,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
    ACTIONS(343), 15,
      anon_sym_LPAREN,
      anon_sym_RBRACE,
      anon_sym_DOT,
      anon_sym_DASH,
      anon_sym_ATmedia,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
      sym_identifier,
  [10753] = 7,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(630), 1,
      anon_sym_LPAREN,
    ACTIONS(632), 1,
      anon_sym_DOT,
    STATE(206), 1,
      sym_comment,
    ACTIONS(351), 3,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
    ACTIONS(349), 13,
      anon_sym_RBRACE,
      anon_sym_DASH,
      anon_sym_ATmedia,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
      sym_identifier,
  [10789] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(207), 1,
      sym_comment,
    ACTIONS(357), 3,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
    ACTIONS(355), 15,
      anon_sym_LPAREN,
      anon_sym_RBRACE,
      anon_sym_DOT,
      anon_sym_DASH,
      anon_sym_ATmedia,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
      sym_identifier,
  [10821] = 6,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(642), 1,
      anon_sym_COLON,
    STATE(208), 1,
      sym_comment,
    ACTIONS(317), 3,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
    ACTIONS(315), 14,
      anon_sym_LPAREN,
      anon_sym_RPAREN,
      anon_sym_COMMA,
      anon_sym_DOT,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
  [10855] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(209), 1,
      sym_comment,
    ACTIONS(361), 3,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
    ACTIONS(359), 15,
      anon_sym_LPAREN,
      anon_sym_RBRACE,
      anon_sym_DOT,
      anon_sym_DASH,
      anon_sym_ATmedia,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
      sym_identifier,
  [10887] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(210), 1,
      sym_comment,
    ACTIONS(365), 3,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
    ACTIONS(363), 15,
      anon_sym_LPAREN,
      anon_sym_RBRACE,
      anon_sym_DOT,
      anon_sym_DASH,
      anon_sym_ATmedia,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
      sym_identifier,
  [10919] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(211), 1,
      sym_comment,
    ACTIONS(369), 3,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
    ACTIONS(367), 15,
      anon_sym_LPAREN,
      anon_sym_RBRACE,
      anon_sym_DOT,
      anon_sym_DASH,
      anon_sym_ATmedia,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
      sym_identifier,
  [10951] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(212), 1,
      sym_comment,
    ACTIONS(377), 3,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
    ACTIONS(375), 15,
      anon_sym_LPAREN,
      anon_sym_RBRACE,
      anon_sym_DOT,
      anon_sym_DASH,
      anon_sym_ATmedia,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
      sym_identifier,
  [10983] = 8,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(630), 1,
      anon_sym_LPAREN,
    ACTIONS(632), 1,
      anon_sym_DOT,
    STATE(213), 1,
      sym_comment,
    ACTIONS(644), 3,
      anon_sym_RBRACE,
      anon_sym_ATmedia,
      sym_identifier,
    ACTIONS(648), 3,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
    ACTIONS(646), 10,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
  [11021] = 10,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(347), 1,
      anon_sym_LPAREN,
    ACTIONS(353), 1,
      anon_sym_DOT,
    ACTIONS(640), 1,
      anon_sym_LBRACE,
    ACTIONS(650), 1,
      anon_sym_COMMA,
    STATE(141), 1,
      sym_block,
    STATE(214), 1,
      sym_comment,
    ACTIONS(422), 3,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
    ACTIONS(420), 10,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
  [11063] = 10,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(347), 1,
      anon_sym_LPAREN,
    ACTIONS(353), 1,
      anon_sym_DOT,
    ACTIONS(652), 1,
      anon_sym_COMMA,
    ACTIONS(654), 1,
      anon_sym_LBRACE,
    STATE(64), 1,
      sym_block,
    STATE(215), 1,
      sym_comment,
    ACTIONS(422), 3,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
    ACTIONS(420), 10,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
  [11105] = 10,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(347), 1,
      anon_sym_LPAREN,
    ACTIONS(353), 1,
      anon_sym_DOT,
    ACTIONS(656), 1,
      anon_sym_COMMA,
    ACTIONS(658), 1,
      anon_sym_LBRACE,
    STATE(76), 1,
      sym_block,
    STATE(216), 1,
      sym_comment,
    ACTIONS(422), 3,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
    ACTIONS(420), 10,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
  [11147] = 10,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(347), 1,
      anon_sym_LPAREN,
    ACTIONS(353), 1,
      anon_sym_DOT,
    ACTIONS(634), 1,
      anon_sym_COMMA,
    ACTIONS(660), 1,
      anon_sym_RBRACK,
    STATE(217), 1,
      sym_comment,
    STATE(304), 1,
      aux_sym_array_repeat1,
    ACTIONS(422), 3,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
    ACTIONS(420), 10,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
  [11189] = 10,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(347), 1,
      anon_sym_LPAREN,
    ACTIONS(353), 1,
      anon_sym_DOT,
    ACTIONS(658), 1,
      anon_sym_LBRACE,
    ACTIONS(662), 1,
      anon_sym_COMMA,
    STATE(89), 1,
      sym_block,
    STATE(218), 1,
      sym_comment,
    ACTIONS(422), 3,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
    ACTIONS(420), 10,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
  [11231] = 10,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(347), 1,
      anon_sym_LPAREN,
    ACTIONS(353), 1,
      anon_sym_DOT,
    ACTIONS(658), 1,
      anon_sym_LBRACE,
    ACTIONS(664), 1,
      anon_sym_COMMA,
    STATE(96), 1,
      sym_block,
    STATE(219), 1,
      sym_comment,
    ACTIONS(422), 3,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
    ACTIONS(420), 10,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
  [11273] = 10,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(347), 1,
      anon_sym_LPAREN,
    ACTIONS(353), 1,
      anon_sym_DOT,
    ACTIONS(634), 1,
      anon_sym_COMMA,
    ACTIONS(666), 1,
      anon_sym_RBRACK,
    STATE(220), 1,
      sym_comment,
    STATE(313), 1,
      aux_sym_array_repeat1,
    ACTIONS(422), 3,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
    ACTIONS(420), 10,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
  [11315] = 10,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(347), 1,
      anon_sym_LPAREN,
    ACTIONS(353), 1,
      anon_sym_DOT,
    ACTIONS(668), 1,
      anon_sym_COMMA,
    ACTIONS(670), 1,
      anon_sym_LBRACE,
    STATE(108), 1,
      sym_block,
    STATE(221), 1,
      sym_comment,
    ACTIONS(422), 3,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
    ACTIONS(420), 10,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
  [11357] = 10,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(347), 1,
      anon_sym_LPAREN,
    ACTIONS(353), 1,
      anon_sym_DOT,
    ACTIONS(640), 1,
      anon_sym_LBRACE,
    ACTIONS(672), 1,
      anon_sym_COMMA,
    STATE(120), 1,
      sym_block,
    STATE(222), 1,
      sym_comment,
    ACTIONS(422), 3,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
    ACTIONS(420), 10,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
  [11399] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(223), 1,
      sym_comment,
    ACTIONS(317), 3,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
    ACTIONS(315), 15,
      anon_sym_LPAREN,
      anon_sym_RBRACE,
      anon_sym_DOT,
      anon_sym_DASH,
      anon_sym_ATmedia,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
      sym_identifier,
  [11431] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(224), 1,
      sym_comment,
    ACTIONS(373), 3,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
    ACTIONS(371), 15,
      anon_sym_LPAREN,
      anon_sym_RBRACE,
      anon_sym_DOT,
      anon_sym_DASH,
      anon_sym_ATmedia,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
      sym_identifier,
  [11463] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(225), 1,
      sym_comment,
    ACTIONS(393), 3,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
    ACTIONS(391), 15,
      anon_sym_LPAREN,
      anon_sym_RBRACE,
      anon_sym_DOT,
      anon_sym_DASH,
      anon_sym_ATmedia,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
      sym_identifier,
  [11495] = 8,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(347), 1,
      anon_sym_LPAREN,
    ACTIONS(353), 1,
      anon_sym_DOT,
    STATE(226), 1,
      sym_comment,
    ACTIONS(674), 2,
      anon_sym_COMMA,
      anon_sym_RBRACE,
    ACTIONS(422), 3,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
    ACTIONS(420), 10,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
  [11532] = 8,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(347), 1,
      anon_sym_LPAREN,
    ACTIONS(353), 1,
      anon_sym_DOT,
    STATE(227), 1,
      sym_comment,
    ACTIONS(676), 2,
      anon_sym_RPAREN,
      anon_sym_COMMA,
    ACTIONS(422), 3,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
    ACTIONS(420), 10,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
  [11569] = 6,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(136), 1,
      sym_fetch_block,
    STATE(228), 1,
      sym_comment,
    ACTIONS(341), 3,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
    ACTIONS(339), 13,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_DOT,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
  [11602] = 8,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(347), 1,
      anon_sym_LPAREN,
    ACTIONS(353), 1,
      anon_sym_DOT,
    STATE(229), 1,
      sym_comment,
    ACTIONS(678), 2,
      anon_sym_RPAREN,
      anon_sym_COMMA,
    ACTIONS(422), 3,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
    ACTIONS(420), 10,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
  [11639] = 8,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(347), 1,
      anon_sym_LPAREN,
    ACTIONS(353), 1,
      anon_sym_DOT,
    STATE(230), 1,
      sym_comment,
    ACTIONS(680), 2,
      anon_sym_RPAREN,
      anon_sym_COMMA,
    ACTIONS(422), 3,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
    ACTIONS(420), 10,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
  [11676] = 9,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(353), 1,
      anon_sym_DOT,
    ACTIONS(682), 1,
      anon_sym_LPAREN,
    ACTIONS(684), 1,
      anon_sym_LBRACE,
    STATE(91), 1,
      sym_fetch_block,
    STATE(231), 1,
      sym_comment,
    ACTIONS(422), 3,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
    ACTIONS(420), 10,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
  [11715] = 8,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(347), 1,
      anon_sym_LPAREN,
    ACTIONS(353), 1,
      anon_sym_DOT,
    STATE(232), 1,
      sym_comment,
    ACTIONS(686), 2,
      anon_sym_RPAREN,
      anon_sym_COMMA,
    ACTIONS(422), 3,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
    ACTIONS(420), 10,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
  [11752] = 6,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(97), 1,
      sym_fetch_block,
    STATE(233), 1,
      sym_comment,
    ACTIONS(341), 3,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
    ACTIONS(339), 13,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_DOT,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
  [11785] = 6,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(99), 1,
      sym_fetch_block,
    STATE(234), 1,
      sym_comment,
    ACTIONS(369), 3,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
    ACTIONS(367), 13,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_DOT,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
  [11818] = 6,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(121), 1,
      sym_fetch_block,
    STATE(235), 1,
      sym_comment,
    ACTIONS(369), 3,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
    ACTIONS(367), 13,
      anon_sym_LPAREN,
      anon_sym_LBRACE,
      anon_sym_DOT,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
  [11851] = 8,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(347), 1,
      anon_sym_LPAREN,
    ACTIONS(353), 1,
      anon_sym_DOT,
    STATE(236), 1,
      sym_comment,
    ACTIONS(688), 2,
      anon_sym_RPAREN,
      anon_sym_COMMA,
    ACTIONS(422), 3,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
    ACTIONS(420), 10,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
  [11888] = 8,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(347), 1,
      anon_sym_LPAREN,
    ACTIONS(353), 1,
      anon_sym_DOT,
    STATE(237), 1,
      sym_comment,
    ACTIONS(690), 2,
      anon_sym_COMMA,
      anon_sym_RBRACK,
    ACTIONS(422), 3,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
    ACTIONS(420), 10,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
  [11925] = 8,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(630), 1,
      anon_sym_LPAREN,
    ACTIONS(632), 1,
      anon_sym_DOT,
    STATE(238), 1,
      sym_comment,
    ACTIONS(692), 2,
      anon_sym_RBRACE,
      sym_identifier,
    ACTIONS(648), 3,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
    ACTIONS(646), 10,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
  [11962] = 8,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(347), 1,
      anon_sym_LPAREN,
    ACTIONS(353), 1,
      anon_sym_DOT,
    STATE(239), 1,
      sym_comment,
    ACTIONS(694), 2,
      anon_sym_RPAREN,
      anon_sym_COMMA,
    ACTIONS(422), 3,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
    ACTIONS(420), 10,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
  [11999] = 9,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(353), 1,
      anon_sym_DOT,
    ACTIONS(696), 1,
      anon_sym_LPAREN,
    ACTIONS(698), 1,
      anon_sym_LBRACE,
    STATE(130), 1,
      sym_fetch_block,
    STATE(240), 1,
      sym_comment,
    ACTIONS(422), 3,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
    ACTIONS(420), 10,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
  [12038] = 8,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(347), 1,
      anon_sym_LPAREN,
    ACTIONS(353), 1,
      anon_sym_DOT,
    ACTIONS(700), 1,
      anon_sym_RPAREN,
    STATE(241), 1,
      sym_comment,
    ACTIONS(422), 3,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
    ACTIONS(420), 10,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
  [12074] = 8,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(347), 1,
      anon_sym_LPAREN,
    ACTIONS(353), 1,
      anon_sym_DOT,
    ACTIONS(702), 1,
      anon_sym_RPAREN,
    STATE(242), 1,
      sym_comment,
    ACTIONS(422), 3,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
    ACTIONS(420), 10,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
  [12110] = 8,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(347), 1,
      anon_sym_LPAREN,
    ACTIONS(353), 1,
      anon_sym_DOT,
    ACTIONS(704), 1,
      anon_sym_RPAREN,
    STATE(243), 1,
      sym_comment,
    ACTIONS(422), 3,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
    ACTIONS(420), 10,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
  [12146] = 8,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(347), 1,
      anon_sym_LPAREN,
    ACTIONS(353), 1,
      anon_sym_DOT,
    ACTIONS(706), 1,
      anon_sym_RBRACE,
    STATE(244), 1,
      sym_comment,
    ACTIONS(422), 3,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
    ACTIONS(420), 10,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
  [12182] = 8,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(347), 1,
      anon_sym_LPAREN,
    ACTIONS(353), 1,
      anon_sym_DOT,
    ACTIONS(708), 1,
      anon_sym_RPAREN,
    STATE(245), 1,
      sym_comment,
    ACTIONS(422), 3,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
    ACTIONS(420), 10,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
  [12218] = 8,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(347), 1,
      anon_sym_LPAREN,
    ACTIONS(353), 1,
      anon_sym_DOT,
    ACTIONS(710), 1,
      anon_sym_RPAREN,
    STATE(246), 1,
      sym_comment,
    ACTIONS(422), 3,
      anon_sym_SLASH,
      anon_sym_LT,
      anon_sym_GT,
    ACTIONS(420), 10,
      anon_sym_DASH,
      anon_sym_PLUS,
      anon_sym_STAR,
      anon_sym_PERCENT,
      anon_sym_EQ_EQ,
      anon_sym_BANG_EQ,
      anon_sym_LT_EQ,
      anon_sym_GT_EQ,
      anon_sym_AMP_AMP,
      anon_sym_PIPE_PIPE,
  [12254] = 11,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(9), 1,
      anon_sym_Page,
    ACTIONS(11), 1,
      anon_sym_Component,
    ACTIONS(13), 1,
      anon_sym_Store,
    ACTIONS(15), 1,
      anon_sym_App,
    ACTIONS(712), 1,
      ts_builtin_sym_end,
    STATE(247), 1,
      sym_comment,
    STATE(248), 1,
      aux_sym_source_file_repeat1,
    STATE(274), 1,
      sym__declaration,
    STATE(269), 4,
      sym_page_declaration,
      sym_component_declaration,
      sym_store_declaration,
      sym_app_declaration,
  [12291] = 10,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(714), 1,
      ts_builtin_sym_end,
    ACTIONS(716), 1,
      anon_sym_Page,
    ACTIONS(719), 1,
      anon_sym_Component,
    ACTIONS(722), 1,
      anon_sym_Store,
    ACTIONS(725), 1,
      anon_sym_App,
    STATE(274), 1,
      sym__declaration,
    STATE(248), 2,
      sym_comment,
      aux_sym_source_file_repeat1,
    STATE(269), 4,
      sym_page_declaration,
      sym_component_declaration,
      sym_store_declaration,
      sym_app_declaration,
  [12326] = 8,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(728), 1,
      anon_sym_RBRACE,
    ACTIONS(730), 1,
      anon_sym_ATmedia,
    ACTIONS(733), 1,
      sym_identifier,
    STATE(384), 1,
      sym_property_name,
    STATE(249), 2,
      sym_comment,
      aux_sym_style_block_repeat1,
    STATE(292), 2,
      sym_style_property,
      sym_media_query,
  [12353] = 9,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(736), 1,
      anon_sym_RBRACE,
    ACTIONS(738), 1,
      anon_sym_ATmedia,
    ACTIONS(740), 1,
      sym_identifier,
    STATE(249), 1,
      aux_sym_style_block_repeat1,
    STATE(250), 1,
      sym_comment,
    STATE(384), 1,
      sym_property_name,
    STATE(292), 2,
      sym_style_property,
      sym_media_query,
  [12382] = 9,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(738), 1,
      anon_sym_ATmedia,
    ACTIONS(740), 1,
      sym_identifier,
    ACTIONS(742), 1,
      anon_sym_RBRACE,
    STATE(250), 1,
      aux_sym_style_block_repeat1,
    STATE(251), 1,
      sym_comment,
    STATE(384), 1,
      sym_property_name,
    STATE(292), 2,
      sym_style_property,
      sym_media_query,
  [12411] = 9,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(738), 1,
      anon_sym_ATmedia,
    ACTIONS(740), 1,
      sym_identifier,
    ACTIONS(744), 1,
      anon_sym_RBRACE,
    STATE(249), 1,
      aux_sym_style_block_repeat1,
    STATE(252), 1,
      sym_comment,
    STATE(384), 1,
      sym_property_name,
    STATE(292), 2,
      sym_style_property,
      sym_media_query,
  [12440] = 9,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(738), 1,
      anon_sym_ATmedia,
    ACTIONS(740), 1,
      sym_identifier,
    ACTIONS(746), 1,
      anon_sym_RBRACE,
    STATE(252), 1,
      aux_sym_style_block_repeat1,
    STATE(253), 1,
      sym_comment,
    STATE(384), 1,
      sym_property_name,
    STATE(292), 2,
      sym_style_property,
      sym_media_query,
  [12469] = 8,
    ACTIONS(748), 1,
      anon_sym_LBRACE,
    ACTIONS(750), 1,
      anon_sym_DQUOTE,
    ACTIONS(754), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(756), 1,
      anon_sym_SLASH_STAR,
    STATE(254), 1,
      sym_comment,
    STATE(257), 1,
      aux_sym_string_repeat1,
    STATE(288), 1,
      sym_interpolation,
    ACTIONS(752), 2,
      sym_string_content,
      sym_escape_sequence,
  [12495] = 8,
    ACTIONS(748), 1,
      anon_sym_LBRACE,
    ACTIONS(754), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(756), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(758), 1,
      anon_sym_DQUOTE,
    STATE(255), 1,
      sym_comment,
    STATE(259), 1,
      aux_sym_string_repeat1,
    STATE(288), 1,
      sym_interpolation,
    ACTIONS(752), 2,
      sym_string_content,
      sym_escape_sequence,
  [12521] = 8,
    ACTIONS(748), 1,
      anon_sym_LBRACE,
    ACTIONS(754), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(756), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(760), 1,
      anon_sym_DQUOTE,
    STATE(254), 1,
      aux_sym_string_repeat1,
    STATE(256), 1,
      sym_comment,
    STATE(288), 1,
      sym_interpolation,
    ACTIONS(752), 2,
      sym_string_content,
      sym_escape_sequence,
  [12547] = 7,
    ACTIONS(754), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(756), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(762), 1,
      anon_sym_LBRACE,
    ACTIONS(765), 1,
      anon_sym_DQUOTE,
    STATE(288), 1,
      sym_interpolation,
    ACTIONS(767), 2,
      sym_string_content,
      sym_escape_sequence,
    STATE(257), 2,
      sym_comment,
      aux_sym_string_repeat1,
  [12571] = 8,
    ACTIONS(748), 1,
      anon_sym_LBRACE,
    ACTIONS(754), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(756), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(770), 1,
      anon_sym_DQUOTE,
    STATE(258), 1,
      sym_comment,
    STATE(260), 1,
      aux_sym_string_repeat1,
    STATE(288), 1,
      sym_interpolation,
    ACTIONS(752), 2,
      sym_string_content,
      sym_escape_sequence,
  [12597] = 8,
    ACTIONS(748), 1,
      anon_sym_LBRACE,
    ACTIONS(754), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(756), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(772), 1,
      anon_sym_DQUOTE,
    STATE(257), 1,
      aux_sym_string_repeat1,
    STATE(259), 1,
      sym_comment,
    STATE(288), 1,
      sym_interpolation,
    ACTIONS(752), 2,
      sym_string_content,
      sym_escape_sequence,
  [12623] = 8,
    ACTIONS(748), 1,
      anon_sym_LBRACE,
    ACTIONS(754), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(756), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(774), 1,
      anon_sym_DQUOTE,
    STATE(257), 1,
      aux_sym_string_repeat1,
    STATE(260), 1,
      sym_comment,
    STATE(288), 1,
      sym_interpolation,
    ACTIONS(752), 2,
      sym_string_content,
      sym_escape_sequence,
  [12649] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(261), 1,
      sym_comment,
    ACTIONS(776), 5,
      ts_builtin_sym_end,
      anon_sym_Page,
      anon_sym_Component,
      anon_sym_Store,
      anon_sym_App,
  [12666] = 8,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(740), 1,
      sym_identifier,
    ACTIONS(778), 1,
      anon_sym_RBRACE,
    STATE(262), 1,
      sym_comment,
    STATE(266), 1,
      aux_sym_media_query_repeat1,
    STATE(353), 1,
      sym_style_property,
    STATE(384), 1,
      sym_property_name,
  [12691] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(263), 1,
      sym_comment,
    ACTIONS(780), 5,
      ts_builtin_sym_end,
      anon_sym_Page,
      anon_sym_Component,
      anon_sym_Store,
      anon_sym_App,
  [12708] = 7,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(784), 1,
      anon_sym_QMARK,
    ACTIONS(786), 1,
      anon_sym_COLON,
    ACTIONS(788), 1,
      anon_sym_EQ,
    STATE(264), 1,
      sym_comment,
    ACTIONS(782), 2,
      anon_sym_RPAREN,
      anon_sym_COMMA,
  [12731] = 8,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(740), 1,
      sym_identifier,
    ACTIONS(790), 1,
      anon_sym_RBRACE,
    STATE(262), 1,
      aux_sym_media_query_repeat1,
    STATE(265), 1,
      sym_comment,
    STATE(353), 1,
      sym_style_property,
    STATE(384), 1,
      sym_property_name,
  [12756] = 7,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(792), 1,
      anon_sym_RBRACE,
    ACTIONS(794), 1,
      sym_identifier,
    STATE(353), 1,
      sym_style_property,
    STATE(384), 1,
      sym_property_name,
    STATE(266), 2,
      sym_comment,
      aux_sym_media_query_repeat1,
  [12779] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(267), 1,
      sym_comment,
    ACTIONS(797), 5,
      ts_builtin_sym_end,
      anon_sym_Page,
      anon_sym_Component,
      anon_sym_Store,
      anon_sym_App,
  [12796] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(268), 1,
      sym_comment,
    ACTIONS(452), 5,
      ts_builtin_sym_end,
      anon_sym_Page,
      anon_sym_Component,
      anon_sym_Store,
      anon_sym_App,
  [12813] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(269), 1,
      sym_comment,
    ACTIONS(799), 5,
      ts_builtin_sym_end,
      anon_sym_Page,
      anon_sym_Component,
      anon_sym_Store,
      anon_sym_App,
  [12830] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(270), 1,
      sym_comment,
    ACTIONS(430), 5,
      ts_builtin_sym_end,
      anon_sym_Page,
      anon_sym_Component,
      anon_sym_Store,
      anon_sym_App,
  [12847] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(271), 1,
      sym_comment,
    ACTIONS(801), 5,
      ts_builtin_sym_end,
      anon_sym_Page,
      anon_sym_Component,
      anon_sym_Store,
      anon_sym_App,
  [12864] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(272), 1,
      sym_comment,
    ACTIONS(803), 5,
      ts_builtin_sym_end,
      anon_sym_Page,
      anon_sym_Component,
      anon_sym_Store,
      anon_sym_App,
  [12881] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(273), 1,
      sym_comment,
    ACTIONS(805), 5,
      ts_builtin_sym_end,
      anon_sym_Page,
      anon_sym_Component,
      anon_sym_Store,
      anon_sym_App,
  [12898] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(274), 1,
      sym_comment,
    ACTIONS(807), 5,
      ts_builtin_sym_end,
      anon_sym_Page,
      anon_sym_Component,
      anon_sym_Store,
      anon_sym_App,
  [12915] = 4,
    ACTIONS(754), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(756), 1,
      anon_sym_SLASH_STAR,
    STATE(275), 1,
      sym_comment,
    ACTIONS(809), 4,
      anon_sym_LBRACE,
      anon_sym_DQUOTE,
      sym_string_content,
      sym_escape_sequence,
  [12931] = 7,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(811), 1,
      anon_sym_RPAREN,
    ACTIONS(813), 1,
      sym_identifier,
    STATE(276), 1,
      sym_comment,
    STATE(299), 1,
      sym_parameter,
    STATE(415), 1,
      sym_parameter_list,
  [12953] = 6,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(640), 1,
      anon_sym_LBRACE,
    ACTIONS(815), 1,
      anon_sym_if,
    STATE(277), 1,
      sym_comment,
    STATE(131), 2,
      sym_block,
      sym_if_statement,
  [12973] = 6,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(819), 1,
      anon_sym_COLON,
    ACTIONS(821), 1,
      anon_sym_EQ,
    STATE(278), 1,
      sym_comment,
    ACTIONS(817), 2,
      anon_sym_RPAREN,
      anon_sym_COMMA,
  [12993] = 6,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(823), 1,
      anon_sym_RBRACE,
    ACTIONS(825), 1,
      sym_identifier,
    STATE(321), 1,
      sym_transition_property,
    STATE(279), 2,
      sym_comment,
      aux_sym_transition_block_repeat1,
  [13013] = 7,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(813), 1,
      sym_identifier,
    ACTIONS(828), 1,
      anon_sym_RPAREN,
    STATE(280), 1,
      sym_comment,
    STATE(299), 1,
      sym_parameter,
    STATE(382), 1,
      sym_parameter_list,
  [13035] = 7,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(830), 1,
      anon_sym_RBRACE,
    ACTIONS(832), 1,
      sym_identifier,
    STATE(279), 1,
      aux_sym_transition_block_repeat1,
    STATE(281), 1,
      sym_comment,
    STATE(321), 1,
      sym_transition_property,
  [13057] = 7,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(832), 1,
      sym_identifier,
    ACTIONS(834), 1,
      anon_sym_RBRACE,
    STATE(282), 1,
      sym_comment,
    STATE(284), 1,
      aux_sym_transition_block_repeat1,
    STATE(321), 1,
      sym_transition_property,
  [13079] = 6,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(640), 1,
      anon_sym_LBRACE,
    ACTIONS(815), 1,
      anon_sym_if,
    STATE(283), 1,
      sym_comment,
    STATE(125), 2,
      sym_block,
      sym_if_statement,
  [13099] = 7,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(832), 1,
      sym_identifier,
    ACTIONS(836), 1,
      anon_sym_RBRACE,
    STATE(279), 1,
      aux_sym_transition_block_repeat1,
    STATE(284), 1,
      sym_comment,
    STATE(321), 1,
      sym_transition_property,
  [13121] = 6,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(658), 1,
      anon_sym_LBRACE,
    ACTIONS(838), 1,
      anon_sym_if,
    STATE(285), 1,
      sym_comment,
    STATE(88), 2,
      sym_block,
      sym_if_statement,
  [13141] = 6,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(658), 1,
      anon_sym_LBRACE,
    ACTIONS(838), 1,
      anon_sym_if,
    STATE(286), 1,
      sym_comment,
    STATE(95), 2,
      sym_block,
      sym_if_statement,
  [13161] = 7,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(832), 1,
      sym_identifier,
    ACTIONS(840), 1,
      anon_sym_RBRACE,
    STATE(281), 1,
      aux_sym_transition_block_repeat1,
    STATE(287), 1,
      sym_comment,
    STATE(321), 1,
      sym_transition_property,
  [13183] = 4,
    ACTIONS(754), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(756), 1,
      anon_sym_SLASH_STAR,
    STATE(288), 1,
      sym_comment,
    ACTIONS(842), 4,
      anon_sym_LBRACE,
      anon_sym_DQUOTE,
      sym_string_content,
      sym_escape_sequence,
  [13199] = 7,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(813), 1,
      sym_identifier,
    ACTIONS(844), 1,
      anon_sym_RPAREN,
    STATE(289), 1,
      sym_comment,
    STATE(299), 1,
      sym_parameter,
    STATE(421), 1,
      sym_parameter_list,
  [13221] = 6,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(846), 1,
      anon_sym_RBRACE,
    ACTIONS(848), 1,
      sym_identifier,
    STATE(290), 1,
      sym_comment,
    STATE(297), 1,
      sym_object_entry,
  [13240] = 6,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(850), 1,
      anon_sym_COLON,
    ACTIONS(852), 1,
      anon_sym_DASH,
    STATE(291), 1,
      sym_comment,
    STATE(296), 1,
      aux_sym_property_name_repeat1,
  [13259] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(292), 1,
      sym_comment,
    ACTIONS(854), 3,
      anon_sym_RBRACE,
      anon_sym_ATmedia,
      sym_identifier,
  [13274] = 6,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(856), 1,
      anon_sym_RPAREN,
    ACTIONS(858), 1,
      anon_sym_COMMA,
    STATE(293), 1,
      sym_comment,
    STATE(314), 1,
      aux_sym_argument_list_repeat1,
  [13293] = 6,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(634), 1,
      anon_sym_COMMA,
    ACTIONS(860), 1,
      anon_sym_RBRACK,
    STATE(294), 1,
      sym_comment,
    STATE(300), 1,
      aux_sym_array_repeat1,
  [13312] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(862), 1,
      anon_sym_RPAREN,
    ACTIONS(864), 1,
      anon_sym_COMMA,
    STATE(295), 2,
      sym_comment,
      aux_sym_parameter_list_repeat1,
  [13329] = 6,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(852), 1,
      anon_sym_DASH,
    ACTIONS(867), 1,
      anon_sym_COLON,
    STATE(296), 1,
      sym_comment,
    STATE(302), 1,
      aux_sym_property_name_repeat1,
  [13348] = 6,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(869), 1,
      anon_sym_COMMA,
    ACTIONS(871), 1,
      anon_sym_RBRACE,
    STATE(297), 1,
      sym_comment,
    STATE(301), 1,
      aux_sym_object_repeat1,
  [13367] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(298), 1,
      sym_comment,
    ACTIONS(873), 3,
      anon_sym_RBRACE,
      anon_sym_ATmedia,
      sym_identifier,
  [13382] = 6,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(875), 1,
      anon_sym_RPAREN,
    ACTIONS(877), 1,
      anon_sym_COMMA,
    STATE(299), 1,
      sym_comment,
    STATE(303), 1,
      aux_sym_parameter_list_repeat1,
  [13401] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(690), 1,
      anon_sym_RBRACK,
    ACTIONS(879), 1,
      anon_sym_COMMA,
    STATE(300), 2,
      sym_comment,
      aux_sym_array_repeat1,
  [13418] = 6,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(869), 1,
      anon_sym_COMMA,
    ACTIONS(882), 1,
      anon_sym_RBRACE,
    STATE(301), 1,
      sym_comment,
    STATE(310), 1,
      aux_sym_object_repeat1,
  [13437] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(884), 1,
      anon_sym_COLON,
    ACTIONS(886), 1,
      anon_sym_DASH,
    STATE(302), 2,
      sym_comment,
      aux_sym_property_name_repeat1,
  [13454] = 6,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(877), 1,
      anon_sym_COMMA,
    ACTIONS(889), 1,
      anon_sym_RPAREN,
    STATE(295), 1,
      aux_sym_parameter_list_repeat1,
    STATE(303), 1,
      sym_comment,
  [13473] = 6,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(634), 1,
      anon_sym_COMMA,
    ACTIONS(891), 1,
      anon_sym_RBRACK,
    STATE(300), 1,
      aux_sym_array_repeat1,
    STATE(304), 1,
      sym_comment,
  [13492] = 6,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(658), 1,
      anon_sym_LBRACE,
    ACTIONS(893), 1,
      anon_sym_LPAREN,
    STATE(104), 1,
      sym_block,
    STATE(305), 1,
      sym_comment,
  [13511] = 6,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(869), 1,
      anon_sym_COMMA,
    ACTIONS(895), 1,
      anon_sym_RBRACE,
    STATE(306), 1,
      sym_comment,
    STATE(310), 1,
      aux_sym_object_repeat1,
  [13530] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(307), 1,
      sym_comment,
    ACTIONS(897), 3,
      anon_sym_RBRACE,
      anon_sym_ATmedia,
      sym_identifier,
  [13545] = 6,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(848), 1,
      sym_identifier,
    ACTIONS(899), 1,
      anon_sym_RBRACE,
    STATE(308), 1,
      sym_comment,
    STATE(311), 1,
      sym_object_entry,
  [13564] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(901), 1,
      anon_sym_EQ,
    STATE(309), 1,
      sym_comment,
    ACTIONS(686), 2,
      anon_sym_RPAREN,
      anon_sym_COMMA,
  [13581] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(903), 1,
      anon_sym_COMMA,
    ACTIONS(906), 1,
      anon_sym_RBRACE,
    STATE(310), 2,
      sym_comment,
      aux_sym_object_repeat1,
  [13598] = 6,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(869), 1,
      anon_sym_COMMA,
    ACTIONS(908), 1,
      anon_sym_RBRACE,
    STATE(311), 1,
      sym_comment,
    STATE(312), 1,
      aux_sym_object_repeat1,
  [13617] = 6,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(869), 1,
      anon_sym_COMMA,
    ACTIONS(910), 1,
      anon_sym_RBRACE,
    STATE(310), 1,
      aux_sym_object_repeat1,
    STATE(312), 1,
      sym_comment,
  [13636] = 6,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(634), 1,
      anon_sym_COMMA,
    ACTIONS(912), 1,
      anon_sym_RBRACK,
    STATE(300), 1,
      aux_sym_array_repeat1,
    STATE(313), 1,
      sym_comment,
  [13655] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(914), 1,
      anon_sym_RPAREN,
    ACTIONS(916), 1,
      anon_sym_COMMA,
    STATE(314), 2,
      sym_comment,
      aux_sym_argument_list_repeat1,
  [13672] = 6,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(848), 1,
      sym_identifier,
    ACTIONS(919), 1,
      anon_sym_RBRACE,
    STATE(315), 1,
      sym_comment,
    STATE(317), 1,
      sym_object_entry,
  [13691] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(921), 1,
      anon_sym_EQ,
    STATE(316), 1,
      sym_comment,
    ACTIONS(694), 2,
      anon_sym_RPAREN,
      anon_sym_COMMA,
  [13708] = 6,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(869), 1,
      anon_sym_COMMA,
    ACTIONS(923), 1,
      anon_sym_RBRACE,
    STATE(306), 1,
      aux_sym_object_repeat1,
    STATE(317), 1,
      sym_comment,
  [13727] = 6,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(858), 1,
      anon_sym_COMMA,
    ACTIONS(925), 1,
      anon_sym_RPAREN,
    STATE(293), 1,
      aux_sym_argument_list_repeat1,
    STATE(318), 1,
      sym_comment,
  [13746] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(927), 1,
      anon_sym_LBRACE,
    STATE(267), 1,
      sym_block,
    STATE(319), 1,
      sym_comment,
  [13762] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(640), 1,
      anon_sym_LBRACE,
    STATE(144), 1,
      sym_block,
    STATE(320), 1,
      sym_comment,
  [13778] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(321), 1,
      sym_comment,
    ACTIONS(929), 2,
      anon_sym_RBRACE,
      sym_identifier,
  [13792] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(927), 1,
      anon_sym_LBRACE,
    STATE(272), 1,
      sym_block,
    STATE(322), 1,
      sym_comment,
  [13808] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(931), 1,
      anon_sym_RPAREN,
    ACTIONS(933), 1,
      anon_sym_COMMA,
    STATE(323), 1,
      sym_comment,
  [13824] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(935), 1,
      anon_sym_animate,
    STATE(324), 1,
      sym_comment,
    STATE(332), 1,
      sym_animate_clause,
  [13840] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(927), 1,
      anon_sym_LBRACE,
    STATE(263), 1,
      sym_block,
    STATE(325), 1,
      sym_comment,
  [13856] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(640), 1,
      anon_sym_LBRACE,
    STATE(141), 1,
      sym_block,
    STATE(326), 1,
      sym_comment,
  [13872] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(658), 1,
      anon_sym_LBRACE,
    STATE(72), 1,
      sym_block,
    STATE(327), 1,
      sym_comment,
  [13888] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(658), 1,
      anon_sym_LBRACE,
    STATE(74), 1,
      sym_block,
    STATE(328), 1,
      sym_comment,
  [13904] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(927), 1,
      anon_sym_LBRACE,
    STATE(261), 1,
      sym_block,
    STATE(329), 1,
      sym_comment,
  [13920] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(670), 1,
      anon_sym_LBRACE,
    STATE(107), 1,
      sym_block,
    STATE(330), 1,
      sym_comment,
  [13936] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(937), 1,
      anon_sym_COMMA,
    ACTIONS(939), 1,
      anon_sym_in,
    STATE(331), 1,
      sym_comment,
  [13952] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(640), 1,
      anon_sym_LBRACE,
    STATE(135), 1,
      sym_block,
    STATE(332), 1,
      sym_comment,
  [13968] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(658), 1,
      anon_sym_LBRACE,
    STATE(105), 1,
      sym_block,
    STATE(333), 1,
      sym_comment,
  [13984] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(927), 1,
      anon_sym_LBRACE,
    STATE(273), 1,
      sym_block,
    STATE(334), 1,
      sym_comment,
  [14000] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(941), 1,
      anon_sym_RPAREN,
    ACTIONS(943), 1,
      anon_sym_COMMA,
    STATE(335), 1,
      sym_comment,
  [14016] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(813), 1,
      sym_identifier,
    STATE(336), 1,
      sym_comment,
    STATE(366), 1,
      sym_parameter,
  [14032] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(658), 1,
      anon_sym_LBRACE,
    STATE(86), 1,
      sym_block,
    STATE(337), 1,
      sym_comment,
  [14048] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(654), 1,
      anon_sym_LBRACE,
    STATE(69), 1,
      sym_block,
    STATE(338), 1,
      sym_comment,
  [14064] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(658), 1,
      anon_sym_LBRACE,
    STATE(90), 1,
      sym_block,
    STATE(339), 1,
      sym_comment,
  [14080] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(658), 1,
      anon_sym_LBRACE,
    STATE(85), 1,
      sym_block,
    STATE(340), 1,
      sym_comment,
  [14096] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(658), 1,
      anon_sym_LBRACE,
    STATE(93), 1,
      sym_block,
    STATE(341), 1,
      sym_comment,
  [14112] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(658), 1,
      anon_sym_LBRACE,
    STATE(96), 1,
      sym_block,
    STATE(342), 1,
      sym_comment,
  [14128] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(935), 1,
      anon_sym_animate,
    STATE(343), 1,
      sym_comment,
    STATE(357), 1,
      sym_animate_clause,
  [14144] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(658), 1,
      anon_sym_LBRACE,
    STATE(80), 1,
      sym_block,
    STATE(344), 1,
      sym_comment,
  [14160] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(658), 1,
      anon_sym_LBRACE,
    STATE(100), 1,
      sym_block,
    STATE(345), 1,
      sym_comment,
  [14176] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(848), 1,
      sym_identifier,
    STATE(346), 1,
      sym_comment,
    STATE(360), 1,
      sym_object_entry,
  [14192] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(200), 1,
      anon_sym_DQUOTE,
    STATE(323), 1,
      sym_string,
    STATE(347), 1,
      sym_comment,
  [14208] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(640), 1,
      anon_sym_LBRACE,
    STATE(122), 1,
      sym_block,
    STATE(348), 1,
      sym_comment,
  [14224] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(640), 1,
      anon_sym_LBRACE,
    STATE(132), 1,
      sym_block,
    STATE(349), 1,
      sym_comment,
  [14240] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(350), 1,
      sym_comment,
    ACTIONS(914), 2,
      anon_sym_RPAREN,
      anon_sym_COMMA,
  [14254] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(945), 1,
      anon_sym_RPAREN,
    ACTIONS(947), 1,
      anon_sym_COMMA,
    STATE(351), 1,
      sym_comment,
  [14270] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(352), 1,
      sym_comment,
    ACTIONS(884), 2,
      anon_sym_COLON,
      anon_sym_DASH,
  [14284] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(353), 1,
      sym_comment,
    ACTIONS(949), 2,
      anon_sym_RBRACE,
      sym_identifier,
  [14298] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(935), 1,
      anon_sym_animate,
    STATE(330), 1,
      sym_animate_clause,
    STATE(354), 1,
      sym_comment,
  [14314] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(927), 1,
      anon_sym_LBRACE,
    STATE(271), 1,
      sym_block,
    STATE(355), 1,
      sym_comment,
  [14330] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(356), 1,
      sym_comment,
    ACTIONS(678), 2,
      anon_sym_RPAREN,
      anon_sym_COMMA,
  [14344] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(640), 1,
      anon_sym_LBRACE,
    STATE(127), 1,
      sym_block,
    STATE(357), 1,
      sym_comment,
  [14360] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(200), 1,
      anon_sym_DQUOTE,
    STATE(335), 1,
      sym_string,
    STATE(358), 1,
      sym_comment,
  [14376] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(935), 1,
      anon_sym_animate,
    STATE(338), 1,
      sym_animate_clause,
    STATE(359), 1,
      sym_comment,
  [14392] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(360), 1,
      sym_comment,
    ACTIONS(906), 2,
      anon_sym_COMMA,
      anon_sym_RBRACE,
  [14406] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(935), 1,
      anon_sym_animate,
    STATE(339), 1,
      sym_animate_clause,
    STATE(361), 1,
      sym_comment,
  [14422] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(935), 1,
      anon_sym_animate,
    STATE(326), 1,
      sym_animate_clause,
    STATE(362), 1,
      sym_comment,
  [14438] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(935), 1,
      anon_sym_animate,
    STATE(342), 1,
      sym_animate_clause,
    STATE(363), 1,
      sym_comment,
  [14454] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(935), 1,
      anon_sym_animate,
    STATE(345), 1,
      sym_animate_clause,
    STATE(364), 1,
      sym_comment,
  [14470] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(200), 1,
      anon_sym_DQUOTE,
    STATE(351), 1,
      sym_string,
    STATE(365), 1,
      sym_comment,
  [14486] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    STATE(366), 1,
      sym_comment,
    ACTIONS(862), 2,
      anon_sym_RPAREN,
      anon_sym_COMMA,
  [14500] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(951), 1,
      anon_sym_COMMA,
    ACTIONS(953), 1,
      anon_sym_in,
    STATE(367), 1,
      sym_comment,
  [14516] = 5,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(640), 1,
      anon_sym_LBRACE,
    STATE(147), 1,
      sym_block,
    STATE(368), 1,
      sym_comment,
  [14532] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(955), 1,
      anon_sym_RPAREN,
    STATE(369), 1,
      sym_comment,
  [14545] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(957), 1,
      anon_sym_RPAREN,
    STATE(370), 1,
      sym_comment,
  [14558] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(959), 1,
      anon_sym_RPAREN,
    STATE(371), 1,
      sym_comment,
  [14571] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(961), 1,
      sym_identifier,
    STATE(372), 1,
      sym_comment,
  [14584] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(963), 1,
      sym_identifier,
    STATE(373), 1,
      sym_comment,
  [14597] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(965), 1,
      sym_identifier,
    STATE(374), 1,
      sym_comment,
  [14610] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(967), 1,
      anon_sym_from,
    STATE(375), 1,
      sym_comment,
  [14623] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(969), 1,
      sym_identifier,
    STATE(376), 1,
      sym_comment,
  [14636] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(971), 1,
      anon_sym_LPAREN,
    STATE(377), 1,
      sym_comment,
  [14649] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(973), 1,
      sym_identifier,
    STATE(378), 1,
      sym_comment,
  [14662] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(975), 1,
      anon_sym_RPAREN,
    STATE(379), 1,
      sym_comment,
  [14675] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(977), 1,
      anon_sym_RPAREN,
    STATE(380), 1,
      sym_comment,
  [14688] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(979), 1,
      sym_identifier,
    STATE(381), 1,
      sym_comment,
  [14701] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(981), 1,
      anon_sym_RPAREN,
    STATE(382), 1,
      sym_comment,
  [14714] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(983), 1,
      anon_sym_SLASH,
    STATE(383), 1,
      sym_comment,
  [14727] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(985), 1,
      anon_sym_COLON,
    STATE(384), 1,
      sym_comment,
  [14740] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(987), 1,
      sym_identifier,
    STATE(385), 1,
      sym_comment,
  [14753] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(989), 1,
      anon_sym_LBRACE,
    STATE(386), 1,
      sym_comment,
  [14766] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(991), 1,
      anon_sym_LBRACE,
    STATE(387), 1,
      sym_comment,
  [14779] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(993), 1,
      anon_sym_LPAREN,
    STATE(388), 1,
      sym_comment,
  [14792] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(995), 1,
      sym_identifier,
    STATE(389), 1,
      sym_comment,
  [14805] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(997), 1,
      anon_sym_in,
    STATE(390), 1,
      sym_comment,
  [14818] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(999), 1,
      anon_sym_LPAREN,
    STATE(391), 1,
      sym_comment,
  [14831] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(1001), 1,
      sym_identifier,
    STATE(392), 1,
      sym_comment,
  [14844] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(1003), 1,
      anon_sym_LPAREN,
    STATE(393), 1,
      sym_comment,
  [14857] = 4,
    ACTIONS(754), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(756), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(1005), 1,
      aux_sym_comment_token1,
    STATE(394), 1,
      sym_comment,
  [14870] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(1007), 1,
      anon_sym_RPAREN,
    STATE(395), 1,
      sym_comment,
  [14883] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(1009), 1,
      anon_sym_LPAREN,
    STATE(396), 1,
      sym_comment,
  [14896] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(1011), 1,
      sym_identifier,
    STATE(397), 1,
      sym_comment,
  [14909] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(1013), 1,
      anon_sym_COLON,
    STATE(398), 1,
      sym_comment,
  [14922] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(1015), 1,
      sym_identifier,
    STATE(399), 1,
      sym_comment,
  [14935] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(1017), 1,
      anon_sym_EQ,
    STATE(400), 1,
      sym_comment,
  [14948] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(1019), 1,
      anon_sym_RPAREN,
    STATE(401), 1,
      sym_comment,
  [14961] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(1021), 1,
      anon_sym_RPAREN,
    STATE(402), 1,
      sym_comment,
  [14974] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(1023), 1,
      anon_sym_LBRACE,
    STATE(403), 1,
      sym_comment,
  [14987] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(1025), 1,
      sym_type_annotation,
    STATE(404), 1,
      sym_comment,
  [15000] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(1027), 1,
      anon_sym_EQ,
    STATE(405), 1,
      sym_comment,
  [15013] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(1029), 1,
      anon_sym_EQ,
    STATE(406), 1,
      sym_comment,
  [15026] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(1031), 1,
      anon_sym_EQ,
    STATE(407), 1,
      sym_comment,
  [15039] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(1033), 1,
      sym_identifier,
    STATE(408), 1,
      sym_comment,
  [15052] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(1035), 1,
      sym_identifier,
    STATE(409), 1,
      sym_comment,
  [15065] = 4,
    ACTIONS(754), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(756), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(1037), 1,
      aux_sym_comment_token2,
    STATE(410), 1,
      sym_comment,
  [15078] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(1039), 1,
      anon_sym_RPAREN,
    STATE(411), 1,
      sym_comment,
  [15091] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(1041), 1,
      anon_sym_LBRACE,
    STATE(412), 1,
      sym_comment,
  [15104] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(1043), 1,
      anon_sym_LBRACE,
    STATE(413), 1,
      sym_comment,
  [15117] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(1045), 1,
      anon_sym_RPAREN,
    STATE(414), 1,
      sym_comment,
  [15130] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(1047), 1,
      anon_sym_RPAREN,
    STATE(415), 1,
      sym_comment,
  [15143] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(1049), 1,
      sym_identifier,
    STATE(416), 1,
      sym_comment,
  [15156] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(1051), 1,
      sym_identifier,
    STATE(417), 1,
      sym_comment,
  [15169] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(1053), 1,
      sym_identifier,
    STATE(418), 1,
      sym_comment,
  [15182] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(1055), 1,
      sym_identifier,
    STATE(419), 1,
      sym_comment,
  [15195] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(1057), 1,
      sym_identifier,
    STATE(420), 1,
      sym_comment,
  [15208] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(1059), 1,
      anon_sym_RPAREN,
    STATE(421), 1,
      sym_comment,
  [15221] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(1061), 1,
      anon_sym_RPAREN,
    STATE(422), 1,
      sym_comment,
  [15234] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(1063), 1,
      sym_identifier,
    STATE(423), 1,
      sym_comment,
  [15247] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(1065), 1,
      sym_type_annotation,
    STATE(424), 1,
      sym_comment,
  [15260] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(1067), 1,
      anon_sym_LBRACE,
    STATE(425), 1,
      sym_comment,
  [15273] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(1069), 1,
      ts_builtin_sym_end,
    STATE(426), 1,
      sym_comment,
  [15286] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(1071), 1,
      anon_sym_LBRACE,
    STATE(427), 1,
      sym_comment,
  [15299] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(1073), 1,
      sym_identifier,
    STATE(428), 1,
      sym_comment,
  [15312] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(1075), 1,
      anon_sym_RPAREN,
    STATE(429), 1,
      sym_comment,
  [15325] = 4,
    ACTIONS(754), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(756), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(1077), 1,
      aux_sym_media_query_token1,
    STATE(430), 1,
      sym_comment,
  [15338] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(1079), 1,
      anon_sym_LPAREN,
    STATE(431), 1,
      sym_comment,
  [15351] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(1081), 1,
      anon_sym_LPAREN,
    STATE(432), 1,
      sym_comment,
  [15364] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(1083), 1,
      anon_sym_LPAREN,
    STATE(433), 1,
      sym_comment,
  [15377] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(1085), 1,
      anon_sym_LPAREN,
    STATE(434), 1,
      sym_comment,
  [15390] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(1087), 1,
      anon_sym_from,
    STATE(435), 1,
      sym_comment,
  [15403] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(1089), 1,
      anon_sym_in,
    STATE(436), 1,
      sym_comment,
  [15416] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(1091), 1,
      anon_sym_LPAREN,
    STATE(437), 1,
      sym_comment,
  [15429] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(1093), 1,
      sym_identifier,
    STATE(438), 1,
      sym_comment,
  [15442] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(1095), 1,
      sym_identifier,
    STATE(439), 1,
      sym_comment,
  [15455] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(1097), 1,
      sym_identifier,
    STATE(440), 1,
      sym_comment,
  [15468] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(1099), 1,
      sym_identifier,
    STATE(441), 1,
      sym_comment,
  [15481] = 4,
    ACTIONS(3), 1,
      anon_sym_SLASH_SLASH,
    ACTIONS(5), 1,
      anon_sym_SLASH_STAR,
    ACTIONS(1101), 1,
      anon_sym_RPAREN,
    STATE(442), 1,
      sym_comment,
  [15494] = 1,
    ACTIONS(1103), 1,
      ts_builtin_sym_end,
  [15498] = 1,
    ACTIONS(1105), 1,
      ts_builtin_sym_end,
};

static const uint32_t ts_small_parse_table_map[] = {
  [SMALL_STATE(7)] = 0,
  [SMALL_STATE(8)] = 130,
  [SMALL_STATE(9)] = 260,
  [SMALL_STATE(10)] = 388,
  [SMALL_STATE(11)] = 518,
  [SMALL_STATE(12)] = 648,
  [SMALL_STATE(13)] = 778,
  [SMALL_STATE(14)] = 908,
  [SMALL_STATE(15)] = 1038,
  [SMALL_STATE(16)] = 1168,
  [SMALL_STATE(17)] = 1298,
  [SMALL_STATE(18)] = 1428,
  [SMALL_STATE(19)] = 1494,
  [SMALL_STATE(20)] = 1551,
  [SMALL_STATE(21)] = 1608,
  [SMALL_STATE(22)] = 1665,
  [SMALL_STATE(23)] = 1722,
  [SMALL_STATE(24)] = 1779,
  [SMALL_STATE(25)] = 1836,
  [SMALL_STATE(26)] = 1893,
  [SMALL_STATE(27)] = 1950,
  [SMALL_STATE(28)] = 2011,
  [SMALL_STATE(29)] = 2068,
  [SMALL_STATE(30)] = 2125,
  [SMALL_STATE(31)] = 2182,
  [SMALL_STATE(32)] = 2239,
  [SMALL_STATE(33)] = 2296,
  [SMALL_STATE(34)] = 2353,
  [SMALL_STATE(35)] = 2418,
  [SMALL_STATE(36)] = 2475,
  [SMALL_STATE(37)] = 2540,
  [SMALL_STATE(38)] = 2605,
  [SMALL_STATE(39)] = 2670,
  [SMALL_STATE(40)] = 2727,
  [SMALL_STATE(41)] = 2784,
  [SMALL_STATE(42)] = 2841,
  [SMALL_STATE(43)] = 2902,
  [SMALL_STATE(44)] = 2959,
  [SMALL_STATE(45)] = 3016,
  [SMALL_STATE(46)] = 3073,
  [SMALL_STATE(47)] = 3130,
  [SMALL_STATE(48)] = 3187,
  [SMALL_STATE(49)] = 3244,
  [SMALL_STATE(50)] = 3301,
  [SMALL_STATE(51)] = 3358,
  [SMALL_STATE(52)] = 3419,
  [SMALL_STATE(53)] = 3476,
  [SMALL_STATE(54)] = 3533,
  [SMALL_STATE(55)] = 3590,
  [SMALL_STATE(56)] = 3647,
  [SMALL_STATE(57)] = 3704,
  [SMALL_STATE(58)] = 3765,
  [SMALL_STATE(59)] = 3822,
  [SMALL_STATE(60)] = 3885,
  [SMALL_STATE(61)] = 3947,
  [SMALL_STATE(62)] = 4009,
  [SMALL_STATE(63)] = 4071,
  [SMALL_STATE(64)] = 4133,
  [SMALL_STATE(65)] = 4179,
  [SMALL_STATE(66)] = 4223,
  [SMALL_STATE(67)] = 4269,
  [SMALL_STATE(68)] = 4315,
  [SMALL_STATE(69)] = 4361,
  [SMALL_STATE(70)] = 4407,
  [SMALL_STATE(71)] = 4451,
  [SMALL_STATE(72)] = 4494,
  [SMALL_STATE(73)] = 4537,
  [SMALL_STATE(74)] = 4580,
  [SMALL_STATE(75)] = 4623,
  [SMALL_STATE(76)] = 4666,
  [SMALL_STATE(77)] = 4709,
  [SMALL_STATE(78)] = 4752,
  [SMALL_STATE(79)] = 4795,
  [SMALL_STATE(80)] = 4838,
  [SMALL_STATE(81)] = 4881,
  [SMALL_STATE(82)] = 4924,
  [SMALL_STATE(83)] = 4967,
  [SMALL_STATE(84)] = 5010,
  [SMALL_STATE(85)] = 5053,
  [SMALL_STATE(86)] = 5096,
  [SMALL_STATE(87)] = 5139,
  [SMALL_STATE(88)] = 5182,
  [SMALL_STATE(89)] = 5225,
  [SMALL_STATE(90)] = 5268,
  [SMALL_STATE(91)] = 5311,
  [SMALL_STATE(92)] = 5354,
  [SMALL_STATE(93)] = 5397,
  [SMALL_STATE(94)] = 5440,
  [SMALL_STATE(95)] = 5483,
  [SMALL_STATE(96)] = 5526,
  [SMALL_STATE(97)] = 5569,
  [SMALL_STATE(98)] = 5612,
  [SMALL_STATE(99)] = 5655,
  [SMALL_STATE(100)] = 5698,
  [SMALL_STATE(101)] = 5741,
  [SMALL_STATE(102)] = 5784,
  [SMALL_STATE(103)] = 5827,
  [SMALL_STATE(104)] = 5870,
  [SMALL_STATE(105)] = 5913,
  [SMALL_STATE(106)] = 5956,
  [SMALL_STATE(107)] = 5997,
  [SMALL_STATE(108)] = 6040,
  [SMALL_STATE(109)] = 6083,
  [SMALL_STATE(110)] = 6124,
  [SMALL_STATE(111)] = 6167,
  [SMALL_STATE(112)] = 6210,
  [SMALL_STATE(113)] = 6253,
  [SMALL_STATE(114)] = 6323,
  [SMALL_STATE(115)] = 6363,
  [SMALL_STATE(116)] = 6403,
  [SMALL_STATE(117)] = 6473,
  [SMALL_STATE(118)] = 6513,
  [SMALL_STATE(119)] = 6553,
  [SMALL_STATE(120)] = 6593,
  [SMALL_STATE(121)] = 6633,
  [SMALL_STATE(122)] = 6673,
  [SMALL_STATE(123)] = 6713,
  [SMALL_STATE(124)] = 6783,
  [SMALL_STATE(125)] = 6823,
  [SMALL_STATE(126)] = 6863,
  [SMALL_STATE(127)] = 6903,
  [SMALL_STATE(128)] = 6943,
  [SMALL_STATE(129)] = 7013,
  [SMALL_STATE(130)] = 7053,
  [SMALL_STATE(131)] = 7093,
  [SMALL_STATE(132)] = 7133,
  [SMALL_STATE(133)] = 7173,
  [SMALL_STATE(134)] = 7213,
  [SMALL_STATE(135)] = 7253,
  [SMALL_STATE(136)] = 7293,
  [SMALL_STATE(137)] = 7333,
  [SMALL_STATE(138)] = 7373,
  [SMALL_STATE(139)] = 7413,
  [SMALL_STATE(140)] = 7453,
  [SMALL_STATE(141)] = 7523,
  [SMALL_STATE(142)] = 7563,
  [SMALL_STATE(143)] = 7603,
  [SMALL_STATE(144)] = 7643,
  [SMALL_STATE(145)] = 7683,
  [SMALL_STATE(146)] = 7753,
  [SMALL_STATE(147)] = 7793,
  [SMALL_STATE(148)] = 7833,
  [SMALL_STATE(149)] = 7873,
  [SMALL_STATE(150)] = 7943,
  [SMALL_STATE(151)] = 7983,
  [SMALL_STATE(152)] = 8050,
  [SMALL_STATE(153)] = 8117,
  [SMALL_STATE(154)] = 8184,
  [SMALL_STATE(155)] = 8248,
  [SMALL_STATE(156)] = 8304,
  [SMALL_STATE(157)] = 8360,
  [SMALL_STATE(158)] = 8416,
  [SMALL_STATE(159)] = 8469,
  [SMALL_STATE(160)] = 8522,
  [SMALL_STATE(161)] = 8575,
  [SMALL_STATE(162)] = 8628,
  [SMALL_STATE(163)] = 8681,
  [SMALL_STATE(164)] = 8734,
  [SMALL_STATE(165)] = 8787,
  [SMALL_STATE(166)] = 8840,
  [SMALL_STATE(167)] = 8893,
  [SMALL_STATE(168)] = 8946,
  [SMALL_STATE(169)] = 8999,
  [SMALL_STATE(170)] = 9052,
  [SMALL_STATE(171)] = 9105,
  [SMALL_STATE(172)] = 9158,
  [SMALL_STATE(173)] = 9211,
  [SMALL_STATE(174)] = 9264,
  [SMALL_STATE(175)] = 9317,
  [SMALL_STATE(176)] = 9370,
  [SMALL_STATE(177)] = 9423,
  [SMALL_STATE(178)] = 9476,
  [SMALL_STATE(179)] = 9529,
  [SMALL_STATE(180)] = 9582,
  [SMALL_STATE(181)] = 9635,
  [SMALL_STATE(182)] = 9688,
  [SMALL_STATE(183)] = 9741,
  [SMALL_STATE(184)] = 9794,
  [SMALL_STATE(185)] = 9847,
  [SMALL_STATE(186)] = 9900,
  [SMALL_STATE(187)] = 9953,
  [SMALL_STATE(188)] = 10006,
  [SMALL_STATE(189)] = 10059,
  [SMALL_STATE(190)] = 10112,
  [SMALL_STATE(191)] = 10165,
  [SMALL_STATE(192)] = 10218,
  [SMALL_STATE(193)] = 10271,
  [SMALL_STATE(194)] = 10324,
  [SMALL_STATE(195)] = 10377,
  [SMALL_STATE(196)] = 10409,
  [SMALL_STATE(197)] = 10441,
  [SMALL_STATE(198)] = 10477,
  [SMALL_STATE(199)] = 10509,
  [SMALL_STATE(200)] = 10551,
  [SMALL_STATE(201)] = 10583,
  [SMALL_STATE(202)] = 10615,
  [SMALL_STATE(203)] = 10647,
  [SMALL_STATE(204)] = 10679,
  [SMALL_STATE(205)] = 10721,
  [SMALL_STATE(206)] = 10753,
  [SMALL_STATE(207)] = 10789,
  [SMALL_STATE(208)] = 10821,
  [SMALL_STATE(209)] = 10855,
  [SMALL_STATE(210)] = 10887,
  [SMALL_STATE(211)] = 10919,
  [SMALL_STATE(212)] = 10951,
  [SMALL_STATE(213)] = 10983,
  [SMALL_STATE(214)] = 11021,
  [SMALL_STATE(215)] = 11063,
  [SMALL_STATE(216)] = 11105,
  [SMALL_STATE(217)] = 11147,
  [SMALL_STATE(218)] = 11189,
  [SMALL_STATE(219)] = 11231,
  [SMALL_STATE(220)] = 11273,
  [SMALL_STATE(221)] = 11315,
  [SMALL_STATE(222)] = 11357,
  [SMALL_STATE(223)] = 11399,
  [SMALL_STATE(224)] = 11431,
  [SMALL_STATE(225)] = 11463,
  [SMALL_STATE(226)] = 11495,
  [SMALL_STATE(227)] = 11532,
  [SMALL_STATE(228)] = 11569,
  [SMALL_STATE(229)] = 11602,
  [SMALL_STATE(230)] = 11639,
  [SMALL_STATE(231)] = 11676,
  [SMALL_STATE(232)] = 11715,
  [SMALL_STATE(233)] = 11752,
  [SMALL_STATE(234)] = 11785,
  [SMALL_STATE(235)] = 11818,
  [SMALL_STATE(236)] = 11851,
  [SMALL_STATE(237)] = 11888,
  [SMALL_STATE(238)] = 11925,
  [SMALL_STATE(239)] = 11962,
  [SMALL_STATE(240)] = 11999,
  [SMALL_STATE(241)] = 12038,
  [SMALL_STATE(242)] = 12074,
  [SMALL_STATE(243)] = 12110,
  [SMALL_STATE(244)] = 12146,
  [SMALL_STATE(245)] = 12182,
  [SMALL_STATE(246)] = 12218,
  [SMALL_STATE(247)] = 12254,
  [SMALL_STATE(248)] = 12291,
  [SMALL_STATE(249)] = 12326,
  [SMALL_STATE(250)] = 12353,
  [SMALL_STATE(251)] = 12382,
  [SMALL_STATE(252)] = 12411,
  [SMALL_STATE(253)] = 12440,
  [SMALL_STATE(254)] = 12469,
  [SMALL_STATE(255)] = 12495,
  [SMALL_STATE(256)] = 12521,
  [SMALL_STATE(257)] = 12547,
  [SMALL_STATE(258)] = 12571,
  [SMALL_STATE(259)] = 12597,
  [SMALL_STATE(260)] = 12623,
  [SMALL_STATE(261)] = 12649,
  [SMALL_STATE(262)] = 12666,
  [SMALL_STATE(263)] = 12691,
  [SMALL_STATE(264)] = 12708,
  [SMALL_STATE(265)] = 12731,
  [SMALL_STATE(266)] = 12756,
  [SMALL_STATE(267)] = 12779,
  [SMALL_STATE(268)] = 12796,
  [SMALL_STATE(269)] = 12813,
  [SMALL_STATE(270)] = 12830,
  [SMALL_STATE(271)] = 12847,
  [SMALL_STATE(272)] = 12864,
  [SMALL_STATE(273)] = 12881,
  [SMALL_STATE(274)] = 12898,
  [SMALL_STATE(275)] = 12915,
  [SMALL_STATE(276)] = 12931,
  [SMALL_STATE(277)] = 12953,
  [SMALL_STATE(278)] = 12973,
  [SMALL_STATE(279)] = 12993,
  [SMALL_STATE(280)] = 13013,
  [SMALL_STATE(281)] = 13035,
  [SMALL_STATE(282)] = 13057,
  [SMALL_STATE(283)] = 13079,
  [SMALL_STATE(284)] = 13099,
  [SMALL_STATE(285)] = 13121,
  [SMALL_STATE(286)] = 13141,
  [SMALL_STATE(287)] = 13161,
  [SMALL_STATE(288)] = 13183,
  [SMALL_STATE(289)] = 13199,
  [SMALL_STATE(290)] = 13221,
  [SMALL_STATE(291)] = 13240,
  [SMALL_STATE(292)] = 13259,
  [SMALL_STATE(293)] = 13274,
  [SMALL_STATE(294)] = 13293,
  [SMALL_STATE(295)] = 13312,
  [SMALL_STATE(296)] = 13329,
  [SMALL_STATE(297)] = 13348,
  [SMALL_STATE(298)] = 13367,
  [SMALL_STATE(299)] = 13382,
  [SMALL_STATE(300)] = 13401,
  [SMALL_STATE(301)] = 13418,
  [SMALL_STATE(302)] = 13437,
  [SMALL_STATE(303)] = 13454,
  [SMALL_STATE(304)] = 13473,
  [SMALL_STATE(305)] = 13492,
  [SMALL_STATE(306)] = 13511,
  [SMALL_STATE(307)] = 13530,
  [SMALL_STATE(308)] = 13545,
  [SMALL_STATE(309)] = 13564,
  [SMALL_STATE(310)] = 13581,
  [SMALL_STATE(311)] = 13598,
  [SMALL_STATE(312)] = 13617,
  [SMALL_STATE(313)] = 13636,
  [SMALL_STATE(314)] = 13655,
  [SMALL_STATE(315)] = 13672,
  [SMALL_STATE(316)] = 13691,
  [SMALL_STATE(317)] = 13708,
  [SMALL_STATE(318)] = 13727,
  [SMALL_STATE(319)] = 13746,
  [SMALL_STATE(320)] = 13762,
  [SMALL_STATE(321)] = 13778,
  [SMALL_STATE(322)] = 13792,
  [SMALL_STATE(323)] = 13808,
  [SMALL_STATE(324)] = 13824,
  [SMALL_STATE(325)] = 13840,
  [SMALL_STATE(326)] = 13856,
  [SMALL_STATE(327)] = 13872,
  [SMALL_STATE(328)] = 13888,
  [SMALL_STATE(329)] = 13904,
  [SMALL_STATE(330)] = 13920,
  [SMALL_STATE(331)] = 13936,
  [SMALL_STATE(332)] = 13952,
  [SMALL_STATE(333)] = 13968,
  [SMALL_STATE(334)] = 13984,
  [SMALL_STATE(335)] = 14000,
  [SMALL_STATE(336)] = 14016,
  [SMALL_STATE(337)] = 14032,
  [SMALL_STATE(338)] = 14048,
  [SMALL_STATE(339)] = 14064,
  [SMALL_STATE(340)] = 14080,
  [SMALL_STATE(341)] = 14096,
  [SMALL_STATE(342)] = 14112,
  [SMALL_STATE(343)] = 14128,
  [SMALL_STATE(344)] = 14144,
  [SMALL_STATE(345)] = 14160,
  [SMALL_STATE(346)] = 14176,
  [SMALL_STATE(347)] = 14192,
  [SMALL_STATE(348)] = 14208,
  [SMALL_STATE(349)] = 14224,
  [SMALL_STATE(350)] = 14240,
  [SMALL_STATE(351)] = 14254,
  [SMALL_STATE(352)] = 14270,
  [SMALL_STATE(353)] = 14284,
  [SMALL_STATE(354)] = 14298,
  [SMALL_STATE(355)] = 14314,
  [SMALL_STATE(356)] = 14330,
  [SMALL_STATE(357)] = 14344,
  [SMALL_STATE(358)] = 14360,
  [SMALL_STATE(359)] = 14376,
  [SMALL_STATE(360)] = 14392,
  [SMALL_STATE(361)] = 14406,
  [SMALL_STATE(362)] = 14422,
  [SMALL_STATE(363)] = 14438,
  [SMALL_STATE(364)] = 14454,
  [SMALL_STATE(365)] = 14470,
  [SMALL_STATE(366)] = 14486,
  [SMALL_STATE(367)] = 14500,
  [SMALL_STATE(368)] = 14516,
  [SMALL_STATE(369)] = 14532,
  [SMALL_STATE(370)] = 14545,
  [SMALL_STATE(371)] = 14558,
  [SMALL_STATE(372)] = 14571,
  [SMALL_STATE(373)] = 14584,
  [SMALL_STATE(374)] = 14597,
  [SMALL_STATE(375)] = 14610,
  [SMALL_STATE(376)] = 14623,
  [SMALL_STATE(377)] = 14636,
  [SMALL_STATE(378)] = 14649,
  [SMALL_STATE(379)] = 14662,
  [SMALL_STATE(380)] = 14675,
  [SMALL_STATE(381)] = 14688,
  [SMALL_STATE(382)] = 14701,
  [SMALL_STATE(383)] = 14714,
  [SMALL_STATE(384)] = 14727,
  [SMALL_STATE(385)] = 14740,
  [SMALL_STATE(386)] = 14753,
  [SMALL_STATE(387)] = 14766,
  [SMALL_STATE(388)] = 14779,
  [SMALL_STATE(389)] = 14792,
  [SMALL_STATE(390)] = 14805,
  [SMALL_STATE(391)] = 14818,
  [SMALL_STATE(392)] = 14831,
  [SMALL_STATE(393)] = 14844,
  [SMALL_STATE(394)] = 14857,
  [SMALL_STATE(395)] = 14870,
  [SMALL_STATE(396)] = 14883,
  [SMALL_STATE(397)] = 14896,
  [SMALL_STATE(398)] = 14909,
  [SMALL_STATE(399)] = 14922,
  [SMALL_STATE(400)] = 14935,
  [SMALL_STATE(401)] = 14948,
  [SMALL_STATE(402)] = 14961,
  [SMALL_STATE(403)] = 14974,
  [SMALL_STATE(404)] = 14987,
  [SMALL_STATE(405)] = 15000,
  [SMALL_STATE(406)] = 15013,
  [SMALL_STATE(407)] = 15026,
  [SMALL_STATE(408)] = 15039,
  [SMALL_STATE(409)] = 15052,
  [SMALL_STATE(410)] = 15065,
  [SMALL_STATE(411)] = 15078,
  [SMALL_STATE(412)] = 15091,
  [SMALL_STATE(413)] = 15104,
  [SMALL_STATE(414)] = 15117,
  [SMALL_STATE(415)] = 15130,
  [SMALL_STATE(416)] = 15143,
  [SMALL_STATE(417)] = 15156,
  [SMALL_STATE(418)] = 15169,
  [SMALL_STATE(419)] = 15182,
  [SMALL_STATE(420)] = 15195,
  [SMALL_STATE(421)] = 15208,
  [SMALL_STATE(422)] = 15221,
  [SMALL_STATE(423)] = 15234,
  [SMALL_STATE(424)] = 15247,
  [SMALL_STATE(425)] = 15260,
  [SMALL_STATE(426)] = 15273,
  [SMALL_STATE(427)] = 15286,
  [SMALL_STATE(428)] = 15299,
  [SMALL_STATE(429)] = 15312,
  [SMALL_STATE(430)] = 15325,
  [SMALL_STATE(431)] = 15338,
  [SMALL_STATE(432)] = 15351,
  [SMALL_STATE(433)] = 15364,
  [SMALL_STATE(434)] = 15377,
  [SMALL_STATE(435)] = 15390,
  [SMALL_STATE(436)] = 15403,
  [SMALL_STATE(437)] = 15416,
  [SMALL_STATE(438)] = 15429,
  [SMALL_STATE(439)] = 15442,
  [SMALL_STATE(440)] = 15455,
  [SMALL_STATE(441)] = 15468,
  [SMALL_STATE(442)] = 15481,
  [SMALL_STATE(443)] = 15494,
  [SMALL_STATE(444)] = 15498,
};

static const TSParseActionEntry ts_parse_actions[] = {
  [0] = {.entry = {.count = 0, .reusable = false}},
  [1] = {.entry = {.count = 1, .reusable = false}}, RECOVER(),
  [3] = {.entry = {.count = 1, .reusable = true}}, SHIFT(394),
  [5] = {.entry = {.count = 1, .reusable = true}}, SHIFT(410),
  [7] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_source_file, 0, 0, 0),
  [9] = {.entry = {.count = 1, .reusable = true}}, SHIFT(381),
  [11] = {.entry = {.count = 1, .reusable = true}}, SHIFT(373),
  [13] = {.entry = {.count = 1, .reusable = true}}, SHIFT(374),
  [15] = {.entry = {.count = 1, .reusable = true}}, SHIFT(329),
  [17] = {.entry = {.count = 1, .reusable = true}}, SHIFT(193),
  [19] = {.entry = {.count = 1, .reusable = true}}, SHIFT(308),
  [21] = {.entry = {.count = 1, .reusable = true}}, SHIFT(133),
  [23] = {.entry = {.count = 1, .reusable = false}}, SHIFT(408),
  [25] = {.entry = {.count = 1, .reusable = false}}, SHIFT(409),
  [27] = {.entry = {.count = 1, .reusable = false}}, SHIFT(327),
  [29] = {.entry = {.count = 1, .reusable = false}}, SHIFT(438),
  [31] = {.entry = {.count = 1, .reusable = false}}, SHIFT(385),
  [33] = {.entry = {.count = 1, .reusable = false}}, SHIFT(431),
  [35] = {.entry = {.count = 1, .reusable = false}}, SHIFT(87),
  [37] = {.entry = {.count = 1, .reusable = false}}, SHIFT(187),
  [39] = {.entry = {.count = 1, .reusable = false}}, SHIFT(439),
  [41] = {.entry = {.count = 1, .reusable = false}}, SHIFT(188),
  [43] = {.entry = {.count = 1, .reusable = false}}, SHIFT(440),
  [45] = {.entry = {.count = 1, .reusable = false}}, SHIFT(333),
  [47] = {.entry = {.count = 1, .reusable = false}}, SHIFT(305),
  [49] = {.entry = {.count = 1, .reusable = false}}, SHIFT(344),
  [51] = {.entry = {.count = 1, .reusable = true}}, SHIFT(328),
  [53] = {.entry = {.count = 1, .reusable = false}}, SHIFT(412),
  [55] = {.entry = {.count = 1, .reusable = false}}, SHIFT(413),
  [57] = {.entry = {.count = 1, .reusable = true}}, SHIFT(184),
  [59] = {.entry = {.count = 1, .reusable = false}}, SHIFT(437),
  [61] = {.entry = {.count = 1, .reusable = true}}, SHIFT(256),
  [63] = {.entry = {.count = 1, .reusable = true}}, SHIFT(39),
  [65] = {.entry = {.count = 1, .reusable = false}}, SHIFT(40),
  [67] = {.entry = {.count = 1, .reusable = false}}, SHIFT(39),
  [69] = {.entry = {.count = 1, .reusable = true}}, SHIFT(155),
  [71] = {.entry = {.count = 1, .reusable = false}}, SHIFT(18),
  [73] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_fetch_block_repeat1, 2, 0, 0), SHIFT_REPEAT(193),
  [76] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_fetch_block_repeat1, 2, 0, 0), SHIFT_REPEAT(308),
  [79] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_fetch_block_repeat1, 2, 0, 0),
  [81] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_fetch_block_repeat1, 2, 0, 0), SHIFT_REPEAT(408),
  [84] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_fetch_block_repeat1, 2, 0, 0), SHIFT_REPEAT(409),
  [87] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_fetch_block_repeat1, 2, 0, 0), SHIFT_REPEAT(327),
  [90] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_fetch_block_repeat1, 2, 0, 0), SHIFT_REPEAT(438),
  [93] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_fetch_block_repeat1, 2, 0, 0), SHIFT_REPEAT(385),
  [96] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_fetch_block_repeat1, 2, 0, 0), SHIFT_REPEAT(431),
  [99] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_fetch_block_repeat1, 2, 0, 0), SHIFT_REPEAT(87),
  [102] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_fetch_block_repeat1, 2, 0, 0), SHIFT_REPEAT(187),
  [105] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_fetch_block_repeat1, 2, 0, 0), SHIFT_REPEAT(439),
  [108] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_fetch_block_repeat1, 2, 0, 0), SHIFT_REPEAT(188),
  [111] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_fetch_block_repeat1, 2, 0, 0), SHIFT_REPEAT(440),
  [114] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_fetch_block_repeat1, 2, 0, 0), SHIFT_REPEAT(333),
  [117] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_fetch_block_repeat1, 2, 0, 0), SHIFT_REPEAT(305),
  [120] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_fetch_block_repeat1, 2, 0, 0), SHIFT_REPEAT(344),
  [123] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_fetch_block_repeat1, 2, 0, 0), SHIFT_REPEAT(328),
  [126] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_fetch_block_repeat1, 2, 0, 0), SHIFT_REPEAT(412),
  [129] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_fetch_block_repeat1, 2, 0, 0), SHIFT_REPEAT(413),
  [132] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_fetch_block_repeat1, 2, 0, 0), SHIFT_REPEAT(184),
  [135] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_fetch_block_repeat1, 2, 0, 0), SHIFT_REPEAT(437),
  [138] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_fetch_block_repeat1, 2, 0, 0), SHIFT_REPEAT(256),
  [141] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_fetch_block_repeat1, 2, 0, 0), SHIFT_REPEAT(39),
  [144] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_fetch_block_repeat1, 2, 0, 0), SHIFT_REPEAT(40),
  [147] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_fetch_block_repeat1, 2, 0, 0), SHIFT_REPEAT(39),
  [150] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_fetch_block_repeat1, 2, 0, 0), SHIFT_REPEAT(155),
  [153] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_fetch_block_repeat1, 2, 0, 0), SHIFT_REPEAT(18),
  [156] = {.entry = {.count = 1, .reusable = true}}, SHIFT(115),
  [158] = {.entry = {.count = 1, .reusable = true}}, SHIFT(98),
  [160] = {.entry = {.count = 1, .reusable = true}}, SHIFT(94),
  [162] = {.entry = {.count = 1, .reusable = true}}, SHIFT(158),
  [164] = {.entry = {.count = 1, .reusable = true}}, SHIFT(315),
  [166] = {.entry = {.count = 1, .reusable = true}}, SHIFT(268),
  [168] = {.entry = {.count = 1, .reusable = false}}, SHIFT(389),
  [170] = {.entry = {.count = 1, .reusable = false}}, SHIFT(416),
  [172] = {.entry = {.count = 1, .reusable = false}}, SHIFT(320),
  [174] = {.entry = {.count = 1, .reusable = false}}, SHIFT(417),
  [176] = {.entry = {.count = 1, .reusable = false}}, SHIFT(399),
  [178] = {.entry = {.count = 1, .reusable = false}}, SHIFT(434),
  [180] = {.entry = {.count = 1, .reusable = false}}, SHIFT(143),
  [182] = {.entry = {.count = 1, .reusable = false}}, SHIFT(181),
  [184] = {.entry = {.count = 1, .reusable = false}}, SHIFT(378),
  [186] = {.entry = {.count = 1, .reusable = false}}, SHIFT(183),
  [188] = {.entry = {.count = 1, .reusable = false}}, SHIFT(419),
  [190] = {.entry = {.count = 1, .reusable = true}}, SHIFT(368),
  [192] = {.entry = {.count = 1, .reusable = false}}, SHIFT(387),
  [194] = {.entry = {.count = 1, .reusable = false}}, SHIFT(427),
  [196] = {.entry = {.count = 1, .reusable = true}}, SHIFT(174),
  [198] = {.entry = {.count = 1, .reusable = false}}, SHIFT(377),
  [200] = {.entry = {.count = 1, .reusable = true}}, SHIFT(258),
  [202] = {.entry = {.count = 1, .reusable = true}}, SHIFT(20),
  [204] = {.entry = {.count = 1, .reusable = false}}, SHIFT(32),
  [206] = {.entry = {.count = 1, .reusable = false}}, SHIFT(20),
  [208] = {.entry = {.count = 1, .reusable = true}}, SHIFT(156),
  [210] = {.entry = {.count = 1, .reusable = false}}, SHIFT(59),
  [212] = {.entry = {.count = 1, .reusable = true}}, SHIFT(270),
  [214] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(158),
  [217] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(315),
  [220] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0),
  [222] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(389),
  [225] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(416),
  [228] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(320),
  [231] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(417),
  [234] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(399),
  [237] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(434),
  [240] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(143),
  [243] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(181),
  [246] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(378),
  [249] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(183),
  [252] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(419),
  [255] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(368),
  [258] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(387),
  [261] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(427),
  [264] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(174),
  [267] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(377),
  [270] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(258),
  [273] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(20),
  [276] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(32),
  [279] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(20),
  [282] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(156),
  [285] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 2, 0, 0), SHIFT_REPEAT(59),
  [288] = {.entry = {.count = 1, .reusable = true}}, SHIFT(139),
  [290] = {.entry = {.count = 1, .reusable = true}}, SHIFT(137),
  [292] = {.entry = {.count = 1, .reusable = true}}, SHIFT(109),
  [294] = {.entry = {.count = 1, .reusable = true}}, SHIFT(106),
  [296] = {.entry = {.count = 1, .reusable = true}}, SHIFT(101),
  [298] = {.entry = {.count = 1, .reusable = true}}, SHIFT(102),
  [300] = {.entry = {.count = 1, .reusable = true}}, SHIFT(70),
  [302] = {.entry = {.count = 1, .reusable = true}}, SHIFT(65),
  [304] = {.entry = {.count = 2, .reusable = true}}, REDUCE(sym_component_name, 1, 0, 0), REDUCE(sym__expression, 1, 0, 0),
  [307] = {.entry = {.count = 1, .reusable = false}}, SHIFT(165),
  [309] = {.entry = {.count = 2, .reusable = false}}, REDUCE(sym_component_name, 1, 0, 0), REDUCE(sym__expression, 1, 0, 0),
  [312] = {.entry = {.count = 2, .reusable = true}}, REDUCE(sym__expression, 1, 0, 0), SHIFT(418),
  [315] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym__expression, 1, 0, 0),
  [317] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym__expression, 1, 0, 0),
  [319] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_object, 2, 0, 0),
  [321] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_object, 2, 0, 0),
  [323] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym__expression, 3, 0, 0),
  [325] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym__expression, 3, 0, 0),
  [327] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_object, 3, 0, 0),
  [329] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_object, 3, 0, 0),
  [331] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_string, 3, 0, 0),
  [333] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_string, 3, 0, 0),
  [335] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array, 3, 0, 0),
  [337] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array, 3, 0, 0),
  [339] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_call_expression, 3, 0, 0),
  [341] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_call_expression, 3, 0, 0),
  [343] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_member_expression, 3, 0, 0),
  [345] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_member_expression, 3, 0, 0),
  [347] = {.entry = {.count = 1, .reusable = true}}, SHIFT(116),
  [349] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_binary_expression, 3, 0, 0),
  [351] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_binary_expression, 3, 0, 0),
  [353] = {.entry = {.count = 1, .reusable = true}}, SHIFT(376),
  [355] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_object, 4, 0, 0),
  [357] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_object, 4, 0, 0),
  [359] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_t_call, 4, 0, 0),
  [361] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_t_call, 4, 0, 0),
  [363] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array, 4, 0, 0),
  [365] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array, 4, 0, 0),
  [367] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_call_expression, 4, 0, 0),
  [369] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_call_expression, 4, 0, 0),
  [371] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_boolean, 1, 0, 0),
  [373] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_boolean, 1, 0, 0),
  [375] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_t_call, 6, 0, 0),
  [377] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_t_call, 6, 0, 0),
  [379] = {.entry = {.count = 1, .reusable = true}}, SHIFT(145),
  [381] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_expression_statement, 1, 0, 0),
  [383] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_expression_statement, 1, 0, 0),
  [385] = {.entry = {.count = 1, .reusable = true}}, SHIFT(397),
  [387] = {.entry = {.count = 1, .reusable = true}}, SHIFT(178),
  [389] = {.entry = {.count = 1, .reusable = false}}, SHIFT(178),
  [391] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_string, 2, 0, 0),
  [393] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_string, 2, 0, 0),
  [395] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_assignment, 3, 0, 0),
  [397] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_assignment, 3, 0, 0),
  [399] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_state_declaration, 4, 0, 0),
  [401] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_state_declaration, 4, 0, 0),
  [403] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_derived_declaration, 4, 0, 0),
  [405] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_derived_declaration, 4, 0, 0),
  [407] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_unary_expression, 2, 0, 0),
  [409] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_unary_expression, 2, 0, 0),
  [411] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_array, 2, 0, 0),
  [413] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_array, 2, 0, 0),
  [415] = {.entry = {.count = 1, .reusable = false}}, SHIFT(167),
  [417] = {.entry = {.count = 2, .reusable = true}}, REDUCE(sym__expression, 1, 0, 0), SHIFT(372),
  [420] = {.entry = {.count = 1, .reusable = true}}, SHIFT(170),
  [422] = {.entry = {.count = 1, .reusable = false}}, SHIFT(170),
  [424] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_if_statement, 3, 0, 0),
  [426] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_if_statement, 3, 0, 0),
  [428] = {.entry = {.count = 1, .reusable = false}}, SHIFT(285),
  [430] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_block, 3, 0, 0),
  [432] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_block, 3, 0, 0),
  [434] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_ui_element, 1, 0, 0),
  [436] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_ui_element, 1, 0, 0),
  [438] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_ui_element, 3, 0, 0),
  [440] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_ui_element, 3, 0, 0),
  [442] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_ui_element, 4, 0, 0),
  [444] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_ui_element, 4, 0, 0),
  [446] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_if_statement, 5, 0, 0),
  [448] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_if_statement, 5, 0, 0),
  [450] = {.entry = {.count = 1, .reusable = false}}, SHIFT(286),
  [452] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_block, 2, 0, 0),
  [454] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_block, 2, 0, 0),
  [456] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_fetch_block_repeat1, 1, 0, 0),
  [458] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_fetch_block_repeat1, 1, 0, 0),
  [460] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_effect_declaration, 2, 0, 0),
  [462] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_effect_declaration, 2, 0, 0),
  [464] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_use_declaration, 2, 0, 0),
  [466] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_use_declaration, 2, 0, 0),
  [468] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_event_handler, 2, 0, 0),
  [470] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_event_handler, 2, 0, 0),
  [472] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_ui_element, 2, 0, 0),
  [474] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_ui_element, 2, 0, 0),
  [476] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_show_statement, 3, 0, 0),
  [478] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_show_statement, 3, 0, 0),
  [480] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_style_block, 3, 0, 0),
  [482] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_style_block, 3, 0, 0),
  [484] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_transition_block, 3, 0, 0),
  [486] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_transition_block, 3, 0, 0),
  [488] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_sub_component, 3, 0, 1),
  [490] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_sub_component, 3, 0, 1),
  [492] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_success_block, 2, 0, 0),
  [494] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_success_block, 2, 0, 0),
  [496] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_navigate_call, 4, 0, 0),
  [498] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_navigate_call, 4, 0, 0),
  [500] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_style_block, 4, 0, 0),
  [502] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_style_block, 4, 0, 0),
  [504] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_transition_block, 4, 0, 0),
  [506] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_transition_block, 4, 0, 0),
  [508] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_error_block, 5, 0, 0),
  [510] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_error_block, 5, 0, 0),
  [512] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_action_declaration, 5, 0, 0),
  [514] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_action_declaration, 5, 0, 0),
  [516] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_component_name, 1, 0, 0),
  [518] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_component_name, 1, 0, 0),
  [520] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_for_statement, 5, 0, 0),
  [522] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_for_statement, 5, 0, 0),
  [524] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_show_statement, 5, 0, 0),
  [526] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_show_statement, 5, 0, 0),
  [528] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_fetch_statement, 5, 0, 0),
  [530] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_fetch_statement, 5, 0, 0),
  [532] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_ui_element, 5, 0, 0),
  [534] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_ui_element, 5, 0, 0),
  [536] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_action_declaration, 6, 0, 0),
  [538] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_action_declaration, 6, 0, 0),
  [540] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_fetch_block, 2, 0, 0),
  [542] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_fetch_block, 2, 0, 0),
  [544] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_if_statement, 7, 0, 0),
  [546] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_if_statement, 7, 0, 0),
  [548] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_for_statement, 7, 0, 0),
  [550] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_for_statement, 7, 0, 0),
  [552] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_fetch_statement, 7, 0, 0),
  [554] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_fetch_statement, 7, 0, 0),
  [556] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_fetch_block, 3, 0, 0),
  [558] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_fetch_block, 3, 0, 0),
  [560] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_fetch_statement, 8, 0, 0),
  [562] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_fetch_statement, 8, 0, 0),
  [564] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_for_statement, 9, 0, 0),
  [566] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_for_statement, 9, 0, 0),
  [568] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym__statement, 1, 0, 0),
  [570] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym__statement, 1, 0, 0),
  [572] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_error_block, 2, 0, 0),
  [574] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_error_block, 2, 0, 0),
  [576] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_loading_block, 2, 0, 0),
  [578] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_loading_block, 2, 0, 0),
  [580] = {.entry = {.count = 1, .reusable = false}}, SHIFT(277),
  [582] = {.entry = {.count = 1, .reusable = false}}, SHIFT(283),
  [584] = {.entry = {.count = 1, .reusable = true}}, SHIFT(355),
  [586] = {.entry = {.count = 1, .reusable = false}}, SHIFT(356),
  [588] = {.entry = {.count = 1, .reusable = false}}, SHIFT(208),
  [590] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_block_repeat1, 1, 0, 0),
  [592] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_block_repeat1, 1, 0, 0),
  [594] = {.entry = {.count = 1, .reusable = true}}, SHIFT(25),
  [596] = {.entry = {.count = 1, .reusable = true}}, SHIFT(403),
  [598] = {.entry = {.count = 1, .reusable = true}}, SHIFT(228),
  [600] = {.entry = {.count = 1, .reusable = true}}, SHIFT(195),
  [602] = {.entry = {.count = 1, .reusable = true}}, SHIFT(49),
  [604] = {.entry = {.count = 1, .reusable = true}}, SHIFT(233),
  [606] = {.entry = {.count = 1, .reusable = true}}, SHIFT(44),
  [608] = {.entry = {.count = 1, .reusable = true}}, SHIFT(58),
  [610] = {.entry = {.count = 1, .reusable = true}}, SHIFT(198),
  [612] = {.entry = {.count = 1, .reusable = true}}, SHIFT(186),
  [614] = {.entry = {.count = 1, .reusable = true}}, SHIFT(290),
  [616] = {.entry = {.count = 1, .reusable = true}}, SHIFT(182),
  [618] = {.entry = {.count = 1, .reusable = false}}, SHIFT(432),
  [620] = {.entry = {.count = 1, .reusable = true}}, SHIFT(255),
  [622] = {.entry = {.count = 1, .reusable = true}}, SHIFT(223),
  [624] = {.entry = {.count = 1, .reusable = false}}, SHIFT(224),
  [626] = {.entry = {.count = 1, .reusable = false}}, SHIFT(223),
  [628] = {.entry = {.count = 1, .reusable = true}}, SHIFT(157),
  [630] = {.entry = {.count = 1, .reusable = true}}, SHIFT(140),
  [632] = {.entry = {.count = 1, .reusable = true}}, SHIFT(420),
  [634] = {.entry = {.count = 1, .reusable = true}}, SHIFT(180),
  [636] = {.entry = {.count = 1, .reusable = true}}, SHIFT(24),
  [638] = {.entry = {.count = 1, .reusable = true}}, SHIFT(362),
  [640] = {.entry = {.count = 1, .reusable = true}}, SHIFT(11),
  [642] = {.entry = {.count = 1, .reusable = true}}, SHIFT(171),
  [644] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_style_property, 3, 0, 0),
  [646] = {.entry = {.count = 1, .reusable = true}}, SHIFT(166),
  [648] = {.entry = {.count = 1, .reusable = false}}, SHIFT(166),
  [650] = {.entry = {.count = 1, .reusable = true}}, SHIFT(324),
  [652] = {.entry = {.count = 1, .reusable = true}}, SHIFT(359),
  [654] = {.entry = {.count = 1, .reusable = true}}, SHIFT(16),
  [656] = {.entry = {.count = 1, .reusable = true}}, SHIFT(361),
  [658] = {.entry = {.count = 1, .reusable = true}}, SHIFT(14),
  [660] = {.entry = {.count = 1, .reusable = true}}, SHIFT(203),
  [662] = {.entry = {.count = 1, .reusable = true}}, SHIFT(363),
  [664] = {.entry = {.count = 1, .reusable = true}}, SHIFT(364),
  [666] = {.entry = {.count = 1, .reusable = true}}, SHIFT(48),
  [668] = {.entry = {.count = 1, .reusable = true}}, SHIFT(354),
  [670] = {.entry = {.count = 1, .reusable = true}}, SHIFT(12),
  [672] = {.entry = {.count = 1, .reusable = true}}, SHIFT(343),
  [674] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_object_entry, 3, 0, 0),
  [676] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_parameter, 5, 0, 0),
  [678] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym__argument, 1, 0, 0),
  [680] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_parameter, 6, 0, 0),
  [682] = {.entry = {.count = 1, .reusable = true}}, SHIFT(149),
  [684] = {.entry = {.count = 1, .reusable = true}}, SHIFT(6),
  [686] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_parameter, 4, 0, 0),
  [688] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_named_argument, 3, 0, 0),
  [690] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_array_repeat1, 2, 0, 0),
  [692] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_transition_property, 2, 0, 0),
  [694] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_parameter, 3, 0, 0),
  [696] = {.entry = {.count = 1, .reusable = true}}, SHIFT(128),
  [698] = {.entry = {.count = 1, .reusable = true}}, SHIFT(2),
  [700] = {.entry = {.count = 1, .reusable = true}}, SHIFT(118),
  [702] = {.entry = {.count = 1, .reusable = true}}, SHIFT(21),
  [704] = {.entry = {.count = 1, .reusable = true}}, SHIFT(200),
  [706] = {.entry = {.count = 1, .reusable = true}}, SHIFT(275),
  [708] = {.entry = {.count = 1, .reusable = true}}, SHIFT(81),
  [710] = {.entry = {.count = 1, .reusable = true}}, SHIFT(45),
  [712] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_source_file, 1, 0, 0),
  [714] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_source_file_repeat1, 2, 0, 0),
  [716] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_source_file_repeat1, 2, 0, 0), SHIFT_REPEAT(381),
  [719] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_source_file_repeat1, 2, 0, 0), SHIFT_REPEAT(373),
  [722] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_source_file_repeat1, 2, 0, 0), SHIFT_REPEAT(374),
  [725] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_source_file_repeat1, 2, 0, 0), SHIFT_REPEAT(329),
  [728] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_style_block_repeat1, 2, 0, 0),
  [730] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_style_block_repeat1, 2, 0, 0), SHIFT_REPEAT(430),
  [733] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_style_block_repeat1, 2, 0, 0), SHIFT_REPEAT(291),
  [736] = {.entry = {.count = 1, .reusable = true}}, SHIFT(124),
  [738] = {.entry = {.count = 1, .reusable = true}}, SHIFT(430),
  [740] = {.entry = {.count = 1, .reusable = true}}, SHIFT(291),
  [742] = {.entry = {.count = 1, .reusable = true}}, SHIFT(129),
  [744] = {.entry = {.count = 1, .reusable = true}}, SHIFT(82),
  [746] = {.entry = {.count = 1, .reusable = true}}, SHIFT(77),
  [748] = {.entry = {.count = 1, .reusable = false}}, SHIFT(164),
  [750] = {.entry = {.count = 1, .reusable = false}}, SHIFT(47),
  [752] = {.entry = {.count = 1, .reusable = false}}, SHIFT(288),
  [754] = {.entry = {.count = 1, .reusable = false}}, SHIFT(394),
  [756] = {.entry = {.count = 1, .reusable = false}}, SHIFT(410),
  [758] = {.entry = {.count = 1, .reusable = false}}, SHIFT(225),
  [760] = {.entry = {.count = 1, .reusable = false}}, SHIFT(43),
  [762] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_string_repeat1, 2, 0, 0), SHIFT_REPEAT(164),
  [765] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_string_repeat1, 2, 0, 0),
  [767] = {.entry = {.count = 2, .reusable = false}}, REDUCE(aux_sym_string_repeat1, 2, 0, 0), SHIFT_REPEAT(288),
  [770] = {.entry = {.count = 1, .reusable = false}}, SHIFT(35),
  [772] = {.entry = {.count = 1, .reusable = false}}, SHIFT(202),
  [774] = {.entry = {.count = 1, .reusable = false}}, SHIFT(23),
  [776] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_app_declaration, 2, 0, 0),
  [778] = {.entry = {.count = 1, .reusable = true}}, SHIFT(298),
  [780] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_store_declaration, 3, 0, 0),
  [782] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_parameter, 1, 0, 0),
  [784] = {.entry = {.count = 1, .reusable = true}}, SHIFT(278),
  [786] = {.entry = {.count = 1, .reusable = true}}, SHIFT(424),
  [788] = {.entry = {.count = 1, .reusable = true}}, SHIFT(172),
  [790] = {.entry = {.count = 1, .reusable = true}}, SHIFT(307),
  [792] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_media_query_repeat1, 2, 0, 0),
  [794] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_media_query_repeat1, 2, 0, 0), SHIFT_REPEAT(291),
  [797] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_component_declaration, 5, 0, 0),
  [799] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym__declaration, 1, 0, 0),
  [801] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_page_declaration, 5, 0, 0),
  [803] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_page_declaration, 6, 0, 0),
  [805] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_component_declaration, 6, 0, 0),
  [807] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_source_file_repeat1, 1, 0, 0),
  [809] = {.entry = {.count = 1, .reusable = false}}, REDUCE(sym_interpolation, 3, 0, 0),
  [811] = {.entry = {.count = 1, .reusable = true}}, SHIFT(319),
  [813] = {.entry = {.count = 1, .reusable = true}}, SHIFT(264),
  [815] = {.entry = {.count = 1, .reusable = true}}, SHIFT(181),
  [817] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_parameter, 2, 0, 0),
  [819] = {.entry = {.count = 1, .reusable = true}}, SHIFT(404),
  [821] = {.entry = {.count = 1, .reusable = true}}, SHIFT(185),
  [823] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_transition_block_repeat1, 2, 0, 0),
  [825] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_transition_block_repeat1, 2, 0, 0), SHIFT_REPEAT(179),
  [828] = {.entry = {.count = 1, .reusable = true}}, SHIFT(348),
  [830] = {.entry = {.count = 1, .reusable = true}}, SHIFT(134),
  [832] = {.entry = {.count = 1, .reusable = true}}, SHIFT(179),
  [834] = {.entry = {.count = 1, .reusable = true}}, SHIFT(78),
  [836] = {.entry = {.count = 1, .reusable = true}}, SHIFT(83),
  [838] = {.entry = {.count = 1, .reusable = true}}, SHIFT(187),
  [840] = {.entry = {.count = 1, .reusable = true}}, SHIFT(148),
  [842] = {.entry = {.count = 1, .reusable = false}}, REDUCE(aux_sym_string_repeat1, 1, 0, 0),
  [844] = {.entry = {.count = 1, .reusable = true}}, SHIFT(337),
  [846] = {.entry = {.count = 1, .reusable = true}}, SHIFT(196),
  [848] = {.entry = {.count = 1, .reusable = true}}, SHIFT(398),
  [850] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_property_name, 1, 0, 0),
  [852] = {.entry = {.count = 1, .reusable = true}}, SHIFT(428),
  [854] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_style_block_repeat1, 1, 0, 0),
  [856] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_argument_list, 2, 0, 0),
  [858] = {.entry = {.count = 1, .reusable = true}}, SHIFT(154),
  [860] = {.entry = {.count = 1, .reusable = true}}, SHIFT(30),
  [862] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_parameter_list_repeat1, 2, 0, 0),
  [864] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_parameter_list_repeat1, 2, 0, 0), SHIFT_REPEAT(336),
  [867] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_property_name, 2, 0, 0),
  [869] = {.entry = {.count = 1, .reusable = true}}, SHIFT(346),
  [871] = {.entry = {.count = 1, .reusable = true}}, SHIFT(201),
  [873] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_media_query, 5, 0, 0),
  [875] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_parameter_list, 1, 0, 0),
  [877] = {.entry = {.count = 1, .reusable = true}}, SHIFT(336),
  [879] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_array_repeat1, 2, 0, 0), SHIFT_REPEAT(180),
  [882] = {.entry = {.count = 1, .reusable = true}}, SHIFT(207),
  [884] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_property_name_repeat1, 2, 0, 0),
  [886] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_property_name_repeat1, 2, 0, 0), SHIFT_REPEAT(428),
  [889] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_parameter_list, 2, 0, 0),
  [891] = {.entry = {.count = 1, .reusable = true}}, SHIFT(210),
  [893] = {.entry = {.count = 1, .reusable = true}}, SHIFT(392),
  [895] = {.entry = {.count = 1, .reusable = true}}, SHIFT(28),
  [897] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_media_query, 4, 0, 0),
  [899] = {.entry = {.count = 1, .reusable = true}}, SHIFT(41),
  [901] = {.entry = {.count = 1, .reusable = true}}, SHIFT(162),
  [903] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_object_repeat1, 2, 0, 0), SHIFT_REPEAT(346),
  [906] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_object_repeat1, 2, 0, 0),
  [908] = {.entry = {.count = 1, .reusable = true}}, SHIFT(46),
  [910] = {.entry = {.count = 1, .reusable = true}}, SHIFT(52),
  [912] = {.entry = {.count = 1, .reusable = true}}, SHIFT(54),
  [914] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_argument_list_repeat1, 2, 0, 0),
  [916] = {.entry = {.count = 2, .reusable = true}}, REDUCE(aux_sym_argument_list_repeat1, 2, 0, 0), SHIFT_REPEAT(154),
  [919] = {.entry = {.count = 1, .reusable = true}}, SHIFT(19),
  [921] = {.entry = {.count = 1, .reusable = true}}, SHIFT(160),
  [923] = {.entry = {.count = 1, .reusable = true}}, SHIFT(22),
  [925] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_argument_list, 1, 0, 0),
  [927] = {.entry = {.count = 1, .reusable = true}}, SHIFT(7),
  [929] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_transition_block_repeat1, 1, 0, 0),
  [931] = {.entry = {.count = 1, .reusable = true}}, SHIFT(29),
  [933] = {.entry = {.count = 1, .reusable = true}}, SHIFT(151),
  [935] = {.entry = {.count = 1, .reusable = true}}, SHIFT(396),
  [937] = {.entry = {.count = 1, .reusable = true}}, SHIFT(423),
  [939] = {.entry = {.count = 1, .reusable = true}}, SHIFT(176),
  [941] = {.entry = {.count = 1, .reusable = true}}, SHIFT(209),
  [943] = {.entry = {.count = 1, .reusable = true}}, SHIFT(152),
  [945] = {.entry = {.count = 1, .reusable = true}}, SHIFT(53),
  [947] = {.entry = {.count = 1, .reusable = true}}, SHIFT(153),
  [949] = {.entry = {.count = 1, .reusable = true}}, REDUCE(aux_sym_media_query_repeat1, 1, 0, 0),
  [951] = {.entry = {.count = 1, .reusable = true}}, SHIFT(441),
  [953] = {.entry = {.count = 1, .reusable = true}}, SHIFT(190),
  [955] = {.entry = {.count = 1, .reusable = true}}, SHIFT(234),
  [957] = {.entry = {.count = 1, .reusable = true}}, SHIFT(68),
  [959] = {.entry = {.count = 1, .reusable = true}}, SHIFT(211),
  [961] = {.entry = {.count = 1, .reusable = true}}, SHIFT(117),
  [963] = {.entry = {.count = 1, .reusable = true}}, SHIFT(391),
  [965] = {.entry = {.count = 1, .reusable = true}}, SHIFT(325),
  [967] = {.entry = {.count = 1, .reusable = true}}, SHIFT(177),
  [969] = {.entry = {.count = 1, .reusable = true}}, SHIFT(26),
  [971] = {.entry = {.count = 1, .reusable = true}}, SHIFT(347),
  [973] = {.entry = {.count = 1, .reusable = true}}, SHIFT(331),
  [975] = {.entry = {.count = 1, .reusable = true}}, SHIFT(31),
  [977] = {.entry = {.count = 1, .reusable = true}}, SHIFT(212),
  [979] = {.entry = {.count = 1, .reusable = true}}, SHIFT(393),
  [981] = {.entry = {.count = 1, .reusable = true}}, SHIFT(349),
  [983] = {.entry = {.count = 1, .reusable = false}}, SHIFT(443),
  [985] = {.entry = {.count = 1, .reusable = true}}, SHIFT(159),
  [987] = {.entry = {.count = 1, .reusable = true}}, SHIFT(73),
  [989] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_animate_clause, 4, 0, 0),
  [991] = {.entry = {.count = 1, .reusable = true}}, SHIFT(251),
  [993] = {.entry = {.count = 1, .reusable = true}}, SHIFT(280),
  [995] = {.entry = {.count = 1, .reusable = true}}, SHIFT(406),
  [997] = {.entry = {.count = 1, .reusable = true}}, SHIFT(161),
  [999] = {.entry = {.count = 1, .reusable = true}}, SHIFT(276),
  [1001] = {.entry = {.count = 1, .reusable = true}}, SHIFT(429),
  [1003] = {.entry = {.count = 1, .reusable = true}}, SHIFT(113),
  [1005] = {.entry = {.count = 1, .reusable = false}}, SHIFT(444),
  [1007] = {.entry = {.count = 1, .reusable = true}}, SHIFT(111),
  [1009] = {.entry = {.count = 1, .reusable = true}}, SHIFT(123),
  [1011] = {.entry = {.count = 1, .reusable = true}}, SHIFT(50),
  [1013] = {.entry = {.count = 1, .reusable = true}}, SHIFT(173),
  [1015] = {.entry = {.count = 1, .reusable = true}}, SHIFT(146),
  [1017] = {.entry = {.count = 1, .reusable = true}}, SHIFT(168),
  [1019] = {.entry = {.count = 1, .reusable = true}}, SHIFT(55),
  [1021] = {.entry = {.count = 1, .reusable = true}}, SHIFT(56),
  [1023] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_animate_clause, 3, 0, 0),
  [1025] = {.entry = {.count = 1, .reusable = true}}, SHIFT(309),
  [1027] = {.entry = {.count = 1, .reusable = true}}, SHIFT(169),
  [1029] = {.entry = {.count = 1, .reusable = true}}, SHIFT(194),
  [1031] = {.entry = {.count = 1, .reusable = true}}, SHIFT(175),
  [1033] = {.entry = {.count = 1, .reusable = true}}, SHIFT(400),
  [1035] = {.entry = {.count = 1, .reusable = true}}, SHIFT(405),
  [1037] = {.entry = {.count = 1, .reusable = false}}, SHIFT(383),
  [1039] = {.entry = {.count = 1, .reusable = true}}, SHIFT(235),
  [1041] = {.entry = {.count = 1, .reusable = true}}, SHIFT(253),
  [1043] = {.entry = {.count = 1, .reusable = true}}, SHIFT(282),
  [1045] = {.entry = {.count = 1, .reusable = true}}, SHIFT(33),
  [1047] = {.entry = {.count = 1, .reusable = true}}, SHIFT(334),
  [1049] = {.entry = {.count = 1, .reusable = true}}, SHIFT(407),
  [1051] = {.entry = {.count = 1, .reusable = true}}, SHIFT(388),
  [1053] = {.entry = {.count = 1, .reusable = true}}, SHIFT(79),
  [1055] = {.entry = {.count = 1, .reusable = true}}, SHIFT(375),
  [1057] = {.entry = {.count = 1, .reusable = true}}, SHIFT(205),
  [1059] = {.entry = {.count = 1, .reusable = true}}, SHIFT(341),
  [1061] = {.entry = {.count = 1, .reusable = true}}, SHIFT(322),
  [1063] = {.entry = {.count = 1, .reusable = true}}, SHIFT(390),
  [1065] = {.entry = {.count = 1, .reusable = true}}, SHIFT(316),
  [1067] = {.entry = {.count = 1, .reusable = true}}, SHIFT(265),
  [1069] = {.entry = {.count = 1, .reusable = true}},  ACCEPT_INPUT(),
  [1071] = {.entry = {.count = 1, .reusable = true}}, SHIFT(287),
  [1073] = {.entry = {.count = 1, .reusable = true}}, SHIFT(352),
  [1075] = {.entry = {.count = 1, .reusable = true}}, SHIFT(340),
  [1077] = {.entry = {.count = 1, .reusable = false}}, SHIFT(425),
  [1079] = {.entry = {.count = 1, .reusable = true}}, SHIFT(189),
  [1081] = {.entry = {.count = 1, .reusable = true}}, SHIFT(358),
  [1083] = {.entry = {.count = 1, .reusable = true}}, SHIFT(289),
  [1085] = {.entry = {.count = 1, .reusable = true}}, SHIFT(163),
  [1087] = {.entry = {.count = 1, .reusable = true}}, SHIFT(191),
  [1089] = {.entry = {.count = 1, .reusable = true}}, SHIFT(192),
  [1091] = {.entry = {.count = 1, .reusable = true}}, SHIFT(365),
  [1093] = {.entry = {.count = 1, .reusable = true}}, SHIFT(433),
  [1095] = {.entry = {.count = 1, .reusable = true}}, SHIFT(367),
  [1097] = {.entry = {.count = 1, .reusable = true}}, SHIFT(435),
  [1099] = {.entry = {.count = 1, .reusable = true}}, SHIFT(436),
  [1101] = {.entry = {.count = 1, .reusable = true}}, SHIFT(386),
  [1103] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_comment, 3, 0, 0),
  [1105] = {.entry = {.count = 1, .reusable = true}}, REDUCE(sym_comment, 2, 0, 0),
};

#ifdef __cplusplus
extern "C" {
#endif
#ifdef TREE_SITTER_HIDE_SYMBOLS
#define TS_PUBLIC
#elif defined(_WIN32)
#define TS_PUBLIC __declspec(dllexport)
#else
#define TS_PUBLIC __attribute__((visibility("default")))
#endif

TS_PUBLIC const TSLanguage *tree_sitter_webfluent(void) {
  static const TSLanguage language = {
    .version = LANGUAGE_VERSION,
    .symbol_count = SYMBOL_COUNT,
    .alias_count = ALIAS_COUNT,
    .token_count = TOKEN_COUNT,
    .external_token_count = EXTERNAL_TOKEN_COUNT,
    .state_count = STATE_COUNT,
    .large_state_count = LARGE_STATE_COUNT,
    .production_id_count = PRODUCTION_ID_COUNT,
    .field_count = FIELD_COUNT,
    .max_alias_sequence_length = MAX_ALIAS_SEQUENCE_LENGTH,
    .parse_table = &ts_parse_table[0][0],
    .small_parse_table = ts_small_parse_table,
    .small_parse_table_map = ts_small_parse_table_map,
    .parse_actions = ts_parse_actions,
    .symbol_names = ts_symbol_names,
    .field_names = ts_field_names,
    .field_map_slices = ts_field_map_slices,
    .field_map_entries = ts_field_map_entries,
    .symbol_metadata = ts_symbol_metadata,
    .public_symbol_map = ts_symbol_map,
    .alias_map = ts_non_terminal_alias_map,
    .alias_sequences = &ts_alias_sequences[0][0],
    .lex_modes = ts_lex_modes,
    .lex_fn = ts_lex,
    .primary_state_ids = ts_primary_state_ids,
  };
  return &language;
}
#ifdef __cplusplus
}
#endif
