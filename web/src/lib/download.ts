export function downloadPdf(bytes: Uint8Array, filename = 'bingo.pdf') {
  const blob = new Blob([bytes.buffer as ArrayBuffer], {
    type: 'application/pdf',
  })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = filename
  a.click()
  URL.revokeObjectURL(url)
}
