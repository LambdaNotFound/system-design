//! Link Creation over the in-memory store, with scripted codes and faults.
//!
//! See design-doc/url-shortener/rfc.md, Link Creation.

mod common;

use std::collections::HashSet;
use std::thread;

use common::{ScriptedCodes, TestStore, code, join, nth_code, stored_url};
use url_shortener::{
    Code, CodeSource, CreateError, Created, InsertError, LinkCreator, LinkStore, MemStore, OsCodes,
    Reject, RngError, StoreError, canonicalize_v1, code_from_bytes,
};

const BASE62: &str = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

/// Inserts `url` under `code` directly, as an earlier creation would have.
fn seed(store: &MemStore, code: &Code, url: &str) {
    let (submitted, key) = canonicalize_v1(url).expect("a valid seed URL");
    store
        .insert(code, &key, &submitted)
        .expect("a seed insert into an empty slot");
}

/// Proves FR1 and FR3: a first submission gets a new code, an FR4-equal
/// spelling gets the same code back, and the store keeps the first spelling.
#[test]
fn first_submission_creates_and_repeats_return_it() {
    let (c1, c2) = (code("first001"), code("second02"));
    let store = MemStore::new();
    let creator = LinkCreator::new(&store, ScriptedCodes::new([c1, c2]));

    assert_eq!(
        creator.create("HTTPS://Example.com:443/a/../b"),
        Ok(Created::New(c1))
    );
    assert_eq!(
        creator.create("https://example.com/b"),
        Ok(Created::Existing(c1))
    );
    assert_eq!(
        creator.create("HTTPS://Example.com:443/a/../b"),
        Ok(Created::Existing(c1))
    );
    assert_eq!(
        stored_url(&store, &c1).as_deref(),
        Some("HTTPS://Example.com:443/a/../b")
    );

    assert_eq!(
        creator.create("https://example.com/c"),
        Ok(Created::New(c2))
    );
    assert_eq!(*Created::New(c2).code(), c2);
    assert_eq!(*Created::Existing(c1).code(), c1);
}

/// Proves FR3 and FR2: first submissions of one URL that all read the key
/// absent before any inserts get one code, which redirects to the winner's
/// spelling.
#[test]
fn concurrent_first_submissions_get_one_code() {
    const SPELLINGS: [&str; 8] = [
        "http://example.com/x",
        "HTTP://example.com/x",
        "http://EXAMPLE.com/x",
        "http://example.com:80/x",
        "http://example.com/./x",
        "http://example.com/%78",
        "http://Example.com/x",
        "http://example.com/a/../x",
    ];
    let store = TestStore::with_rendezvous(SPELLINGS.len());
    let codes = ScriptedCodes::new((0..16).map(nth_code));
    let creator = LinkCreator::new(&store, codes);

    let results: Vec<(&str, Result<Created, CreateError>)> = thread::scope(|s| {
        let handles: Vec<_> = SPELLINGS
            .iter()
            .map(|&raw| {
                let creator = &creator;
                s.spawn(move || (raw, creator.create(raw)))
            })
            .collect();
        handles.into_iter().map(join).collect()
    });

    let winners: Vec<_> = results
        .iter()
        .filter(|(_, r)| matches!(r, Ok(Created::New(_))))
        .collect();
    assert_eq!(winners.len(), 1, "exactly one New: {results:?}");
    let (winner_spelling, winner) = winners[0];
    let winner_code = *winner.as_ref().unwrap().code();
    for (raw, result) in &results {
        assert_eq!(
            result.as_ref().map(|c| *c.code()),
            Ok(winner_code),
            "{raw:?} got another code"
        );
    }
    assert_eq!(
        stored_url(&store.mem, &winner_code).as_deref(),
        Some(*winner_spelling)
    );
}

