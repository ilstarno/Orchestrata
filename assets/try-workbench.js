import { tryCopy } from './try-i18n.js'
import { compileSetup, MAX_RECORDS, SAMPLE, validateSetup } from './try-model.js?v=3'
import { downloadPlan, makePlanExport } from './try-plan-export.js'

const $ = id => document.getElementById(id)
const count = () => setup.jobs.length + setup.people.length
const copy = tryCopy(document.documentElement.lang)
const sample = copy.sample ?? SAMPLE
let setup = structuredClone(sample)
let running = null
let hasAttempt = false
let requestId = 0
let currentPlan = null

function element(tag, text) {
  const node = document.createElement(tag)
  if (text != null) node.textContent = text
  return node
}

function stopRun() {
  if (!running) return
  clearTimeout(running.timer)
  running.worker.terminate()
  running = null
  $('run').disabled = false
  $('result').removeAttribute('aria-busy')
}

function clearPlan() {
  currentPlan = null
  $('result').hidden = true
  $('result').replaceChildren()
  $('download-actions').hidden = true
  $('download-status').textContent = ''
}

function changeSetup({ reset = false } = {}) {
  const wasChecked = hasAttempt
  stopRun()
  clearPlan()
  $('setup-error').textContent = ''
  if (reset) hasAttempt = false
  const next = reset ? copy.resetNotice
    : wasChecked ? copy.changed
    : ''
  $('setup-status').textContent = next
  $('status').textContent = next || copy.initial
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
    changeSetup()
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
    changeSetup()
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
  $('record-count').textContent = copy.recordCount(count(), MAX_RECORDS)
  for (const button of ['add-job', 'add-person']) $(button).disabled = count() >= MAX_RECORDS
}

function addRow(group, row) {
  if (count() >= MAX_RECORDS) return
  setup[group].push(row)
  changeSetup()
  renderEditor()
  $(group).lastElementChild.querySelector('input').focus()
}

function renderResult(input, problem, solution) {
  const plan = makePlanExport(input, problem, solution)
  const list = element('dl')
  for (const row of plan.assignments) {
    const item = element('div')
    const detail = row.person?.name ?? (row.open_reason === 'no_eligible_person'
      ? copy.ineligible
      : copy.open)
    item.append(element('dt', row.job.name), element('dd', detail))
    list.append(item)
  }
  $('result').replaceChildren(list)
  $('result').hidden = false
  currentPlan = plan
  $('download-actions').hidden = false
  $('status').textContent = copy.resultCount(plan.assigned, plan.assignments.length)
  if (document.activeElement === $('run') || document.activeElement === document.body) $('review-heading').focus()
}

function reviewError(message) {
  stopRun()
  clearPlan()
  $('status').textContent = message
}

function runPlan() {
  if (running) return
  hasAttempt = true
  clearPlan()
  $('setup-status').textContent = ''
  $('setup-error').textContent = ''
  const input = structuredClone(setup)
  const compiled = compileSetup(input, copy.validation)
  if (compiled.error) {
    $('setup-error').textContent = compiled.error
    $('status').textContent = copy.noPlan
    return
  }

  let worker
  try {
    worker = new Worker(new URL('./try-engine-worker.js', import.meta.url), { type: 'module' })
  } catch {
    reviewError(copy.startFailure)
    return
  }
  const run = { worker, timer: 0 }
  running = run
  $('run').disabled = true
  $('result').setAttribute('aria-busy', 'true')
  $('status').textContent = copy.running
  run.timer = setTimeout(() => reviewError(copy.timeout), 15000)
  const currentId = ++requestId
  worker.onmessage = ({ data }) => {
    if (running !== run || data?.requestId !== currentId) return
    if (data.status !== 'solved') {
      reviewError(copy.solverFailure)
      return
    }
    stopRun()
    try {
      renderResult(input, compiled.problem, data.solution)
    } catch {
      reviewError(copy.invalidPlan)
    }
  }
  worker.onerror = () => {
    if (running === run) reviewError(copy.loadFailure)
  }
  worker.postMessage({ requestId: currentId, problem: compiled.problem })
}

$('add-job').addEventListener('click', () => addRow('jobs', { name: '', skill: '' }))
$('add-person').addEventListener('click', () => addRow('people', { name: '', skills: [], available: true, capacity: 1 }))
$('reset').addEventListener('click', () => {
  setup = structuredClone(sample)
  changeSetup({ reset: true })
  renderEditor()
  $('import-status').textContent = ''
})
$('show-example').addEventListener('click', () => {
  $('import-source').value = JSON.stringify(sample, null, 2)
  $('import-source').focus()
})
$('import').addEventListener('click', () => {
  const raw = $('import-source').value
  if (raw.length > 2500000) {
    $('import-status').textContent = copy.importTooLong
    return
  }
  let input
  try {
    input = JSON.parse(raw)
  } catch {
    $('import-status').textContent = copy.invalidJson
    return
  }
  if (input?.version === 1 && Array.isArray(input.assignments)) {
    $('import-status').textContent = copy.importResult
    return
  }
  const error = validateSetup(input, copy.validation)
  if (error) {
    $('import-status').textContent = error
    return
  }
  setup = {
    jobs: input.jobs.map(({ name, skill }) => ({ name, skill })),
    people: input.people.map(({ name, skills, available, capacity }) => ({ name, skills, available, capacity })),
  }
  changeSetup()
  renderEditor()
  $('import-status').textContent = copy.imported
})
$('download-plan').addEventListener('click', () => {
  if (!currentPlan || running) return
  try {
    $('download-status').textContent = ''
    downloadPlan(currentPlan)
  } catch {
    $('download-status').textContent = copy.downloadFailed
  }
})
$('run').addEventListener('click', runPlan)
renderEditor()
$('workbench').hidden = false
