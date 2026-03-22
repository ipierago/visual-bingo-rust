import type { ImageItem, ImageSet } from '../lib/images'

interface Props {
  set: ImageSet | undefined
  selectedIds: Set<string>
  onToggleImage: (id: string) => void
  onSelectSet: () => void
  onClearSet: () => void
}

export default function ThumbnailPanel({
  set,
  selectedIds,
  onToggleImage,
  onSelectSet,
  onClearSet,
}: Props) {
  if (!set) return <div className="thumbnail-panel">No set selected</div>

  return (
    <div className="thumbnail-panel">
      <div className="thumbnail-toolbar">
        <span className="thumbnail-set-name">{set.name}</span>
        <button onClick={onSelectSet}>Select All</button>
        <button onClick={onClearSet}>Clear</button>
      </div>

      <div className="thumbnail-grid">
        {set.images.map((img) => (
          <ImageTile
            key={img.id}
            image={img}
            selected={selectedIds.has(img.id)}
            onToggle={() => onToggleImage(img.id)}
          />
        ))}
      </div>
    </div>
  )
}

interface TileProps {
  image: ImageItem
  selected: boolean
  onToggle: () => void
}

function ImageTile({ image, selected, onToggle }: TileProps) {
  return (
    <div
      className={['tile', selected ? 'tile--selected' : ''].join(' ')}
      onClick={onToggle}
    >
      <div className="tile-img-wrap">
        <img src={image.url} alt={image.label} loading="lazy" />
      </div>
      <span className="tile-label">{image.label}</span>
    </div>
  )
}
