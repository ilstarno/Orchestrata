const workbookLimit = 2 * 1024 * 1024
const hasFiles = transfer => Boolean(transfer && Array.from(transfer.types ?? []).includes('Files'))

function sizeText(size) {
  return size < 1024 ? `${size} B` : `${new Intl.NumberFormat(document.documentElement.lang, {
    maximumFractionDigits: 0,
  }).format(size / 1024)} KB`
}

export function wireFileSlots(root) {
  const slots = [...root.querySelectorAll('[data-file-slot]')].flatMap(slot => {
    const input = slot.querySelector('input[type="file"]')
    const receipt = slot.querySelector('[data-file-receipt]')
    const remove = slot.querySelector('[data-remove-file]')
    return input && receipt && remove ? [{ slot, input, receipt, remove }] : []
  })
  function sync() {
    for (const { input, receipt, remove } of slots) {
      const file = input.files?.[0]
      remove.hidden = !file
      receipt.textContent = file ? `${file.name} (${sizeText(file.size)})` : ''
    }
  }
  for (const { slot, input, receipt, remove } of slots) {
    const excel = input.id === 'sheet-workbook'
    const accepted = excel ? /\.xlsx$/iu : /\.csv$/iu
    const picked = () => input.dispatchEvent(new Event('change', { bubbles: true }))
    function refuse(message) {
      const current = input.files?.[0]
      receipt.textContent = current ? `${message} Still selected: ${current.name}.` : message
    }
    remove.addEventListener('click', () => {
      input.value = ''
      picked()
      input.focus()
    })
    slot.addEventListener('dragover', event => {
      if (!hasFiles(event.dataTransfer)) return
      event.preventDefault()
      slot.setAttribute('data-drop-ready', '')
      event.dataTransfer.dropEffect = 'copy'
    })
    slot.addEventListener('dragleave', event => {
      if (!event.relatedTarget?.nodeType || !slot.contains(event.relatedTarget)) {
        slot.removeAttribute('data-drop-ready')
      }
    })
    slot.addEventListener('drop', event => {
      slot.removeAttribute('data-drop-ready')
      if (!hasFiles(event.dataTransfer)) return
      event.preventDefault()
      const [file] = event.dataTransfer.files
      if (event.dataTransfer.files.length !== 1 || !file || !accepted.test(file.name)) {
        refuse(excel ? 'Drop one .xlsx workbook here.' : 'Drop one .csv file here.')
        return
      }
      if (file.size > workbookLimit) {
        refuse('That file exceeds 2 MB. Choose a smaller file.')
        return
      }
      try {
        const next = new DataTransfer()
        next.items.add(file)
        input.files = next.files
      } catch {
        refuse('This browser could not add the dropped file. Use the file chooser instead.')
        return
      }
      picked()
    })
  }
  // Run after the importer has finished clearing any conflicting file selections.
  root.addEventListener('change', () => queueMicrotask(sync))
  return { sync }
}

const slotsRoot = globalThis.document?.getElementById('spreadsheet-import')
if (slotsRoot) wireFileSlots(slotsRoot)
