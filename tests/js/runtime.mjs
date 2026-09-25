//! Assemble the runtime the way the compiler does.
//!
//! The runtime is a set of feature modules under `src/runtime/modules/`, and a
//! build carries only the ones its program reaches. `modules/manifest.json` is
//! written by `src/runtime/mod.rs` — a Rust test fails when it goes stale — so
//! the whole runtime a test loads here is assembled by the same rules.

import { readFileSync } from "node:fs";

const MANIFEST = JSON.parse(
  readFileSync(new URL("../../src/runtime/modules/manifest.json", import.meta.url), "utf8"),
);

/// Every module, in one closure, exporting everything it holds.
export function fullRuntime() {
  let out = MANIFEST.header;
  for (const m of MANIFEST.modules) {
    let src = readFileSync(
      new URL(`../../src/runtime/modules/${m.name}.js`, import.meta.url),
      "utf8",
    );
    if (!src.endsWith("\n")) src += "\n";
    out += src;
  }
  out += "  return {\n";
  for (const m of MANIFEST.modules) {
    if (!m.exports.length) continue;
    out += "    " + m.exports.map(([name, local]) => (name === local ? name : `${name}: ${local}`)).join(", ") + ",\n";
  }
  out += "    get _basePath() { return _basePath; },\n";
  out += "  };\n})();\n";
  return out;
}

/// One module and what it needs, assembled the way a build would.
///
/// A module's source is a fragment of the closure, so a module alone is not a
/// program; what can be asserted is that core plus a module plus the modules
/// it declares reaching for evaluates, and exports what it says it does.
export function runtimeFor(name) {
  const want = new Set(["core", name]);
  for (let added = true; added; ) {
    added = false;
    for (const m of MANIFEST.modules) {
      if (!want.has(m.name)) continue;
      for (const d of m.deps) if (!want.has(d)) { want.add(d); added = true; }
    }
  }
  let out = MANIFEST.header;
  const kept = MANIFEST.modules.filter((m) => want.has(m.name));
  for (const m of kept) {
    let src = readFileSync(new URL(`../../src/runtime/modules/${m.name}.js`, import.meta.url), "utf8");
    if (!src.endsWith("\n")) src += "\n";
    out += src;
  }
  out += "  return {\n";
  for (const m of kept) {
    if (!m.exports.length) continue;
    out += "    " + m.exports.map(([n, l]) => (n === l ? n : `${n}: ${l}`)).join(", ") + ",\n";
  }
  out += "    get _basePath() { return _basePath; },\n  };\n})();\n";
  return out;
}

/// Every module's name, in the order a build emits them.
export const moduleNames = MANIFEST.modules.map((m) => m.name);

/// What a module puts on `WF`.
export function exportsOf(name) {
  return MANIFEST.modules.find((m) => m.name === name).exports.map(([n]) => n);
}
