# Provider-labelled contact-center planning review

The same eight-position NYC staffing window is expressed in three **illustrative** sample envelopes: Genesys Cloud CX, RingCentral RingCX, and Asterisk PBX. These are not vendor API responses, actual installations, official adapters, or evidence of platform integration. The Asterisk-labelled input includes separately staged illustrative chat demand; no claim is made that PBX voice logs supply chat data.

```sh
python3 verify.py
python3 asterisk_queue_log.py sample-queue.log
```

The verifier reads the sibling five-branch sample roster (500 operators); eligibility and assignment checks apply only to its 100-profile NYC branch and the recorded early baseline window. It independently rebuilds each queue's eligibility, exhaustively checks 256 assignments (81 hard-feasible), and matches the archived eight-position native result. It does not make a new native-engine call. Each current snapshot normalizes to those same eight proposed positions. Missing, stale (>5 minutes), incomplete, or unsupported input returns manual review, no proposed positions, no prior-plan reuse, and no writeback. This does not change a queue, route a caller, transfer a contact, or change any schedule.

- `provider-samples.json`: illustrative normalized-input comparison across the three labels.
- `decisions.json`: matched assignment, checks, and current/missing/stale/invalid decisions per provider.
- `operational-case.json`: structured operating context and per-assignment evidence.
- Run `python3 verify.py --write` to regenerate the three JSON artifacts.

The optional local `asterisk_queue_log.py` counter is a separate extension. It reads a user-selected local log and counts only `ENTERQUEUE` event rows for four explicitly listed **sample** queue labels; it does not deduplicate callers or calculate queue health. It uses the [Asterisk queue-log format](https://docs.asterisk.org/Operation/Logging/Queue-Logs/) and emits only aggregate counts. It does not retain or print caller identifiers, interaction IDs, or event payloads. It neither connects to a PBX nor reads any default production path. The provided synthetic log exercised this utility; it is not input to the staffing calculation.
