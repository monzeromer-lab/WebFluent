#!/usr/bin/env bash
# How far the code is from BeautifulRust's lint set: every lint Cargo.toml's
# [workspace.lints] still holds at "allow" because it has findings, switched
# to "warn" here, and counted — one line per lint, findings in tests left
# out, the same finding in two build targets counted once.
#
# Usage: scripts/lint-debt.sh
# It never fails on findings; it fails only if clippy cannot run.
set -euo pipefail
cd "$(dirname "$0")/.."

# The lints held at "allow" with a count beside them in Cargo.toml.
mapfile -t held < <(sed -n 's/^\([a-z_0-9]*\) = .*"allow".*# *[0-9].*/\1/p; s/^\([a-z_0-9]*\) = { level = "allow".*# *[0-9].*/\1/p' Cargo.toml)
flags=()
for lint in "${held[@]}"; do
    case "$lint" in
        missing_debug_implementations | missing_docs | unreachable_pub | rust_2018_idioms) flags+=("-W" "$lint") ;;
        *) flags+=("-W" "clippy::$lint") ;;
    esac
done
# What BeautifulRust leaves off on purpose stays off: `-W clippy::pedantic`
# above would otherwise bring these back.
for lint in module_name_repetitions must_use_candidate module_inception enum_glob_use expect_used; do
    flags+=("-A" "clippy::$lint")
done

cargo clippy -q --workspace --all-targets --message-format=json -- "${flags[@]}" 2>/dev/null |
    python3 -c '
import collections, json, sys
seen, count = set(), collections.Counter()
for line in sys.stdin:
    try:
        m = json.loads(line)
    except ValueError:
        continue
    if m.get("reason") != "compiler-message":
        continue
    msg = m["message"]
    code = (msg.get("code") or {}).get("code")
    spans = [s for s in msg["spans"] if s["is_primary"]]
    if not code or msg["level"] != "warning" or not spans:
        continue
    span = spans[0]
    path = span["file_name"]
    if path.startswith("/") or path.startswith("tests/") or "/tests/" in path:
        continue
    key = (code, path, span["line_start"], span["column_start"])
    if key not in seen:
        seen.add(key)
        count[code] += 1
for code, n in sorted(count.items(), key=lambda kv: -kv[1]):
    print(f"{n:6}  {code}")
print(f"{sum(count.values()):6}  in all (outside tests)")
'
