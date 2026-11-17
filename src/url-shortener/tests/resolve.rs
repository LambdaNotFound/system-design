//! The Redirect Resolver's steps 2 to 4 over a scripted two-tier reader and
//! a manual clock.
//!
//! See design-doc/url-shortener/rfc.md, Redirect Resolver.

mod common;

use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use common::{
    Call, FakeReader, ManualClock, ScriptedCodes, Tier, code, disabled, join, live, wait_until,
};
use url_shortener::{
    Created, LinkCreator, MemStore, Resolution, Resolver, ResolverConfig, StoreError,
};

/// The clock's start, away from zero so no subtraction can wrap unnoticed.
const START: Duration = Duration::from_secs(1_000);
const T: Duration = Duration::from_secs(60);
const S: Duration = Duration::from_secs(30);
const MS: Duration = Duration::from_millis(1);

fn redirect(url: &str) -> Resolution {
    Resolution::Redirect(url.into())
}

fn resolver<'a>(
    reader: &'a FakeReader,
    clock: &ManualClock,
) -> Resolver<&'a FakeReader, ManualClock> {
    Resolver::new(reader, clock.clone(), ResolverConfig::default())
}

fn with_cap(f: usize) -> ResolverConfig {
    ResolverConfig {
        max_consistent_in_flight: f,
        ..ResolverConfig::default()
    }
}

/// Proves FR2 and FR6's browser tail: a created code redirects to the exact
/// submitted spelling, cacheable privately for 60 s.
#[test]
fn redirects_to_the_exact_submitted_spelling() {
    let c = code("redir001");
    let store = MemStore::new();
    let creator = LinkCreator::new(&store, ScriptedCodes::new([c]));
    assert_eq!(
        creator.create("HTTPS://Example.com:443/a/../b"),
        Ok(Created::New(c))
    );

    let clock = ManualClock::at(START);
    let resolver = Resolver::new(&store, clock, ResolverConfig::default());
    let answer = resolver.resolve(c.as_str());
    assert_eq!(answer, redirect("HTTPS://Example.com:443/a/../b"));
    assert_eq!(answer.status(), 302);
    assert_eq!(answer.cache_control(), Some("private, max-age=60"));
}

/// Proves NFR11 and the one-read-per-T bound: a code the local replica lacks
/// is confirmed at the leaseholder, not answered `404`, and what the
/// leaseholder found fills the cache like a local find.
#[test]
fn local_miss_falls_through_to_a_consistent_read() {
    let c = code("fresh001");
    let reader = FakeReader::new();
    reader.put_on(Tier::Consistent, &c, live("http://example.com/"));
    let clock = ManualClock::at(START);
    let resolver = resolver(&reader, &clock);

    assert_eq!(
        resolver.resolve(c.as_str()),
        redirect("http://example.com/")
    );
    assert_eq!(reader.calls(Tier::Local), 1);
    assert_eq!(reader.calls(Tier::Consistent), 1);

    clock.set(START + T - MS);
    assert_eq!(
        resolver.resolve(c.as_str()),
        redirect("http://example.com/")
    );
    assert_eq!(reader.calls(Tier::Local), 1, "a hit within T read locally");
    assert_eq!(
        reader.calls(Tier::Consistent),
        1,
        "a hit within T crossed regions"
    );
    assert_eq!(resolver.stats().cache_hits, 1);
}

