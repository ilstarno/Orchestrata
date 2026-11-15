import { Braces, Check, ClipboardList } from 'lucide-react'
import { type SolveResult, SolveError } from '../../shared/api/solve'
import { exampleJobs, exampleTechnicians } from '../../shared/examples/assignment'

const failureCopy = {
  'invalid-input': ['Check the setup', 'The job definition isn’t valid. Check the JSON or try the example again.'],
  'search-limit': ['Too many combinations', 'This run was too large to check. Reduce the options per job or try a smaller group of jobs. This does not mean the jobs can’t be assigned.'],
  'ambiguous-request': ['The setup couldn’t be checked', 'The service could not accept this setup or it exceeded the search limit. Check the definition and try a smaller problem.'],
  busy: ['The service is busy', 'Give it a moment, then run the plan again.'],
  offline: ['Can’t reach the service', 'Check your connection or try again shortly.'],
  execution: ['The run didn’t finish', 'The service encountered an error. No conclusion was reached about these jobs.'],
  'unexpected-response': ['Couldn’t read the result', 'The service didn’t return an assignment plan in the expected format.'],
} as const

interface PlanResultProps {
  pending: boolean
  result?: SolveResult
  error: Error | null
  example: boolean
  edited: boolean
}

function getStatus(pending: boolean, result: SolveResult | undefined, error: Error | null) {
  if (pending) return 'Checking'
  if (error) return 'Not completed'
  if (result?.kind === 'infeasible') return 'No plan'
  if (result) return 'Plan ready'
  return 'Not started'
}

export function PlanResult({ pending, result, error, example, edited }: PlanResultProps) {
  return (
    <section className="panel result-panel" aria-labelledby="plan-heading">
      <div className="panel-header">
        <div><span className="panel-kicker">2. Review</span><h2 id="plan-heading">Assignment plan</h2></div>
        <span className="result-status">{getStatus(pending, result, error)}</span>
      </div>
      <div className="sr-only" role="status">{getStatus(pending, result, error)}</div>
      <div className="result-content" aria-busy={pending}>
        {pending && <EmptyPlan title="Finding a plan" text="Checking which assignments fit the rules." />}
        {!pending && error && <Failure error={error} />}
        {!pending && result && (
          <div className="plan">
            {edited && <p className="rule-note">This result is from the last run. Run the edited setup to refresh it.</p>}
            <Result result={result} example={example} />
          </div>
        )}
        {!pending && !error && !result && (
          <EmptyPlan title="No plan yet" text="Run the example to see who should take each job." />
        )}
      </div>
      <div className="panel-footer muted">Up to 100,000 possible assignments per run</div>
    </section>
  )
}

function EmptyPlan({ title, text }: { title: string; text: string }) {
  return (
    <div className="empty-state">
      <div className="empty-icon"><ClipboardList size={27} strokeWidth={1.5} aria-hidden /></div>
      <h3>{title}</h3>
      <p>{text}</p>
    </div>
  )
}

function Failure({ error }: { error: Error }) {
  const [title, text] = failureCopy[error instanceof SolveError ? error.kind : 'execution']
  return <div className="plan plan-error" role="alert"><h3>{title}</h3><p>{text}</p></div>
}

function Result({ result, example }: { result: SolveResult; example: boolean }) {
  if (result.kind === 'infeasible') {
    return <><h3>No assignment fits all the rules</h3><p>Try making another person available or changing which jobs overlap.</p></>
  }
  if (result.kind === 'exact-json') {
    return (
      <>
        <h3>Assignment found</h3>
        <p>Some numbers are too large to display safely in the summary. The exact result is below.</p>
        <pre className="result-json" tabIndex={0} aria-label="Exact assignment JSON">{result.json}</pre>
      </>
    )
  }

  const { values, score, evaluated } = result.assignment
  return (
    <>
      <h3 className="plan-success"><Check size={20} aria-hidden /> {example ? 'All three jobs covered' : 'Assignment found'}</h3>
      <p>Every required rule was met.</p>
      <dl className="assignment-list">
        {values.map((value, index) => (
          <div key={index}>
            <dt>{(example && exampleJobs[index]) || `Decision ${index + 1}`}</dt>
            <dd>{(example && exampleTechnicians.get(value)) || value}</dd>
          </div>
        ))}
      </dl>
      <details className="advanced-panel">
        <summary><Braces size={16} aria-hidden /> Technical details</summary>
        <dl className="assignment-list">
          <div><dt>Hard score</dt><dd>{score.hard}</dd></div>
          <div><dt>Preference score</dt><dd>{score.soft}</dd></div>
          <div><dt>Combinations checked</dt><dd>{evaluated.toLocaleString()}</dd></div>
        </dl>
      </details>
    </>
  )
}
