import { describe, expect, it } from 'vitest'
import { assignmentExample } from '../../shared/examples/assignment'

describe('assignment example', () => {
  it('has distinct variable names', () => {
    const names = assignmentExample.variables.map(variable => variable.name)
    expect(new Set(names).size).toBe(names.length)
  })

  it('only references existing variables', () => {
    const rule = assignmentExample.constraints.find(constraint => constraint.kind === 'all_different')
    expect(rule).toBeDefined()
    if (!rule?.variables) throw new Error('missing constraint variables')
    for (const index of rule.variables) {
      expect(index).toBeGreaterThanOrEqual(0)
      expect(index).toBeLessThan(assignmentExample.variables.length)
    }
  })
})
