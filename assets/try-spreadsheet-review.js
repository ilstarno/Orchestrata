import { fieldsByKind } from './try-sheet-mapper.js'
import { headingPicker } from './try-heading-picker.js'
import { columnCaption, renderSourcePreview } from './try-source-preview.js'

const issuesByCode = {
  'heading-review': 'Choose which row contains the column names before calculating.',
  mapping: 'Choose separate job and people sheets and one different column for each field.',
  'invalid-file': 'These rows could not be checked. Try reopening the spreadsheet.',
  'input-limit': 'The selected columns are too large. Use a smaller workbook.',
  'unmapped-row': 'This row has data, but the selected columns are empty. Check the mapping or remove the row if it is only a note.',
  'record-limit': 'Use no more than 200 jobs and people combined.',
  'no-jobs': 'Add at least one job.',
  'column-count': 'A row is missing one or more selected columns.',
  'invalid-quote': 'A quoted cell could not be read.',
  'invalid-skills': 'Use up to 20 comma-separated skills.',
  'invalid-available': 'Use Yes or No for availability.',
  'invalid-capacity': 'Use a whole number from 0 to 200 for the job limit.',
}

function issueText(issue, sourceName) {
  const detail = issue.code === 'invalid-text'
    ? (issue.field === 'skill' ? 'Enter one required skill.' : 'Enter a name.')
    : issuesByCode[issue.code] ?? 'This row could not be read.'
  const range = issue.range === 'people' ? 'People' : 'Jobs'
  const label = sourceName || range
  if (Number.isInteger(issue.row)) return `${label}, source row ${issue.row}: ${detail}`
  return sourceName ? `${label}: ${detail}` : detail
}

const element = (tag, text) => {
  const node = document.createElement(tag)
  if (text != null) node.textContent = text
  return node
}

function labeledSelect(label, items, current) {
  const root = element('label', label)
  const select = element('select')
  const none = element('option', 'Choose…')
  none.value = ''
  select.append(none)
  for (const [index, item] of items.entries()) {
    const option = element('option', item || `Column ${index + 1}`)
    option.value = String(index)
    select.append(option)
  }
  select.value = current === null ? '' : String(current)
  root.append(select)
  return { root, select }
}

const indexOf = select => select.value === '' ? null : Number(select.value)

