import { describePlan } from './try-model.js'

export function makePlanExport(input, problem, solution) {
  const plan = describePlan(input, problem, solution)
  return {
    version: 1,
    assigned: solution.assigned,
    assignments: plan.map((row, index) => {
      const personIndex = solution.workers_by_job[index]
      return {
        job: { position: index + 1, name: row.job },
        person: personIndex === null ? null : { position: personIndex + 1, name: row.person },
        open_reason: row.reason === 'eligible' ? 'no_eligible_person'
          : row.reason === 'open' ? 'unassigned_in_plan' : null,
      }
    }),
  }
}

export function downloadPlan(plan) {
  const json = JSON.stringify(plan, null, 2) + '\n'
  const url = URL.createObjectURL(new Blob([json], { type: 'application/json' }))
  const link = document.createElement('a')
  link.href = url
  link.download = 'orchestrata-plan.json'
  try {
    document.body.append(link)
    link.click()
  } finally {
    link.remove()
    // Let the browser consume the URL before releasing it.
    setTimeout(() => URL.revokeObjectURL(url), 1000)
  }
}
