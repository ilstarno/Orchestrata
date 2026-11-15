import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { renderToStaticMarkup } from 'react-dom/server'
import { describe, expect, it } from 'vitest'
import { App } from '../../app'
import { PlanResult } from '../../features/assignment/plan-result'
import { SolveError } from '../../shared/api/solve'

describe('assignment workspace', () => {
  it('starts with an understandable example and keeps JSON closed', () => {
    const page = renderToStaticMarkup(
      <QueryClientProvider client={new QueryClient()}><App /></QueryClientProvider>,
    )

    expect(page).toContain('Assign jobs to the right people.')
    expect(page).toContain('Each person can do one job.')
    expect(page).toContain('Lift repair')
    expect(page).toContain('Run example')
    expect(page).toContain('<details class="advanced-panel setup-advanced">')
    expect(page).not.toContain('Run edited setup')
  })

  it('presents named assignments and labels stale results', () => {
    const page = renderToStaticMarkup(
      <PlanResult
        pending={false}
        result={{ kind: 'solved', assignment: { values: [8, 2, 3], score: { hard: 0, soft: 0 }, evaluated: 8 } }}
        error={null}
        example={true}
        edited={true}
      />,
    )

    expect(page).toContain('All three jobs covered')
    expect(page).toContain('This result is from the last run.')
    expect(page).toContain('Lift repair</dt><dd>Alex')
    expect(page).toContain('Safety inspection</dt><dd>Sam')
    expect(page).toContain('Preventive maintenance</dt><dd>Noor')
  })

  it('distinguishes an unchecked run from a proven infeasible plan', () => {
    const limited = renderToStaticMarkup(
      <PlanResult pending={false} error={new SolveError('search-limit')} example={false} edited={false} />,
    )
    const infeasible = renderToStaticMarkup(
      <PlanResult pending={false} error={null} result={{ kind: 'infeasible' }} example={false} edited={false} />,
    )

    expect(limited).toContain('role="alert"')
    expect(limited).toContain('Too many combinations')
    expect(limited).not.toContain('No assignment fits all the rules')
    expect(infeasible).toContain('No assignment fits all the rules')
    expect(infeasible).not.toContain('role="alert"')
  })
})
