# Implementation review: Rate limiter

## Tests round 1

11/12/26. Derivation: the check must reject once the weighted estimate passes the limit (FR1), say how long until the next minute (FR2), weight the previous minute by its overlap with the last 60 seconds (NFR4), and allow the request when the store times out (NFR3). The interface needs the store behind a trait and the time passed in, or neither the timeout nor the overlap can be tested.

No objections. Each of the four promises has a test that a wrong implementation fails: a fixed-window check fails `previous_minute_is_weighted_by_overlap`, and a fail-closed check fails `store_timeout_allows_the_request`.

Strong: time is an argument, so the boundary test needs no sleep.

Checked: Ownership and borrowing ok, unsafe n/a, Async n/a, Atomics n/a, Errors and panics ok, Arithmetic and casts ok, Traits and API evolution n/a, Performance patterns worth flagging n/a, Naming ok, Idiom ok, Tests and tooling ok
Verdict: approve

## Round 1

11/12/26. Derivation: one store call, then the estimate, then one comparison; the timeout arm returns before any arithmetic. Nothing is shared, so nothing needs a guard.

No objections. The check record `impl-checks/R0.md` reads `Result: pass`, and the frozen tests are unchanged.

Strong: the timeout arm is matched first, so a store failure can never reach the estimate.

Checked: Ownership and borrowing ok, unsafe n/a, Async n/a, Atomics n/a, Errors and panics ok, Arithmetic and casts ok, Traits and API evolution n/a, Performance patterns worth flagging n/a, Naming ok, Idiom ok, Tests and tooling ok
Verdict: approve
