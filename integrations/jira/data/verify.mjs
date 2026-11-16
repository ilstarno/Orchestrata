import assert from 'node:assert/strict'
import { readFileSync } from 'node:fs'

const read = name => JSON.parse(readFileSync(new URL(name, import.meta.url), 'utf8'))
const { cases } = read('cases.json')
const { results } = read('results.json')
const expected = {
  'jsm-queue': {
    assigned: [['HELP-101','AGENT-A',0,25],['HELP-103','AGENT-B',10,40]],
    held: [['HELP-102','no_slot_before_deadline'],['HELP-104','prerequisite_held'],['HELP-105','no_qualified_resource']],
  },
  'jira-release': {
    assigned: [['REL-201','ENG-1',0,100],['REL-202','QA-1',100,145],['REL-203','REL-1',180,210]],
    held: [['REL-204','no_slot_before_deadline'],['REL-205','prerequisite_held']],
  },
  'jsm-change-window': {
    assigned: [['OPS-301','ONCALL-1',0,40],['OPS-302','ONCALL-2',40,70],['OPS-303','ONCALL-1',90,110]],
    held: [['OPS-304','approval_pending'],['OPS-305','prerequisite_held']],
  },
}
assert.equal(cases.length, 3)
assert.equal(results.length, 3)
for (const [i, result] of results.entries()) {
  const { id, items, resources } = cases[i]
  assert.equal(result.id, id)
  assert.deepEqual(result.assignments.map(a => [a.itemId, a.resourceId, a.startMinute, a.endMinute]), expected[id].assigned)
  assert.deepEqual(result.held.map(a => [a.itemId, a.code]), expected[id].held)
  const draft = new Map(result.assignments.map(a => [a.itemId, a]))
  const holds = new Map(result.held.map(a => [a.itemId, a]))
  assert.deepEqual(result.summary, { assigned: draft.size, held: holds.size, total: items.length })
  for (const item of items) {
    const assignment = draft.get(item.id)
    const held = holds.get(item.id)
    assert.notEqual(Boolean(assignment), Boolean(held))
    if (held) {
      if (held.code === 'approval_pending') assert.equal(item.approval, 'pending')
      if (held.code === 'prerequisite_held') {
        assert.ok(held.prerequisiteIds.length)
        for (const p of held.prerequisiteIds) assert.ok(item.requires.includes(p) && holds.has(p))
      }
      if (held.code === 'no_qualified_resource') {
        assert.ok(!resources.some(r => item.skills.every(skill => r.skills.includes(skill))))
      }
      if (held.code === 'no_slot_before_deadline') {
        const rivals = resources.filter(r => item.skills.every(skill => r.skills.includes(skill)))
        assert.deepEqual(rivals.map(r => r.id), held.eligibleResourceIds)
        for (const r of rivals) {
          const first = Math.max(item.window[0], r.window[0], ...item.requires.map(p => draft.get(p).endMinute))
          const last = Math.min(item.window[1], r.window[1]) - item.duration
          for (let t = first; t <= last; t++) {
            assert.ok(result.assignments.some(a => a.resourceId === r.id && a.startMinute < t + item.duration && a.endMinute > t))
          }
        }
      }
      continue
    }
    const resource = resources.find(r => r.id === assignment.resourceId)
    assert.ok(item.skills.every(skill => resource.skills.includes(skill)))
    assert.notEqual(item.approval, 'pending')
    assert.equal(assignment.endMinute - assignment.startMinute, item.duration)
    for (const [start, end] of [item.window, resource.window]) {
      assert.ok(start <= assignment.startMinute && assignment.endMinute <= end)
    }
    for (const p of item.requires) assert.ok(draft.get(p).endMinute <= assignment.startMinute)
    assert.ok(!result.assignments.some(other => other.itemId !== item.id && other.resourceId === resource.id &&
      other.startMinute < assignment.endMinute && other.endMinute > assignment.startMinute))
  }
}
console.log('Verified 3 Jira drafts, 15 issues, 8 assignments, and 7 explicit holds.')
