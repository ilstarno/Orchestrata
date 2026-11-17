import { tryCopy } from './try-i18n.js'
import { compileSetup, MAX_RECORDS, SAMPLE } from './try-model.js?v=3'
import { makePlanExport } from './try-plan-export.js'
import { downloadPlanCsv } from './try-plan-csv.js'

const $ = id => document.getElementById(id)
const copy = tryCopy('en')
const count = () => setup.jobs.length + setup.people.length
let setup = structuredClone(SAMPLE)
let source = 'sample'
let importedFile = null
let absent = false
let baseline = null
let currentPlan = null
let running = null
let requestId = 0
let importer = null

function element(tag, text) {
  const node = document.createElement(tag)
  if (text != null) node.textContent = text
  return node
}

function sourceLabel() {
  return source === 'sample' ? (absent ? 'Sample field service plan · Ana unavailable' : 'Sample field service plan')
    : source === 'spreadsheet' ? (importedFile || 'Your spreadsheet')
    : source === 'spreadsheet-edited' ? ((importedFile || 'Your spreadsheet') + ' · edited here')
    : 'Edited sample · on this device'
}

function syncControls(retry = false) {
  $('plan-source').textContent = sourceLabel()
  $('scenario').hidden = source !== 'sample'
  $('scenario').disabled = Boolean(running) || !baseline
  $('scenario').textContent = absent ? 'Put Ana back' : 'Try without Ana'
  $('calculate-plan').hidden = source === 'sample' && !retry
  $('calculate-plan').disabled = Boolean(running)
  $('run').disabled = Boolean(running)
}

function stopRun() {
  if (running) {
    clearTimeout(running.timer)
    running.worker.terminate()
    running = null
  }
  $('result').removeAttribute('aria-busy')
}

function clearResult() {
  currentPlan = null
  $('result').hidden = true
  $('result').replaceChildren()
  $('download-actions').hidden = true
  $('download-status').textContent = ''
}

function changeSetup(message) {
  stopRun()
  clearResult()
  $('setup-error').textContent = ''
  $('status').textContent = message
  syncControls()
}

function markEdited() {
  source = source === 'spreadsheet' || source === 'spreadsheet-edited' ? 'spreadsheet-edited' : 'edited'
  importer?.clearPreview()
  changeSetup('Inputs changed. Calculate a new plan before reviewing or downloading it.')
}

function field(title, value, onChange, options = {}) {
  const label = element('label')
  const input = element('input')
  input.type = options.type ?? 'text'
  if (input.type === 'checkbox') {
    input.checked = value
    label.className = 'toggle'
    label.append(input, element('span', title))
  } else {
    input.value = value
    if (options.maxLength) input.maxLength = options.maxLength
    if (input.type === 'number') {
      input.min = '0'
      input.max = '200'
      input.step = '1'
      input.inputMode = 'numeric'
    }
    label.append(element('span', title), input)
  }
  input.addEventListener('input', () => {
    const updated = input.type === 'checkbox' ? input.checked
      : input.type === 'number' ? input.valueAsNumber : input.value
    onChange(updated)
    markEdited()
  })
  return label
}

function editorRow(group, row, index) {
  const isJob = group === 'jobs'
  const item = element('li')
  const groupFields = element('fieldset')
  const remove = element('button', copy.remove)
  remove.type = 'button'
  remove.setAttribute('aria-label', copy.removeRow(isJob, index + 1))
  remove.addEventListener('click', () => {
    setup[group].splice(index, 1)
    markEdited()
    renderEditor()
    const next = $(group).children[Math.min(index, setup[group].length - 1)]
    ;(next?.querySelector('input') ?? $(isJob ? 'add-job' : 'add-person')).focus()
  })
  groupFields.append(element('legend', copy.rowLegend(isJob, index + 1)),
    field(copy.rowName(isJob), row.name, value => { row.name = value }, { maxLength: 80 }))
  if (isJob) {
    groupFields.append(field(copy.requiredSkill, row.skill, value => { row.skill = value }, { maxLength: 80 }))
  } else {
    groupFields.append(
      field(copy.skills, row.skills.join(', '), value => { row.skills = value.split(',').map(x => x.trim()).filter(Boolean) }, { maxLength: 1620 }),
      field(copy.jobLimit, row.capacity, value => { row.capacity = value }, { type: 'number' }),
      field(copy.available, row.available, value => { row.available = value }, { type: 'checkbox' }),
    )
  }
  groupFields.append(remove)
  item.append(groupFields)
  return item
}

