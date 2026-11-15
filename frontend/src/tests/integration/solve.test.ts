import { afterEach, describe, expect, it, vi } from 'vitest'
import { solveProblem } from '../../shared/api/solve'

afterEach(() => {
  vi.unstubAllGlobals()
})

function mockResponse(body: string, status = 200) {
  const fetchMock = vi.fn().mockResolvedValue(new Response(body, { status }))
  vi.stubGlobal('fetch', fetchMock)
  return fetchMock
}

describe('solve API client', () => {
  it('returns a checked assignment and sends the original definition', async () => {
    const assignment = { values: [1, 0], score: { hard: 0, soft: 0 }, evaluated: 4 }
    const fetchMock = mockResponse(JSON.stringify({ solution: assignment }))

    await expect(solveProblem('{}')).resolves.toEqual({ kind: 'solved', assignment })
    expect(fetchMock).toHaveBeenCalledWith('/api/v1/solve', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: '{}',
    })
  })

  it('recognizes proven infeasibility without treating it as an error', async () => {
    mockResponse('{"solution":null}')
    await expect(solveProblem('{}')).resolves.toEqual({ kind: 'infeasible' })
  })

  it('preserves the exact response if displaying it as numbers would lose precision', async () => {
    const json = '{"solution":{"values":[9223372036854775807],"score":{"hard":0,"soft":0},"evaluated":1}}'
    mockResponse(json)
    await expect(solveProblem('{}')).resolves.toEqual({ kind: 'exact-json', json })
  })

  it.each([
    [422, 'Search space exceeds the evaluation limit', 'search-limit'],
    [422, 'Invalid problem', 'invalid-input'],
    [422, 'Invalid problem or search space too large', 'ambiguous-request'],
    [500, 'Solver execution failed', 'execution'],
    [503, 'Service unavailable', 'busy'],
  ])('classifies HTTP %i with diagnostic %s', async (status, body, kind) => {
    mockResponse(body, status)
    await expect(solveProblem('{}')).rejects.toMatchObject({ kind })
  })

  it('does not present a hard-invalid result as an assignment', async () => {
    mockResponse('{"solution":{"values":[1,1],"score":{"hard":-1,"soft":0},"evaluated":4}}')
    await expect(solveProblem('{}')).rejects.toMatchObject({ kind: 'unexpected-response' })
  })

  it('rejects malformed JSON before sending a request', async () => {
    const fetchMock = mockResponse('{}')
    await expect(solveProblem('{')).rejects.toMatchObject({ kind: 'invalid-input' })
    expect(fetchMock).not.toHaveBeenCalled()
  })

  it('reports network failures without claiming infeasibility', async () => {
    vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new TypeError('connection closed')))
    await expect(solveProblem('{}')).rejects.toMatchObject({ kind: 'offline' })
  })
})
