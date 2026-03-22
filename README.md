# visual-bingo-rust

A client-side web application for generating printable visual bingo cards (A4 PDF) for teaching English vocabulary. Card generation and PDF rendering are written in Rust and compiled to WebAssembly. The UI is React + TypeScript.

This is part of a learning experiment building the same app with different tech stacks. See also:

- [visual-bingo-spa](https://github.com/ipierago/visual-bingo-spa) — pure TypeScript/React version

---

## Features

- Select images grouped into vocabulary sets
- Generate deterministic bingo cards using a seed
- Export A4 PDF with bingo cards + text call list
- Progress bar with cancel support
- Persists selections, seed, and card count across sessions
- Card generation and PDF rendering in Rust/WASM

---

## Project Structure

```
visual-bingo-rust/
  bingo-core/       Rust library — types, seeded shuffle, card generation, PDF rendering
  bingo-cli/        Rust binary — reads TOML config, writes PDF to disk
  bingo-wasm/       Rust WASM crate — exposes bingo-core to the browser
  web/              React + TypeScript UI
  scripts/
    build.sh        Full production build (WASM + web)
    dev.sh          Build WASM and start dev server
  bingo.toml        CLI config file (sample)
  Cargo.toml        Rust workspace
  netlify.toml      Netlify deployment config
```

---

## Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Node.js](https://nodejs.org/) 20+
- [wasm-pack](https://rustwasm.github.io/wasm-pack/)

On first run, `scripts/build.sh` will install Rust and wasm-pack automatically if they are not present.

---

## Quick Start

### Development

```bash
bash scripts/dev.sh
```

This will:

1. Build the WASM package from `bingo-wasm`
2. Install web dependencies
3. Start the Vite dev server at `http://localhost:5173`

> You must rebuild WASM any time you change Rust code in `bingo-core` or `bingo-wasm`.

### Production Build

```bash
bash scripts/build.sh
```

Output is in `web/dist/`. Deploy as static files to any host.

---

## Adding Images

Images live in `web/src/assets/images/vocabulary/`. Each top-level folder is one set:

```
web/src/assets/images/vocabulary/
  animals/
    cat.jpg
    dog.jpg
  food/
    apple.jpg
    banana.jpg
```

Rules:

- Each top-level folder = one set
- No nested sets
- Supported formats: jpg, jpeg, png
- Filenames become labels: `fire-truck.jpg` → "fire truck"

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

## CLI Usage

The CLI reads a TOML config file and writes a PDF directly to disk — useful for batch generation and debugging Rust code without the browser.

```bash
cargo run -p bingo-cli -- bingo.toml
```

Example `bingo.toml`:

```toml
[settings]
seed = "test-1234"
card_count = 3
output = "bingo.pdf"

[[images]]
path = "web/src/assets/images/vocabulary/animals/cat.jpg"
label = "cat"

[[images]]
path = "web/src/assets/images/vocabulary/food/apple.jpg"
label = "apple"

# ... at least 25 images required
```

---

## Architecture

```
bingo-core (Rust library)
  - Seeded PRNG (Mulberry32)
  - Fisher-Yates shuffle
  - Card generation (5x5 grid, deterministic)
  - PDF rendering (pdf-writer crate)
  - ImageData input — caller provides raw bytes (CLI reads from disk, WASM fetches via browser)

bingo-cli (Rust binary)
  - Reads TOML config
  - Loads image bytes from disk
  - Calls bingo-core
  - Writes PDF to disk

bingo-wasm (Rust WASM crate)
  - Thin wasm-bindgen wrapper around bingo-core
  - Exposes generate_cards() and generate_pdf_wasm() to JavaScript
  - JSON in/out for data, Uint8Array out for PDF bytes

web (React + TypeScript)
  - Image discovery via Vite import.meta.glob
  - Selection state, persistence (localStorage)
  - Fetches image bytes from browser, passes to WASM
  - Downloads generated PDF
```

### Seed System

The seed is a user-editable string. It is hashed (FNV-1a) into a u32 and used to seed a Mulberry32 PRNG. Each card derives its own RNG from `seed-N` so cards are stable regardless of how many you generate. The call list uses `seed-calllist`.

Same seed always produces identical output.

---

## Development Notes

### Rebuilding WASM

Any time you change Rust code you need to rebuild the WASM:

```bash
cd bingo-wasm
wasm-pack build --target web --out-dir ../web/src/wasm
```

### Running Rust tests

```bash
cargo test -p bingo-core
```

### Linting and formatting

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cd web && npm run lint
```

---

## Deployment

The app deploys to Netlify automatically on push to `main`.

Build settings are in `netlify.toml`. The build script installs Rust and wasm-pack on the Netlify build machine, compiles the WASM, then builds the web app.

To deploy manually:

```bash
bash scripts/build.sh
netlify deploy --prod --dir web/dist
```

---

## Tech Stack

| Layer          | Technology               |
| -------------- | ------------------------ |
| UI             | React 19 + TypeScript    |
| Build tool     | Vite                     |
| Core logic     | Rust (stable)            |
| PDF rendering  | pdf-writer crate         |
| Image decoding | image crate              |
| WASM bridge    | wasm-bindgen + wasm-pack |
| Persistence    | localStorage             |
| Hosting        | Netlify                  |
