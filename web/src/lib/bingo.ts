import type { ImageItem } from './images'
import { initWasm, generate_cards } from './wasm'

export interface BingoCard {
  cells: ImageItem[]
}

export interface GenerateResponse {
  cards: BingoCard[]
  call_list: ImageItem[]
}

export async function generateCards(
  images: ImageItem[],
  seed: string,
  cardCount: number,
): Promise<GenerateResponse> {
  await initWasm()

  const request = {
    images,
    seed,
    card_count: cardCount,
  }

  const responseJson = generate_cards(JSON.stringify(request))
  return JSON.parse(responseJson) as GenerateResponse
}
