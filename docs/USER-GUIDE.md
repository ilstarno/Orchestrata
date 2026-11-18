# Set up a planning problem

Use the [quickstart](QUICKSTART.md) to run the bundled example. This guide helps you choose rules, try a small change, and interpret the result.

## Separate rules from preferences

Write down each decision and the options it can take. Then ask: **Would I reject a plan for breaking this rule, even if everything else looked better?**

- **Hard rule:** Two jobs cannot use the same slot. Reject any plan that puts both in one slot.
- **Preference:** Give job A the later slot if possible. The earlier slot is still acceptable.
- **Allowed options:** If job A cannot use the earlier slot at all, remove that slot from its options. This is not a preference.

Start with your hard rules and a small example where you know the right answer. Add preferences after you have checked that the example behaves as expected.

## Try the sample

The [assignment example](../backend/examples/assignment.json) gives jobs A and B two possible slots, numbered 0 and 1. The jobs must use different slots, and job A prefers slot 1. Run the example as written. It should return `[1, 0]`: job A gets slot 1 and job B gets slot 0. Results follow the order of the jobs in your input.

To try a preference change, copy the sample to a new file, set the preferred `value` for job A to `0`, and run the quickstart command with the new filename. The result should become `[0, 1]`, with the hard rule unchanged.

To see what happens when no plan is possible, start from a fresh copy and set both jobs' `domain` to `[0]`. Keep the different-slots rule. No assignment can meet that rule.

## Read the result

| What you see | What it means | What to do |
| --- | --- | --- |
| An assignment, such as `[1, 0]` | It meets the hard rules you entered. A soft penalty means a preference was missed, not that a hard rule failed. | Check the assignment against real-world rules you may have left out before using it. |
| `"solution": null` in the API response, or `null` from the CLI | No assignment meets all the hard rules in this example. | Look for conflicting rules or missing options. Change a hard rule only if it is not truly mandatory. |
| An error that the search space exceeds the evaluation limit | The request was too large to check within the limit. It does not tell you whether a plan exists. | Try a smaller representative set of jobs or remove options that are genuinely unavailable. |
| An invalid-input error | The submitted problem could not be used. It is not a planning result. | Check that every job has at least one option and that each rule refers to the correct jobs. |
