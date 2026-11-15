import { assignmentExample, exampleJobs, exampleTechnicians } from '../../shared/examples/assignment'

export function ExampleOverview() {
  return (
    <div className="example-overview">
      <p>Three jobs start at the same time. Each person can do one job.</p>
      <dl className="assignment-list">
        {assignmentExample.variables.map(({ domain }, index) => (
          <div key={exampleJobs[index]}>
            <dt>{exampleJobs[index]}</dt>
            <dd>{domain.map(id => exampleTechnicians.get(id)).join(' or ')}</dd>
          </div>
        ))}
      </dl>
      <p className="rule-note"><strong>Preference:</strong> Give the lift repair to Alex if all three jobs can still be covered.</p>
    </div>
  )
}
