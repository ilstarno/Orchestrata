# Orchestrata brain

This directory records the durable technical knowledge needed to maintain Orchestrata. The README explains the product; this directory explains the implementation, its boundaries, and why decisions were made.

## Start here

- [Product and vocabulary](knowledge/product/index.md)
- [Solver](knowledge/solver/index.md)
- [Backend](knowledge/backend/index.md)
- [Frontend](knowledge/frontend/index.md)
- [Data and connectors](knowledge/data/index.md)
- [Architecture decisions](decisions/index.md)

## Maintenance

Read the relevant page before changing a subsystem. Update the existing page when an invariant, data flow, or operational limitation changes. Add a new page only for a new topic. Record hard-to-reverse decisions in `decisions/`.

Keep verified behavior separate from planned work. Do not record ephemeral debugging notes or duplicate the README.
