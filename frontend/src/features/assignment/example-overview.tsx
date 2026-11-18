import { Checkbox } from "react-aria-components";
import { Check } from "lucide-react";
import {
  type Eligibility,
  exampleJobs,
  exampleTechnicians,
} from "../../shared/examples/assignment";

interface ExampleOverviewProps {
  eligibility: Eligibility;
  onChange: (job: number, technician: number, eligible: boolean) => void;
}

export function ExampleOverview({
  eligibility,
  onChange,
}: ExampleOverviewProps) {
  return (
    <div className="example-overview">
      <p>Three jobs start at the same time. Each person can do one job.</p>
      <dl className="assignment-list">
        {eligibility.map((domain, index) => (
          <div key={exampleJobs[index]}>
            <dt>{exampleJobs[index]}</dt>
            <dd>
              {domain.map((id) => exampleTechnicians.get(id)).join(" or ") ||
                "Nobody selected"}
            </dd>
          </div>
        ))}
      </dl>
      <p className="rule-note">
        <strong>Preference:</strong> Give the lift repair to Alex if all three
        jobs can still be covered.
      </p>

      <details className="advanced-panel eligibility-editor">
        <summary>Change who can take each job</summary>
        <p>
          Select the technicians who are qualified and available for each job.
        </p>
        {eligibility.map((domain, job) => (
          <fieldset className="eligibility-group" key={exampleJobs[job]}>
            <legend>{exampleJobs[job]}</legend>
            {[...exampleTechnicians].map(([id, name]) => (
              <Checkbox
                className="eligibility-option"
                key={id}
                isSelected={domain.includes(id)}
                onChange={(selected) => onChange(job, id, selected)}
              >
                <span className="eligibility-check" aria-hidden="true">
                  <Check size={14} />
                </span>
                {name}
              </Checkbox>
            ))}
          </fieldset>
        ))}
      </details>
    </div>
  );
}
