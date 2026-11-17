//! Scripted doubles shared by the integration tests: a manual clock, a store
//! that counts, fails and gathers callers, a code source that hands out set
//! codes, and a reader with two scripted tiers and gates.

// Each test binary uses a different part of this module.
#![allow(dead_code)]

use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;

use url_shortener::{
    Clock, Code, CodeSource, InsertError, Link, LinkReader, LinkStore, MemStore, RngError,
    StoreError, SubmittedUrl, UrlKey,
};

/// Spins this many times at most before a wait gives up, so a wrong
/// implementation fails instead of hanging. Generous: a correct one meets
/// each wait within a few thread switches.
const SPIN_CAP: u64 = 20_000_000;

/// Spins until `cond` holds, yielding between checks; panics past the cap.
pub fn wait_until(what: &str, mut cond: impl FnMut() -> bool) {
    for _ in 0..SPIN_CAP {
        if cond() {
            return;
        }
        thread::yield_now();
    }
    panic!("gave up waiting for {what}");
}

/// Joins a scoped thread, re-raising its panic with its own message.
pub fn join<T>(handle: thread::ScopedJoinHandle<'_, T>) -> T {
    match handle.join() {
        Ok(value) => value,
        Err(payload) => std::panic::resume_unwind(payload),
    }
}

pub fn code(s: &str) -> Code {
    Code::parse(s).expect("a test code is 8 base62 characters")
}

/// The `i`th of a run of distinct test codes.
pub fn nth_code(i: usize) -> Code {
    code(&format!("t{i:07}"))
}

pub fn live(url: &str) -> Link {
    Link {
        url: url.into(),
        disabled: false,
    }
}

pub fn disabled(url: &str) -> Link {
    Link {
        url: url.into(),
        disabled: true,
    }
}

pub const TIMEOUT: Duration = Duration::from_millis(100);

/// The URL a consistent read of `code` finds, if any.
pub fn stored_url(reader: &impl LinkReader, code: &Code) -> Option<String> {
    reader
        .read_consistent(code, TIMEOUT)
        .expect("the in-memory store does not fail")
        .map(|link| link.url.to_string())
}

/// A time that only moves when a test sets it.
#[derive(Clone, Debug, Default)]
pub struct ManualClock(Arc<Mutex<Duration>>);

impl ManualClock {
    pub fn at(t: Duration) -> Self {
        Self(Arc::new(Mutex::new(t)))
    }

    pub fn set(&self, t: Duration) {
        *self.0.lock().unwrap() = t;
    }
}

impl Clock for ManualClock {
    fn now(&self) -> Duration {
        *self.0.lock().unwrap()
    }
}

/// Codes handed out in a set order, then `RngError` once they run out.
#[derive(Clone, Default)]
pub struct ScriptedCodes {
    queue: Arc<Mutex<VecDeque<Code>>>,
    draws: Arc<AtomicUsize>,
}

impl ScriptedCodes {
    pub fn new(codes: impl IntoIterator<Item = Code>) -> Self {
        Self {
            queue: Arc::new(Mutex::new(codes.into_iter().collect())),
            draws: Arc::default(),
        }
    }

    pub fn draws(&self) -> usize {
        self.draws.load(Ordering::SeqCst)
    }
}

impl CodeSource for ScriptedCodes {
    fn draw(&self) -> Result<Code, RngError> {
        self.draws.fetch_add(1, Ordering::SeqCst);
        self.queue.lock().unwrap().pop_front().ok_or(RngError)
    }
}

enum Fault {
    /// Fail before writing anything.
    Fail(InsertError),
    /// Write, then report the error: a commit whose reply was lost.
    CommitThenFail(StoreError),
}

/// A `MemStore` behind counters, scripted find and insert faults, and an
/// optional rendezvous that holds the first `n` absent `find_by_key` results
/// until all `n` have been read.
#[derive(Default)]
pub struct TestStore {
    pub mem: MemStore,
    faults: Mutex<VecDeque<Fault>>,
    find_faults: Mutex<VecDeque<StoreError>>,
    finds: AtomicUsize,
    inserts: AtomicUsize,
    rendezvous: usize,
    absent_reads: AtomicUsize,
}

impl TestStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_rendezvous(parties: usize) -> Self {
        Self {
            rendezvous: parties,
            ..Self::default()
        }
    }

    /// The next `find_by_key` fails with `err` without reading.
    pub fn fail_next_find(&self, err: StoreError) {
        self.find_faults.lock().unwrap().push_back(err);
    }

    pub fn fail_next_insert(&self, err: InsertError) {
        self.faults.lock().unwrap().push_back(Fault::Fail(err));
    }

    pub fn commit_then_fail_next_insert(&self, err: StoreError) {
        self.faults
            .lock()
            .unwrap()
            .push_back(Fault::CommitThenFail(err));
    }

    pub fn finds(&self) -> usize {
        self.finds.load(Ordering::SeqCst)
    }

    pub fn inserts(&self) -> usize {
        self.inserts.load(Ordering::SeqCst)
    }
}

