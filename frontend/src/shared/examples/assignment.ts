export const assignmentExample = {
  variables: [
    { name: 'resource_a', domain: [0, 1] },
    { name: 'resource_b', domain: [0, 1] },
  ],
  constraints: [
    { kind: 'all_different', variables: [0, 1], hard_weight: 1 },
    { kind: 'prefer_value', variable: 0, value: 1, soft_weight: 3 },
  ],
}
