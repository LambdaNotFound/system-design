# When the loop waits for the user

Read by the lead whenever the loop stops for a decision, and on a resume that finds one pending.

Four things stop the loop for a decision. Put the matter to the user in one message: the escalation paragraph, the finding, or the reason, with the options below. Then record their answer as one line in `loop.md`, so a resume knows it was given:

`D<n> | <date> | <what it answers> | continue or accept or stop | <note>`

| Waiting on | `continue` | `accept` |
| --- | --- | --- |
| `R0: not answerable` | Once the user has amended the requirement doc, dispatch `round 0` again. | Not offered. |
| `Round <N> escalate` | The verdict counts as `revise`. Note `rounds through <N+2>`, then step 3. | The user approves over the bar-raiser: the verdict counts as `approve-with-risks`, the escalation paragraph travels as a residual risk, then step 4. |
| `Editorial check <k>` not approved | Dispatch the investigator with `respond to editorial check <k>`, then the bar-raiser with `editorial check <k+1>`. | Close as it stands, and report the finding as a residual risk. |
| `A<n>` or `V<k>` reopens the design | Delete that pass's snapshot, note `rounds through <N+2>`, and dispatch the bar-raiser with `review round <N+1>` and `Reopened by:`. | Not offered: the user withdraws the amendment with another one, or drops the fix. |

`stop` ends the loop where it is. A design that was never approved is not exported. One decision line answers one occurrence: a second escalation needs its own line.
