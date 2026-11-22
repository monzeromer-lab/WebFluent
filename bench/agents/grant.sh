#!/usr/bin/env bash
# The two steps of the agent benchmark that need a person:
#   1. let Workshop's bridge through Docker's firewall (sudo; lost on reboot, run again then)
#   2. a long-lived subscription token from `claude setup-token`, saved where queue.mjs reads it
# Run it in a terminal: bench/agents/grant.sh
set -euo pipefail

echo "1/2  Letting workshopbr0 through Docker's firewall (asks for your password)"
if sudo nft list chain ip filter DOCKER-USER 2>/dev/null | grep -q 'iifname "workshopbr0" accept'; then
  echo "     already done"
else
  sudo nft insert rule ip filter DOCKER-USER iifname workshopbr0 accept
  sudo nft insert rule ip filter DOCKER-USER oifname workshopbr0 ct state related,established accept
  echo "     done"
fi

TOKEN_FILE="$HOME/.config/webfluent-bench/oauth-token"
echo
echo "2/2  A subscription token for the benchmark's agents"
if [ -s "$TOKEN_FILE" ]; then
  echo "     already saved in $TOKEN_FILE (delete it to make a new one)"
else
  echo "     Signing in: follow the link it prints, then copy the token it shows at the end."
  claude setup-token
  echo
  read -r -s -p "     Paste the token here (it is not shown) and press Enter: " TOKEN
  echo
  [ -n "$TOKEN" ] || { echo "     no token given; run this again" >&2; exit 1; }
  mkdir -p "$(dirname "$TOKEN_FILE")"
  (umask 077; printf '%s\n' "$TOKEN" > "$TOKEN_FILE")
  echo "     saved in $TOKEN_FILE (readable by you only)"
fi
echo
echo "All set. You can close this tab; the benchmark carries on by itself."
