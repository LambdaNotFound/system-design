use rate_limiter::{CounterStore, Counts, Decision, StoreTimeout, check, estimate};

struct FixedStore(Result<Counts, StoreTimeout>);

impl CounterStore for FixedStore {
    fn increment(&self, _client: &str, _minute: u64) -> Result<Counts, StoreTimeout> {
        self.0
    }
}

fn store(previous: u64, current: u64) -> FixedStore {
    FixedStore(Ok(Counts { previous, current }))
}

/// Proves FR1.
#[test]
fn under_limit_is_allowed() {
    assert_eq!(check(&store(0, 10), "c1", 100, 120), Decision::Allow);
}

/// Proves FR1.
#[test]
fn over_limit_is_rejected() {
    assert!(matches!(
        check(&store(0, 101), "c1", 100, 120),
        Decision::Reject { .. }
    ));
}

/// Proves FR2.
#[test]
fn rejection_carries_seconds_to_next_minute() {
    assert_eq!(
        check(&store(0, 101), "c1", 100, 135),
        Decision::Reject {
            retry_after_secs: 45
        }
    );
}

/// Proves NFR4.
#[test]
fn previous_minute_is_weighted_by_overlap() {
    // 15 seconds in, three quarters of the previous minute still overlaps the last 60 seconds.
    let counts = Counts {
        previous: 100,
        current: 20,
    };
    assert_eq!(estimate(counts, 15), 95.0);
}

/// Proves NFR3.
#[test]
fn store_timeout_allows_the_request() {
    assert_eq!(
        check(&FixedStore(Err(StoreTimeout)), "c1", 100, 120),
        Decision::AllowedStoreTimeout
    );
}
