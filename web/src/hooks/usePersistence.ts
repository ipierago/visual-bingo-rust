import { useEffect } from 'react'
import { saveState, type PersistedState } from '../lib/storage'

export function usePersistence(state: PersistedState) {
  useEffect(() => {
    const timer = setTimeout(() => saveState(state), 300)
    return () => clearTimeout(timer)
  }, [state])
}
