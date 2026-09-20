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

const checks = [
  {
    what: "built-in component names",
    expected: rustList(read("src/lexer/token.rs"), "ALL_COMPONENT_NAMES"),
    actual: jsList(read("editors/tree-sitter-webfluent/grammar.js"), "BUILTIN_COMPONENTS"),
  },
];

let failed = false;
for (const { what, expected, actual } of checks) {
  const missing = expected.filter((w) => !actual.includes(w));
  const extra = actual.filter((w) => !expected.includes(w));
  if (missing.length || extra.length) {
    failed = true;
    console.error(`grammar.js ${what} are out of step with the compiler:`);
    if (missing.length) console.error(`  missing: ${missing.join(", ")}`);
    if (extra.length) console.error(`  extra:   ${extra.join(", ")}`);
  } else {
    console.log(`${what}: ${expected.length} names, in step`);
  }
}
process.exit(failed ? 1 : 0);
