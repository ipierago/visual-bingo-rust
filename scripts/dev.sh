#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

source "$HOME/.cargo/env" 2>/dev/null || true

# Build WASM
echo "==> Building bingo-wasm..."
cd "$REPO_ROOT/bingo-wasm"
wasm-pack build --target web --out-dir ../web/src/wasm

# Start dev server
echo "==> Starting dev server..."
cd "$REPO_ROOT/web"
npm install
npm run dev
