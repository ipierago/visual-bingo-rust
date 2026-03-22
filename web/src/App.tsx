import { useState, useRef, useMemo } from 'react'
import { IMAGE_SETS, ALL_IMAGES, type ImageSet } from './lib/images'
import SetPanel from './components/SetPanel'
import ThumbnailPanel from './components/ThumbnailPanel'
import GenerationPanel from './components/GenerationPanel'
import { generateSeed } from './lib/seed'
import { generateCards } from './lib/bingo'
import { generatePdf, type PdfProgress } from './lib/pdf'
import { downloadPdf } from './lib/download'
import { loadState, clearState } from './lib/storage'
import { usePersistence } from './hooks/usePersistence'

function initState() {
  const saved = loadState()
  const seed = saved.seed ?? generateSeed()
  const cardCount = saved.cardCount ?? 1
  const activeSet = saved.activeSet ?? IMAGE_SETS[0]?.name ?? ''

  const validIds = new Set(ALL_IMAGES.map((img) => img.id))
  const selectedIds = new Set((saved.selectedIds ?? []).filter((id) => validIds.has(id)))

  return { seed, cardCount, activeSet, selectedIds }
}

const initial = initState()

export default function App() {
  const [selectedIds, setSelectedIds] = useState<Set<string>>(initial.selectedIds)
  const [activeSet, setActiveSet] = useState<string>(initial.activeSet)
  const [seed, setSeed] = useState<string>(initial.seed)
  const [cardCount, setCardCount] = useState<number>(initial.cardCount)
  const [progress, setProgress] = useState<PdfProgress | null>(null)
  const abortRef = useRef<AbortController | null>(null)

  const persistedState = useMemo(
    () => ({
      selectedIds: [...selectedIds],
      activeSet,
      seed,
      cardCount,
    }),
    [selectedIds, activeSet, seed, cardCount],
  )

  usePersistence(persistedState)

  const toggleSet = (set: ImageSet) => {
    setSelectedIds((prev) => {
      const next = new Set(prev)
      const allSelected = set.images.every((img) => next.has(img.id))
      set.images.forEach((img) => (allSelected ? next.delete(img.id) : next.add(img.id)))
      return next
    })
  }

  const toggleImage = (id: string) => {
    setSelectedIds((prev) => {
      const next = new Set(prev)
      if (next.has(id)) {
        next.delete(id)
      } else {
        next.add(id)
      }
      return next
    })
  }

  const selectAll = () => setSelectedIds(new Set(ALL_IMAGES.map((img) => img.id)))

  const clearAll = () => setSelectedIds(new Set())

  const currentSet = IMAGE_SETS.find((s) => s.name === activeSet)

  const handleGenerate = async () => {
    const selected = ALL_IMAGES.filter((img) => selectedIds.has(img.id))

    const abort = new AbortController()
    abortRef.current = abort
    setProgress({ current: 0, total: 1, phase: 'loading' })

    try {
      const { cards, call_list } = await generateCards(selected, seed, cardCount)

      const bytes = await generatePdf({
        cards,
        callList: call_list,
        signal: abort.signal,
        onProgress: setProgress,
      })

      downloadPdf(bytes, `bingo-${seed}.pdf`)
    } catch (e) {
      if (e instanceof DOMException && e.name === 'AbortError') {
        console.log('Cancelled')
      } else {
        console.error(e)
      }
    } finally {
      setProgress(null)
      abortRef.current = null
    }
  }

  const handleCancel = () => {
    abortRef.current?.abort()
  }

  const handleReset = () => {
    clearState()
    window.location.reload()
  }

  return (
    <div style={{ display: 'flex', height: '100vh' }}>
      <SetPanel
        sets={IMAGE_SETS}
        selectedIds={selectedIds}
        activeSet={activeSet}
        onSetClick={setActiveSet}
        onSetToggle={toggleSet}
        onSelectAll={selectAll}
        onClearAll={clearAll}
        onReset={handleReset}
      />
      <div style={{ flex: 1, display: 'flex', flexDirection: 'column', overflow: 'hidden' }}>
        <ThumbnailPanel
          set={currentSet}
          selectedIds={selectedIds}
          onToggleImage={toggleImage}
          onSelectSet={() => currentSet && toggleSet(currentSet)}
          onClearSet={() => {
            setSelectedIds((prev) => {
              const next = new Set(prev)
              currentSet?.images.forEach((img) => next.delete(img.id))
              return next
            })
          }}
        />
        <GenerationPanel
          totalSelected={selectedIds.size}
          seed={seed}
          cardCount={cardCount}
          onSeedChange={setSeed}
          onRandomSeed={() => setSeed(generateSeed())}
          onCardCountChange={setCardCount}
          onGenerate={handleGenerate}
          progress={progress}
          onCancel={handleCancel}
        />
      </div>
    </div>
  )
}
