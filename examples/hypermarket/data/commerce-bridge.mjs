import { open } from 'node:fs/promises'
import { pathToFileURL } from 'node:url'

const maxSize = 2 * 1024 * 1024
const key = value => typeof value === 'string' && /^[A-Za-z0-9._:-]{1,70}$/.test(value)
const count = value => Number.isSafeInteger(value) && value >= 0
const catalogRow = row => row && key(row.sku) && key(row.variantId) && count(row.sellableUnits)
const warehouseRow = row => row && key(row.sku) && key(row.locationId) && count(row.physicalUnits)

export function mapCatalog(exported) {
  if (!exported || !Array.isArray(exported.storefront) || !Array.isArray(exported.warehouse) ||
      exported.storefront.length > 10_000 || exported.warehouse.length > 10_000) throw Error('Expected bounded storefront and warehouse exports')
  const products = new Map(), variants = new Set(), positions = new Map(), seen = new Set()
  for (const row of exported.storefront) {
    if (!catalogRow(row) || products.has(row.sku) || variants.has(row.variantId)) throw Error('Invalid or duplicate storefront SKU/variant')
    products.set(row.sku, { variantId: row.variantId, sellableUnits: row.sellableUnits })
    variants.add(row.variantId)
  }
  for (const row of exported.warehouse) {
    if (!warehouseRow(row)) throw Error('Invalid warehouse row')
    const id = JSON.stringify([row.sku, row.locationId])
    if (seen.has(id)) throw Error('Duplicate SKU/location')
    seen.add(id)
    if (!positions.has(row.sku)) positions.set(row.sku, [])
    positions.get(row.sku).push({ locationId: row.locationId, physicalUnits: row.physicalUnits })
  }
  const matched = [], unresolved = []
  for (const sku of [...new Set([...products.keys(), ...positions.keys()])].sort()) {
    const commerce = products.get(sku), physicalStock = positions.get(sku)
    if (commerce && physicalStock) matched.push({ sku, storefront: commerce, physicalStock })
    else unresolved.push({ sku, reason: commerce ? 'NO_WAREHOUSE_MAPPING' : 'NO_STOREFRONT_MAPPING' })
  }
  return { matched, unresolved }
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const source = await open(process.argv[2], 'r')
  try {
    const buffer = Buffer.alloc(maxSize + 1)
    const { bytesRead } = await source.read(buffer, 0, buffer.length, 0)
    if (bytesRead > maxSize) throw Error('Export is too large for this sample')
    const input = JSON.parse(buffer.subarray(0, bytesRead).toString('utf8'))
    process.stdout.write(JSON.stringify(mapCatalog(input), null, 2) + '\n')
  } finally {
    await source.close()
  }
}
