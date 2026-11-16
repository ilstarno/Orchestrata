import { readFileSync } from 'node:fs'
import assert from 'node:assert/strict'

const read = file => JSON.parse(readFileSync(new URL(file, import.meta.url)))
const { cases } = read('cases.json')
const { results } = read('results.json')
const expected = [
  {
    assigned: [['INC-1002','IAM-A',0,35],['INC-1001','NET-A',0,45],['INC-1003','APP-A',5,35],['INC-1004','NET-A',45,80],['INC-1005','IAM-A',35,65]],
    held: [['INC-1006','skills_unavailable'],['INC-1007','right_missing'],['INC-1008','workload_full']],
  },
  {
    assigned: [['CHG-2001','CHG-QA',10,30],['CHG-2002','CHG-NET',40,70],['CHG-2004','CHG-DB',55,95],['CHG-2003','CHG-QA',70,90],['CHG-2005','CHG-QA',95,120]],
    held: [['CHG-2006','approval_pending'],['CHG-2008','no_slot_before_deadline'],['CHG-2007','prerequisite_held']],
  },
  {
    assigned: [['TASK-3001','DUTY-DAY',0,35],['TASK-3002','DUTY-DAY',40,60],['TASK-3003','DUTY-NIGHT',65,90],['TASK-3004','DUTY-OBS',90,120],['TASK-3005','DUTY-NIGHT',100,140],['TASK-3006','DUTY-NIGHT',140,160]],
    held: [['TASK-3007','territory_uncovered'],['TASK-3008','workload_full']],
  },
]
assert.equal(cases.length, 3)
assert.equal(results.length, 3)

for (const [k, scenario] of cases.entries()) {
  const { id, assignments, held, summary } = results[k]
  assert.equal(id, scenario.id)
  assert.deepEqual(summary, { assigned: expected[k].assigned.length, held: expected[k].held.length, total: 8 })
  assert.deepEqual(assignments.map(a => [a.itemId,a.resourceId,a.startMinute,a.endMinute]), expected[k].assigned)
  assert.deepEqual(held.map(h => [h.itemId,h.code]), expected[k].held)
  const assigned = new Map(assignments.map(a => [a.itemId, a]))
  const holds = new Map(held.map(h => [h.itemId, h]))
  const allIds = new Set([...assigned.keys(), ...holds.keys()])
  assert.deepEqual(allIds, new Set(scenario.items.map(item => item.id)))
  for (const item of scenario.items) {
    const plan = assigned.get(item.id)
    const hold = holds.get(item.id)
    assert.notEqual(Boolean(plan), Boolean(hold))
    const allRegion = scenario.resources.filter(r => r.regions.includes(item.region))
    const allSkill = allRegion.filter(r => item.skills.every(skill => r.skills.includes(skill)))
    const allPermitted = allSkill.filter(r => r.rights.includes(item.right))
    if (hold) {
      const { code } = hold
      if (code === 'approval_pending') {
        assert.equal(item.right, 'stage_change')
        assert.notEqual(item.approval, 'sample-approved')
      }
      if (code === 'skills_unavailable') assert.ok(allRegion.length && !allSkill.length)
      if (code === 'territory_uncovered') assert.equal(allRegion.length, 0)
      if (code === 'right_missing') assert.ok(allSkill.length && !allPermitted.length)
      if (code === 'workload_full') {
        assert.ok(allPermitted.length)
        for (const r of allPermitted) assert.ok(r.currentLoad + assignments.filter(a => a.resourceId === r.id).length === r.capacity)
      }
      if (code === 'prerequisite_held') {
        assert.deepEqual(hold.prerequisiteIds, item.requires.filter(p => holds.has(p)))
        assert.ok(hold.prerequisiteIds.length)
      }
      if (code === 'no_slot_before_deadline') {
        assert.ok(allPermitted.length)
        for (const r of allPermitted) {
          const start = Math.max(item.window[0], r.window[0], ...item.requires.map(p => assigned.get(p).endMinute))
          const latest = Math.min(item.window[1], r.window[1]) - item.duration
          for (let t = start; t <= latest; ++t) {
            assert.ok(assignments.some(a => a.resourceId === r.id && a.startMinute < t + item.duration && a.endMinute > t))
          }
        }
      }
      continue
    }
    const r = allPermitted.find(r => r.id === plan.resourceId)
    assert.ok(r)
    if (item.right === 'stage_change') assert.equal(item.approval, 'sample-approved')
    assert.equal(plan.endMinute - plan.startMinute, item.duration)
    assert.ok(plan.startMinute >= Math.max(item.window[0], r.window[0]))
    assert.ok(plan.endMinute <= Math.min(item.window[1], r.window[1]))
    for (const dep of item.requires) assert.ok(assigned.get(dep).endMinute <= plan.startMinute)
    const coworkers = assignments.filter(a => a.resourceId === r.id)
    assert.ok(r.currentLoad + coworkers.length <= r.capacity)
    assert.ok(!coworkers.some(a => a.itemId !== item.id && a.startMinute < plan.endMinute && a.endMinute > plan.startMinute))
  }
}
console.log('Verified 3 ServiceNow drafts: 24 records, 16 assignments, and 8 explicit holds.')
