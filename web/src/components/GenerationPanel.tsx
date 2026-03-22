import type { PdfProgress } from '../lib/pdf'

interface Props {
  totalSelected: number
  seed: string
  cardCount: number
  onSeedChange: (seed: string) => void
  onRandomSeed: () => void
  onCardCountChange: (count: number) => void
  onGenerate: () => void
  progress: PdfProgress | null
  onCancel: () => void
}

const CELLS_PER_CARD = 25
const MIN_IMAGES = 25

function calcProbability(selected: number): string {
  if (selected < MIN_IMAGES) return '—'
  const p = Math.min(1, CELLS_PER_CARD / selected)
  return (p * 100).toFixed(1) + '%'
}

export default function GenerationPanel({
  totalSelected,
  seed,
  cardCount,
  onSeedChange,
  onRandomSeed,
  onCardCountChange,
  onGenerate,
  progress,
  onCancel,
}: Props) {
  const canGenerate = totalSelected >= MIN_IMAGES && !progress
  const pct = progress
    ? Math.round((progress.current / progress.total) * 100)
    : 0

  return (
    <div className="gen-panel">
      <div className="gen-stats">
        <Stat label="Selected" value={totalSelected} />
        <Stat label="Per card" value={CELLS_PER_CARD} />
        <Stat label="Probability" value={calcProbability(totalSelected)} />
      </div>

      <div className="gen-controls">
        <div className="gen-row">
          <label>Seed</label>
          <div className="seed-input-wrap">
            <input
              type="text"
              value={seed}
              onChange={(e) => onSeedChange(e.target.value)}
              className="seed-input"
              disabled={!!progress}
            />
            <button onClick={onRandomSeed} disabled={!!progress}>
              🎲
            </button>
          </div>
        </div>

        <div className="gen-row">
          <label>Cards</label>
          <div className="card-count-wrap">
            <button
              onClick={() => onCardCountChange(Math.max(1, cardCount - 1))}
              disabled={!!progress}
            >
              −
            </button>
            <span>{cardCount}</span>
            <button
              onClick={() => onCardCountChange(Math.min(50, cardCount + 1))}
              disabled={!!progress}
            >
              +
            </button>
          </div>
        </div>

        <div className="gen-action">
          {progress ? (
            <div className="gen-progress">
              <div className="progress-bar">
                <div className="progress-fill" style={{ width: `${pct}%` }} />
              </div>
              <span className="progress-label">
                {progress.phase === 'loading'
                  ? `Loading ${progress.current} / ${progress.total}`
                  : progress.phase === 'cards'
                    ? `Card ${progress.current} / ${progress.total}`
                    : 'Writing call list…'}
              </span>
              <button className="cancel-btn" onClick={onCancel}>
                Cancel
              </button>
            </div>
          ) : (
            <div className="gen-progress">
              <button
                className="generate-btn"
                onClick={onGenerate}
                disabled={!canGenerate}
                title={
                  !canGenerate ? `Select at least ${MIN_IMAGES} images` : ''
                }
              >
                Generate PDF
              </button>
              {!canGenerate && (
                <p className="gen-warning">
                  Need {MIN_IMAGES - totalSelected} more images
                </p>
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  )
}

function Stat({ label, value }: { label: string; value: string | number }) {
  return (
    <div className="stat">
      <span className="stat-value">{value}</span>
      <span className="stat-label">{label}</span>
    </div>
  )
}
