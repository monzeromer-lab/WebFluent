/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

// The grammar of a `.wfx` file: WebFluent 4 with its blocks written by
// indentation. It is `../tree-sitter-webfluent/grammar.js` generated with
// the layout tokens on — one grammar file, two parsers — so this file only
// sets the flag and hands over. The scanner (`src/scanner.c`) is a copy of
// the base grammar's; `just grammar-sync` keeps it one.
process.env.WFX = "1";
module.exports = require("../tree-sitter-webfluent/grammar.js");
