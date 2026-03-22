export interface ImageItem {
  id: string // unique key, e.g. "animals/cat"
  set: string // "animals"
  label: string // "cat" (derived from filename)
  url: string // resolved URL Vite gives us
}

export interface ImageSet {
  name: string
  images: ImageItem[]
}

// Convert filename to display label
// "fire-truck.jpg" → "fire truck"
// "ice_cream.png"  → "ice cream"
function fileToLabel(filename: string): string {
  return filename
    .replace(/\.[^.]+$/, '') // strip extension
    .replace(/[-_]/g, ' ') // replace - and _ with space
}

// Parse all images from the glob result into sets
function loadImageSets(): ImageSet[] {
  const modules = import.meta.glob('../assets/images/vocabulary/**/*.{jpg,jpeg,png}', {
    eager: true,
  }) as Record<string, { default: string }>

  const setMap = new Map<string, ImageItem[]>()

  for (const [path, mod] of Object.entries(modules)) {
    // path: "../assets/images/vocabulary/animals/cat.jpg"
    const parts = path.split('/')
    // parts: ["..", "assets", "images", "vocabulary", "animals", "cat.jpg"]

    const setName = parts.at(-2)!
    const filename = parts.at(-1)!
    const label = fileToLabel(filename)
    const id = `${setName}/${label}`

    if (!setMap.has(setName)) {
      setMap.set(setName, [])
    }

    setMap.get(setName)!.push({
      id,
      set: setName,
      label,
      url: mod.default,
    })
  }

  // Sort sets alphabetically, images within each set alphabetically
  return Array.from(setMap.entries())
    .sort(([a], [b]) => a.localeCompare(b))
    .map(([name, images]) => ({
      name,
      images: images.sort((a, b) => a.label.localeCompare(b.label)),
    }))
}

// Call once — result is stable for the lifetime of the app
export const IMAGE_SETS: ImageSet[] = loadImageSets()

// Flat list of all images across all sets
export const ALL_IMAGES: ImageItem[] = IMAGE_SETS.flatMap((s) => s.images)
