# Transactional printing: from intake to dispatch

Run `python3 replay.py` in this folder to check the results or `python3 replay.py --write` to regenerate them. No dependencies, equipment, carrier account, recipient data, or service connections are required.

This stress-case shift deliberately alternates three kinds of mail: 24 invoices (2 sheets), 24 statements (4 sheets), and 24 notices (3 sheets). The examples have three paper and envelope combinations. Both plans receive 72 records, hold six for unavailable templates or incomplete inputs, and release the same 66 pieces. Both detect exactly seven deliberately injected sheet faults. No mail is printed or sent.

| Stage | In both plans | Proposed change |
| --- | --- | --- |
| Receive and validate | Check the record and approved template revision; stop incomplete or unsupported work. | No change to validation or release eligibility. |
| Compose | Derive the expected sheet and envelope recipe. | Keep the same compositions. |
| Batch | Start a run for a contiguous paper and envelope combination. | Group compatible pieces within this one common pickup window. |
| Print | Count good sheets and production setup sheets. | Fewer setups as compatible runs move together. |
| Insert and scan | Check each internal piece reference against its expected packet. | Catch a faulty component before sealing. Replace the affected sheet and rescan the packet. |
| Inspect and reprint | Reconcile physical output. Late failures need a whole-piece reprint in the baseline. | Quality reviews scan evidence. Final review time falls because recovery happened at insert. |
| Dispatch and reconcile | One released manifest entry per accepted piece; holds remain separate. | Same release controls. Nothing leaves the sample. |

Within a single sequential line, the modeled shift time goes from 162 to 77 minutes. Setup stock and spoilage go from 156 to 13 sheets. Replacement sheets go from 24 to 7. The model assumes two setup sheets and one setup minute per production run, a fixed press and handling throughput, and a two- or four-minute recovery allowance by detection point that includes replacement printing and rechecks. Real shifts may overlap tasks and incur different setup, transport, staffing and postal cutoffs; pilot against historical site data before using these numbers. The 66-to-3 run comparison is intentionally a stress baseline, not a general estimate.

Published data includes the source records, both batch plans, held records, each piece's replacement disposition and one matched release trace. Internal sample piece IDs are not postal barcodes. In the United States, actual [USPS Intelligent Mail barcodes](https://postalpro.usps.com/mailing/intelligent-mail-barcode/) and [barcode standards](https://pe.usps.com/TEXT/DMM300/204.htm) are separate from print-line identity and require the mailer's local checks.
