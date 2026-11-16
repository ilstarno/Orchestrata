import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { resolve } from 'node:path'

const cmp = (a, b) => a < b ? -1 : a > b ? 1 : 0
const reasons = {
  prerequisite_held: 'At least one required earlier step was held.',
  territory_uncovered: 'No candidate covers the record’s service region or sales territory.',
  skills_unavailable: 'No candidate in the region has all the required skills.',
  right_missing: 'No qualified candidate in the region has the required record or review permission.',
  workload_full: 'Every eligible candidate is at the modeled workload cap.',
  no_slot_before_deadline: 'No eligible candidate has an uninterrupted slot before the deadline.',
}

export function calculate({ id, resources, items }) {
  const ordered = items.toSorted((a,b) => b.priority - a.priority || a.window[1] - b.window[1] || cmp(a.id,b.id))
  const pending = new Map(ordered.map(i => [i.id, i]))
  const scheduled = new Map(), held = new Map()
  const calendar = new Map(resources.map(r => [r.id, []]))
  const hold = (item, code, details = {}) => held.set(item.id, { itemId: item.id, code, reason: reasons[code], ...details })
  if (pending.size !== items.length || calendar.size !== resources.length) throw new Error('Duplicate IDs')

  while (pending.size) {
    const item = [...pending.values()].find(i => i.requires.every(p => scheduled.has(p) || held.has(p)))
    if (!item) throw new Error('Unresolved prerequisite or dependency cycle')
    pending.delete(item.id)
    const blocked = item.requires.filter(p => held.has(p))
    if (blocked.length) {
      hold(item, 'prerequisite_held', { prerequisiteIds: blocked })
      continue
    }
    const region = resources.filter(r => r.regions.includes(item.region))
    const skills = region.filter(r => item.skills.every(s => r.skills.includes(s)))
    const authorized = skills.filter(r => r.rights.includes(item.right))
    const available = authorized.filter(r => r.currentLoad + calendar.get(r.id).length < r.capacity)
    const depEnd = Math.max(0, ...item.requires.map(p => scheduled.get(p).endMinute))
    const slots = available.flatMap(r => {
      let t = Math.max(item.window[0], r.window[0], depEnd)
      for (const booking of calendar.get(r.id)) {
        if (booking.startMinute >= t + item.duration) break
        t = Math.max(t, booking.endMinute)
      }
      const end = t + item.duration
      return end <= Math.min(item.window[1], r.window[1]) ?
        [{ itemId: item.id, resourceId: r.id, startMinute: t, endMinute: end }] : []
    }).toSorted((a,b) => a.endMinute - b.endMinute || cmp(a.resourceId,b.resourceId))
    if (slots.length) {
      const next = slots[0]
      calendar.get(next.resourceId).push(next)
      calendar.get(next.resourceId).sort((a,b) => a.startMinute - b.startMinute)
      scheduled.set(item.id, next)
    } else {
      const code = !region.length ? 'territory_uncovered' : !skills.length ? 'skills_unavailable' :
        !authorized.length ? 'right_missing' : !available.length ? 'workload_full' : 'no_slot_before_deadline'
      hold(item, code, { candidateResourceIds: authorized.map(r => r.id).sort(cmp) })
    }
  }
  return { id, assignments: [...scheduled.values()], held: [...held.values()],
    summary: { assigned: scheduled.size, held: held.size, total: pending.size + scheduled.size + held.size } }
}

if (process.argv[1] && fileURLToPath(import.meta.url) === resolve(process.argv[1])) {
  const { scope, cases } = JSON.parse(readFileSync(new URL('./cases.json', import.meta.url), 'utf8'))
  console.log(JSON.stringify({ schemaVersion: 1, scope, results: cases.map(calculate) }, null, 2))
}
