import { compileSetup, describePlan, MAX_RECORDS, SAMPLE, validateSetup } from './try-model.js'

const $ = id => document.getElementById(id)
const count = () => setup.jobs.length + setup.people.length
let setup = structuredClone(SAMPLE)
let running = null
let hasAttempt = false
let requestId = 0

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

function changeSetup() {
  stopRun()
  $('setup-error').textContent = ''
  $('result').hidden = true
  $('status').textContent = hasAttempt
    ? 'The setup changed. Find assignments to check it again.'
    : 'Run the sample or edit it first.'
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
  const rowType = isJob ? 'Job' : 'Person'
  const item = element('li')
  const groupFields = element('fieldset')
  const remove = element('button', 'Remove')
  remove.type = 'button'
  remove.setAttribute('aria-label', `Remove ${rowType.toLowerCase()} ${index + 1}`)
  remove.addEventListener('click', () => {
    setup[group].splice(index, 1)
    changeSetup()
    renderEditor()
    const next = $(group).children[Math.min(index, setup[group].length - 1)]
    ;(next?.querySelector('input') ?? $(isJob ? 'add-job' : 'add-person')).focus()
  })
  groupFields.append(element('legend', `${rowType} ${index + 1}`),
    field(`${rowType} name`, row.name, value => { row.name = value }, { maxLength: 80 }))
  if (isJob) {
    groupFields.append(field('Required skill', row.skill, value => { row.skill = value }, { maxLength: 80 }))
  } else {
    groupFields.append(
      field('Skills (separate with commas)', row.skills.join(', '), value => { row.skills = value.split(',').map(x => x.trim()).filter(Boolean) }, { maxLength: 1620 }),
      field('Job limit', row.capacity, value => { row.capacity = value }, { type: 'number' }),
      field('Available', row.available, value => { row.available = value }, { type: 'checkbox' }),
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
  $('record-count').textContent = `${count()} of 200 records`
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
  const plan = describePlan(input, problem, solution)
  const assigned = plan.filter(row => row.person !== null).length
  const list = element('dl')
  for (const row of plan) {
    const item = element('div')
    const detail = row.person ?? (row.reason === 'eligible'
      ? 'Open: no available person with the required skill and a positive job limit.'
      : 'Open: not assigned in this plan.')
    item.append(element('dt', row.job), element('dd', detail))
    list.append(item)
  }
  $('result').replaceChildren(list)
  $('result').hidden = false
  $('status').textContent = `${assigned} of ${plan.length} jobs assigned by this plan.`
  if (document.activeElement === $('run') || document.activeElement === document.body) $('review-heading').focus()
}

function reviewError(message) {
  stopRun()
  $('result').hidden = true
  $('status').textContent = message
}

function runPlan() {
  if (running) return
  hasAttempt = true
  $('result').hidden = true
  $('setup-error').textContent = ''
  const input = structuredClone(setup)
  const compiled = compileSetup(input)
  if (compiled.error) {
    $('setup-error').textContent = compiled.error
    $('status').textContent = 'No plan was run.'
    return
  }

  let worker
  try {
    worker = new Worker(new URL('./try-engine-worker.js', import.meta.url), { type: 'module' })
  } catch {
    reviewError('The solver could not start. No plan was calculated.')
    return
  }
  const run = { worker, timer: 0 }
  running = run
  $('run').disabled = true
  $('result').setAttribute('aria-busy', 'true')
  $('status').textContent = 'Finding assignments…'
  run.timer = setTimeout(() => reviewError('The solver did not finish. No plan was calculated.'), 15000)
  const currentId = ++requestId
  worker.onmessage = ({ data }) => {
    if (running !== run || data?.requestId !== currentId) return
    if (data.status !== 'solved') {
      reviewError('The solver could not return a plan. No plan was calculated.')
      return
    }
    stopRun()
    try {
      renderResult(input, compiled.problem, data.solution)
    } catch {
      reviewError('The solver did not return a valid plan. No plan was calculated.')
    }
  }
  worker.onerror = () => {
    if (running === run) reviewError('The solver could not load. No plan was calculated.')
  }
  worker.postMessage({ requestId: currentId, problem: compiled.problem })
}

$('add-job').addEventListener('click', () => addRow('jobs', { name: '', skill: '' }))
$('add-person').addEventListener('click', () => addRow('people', { name: '', skills: [], available: true, capacity: 1 }))
$('reset').addEventListener('click', () => {
  setup = structuredClone(SAMPLE)
  changeSetup()
  renderEditor()
  $('import-status').textContent = ''
})
$('show-example').addEventListener('click', () => {
  $('import-source').value = JSON.stringify(SAMPLE, null, 2)
  $('import-source').focus()
})
$('import').addEventListener('click', () => {
  const raw = $('import-source').value
  if (raw.length > 2500000) {
    $('import-status').textContent = 'This JSON is too long. Use a smaller setup.'
    return
  }
  let input
  try {
    input = JSON.parse(raw)
  } catch {
    $('import-status').textContent = 'That is not valid JSON. Check its format and try again.'
    return
  }
  const error = validateSetup(input)
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
  $('import-status').textContent = 'Setup imported.'
})
$('run').addEventListener('click', runPlan)
renderEditor()
$('workbench').hidden = false