impl LinkStore for TestStore {
    fn find_by_key(&self, key: &UrlKey) -> Result<Option<(Code, Link)>, StoreError> {
        self.finds.fetch_add(1, Ordering::SeqCst);
        if let Some(err) = self.find_faults.lock().unwrap().pop_front() {
            return Err(err);
        }
        let found = self.mem.find_by_key(key);
        if matches!(found, Ok(None)) {
            let arrived = self.absent_reads.fetch_add(1, Ordering::SeqCst) + 1;
            if arrived <= self.rendezvous {
                wait_until("every first submission to read the key absent", || {
                    self.absent_reads.load(Ordering::SeqCst) >= self.rendezvous
                });
            }
        }
        found
    }

    fn insert(&self, code: &Code, key: &UrlKey, url: &SubmittedUrl) -> Result<(), InsertError> {
        self.inserts.fetch_add(1, Ordering::SeqCst);
        let fault = self.faults.lock().unwrap().pop_front();
        match fault {
            None => self.mem.insert(code, key, url),
            Some(Fault::Fail(err)) => Err(err),
            Some(Fault::CommitThenFail(err)) => {
                self.mem.insert(code, key, url)?;
                Err(InsertError::Store(err))
            }
        }
    }
}

impl LinkReader for TestStore {
    fn read_local(
        &self,
        code: &Code,
        max_staleness: Duration,
        timeout: Duration,
    ) -> Result<Option<Link>, StoreError> {
        self.mem.read_local(code, max_staleness, timeout)
    }

    fn read_consistent(&self, code: &Code, timeout: Duration) -> Result<Option<Link>, StoreError> {
        self.mem.read_consistent(code, timeout)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tier {
    Local,
    Consistent,
}

impl Tier {
    fn index(self) -> usize {
        match self {
            Tier::Local => 0,
            Tier::Consistent => 1,
        }
    }
}

/// One read the resolver made, with the bounds it passed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Call {
    pub tier: Tier,
    pub code: Code,
    pub max_staleness: Option<Duration>,
    pub timeout: Duration,
}

#[derive(Default)]
struct ReaderState {
    views: [HashMap<Code, Link>; 2],
    faults: [VecDeque<StoreError>; 2],
    held: [bool; 2],
    in_flight: [usize; 2],
    calls: Vec<Call>,
}

/// A reader whose local replica and leaseholder hold separate contents, so
/// a test can put a code on one and not the other. A held tier takes its
/// answer when a read arrives, then blocks the read until the hold ends.
#[derive(Default)]
pub struct FakeReader {
    state: Mutex<ReaderState>,
    released: Condvar,
}

/// Holds a tier's reads while it lives; dropping it, on a panic too, lets
/// them return.
pub struct Hold<'a> {
    reader: &'a FakeReader,
    tier: Tier,
}

impl Hold<'_> {
    pub fn release(self) {}
}

impl Drop for Hold<'_> {
    fn drop(&mut self) {
        self.reader.state.lock().unwrap().held[self.tier.index()] = false;
        self.reader.released.notify_all();
    }
}

impl FakeReader {
    pub fn new() -> Self {
        Self::default()
    }

    /// Puts `link` on both tiers: a code the local replica has caught up on.
    pub fn put(&self, code: &Code, link: Link) {
        let mut st = self.state.lock().unwrap();
        st.views[0].insert(*code, link.clone());
        st.views[1].insert(*code, link);
    }

    /// Puts `link` on one tier only.
    pub fn put_on(&self, tier: Tier, code: &Code, link: Link) {
        self.state.lock().unwrap().views[tier.index()].insert(*code, link);
    }

    /// The next read on `tier` fails with `err`, whatever the tier holds.
    pub fn fail_next(&self, tier: Tier, err: StoreError) {
        self.state.lock().unwrap().faults[tier.index()].push_back(err);
    }

    pub fn hold(&self, tier: Tier) -> Hold<'_> {
        self.state.lock().unwrap().held[tier.index()] = true;
        Hold { reader: self, tier }
    }

    /// Reads on `tier` that have taken their answer and not yet returned.
    pub fn in_flight(&self, tier: Tier) -> usize {
        self.state.lock().unwrap().in_flight[tier.index()]
    }

    /// Reads made on `tier` so far.
    pub fn calls(&self, tier: Tier) -> usize {
        let st = self.state.lock().unwrap();
        st.calls.iter().filter(|c| c.tier == tier).count()
    }

    pub fn call_log(&self) -> Vec<Call> {
        self.state.lock().unwrap().calls.clone()
    }

    fn read(
        &self,
        tier: Tier,
        code: &Code,
        max_staleness: Option<Duration>,
        timeout: Duration,
    ) -> Result<Option<Link>, StoreError> {
        let i = tier.index();
        let mut st = self.state.lock().unwrap();
        st.calls.push(Call {
            tier,
            code: *code,
            max_staleness,
            timeout,
        });
        let answer = match st.faults[i].pop_front() {
            Some(err) => Err(err),
            None => Ok(st.views[i].get(code).cloned()),
        };
        st.in_flight[i] += 1;
        while st.held[i] {
            st = self.released.wait(st).unwrap();
        }
        st.in_flight[i] -= 1;
        answer
    }
}

impl LinkReader for FakeReader {
    fn read_local(
        &self,
        code: &Code,
        max_staleness: Duration,
        timeout: Duration,
    ) -> Result<Option<Link>, StoreError> {
        self.read(Tier::Local, code, Some(max_staleness), timeout)
    }

    fn read_consistent(&self, code: &Code, timeout: Duration) -> Result<Option<Link>, StoreError> {
        self.read(Tier::Consistent, code, None, timeout)
    }
}
