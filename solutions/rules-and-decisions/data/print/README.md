# Transactional print: release evidence at the manifest

A correct packet can be matched to the local manifest. That result is not postal acceptance. This self-contained replay uses anonymous training identifiers for envelopes and component scans. It does not scan a physical print stream, read customer data, approve a shipment, or connect to any carrier.

## Worked example

Three envelopes are expected. Nine observations include a packet outside this batch, the wrong component, a repeated component, an incomplete packet, a corrected rescan, a second attempt to match the same envelope, reversed component order, and an unreadable packet. The replay records one internal match apiece for A and B after corrected rescans. Seven observations are held; C is still unresolved. An unreadable packet is unknown, not evidence of an incorrect match. A skipped rule is explicitly not evaluated. These are example observations; observed holds are not seven distinct pieces.

| Control | What the replay checks | Authority |
| --- | --- | --- |
| Batch enrollment | The envelope is on the local manifest. | Sample facility policy 1.0 |
| Duplicate match | The envelope has not already matched in the same run. | Sample facility policy 1.0 |
| Readability | A scanner observation includes ordered component IDs. Hold if unreadable. | Sample facility policy 1.0 |
| Packet reconciliation | Component IDs occur once and in the expected sequence. | Sample facility policy 1.0 |
| External acceptance | Not assessed. No enrollment record, Intelligent Mail barcode, electronic documentation, carrier observations, or physical mail. | [USPS Domestic Mail Manual 705 §22](https://pe.usps.com/text/DMM300/705.htm), applicable only within that carrier program |

USPS [Seamless Acceptance](https://postalpro.usps.com/mailing/seamless-acceptance) reconciles eligible mailers' electronic documentation with sampling and scans of Intelligent Mail barcodes. The [participation resources](https://postalpro.usps.com/resources/mailing/seamless-acceptance) describe eligibility, enrollment, and preparation. These are **carrier-program references**, not this replay's facility policy, a determination of legal obligations, or evidence that an internal identifier is a valid postal barcode. A carrier verdict needs applicable current program evidence and an actual carrier result.

## Reproduce

Run from this directory with Python 3.11 or newer; no packages or network connection are required:

```sh
python3 -B replay.py
python3 -B verify.py
```

`inputs.json` supplies a sample manifest, time-bounded scan log and policy revision. `operational-case.json` retains per-observation rule status, operands and provenance. `visual.json` has display values and no carrier result; the null carrier counter must not be shown as a measured zero. The independent verifier compares all nine decisions with known outcomes. To regenerate the JSON after an intentional local input change, run `python3 -B replay.py --write`. This single-pass replay uses exact strings and compares the selected batch only; it does not implement a persistent or concurrent release ledger.
