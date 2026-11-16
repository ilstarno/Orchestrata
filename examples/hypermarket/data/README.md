# Hypermarket stockflow worked case

Sixty illustrative products fill six compatible stock-handling zones, each with ten warehouse bins and ten shop-floor shelf slots. Both a regular week and a promotion week use the same feasible current layout. A recommendation reorders products within each location group for each demand profile.

```sh
python3 calculate.py
python3 calculate.py --check
python3 verify.py
```

The calculator checks one SKU per position, zone and temperature match, case mass/height bounds, and shelf fixture compatibility. Within each modeled zone, all pairs meet the scoped placement constraints. The fixture does not check stock rotation, aisle obstruction, accessibility, cross-zone moves, multi-item journeys, display adjacency or labor costs. The product group is held fixed.

The objective is the sum over products of weekly round-trip distance multiplied by modeled pick trips for warehouse slots or replenishment trips for floor slots. Distance follows an illustrative rectilinear grid from a fixed origin for each stage, not a measured path. The floor metric is staff replenishment distance, not customer traffic. Actual travel, sales, safety, or financial outcomes are not claimed; the case does not use a connected store, live inventory, or the Orchestrata engine.

`inputs.json` keeps profile, slots and scenario volumes; `visual.json` exposes the two layouts and baseline/recommended placements; `operational-case.json` contains SKU-level evidence, scenario metrics and policy checks. The separate verifier recomputes costs and tests the exact minimum for each ten-item zone via subset dynamic programming.