/// Proves FR5 and NFR7: a drawn code that is taken is redrawn, and the taken
/// code keeps its URL. The store itself never overwrites either table.
#[test]
fn code_collision_redraws_and_never_overwrites() {
    let (taken, fresh) = (code("taken001"), code("fresh002"));
    let store = MemStore::new();
    seed(&store, &taken, "http://a.example/");

    let codes = ScriptedCodes::new([taken, fresh]);
    let creator = LinkCreator::new(&store, codes.clone());
    assert_eq!(creator.create("http://b.example/"), Ok(Created::New(fresh)));
    assert_eq!(codes.draws(), 2);
    assert_eq!(
        stored_url(&store, &taken).as_deref(),
        Some("http://a.example/")
    );
    assert_eq!(
        stored_url(&store, &fresh).as_deref(),
        Some("http://b.example/")
    );

    let (c_url, c_key) = canonicalize_v1("http://c.example/").unwrap();
    assert_eq!(
        store.insert(&taken, &c_key, &c_url),
        Err(InsertError::CodeTaken)
    );
    assert_eq!(
        stored_url(&store, &taken).as_deref(),
        Some("http://a.example/")
    );
    assert_eq!(
        store.find_by_key(&c_key),
        Ok(None),
        "a failed insert wrote the index"
    );

    let (_, a_key) = canonicalize_v1("http://a.example/").unwrap();
    let unused = code("unused03");
    assert_eq!(
        store.insert(&unused, &a_key, &c_url),
        Err(InsertError::KeyTaken)
    );
    assert_eq!(
        stored_url(&store, &unused),
        None,
        "a failed insert wrote a link"
    );
    assert_eq!(
        store.find_by_key(&a_key).map(|found| found.map(|(c, _)| c)),
        Ok(Some(taken))
    );
}

/// Proves FR1's bound on retries: the first draw and 5 redraws, and no more.
#[test]
fn five_redraws_then_gives_up() {
    let store = MemStore::new();
    let taken: Vec<Code> = (0..6).map(nth_code).collect();
    for (i, c) in taken.iter().enumerate() {
        seed(&store, c, &format!("http://seed{i}.example/"));
    }

    let free = code("free0001");
    let codes = ScriptedCodes::new(taken[..5].iter().copied().chain([free]));
    let creator = LinkCreator::new(&store, codes.clone());
    assert_eq!(
        creator.create("http://new1.example/"),
        Ok(Created::New(free))
    );
    assert_eq!(codes.draws(), 6);

    let spare = code("spare001");
    let codes = ScriptedCodes::new(taken.iter().copied().chain([spare]));
    let creator = LinkCreator::new(&store, codes.clone());
    assert_eq!(
        creator.create("http://new2.example/"),
        Err(CreateError::CodesExhausted)
    );
    assert_eq!(codes.draws(), 6);
    assert_eq!(stored_url(&store, &spare), None);
}

/// Proves FR9: once a code is disabled, its URL and every FR4-equal
/// spelling are rejected on resubmission.
#[test]
fn disabled_url_is_rejected_on_resubmission() {
    let c = code("disabl01");
    let store = MemStore::new();
    let creator = LinkCreator::new(&store, ScriptedCodes::new([c, code("unused02")]));

    assert_eq!(creator.create("http://example.com/x"), Ok(Created::New(c)));
    assert!(store.disable(&c));
    assert_eq!(
        creator.create("http://example.com/x"),
        Err(CreateError::Disabled)
    );
    assert_eq!(
        creator.create("HTTP://EXAMPLE.COM:80/x"),
        Err(CreateError::Disabled)
    );
    assert!(!store.disable(&code("nosuch01")));
}

/// Proves NFR7: an index entry whose row holds a URL with another key is an
/// integrity error, never an `Existing` answer and never a new link.
#[test]
fn index_to_a_foreign_url_is_an_integrity_error() {
    let c = code("foreign1");
    let store = TestStore::new();
    let (_, key_a) = canonicalize_v1("http://a.example/").unwrap();
    let (url_b, _) = canonicalize_v1("http://b.example/").unwrap();
    store.mem.insert(&c, &key_a, &url_b).unwrap();

    let creator = LinkCreator::new(&store, ScriptedCodes::new([code("unused01")]));
    assert_eq!(
        creator.create("http://a.example/"),
        Err(CreateError::Integrity)
    );
    assert_eq!(
        creator.create("HTTP://A.example:80/"),
        Err(CreateError::Integrity)
    );
    assert_eq!(store.inserts(), 0);
}

