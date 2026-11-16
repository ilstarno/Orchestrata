import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { resolve } from 'node:path'

const inputFile = new URL('./cases.json', import.meta.url)
const cmp = (a, b) => a < b ? -1 : a > b ? 1 : 0
const boundedWindow = (window) => Array.isArray(window) && window.length === 2 &&
  window.every(value => Number.isSafeInteger(value) && value >= 0 && value <= 10080) && window[0] < window[1]

export const reasons = {
  no_qualified_resource: 'No resource has all required skills.',
  no_slot_before_deadline: 'No qualified team member has enough time to finish this item before its deadline.',
  prerequisite_held: 'An earlier task must be scheduled before this item can start.',
  approval_pending: 'The required sample change approval is pending.',
}

export function calculate(scenario) {
  const { id, items, resources } = scenario
  if (items.length > 100 || resources.length > 32 ||
    new Set(items.map(i => i.id)).size !== items.length ||
    new Set(resources.map(r => r.id)).size !== resources.length ||
    items.some(i => !boundedWindow(i.window) || !Number.isSafeInteger(i.duration) || i.duration <= 0 ||
      i.duration > 10080 || !Number.isSafeInteger(i.priority) ||
      ![undefined, 'pending', 'sample-approved'].includes(i.approval) ||
      i.requires.some(p => p === i.id || !items.some(x => x.id === p))) ||
    resources.some(r => !boundedWindow(r.window))) throw new Error('Invalid scenario')

  const ordered = [...items].sort((a, b) => b.priority - a.priority || a.window[1] - b.window[1] || cmp(a.id, b.id))
  const pending = new Map(ordered.map(task => [task.id, task]))
  const assigned = new Map()
  const held = new Map()
  const calendar = new Map(resources.map(resource => [resource.id, []]))
  const hold = (item, code, details = {}) => held.set(item.id, { itemId: item.id, code, reason: reasons[code], ...details })

  while (pending.size) {
    const item = [...pending.values()].find(i => i.requires.every(p => assigned.has(p) || held.has(p)))
    if (!item) throw new Error('Dependency cycle')
    pending.delete(item.id)
    const blocked = item.requires.filter(dep => held.has(dep))
    if (blocked.length) {
      hold(item, 'prerequisite_held', { prerequisiteIds: blocked })
      continue
    }
    if (item.approval === 'pending') {
      hold(item, 'approval_pending')
      continue
    }
    const eligible = resources.filter(r => item.skills.every(skill => r.skills.includes(skill)))
    if (!eligible.length) {
      hold(item, 'no_qualified_resource')
      continue
    }
    const dependencyEnd = Math.max(0, ...item.requires.map(p => assigned.get(p).endMinute))
    const candidates = []
    for (const { id: resourceId, window } of eligible) {
      const booked = calendar.get(resourceId)
      let startMinute = Math.max(dependencyEnd, window[0], item.window[0])
      for (const slot of booked) {
        if (startMinute + item.duration <= slot.startMinute) break
        startMinute = Math.max(startMinute, slot.endMinute)
      }
      const endMinute = startMinute + item.duration
      if (endMinute <= Math.min(window[1], item.window[1])) candidates.push({ itemId: item.id, resourceId, startMinute, endMinute })
    }
    candidates.sort((a, b) => a.endMinute - b.endMinute || cmp(a.resourceId, b.resourceId))
    const best = candidates[0]
    if (!best) {
      hold(item, 'no_slot_before_deadline', { eligibleResourceIds: eligible.map(r => r.id).sort(cmp) })
      continue
    }
    calendar.get(best.resourceId).push(best)
    calendar.get(best.resourceId).sort((a, b) => a.startMinute - b.startMinute)
    assigned.set(item.id, best)
  }
  return { id, assignments: [...assigned.values()], held: [...held.values()], summary: {
    assigned: assigned.size, held: held.size, total: items.length,
  } }
}

if (process.argv[1] && fileURLToPath(import.meta.url) === resolve(process.argv[1])) {
  const source = JSON.parse(readFileSync(inputFile, 'utf8'))
  console.log(JSON.stringify({ schemaVersion: 1, scope: source.scope, results: source.cases.map(calculate) }, null, 2))
}
