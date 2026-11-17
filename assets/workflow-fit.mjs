const clean = value => String(value ?? '').normalize('NFKD').replace(/\p{M}/gu, '').toLowerCase().replace(/[^\p{L}\p{N}]+/gu, ' ').trim()
const plain = new Set(['the','are','for','not','and','this','with','team','unknown','match','example','open'])
const terms = text => new Set(clean(text).split(' ').filter(word => word.length > 2 && !plain.has(word)))

export function chooseWorkflow(cases, { sector, team, region }) {
  const selectedSector = clean(sector)
  const selectedTeam = clean(team)
  const teamWords = terms(team)
  const selectedRegion = String(region || '').toUpperCase()
  const restricted = Boolean(selectedSector && cases.some(item => clean(item?.sector) === selectedSector))
  return cases.flatMap(item => {
    if (!item || typeof item.url !== 'string' || typeof item.title !== 'string' || !/^\/Orchestrata\/(?:[a-z0-9-]+\/)+$/u.test(item.url)) return []
    const sectorMatch = Boolean(selectedSector && clean(item.sector) === selectedSector)
    if (restricted && !sectorMatch) return []
    const teamMatch = Boolean(selectedTeam && clean(item.team) === selectedTeam)
    const tagged = terms(item.keywords)
    const hits = [...teamWords].filter(term => tagged.has(term)).length
    if (!sectorMatch && !teamMatch && hits === 0) return []
    const regionMatch = selectedRegion && Array.isArray(item.regions) && item.regions.includes(selectedRegion)
    // Region can break a tie when a relevant local example exists. It adds no rules.
    const title = terms(item.title)
    const titleHits = teamMatch ? 0 : [...teamWords].filter(term => title.has(term)).length
    return [{ item, score: (sectorMatch ? 14 : 0) + (teamMatch ? 8 : 0) + Math.min(hits, 4) + 2 * Math.min(titleHits, 4) + (regionMatch ? 3 : 0) }]
  }).sort((a, b) => b.score - a.score || a.item.title.localeCompare(b.item.title, 'en'))[0]?.item ?? null
}
