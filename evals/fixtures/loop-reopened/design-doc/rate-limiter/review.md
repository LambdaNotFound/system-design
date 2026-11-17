# Review ledger: rate-limiter

## Round 1

11/09/26. Derivation: any correct limiter needs a per-client count that every API node agrees on within NFR4's 5%, a decision path that never waits more than NFR3's 1 ms on the store, and a limit source that refreshes within FR3's 60 seconds. The minimum parts are one shared counter store (LLD1 fixes Redis), one check on the request path, and one limit table read with bounded staleness.

R1-1 | blocking | The fixed one-minute window lets a client send its full limit at 0:59 and again at 1:00, twice the limit inside 60 seconds, which breaks NFR4. | A window scheme whose worst-case overshoot is addressed against the 5% bound.
R1-2 | should-fix | The Redis call has no stated timeout, so NFR3's 1 ms bound has no mechanism. | The timeout value, and what the request path does when it fires.

Verdict: revise

## Round 2

11/09/26. Derivation carried forward from round 1, unchanged.

Closed: R1-1 resolved (sliding window counter); R1-2 resolved (1 ms timeout, fail open).

Spot-checks: the NFR4 row held; the NFR3 row held; the Redis EVAL page held.

Verdict: approve

## Round 3

11/11/26. Reopened by amendment A1. Derivation carried forward, changed in one place: the limit source must now refresh within FR3's 2 seconds, not 60.

R3-1 | blocking | Revised material (amendment A1). Limit Refresh reloads a cached limit every 30 seconds, so a change can take 30 seconds to reach a node, against FR3's 2. | A refresh mechanism whose worst case is inside 2 seconds, and what a node does when it misses a change.

Verdict: revise

## Round 4

11/11/26. Derivation carried forward from round 3, unchanged.

Closed: R3-1 resolved (published change, 30-second fallback reload, the miss stated as a cost and a failure mode).

Spot-checks: the amended FR3 row held, verbatim; the Redis PUBLISH page held; the Work Breakdown row for FR3 held.

Verdict: approve