function renderEditor() {
  for (const group of ['jobs', 'people']) {
    $(group).replaceChildren(...setup[group].map((row, index) => editorRow(group, row, index)))
  }
  $('record-count').textContent = `${count()} of ${MAX_RECORDS} entries`
  for (const button of ['add-job', 'add-person']) $(button).disabled = count() >= MAX_RECORDS
}

function addRow(group, row) {
  if (count() >= MAX_RECORDS) return
  setup[group].push(row)
  markEdited()
  renderEditor()
  $(group).lastElementChild.querySelector('input').focus()
}

function planCell(row, atLimit = false) {
  const cell = element('td', row.person?.name ?? 'Open')
  if (row.open_reason) {
    cell.className = 'open'
    cell.append(element('small', row.open_reason === 'no_eligible_person'
      ? 'No available person meets the skill and job limit.'
      : atLimit ? 'Eligible people are at their job limit in this plan.'
      : 'Not assigned in this plan.'))
  }
  return cell
}

function renderPlan(plan, problem) {
  const used = problem.worker_capacities.map(() => 0)
  for (const { person } of plan.assignments) if (person) used[person.position - 1]++
  const comparison = source === 'sample' && absent ? baseline : null
  const table = element('table')
  if (comparison) table.className = 'comparison'
  table.setAttribute('aria-label', comparison ? 'Compare original and current assignments' : 'Current assignments')
  const head = element('thead')
  const hr = element('tr')
  for (const title of comparison ? ['Job', 'Original plan', 'With Ana out'] : ['Job', 'Assigned to']) {
    const label = element('th', title)
    label.scope = 'col'
    hr.append(label)
  }
  head.append(hr)
  table.append(head)
  const body = element('tbody')
  for (const [index, row] of plan.assignments.entries()) {
    const tr = element('tr')
    const name = element('th', row.job.name)
    name.scope = 'row'
    tr.append(name)
    const before = comparison?.assignments[index]
    if (before) tr.append(planCell(before))
    const eligible = problem.eligible_workers_by_job[index]
    const atLimit = eligible.length > 0 && eligible.every(worker => used[worker] === problem.worker_capacities[worker])
    const cell = planCell(row, atLimit)
    if (before && before.person?.position !== row.person?.position) cell.classList.add('changed')
    tr.append(cell)
    body.append(tr)
  }
  table.append(body)
  $('result').replaceChildren(table)
  $('result').hidden = false
  $('download-actions').hidden = false
  const open = comparison && plan.assignments.find((row, i) => !row.person && comparison.assignments[i].person)
  $('status').textContent = `${plan.assigned} of ${plan.assignments.length} jobs covered.${open ? ` ${open.job.name} is unassigned.` : ''}`
  currentPlan = plan
}

function runFailure(message) {
  stopRun()
  clearResult()
  $('status').textContent = message
  syncControls(true)
}

function runPlan(focus = false) {
  if (running) return
  clearResult()
  $('setup-error').textContent = ''
  const input = structuredClone(setup)
  const compiled = compileSetup(input, copy.validation)
  if (compiled.error) {
    $('editor').open = true
    $('setup-error').textContent = compiled.error
    $('status').textContent = 'No plan was calculated. Check the inputs and try again.'
    $('setup-error').focus()
    syncControls(true)
    return
  }

  let worker
  try {
    worker = new Worker(new URL('./try-review-worker.js', import.meta.url), { type: 'module' })
  } catch {
    runFailure(copy.startFailure)
    return
  }
  const active = { worker, id: ++requestId, timer: 0 }
  running = active
  $('result').setAttribute('aria-busy', 'true')
  $('status').textContent = source === 'sample' && !baseline ? 'Calculating the sample…' : 'Calculating assignments…'
  syncControls()
  active.timer = setTimeout(() => runFailure(copy.timeout), 15000)
  worker.onmessage = ({ data }) => {
    if (running !== active || data?.requestId !== active.id) return
    if (data.status !== 'solved') {
      runFailure(copy.solverFailure)
      return
    }
    stopRun()
    try {
      const plan = makePlanExport(input, compiled.problem, data.solution)
      if (source === 'sample' && !absent) baseline = plan
      renderPlan(plan, compiled.problem)
      syncControls()
      if (focus) $('review-heading').focus()
    } catch {
      runFailure(copy.invalidPlan)
    }
  }
  worker.onerror = () => {
    if (running === active) runFailure(copy.loadFailure)
  }
  worker.postMessage({ requestId: active.id, problem: compiled.problem })
}

