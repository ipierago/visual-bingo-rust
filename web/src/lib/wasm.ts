import init, { generate_cards, generate_pdf_wasm } from '../wasm/bingo_wasm'

let initialised = false

export async function initWasm(): Promise<void> {
  if (initialised) return
  // fetch the wasm binary and pass it directly
  const wasmModule = await fetch(new URL('../wasm/bingo_wasm_bg.wasm', import.meta.url))
  await init(wasmModule)
  initialised = true
}

export { generate_cards, generate_pdf_wasm }
