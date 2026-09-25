#!/usr/bin/env node
// The built-in component names the grammar knows (`BUILTIN_COMPONENTS` in
// `grammar.js`) must be exactly the names the compiler knows
// (`ALL_COMPONENT_NAMES` in `src/lexer/token.rs`, which the registry is
// pinned to): a name missing here is drawn as a user component, one the
// compiler dropped is drawn as built in. The keywords the highlight query
// colours must all be words the grammar has, or the query fails to compile
// — which `tree-sitter query` checks; this script checks the names.
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..", "..", "..");
const read = (path) => readFileSync(join(root, path), "utf8");

function rustList(source, name) {
  const start = source.indexOf(name);
  const end = source.indexOf("];", start);
  return [...source.slice(start, end).matchAll(/"([A-Za-z0-9_-]+)"/g)].map((m) => m[1]);
}

function jsList(source, name) {
  const start = source.indexOf(name);
  const end = source.indexOf("];", start);
  return [...source.slice(start, end).matchAll(/"([A-Za-z0-9_-]+)"/g)].map((m) => m[1]);
}

// The VS Code extension highlights with a TextMate grammar: no parser, only
// word lists. It kept its own copy of the component names and the keywords,
// with nothing holding either to anything, and by 4.0 it was missing twelve
// built-ins (`Textarea`, `Audio`, `Host`, `Unsafe`, the slides) and fifteen
// keywords the Zed grammar colours. Its component list is held to the
// compiler's here, and every keyword Zed colours must appear in it.
const tmLanguage = JSON.parse(read("editors/vscode/syntaxes/webfluent.tmLanguage.json"));
const tmText = JSON.stringify(tmLanguage);
const vscodeComponents = (() => {
  const rule = tmLanguage.repository["built-in-components"].patterns[0].match;
  const inner = rule.match(/\\b\(([^)]*)\)\\b/);
  return inner ? inner[1].split("|") : [];
})();
const zedKeywords = (() => {
  const query = read("editors/zed/languages/webfluent/highlights.scm");
  const words = new Set();
  for (const [, block] of query.matchAll(/\[([^\]]*)\]\s*@keyword/g)) {
    for (const [, w] of block.matchAll(/"([a-z_]+)"/g)) words.add(w);
  }
  for (const [, w] of query.matchAll(/"([a-z_]+)"\s*@keyword/g)) words.add(w);
  return [...words].sort();
})();

const checks = [
  {
    what: "built-in component names",
    file: "grammar.js",
    expected: rustList(read("src/lexer/token.rs"), "ALL_COMPONENT_NAMES"),
    actual: jsList(read("editors/tree-sitter-webfluent/grammar.js"), "BUILTIN_COMPONENTS"),
  },
  {
    what: "built-in component names",
    file: "the VS Code grammar",
    expected: rustList(read("src/lexer/token.rs"), "ALL_COMPONENT_NAMES"),
    actual: vscodeComponents,
  },
  {
    what: "keywords Zed colours",
    file: "the VS Code grammar",
    expected: zedKeywords,
    // A word counts wherever the TextMate grammar names it: it colours by
    // pattern, and several words sit in rules with lookaheads.
    actual: zedKeywords.filter((w) => new RegExp(`\\b${w}\\b`).test(tmText)),
    extraAllowed: true,
  },
];

let failed = false;
for (const { what, file, expected, actual, extraAllowed } of checks) {
  const missing = expected.filter((w) => !actual.includes(w));
  const extra = extraAllowed ? [] : actual.filter((w) => !expected.includes(w));
  if (missing.length || extra.length) {
    failed = true;
    console.error(`${file}: ${what} are out of step:`);
    if (missing.length) console.error(`  missing: ${missing.join(", ")}`);
    if (extra.length) console.error(`  extra:   ${extra.join(", ")}`);
  } else {
    console.log(`${file}: ${what}: ${expected.length}, in step`);
  }
}
process.exit(failed ? 1 : 0);
