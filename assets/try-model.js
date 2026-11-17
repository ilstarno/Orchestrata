export const MAX_RECORDS = 200
export const SAMPLE = {
  jobs: [
    { name: 'Lift repair', skill: 'Repair' },
    { name: 'Safety inspection', skill: 'Inspection' },
    { name: 'Preventive maintenance', skill: 'Maintenance' },
  ],
  people: [
    { name: 'Ana', skills: ['Repair'], available: true, capacity: 1 },
    { name: 'Rami', skills: ['Repair', 'Maintenance'], available: true, capacity: 1 },
    { name: 'Noor', skills: ['Inspection', 'Maintenance'], available: true, capacity: 1 },
  ],
}

const text = value => typeof value === 'string' && value.trim().length > 0 && value.trim().length <= 80
const list = value => Array.isArray(value) && value.length <= 20 && value.every(text)
const object = value => value !== null && typeof value === 'object' && !Array.isArray(value)
const key = value => value.trim().toLowerCase()

export function validateSetup(input) {
  if (!object(input) || !Array.isArray(input.jobs) || !Array.isArray(input.people)) {
    return 'Include a jobs list and a people list.'
  }
  if (input.jobs.length + input.people.length > MAX_RECORDS) return 'Use no more than 200 jobs and people combined.'
  if (!input.jobs.length) return 'Add at least one job.'
  for (const [index, row] of input.jobs.entries()) {
    if (!object(row) || !text(row.name) || !text(row.skill)) {
      return `Job ${index + 1}: enter a name and required skill (1–80 characters each).`
    }
  }
  for (const [index, row] of input.people.entries()) {
    if (!object(row) || !text(row.name) || !list(row.skills) || typeof row.available !== 'boolean'
      || !Number.isInteger(row.capacity) || row.capacity < 0 || row.capacity > MAX_RECORDS) {
      return `Person ${index + 1}: enter a name, up to 20 skills, availability, and a whole-number job limit from 0 to 200.`
    }
  }
  return null
}

export function compileSetup(input) {
  const error = validateSetup(input)
  if (error) return { error }
  const skillSets = input.people.map(person => new Set(person.skills.map(key)))
  const worker_capacities = input.people.map(person => person.available ? person.capacity : 0)
  const eligible_workers_by_job = input.jobs.map(job => {
    const required = key(job.skill)
    return skillSets.flatMap((skills, index) => worker_capacities[index] > 0 && skills.has(required) ? [index] : [])
  })
  return { problem: { worker_capacities, eligible_workers_by_job } }
}

export function describePlan(input, problem, solution) {
  const assignments = solution?.workers_by_job
  if (!Array.isArray(assignments) || assignments.length !== input.jobs.length) {
    throw new Error('Invalid plan')
  }
  const use = problem.worker_capacities.map(() => 0)
  let count = 0
  for (const [index, worker] of assignments.entries()) {
    if (worker === null) continue
    if (!problem.eligible_workers_by_job[index].includes(worker)
      || ++use[worker] > problem.worker_capacities[worker]) throw new Error('Invalid plan')
    count++
  }
  if (solution.assigned !== count) throw new Error('Invalid plan count')
  return input.jobs.map((job, index) => ({
    job: job.name,
    person: assignments[index] === null ? null : input.people[assignments[index]].name,
    reason: assignments[index] !== null ? null : problem.eligible_workers_by_job[index].length ? 'open' : 'eligible',
  }))
}
