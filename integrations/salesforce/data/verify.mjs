import { readFileSync } from 'node:fs'
import assert from 'node:assert/strict'

const read = file => JSON.parse(readFileSync(new URL(file, import.meta.url)))
const { cases } = read('cases.json')
const { results } = read('results.json')
const expected = [
  {
    assigned: [['CASE-101','AGENT-EMEA-1',0,25],['CASE-102','AGENT-EMEA-2',15,35],['CASE-104','AGENT-EMEA-1',25,40]],
    held: [['CASE-106','no_slot_before_deadline'],['CASE-103','right_missing'],['CASE-105','workload_full']],
  },
  {
    assigned: [['OPP-201','SELLER-DACH',0,45],['OPP-203','SELLER-FR',10,40],['OPP-202','DESK-EU',60,95]],
    held: [['OPP-204','no_slot_before_deadline'],['OPP-205','prerequisite_held'],['OPP-206','territory_uncovered']],
  },
]
assert.equal(cases.length, 2)
assert.equal(results.length, 2)

for (const [k, scenario] of cases.entries()) {
  const { id, assignments, held, summary } = results[k]
  assert.equal(id, scenario.id)
  assert.deepEqual(summary, { assigned: 3, held: 3, total: 6 })
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
    assert.equal(plan.endMinute - plan.startMinute, item.duration)
    assert.ok(plan.startMinute >= Math.max(item.window[0], r.window[0]))
    assert.ok(plan.endMinute <= Math.min(item.window[1], r.window[1]))
    for (const dep of item.requires) assert.ok(assigned.get(dep).endMinute <= plan.startMinute)
    const coworkers = assignments.filter(a => a.resourceId === r.id)
    assert.ok(r.currentLoad + coworkers.length <= r.capacity)
    assert.ok(!coworkers.some(a => a.itemId !== item.id && a.startMinute < plan.endMinute && a.endMinute > plan.startMinute))
  }
}
console.log('Verified 2 Salesforce drafts: 12 items, 6 assignments, and 6 explicit holds.')
