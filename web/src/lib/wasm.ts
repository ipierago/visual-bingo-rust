import init, { generate_cards, generate_pdf_wasm } from '../wasm/bingo_wasm'
import wasmUrl from '../wasm/bingo_wasm_bg.wasm?url'

let initialised = false

export async function initWasm(): Promise<void> {
  if (initialised) return
  await init(wasmUrl)
  initialised = true
}

export { generate_cards, generate_pdf_wasm }