/// Proves FR6 and NFR11 under a store outage: an entry T old is never served
/// in place of a failed read, so a disable cannot outlive S + T while both
/// tiers fail (rfc.md, Failure Modes, "Two regions fail"). Covers step 4
/// failing and step 4 over F.
#[test]
fn expired_entry_is_not_served_while_the_store_fails() {
    let c = code("outage01");
    let reader = FakeReader::new();
    reader.put(&c, live("http://example.com/"));
    let clock = ManualClock::at(START);
    let resolver = resolver(&reader, &clock);
    assert_eq!(
        resolver.resolve(c.as_str()),
        redirect("http://example.com/")
    );

    clock.set(START + T);
    reader.fail_next(Tier::Local, StoreError::TooStale);
    reader.fail_next(Tier::Consistent, StoreError::Timeout);
    let answer = resolver.resolve(c.as_str());
    assert_eq!(answer, Resolution::Unavailable);
    assert_eq!(answer.status(), 503);
    assert_eq!(reader.calls(Tier::Consistent), 1);

    let (busy, d) = (code("busy0001"), code("outage02"));
    let reader = FakeReader::new();
    reader.put(&d, live("http://example.com/"));
    reader.put_on(Tier::Consistent, &busy, live("http://busy.example/"));
    let clock = ManualClock::at(START);
    let resolver = Resolver::new(&reader, clock.clone(), with_cap(1));
    assert_eq!(
        resolver.resolve(d.as_str()),
        redirect("http://example.com/")
    );

    clock.set(START + T);
    let (over_f, busy_answer) = thread::scope(|s| {
        let hold = reader.hold(Tier::Consistent);
        let busy_lookup = s.spawn(|| resolver.resolve(busy.as_str()));
        wait_until("the busy code's consistent read", || {
            reader.in_flight(Tier::Consistent) == 1
        });
        reader.fail_next(Tier::Local, StoreError::Timeout);
        let over_f = resolver.resolve(d.as_str());
        hold.release();
        (over_f, join(busy_lookup))
    });
    assert_eq!(over_f, Resolution::Unavailable);
    assert_eq!(resolver.stats().consistent_rejected, 1);
    assert_eq!(busy_answer, redirect("http://busy.example/"));
}

/// Proves NFR11 under step 3 failures: a too-stale replica or a step 3
/// timeout falls through to the consistent read.
#[test]
fn local_error_falls_through_to_a_consistent_read() {
    let (c1, c2) = (code("stale001"), code("tmout002"));
    let reader = FakeReader::new();
    reader.put(&c1, live("http://one.example/"));
    reader.put(&c2, live("http://two.example/"));
    let clock = ManualClock::at(START);
    let resolver = resolver(&reader, &clock);

    reader.fail_next(Tier::Local, StoreError::TooStale);
    assert_eq!(
        resolver.resolve(c1.as_str()),
        redirect("http://one.example/")
    );
    reader.fail_next(Tier::Local, StoreError::Timeout);
    assert_eq!(
        resolver.resolve(c2.as_str()),
        redirect("http://two.example/")
    );
    assert_eq!(reader.calls(Tier::Consistent), 2);
}

/// Proves NFR11: a `404` is never cached, so a code created right after a
/// miss resolves at the same instant.
#[test]
fn absent_code_is_never_cached() {
    let c = code("later001");
    let reader = FakeReader::new();
    let clock = ManualClock::at(START);
    let resolver = resolver(&reader, &clock);

    let answer = resolver.resolve(c.as_str());
    assert_eq!(answer, Resolution::NotFound);
    assert_eq!(answer.status(), 404);
    assert_eq!(answer.cache_control(), Some("no-store"));

    reader.put_on(Tier::Consistent, &c, live("http://example.com/"));
    assert_eq!(
        resolver.resolve(c.as_str()),
        redirect("http://example.com/")
    );
    assert_eq!(reader.calls(Tier::Consistent), 2);
}

/// Proves NFR4 and NFR11: a failed consistent read is `503`, not `404`, and
/// is not cached.
#[test]
fn consistent_failure_is_503_and_not_cached() {
    let c = code("unconf01");
    let reader = FakeReader::new();
    reader.put_on(Tier::Consistent, &c, live("http://example.com/"));
    reader.fail_next(Tier::Consistent, StoreError::Timeout);
    let clock = ManualClock::at(START);
    let resolver = resolver(&reader, &clock);

    let answer = resolver.resolve(c.as_str());
    assert_eq!(answer, Resolution::Unavailable);
    assert_eq!(answer.status(), 503);
    assert_eq!(answer.cache_control(), None);

    assert_eq!(
        resolver.resolve(c.as_str()),
        redirect("http://example.com/")
    );
    assert_eq!(reader.calls(Tier::Consistent), 2);
}

