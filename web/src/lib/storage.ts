const KEYS = {
  selectedIds: 'bingo:selectedIds',
  activeSet: 'bingo:activeSet',
  seed: 'bingo:seed',
  cardCount: 'bingo:cardCount',
} as const

export interface PersistedState {
  selectedIds: string[]
  activeSet: string
  seed: string
  cardCount: number
}

export function saveState(state: PersistedState): void {
  try {
    localStorage.setItem(KEYS.selectedIds, JSON.stringify(state.selectedIds))
    localStorage.setItem(KEYS.activeSet, state.activeSet)
    localStorage.setItem(KEYS.seed, state.seed)
    localStorage.setItem(KEYS.cardCount, String(state.cardCount))
  } catch {
    // localStorage unavailable or full — fail silently
  }
}

export function loadState(): Partial<PersistedState> {
  try {
    const selectedIds = localStorage.getItem(KEYS.selectedIds)
    const activeSet = localStorage.getItem(KEYS.activeSet)
    const seed = localStorage.getItem(KEYS.seed)
    const cardCount = localStorage.getItem(KEYS.cardCount)

    return {
      selectedIds: selectedIds ? JSON.parse(selectedIds) : undefined,
      activeSet: activeSet ?? undefined,
      seed: seed ?? undefined,
      cardCount: cardCount ? Number(cardCount) : undefined,
    }
  } catch {
    return {}
  }
}

export function clearState(): void {
  try {
    Object.values(KEYS).forEach((k) => localStorage.removeItem(k))
  } catch {
    // fail silently
  }
}
