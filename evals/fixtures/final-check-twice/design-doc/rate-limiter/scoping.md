# Scoping record: Rate limiter for the public API

Mode: interview
Origin: scope-problem 11/09/26
Frame: 11/09/26
Grounded: 11/09/26
Exit check: 11/09/26, not offered
Covered: 11/09/26
Decomposed: 11/09/26
Reader test: 11/09/26
Swept: 11/10/26
Humanized: 11/10/26

## Questions asked

| # | Date | Question | Answer |
| --- | --- | --- | --- |
| F1 | 11/09/26 | What are we trying to change or build? | A per-client rate limiter in front of the public API. |
| F2 | 11/09/26 | Who is it for, and who else is affected? | API clients; the on-call operators. |
| F3 | 11/09/26 | What does done look like? | A client over its limit gets 429s and nobody else slows down. |
| F4 | 11/09/26 | Anything that must or must not be used? | No new datastore beyond Redis, which we already run. |
| F5 | 11/09/26 | Anything I should read first? | Nothing. |
| 2.1 | 11/09/26 | If the store is down, fail open or fail closed? Default: open. | 1: open |
| 2.2 | 11/09/26 | How much latency may the limiter add to a request? Default: at most 2 ms at p99. | 2: yes |

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
| Operations | settled | NFR3, FR3 |
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
| rate-limiter | FR1-FR3, NFR1-NFR4, LLD1 | design | none | /new-design rate-limiter |

## Reader test

11/09/26: the reader would build a per-client limiter returning 429 with Retry-After. It missed nothing that changes the build. No wording fixes.

**11/10/26, final check (phase 6, step 4).** A fresh reader with the doc alone.

- **Got right:** a per-client limiter in Redis that returns 429 with `Retry-After`, fails open with an alert, and takes limit changes without a deploy.
- **Contradiction:** FR1 rejected every request over the limit, while NFR4 lets a client go 5% over. Sent back to step 1 as C3. The decision changed FR1, so the sweep, the pass and the final check ran again.
- **Left to the design:** the counting algorithm and its window.

**11/10/26, second final check.** Another fresh reader with the doc alone.

- **Got right:** the whole build, including C2 and C3.
- **Contradiction:** FR3 lets an operator lower a limit in the middle of a minute, and NFR4 then fails for that minute, because requests already counted can be more than 5% over the new limit. Put to the user as C4.
- **Left to the design:** how a limit change reaches every limiter instance.

## Conflicts

| # | Between | Clash | Options | Decision | Date |
| --- | --- | --- | --- | --- | --- |
| C1 | NFR3, FR1 | Failing open means an over-limit client is not rejected while the store is down. | a) fail open and alert; b) fail closed | a | 11/09/26 |
| C2 | NFR3, NFR4 | While the store is unreachable every request is allowed, so the 5% bar cannot hold. | a) keep NFR3, and NFR4 covers only minutes during which the store stays reachable; b) fail closed so NFR4 always holds | a (NFR4) | 11/10/26 |
| G1 | F4, Low-level Design Preferences | The Redis-only constraint from F4 never reached the doc. | a) add it as LLD1; b) leave it to the design | a (LLD1) | 11/10/26 |
| C3 | FR1, NFR4 | FR1 rejected every request over the limit, while NFR4 lets a client go 5% over. | a) FR1 rejects what the limiter counts as over, and NFR4 bounds how far that count may be off; b) exact enforcement, drop the 5% | a (FR1) | 11/10/26 |
| C4 | FR3, NFR4 | An operator can lower a limit mid-minute (FR3), and the requests already counted can then be more than 5% over the new limit, failing NFR4 for that minute. | a) NFR4 covers only minutes in which the client's limit does not change; b) a lowered limit takes effect at the next minute boundary (FR3) | | |

## Humanizer

**11/10/26.** No humanizer skill in this session, so the scoping lead made the pass over all four sections. No section changed for style. Lines kept original: all.

**11/10/26, after C3.** FR1 only. No change for style. Lines kept original: all.