/// Proves NFR12's use in step 1: a path that is not 8 base62 characters is
/// `404` without any store read.
#[test]
fn not_a_code_is_404_without_a_store_read() {
    let reader = FakeReader::new();
    let clock = ManualClock::at(START);
    let resolver = resolver(&reader, &clock);

    for path in ["", "abc", "v1/links", "abcdefghi", "abc-1234", "abcdéfg"] {
        let answer = resolver.resolve(path);
        assert_eq!(answer, Resolution::NotFound, "{path:?}");
        assert_eq!(answer.cache_control(), Some("no-store"), "{path:?}");
    }
    assert_eq!(reader.call_log(), Vec::<Call>::new());
}

/// Proves FR6's S and NFR3's per-read budgets: step 3 asks for at most 30 s
/// of staleness within 20 ms, and step 4 reads within 100 ms; the defaults
/// hold the RFC's T and F.
#[test]
fn reads_carry_the_designs_bounds() {
    let (near, far) = (code("near0001"), code("far00001"));
    let reader = FakeReader::new();
    reader.put(&near, live("http://near.example/"));
    reader.put_on(Tier::Consistent, &far, live("http://far.example/"));
    let clock = ManualClock::at(START);
    let resolver = resolver(&reader, &clock);

    assert_eq!(
        resolver.resolve(near.as_str()),
        redirect("http://near.example/")
    );
    assert_eq!(
        resolver.resolve(far.as_str()),
        redirect("http://far.example/")
    );
    assert_eq!(
        reader.call_log(),
        vec![
            Call {
                tier: Tier::Local,
                code: near,
                max_staleness: Some(S),
                timeout: Duration::from_millis(20),
            },
            Call {
                tier: Tier::Local,
                code: far,
                max_staleness: Some(S),
                timeout: Duration::from_millis(20),
            },
            Call {
                tier: Tier::Consistent,
                code: far,
                max_staleness: None,
                timeout: Duration::from_millis(100),
            },
        ]
    );
    let defaults = ResolverConfig::default();
    assert_eq!(defaults.cache_ttl, T);
    assert_eq!(defaults.max_consistent_in_flight, 64);
}

/// Proves FR6's T: an entry serves while younger than 60 s, then is read
/// again.
#[test]
fn cache_serves_until_ttl_then_rereads() {
    let c = code("cached01");
    let reader = FakeReader::new();
    reader.put(&c, live("http://example.com/"));
    let clock = ManualClock::at(START);
    let resolver = resolver(&reader, &clock);

    assert_eq!(
        resolver.resolve(c.as_str()),
        redirect("http://example.com/")
    );
    assert_eq!(reader.calls(Tier::Local), 1);

    clock.set(START + T - MS);
    assert_eq!(
        resolver.resolve(c.as_str()),
        redirect("http://example.com/")
    );
    assert_eq!(
        reader.calls(Tier::Local),
        1,
        "a hit within T read the store"
    );
    assert_eq!(resolver.stats().cache_hits, 1);

    clock.set(START + T);
    assert_eq!(
        resolver.resolve(c.as_str()),
        redirect("http://example.com/")
    );
    assert_eq!(reader.calls(Tier::Local), 2, "an entry T old was served");
}

/// Proves FR6: a disabled link is `410`, not stored by other caches, and
/// served from this server's cache within T.
#[test]
fn disabled_is_cached_as_gone() {
    let c = code("gone0001");
    let reader = FakeReader::new();
    reader.put(&c, disabled("http://example.com/"));
    let clock = ManualClock::at(START);
    let resolver = resolver(&reader, &clock);

    let answer = resolver.resolve(c.as_str());
    assert_eq!(answer, Resolution::Gone);
    assert_eq!(answer.status(), 410);
    assert_eq!(answer.cache_control(), Some("no-store"));

    clock.set(START + T - MS);
    assert_eq!(resolver.resolve(c.as_str()), Resolution::Gone);
    assert_eq!(reader.calls(Tier::Local), 1);
    assert_eq!(reader.calls(Tier::Consistent), 0);
}

