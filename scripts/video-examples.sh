#!/usr/bin/env bash
# The launch videos' projects under examples/videos, held to what each video
# shows: every project checks clean with warnings denied and builds; the two
# invoices are one page; `typo` fails `wf check` with the T05 its video
# shows; `one-file-three-outputs` renders its one file three ways.
#
#   scripts/video-examples.sh [path/to/wf]    (default: target/release/wf)
#
# `just examples` runs it, and so does CI.
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"
wf="$(realpath "${1:-target/release/wf}")"
failed=0
fail() { echo "::error::$*"; failed=1; }
ok() { echo "  ok  $*"; }

for dir in examples/videos/*/; do
  name="$(basename "$dir")"
  case "$name" in
    typo)
      if out="$(cd "$dir" && "$wf" check 2>&1)"; then
        fail "typo: wf check passed; its video needs it to fail with T05"
      elif grep -qF "error[T05]: \`User\` has no field \`nmae\`" <<<"$out" \
        && grep -qF "help: Did you mean \`name\`?" <<<"$out"; then
        ok "typo fails with T05 and suggests \`name\`"
      else
        fail "typo: wf check failed, but not with the T05 its video shows"
        echo "$out"
      fi
      ;;
    one-file-three-outputs)
      # One project builds one kind of output; the file's three pages are
      # drawn by `wf render`, which needs no data for this one.
      if (cd "$dir" \
        && "$wf" check --deny-warnings \
        && "$wf" render src/Talk.wf --page Site -o build/talk.html \
        && "$wf" render src/Talk.wf --page Handout --format pdf -o build/handout.pdf \
        && "$wf" render src/Talk.wf --page Deck --format slides -o build/deck.pdf \
        && test -s build/talk.html && test -s build/handout.pdf && test -s build/deck.pdf); then
        ok "$name renders a page, a PDF and a deck"
      else
        fail "$name does not render its three outputs"
      fi
      ;;
    *)
      if out="$(cd "$dir" && "$wf" check --deny-warnings 2>&1 && "$wf" build 2>&1)"; then
        case "$name" in
          invoice | arabic-invoice)
            if grep -q ', 1 page(s)' <<<"$out"; then
              ok "$name builds clean, on one page"
            else
              fail "$name is no longer one page"
              echo "$out"
            fi
            ;;
          *) ok "$name builds clean" ;;
        esac
      else
        fail "$name does not build clean"
        echo "$out"
      fi
      ;;
  esac
done
exit "$failed"
