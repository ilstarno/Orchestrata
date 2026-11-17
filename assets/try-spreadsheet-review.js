import { fieldsByKind, suggestMapping } from './try-sheet-mapper.js'

const issuesByCode = {
  mapping: 'Choose separate job and people sheets and one different column for each field.',
  'invalid-file': 'These rows could not be checked. Try reopening the spreadsheet.',
  'input-limit': 'The selected columns are too large. Use a smaller workbook.',
  'record-limit': 'Use no more than 200 jobs and people combined.',
  'no-jobs': 'Add at least one job.',
  'column-count': 'A row is missing one or more selected columns.',
  'invalid-quote': 'A quoted cell could not be read.',
  'invalid-skills': 'Use up to 20 comma-separated skills.',
  'invalid-available': 'Use Yes or No for availability.',
  'invalid-capacity': 'Use a whole number from 0 to 200 for the job limit.',
}

function issueText(issue) {
  const detail = issue.code === 'invalid-text'
    ? (issue.field === 'skill' ? 'Enter one required skill.' : 'Enter a name.')
    : issuesByCode[issue.code] ?? 'This row could not be read.'
  const range = issue.range === 'people' ? 'People' : 'Jobs'
  return Number.isInteger(issue.row) ? `${range}, row ${issue.row}: ${detail}` : detail
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
export function wireSpreadsheetReview(root, { readFiles, suggestColumnMapping, mapSpreadsheetWorkbook, replaceSetup }) {
  const find = id => root.querySelector('#' + id)
  const load = find('sheet-load')
  const workbook = find('sheet-workbook')
  const jobsFile = find('sheet-jobs-csv')
  const peopleFile = find('sheet-people-csv')
  const mapper = find('sheet-mapper')
  const check = find('sheet-check')
  const apply = find('sheet-apply')
  const status = find('sheet-status')
  const rules = find('sheet-issues')
  if (![load, workbook, jobsFile, peopleFile, mapper, check, apply, status, rules].every(Boolean)) return null
  let generation = 0
  let data = null
  let groups = null
  let candidate = null
  let sourceName = null

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
    groups = null
    load.disabled = false
    mapper.hidden = true
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
    const names = data.sheets.map(sheet => sheet.name)
    const explicit = mapping.sheet ?? data.sheets.findIndex(sheet => sheet.name.trim().toLowerCase() === kind)
    const { root: sheetLabel, select: sheetSelect } = labeledSelect(kind === 'jobs' ? 'Jobs sheet' : 'People sheet', names, explicit < 0 ? null : explicit)
    const columns = element('div')
    columns.className = 'import-fields'
    let fieldControls = []
    function refresh(initial = false) {
      const selected = indexOf(sheetSelect)
      const sheet = data.sheets[selected]
      columns.replaceChildren()
      fieldControls = []
      if (!sheet) return
      const guess = suggestMapping(kind, sheet.headers)
      for (const [i, spec] of fieldsByKind[kind].entries()) {
        const { root: label, select } = labeledSelect(spec.label, sheet.headers, initial ? mapping[spec.key] ?? guess[i] : guess[i])
        select.addEventListener('change', clearPreview)
        columns.append(label)
        fieldControls.push(select)
      }
    }
    sheetSelect.addEventListener('change', () => { clearPreview(); refresh() })
    container.replaceChildren(sheetLabel, columns)
    refresh(true)
    return () => ({
      sheet: indexOf(sheetSelect),
      ...Object.fromEntries(fieldsByKind[kind].map((spec, index) =>
        [spec.key, fieldControls[index] ? indexOf(fieldControls[index]) : null])),
    })
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
      result = await readFiles({ workbook: excel, jobs, people })
    } catch {
      result = null
    }
    if (version !== generation) return
    load.disabled = false
    if (!result || !result.ok || !Array.isArray(result.workbook?.sheets) || result.workbook.sheets.length < 2) {
      const known = new Set(result?.issues?.map(issue => issue.code))
      status.textContent = known.has('formula') ? 'The workbook contains formulas. Replace formulas with values and try again.'
        : known.has('file-limit') ? 'This file is too large. Choose a smaller workbook or CSV.'
        : known.has('sheet-limit') ? 'This file has too many rows, columns or sheets. Choose a smaller workbook or CSV.'
        : 'Could not read the file. Choose an Excel workbook (.xlsx) or export each sheet as CSV.'
      return
    }
    data = result.workbook
    sourceName = excel ? excel.name : [jobs.name, people.name].join(' and ')
    mapper.hidden = false
    const suggestions = suggestColumnMapping(data)
    groups = {
      jobs: makeGroup('jobs', find('sheet-map-jobs'), suggestions.jobs),
      people: makeGroup('people', find('sheet-map-people'), suggestions.people),
    }
    if (!checkRows(false)) status.textContent = status.textContent || 'Match the remaining columns, then check the rows.'
    find('sheet-mapper-heading').focus()
  }

  load.addEventListener('click', openSpreadsheet)
  check.addEventListener('click', () => checkRows(true))

  function checkRows(showMissing) {
    if (!groups) return
    clearPreview()
    const jobs = groups.jobs()
    const people = groups.people()
    const different = (mapping, keys) =>
      keys.every(k => mapping[k] !== null) && new Set(keys.map(k => mapping[k])).size === keys.length
    const allowed = jobs.sheet !== null && people.sheet !== null && jobs.sheet !== people.sheet
      && different(jobs, ['name', 'skill'])
      && different(people, ['name', 'skills', 'available', 'capacity'])
    if (!allowed && !showMissing) return false
    let parsed
    try {
      parsed = allowed ? mapSpreadsheetWorkbook(data, { jobs, people }) : { ok: false, issues: [{ code: 'mapping' }] }
    } catch {
      parsed = { ok: false, issues: [{ code: 'invalid-file' }] }
    }
    if (!parsed.ok) {
      status.textContent = 'Nothing changed. Check the fields or rows below.'
      for (const issue of parsed.issues.slice(0, 5)) rules.append(element('li', issueText(issue)))
      if (parsed.issues.length > 5) rules.append(element('li', `${parsed.issues.length - 5} more rows need attention.`))
      return false
    }
    candidate = parsed.setup
    status.textContent = `Ready: ${candidate.jobs.length} job records and ${candidate.people.length} people records. Nothing has changed yet.`
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

  function reset() {
    clearFiles()
    for (const input of [workbook, jobsFile, peopleFile]) input.value = ''
  }

  root.hidden = false
  return { clearPreview, reset }
}
