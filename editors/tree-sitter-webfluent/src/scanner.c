// The one token the grammar cannot read by itself: a chunk of raw CSS in a
// style value. A value runs to the end of its line, a `;` or the closing
// brace, whichever comes first outside parentheses and quotes; `$token` and
// `{expr}` splices inside it are the grammar's own tokens, so a chunk stops
// before `$` and `{`. A line that ends with a comma continues on the next,
// as in the compiler's lexer (`src/lexer/v2.rs`).
//
// The end of a line means nothing anywhere else in the language, which is
// why this lives in a scanner rather than the grammar: whitespace,
// newlines included, is an extra everywhere but here.

#include "tree_sitter/parser.h"
#include <stdbool.h>

enum TokenType { STYLE_TEXT };

void *tree_sitter_webfluent_external_scanner_create(void) { return NULL; }
void tree_sitter_webfluent_external_scanner_destroy(void *payload) { (void)payload; }
unsigned tree_sitter_webfluent_external_scanner_serialize(void *payload, char *buffer) {
  (void)payload;
  (void)buffer;
  return 0;
}
void tree_sitter_webfluent_external_scanner_deserialize(void *payload, const char *buffer,
                                                        unsigned length) {
  (void)payload;
  (void)buffer;
  (void)length;
}

static bool is_space(int32_t c) { return c == ' ' || c == '\t' || c == '\r'; }

bool tree_sitter_webfluent_external_scanner_scan(void *payload, TSLexer *lexer,
                                                 const bool *valid_symbols) {
  (void)payload;
  if (!valid_symbols[STYLE_TEXT]) return false;

  // Leading blanks belong to nobody; a line break before any text means
  // the value is over.
  while (is_space(lexer->lookahead)) lexer->advance(lexer, true);
  if (lexer->lookahead == '\n' || lexer->lookahead == 0) return false;
  if (lexer->lookahead == '{' || lexer->lookahead == '}' || lexer->lookahead == '$' ||
      lexer->lookahead == ';') {
    return false;
  }

  int parens = 0;
  bool consumed = false;
  int32_t last = 0;
  while (lexer->lookahead != 0) {
    int32_t c = lexer->lookahead;
    if (c == '\n') {
      // A trailing comma carries the value onto the next line.
      if (last == ',' || parens > 0) {
        lexer->advance(lexer, false);
        consumed = true;
        continue;
      }
      break;
    }
    if (c == '{' || c == '$') break;
    if (c == '}') break;
    if (c == ';' && parens == 0) break;
    // A comment after a space ends the value; `url(http://…)` keeps its
    // slashes.
    if (c == '/' && parens == 0 && last == ' ') {
      lexer->mark_end(lexer);
      lexer->advance(lexer, false);
      if (lexer->lookahead == '/') return consumed;
      consumed = true;
      last = '/';
      continue;
    }
    if (c == '"' || c == '\'') {
      // A quoted string, whole.
      int32_t quote = c;
      lexer->advance(lexer, false);
      while (lexer->lookahead != 0 && lexer->lookahead != quote && lexer->lookahead != '\n') {
        lexer->advance(lexer, false);
      }
      if (lexer->lookahead == quote) lexer->advance(lexer, false);
      consumed = true;
      last = quote;
      lexer->mark_end(lexer);
      continue;
    }
    if (c == '(') parens++;
    if (c == ')' && parens > 0) parens--;
    lexer->advance(lexer, false);
    consumed = true;
    if (!is_space(c)) last = c;
    else if (last != ',') last = ' ';
    lexer->mark_end(lexer);
  }
  if (!consumed) return false;
  lexer->result_symbol = STYLE_TEXT;
  return true;
}