/// Proves FR1 under a slow store: a timed-out transaction is retried once,
/// a second timeout is returned, and other store errors are not retried.
#[test]
fn timed_out_transaction_is_retried_once() {
    let store = TestStore::new();
    store.fail_next_insert(InsertError::Store(StoreError::Timeout));
    let creator = LinkCreator::new(&store, ScriptedCodes::new((0..4).map(nth_code)));
    let created = creator.create("http://one.example/");
    let Ok(Created::New(c)) = created else {
        panic!("one timeout then success should be New, got {created:?}");
    };
    assert_eq!(store.inserts(), 2);
    assert_eq!(
        stored_url(&store.mem, &c).as_deref(),
        Some("http://one.example/")
    );

    let store = TestStore::new();
    store.fail_next_insert(InsertError::Store(StoreError::Timeout));
    store.fail_next_insert(InsertError::Store(StoreError::Timeout));
    let creator = LinkCreator::new(&store, ScriptedCodes::new((0..4).map(nth_code)));
    assert_eq!(
        creator.create("http://two.example/"),
        Err(CreateError::Store(StoreError::Timeout))
    );
    assert_eq!(store.inserts(), 2);

    let store = TestStore::new();
    store.fail_next_insert(InsertError::Store(StoreError::Unavailable));
    let creator = LinkCreator::new(&store, ScriptedCodes::new((0..4).map(nth_code)));
    assert_eq!(
        creator.create("http://three.example/"),
        Err(CreateError::Store(StoreError::Unavailable))
    );
    assert_eq!(store.inserts(), 1);
}

/// Proves FR1 and FR3 when Phase 1's read fails: a timed-out find is retried
/// within the submission's one retry, and a failed find is never taken for
/// an absent key, so nothing is inserted on it.
#[test]
fn failed_lookup_is_retried_once_and_never_read_as_absent() {
    let store = TestStore::new();
    store.fail_next_find(StoreError::Timeout);
    let creator = LinkCreator::new(&store, ScriptedCodes::new((0..4).map(nth_code)));
    let created = creator.create("http://find-once.example/");
    assert!(
        matches!(created, Ok(Created::New(_))),
        "one find timeout then success should be New, got {created:?}"
    );
    assert_eq!(store.finds(), 2);
    assert_eq!(store.inserts(), 1);

    let store = TestStore::new();
    store.fail_next_find(StoreError::Timeout);
    store.fail_next_find(StoreError::Timeout);
    let creator = LinkCreator::new(&store, ScriptedCodes::new((0..4).map(nth_code)));
    assert_eq!(
        creator.create("http://find-twice.example/"),
        Err(CreateError::Store(StoreError::Timeout))
    );
    assert_eq!(store.finds(), 2);
    assert_eq!(store.inserts(), 0);

    // The one retry covers the whole transaction, find and insert together.
    let store = TestStore::new();
    store.fail_next_find(StoreError::Timeout);
    store.fail_next_insert(InsertError::Store(StoreError::Timeout));
    let creator = LinkCreator::new(&store, ScriptedCodes::new((0..4).map(nth_code)));
    assert_eq!(
        creator.create("http://find-then-insert.example/"),
        Err(CreateError::Store(StoreError::Timeout))
    );
    assert_eq!(store.inserts(), 1);

    let store = TestStore::new();
    store.fail_next_find(StoreError::Unavailable);
    let codes = ScriptedCodes::new((0..4).map(nth_code));
    let creator = LinkCreator::new(&store, codes.clone());
    assert_eq!(
        creator.create("http://find-down.example/"),
        Err(CreateError::Store(StoreError::Unavailable))
    );
    assert_eq!(store.finds(), 1);
    assert_eq!(store.inserts(), 0);
    assert_eq!(codes.draws(), 0);

    // Against a key that exists, a failed find must not turn into an insert.
    let store = TestStore::new();
    let existing = code("exists01");
    seed(&store.mem, &existing, "http://exists.example/");
    store.fail_next_find(StoreError::Unavailable);
    let creator = LinkCreator::new(&store, ScriptedCodes::new((0..4).map(nth_code)));
    assert_eq!(
        creator.create("http://exists.example/"),
        Err(CreateError::Store(StoreError::Unavailable))
    );
    assert_eq!(store.inserts(), 0);
}

/// Proves FR3 and FR1 under a failing code source: a new URL is `Rng` with
/// nothing inserted, and a repeat submission still gets its code, since a
/// code is drawn only once Phase 1 finds the key absent.
#[test]
fn rng_failure_blocks_only_new_links() {
    let store = TestStore::new();
    let existing = code("exists02");
    seed(&store.mem, &existing, "http://exists.example/");
    let codes = ScriptedCodes::new([]);
    let creator = LinkCreator::new(&store, codes.clone());

    assert_eq!(
        creator.create("http://brand-new.example/"),
        Err(CreateError::Rng(RngError))
    );
    assert_eq!(store.inserts(), 0);

    assert_eq!(
        creator.create("HTTP://EXISTS.example:80/"),
        Ok(Created::Existing(existing))
    );
    assert_eq!(codes.draws(), 1);
}

