//! The Redirect Resolver after a fill that found nothing: its waiters go
//! back through single flight once, instead of each reading the store.
//!
//! See design-doc/url-shortener/rfc.md, Redirect Resolver, Why each tier.

mod common;

use std::thread;
use std::time::Duration;

use common::{FakeReader, ManualClock, Tier, code, join, live, wait_until};
use url_shortener::{Resolution, Resolver, ResolverConfig, StoreError};

const START: Duration = Duration::from_secs(1_000);
const WAITERS: u64 = 7;

/// Runs a viral lookup whose first fill fails at both tiers while `WAITERS`
/// lookups wait on it, and holds the second fill's local read until every
/// waiter has joined it. Returns the first fill's answer and the waiters'.
fn after_a_failed_fill(reader: &FakeReader) -> (Resolution, Vec<Resolution>) {
    let c = code("viral002");
    let clock = ManualClock::at(START);
    let resolver = Resolver::new(reader, clock, ResolverConfig::default());

    thread::scope(|s| {
        let local = reader.hold(Tier::Local);
        let consistent = reader.hold(Tier::Consistent);
        let leader = s.spawn(|| resolver.resolve(c.as_str()));
        wait_until("the leader's local read", || {
            reader.in_flight(Tier::Local) == 1
        });
        let waiters: Vec<_> = (0..WAITERS)
            .map(|_| s.spawn(|| resolver.resolve(c.as_str())))
            .collect();
        wait_until("every waiter to join or a second read", || {
            resolver.stats().fill_joins == WAITERS || reader.calls(Tier::Local) > 1
        });
        assert_eq!(reader.calls(Tier::Local), 1, "a waiter read the store");

        local.release();
        wait_until("the leader's consistent read", || {
            reader.in_flight(Tier::Consistent) == 1
        });
        let local = reader.hold(Tier::Local);
        consistent.release();
        let leader_answer = join(leader);

        wait_until("the waiters to join the second fill or read more", || {
            resolver.stats().fill_joins == 2 * WAITERS - 1 || reader.calls(Tier::Local) > 2
        });
        assert_eq!(
            reader.calls(Tier::Local),
            2,
            "waiters of a failed fill read the store on their own"
        );
        local.release();
        (leader_answer, waiters.into_iter().map(join).collect())
    })
}

/// Proves the RFC's "one store read per server per T" when a fill fails
/// (Failure Modes, The store is slow): the waiters send one more fill
/// through single flight, and its found link answers them all.
#[test]
fn waiters_of_a_failed_fill_share_one_refill() {
    let reader = FakeReader::new();
    reader.put(&code("viral002"), live("http://viral.example/"));
    reader.fail_next(Tier::Local, StoreError::Timeout);
    reader.fail_next(Tier::Consistent, StoreError::Timeout);

    let (leader, waiters) = after_a_failed_fill(&reader);

    assert_eq!(leader, Resolution::Unavailable);
    assert!(
        waiters
            .iter()
            .all(|a| *a == Resolution::Redirect("http://viral.example/".into())),
        "{waiters:?}"
    );
    assert_eq!(reader.calls(Tier::Local), 2);
    assert_eq!(reader.calls(Tier::Consistent), 1);
}

/// Proves the same bound when the refill fails too: a waiter takes the
/// second fill's `503`, which began after it arrived, instead of reading.
#[test]
fn waiters_take_the_refills_failure() {
    let reader = FakeReader::new();
    for _ in 0..2 {
        reader.fail_next(Tier::Local, StoreError::Timeout);
        reader.fail_next(Tier::Consistent, StoreError::Timeout);
    }

    let (leader, waiters) = after_a_failed_fill(&reader);

    assert_eq!(leader, Resolution::Unavailable);
    assert!(
        waiters.iter().all(|a| *a == Resolution::Unavailable),
        "{waiters:?}"
    );
    assert_eq!(reader.calls(Tier::Local), 2);
    assert_eq!(reader.calls(Tier::Consistent), 2);
}
