import { createEngine } from './try-review-engine.mjs'

let runtime
function loadAsset(path, kind) {
  return fetch(new URL(path, import.meta.url)).then(response => {
    if (!response.ok) throw new Error('The planning tools could not load.')
    return kind === 'bytes' ? response.arrayBuffer() : response.text()
  })
}

self.onmessage = async ({ data }) => {
  const { requestId, problem } = data
  try {
    runtime ??= Promise.all([
      loadAsset('./try-review-engine.wasm', 'bytes').then(createEngine),
      loadAsset('./try-assignment-recipe.xml', 'text')
    ])
    const [engine, recipe] = await runtime
    const pairIndices = []
    problem.eligible_workers_by_job.forEach((workers, job) => {
      for (const worker of workers) pairIndices.push(job, worker)
    })
    const inputs = {
      'job-names': problem.eligible_workers_by_job.map((_, index) => `job-${index + 1}`),
      'technician-names': problem.worker_capacities.map((_, index) => `worker-${index + 1}`),
      'technician-capacities': problem.worker_capacities,
      'eligible-pairs': pairIndices
    }
    const solution = engine.runAssignmentRecipe(recipe, inputs)
    self.postMessage({ requestId, status: 'solved', solution })
  } catch {
    self.postMessage({ requestId, status: 'error' })
  }
}
