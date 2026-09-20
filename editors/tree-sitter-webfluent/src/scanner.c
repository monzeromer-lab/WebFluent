// What the grammar cannot read by itself: where the end of a line means
// something.
//
// A chunk of raw CSS in a style value runs to the end of its line, a `;` or
// the closing brace, whichever comes first outside parentheses and quotes;
// `$token` and `{expr}` splices inside it are the grammar's own tokens, so a
// chunk stops before `$` and `{`. A line that ends with a comma continues
// on the next, as in the compiler's lexer (`src/lexer/v2.rs`).
//
// In the indented layout a block is the lines indented under the line that
// opens it: where the grammar allows a block, a line indented deeper than
// the line before it is an `_indent`, and a line indented less than the
// innermost indented block closes one block per level it leaves, a
// `_dedent` each. Both are zero-width, at the end of the line before, and
// are only offered where the grammar could take them. Blank lines and
// comment lines do not count. The stack of open indents, the indentation
// of the current line and the count of braces the writer has opened are
// the scanner's state; a line is measured against the line before, not the
// stack, so a braced block's lines — which push nothing — open no block of
// their own, and inside the writer's own braces the layout is free, as it
// is for the compiler.
//
// Because the state only survives with a token, the braces pass through
// the scanner, and `_layout`, an empty extra, records a line at the same
// level. Blanks the scanner skipped without finding a token are the
// parser's again when it declines. A comment is the grammar's own token: a
// line that starts with `/` decides nothing, and the line after it is
// measured once the comment has been read.

#include "tree_sitter/parser.h"
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

// One scanner, two grammars: `tree-sitter-webfluentx/src/scanner.c`
// includes this file with `WF_GRAMMAR` set to its own name, so the entry
// points carry the name the generated parser calls.
#ifndef WF_GRAMMAR
#define WF_GRAMMAR webfluent
#endif
#define WF_CAT2(a, b) a##b
#define WF_CAT(a, b) WF_CAT2(a, b)
#define WF_SCANNER(fn) WF_CAT(WF_CAT(tree_sitter_, WF_GRAMMAR), WF_CAT(_external_scanner_, fn))

enum TokenType { STYLE_TEXT, INDENT, DEDENT, LAYOUT, LBRACE, RBRACE };

#define MAX_DEPTH 64

typedef struct {
  uint16_t line_indent;
  // Braces the writer wrote and has not closed.
  uint16_t explicit;
  uint16_t depth;
  uint16_t indents[MAX_DEPTH];
} Scanner;

void *WF_SCANNER(create)(void) { return calloc(1, sizeof(Scanner)); }

void WF_SCANNER(destroy)(void *payload) { free(payload); }

#define HEAD (2 * sizeof(uint16_t))

unsigned WF_SCANNER(serialize)(void *payload, char *buffer) {
  Scanner *s = payload;
  unsigned n = HEAD + s->depth * sizeof(uint16_t);
  if (n > TREE_SITTER_SERIALIZATION_BUFFER_SIZE) n = TREE_SITTER_SERIALIZATION_BUFFER_SIZE;
  memcpy(buffer, &s->line_indent, sizeof(uint16_t));
  memcpy(buffer + sizeof(uint16_t), &s->explicit, sizeof(uint16_t));
  memcpy(buffer + HEAD, s->indents, n - HEAD);
  return n;
}

void WF_SCANNER(deserialize)(void *payload, const char *buffer,
                                                        unsigned length) {
  Scanner *s = payload;
  s->line_indent = 0;
  s->explicit = 0;
  s->depth = 0;
  if (length < HEAD) return;
  memcpy(&s->line_indent, buffer, sizeof(uint16_t));
  memcpy(&s->explicit, buffer + sizeof(uint16_t), sizeof(uint16_t));
  s->depth = (length - HEAD) / sizeof(uint16_t);
  if (s->depth > MAX_DEPTH) s->depth = MAX_DEPTH;
  memcpy(s->indents, buffer + HEAD, s->depth * sizeof(uint16_t));
}

static bool is_space(int32_t c) { return c == ' ' || c == '\t' || c == '\r'; }

static bool scan_style_text(TSLexer *lexer) {
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

bool WF_SCANNER(scan)(void *payload, TSLexer *lexer,
                                                 const bool *valid_symbols) {
  Scanner *s = payload;
  // Every symbol at once is error recovery: nothing is decided here then.
  if (valid_symbols[STYLE_TEXT] && valid_symbols[INDENT] && valid_symbols[DEDENT] &&
      valid_symbols[LBRACE] && valid_symbols[RBRACE]) {
    return false;
  }
  // A value first: it stops at the end of its line without crossing it,
  // where the layout is then read.
  if (valid_symbols[STYLE_TEXT] && scan_style_text(lexer)) return true;

  // A layout token, if any, is zero-width at the end of the line before.
  lexer->mark_end(lexer);
  bool end_of_line = false;
  uint32_t indent = 0;
  for (;;) {
    int32_t c = lexer->lookahead;
    if (c == '\n') {
      end_of_line = true;
      indent = 0;
    } else if (c == ' ' || c == '\r') {
      indent++;
    } else if (c == '\t') {
      indent += 4;
    } else if (c == 0) {
      end_of_line = true;
      indent = 0;
      break;
    } else {
      break;
    }
    lexer->advance(lexer, true);
  }
  if (end_of_line && s->explicit == 0 && lexer->lookahead != '/') {
    if (valid_symbols[INDENT] && indent > s->line_indent && s->depth < MAX_DEPTH) {
      s->indents[s->depth++] = (uint16_t)indent;
      s->line_indent = (uint16_t)indent;
      lexer->result_symbol = INDENT;
      return true;
    }
    if (valid_symbols[DEDENT] && s->depth > 0 && indent < s->indents[s->depth - 1]) {
      s->depth--;
      lexer->result_symbol = DEDENT;
      return true;
    }
    if (valid_symbols[LAYOUT] && indent != s->line_indent) {
      s->line_indent = (uint16_t)indent;
      lexer->result_symbol = LAYOUT;
      return true;
    }
  }
  // A brace the writer wrote: counted, so the layout inside it is free.
  if (valid_symbols[LBRACE] && lexer->lookahead == '{') {
    lexer->advance(lexer, false);
    lexer->mark_end(lexer);
    if (s->explicit < UINT16_MAX) s->explicit++;
    lexer->result_symbol = LBRACE;
    return true;
  }
  if (valid_symbols[RBRACE] && lexer->lookahead == '}') {
    lexer->advance(lexer, false);
    lexer->mark_end(lexer);
    if (s->explicit > 0) s->explicit--;
    lexer->result_symbol = RBRACE;
    return true;
  }
  return false;
}
