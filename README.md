# visual-bingo-rust

Visual bingo card generator. React + TypeScript UI, Rust/WASM for card generation and PDF rendering.

---

## Repo Structure

```
bingo-core/     Rust library — shuffle, card generation, PDF rendering
bingo-cli/      Rust binary — reads TOML config, writes PDF to disk
bingo-wasm/     Rust WASM crate — exposes bingo-core to the browser
web/            React + TypeScript UI
scripts/
  dev.sh                  Build WASM + start dev server
  build.sh                Full local production build
  cloudflare-build.sh     Used by Cloudflare Pages CI
bingo.toml      CLI config (sample)
```

---

## Development

### First time setup

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add wasm32-unknown-unknown

# Install wasm-pack
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Install Node deps
cd web && npm install
```

### Run dev server

```bash
bash scripts/dev.sh
```

Builds WASM then starts Vite at `http://localhost:5173`.

> Rebuild WASM any time you change Rust code in `bingo-core` or `bingo-wasm`.

### Rebuild WASM only

```bash
cd bingo-wasm
wasm-pack build --target web --out-dir ../web/src/wasm
```

### Run Rust tests

```bash
cargo test -p bingo-core
```

### Lint and format

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cd web && npm run lint
```

---

## Adding Images

Images live in `web/src/assets/images/vocabulary/`. Each top-level folder is one set.

```
web/src/assets/images/vocabulary/
  animals/
    cat.jpg
  food/
    apple.jpg
```

Filenames become labels: `fire-truck.jpg` → "fire truck".

To generate `bingo.toml` entries from your images:

```bash
find web/src/assets/images/vocabulary -name "*.jpg" -o -name "*.png" | sort | while read f; do
  label=$(basename "$f" | sed 's/\.[^.]*$//' | sed 's/[-_]/ /g')
  echo "[[images]]"
  echo "path = \"$f\""
  echo "label = \"$label\""
  echo ""
done
```

---

## CLI

Generates a PDF directly from disk — useful for testing without the browser.

```bash
cargo run -p bingo-cli -- bingo.toml
```

`bingo.toml` format:

```toml
[settings]
seed = "test-1234"
card_count = 3
output = "bingo.pdf"

[[images]]
path = "web/src/assets/images/vocabulary/animals/cat.jpg"
label = "cat"

# ... at least 25 images required
```

---

## Deployment

Deploys automatically to Cloudflare Pages on push to `main`.

Build settings in Cloudflare dashboard:

| Setting                | Value                              |
| ---------------------- | ---------------------------------- |
| Build command          | `bash scripts/cloudflare-build.sh` |
| Build output directory | `web/dist`                         |
| Root directory         | `/`                                |
