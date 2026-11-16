# Phase 3: rounds and coverage

## Contents

- Ask in rounds: the round plan, the two halves of a round, the check after each round, and when to stop
- Coverage: the twelve rows that must be settled, and their four states
- Scaling the list, stopping early, and keeping the doc tight

Read at the start of phase 3.

## Ask in rounds

The interview is where the requirements are argued out. A clash or a gap the user first meets in the phase 6 sweep is one the interview should have found: there it reaches them as a queue of items to rule on, and here it is a conversation.

### Plan the rounds

Size the interview before the first question:

- **A system-sized problem** (`Exit check:` reads `not offered`) gets five to eight rounds.
- **A small change** (the exit was offered and declined) gets as many rounds as its coverage rows need, often one or two.

Then give the user the plan: the rounds by theme, in order, in one short list. They can reorder it, cut a round or add one. The default themes, merged down to the count the problem needs:

1. **Goal and users:** the real goal, who it serves, why now, why doing nothing loses.
2. **Core behavior:** what each caller can do, walked as scenarios, and the check that proves each.
3. **Scale and performance.**
4. **Failure and correctness:** what may be lost, stale or late, and what stays up when something breaks.
5. **Abuse, security and operations.**
6. **Boundary and constraints:** what is out, and what is imposed.
7. **Trade-offs:** the answers so far that pull against each other.
8. **Walk-back:** the edge cases and trade-offs stated back for the user to confirm, then anything else.

A plan always ends with the Trade-offs and Walk-back rounds. Write the plan into the record's Round plan table before you ask round 1, and mark each round `done` when it closes, so a resume continues at the first round that is not done.

### Run one round

Open each round with its number, the plan's length and its theme ("Round 4 of 6: failure and correctness"). A round has two halves:

1. **Open questions, in plain text.** Three to six numbered questions, each led by what you already found and carrying a proposed default. End the turn there and take the answers, shorthand included.
2. **Trade-offs, in a dialog.** A question is a trade-off when every answer costs something against another requirement or another answer. Put the round's trade-offs, including the ones the answers just opened, to the user with `AskUserQuestion`: at most four a dialog, two to four options each, your recommendation first, and each option's description saying what it costs. The user may see only the dialog, so each question names the behavior and the rows it touches in full. A round with no trade-off has no dialog.

Check whether a source answers a question before you ask it. A round left with fewer than three questions merges into the next one: never pad a round to fill the plan.

### Check the draft after every round

When a round's answers are in the doc, look for what the sweep and the fresh readers would find, while the user is still in the conversation:

- **Clashes:** each row against every other row and against the `Out of scope:` line, by the list in [sign-off.md](sign-off.md), phase 6, step 1.
- **Gaps:** walk each functional requirement as a scenario and ask what the doc says when the input is invalid, when the request repeats, when two arrive at once, when the thing it acts on has changed state (disabled, deleted, expired), and at each limit an NFR sets. Then check that every answer in Questions asked has reached a row.

Keep a finding only when it leaves a behavior the user would notice, or a bar the design must meet, open to two readings. How to meet a bar is the design's to decide. What you keep opens the next round: a clash or a gap with real options goes into that round's dialog, and the rest become its numbered questions. Log each clash in the Conflicts table with the decision.

### Stop

The interview ends when the plan's rounds are done, the check after the last round finds nothing new, and the user has confirmed the walk-back. In the walk-back, ask outright, once, whether there is anything else: what a user volunteers at that prompt is routinely the constraint that would have invalidated the design.

Fewer rounds than planned happens only when the user cuts the plan or says to stop. If the eighth round still opens new clashes, ask the user whether to add a round or to carry what is left as open questions with defaults.

### Word the questions

Lead each question with what you already found, and carry a proposed default so the user corrects rather than authors:

> 3. The scheduler already retries three times (`scheduler.rs:141`), so a failed run is not silent today. Is the gap the retry count, or that nobody is told after the third? I assume the second.

A default is a starting point for an answer, never a substitute for asking. Never silently adopt one on a coverage row. A trade-off never goes out as a single default to confirm: it goes out with its options and their costs.

## Coverage

What must be settled before the requirement doc can be signed. Every row below ends in one of four states, recorded in the record's Coverage table:

- **settled**: the user answered it.
- **defaulted**: logged as an open question with an owner and a default the user accepted.
- **assumed**: written into the doc as an `(assumed)` statement. This is also where a row ends when the user tells you to stop asking.
- **collapsed**: resolved in a clause because it does not apply at this blast radius, with the reason.

A row must never decide itself in your head. A row with no state is unasked, and an unasked row is not an answered one.

| Must be settled | The question underneath | Lands in |
| --- | --- | --- |
| Why now | What changes for whom when this ships, and why it beats waiting | Problem Statement |
| Who it serves | Who calls it, who operates it, who is hurt when it breaks | Problem Statement |
| Done means | The check that proves it: a number, a test, an observable behavior | FR wording |
| Scale | How much data, how many callers, how often, what growth is assumed | NFR |
| Failure behavior | What happens when it breaks, what degradation is acceptable, who finds out | NFR |
| Correctness bar | Can it lose data, serve stale reads, or double-process? Which is fatal? | NFR |
| Existing state | What is live, what must be migrated or backfilled, how to roll back | NFR |
| Blast radius | Who depends on what changes, and what breaks if its contract moves | NFR |
| Operations | What must be logged, measured, or alerted for this to be supportable | NFR |
| Imposed constraints | A deadline or team; a mandated tool, library, or pattern | Problem Statement; Low-level Design Preferences |
| The boundary | What is explicitly out | The `Out of scope:` line |
| The null option | Why doing nothing loses | Problem Statement |

### Scale the list to the blast radius

For a change confined to one crate with no outside callers, Scale, Operations, and Blast radius resolve in a clause ("single process, no external callers"). Resolving a row cheaply is fine; skipping it silently is not. Say which rows you collapse and why, and let the user object.

### When the user says to stop asking

Stop, and produce the doc. Every row still without a state becomes `assumed`: write what you assumed into the doc with its `(assumed)` tag. Name those rows, and what you assumed for each, in the phase 6 walk and again in the hand off.

### Keep the doc tight while you work

One testable sentence per table row, and a Problem Statement of one short paragraph. When a row needs a second sentence, the fix is sharper wording, a second row, or a move to the right table, never an extra section.
