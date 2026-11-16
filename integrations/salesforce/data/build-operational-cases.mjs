import assert from 'node:assert/strict'
import { readFileSync } from 'node:fs'

const read = name => JSON.parse(readFileSync(new URL(name, import.meta.url), 'utf8'))
const sample = read('cases.json')
const { results } = read('results.json')
const profiles = {
  'service-cloud-queue': ['Customer operations', 'Customer support', 'Service Cloud', 'Case routing', 'Response commitments'],
  'sales-cloud-renewal': ['Revenue operations', 'Account management', 'Sales Cloud', 'Renewal ownership and review', 'Renewal deadlines'],
  'servicenow-incident-triage': ['Technology operations', 'Service reliability', 'IT Service Management', 'Incident triage', 'Incident response windows'],
  'servicenow-change-calendar': ['Technology operations', 'Release operations', 'Change Enablement', 'Maintenance scheduling', 'Controlled change windows'],
  'servicenow-on-call-handoff': ['Technology operations', 'Service ownership', 'On-call operations', 'Shift handoff', 'Coverage continuity'],
}

const describe = (scenario, result) => {
  const [unit, functionName, productArea, workstreamName, mandate] = profiles[scenario.id]
  const lookup = new Map(scenario.items.map(item => [item.id, item]))
  const people = new Map(scenario.resources.map(resource => [resource.id, resource]))
  const assigned = new Map(result.assignments.map(plan => [plan.itemId, plan]))
  const held = new Map(result.held.map(hold => [hold.itemId, hold]))
  const itemsFor = id => result.assignments.filter(plan => plan.resourceId === id)
  const candidateProfile = resource => ({
    profileId: resource.id,
    operatorRole: resource.role,
    capabilityMatrix: {
      configuredSkills: resource.skills,
      ownershipScopes: resource.regions,
      permittedActivities: resource.rights,
    },
    staffingEnvelope: {
      availableFromMinute: resource.window[0],
      availableUntilMinute: resource.window[1],
      maximumWorkItems: resource.capacity,
      workAlreadyCounted: resource.currentLoad,
      newlyPlannedWorkItems: itemsFor(resource.id).length,
      remainingPlanningSlots: resource.capacity - resource.currentLoad - itemsFor(resource.id).length,
    },
  })
  const requestBasis = item => ({
    displayTitle: item.title,
    policyScope: item.region,
    requiredCapabilities: item.skills,
    requiredActivity: item.right,
    priorityRank: item.priority,
    estimatedEffortMinutes: item.duration,
    releaseMinute: item.window[0],
    latestFinishMinute: item.window[1],
    upstreamRecordIds: item.requires,
    sampleApprovalState: item.approval ?? null,
  })
  const planned = result.assignments.map(plan => {
    const item = lookup.get(plan.itemId)
    const person = people.get(plan.resourceId)
    const bookings = itemsFor(person.id)
    const predecessors = item.requires.map(recordId => ({ recordId, projectedFinishMinute: assigned.get(recordId).endMinute }))
    const checks = {
      authority: {
        requestedActivity: item.right,
        matchingAuthority: person.rights.find(right => right === item.right) ?? null,
        scopeCovered: person.regions.includes(item.region),
      },
      capability: {
        missingSkills: item.skills.filter(skill => !person.skills.includes(skill)),
        operatorSkills: person.skills,
      },
      timeWindow: {
        effectiveStartMinute: Math.max(item.window[0], person.window[0], ...predecessors.map(p => p.projectedFinishMinute)),
        deadlineMarginMinutes: Math.min(item.window[1], person.window[1]) - plan.endMinute,
      },
      capacity: {
        loadWithDraft: person.currentLoad + bookings.length,
        slotLimit: person.capacity,
      },
      dependency: {
        predecessorCompletions: predecessors,
        readyAtStart: predecessors.every(p => p.projectedFinishMinute <= plan.startMinute),
      },
    }
    assert.equal(checks.authority.matchingAuthority, item.right)
    assert.ok(checks.authority.scopeCovered && !checks.capability.missingSkills.length)
    assert.ok(plan.startMinute >= checks.timeWindow.effectiveStartMinute && checks.timeWindow.deadlineMarginMinutes >= 0)
    assert.ok(checks.capacity.loadWithDraft <= checks.capacity.slotLimit && checks.dependency.readyAtStart)
    assert.equal(plan.endMinute - plan.startMinute, item.duration)
    return {
      issueReference: plan.itemId,
      requestBasis: requestBasis(item),
      proposedCommitment: {
        proposedOperatorId: plan.resourceId,
        plannedStartMinute: plan.startMinute,
        plannedEndMinute: plan.endMinute,
        actualExecution: false,
        reviewBasis: { eligibilityChecks: checks },
      },
    }
  })
  const exceptions = result.held.map(hold => {
    const request = lookup.get(hold.itemId)
    const inScope = scenario.resources.filter(r => r.regions.includes(request.region))
    const skilled = inScope.filter(r => request.skills.every(skill => r.skills.includes(skill)))
    const authorized = skilled.filter(r => r.rights.includes(request.right))
    return {
      issueReference: hold.itemId,
      requestBasis: requestBasis(request),
      reviewRequired: true,
      reportedHold: {
        reasonCode: hold.code,
        reasonText: hold.reason,
        supportingCandidateMatrix: {
          scopeCandidateIds: inScope.map(r => r.id),
          skillCandidateIds: skilled.map(r => r.id),
          authorizedCandidateIds: authorized.map(r => r.id),
        },
        blockingDependencies: (hold.prerequisiteIds ?? []).map(recordId => ({
          recordId,
          upstreamHoldCode: held.get(recordId).code,
        })),
      },
    }
  })
  assert.equal(planned.length + exceptions.length, lookup.size)
  assert.equal(assigned.size, result.summary.assigned)
  assert.equal(held.size, result.summary.held)

  return {
    id: scenario.id,
    title: workstreamName,
    dataClassification: 'Illustrative planning data',
    sourceEvidence: {
      scenarioInput: 'cases.json',
      calculationOutput: 'results.json',
      reproducibilityCheck: 'verify.mjs',
      sourceCaseId: scenario.id,
      liveSystemConnection: false,
    },
    operatingOrganization: {
      organizationRole: 'Example operator of the planning process',
      businessUnit: {
        businessUnitName: unit,
        serviceFunction: {
          functionName,
          productArea,
          workstream: {
            workstreamName,
            mandate,
            narrative: scenario.context,
            operatingPolicy: {
              priorityRule: scenario.policy,
              timeBasis: 'Integer minutes from illustrative plan start',
              planIsOperationalApproval: false,
            },
            planningRun: {
              demandCount: result.summary.total,
              proposedCount: result.summary.assigned,
              exceptionCount: result.summary.held,
              resourceRoster: scenario.resources.map(candidateProfile),
              decisionLedger: { proposedAssignments: planned, heldForReview: exceptions },
            },
          },
        },
      },
    },
  }
}

export function profile(value, levels = 0, keys = new Set()) {
  if (!value || typeof value !== 'object') return levels
  if (Array.isArray(value)) return Math.max(levels, ...value.map(v => profile(v, levels, keys)))
  for (const key of Object.keys(value)) keys.add(key)
  return Math.max(levels + 1, ...Object.values(value).map(v => profile(v, levels + 1, keys)))
}

const cases = sample.cases.map(scenario => describe(scenario, results.find(result => result.id === scenario.id)))
for (const operationalCase of cases) {
  const keys = new Set()
  const depth = profile(operationalCase, 0, keys)
  assert.ok(depth >= 10 && keys.size >= 50, `${operationalCase.id}: ${depth} objects, ${keys.size} keys`)
  if (process.argv.includes('--check')) process.stderr.write(`${operationalCase.id}: ${depth} object levels, ${keys.size} properties verified\n`)
}
const document = JSON.stringify({ schemaVersion: 1, scope: sample.scope, cases }, null, 2) + '\n'
if (process.argv.includes('--check')) assert.equal(document, readFileSync(new URL('operational-cases.json', import.meta.url), 'utf8'))
else process.stdout.write(document)
