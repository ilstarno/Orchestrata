import { afterEach, describe, expect, it, vi } from 'vitest'
import { solveProblem } from '../../shared/api/solve'

afterEach(() => {
  vi.unstubAllGlobals()
})

describe('solve api client', () => {
  it('returns a successful response', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({ solution: { values: [1, 0] } }),
    }))

    await expect(solveProblem('{}')).resolves.toEqual({
      solution: { values: [1, 0] },
    })
    expect(fetch).toHaveBeenCalledWith('/api/v1/solve', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: '{}',
    })
  })

  it('surfaces a failed request', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue({
      ok: false,
      text: async () => 'invalid problem',
    }))

    await expect(solveProblem('{}')).rejects.toThrow('invalid problem')
  })
})
