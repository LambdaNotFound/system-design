Covers: 0 amendments, 0 scope lines

# Implementation: Rate limiter

## Mechanisms

- **Sliding window check.** Work Breakdown task 1; serves FR1, FR2 and NFR4. The estimate and the reject decision are where the design keeps its correctness bar.
- **Fail open.** Work Breakdown task 2; serves NFR3. The timeout path decides what a store outage costs.

## Types

```rust
pub struct Counts { pub previous: u64, pub current: u64 }
pub enum Decision { Allow, Reject { retry_after_secs: u64 }, AllowedStoreTimeout }
pub struct StoreTimeout;
pub trait CounterStore {
    fn increment(&self, client: &str, minute: u64) -> Result<Counts, StoreTimeout>;
}
pub fn estimate(counts: Counts, elapsed_secs: u64) -> f64;
pub fn check<S: CounterStore>(store: &S, client: &str, limit: u64, now_secs: u64) -> Decision;
```

The store is a trait, so a test can make it time out. Time is the `now_secs` argument.

## Test plan

| Test | Proves | Kind |
| --- | --- | --- |
| `under_limit_is_allowed` | FR1 | behavior |
| `over_limit_is_rejected` | FR1 | behavior |
| `rejection_carries_seconds_to_next_minute` | FR2 | boundary |
| `previous_minute_is_weighted_by_overlap` | NFR4 | boundary |
| `store_timeout_allows_the_request` | NFR3 | failure |

## Not proven here

- NFR1 and NFR2: load and p99 latency need the real store under production traffic.
- FR3: the limit refresh is Work Breakdown task 3, which this crate leaves out.

## Dependencies

none

## Checks

```sh
cargo fmt -p rate-limiter -- --check
cargo clippy -p rate-limiter --all-targets -- -D warnings
cargo test -p rate-limiter --no-fail-fast
```

## Follow-ups

- Limit refresh (FR3), Work Breakdown task 3.

## Revision log

## Objection responses
