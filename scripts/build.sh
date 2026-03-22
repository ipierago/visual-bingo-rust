#!/bin/bash
set -e

# Determine script location so it works from any directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "==> Repo root: $REPO_ROOT"

# Install Rust if not present
if ! command -v rustup &>/dev/null; then
  echo "==> Installing Rust..."
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
  source "$HOME/.cargo/env"
else
  echo "==> Rust already installed: $(rustc --version)"
  source "$HOME/.cargo/env" 2>/dev/null || true
fi

# Add wasm32 target
echo "==> Adding wasm32-unknown-unknown target..."
rustup target add wasm32-unknown-unknown

# Install wasm-pack if not present
if ! command -v wasm-pack &>/dev/null; then
  echo "==> Installing wasm-pack..."
  curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
else
  echo "==> wasm-pack already installed: $(wasm-pack --version)"
fi

# Build WASM
echo "==> Building bingo-wasm..."
cd "$REPO_ROOT/bingo-wasm"
wasm-pack build --target web --out-dir ../web/src/wasm

# Build web
echo "==> Building web..."
cd "$REPO_ROOT/web"
npm install
npm run build

echo ""
echo "==> Done. Web output is in web/dist/"
