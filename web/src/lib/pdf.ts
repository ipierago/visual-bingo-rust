import type { ImageItem } from './images'
import type { BingoCard } from './bingo'
import { initWasm, generate_pdf_wasm } from './wasm'

export interface PdfProgress {
  current: number
  total: number
  phase: 'loading' | 'cards' | 'calllist'
}

export interface GenerateOptions {
  cards: BingoCard[]
  callList: ImageItem[]
  onProgress?: (p: PdfProgress) => void
  signal?: AbortSignal
}

async function fetchImageBytes(url: string): Promise<Uint8Array> {
  const res = await fetch(url)
  const buf = await res.arrayBuffer()
  return new Uint8Array(buf)
}

export async function generatePdf(options: GenerateOptions): Promise<Uint8Array> {
  const { cards, callList, onProgress, signal } = options

  await initWasm()

  // Collect unique images
  const allItems = [...cards.flatMap((c) => c.cells), ...callList]
  const uniqueItems = [...new Map(allItems.map((i) => [i.id, i])).values()]

  // Fetch all image bytes
  const imageDataList = []
  for (let i = 0; i < uniqueItems.length; i++) {
    if (signal?.aborted) throw new DOMException('Aborted', 'AbortError')
    const item = uniqueItems[i]
    const bytes = await fetchImageBytes(item.url)
    imageDataList.push({
      id: item.id,
      bytes: Array.from(bytes), // JSON serializable
      is_png: item.url.toLowerCase().endsWith('.png'),
    })
    onProgress?.({
      current: i + 1,
      total: uniqueItems.length,
      phase: 'loading',
    })
    await yieldToMain()
  }

  if (signal?.aborted) throw new DOMException('Aborted', 'AbortError')

  // Build request
  const request = {
    images: uniqueItems,
    seed: 'unused', // cards already generated, just need for call list order
    card_count: cards.length,
  }

  onProgress?.({ current: 0, total: cards.length, phase: 'cards' })
  await yieldToMain()

  // Generate PDF in Rust
  const t0 = performance.now()
  const pdfBytes = generate_pdf_wasm(JSON.stringify(request), JSON.stringify(imageDataList))
  console.log(`[timing] PDF render (Rust): ${Math.round(performance.now() - t0)}ms`)

  onProgress?.({ current: cards.length, total: cards.length, phase: 'calllist' })

  return pdfBytes
}

function yieldToMain(): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, 0))
}
