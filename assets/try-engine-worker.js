import { createEngine } from './try-engine.mjs'

let engine
self.onmessage = async ({ data: { requestId, problem } }) => {
  try {
    engine ??= fetch(new URL('./try-engine.wasm', import.meta.url))
      .then(response => {
        if (!response.ok) throw new Error('Solver not available')
        return response.arrayBuffer()
      })
      .then(createEngine)
    const solution = (await engine).matchCapacity(problem)
    self.postMessage({ requestId, status: 'solved', solution })
  } catch {
    self.postMessage({ requestId, status: 'error' })
  }
}