/// Proves FR6's server bound: with the replica S behind, a disable stops
/// this server's redirects by S + T = 90 s, within FR6's 300 s.
#[test]
fn disable_takes_effect_within_staleness_plus_ttl() {
    let c = code("takedown");
    let reader = FakeReader::new();
    let disabled_at = START;
    reader.put_on(Tier::Local, &c, live("http://example.com/"));
    reader.put_on(Tier::Consistent, &c, disabled("http://example.com/"));
    let clock = ManualClock::at(disabled_at + S - MS);
    let resolver = resolver(&reader, &clock);

    // The replica's last moment before it shows the disable: filled live.
    assert_eq!(
        resolver.resolve(c.as_str()),
        redirect("http://example.com/")
    );
    reader.put_on(Tier::Local, &c, disabled("http://example.com/"));

    clock.set(disabled_at + S + T - 2 * MS);
    assert_eq!(
        resolver.resolve(c.as_str()),
        redirect("http://example.com/")
    );
    clock.set(disabled_at + S + T - MS);
    assert_eq!(resolver.resolve(c.as_str()), Resolution::Gone);
}

/// Proves NFR1's read amplification bound: lookups of a code whose fill is
/// in flight wait for it, so a viral code costs one store read per fill.
#[test]
fn viral_code_is_read_once_per_fill() {
    let c = code("viral001");
    let reader = FakeReader::new();
    reader.put(&c, live("http://viral.example/"));
    let clock = ManualClock::at(START);
    let resolver = resolver(&reader, &clock);

    let answers: Vec<Resolution> = thread::scope(|s| {
        let hold = reader.hold(Tier::Local);
        let leader = s.spawn(|| resolver.resolve(c.as_str()));
        wait_until("the leader's local read", || {
            reader.in_flight(Tier::Local) == 1
        });
        let joiners: Vec<_> = (0..7)
            .map(|_| s.spawn(|| resolver.resolve(c.as_str())))
            .collect();
        wait_until("7 joins or a second read", || {
            resolver.stats().fill_joins == 7 || reader.calls(Tier::Local) > 1
        });
        assert_eq!(reader.calls(Tier::Local), 1, "a joiner read the store");
        hold.release();
        std::iter::once(leader).chain(joiners).map(join).collect()
    });

    assert!(
        answers
            .iter()
            .all(|a| *a == redirect("http://viral.example/")),
        "{answers:?}"
    );
    assert_eq!(reader.calls(Tier::Local), 1);
    assert_eq!(reader.calls(Tier::Consistent), 0);
}

/// Proves NFR7 within the fill: concurrent fills of two codes never hand a
/// joiner the other code's URL.
#[test]
fn joiners_never_get_another_codes_url() {
    let (a, b) = (code("codeAAAA"), code("codeBBBB"));
    let reader = FakeReader::new();
    reader.put(&a, live("http://a.example/"));
    reader.put(&b, live("http://b.example/"));
    let clock = ManualClock::at(START);
    let resolver = resolver(&reader, &clock);

    let answers: Vec<(char, Resolution)> = thread::scope(|s| {
        let hold = reader.hold(Tier::Local);
        let mut handles = vec![
            s.spawn(|| ('a', resolver.resolve(a.as_str()))),
            s.spawn(|| ('b', resolver.resolve(b.as_str()))),
        ];
        wait_until("both leaders' local reads", || {
            reader.in_flight(Tier::Local) == 2
        });
        for _ in 0..3 {
            handles.push(s.spawn(|| ('a', resolver.resolve(a.as_str()))));
            handles.push(s.spawn(|| ('b', resolver.resolve(b.as_str()))));
        }
        wait_until("6 joins or a third read", || {
            resolver.stats().fill_joins == 6 || reader.calls(Tier::Local) > 2
        });
        assert_eq!(reader.calls(Tier::Local), 2, "a joiner read the store");
        hold.release();
        handles.into_iter().map(join).collect()
    });

    for (which, answer) in answers {
        let want = if which == 'a' {
            "http://a.example/"
        } else {
            "http://b.example/"
        };
        assert_eq!(answer, redirect(want), "a lookup of code {which}");
    }
}

