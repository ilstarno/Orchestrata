export const exampleJobs = ['Lift repair', 'Safety inspection', 'Preventive maintenance']
export const exampleTechnicians = new Map([
  [2, 'Sam'],
  [3, 'Noor'],
  [8, 'Alex'],
])

export const assignmentExample = {
  variables: [
    { name: 'lift_repair', domain: [8, 3] },
    { name: 'safety_inspection', domain: [8, 2] },
    { name: 'preventive_maintenance', domain: [3, 2] },
  ],
  constraints: [
    { kind: 'all_different', variables: [0, 1, 2], hard_weight: 1 },
    { kind: 'prefer_value', variable: 0, value: 8, soft_weight: 5 },
  ],
}

export const exampleDefinition = JSON.stringify(assignmentExample, null, 2)
