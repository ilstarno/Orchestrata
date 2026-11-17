const element = (tag, text) => {
  const node = document.createElement(tag)
  if (text != null) node.textContent = text
  return node
}

export function columnCaption(sheet, index) {
  const heading = sheet.headers[index] || `Column ${index + 1} (no heading)`
  const sample = sheet.rows.find(row => row[index] != null && String(row[index]).trim() !== '')?.[index]
  return sample == null ? heading : `${heading} — ${sample}`
}

export function renderSourcePreview(container, workbook, selected, acceptValue) {
  const editors = new Map()
  if (!container) return editors
  container.replaceChildren()
  const inspected = [...new Set(selected.filter(Number.isInteger))]
  const indices = inspected.length === 2 ? inspected : [
    ...inspected, ...workbook.sheets.map((_, i) => i).filter(i => !inspected.includes(i)),
  ]
  for (const [position, index] of indices.entries()) {
    const sheet = workbook.sheets[index]
    if (!sheet) continue
    const section = element('details')
    section.className = 'source-sheet'
    section.open = position < 2
    const count = sheet.rows.length
    section.append(element('summary', `${sheet.name} · ${count} source ${count === 1 ? 'row' : 'rows'}`))
    const scroll = element('div')
    scroll.className = 'source-scroll'
    scroll.tabIndex = 0
    scroll.setAttribute('role', 'region')
    scroll.setAttribute('aria-label', `${sheet.name} source records`)
    const table = element('table')
    const head = element('thead')
    const titles = element('tr')
    for (const [col, heading] of ['Source row', ...sheet.headers].entries()) {
      const cell = element('th', heading || `Column ${col}`)
      cell.scope = 'col'
      titles.append(cell)
    }
    head.append(titles)
    const body = element('tbody')
    function renderRows(full) {
      body.replaceChildren()
      for (const [row, cells] of sheet.rows.entries()) {
        if (!full && row >= 3) break
        const line = element('tr')
        const source = element('th', String(sheet.rowNumbers?.[row] ?? row + 2))
        source.scope = 'row'
        line.append(source)
        for (let column = 0; column < sheet.headers.length; column++) {
          const value = cells[column]
          line.append(element('td', value == null || value === '' ? '—' : String(value)))
        }
        body.append(line)
      }
    }
    renderRows(false)
    table.append(head, body)
    scroll.append(table)
    section.append(scroll)
    if (count > 3) {
      const expand = element('button', `Show all ${count} source rows`)
      expand.type = 'button'
      expand.setAttribute('aria-expanded', 'false')
      let full = false
      expand.addEventListener('click', () => {
        full = !full
        renderRows(full)
        expand.textContent = full ? 'Show first 3 source rows' : `Show all ${count} source rows`
        expand.setAttribute('aria-expanded', String(full))
      })
      section.append(expand)
    }
    if (acceptValue && count && sheet.headers.length) {
      const edit = element('details')
      const rowLabel = element('label', 'Source row')
      const columnLabel = element('label', 'Source column')
      const valueLabel = element('label', 'Corrected value')
      const rowPick = element('select')
      const colPick = element('select')
      const value = element('input')
      value.type = 'text'
      function addOption(select, label, code) {
        const option = element('option', label)
        option.value = String(code)
        select.append(option)
      }
      for (let row = 0; row < count; row++) addOption(rowPick, String(sheet.rowNumbers?.[row] ?? row + 2), row)
      for (let col = 0; col < sheet.headers.length; col++) addOption(colPick, sheet.headers[col] || `Column ${col + 1}`, col)
      rowPick.value = '0'
      colPick.value = '0'
      function loadValue() {
        value.value = String(sheet.rows[Number(rowPick.value)]?.[Number(colPick.value)] ?? '')
      }
      rowPick.addEventListener('change', loadValue)
      colPick.addEventListener('change', loadValue)
      loadValue()
      rowLabel.append(rowPick)
      columnLabel.append(colPick)
      valueLabel.append(value)
      const fields = element('div')
      fields.className = 'source-editor'
      const use = element('button', 'Use this value')
      use.type = 'button'
      use.addEventListener('click', () => {
        const indexInRows = Number(rowPick.value)
        const col = Number(colPick.value)
        const rowNumber = sheet.rowNumbers?.[indexInRows] ?? indexInRows + 2
        acceptValue(index, rowNumber, col, value.value)
      })
      fields.append(rowLabel, columnLabel, valueLabel, use)
      edit.append(element('summary', 'Edit a source value'), fields)
      section.append(edit)
      editors.set(index, (sourceRow, column) => {
        const row = sheet.rowNumbers?.indexOf(sourceRow) ?? sourceRow - 2
        if (row < 0 || row >= count || column < 0 || column >= sheet.headers.length) return
        section.open = true
        edit.open = true
        rowPick.value = String(row)
        colPick.value = String(column)
        loadValue()
        value.focus()
      })
    }
    container.append(section)
  }
  return editors
}
