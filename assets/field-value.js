const limits = {
  visits: 1_000_000, minutes: 1_440, improvement: 100, overtime: 1_000_000,
  share: 100, rate: 1_000_000, late: 1_000_000, sla: 100,
}

export const exampleFieldValue = Object.freeze({
  visits: 120, minutes: 30, improvement: 20, overtime: 5, share: 50, rate: 40, late: 10, sla: 30,
})

export function estimateFieldValue(input) {
  if (Object.entries(limits).some(([key, max]) => !Number.isFinite(input[key]) || input[key] < 0 || input[key] > max) ||
      !Number.isInteger(input.visits) || !Number.isInteger(input.late) || input.late > input.visits) return null

  const travel = (input.visits * input.minutes / 60) * (input.improvement / 100)
  const saved = Math.min(input.overtime, travel * (input.share / 100))
  const slaResult = input.late * (input.sla / 100)
  const onTimeBefore = input.visits ? (input.visits - input.late) / input.visits * 100 : null
  const onTimeAfter = input.visits ? (input.visits - input.late + slaResult) / input.visits * 100 : null
  return { travel, saved, cash: saved * input.rate, slaResult, onTimeBefore, onTimeAfter }
}

function mountEstimator(section) {
  const form = section.querySelector('[data-field-form]')
  const error = section.querySelector('[data-field-error]')
  const results = section.querySelector('[data-field-results]')
  const late = form.elements.namedItem('late')
  const number = new Intl.NumberFormat(section.lang, { maximumFractionDigits: 2 })
  const reset = () => {
    error.hidden = true
    results.hidden = true
    late.removeAttribute('aria-invalid')
  }

  form.addEventListener('input', reset)
  form.querySelector('[data-field-example]').addEventListener('click', () => {
    for (const [key, value] of Object.entries(exampleFieldValue)) form.elements.namedItem(key).value = String(value)
    form.requestSubmit()
  })
  form.addEventListener('submit', (event) => {
    event.preventDefault()
    reset()
    const input = Object.fromEntries(Object.keys(limits).map(key => [key, form.elements.namedItem(key).valueAsNumber]))
    const value = estimateFieldValue(input)
    if (!value) {
      error.hidden = false
      if (input.late > input.visits) {
        late.setAttribute('aria-invalid', 'true')
        late.focus()
      }
      return
    }

    for (const [key, estimate] of Object.entries(value)) {
      const output = results.querySelector(`[data-field-result="${key}"]`)
      output.parentElement.hidden = estimate === null
      output.textContent = estimate === null ? '' : number.format(estimate)
    }
    results.hidden = false
  })
  form.hidden = false
}

if (typeof document !== 'undefined') {
  for (const section of document.querySelectorAll('[data-field-value]')) mountEstimator(section)
}
