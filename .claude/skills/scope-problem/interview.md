# Phase 3: rounds and coverage

## Contents

- Ask in rounds: what each round covers, and when to stop
- Coverage: the twelve rows that must be settled, and their four states
- Scaling the list, stopping early, and keeping the doc tight

Read at the start of phase 3.

## Ask in rounds

- **Round 1** covers what you need to understand the problem at all: the real goal, who it is for, what done means.
- **Round 2 and on** cover what the previous answers opened. An answer that names a scale, a deadline, or an existing system usually creates three new questions. Check whether a source answers them before you ask.
- **Stop** when you can state the edge cases and trade-offs back to the user and they confirm them. That, not a question count or a quiet round, is convergence. Never run a round only to see whether it changes anything. A change behind one interface can converge in one round.
- **Before leaving**, ask outright, once, whether there is anything else. What a user volunteers at that prompt is routinely the constraint that would have invalidated the design.

Lead each question with what you already found, and carry a proposed default so the user corrects rather than authors:

> 3. The scheduler already retries three times (`scheduler.rs:141`), so a failed run is not silent today. Is the gap the retry count, or that nobody is told after the third? I assume the second.

A default is a starting point for an answer, never a substitute for asking. Never silently adopt one on a coverage row.

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