/// Proves NFR11 within the fill: a lookup that joins a fill after the code
/// was created does not take that fill's `404`, which a read started before
/// the creation returned.
#[test]
fn negative_fill_is_not_shared() {
    let c = code("racing01");
    let reader = FakeReader::new();
    let clock = ManualClock::at(START);
    let resolver = resolver(&reader, &clock);

    let (leader_answer, joiner_answer) = thread::scope(|s| {
        let hold = reader.hold(Tier::Consistent);
        let leader = s.spawn(|| resolver.resolve(c.as_str()));
        wait_until("the leader's consistent read", || {
            reader.in_flight(Tier::Consistent) == 1
        });
        // The leader's read has already taken its answer: absent.
        reader.put_on(Tier::Consistent, &c, live("http://new.example/"));
        let joiner = s.spawn(|| resolver.resolve(c.as_str()));
        wait_until("the joiner to join the fill", || {
            resolver.stats().fill_joins == 1
        });
        hold.release();
        (join(leader), join(joiner))
    });

    assert_eq!(leader_answer, Resolution::NotFound);
    assert_eq!(joiner_answer, redirect("http://new.example/"));
}

/// Proves NFR4's bound on step 4: at most F consistent reads in flight per
/// server; one over F is `503` at once and is counted; slots free on return.
#[test]
fn consistent_reads_are_capped_per_server() {
    let codes = [
        code("capped01"),
        code("capped02"),
        code("capped03"),
        code("capped04"),
    ];
    let reader = FakeReader::new();
    for (i, c) in codes.iter().enumerate() {
        reader.put_on(Tier::Consistent, c, live(&format!("http://{i}.example/")));
    }
    let clock = ManualClock::at(START);
    let resolver = Resolver::new(&reader, clock, with_cap(2));

    let (first, second, third) = thread::scope(|s| {
        let hold = reader.hold(Tier::Consistent);
        let first = s.spawn(|| resolver.resolve(codes[0].as_str()));
        let second = s.spawn(|| resolver.resolve(codes[1].as_str()));
        wait_until("two consistent reads in flight", || {
            reader.in_flight(Tier::Consistent) == 2
        });

        let (tx, rx) = mpsc::channel();
        let resolver = &resolver;
        let third = s.spawn(move || tx.send(resolver.resolve(codes[2].as_str())).unwrap());
        let mut third_answer = None;
        wait_until("the third lookup's answer or a third read", || {
            third_answer = rx.try_recv().ok();
            third_answer.is_some() || reader.in_flight(Tier::Consistent) > 2
        });
        assert_eq!(
            reader.in_flight(Tier::Consistent),
            2,
            "a third consistent read went out"
        );
        assert_eq!(resolver.stats().consistent_rejected, 1);
        hold.release();
        join(third);
        (join(first), join(second), third_answer)
    });

    assert_eq!(third, Some(Resolution::Unavailable));
    assert_eq!(first, redirect("http://0.example/"));
    assert_eq!(second, redirect("http://1.example/"));
    assert_eq!(
        resolver.resolve(codes[3].as_str()),
        redirect("http://3.example/")
    );
    assert_eq!(reader.calls(Tier::Consistent), 3);
}

/// Proves NFR4: a step 4 slot is returned on every exit, error, absent and
/// found, so with F = 1 lookups keep being confirmed.
#[test]
fn step4_slot_is_released_on_error() {
    let (failed, absent, found, after) = (
        code("slotfail"),
        code("slotnone"),
        code("slotfind"),
        code("slotnext"),
    );
    let reader = FakeReader::new();
    reader.put_on(Tier::Consistent, &failed, live("http://failed.example/"));
    reader.put_on(Tier::Consistent, &found, live("http://found.example/"));
    reader.put_on(Tier::Consistent, &after, live("http://after.example/"));
    let clock = ManualClock::at(START);
    let resolver = Resolver::new(&reader, clock, with_cap(1));

    reader.fail_next(Tier::Consistent, StoreError::Timeout);
    assert_eq!(resolver.resolve(failed.as_str()), Resolution::Unavailable);
    assert_eq!(resolver.resolve(absent.as_str()), Resolution::NotFound);
    assert_eq!(
        resolver.resolve(found.as_str()),
        redirect("http://found.example/")
    );
    assert_eq!(
        resolver.resolve(after.as_str()),
        redirect("http://after.example/")
    );
    assert_eq!(resolver.stats().consistent_rejected, 0);
    assert_eq!(reader.calls(Tier::Consistent), 4);
}
