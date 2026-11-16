A recipe describes a small planning scenario and the fields it uses. The runnable examples have fictional data. Read the description, run the included JSON model, and compare the assignments with the expected result. The examples test specific decisions; they do not replace your scheduling or dispatch system.

| Example | What you can inspect | What it does not establish |
| --- | --- | --- |
| Field-service technician assignment | Match overlapping appointments to separate technicians from a list of eligible people. | Live dispatch, road routing, or a full-day staffing plan. |
| Two-store staffing what-if | Compare the baseline plan with a callout across two stores while keeping a fixed shift. | Forecasting, calendar integration, or labor compliance. |
| Fleet pickup and delivery assignment | Match jobs to vehicles that can carry the job's load and cover its time window. | Construction of vehicle routes or cumulative multi-stop capacity. |
| Airport gate assignment | Place overlapping flights at compatible available gates. | Live arrivals, gate systems, or full airport planning. |
| Call-center staffing | Cover concurrent queues using basic language and skill eligibility. | Forecasting call volume, service levels, or live workforce management. |

## Run a catalog fixture

With a checkout of the [draft recipe import](https://github.com/ilstarno/orchestrata-recipes/pull/1) and a built `orchestrata-cli` on your path, run from the recipe repository root:

```sh
orchestrata-cli recipes/examples/fleet-pickup-delivery/problem.json
```

The checked-in result has values `[1, 2, 0]`, with zero hard and soft penalties. Read the accompanying `inputs.json` to see the business inputs and `expected.json` to compare the result. The XML describes the scenario; the CLI receives the normalized JSON model. The standalone recipe import branch is source material, not a versioned catalog release.

Use the [assignment guide](../evaluate/) to run a smaller example without the catalog. Use a catalog example as a starting point: choose a decision, write down the allowed options, and model only rules you can verify. Do not send real customer or employee data into a local preview. Synthetic fleet workloads can generate repeatable stop inputs; the dataset generator does not route vehicles or estimate travel time.
