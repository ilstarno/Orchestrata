# Product and terminology

Orchestrata is a **domain-independent constraint optimization platform**. A caller defines decision variables, their possible values, hard rules, and objectives. The engine evaluates candidate solutions without requiring a solver implementation for each industry.

Potential applications include assignment, scheduling, routing, resource allocation, packing, sequencing, production planning, and other finite decision problems. These are **target use cases**, not claims that all are currently implemented.

- **Planning variable:** a decision whose value is selected from a domain.
- **Domain:** the allowed values for a variable.
- **Hard constraint:** a rule that must be satisfied for a feasible result.
- **Soft constraint:** a preference used to compare solutions.
- **Solution:** assignments and a verified score.
- **Reference scorer:** full recomputation used to test optimized scoring implementations.

## Implemented

The Rust solver currently supports integer domains, all-different constraints, offset inequalities, and preferred values. It exhaustively evaluates small problems with lexicographic hard/soft scoring.

## Planned

A typed constraint language and compiler, construction heuristics, local search, incremental scoring, shadow variables, list and chained variables, independent validation, durable jobs, and integrations. Use unrelated problem fixtures to demonstrate that the solver remains domain-independent.

## Invariants

- Never describe a negative hard score as feasible.
- Do not claim optimality for incomplete search.
- Industry terminology belongs in examples or adapters, not in the generic solver.
