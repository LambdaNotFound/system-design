---
type: regex
target: last_message
pattern: "src/rate-limiter[\\s\\S]*cargo test|cargo test[\\s\\S]*src/rate-limiter"
weight: 1
---

The reply names the crate, `src/rate-limiter`, and the `cargo test` command that runs its tests.
