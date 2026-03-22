#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

source "$HOME/.cargo/env" 2>/dev/null || true

echo "==> Rust version: $(rustc --version)"

echo "==> Adding wasm32-unknown-unknown target..."
rustup target add wasm32-unknown-unknown

echo "==> Installing wasm-pack..."
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

echo "==> Building bingo-wasm..."
cd "$REPO_ROOT/bingo-wasm"
wasm-pack build --target web --out-dir ../web/src/wasm

echo "==> Building web..."
cd "$REPO_ROOT/web"
npm install
npm run build

echo "==> Done."
