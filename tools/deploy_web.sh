#!/usr/bin/env bash
# Publish the staged web build to the dedicated Pages repository.
#
#   tools/deploy_web.sh [path-to-tapper-web-clone]
#
# The destination defaults to ../tapper-web. It must already be a clone of
# github.com/orbitmechanic/tapper-web (create it once with `gh repo create`).
# README/LICENSE are preserved; everything else is replaced by web/dist.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEST="${1:-$ROOT/../tapper-web}"

if [[ ! -d "$DEST/.git" ]]; then
  echo "destination is not a git clone: $DEST" >&2
  echo "clone orbitmechanic/tapper-web there first" >&2
  exit 1
fi

if [[ ! -f "$ROOT/web/dist/index.html" ]]; then
  echo "web/dist is missing; run tools/build_web.sh first" >&2
  exit 1
fi

rsync -a --delete \
  --exclude '.git' \
  --exclude 'README.md' \
  --exclude 'LICENSE' \
  "$ROOT/web/dist/" "$DEST/"
cp "$ROOT/web/README.md" "$DEST/README.md"
cp "$ROOT/LICENSE" "$DEST/LICENSE"

cd "$DEST"
git add -A
if git diff --cached --quiet; then
  echo "web build already up to date"
  exit 0
fi
git commit -m "Publish TAPPER: LAST CALL web build"
git push origin main
echo "published from $DEST"
