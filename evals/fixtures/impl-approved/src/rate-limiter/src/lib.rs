//! The core of the rate limiter: the sliding window estimate and the fail-open check.
//!
//! See design-doc/rate-limiter/rfc.md, Sliding Window Counter and Fail Open.

/// One client's request counts for the previous and the current minute.
///
/// See design-doc/rate-limiter/rfc.md, Data model and storage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Counts {
    pub previous: u64,
    pub current: u64,
}

/// What the limiter decided for one request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    Allow,
    /// See design-doc/rate-limiter/requirement.md, FR2.
    Reject {
        retry_after_secs: u64,
    },
    /// See design-doc/rate-limiter/requirement.md, NFR3.
    AllowedStoreTimeout,
}

/// The store did not answer within its timeout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StoreTimeout;

/// The counter store: one call increments the current minute's counter and returns both counts.
///
/// See design-doc/rate-limiter/rfc.md, Sliding Window Counter.
pub trait CounterStore {
    fn increment(&self, client: &str, minute: u64) -> Result<Counts, StoreTimeout>;
}

/// The estimated number of requests in the last 60 seconds.
///
/// See design-doc/rate-limiter/rfc.md, Sliding Window Counter.
pub fn estimate(counts: Counts, elapsed_secs: u64) -> f64 {
    let elapsed_fraction = elapsed_secs as f64 / 60.0;
    counts.previous as f64 * (1.0 - elapsed_fraction) + counts.current as f64
}

/// Checks one request at `now_secs`, the seconds since the epoch.
///
/// See design-doc/rate-limiter/rfc.md, Sliding Window Counter and Fail Open.
pub fn check<S: CounterStore>(store: &S, client: &str, limit: u64, now_secs: u64) -> Decision {
    let elapsed_secs = now_secs % 60;
    match store.increment(client, now_secs / 60) {
        // The timeout is matched first, so a store failure never reaches the estimate.
        Err(StoreTimeout) => Decision::AllowedStoreTimeout,
        Ok(counts) if estimate(counts, elapsed_secs) > limit as f64 => Decision::Reject {
            retry_after_secs: 60 - elapsed_secs,
        },
        Ok(_) => Decision::Allow,
    }
}
