# Scoping record: Rate limiter for the public API

Mode: interview
Origin: scope-problem 11/09/26
Frame: 11/09/26
Grounded: 11/09/26
Exit check: 11/09/26, not offered

## Questions asked

| # | Date | Question | Answer |
| --- | --- | --- | --- |
| F1 | 11/09/26 | What are we trying to change or build? | A per-client rate limiter in front of the public API. |
| F2 | 11/09/26 | Who is it for, and who else is affected? | API clients; the on-call operators. |
| F3 | 11/09/26 | What does done look like? | A client over its limit gets 429s and nobody else slows down. |
| F4 | 11/09/26 | Anything that must or must not be used? | No new datastore beyond Redis, which we already run. |
| F5 | 11/09/26 | Anything I should read first? | Nothing. |

## Round plan

| Round | Theme | State |
| --- | --- | --- |

## Coverage

| Row | State | Where it landed |
| --- | --- | --- |

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

## Reader test

## Conflicts

| # | Between | Clash | Options | Decision | Date |
| --- | --- | --- | --- | --- | --- |

## Humanizer
