import { chooseWorkflow } from './workflow-fit.mjs'

const element = (tag, text = '') => {
  const item = document.createElement(tag)
  item.textContent = text
  return item
}
const option = (value, label) => {
  const item = element('option', label)
  item.value = value
  return item
}

async function start(target) {
  const response = await fetch(new URL('./workflow-options.json', import.meta.url))
  if (!response.ok) return
  const { cases, regions } = await response.json()
  if (!Array.isArray(cases) || !Array.isArray(regions)) return
  const form = element('form')
  form.className = 'workflow-questions'
  const result = element('div')
  result.setAttribute('role', 'status')
  result.className = 'workflow-answer'
  const select = (question, values, hint) => {
    const label = element('label', question)
    const field = element('select')
    field.append(option('', hint), ...values.map(([id, name]) => option(id, name)))
    label.append(field)
    form.append(label)
    return field
  }
  const categories = key => [...new Set(cases.map(item => item[key]))].sort().map(value => [value, value])
  const sector = select('What sector are you in?', categories('sector'), 'Choose a sector')
  const teamLabel = element('label', 'What kind of team do you represent?')
  const team = element('input')
  team.placeholder = 'For example, dispatch'
  team.maxLength = 80
  team.autocomplete = 'off'
  team.setAttribute('list', 'workflow-teams')
  const teamList = element('datalist')
  teamList.id = 'workflow-teams'
  teamList.append(...categories('team').map(([id, label]) => option(id, label)))
  teamLabel.append(team, teamList)
  form.append(teamLabel)
  const display = typeof Intl.DisplayNames === 'function' ? new Intl.DisplayNames([document.documentElement.lang || 'en'], { type: 'region' }) : null
  const countries = regions.map(code => [code, display?.of(code) || code]).sort((a, b) => a[1].localeCompare(b[1]))
  const region = select('What country or region are you based in?', countries, 'Prefer not to say')
  const submit = element('button', 'Find an example')
  submit.type = 'submit'
  form.append(submit)
  form.addEventListener('input', () => result.replaceChildren())
  form.addEventListener('submit', event => {
    event.preventDefault()
    const match = chooseWorkflow(cases, { sector: sector.value, team: team.value, region: region.value })
    result.replaceChildren()
    if (!match) {
      result.append(element('p', 'There isn’t a close match yet. You can still try the job assignment example above.'))
      return
    }
    const link = element('a', match.title)
    link.href = match.url
    result.append(element('h3', 'A worked example to start with'), element('p', match.summary), link)
  })
  target.replaceChildren(element('h2', 'Find an example for your team'), element('p', 'Your answers stay in this browser. They help choose an example and don’t change any rules.'), form, result)
  target.hidden = false
}

const target = document.getElementById('planning-profile')
if (target) start(target).catch(() => { target.hidden = true })
