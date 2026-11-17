# When the loop waits for the user

Read by the lead whenever the loop stops for a decision, and on a resume that finds one pending.

Five things stop the loop for a decision. Put the matter to the user in one message: the escalation paragraph, the finding, or the reason, with the options below. Then record their answer as one line in `loop.md`, so a resume knows it was given:

`D<n> | <date> | <what it answers> | continue or accept or stop | <note>`

| Waiting on | `continue` | `accept` |
| --- | --- | --- |
| `R0: not answerable` | Once the user has amended the requirement doc, dispatch `round 0` again. | Not offered. |
| `Round <N> escalate`, or a `revise` or `reject-approach` in the budget's last round | The verdict counts as `revise`. Note `rounds through <N+2>`, then step 3. | The user approves over the bar-raiser: the verdict counts as `approve-with-risks`, the escalation paragraph or the round's open blocking objections travel as a residual risk, then step 4. |
| `Editorial check <k>` not approved | Dispatch the investigator with `respond to editorial check <k>`, unless the revision log already has `E<k>:`, then the bar-raiser with `editorial check <k+1>`. | Close as it stands, and report the finding as a residual risk. |
| `Amendment A<n> check <p>` escalates, or says `revise` when `<p>` is 2 or more | The verdict counts as `revise`: the investigator answers in one more pass, and the bar-raiser checks again. | The user approves over the bar-raiser: the verdict counts as `approve-with-risks`, the escalation paragraph travels as a residual risk, then the amendment's Close. |
| `A<n>` or `V<k>` reopens the design | Delete that pass's snapshot, note `rounds through <N+2>`, and dispatch the bar-raiser with `review round <N+1>` and `Reopened by:`. | Not offered. To decline the reopening, the user drops the fix or withdraws the amendment with another one: record `stop`. |

`stop` ends the loop where it is, and a resume leaves it there: it reports where the loop stopped and what is open, dispatches nothing, and does not run Close. So a design that was never approved is not exported, and when you record that `stop` its row in the root `README.md` goes back to `Scoped`; a later `continue` sets it to `Drafting` again. The loop starts again only when the user answers that stop anew, in a later decision line. A `stop` on a reopening is different: the design stays closed as it was approved, and a resume reads on past it.

One decision line answers one occurrence: a second escalation needs its own line.