/// Proves FR3 and FR1: when a timed-out insert had in fact committed, the
/// retry returns the code that committed, and no second code is written.
#[test]
fn timed_out_insert_that_committed_returns_its_code() {
    let store = TestStore::new();
    store.commit_then_fail_next_insert(StoreError::Timeout);
    let (c1, c2) = (code("commit01"), code("commit02"));
    let creator = LinkCreator::new(&store, ScriptedCodes::new([c1, c2, code("commit03")]));

    let created = creator
        .create("http://lost-reply.example/")
        .expect("the retry finds the committed link");
    assert_eq!(*created.code(), c1);
    assert_eq!(
        stored_url(&store.mem, &c1).as_deref(),
        Some("http://lost-reply.example/")
    );
    assert_eq!(stored_url(&store.mem, &c2), None);
}

/// Proves FR7: a rejected URL is answered with its API reason before any
/// code is drawn or the store is called.
#[test]
fn rejected_url_never_reaches_the_store() {
    let store = TestStore::new();
    let codes = ScriptedCodes::new([code("unused01")]);
    let creator = LinkCreator::new(&store, codes.clone());
    let too_long = format!("http://example.com/{}", "a".repeat(2_030));
    let cases = [
        ("ftp://example.com/", Reject::BadScheme, "bad_scheme"),
        ("http://example.com/a b", Reject::InvalidUrl, "invalid_url"),
        (too_long.as_str(), Reject::TooLong, "too_long"),
        ("http://s.example/", Reject::OwnDomain, "own_domain"),
        ("http://u@example.com/", Reject::Userinfo, "userinfo"),
    ];
    for (raw, reject, reason) in cases {
        assert_eq!(creator.create(raw), Err(CreateError::Rejected(reject)));
        assert_eq!(reject.reason(), reason);
    }
    assert_eq!(store.finds(), 0);
    assert_eq!(store.inserts(), 0);
    assert_eq!(codes.draws(), 0);
}

/// Proves NFR12 and the checkable part of NFR8: OS codes are 8 base62
/// characters, parse back to themselves, and do not repeat.
#[test]
fn drawn_codes_are_eight_base62_characters() {
    let mut seen = HashSet::new();
    for _ in 0..10_000 {
        let c = OsCodes.draw().expect("the OS random source works");
        let s = c.as_str();
        assert_eq!(s.len(), 8, "{s:?}");
        assert!(s.chars().all(|ch| BASE62.contains(ch)), "{s:?}");
        assert_eq!(Code::parse(s), Some(c));
        assert!(seen.insert(c), "{s:?} drawn twice");
    }
}

/// Proves NFR8's uniform mapping and NFR12: a kept byte gives the symbol at
/// `byte % 62`, bytes of 248 and more are redrawn rather than folded onto the
/// low symbols, and exactly 8 bytes are kept.
#[test]
fn code_bytes_map_to_base62_with_rejection() {
    let script = [248, 0, 61, 255, 62, 247, 10, 36, 124, 9, 1];
    let mut bytes = script.into_iter();
    let mut taken = 0;
    let c = code_from_bytes(|| {
        taken += 1;
        Ok(bytes.next().expect("the script ran out"))
    })
    .expect("a scripted source does not fail");
    assert_eq!(c.as_str(), "0z0zAa09");
    assert_eq!(taken, 10, "bytes taken: the 8 kept and the 2 redrawn");

    let mut bytes = [5, 6, 250].into_iter();
    assert_eq!(
        code_from_bytes(|| bytes.next().ok_or(RngError)),
        Err(RngError)
    );
}

/// Proves NFR12: only exactly 8 base62 characters parse, case kept.
#[test]
fn code_parse_accepts_only_eight_base62() {
    for s in ["abcd1234", "AbCd1234", "00000000", "zzzzzzzz"] {
        assert_eq!(
            Code::parse(s).map(|c| c.as_str().to_owned()),
            Some(s.to_owned())
        );
    }
    assert_ne!(Code::parse("abcd1234"), Code::parse("ABCD1234"));
    for s in [
        "",
        "abc1234",
        "abc123456",
        "abc-1234",
        "abc/1234",
        "abc 1234",
        "abcdéfg",
        "abcdéfgh",
        "v1/links",
    ] {
        assert_eq!(Code::parse(s), None, "{s:?}");
    }
}
