#!/usr/bin/env bash
# Build a self-contained Linux x86_64 release and tar it up.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VERSION="$(awk -F'"' '/^version = /{print $2; exit}' "$ROOT/Cargo.toml")"
NAME="tapper-last-call-linux-x86_64"
STAGE="$ROOT/dist/$NAME"
TAR="$ROOT/dist/$NAME-$VERSION.tar.gz"

cd "$ROOT"

echo "==> cargo build --release"
cargo build --release --bin tapper-last-call

rm -rf "$STAGE"
mkdir -p "$STAGE/assets"
cp "$ROOT/target/release/tapper-last-call" "$STAGE/"
cp -R "$ROOT/assets/sprites" "$ROOT/assets/backgrounds" "$ROOT/assets/audio" "$ROOT/assets/fonts" "$STAGE/assets/"
cp "$ROOT/README.md" "$ROOT/LICENSE" "$STAGE/"

cat > "$STAGE/run.sh" <<'EOF'
#!/usr/bin/env bash
cd "$(dirname "$0")"
exec ./tapper-last-call "$@"
EOF
chmod +x "$STAGE/run.sh" "$STAGE/tapper-last-call"

tar -czf "$TAR" -C "$ROOT/dist" "$NAME"

echo
echo "packaged: $TAR"
du -sh "$TAR" "$STAGE"
