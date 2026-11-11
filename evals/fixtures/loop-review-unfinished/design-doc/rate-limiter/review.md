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

## Review 1

11/11/26. Design: design-doc/rate-limiter/design.md
Focus: what happens when Redis is slow or down

Derivation: any correct limiter must decide within NFR3's 1 ms whether or not the store answers, so the store call needs a hard timeout and a defined answer when it fires. Fail open is the only answer NFR3 allows, so the remaining question is what bounds a client while the store is down.
