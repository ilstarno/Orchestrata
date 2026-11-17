function cell(value) {
  let text = String(value ?? '')
  if (typeof value === 'string' && (/^[\s\uFEFF]*[=+\-@]/u.test(text) || /[\u0000-\u001f\u007f]/u.test(text))) {
    text = "'" + text
  }
  return '"' + text.replaceAll('"', '""') + '"'
}

export function encodePlanCsv(plan) {
  const header = ['Job number', 'Job', 'Assigned person number', 'Assigned person', 'Status', 'Reason']
  const rows = plan.assignments.map(({ job, person, open_reason }) => [
    job.position, job.name, person?.position, person?.name, person ? 'Assigned' : 'Open',
    open_reason === 'no_eligible_person' ? 'No available person meets the skill and job limit'
      : open_reason === 'unassigned_in_plan' ? 'Not assigned in this plan' : '',
  ])
  return '\uFEFF' + [header, ...rows].map(row => row.map(cell).join(',')).join('\r\n') + '\r\n'
}

export function downloadPlanCsv(plan) {
  const url = URL.createObjectURL(new Blob([encodePlanCsv(plan)], { type: 'text/csv;charset=utf-8' }))
  const link = document.createElement('a')
  link.href = url
  link.download = 'orchestrata-plan.csv'
  try {
    document.body.append(link)
    link.click()
  } finally {
    link.remove()
    setTimeout(() => URL.revokeObjectURL(url), 1000)
  }
}
