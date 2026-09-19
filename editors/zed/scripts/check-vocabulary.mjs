#!/usr/bin/env node
// The modifier words highlights.scm colours as attributes must be exactly the
// words the compiler accepts as modifiers (`src/parser/vocabulary.rs`), and
// the pseudo-states it colours as keywords exactly `Parser::PSEUDO_STATES`.
// Either list drifting means a word the compiler takes is drawn as a plain
// variable, or one it rejects is drawn as if it were fine.
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

function queryList(source, marker) {
  const start = source.indexOf(marker);
  const end = source.indexOf("))", start);
  return [...source.slice(start, end).matchAll(/"([A-Za-z0-9_-]+)"/g)].map((m) => m[1]);
}

const highlights = read("editors/zed/languages/webfluent/highlights.scm");
const checks = [
  {
    what: "modifier vocabulary",
    expected: rustList(read("src/parser/vocabulary.rs"), "MODIFIER_KEYWORDS"),
    actual: queryList(highlights, "(#any-of? @attribute"),
  },
  {
    what: "pseudo-states",
    expected: rustList(read("src/parser/parser.rs"), "PSEUDO_STATES"),
    actual: queryList(highlights, "(#any-of? @keyword"),
  },
];

let failed = false;
for (const { what, expected, actual } of checks) {
  const missing = expected.filter((w) => !actual.includes(w));
  const extra = actual.filter((w) => !expected.includes(w));
  if (missing.length || extra.length) {
    failed = true;
    console.error(`highlights.scm ${what} is out of step with the compiler:`);
    if (missing.length) console.error(`  missing: ${missing.join(", ")}`);
    if (extra.length) console.error(`  extra:   ${extra.join(", ")}`);
  } else {
    console.log(`${what}: ${expected.length} words, in step`);
  }
}
process.exit(failed ? 1 : 0);