function replaceFromSpreadsheet(input, file) {
  importedFile = file || null
  setup = structuredClone(input)
  source = 'spreadsheet'
  changeSetup('Spreadsheet loaded. Calculating assignments…')
  renderEditor()
  runPlan(true)
}

$('scenario').addEventListener('click', () => {
  if (running || !baseline || source !== 'sample') return
  absent = !absent
  setup = structuredClone(SAMPLE)
  if (absent) setup.people[0].available = false
  renderEditor()
  runPlan(true)
})
$('add-job').addEventListener('click', () => addRow('jobs', { name: '', skill: '' }))
$('add-person').addEventListener('click', () => addRow('people', { name: '', skills: [], available: true, capacity: 1 }))
$('reset').addEventListener('click', () => {
  setup = structuredClone(SAMPLE)
  source = 'sample'
  importedFile = null
  absent = false
  baseline = null
  importer?.reset()
  changeSetup('Sample restored.')
  renderEditor()
  runPlan(true)
})
for (const control of ['run', 'calculate-plan']) $(control).addEventListener('click', () => runPlan(true))
$('download-plan').addEventListener('click', () => {
  if (!currentPlan || running) return
  try {
    $('download-status').textContent = ''
    downloadPlanCsv(currentPlan)
  } catch {
    $('download-status').textContent = 'Could not prepare the plan. Try again.'
  }
})

renderEditor()
$('workbench').hidden = false
runPlan()

$('open-excel').addEventListener('click', event => {
  if (!importer) return
  event.preventDefault()
  $('sheet-workbook').click()
})
$('open-csv').addEventListener('click', () => {
  const csv = document.querySelector('.csv-option')
  if (csv) csv.open = true
})

const spreadsheetRoot = $('spreadsheet-import')
if (spreadsheetRoot) {
  Promise.all([import('./try-spreadsheet-review.js?v=records1'), import('./try-spreadsheet-cell.js')]).then(
    ([{ wireSpreadsheetReview }, { updateSpreadsheetCell }]) => {
    let files
    importer = wireSpreadsheetReview(spreadsheetRoot, {
      replaceSetup: replaceFromSpreadsheet,
      suggestColumnMapping: workbook => files.suggestColumnMapping(workbook),
      suggestSheetColumns: (kind, headers) => files.suggestSheetColumns?.(kind, headers),
      inspectSpreadsheetHeadingRows: (workbook, sheet, kind) => files.inspectSpreadsheetHeadingRows?.(workbook, sheet, kind),
      selectSpreadsheetHeadingRow: (workbook, sheet, row) => files.selectSpreadsheetHeadingRow?.(workbook, sheet, row),
      canUpdateSpreadsheetCells: () => true,
      updateSpreadsheetCell,
      mapSpreadsheetWorkbook: (workbook, mapping) => files.mapSpreadsheetWorkbook(workbook, mapping),
      async readFiles({ workbook, jobs, people, csvDelimiters }) {
        files ??= await import('./try-spreadsheet-file.js?v=delimiter1')
        if (workbook) return files.parseSpreadsheetFile(workbook)
        async function readCsv(file, csvSlot, requiredColumns) {
          const csvDelimiter = csvDelimiters?.[csvSlot]
          const parsed = await files.parseSpreadsheetFile(file, { csvDelimiter })
          if (csvDelimiter && (
            (parsed.ok && parsed.workbook.sheets[0]?.headers.length < requiredColumns)
            || (!parsed.ok && parsed.issues.some(issue => issue.code === 'invalid-file'))
          )) return { ok: false, csvSlot, issues: [{ code: 'csv-delimiter-choice-invalid' }] }
          return parsed.ok ? parsed : { ...parsed, csvSlot }
        }
        const [a, b] = await Promise.all([readCsv(jobs, 'jobs', 2), readCsv(people, 'people', 4)])
        if (!a.ok) return a
        if (!b.ok) return b
        const inputs = [
          { ...a.workbook, sheets: a.workbook.sheets.map(sheet => ({ ...sheet, name: 'Jobs' })) },
          { ...b.workbook, sheets: b.workbook.sheets.map(sheet => ({ ...sheet, name: 'People' })) },
        ]
        return { ok: true, workbook: files.combineSpreadsheetWorkbooks(...inputs) }
      },
    })
  }).catch(() => {
    $('workbook-status').textContent = 'Spreadsheet tools could not load. The sample is still available.'
  })
}
