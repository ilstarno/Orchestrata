export interface Assignment {
  values: number[]
  score: { hard: number; soft: number }
  evaluated: number
}

export type SolveResult =
  | { kind: 'solved'; assignment: Assignment }
  | { kind: 'exact-json'; json: string }
  | { kind: 'infeasible' }

export type SolveFailure =
  | 'invalid-input'
  | 'search-limit'
  | 'ambiguous-request'
  | 'busy'
  | 'offline'
  | 'execution'
  | 'unexpected-response'

export class SolveError extends Error {
  constructor(readonly kind: SolveFailure) {
    super(kind)
    this.name = 'SolveError'
  }
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null
}

function isInteger(value: unknown): value is number {
  return typeof value === 'number' && Number.isInteger(value)
}

function isAssignment(value: unknown): value is Assignment {
  return isRecord(value)
    && Array.isArray(value.values)
    && value.values.every(isInteger)
    && isRecord(value.score)
    && value.score.hard === 0
    && isInteger(value.score.soft)
    && value.score.soft <= 0
    && isInteger(value.evaluated)
    && value.evaluated > 0
}

function parseResult(raw: string): SolveResult {
  let response: unknown
  try {
    response = JSON.parse(raw)
  } catch {
    throw new SolveError('unexpected-response')
  }

  if (!isRecord(response)) throw new SolveError('unexpected-response')
  const { solution } = response
  if (solution === null) return { kind: 'infeasible' }
  if (!isAssignment(solution)) throw new SolveError('unexpected-response')

  // The backend permits 64-bit integers. Preserve the wire text rather than display rounded numbers.
  const safeIntegers = [...solution.values, solution.score.soft, solution.evaluated].every(Number.isSafeInteger)
  if (!safeIntegers) return { kind: 'exact-json', json: raw }

  return { kind: 'solved', assignment: solution }
}

function classifyFailure(status: number, detail: string): SolveFailure {
  if (status === 429 || status === 503) return 'busy'
  if (status >= 500) return 'execution'
  if (status === 400 || status === 413 || status === 415) return 'invalid-input'
  if (status !== 422) return 'unexpected-response'
  if (detail === 'Search space exceeds the evaluation limit') return 'search-limit'
  if (detail === 'Invalid problem or search space too large') return 'ambiguous-request'
  return 'invalid-input'
}

export async function solveProblem(body: string): Promise<SolveResult> {
  try {
    JSON.parse(body)
  } catch {
    throw new SolveError('invalid-input')
  }

  let response: Response
  let raw: string
  try {
    response = await fetch('/api/v1/solve', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body,
    })
    raw = await response.text()
  } catch {
    throw new SolveError('offline')
  }

  if (!response.ok) throw new SolveError(classifyFailure(response.status, raw.trim()))
  return parseResult(raw)
}