// The file reader never gets editor state or a server destination.
export function wireSpreadsheetReview(root, { readFiles, suggestColumnMapping, suggestSheetColumns, inspectSpreadsheetHeadingRows, selectSpreadsheetHeadingRow, updateSpreadsheetCell, canUpdateSpreadsheetCells, mapSpreadsheetWorkbook, replaceSetup }) {
  const find = id => root.querySelector('#' + id)
  const load = find('sheet-load')
  const workbook = find('sheet-workbook')
  const jobsFile = find('sheet-jobs-csv')
  const peopleFile = find('sheet-people-csv')
  const separator = find('sheet-separator')
  const choices = find('sheet-separator-choices')
  const mapper = find('sheet-mapper')
  const sourcePreview = find('sheet-source-preview')
  const discard = find('sheet-discard')
  const check = find('sheet-check')
  const progress = find('sheet-progress')
  const apply = find('sheet-apply')
  const status = find('sheet-status')
  const rules = find('sheet-issues')
  if (![load, workbook, jobsFile, peopleFile, mapper, check, progress, apply, status, rules].every(Boolean)) return null
  let generation = 0
  let data = null
  let groups = null
  let sourceEditors = new Map()
  let candidate = null
  let sourceName = null
  let csvSources = []
  let separatorsByFile = new WeakMap()
  let ambiguousFiles = new WeakSet()

  function clearPreview() {
    candidate = null
    apply.disabled = true
    status.textContent = ''
    rules.replaceChildren()
  }

  function clearFiles() {
    generation++
    data = null
    sourceName = null
    csvSources = []
    groups = null
    load.disabled = false
    if (separator) {
      separator.hidden = true
      separator.replaceChildren()
    }
    if (choices) {
      choices.hidden = true
      choices.replaceChildren()
    }
    mapper.hidden = true
    sourcePreview?.replaceChildren()
    sourceEditors.clear()
    find('sheet-map-jobs').replaceChildren()
    find('sheet-map-people').replaceChildren()
    clearPreview()
  }

  load.hidden = true
  for (const input of [workbook, jobsFile, peopleFile]) {
    input.addEventListener('change', () => {
      clearFiles()
      if (input.files.length) {
        if (input === workbook) {
          jobsFile.value = ''
          peopleFile.value = ''
        } else workbook.value = ''
      }
      if (workbook.files[0] || (jobsFile.files[0] && peopleFile.files[0])) openSpreadsheet()
      else if (jobsFile.files[0] || peopleFile.files[0]) status.textContent = 'Choose the second CSV to continue.'
    })
  }

  function makeGroup(kind, container, mapping) {
    const task = kind === 'jobs' ? 'jobs' : 'people'
    const specs = fieldsByKind[kind]
    const names = data.sheets.map(sheet => sheet.name)
    const { root: sheetLabel, select: sheetSelect } = labeledSelect(`Which sheet has the ${task}?`, names, mapping.sheet)
    const sheetSource = element('p')
    sheetSource.className = 'mapping-sheet'
    const queue = element('div')
    queue.className = 'mapping-queue'
    const edit = element('details')
    edit.className = 'mapping-edit'
    const columns = element('div')
    const recoverySlot = element('div')
    const toggle = element('summary', `Review or change ${task} matches`)
    edit.append(toggle, columns)
    const pickSheet = element('div')
    let state = {}
    let draft = new Set()
    let fieldControls = {}
    let recovery = null

    function value() {
      return { sheet: indexOf(sheetSelect), ...state }
    }

    function firstUnresolved() {
      if (indexOf(sheetSelect) === null) return sheetSelect
      const missing = specs.find(spec => state[spec.key] === null)
      return missing ? fieldControls[missing.key] : null
    }

    function focus(control) {
      if (!control) return false
      if (control === sheetSelect && indexOf(sheetSelect) !== null) edit.open = true
      control.focus()
      return true
    }

    function updateColumns() {
      const sheetIndex = indexOf(sheetSelect)
      const sheet = data.sheets[sheetIndex]
      const inspected = sheet && inspectSpreadsheetHeadingRows?.(data, sheetIndex, kind)
      const selected = inspected?.ok ? inspected.rows.find(row => row.row === inspected.currentRow) : null
      const note = selected && selected.row > 1
        ? ` · Headings in row ${selected.row} · ${selected.skippedNonblankCount} nonblank ${selected.skippedNonblankCount === 1 ? 'row' : 'rows'} skipped`
        : ''
      sheetSource.textContent = sheet ? `${sheet.name}${note}` : ''
      queue.replaceChildren()
      columns.replaceChildren()
      pickSheet.replaceChildren()
      fieldControls = {}
      if (!sheet) pickSheet.append(sheetLabel)
      else columns.append(sheetLabel)
      if (sheet) {
        const missing = specs.filter(spec => state[spec.key] === null)
        if (missing.length) queue.append(element('p', missing.length === 1 ? 'One match to choose' : `${missing.length} matches to choose`))
        for (const spec of specs) {
          const current = state[spec.key]
          const target = current === null ? queue : columns
          const row = element('div')
          row.className = 'mapping-decision'
          const captions = sheet.headers.map((_, index) => columnCaption(sheet, index))
          const { root: label, select } = labeledSelect(spec.label, captions, current)
          for (const [index, option] of [...select.children].slice(1).entries()) {
            option.disabled = specs.some(other => other.key !== spec.key && state[other.key] === index)
          }
          const confirm = element('button', `Use as ${spec.label.toLowerCase()}`)
          confirm.type = 'button'
          confirm.disabled = true
          const example = element('p', current === null ? 'Choose a source column.' : captions[current])
          example.className = 'mapping-sheet'
          select.addEventListener('change', () => {
            const choice = indexOf(select)
            confirm.disabled = choice === current
            example.textContent = choice === null ? 'No source column selected.' : captions[choice]
            if (choice === current) draft.delete(spec.key)
            else draft.add(spec.key)
            previewMatches('Use the selected column before calculating.')
          })
          confirm.addEventListener('click', () => {
            const choice = indexOf(select)
            if (choice !== null && specs.some(other => other.key !== spec.key && state[other.key] === choice)) return
            state[spec.key] = choice
            draft = new Set()
            updateColumns()
            const ready = previewMatches('Choose the remaining source columns.')
            const next = firstUnresolved()
            if (next) focus(next)
            else if (ready) apply.focus()
            else toggle.focus()
          })
          row.append(label, example, confirm)
          target.append(row)
          fieldControls[spec.key] = select
        }
      }
      queue.hidden = !queue.children.length
      edit.hidden = !sheet
      recovery = headingPicker(inspected, Boolean(selectSpreadsheetHeadingRow
        && (inspected?.needsReview || inspected?.currentRow > 1 || firstUnresolved())),
      row => chooseHeadingRow(sheetIndex, row))
      recoverySlot.replaceChildren(recovery.panel)
    }

    function refresh(initial = false) {
      const sheet = data.sheets[indexOf(sheetSelect)]
      const suggestions = initial ? mapping : sheet && suggestSheetColumns?.(kind, sheet.headers)?.fields || {}
      const used = new Set()
      state = Object.fromEntries(specs.map(spec => {
        const index = suggestions[spec.key]
        if (!sheet || !Number.isInteger(index) || index < 0 || index >= sheet.headers.length || used.has(index)) return [spec.key, null]
        used.add(index)
        return [spec.key, index]
      }))
      draft = new Set()
      updateColumns()
    }

    sheetSelect.addEventListener('change', () => {
      refresh()
      showRecords()
      previewMatches('Review the source sheet and choose any missing columns.')
    })
    container.replaceChildren(sheetSource, pickSheet, recoverySlot, queue, edit)
    refresh(true)
    return {
      value, firstUnresolved, focus, sheetSelect,
      pending: () => draft.size > 0,
      headingBlocking: () => recovery?.blocking === true,
      focusHeading: () => {
        if (!recovery?.blocking) return false
        recovery.summary.focus()
        return true
      },
    }
  }

  function showRecords() {
    sourceEditors = renderSourcePreview(sourcePreview, data, [
      groups.jobs.value().sheet, groups.people.value().sheet,
    ], canUpdateSpreadsheetCells?.() ? reviseSource : null)
  }

  function reviseSource(sheet, row, column, value) {
    const result = updateSpreadsheetCell?.(data, sheet, row, column, value)
    if (!result?.ok) {
      status.textContent = 'That value could not be saved to the preview. Your current plan is unchanged.'
      return
    }
    const mapping = { jobs: groups.jobs.value(), people: groups.people.value() }
    clearPreview()
    data = result.workbook
    renderGroups(mapping)
    const ready = previewMatches('Review the remaining matches. Your current plan is unchanged.')
    status.textContent = `Source row ${row} updated. ${status.textContent}`
    if (ready) apply.focus()
    else if (rules.children.length) rules.focus()
    else focusUnresolved()
  }

  function focusUnresolved() {
    const { jobs, people } = groups
    const repeated = jobs.value().sheet !== null && jobs.value().sheet === people.value().sheet
    return jobs.focusHeading() || people.focusHeading()
      || jobs.focus(jobs.firstUnresolved()) || people.focus(people.firstUnresolved())
      || (repeated && people.focus(people.sheetSelect))
  }

  function renderGroups(selections) {
    groups = {
      jobs: makeGroup('jobs', find('sheet-map-jobs'), selections.jobs),
      people: makeGroup('people', find('sheet-map-people'), selections.people),
    }
    showRecords()
  }

  function chooseHeadingRow(sheetIndex, row) {
    if (!groups || !selectSpreadsheetHeadingRow) return
    const next = selectSpreadsheetHeadingRow(data, sheetIndex, row)
    if (!next?.ok) {
      status.textContent = 'This source row cannot be used as headings. The current plan is unchanged.'
      return
    }
    const before = { jobs: groups.jobs.value(), people: groups.people.value() }
    clearPreview()
    data = next.workbook
    for (const kind of ['jobs', 'people']) {
      if (before[kind].sheet !== sheetIndex) continue
      const fields = suggestSheetColumns?.(kind, data.sheets[sheetIndex].headers)?.fields ?? {}
      before[kind] = { sheet: sheetIndex, ...fields }
    }
    renderGroups(before)
    const ready = checkRows(false)
    const skipped = next.skippedRows.length
    const nextStep = status.textContent || 'Review the missing matches. The current plan is unchanged.'
    status.textContent = `Using source row ${row} as headings. Skipped ${skipped} nonblank ${skipped === 1 ? 'row' : 'rows'} above it. ${nextStep}`
    if (focusUnresolved()) return
    if (ready) apply.focus()
    else if (rules.children.length) rules.focus()
    else find('sheet-mapper-heading').focus()
  }

  function askCsvSeparator(result, jobs, people, version) {
    if (!separator || !result?.issues) return false
    const slot = result.csvSlot
    if (slot !== 'jobs' && slot !== 'people') return false
    const input = slot === 'jobs' ? jobsFile : peopleFile
    const file = slot === 'jobs' ? jobs : people
    if (!file || input.files[0] !== file) return false
    const codes = result.issues.map(issue => issue.code)
    const ambiguous = codes.includes('ambiguous-csv-delimiter') || codes.includes('unconfirmed-csv-delimiter')
    const wrongChoice = codes.includes('csv-delimiter-choice-invalid') && ambiguousFiles.has(file)
    if (!ambiguous && !wrongChoice) return false
    ambiguousFiles.add(file)

    const controls = element('div')
    controls.className = 'actions'
    separator.replaceChildren(
      element('legend', `Choose the separator for ${file.name}`),
      element('p', wrongChoice
        ? 'That separator did not produce the columns this file needs. Try the other separator. The current plan is unchanged.'
        : 'We couldn’t tell what separates the columns in this CSV. Choose the separator used in the file. The current plan is unchanged.'),
      controls,
    )
    for (const [delimiter, label] of [[',', 'Use commas'], [';', 'Use semicolons']]) {
      const button = element('button', label)
      button.type = 'button'
      button.addEventListener('click', () => {
        if (version !== generation || input.files[0] !== file) return
        separatorsByFile.set(file, delimiter)
        return openSpreadsheet()
      })
      controls.append(button)
    }
    separator.hidden = false
    status.textContent = 'Choose a separator to continue.'
    controls.children[0].focus()
    return true
  }

  function showCsvChoices(jobs, people, version, omit) {
    if (!choices) return
    choices.replaceChildren()
    for (const [slot, input, file, label] of [
      ['jobs', jobsFile, jobs, 'Jobs CSV'],
      ['people', peopleFile, people, 'People CSV'],
    ]) {
      const current = separatorsByFile.get(file)
      if (slot === omit || !file || input.files[0] !== file || !current) continue
      const next = current === ',' ? ';' : ','
      const row = element('div')
      const switchButton = element('button', `Use ${next === ',' ? 'commas' : 'semicolons'} for ${file.name}`)
      switchButton.type = 'button'
      row.append(element('p', `${label}: ${file.name} uses ${current === ',' ? 'commas' : 'semicolons'}.`), switchButton)
      switchButton.addEventListener('click', () => {
        if (version !== generation || input.files[0] !== file || workbook.files[0]) return
        separatorsByFile.set(file, next)
        return openSpreadsheet()
      })
      choices.append(row)
    }
    choices.hidden = choices.children.length === 0
  }

  async function openSpreadsheet() {
    clearFiles()
    const version = generation
    const excel = workbook.files[0]
    const jobs = jobsFile.files[0]
    const people = peopleFile.files[0]
    if (!excel && (!jobs || !people)) {
      status.textContent = 'Choose one Excel workbook or both CSV files.'
      return
    }
    load.disabled = true
    status.textContent = 'Reading column names on this device…'
    let result
    try {
      result = await readFiles({
        workbook: excel, jobs, people,
        csvDelimiters: excel ? undefined : { jobs: separatorsByFile.get(jobs), people: separatorsByFile.get(people) },
      })
    } catch {
      result = null
    }
    if (version !== generation) return
    load.disabled = false
    if (!result || !result.ok || !Array.isArray(result.workbook?.sheets) || result.workbook.sheets.length < 2) {
      if (!excel && askCsvSeparator(result, jobs, people, version)) {
        showCsvChoices(jobs, people, version, result.csvSlot)
        return
      }
      if (!excel) showCsvChoices(jobs, people, version)
      const known = new Set(result?.issues?.map(issue => issue.code))
      status.textContent = known.has('formula') ? 'The workbook contains formulas. Replace formulas with values and try again.'
        : known.has('file-limit') ? 'This file is too large. Choose a smaller workbook or CSV.'
        : known.has('sheet-limit') ? 'This file has too many rows, columns or sheets. Choose a smaller workbook or CSV.'
        : 'Could not read the file. Choose an Excel workbook (.xlsx) or export each sheet as CSV.'
      return
    }
    if (!excel) showCsvChoices(jobs, people, version)
    data = result.workbook
    sourceName = excel ? excel.name : [jobs.name, people.name].join(' and ')
    csvSources = excel ? [] : [jobs.name, people.name]
    mapper.hidden = false
    renderGroups(suggestColumnMapping(data))
    previewMatches('Choose the missing source columns.')
    find('sheet-mapper-heading').focus()
  }

  load.addEventListener('click', openSpreadsheet)
  check.addEventListener('click', () => checkRows(true))

  function previewMatches(hint) {
    const ready = checkRows(false)
    if (!ready && !status.textContent) {
      status.textContent = groups.jobs.headingBlocking() || groups.people.headingBlocking()
        ? 'Check which row contains the column names before continuing.'
        : hint
    }
    return ready
  }

  function measureMatches() {
    const jobs = groups.jobs.value()
    const people = groups.people.value()
    const countMatches = (kind, mapping) => mapping.sheet === null ? 0 : new Set(
      fieldsByKind[kind].map(field => mapping[field.key]).filter(index => index !== null),
    ).size
    const separate = jobs.sheet !== people.sheet
    const jobsMatched = countMatches('jobs', jobs)
    const peopleMatched = separate ? countMatches('people', people) : 0
    progress.textContent = `${jobsMatched + peopleMatched} of 6 fields matched`
    const allowed = separate && jobsMatched === fieldsByKind.jobs.length
      && peopleMatched === fieldsByKind.people.length
    return { jobs, people, allowed }
  }

  function checkRows(showMissing) {
    if (!groups) return
    clearPreview()
    const { jobs, people, allowed } = measureMatches()
    if (groups.jobs.pending() || groups.people.pending()) {
      status.textContent = 'Use the selected column to update this preview. The current plan is unchanged.'
      return false
    }
    if (groups.jobs.headingBlocking() || groups.people.headingBlocking()) {
      status.textContent = 'Check which row contains the column names. The current plan is unchanged.'
      if (showMissing) focusUnresolved()
      return false
    }
    if (!allowed && !showMissing) return false
    let parsed
    try {
      parsed = allowed ? mapSpreadsheetWorkbook(data, { jobs, people }) : { ok: false, issues: [{ code: 'mapping' }] }
    } catch {
      parsed = { ok: false, issues: [{ code: 'invalid-file' }] }
    }
    if (!parsed.ok) {
      status.textContent = 'Nothing changed. Check the fields or rows below.'
      for (const issue of parsed.issues.slice(0, 5)) {
        const index = issue.sheet ?? (issue.range === 'jobs' ? jobs.sheet : issue.range === 'people' ? people.sheet : null)
        const label = Number.isInteger(index) ? csvSources[index] || data.sheets[index]?.name : undefined
        const line = element('li', issueText(issue, label))
        const edit = sourceEditors.get(index)
        if (edit && Number.isInteger(issue.row) && Number.isInteger(issue.column)) {
          const button = element('button', `Edit source row ${issue.row}`)
          button.type = 'button'
          button.addEventListener('click', () => edit(issue.row, issue.column))
          line.append(button)
        }
        rules.append(line)
      }
      if (parsed.issues.length > 5) rules.append(element('li', `${parsed.issues.length - 5} more rows need attention.`))
      if (showMissing && !focusUnresolved()) rules.focus()
      return false
    }
    candidate = parsed.setup
    status.textContent = `${candidate.jobs.length} ${candidate.jobs.length === 1 ? 'job' : 'jobs'} and ${candidate.people.length} ${candidate.people.length === 1 ? 'person' : 'people'} checked. Review the matches before calculating. Your current plan is unchanged.`
    apply.disabled = false
    return true
  }

  apply.addEventListener('click', () => {
    if (!candidate) return
    const next = candidate
    const file = sourceName
    reset()
    replaceSetup(next, file)
    status.textContent = 'Spreadsheet applied. Review the new plan above.'
  })

  discard?.addEventListener('click', () => {
    reset()
    status.textContent = 'Spreadsheet preview discarded. Your current plan is unchanged.'
    workbook.focus()
  })

  function reset() {
    separatorsByFile = new WeakMap()
    ambiguousFiles = new WeakSet()
    clearFiles()
    for (const input of [workbook, jobsFile, peopleFile]) input.value = ''
  }

  root.hidden = false
  return { clearPreview, reset }
}
