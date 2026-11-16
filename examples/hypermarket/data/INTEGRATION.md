# Connect a hypermarket's commerce, warehouse and shelf data

The interactive example changes placement for 60 SKUs across six zones. It minimizes modeled staff travel per projected pick or refill while respecting the sample slot compatibility rules. It does not calculate sales lift, connect to a live store, change inventory, or model customer paths. Product placement that targets sales or basket size should be evaluated separately by comparable stores or time blocks, accounting for promotions, in-stock rate, price, and seasonality.

## Keep the four inputs separate

| System | Read for planning | Check before use |
| --- | --- | --- |
| Storefront or commerce platform | Product variants, SKU, category, barcode, promotion and sales data in aggregate | A variant ID is not a storage location. Available-to-sell is not physical shelf stock. Use a server-side export for anything private. |
| Warehouse management | Pick bins, unit of measure, rack capacity, stock lots and expiry, velocity, moves | Keep lot and handling constraints with their owner. Check expiry, cold chain, segregation and aisle restrictions with warehouse staff. |
| Store inventory | Backroom/on-shelf stock, deliveries, shelf replenishment tasks | Reconcile on-hand counts to physical checks and do not interpret online reservations as shelf shortages. |
| Planogram / fixture data | Fixture positions, usable dimensions, shelf weight, facings, adjacency and accessibility guides | A selling fixture and a warehouse slot have different constraints. Validate safety, signage, stockout risk, and service aisle accessibility before a move. |

## Website import, without browser credentials

Export a small catalog and WMS item crosswalk to a protected job. [The local bridge](commerce-bridge.mjs) accepts a [sample export](commerce-sample.json), joins on an explicit SKU, and reports separately the SKU mapping, physical location stock and any unresolved items. Its output contains no customer identifiers. Run `node commerce-bridge.mjs commerce-sample.json`. It does not fetch credentials, scrape storefront pages, or write inventory. The 60-SKU layout replay runs separately; it uses the placement data in [inputs.json](inputs.json).

For a pilot, compare pick and replenishment walking time, move count and on-shelf availability. Keep checkout conversion or basket value as separate, controlled measurements. Approve WMS moves and planogram updates in the responsible systems, then reconcile against scans and stock counts.

## Official references

- [Oracle WMS reslotting for demand and location rank](https://docs.oracle.com/en/cloud/saas/readiness/logistics/25d/wms25d/25D-wms-wn-f40864.htm)
- [SAP EWM slotting](https://help.sap.com/docs/SAP_EXTENDED_WAREHOUSE_MANAGEMENT/3d97bec9bf1649099384bb8167df3cf2/4bc46aed5a304b41e10000000a42189e.html)
- [Oracle Retail Shelf Space Optimization](https://docs.oracle.com/en/industries/retail/assortment-space-optimization/25.1.201.0/spoug/assortment-and-space-optimization.htm)
- [Oracle Retail shop-floor and backroom inventory services](https://docs.oracle.com/en/industries/retail/store-inventory-op-cloud/25.1.401.0/reiob/rest-web-services.htm)
- [Shopify ProductVariant](https://shopify.dev/docs/api/admin-graphql/2026-07/objects/ProductVariant) and [InventoryLevel](https://shopify.dev/docs/api/admin-graphql/2026-07/queries/inventoryLevel)
- [GS1 Logistic Label Guideline](https://ref.gs1.org/guidelines/logistic-label/): GTIN, lot, expiry and shipment-unit identifiers are distinct
