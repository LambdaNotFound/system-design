# Scoping record: Rate limiter for the public API

Mode: interview
Origin: scope-problem 11/09/26
Frame: 11/09/26
Grounded: 11/09/26
Exit check: 11/09/26, not offered
Covered: 11/09/26
Decomposed: 11/09/26
Reader test: 11/09/26
Swept: 11/09/26
Humanized: 11/09/26
Final check: 11/09/26

## Questions asked

| # | Date | Question | Answer |
| --- | --- | --- | --- |
| F1 | 11/09/26 | What are we trying to change or build? | A per-client rate limiter in front of the public API. |
| F2 | 11/09/26 | Who is it for, and who else is affected? | API clients; the on-call operators. |
| F3 | 11/09/26 | What does done look like? | A client over its limit gets 429s and nobody else slows down. |
| F4 | 11/09/26 | Anything that must or must not be used? | No new datastore beyond Redis, which we already run. |
| F5 | 11/09/26 | Anything I should read first? | Nothing. |
| 2.1 | 11/09/26 | If the store is down, fail open or fail closed? Default: open. | 1: open |

## Coverage

| Row | State | Where it landed |
| --- | --- | --- |
| Why now | settled | Problem Statement |
| Who it serves | settled | Problem Statement |
| Done means | settled | FR1, FR2 |
| Scale | assumed | NFR1 |
| Failure behavior | settled | NFR3 |
| Correctness bar | settled | NFR4 |
| Existing state | collapsed: nothing live to migrate | |
| Blast radius | settled | Problem Statement |
| Operations | settled | NFR3 |
| Imposed constraints | settled | LLD1 |
| The boundary | settled | Out of scope line |
| The null option | settled | Problem Statement |

## Evidence

| Claim | Source | Label |
| --- | --- | --- |
| Nothing in this repo to read for this problem. | | |

## Open questions

| Question | Owner | Default |
| --- | --- | --- |

## Decomposition

| Piece slug | Serves (parent IDs, mapped to piece IDs) | Route | Depends on | Handoff |
| --- | --- | --- | --- | --- |
| rate-limiter | FR1-FR3, NFR1-NFR4 | design | none | /new-design rate-limiter |

## Reader test

11/09/26: the reader would build a per-client limiter returning 429 with Retry-After. It missed nothing that changes the build. No wording fixes.

11/09/26, final check: no contradictions or ambiguities found.

## Conflicts

| # | Between | Clash | Options | Decision | Date |
| --- | --- | --- | --- | --- | --- |
| C1 | NFR3, FR1 | Failing open means an over-limit client is not rejected while the store is down. | a) fail open and alert; b) fail closed | a | 11/09/26 |
| C2 | NFR3, NFR4 | Failing open lets a client exceed its limit by any amount during a store outage. | a) NFR4 applies only while the store is reachable; b) a per-instance fallback limit | a | 11/09/26 |
| C3 | NFR2, NFR3 | A store that hangs adds its timeout to every request before failing open. | a) a 1 ms store timeout; b) relax NFR2 during outages | a | 11/09/26 |

## Humanizer

11/09/26: Problem Statement reworded; no table rows changed.
