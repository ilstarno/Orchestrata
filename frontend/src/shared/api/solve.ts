export async function solveProblem(body: string): Promise<unknown> {
  const response = await fetch('/api/v1/solve', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body,
  })

  if (!response.ok) {
    throw new Error(await response.text())
  }

  return response.json() as Promise<unknown>
}
