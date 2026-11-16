# Reproduce the Tirana example

Save [the geometry](tirana-courier-streets.geojson), [the calculated case](tirana-dispatch.json), [the calculator](calculate.py) and [the checker](verify.py) in the same folder. With Python 3.10 or newer:

```sh
python3 verify.py
python3 calculate.py > reproduced.json
cmp reproduced.json tirana-dispatch.json
```

The calculator uses fixed paths on the mapped street network. It checks order, capacity, fit, qualifications, shift and handoff windows, cross-team approval and the evidence recorded in this case. The checker independently reads the published data and tests the geometry and decisions. No account or network call is needed. The script does not call the engine or find new routes. Actual traffic, curb availability and legal clearance have not been verified.

Route geometry: © [OpenStreetMap contributors](https://www.openstreetmap.org/copyright), ODbL 1.0, from the Geofabrik Albania extract dated 13 November 2026. Scenario organizations, people, assets and requests are illustrative. Times are local to Tirana. Distance is in metres, duration in seconds, mass in grams and dimensions in millimetres.
