# Midtown Manhattan street-traffic replay

One vehicle trip replays three states of the same directed OpenStreetMap street graph. The slowdown and closure are illustrative, not live inputs or records of historical events. Origin and destination are selected street nodes, not customer locations.

| Replay state | Route length | Modeled travel time | Decision |
| --- | ---: | ---: | --- |
| Baseline at 30 km/h | 451.9 m | 54.225 s | Use baseline path |
| Nine directed segments reduced to 8 km/h | 451.9 m | 82.298 s | Keep path; allow 28.073 s more |
| Same slowdown plus one directed edge closed | 1,232.5 m | 147.898 s | Baseline blocked; use detour |

Reproduce with Python 3.11+ and the pinned source XML and fixture under the repo-workspace sibling `orch-multicity-streets`:

```sh
python3 calculate.py
python3 verify.py
python3 build_operational.py
```

The display assets are `replay.geojson` (routes, event segments and endpoints) and `replay.json` (stable node IDs, event inputs, route metrics). The companion `operational-case.json` traces the result through operation, routing workstream, network, update log, decision and review evidence.

The calculator minimizes summed, integer-millisecond edge time using Dijkstra. A separate Bellman–Ford implementation derives the same optimal cost from the specified updates for each variant, verifies all output geometry on the pinned snapshot, and checks whether the old path remains open. The proof is for the clipped graph and this simplified cost model only. No live traffic, lane capacity, signal delay, real incident, legal clearance, navigation, or dispatch execution is implied.

Street geometry: [© OpenStreetMap contributors · ODbL 1.0](https://www.openstreetmap.org/copyright). Preserve attribution in every map.
