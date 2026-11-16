import { strict as assert } from 'node:assert'
import { readFileSync } from 'node:fs'
import { calculate } from './calculate.mjs'

const roster = JSON.parse(readFileSync(new URL('./stress-roster.json', import.meta.url)))
const result = calculate(roster)
assert.deepEqual(calculate(roster), result, 'deterministic replay')
const tasks = new Map(roster.items.map(i => [i.id, i]))
const staff = new Map(roster.resources.map(i => [i.id, i]))
const assignments = new Map(result.assignments.map(i => [i.itemId, i]))
const held = new Map(result.held.map(i => [i.itemId, i]))
assert.ok(tasks.size >= 50)
assert.equal(tasks.size, assignments.size + held.size)
for (const [id, item] of tasks) {
  const slot = assignments.get(id)
  const pause = held.get(id)
  assert.ok(!!slot !== !!pause, id)
  if (!slot) continue
  const person = staff.get(slot.resourceId)
  assert.ok(person && item.skills.every(skill => person.skills.includes(skill)), id)
  assert.notEqual(item.approval, 'pending', id)
  assert.equal(slot.endMinute - slot.startMinute, item.duration, id)
  for (const w of [person.window, item.window]) assert.ok(slot.startMinute >= w[0] && slot.endMinute <= w[1], id)
  for (const dependency of item.requires) {
    assert.ok(assignments.has(dependency), id)
    assert.ok(assignments.get(dependency).endMinute <= slot.startMinute, id)
  }
}
for (const resource of staff.values()) {
  const slots = result.assignments.filter(slot => slot.resourceId === resource.id).sort((a, b) => a.startMinute - b.startMinute)
  for (let i = 1; i < slots.length; i++) assert.ok(slots[i - 1].endMinute <= slots[i].startMinute, resource.id)
}
assert.equal(held.get('CHG-204').code, 'approval_pending')
assert.equal(held.get('CHG-205').code, 'prerequisite_held')
assert.equal(held.get('SEC-205').code, 'no_qualified_resource')
assert.equal(held.get('WAN-206').code, 'no_slot_before_deadline')

console.log(JSON.stringify({ ...result.summary, staff: staff.size, queues: new Set(roster.items.map(i => i.queue)).size,
  holdReasons: Object.fromEntries([...new Set([...held.values()].map(x => x.code))].map(code => [
    code, [...held.values()].filter(x => x.code === code).length,
  ])) }))
