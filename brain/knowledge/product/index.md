# Product and terminology

Orchestrata models planning problems with resources, decisions, constraints, and objectives. It aims to find assignments that satisfy mandatory rules and improve business objectives.

- **Planning variable:** a decision whose value is selected from a domain.
- **Domain:** allowed values for a variable.
- **Hard constraint:** a rule whose violation reduces the hard score; zero hard penalty indicates feasibility under the modeled rules.
- **Soft constraint:** a preference used to rank otherwise comparable solutions.
- **Solution:** chosen values plus evaluated score and execution metadata.
- **Reference scorer:** a full evaluator used as a correctness baseline.

## Current scope

The Rust solver currently handles integer domains, all-different constraints, offset inequalities, and preferred values. It enumerates assignments and chooses the best lexicographic hard/soft score. It is not yet a field-service, routing, or workforce scheduling engine.

## Planned scope

Typed constraint compilation, scalable search, independent verification, connectors, and controlled export are planned, not implemented.

## Gotchas

- A feasible solution is not necessarily globally optimal unless search is exhaustive and completes.
- A negative hard score must not be presented as a feasible schedule.
