// Offline advisory preview. Never produce a provider command, address or resource binding.
import { readFile } from 'node:fs/promises'
import { pathToFileURL } from 'node:url'

const MAX_COUNT = 1_000_000
const MAX_ROWS = 64
const MAX_BYTES = 128 * 1024
const states = new Set(['healthy', 'stale', 'unknown', 'draining', 'down'])
const usage = 'Usage: node provider-preview.mjs <route53|gateway-api> <scenario> <market|region>'

function demand(condition, reason) {
  if (!condition) throw new Error(reason)
}

const identifier = value => typeof value === 'string' && /^[A-Za-z0-9][A-Za-z0-9_-]{0,47}$/.test(value)
const integer = (value, min = 0, max = MAX_COUNT) => Number.isSafeInteger(value) && value >= min && value <= max
const object = value => value !== null && typeof value === 'object' && !Array.isArray(value)
const sum = (rows, field) => rows.reduce((total, row) => total + row[field], 0)

function indexed(rows, key) {
  demand(Array.isArray(rows) && rows.length <= MAX_ROWS, 'invalid-row-count')
  const map = new Map()
  for (const row of rows) {
    demand(object(row) && identifier(row[key]) && !map.has(row[key]), 'invalid-or-duplicate-id')
    map.set(row[key], row)
  }
  return map
}

function checkRegion(region) {
  demand(states.has(region.status), 'invalid-region-health')
  demand(['demand', 'allocated', 'unserved', 'usableCapacity'].every(key => integer(region[key])), 'invalid-region-counters')
  demand(region.allocated + region.unserved === region.demand, 'demand-does-not-reconcile')
  const backends = [...indexed(region.backends, 'id').values()]
  for (const backend of backends) {
    demand(states.has(backend.status) && identifier(backend.failureDomain), 'invalid-backend-state')
    demand(['capacity', 'reserve', 'usable', 'allocated'].every(key => integer(backend[key])), 'invalid-backend-counters')
    demand(backend.reserve <= backend.capacity && backend.usable <= backend.capacity - backend.reserve
      && backend.allocated <= backend.usable, 'capacity-or-reserve-violation')
    demand((region.status === 'healthy' && backend.status === 'healthy') || backend.usable === 0,
      'unhealthy-backend-has-usable-capacity')
  }
  demand(sum(backends, 'allocated') === region.allocated && sum(backends, 'usable') === region.usableCapacity,
    'backend-totals-do-not-reconcile')
  const domains = new Set(backends.filter(row => row.usable > 0).map(row => row.failureDomain)).size
  demand(region.healthyDomains === domains, 'failure-domain-count-does-not-reconcile')
  return backends
}

const gcd = (a, b) => b === 0 ? a : gcd(b, a % b)
const scope = provider => ({ provider, advisoryOnly: true, deployable: false })

function route53(scenarioId, selector, scenario) {
  const group = indexed(scenario.dns, 'market').get(selector)
  demand(group, 'unknown-market')
  const { lookupWeight, ttlSeconds, unassignedWeight } = group
  demand(integer(lookupWeight, 100, 100) && integer(unassignedWeight, 0, lookupWeight)
    && integer(ttlSeconds, 0, 2_147_483_647), 'invalid-dns-budget-or-ttl')
  const routes = [...indexed(group.routes, 'region').values()]
  demand(routes.every(row => integer(row.weightPercent, 1, 100)), 'invalid-dns-weight')
  demand(sum(routes, 'weightPercent') + unassignedWeight === lookupWeight, 'dns-weights-do-not-reconcile')
  const regions = indexed(scenario.regions, 'region')
  const regionOutcomes = routes.map(({ region }) => {
    const record = regions.get(region)
    demand(record, 'unknown-dns-target')
    checkRegion(record)
    return { region, status: record.status, unservedRequestUnits: record.unserved }
  })
  const base = { ...scope('aws-route-53'), scenario: scenarioId, market: selector, lookupWeight, ttlSeconds,
    unassignedLookupWeight: unassignedWeight, downstreamRegions: regionOutcomes }
  const reason = routes.length === 0 ? 'no-eligible-dns-routing'
    : unassignedWeight > 0 ? 'unassigned-dns-weight'
    : regionOutcomes.some(row => row.status !== 'healthy') ? 'downstream-target-not-healthy'
    : regionOutcomes.some(row => row.unservedRequestUnits > 0) ? 'downstream-target-has-unserved-demand' : null
  if (reason) return { ...base, status: 'review-required', reason, relativeWeights: [] }
  return { ...base, status: 'advisory', relativeWeights: routes.map(({ region, weightPercent }) => ({ sampleTarget: region, weight: weightPercent })) }
}

function gateway(scenarioId, selector, scenario) {
  const region = indexed(scenario.regions, 'region').get(selector)
  demand(region, 'unknown-region')
  const backends = checkRegion(region).filter(row => row.allocated > 0)
  const base = { ...scope('kubernetes-gateway-api'), scenario: scenarioId, region: selector, demandRequestUnits: region.demand,
    allocatedRequestUnits: region.allocated, unservedRequestUnits: region.unserved }
  const reason = region.status !== 'healthy' ? 'region-not-healthy'
    : region.unserved > 0 ? 'unserved-demand-would-be-hidden'
    : backends.length === 0 ? 'no-allocated-backends' : null
  if (reason) return { ...base, status: 'review-required', reason, backendRefWeightSuggestions: [] }
  const divisor = backends.reduce((acc, row) => gcd(acc, row.allocated), 0)
  const backendRefWeightSuggestions = backends.map(({ id, allocated }) => ({
    sampleBackend: id, weight: allocated / divisor,
  }))
  return { ...base, status: 'advisory', backendRefWeightSuggestions }
}

export function providerPreview(dataset, mode, scenarioId, selector) {
  demand(mode === 'route53' || mode === 'gateway-api', 'unknown-preview-mode')
  demand(identifier(scenarioId) && identifier(selector), 'invalid-selector')
  demand(object(dataset), 'invalid-dataset')
  const scenario = indexed(dataset.scenarios, 'id').get(scenarioId)
  demand(scenario, 'unknown-scenario')
  return mode === 'route53' ? route53(scenarioId, selector, scenario) : gateway(scenarioId, selector, scenario)
}

async function main() {
  const args = process.argv.slice(2)
  demand(args.length === 3, usage)
  const bytes = await readFile(new URL('./visual.json', import.meta.url))
  demand(bytes.length <= MAX_BYTES, 'dataset-byte-limit')
  const result = providerPreview(JSON.parse(bytes), ...args)
  console.log(JSON.stringify(result, null, 2))
  if (result.status !== 'advisory') process.exitCode = 2
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  main().catch(error => {
    console.error('Preview refused: ' + error.message)
    process.exitCode = 1
  })
}
