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

interface ImageEntry {
  id: string
  bytes: Uint8Array
  is_png: boolean
}

async function fetchImageBytes(url: string): Promise<Uint8Array> {
  const res = await fetch(url)
  const buf = await res.arrayBuffer()
  return new Uint8Array(buf)
}

function yieldToMain(): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, 0))
}

export async function generatePdf(options: GenerateOptions): Promise<Uint8Array> {
  const { cards, callList, onProgress, signal } = options

  await initWasm()

  // Collect unique images
  const allItems = [...cards.flatMap((c) => c.cells), ...callList]
  const uniqueItems = [...new Map(allItems.map((i) => [i.id, i])).values()]

  // Fetch all images in parallel, keep as Uint8Array
  const imageDataList: ImageEntry[] = await Promise.all(
    uniqueItems.map(async (item) => {
      const bytes = await fetchImageBytes(item.url)
      return {
        id: item.id,
        bytes,
        is_png: item.url.toLowerCase().endsWith('.png'),
      }
    }),
  )
  onProgress?.({ current: uniqueItems.length, total: uniqueItems.length, phase: 'loading' })

  if (signal?.aborted) throw new DOMException('Aborted', 'AbortError')
  await yieldToMain()

  // Build flat binary buffer — no Array.from, no JSON for image bytes
  const offsets: number[] = []
  const lengths: number[] = []
  let totalBytes = 0
  for (const item of imageDataList) {
    offsets.push(totalBytes)
    lengths.push(item.bytes.length)
    totalBytes += item.bytes.length
  }

  const flatBuffer = new Uint8Array(totalBytes)
  for (let i = 0; i < imageDataList.length; i++) {
    flatBuffer.set(imageDataList[i].bytes, offsets[i])
  }

  const ids = imageDataList.map((d) => d.id)
  const isPng = imageDataList.map((d) => d.is_png)

  const request = {
    images: uniqueItems,
    seed: 'unused',
    card_count: cards.length,
  }

  onProgress?.({ current: 0, total: cards.length, phase: 'cards' })
  await yieldToMain()

  const t0 = performance.now()

  const pdfBytes = generate_pdf_wasm(
    JSON.stringify(request),
    JSON.stringify(ids),
    JSON.stringify(isPng),
    flatBuffer,
    new Uint32Array(offsets),
    new Uint32Array(lengths),
  )

  console.log(`[timing] PDF render (Rust): ${Math.round(performance.now() - t0)}ms`)

  onProgress?.({ current: cards.length, total: cards.length, phase: 'calllist' })

  return pdfBytes
}
