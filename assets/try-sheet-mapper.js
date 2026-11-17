export const fieldsByKind = {
  jobs: [
    { key: 'name', label: 'Job name', aliases: ['name', 'job', 'job title', 'task', 'work order'] },
    { key: 'skill', label: 'Required skill', aliases: ['skill required', 'skill', 'qualification'] },
  ],
  people: [
    { key: 'name', label: 'Person name', aliases: ['name', 'person', 'employee', 'technician', 'worker'] },
    { key: 'skills', label: 'Skills', aliases: ['qualifications', 'skill set', 'skill', 'specialties'] },
    { key: 'available', label: 'Available', aliases: ['availability', 'on duty', 'can work'] },
    { key: 'capacity', label: 'Job limit', aliases: ['capacity', 'maximum jobs', 'max jobs', 'jobs available'] },
  ],
}

const normalized = value => String(value ?? '').normalize('NFKC').trim().toLowerCase().replace(/[_-]+/gu, ' ').replace(/\s+/gu, ' ')

export function suggestMapping(kind, headers) {
  return fieldsByKind[kind].map(field => {
    const terms = [field.label, ...field.aliases].map(normalized)
    const matches = headers.flatMap((name, index) => terms.includes(normalized(name)) ? [index] : [])
    return matches.length === 1 ? matches[0] : null
  })
}
