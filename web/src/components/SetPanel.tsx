import { type ImageSet } from '../lib/images'

type SelectionState = 'none' | 'partial' | 'full'

interface Props {
  sets: ImageSet[]
  selectedIds: Set<string>
  activeSet: string
  onSetClick: (name: string) => void
  onSetToggle: (set: ImageSet) => void
  onSelectAll: () => void
  onClearAll: () => void
  onReset: () => void
}

function getSelectionState(
  set: ImageSet,
  selectedIds: Set<string>,
): SelectionState {
  const count = set.images.filter((img) => selectedIds.has(img.id)).length
  if (count === 0) return 'none'
  if (count === set.images.length) return 'full'
  return 'partial'
}

export default function SetPanel({
  sets,
  selectedIds,
  activeSet,
  onSetClick,
  onSetToggle,
  onSelectAll,
  onClearAll,
  onReset,
}: Props) {
  return (
    <aside className="set-panel">
      <div className="set-panel-toolbar">
        <button onClick={onSelectAll}>Select All</button>
        <button onClick={onClearAll}>Clear All</button>
        <button onClick={onReset}>Reset</button>
      </div>

      <div className="set-list">
        {sets.map((set) => {
          const state = getSelectionState(set, selectedIds)
          const selectedCount = set.images.filter((img) =>
            selectedIds.has(img.id),
          ).length
          const isActive = set.name === activeSet

          return (
            <div
              key={set.name}
              className={[
                'set-card',
                isActive ? 'set-card--active' : '',
                `set-card--${state}`,
              ].join(' ')}
              onClick={() => onSetClick(set.name)}
            >
              <input
                type="checkbox"
                checked={state === 'full'}
                ref={(el) => {
                  if (el) el.indeterminate = state === 'partial'
                }}
                onChange={() => onSetToggle(set)}
                onClick={(e) => e.stopPropagation()}
              />
              <div className="set-card-info">
                <span className="set-card-name">{set.name}</span>
                <span className="set-card-count">
                  {selectedCount} / {set.images.length}
                </span>
              </div>
            </div>
          )
        })}
      </div>
    </aside>
  )
}
