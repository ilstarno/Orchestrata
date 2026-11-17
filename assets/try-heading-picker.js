// Recovery for title rows, separate from ordinary column mapping.
const element = (name, text) => {
  const node = document.createElement(name)
  if (text != null) node.textContent = text
  return node
}

export function headingPicker(inspection, show, select) {
  const panel = element('details')
  panel.className = 'mapping-help heading-picker'
  panel.hidden = !show || !inspection?.ok || inspection.rows.length < 2
  const summary = element('summary', 'Headings start on a different row?')
  panel.append(summary)
  if (panel.hidden) return { panel, summary, blocking: false }

  panel.append(element('p', 'Choose the row that contains the column names. Only choose a later row if the rows above it are not data you need.'))
  const choices = element('ol')
  for (const row of inspection.rows) {
    const item = element('li')
    const values = row.cells.filter(value => value != null && String(value).trim() !== '')
    const sample = values.slice(0, 4).map(String).join(' · ')
    const remainder = values.length > 4 ? ` · ${values.length - 4} more values` : ''
    item.append(element('strong', `Source row ${row.row}`),
      element('p', sample ? sample + remainder : 'Blank row'))
    if (row.strong) item.append(element('small', 'These values look like column names.'))
    item.append(element('small', row.skippedNonblankCount
      ? `Skips ${row.skippedNonblankCount} nonblank ${row.skippedNonblankCount === 1 ? 'row' : 'rows'} above it.`
      : 'Skips no nonblank rows.'))
    const button = element('button', `Use row ${row.row} as headings`)
    button.type = 'button'
    button.disabled = !values.length
    button.addEventListener('click', () => select(row.row))
    item.append(button)
    choices.append(item)
  }
  panel.append(choices)
  return { panel, summary, blocking: inspection.needsReview && !inspection.confirmed }
}
