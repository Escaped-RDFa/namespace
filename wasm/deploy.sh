#!/bin/sh
set -e
DIR="$(cd "$(dirname "$0")" && pwd)"
DEST=/var/www/solana.solfunmeme.com/erdfa

cd "$DIR"
nix-shell -p wasm-pack rustup --run "wasm-pack build --target web --release"

sudo cp index.html "$DEST/"
sudo cp pkg/erdfa_wasm.js "$DEST/pkg/"
sudo cp pkg/erdfa_wasm_bg.wasm "$DEST/pkg/"
echo "✅ deployed to $DEST"
