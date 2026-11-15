import React, { useState } from 'react'
import { createRoot } from 'react-dom/client'
import { QueryClient, QueryClientProvider, useMutation } from '@tanstack/react-query'
import { createRootRoute, createRoute, createRouter, RouterProvider } from '@tanstack/react-router'

const sample = {
  variables: [{ name: 'job_a', domain: [0, 1] }, { name: 'job_b', domain: [0, 1] }],
  constraints: [
    { kind: 'all_different', variables: [0, 1], hard_weight: 1 },
    { kind: 'prefer_value', variable: 0, value: 1, soft_weight: 3 },
  ],
}

function Workspace() {
  const [input, setInput] = useState(JSON.stringify(sample, null, 2))
  const mutation = useMutation({
    mutationFn: async (payload: string): Promise<unknown> => {
      const response = await fetch('/api/v1/solve', {
        method: 'POST', headers: { 'Content-Type': 'application/json' }, body: payload,
      })
      if (!response.ok) throw new Error(await response.text())
      return response.json() as Promise<unknown>
    },
  })
  return <main style={{ maxWidth: 900, margin: '3rem auto', padding: '0 1rem', fontFamily: 'system-ui' }}>
    <h1>Orchestrata</h1>
    <p>Define a finite-domain planning problem and inspect its evaluated solution.</p>
    <label htmlFor="problem">Problem definition</label>
    <textarea id="problem" value={input} onChange={event => setInput(event.target.value)}
      style={{ display: 'block', width: '100%', minHeight: 320, fontFamily: 'monospace', margin: '1rem 0' }} />
    <button disabled={mutation.isPending} onClick={() => mutation.mutate(input)}>
      {mutation.isPending ? 'Solving…' : 'Solve'}
    </button>
    {mutation.isError && <p role="alert">{mutation.error.message}</p>}
    {mutation.isSuccess && <section aria-label="Solution"><h2>Result</h2>
      <pre>{JSON.stringify(mutation.data, null, 2)}</pre></section>}
  </main>
}
const root = createRootRoute()
const index = createRoute({ getParentRoute: () => root, path: '/', component: Workspace })
const router = createRouter({ routeTree: root.addChildren([index]) })
declare module '@tanstack/react-router' { interface Register { router: typeof router } }
const client = new QueryClient()
createRoot(document.getElementById('root')!).render(
  <React.StrictMode><QueryClientProvider client={client}>
    <RouterProvider router={router} />
  </QueryClientProvider></React.StrictMode>,
)
