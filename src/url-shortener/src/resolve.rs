//! The Redirect Resolver's steps 2 to 4: the cache, the local follower read
//! and the bounded consistent read, with single-flight fills.
//!
//! See design-doc/url-shortener/rfc.md, Redirect Resolver.

use std::collections::HashMap;
use std::fmt;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use crate::code::Code;
use crate::store::{Link, LinkReader};

/// The browser's tail M (rfc.md, Estimates, Disable budget): what a `302`
/// tells clients to cache, whatever this server's T.
const REDIRECT_CACHE_CONTROL: &str = "private, max-age=60";

/// A monotonic clock, injected so tests set the time.
pub trait Clock: Send + Sync {
    /// Time since an arbitrary fixed origin.
    fn now(&self) -> Duration;
}

/// The resolver's bounds. Open Question 2 marks them as assumed; `Default`
/// holds the RFC's values.
///
/// See design-doc/url-shortener/rfc.md, Redirect Resolver; Open Questions, 2.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResolverConfig {
    /// T: a cache entry serves while younger than this.
    pub cache_ttl: Duration,
    /// S: the follower read's bound on replica staleness.
    pub max_staleness: Duration,
    /// Step 3's deadline.
    pub local_timeout: Duration,
    /// Step 4's deadline.
    pub consistent_timeout: Duration,
    /// F: step 4 reads in flight per server, at most.
    pub max_consistent_in_flight: usize,
}

impl Default for ResolverConfig {
    fn default() -> Self {
        Self {
            cache_ttl: Duration::from_secs(60),
            max_staleness: Duration::from_secs(30),
            local_timeout: Duration::from_millis(20),
            consistent_timeout: Duration::from_millis(100),
            max_consistent_in_flight: 64,
        }
    }
}

/// A lookup's answer.
///
/// See design-doc/url-shortener/rfc.md, API, `GET /{code}`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Resolution {
    /// `302` to the submitted URL (FR2).
    Redirect(Arc<str>),
    /// `410`: disabled (FR6).
    Gone,
    /// `404`: not a code, or a code no consistent read found.
    NotFound,
    /// `503`: step 4 failed, timed out or was over F.
    Unavailable,
}

impl Resolution {
    /// The HTTP status: 302, 410, 404 or 503.
    pub fn status(&self) -> u16 {
        match self {
            Resolution::Redirect(_) => 302,
            Resolution::Gone => 410,
            Resolution::NotFound => 404,
            Resolution::Unavailable => 503,
        }
    }

    /// `private, max-age=60` for a redirect (FR2, FR6's browser tail M),
    /// `no-store` for `410` and `404`, and none for `503`.
    ///
    /// See design-doc/url-shortener/rfc.md, API.
    pub fn cache_control(&self) -> Option<&'static str> {
        match self {
            Resolution::Redirect(_) => Some(REDIRECT_CACHE_CONTROL),
            Resolution::Gone | Resolution::NotFound => Some("no-store"),
            Resolution::Unavailable => None,
        }
    }
}

/// Counters the RFC lists among its metrics, which tests also use to see
/// what a lookup did.
///
/// See design-doc/url-shortener/rfc.md, Failure Modes and Observability.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ResolverStats {
    /// Lookups answered by step 2.
    pub cache_hits: u64,
    /// Waits on a fill of the lookup's code in flight, instead of reading;
    /// counted when each wait begins, so a lookup that waits twice counts
    /// twice.
    pub fill_joins: u64,
    /// Lookups that reached step 4 with F reads already in flight.
    pub consistent_rejected: u64,
}

/// Steps 2 to 4 of one server's redirect path. Step 1 is reduced to "not a
/// code is `404` with no store read"; cool-offs and miss limits are the
/// Abuse Limiter's.
///
/// - Step 2: an entry younger than T answers `302` or `410`. Only found
///   links are cached (NFR11).
/// - Step 3: `read_local` with S and its deadline. Found: fill and answer.
/// - Step 4: absent, error or timeout in step 3: `read_consistent`, at most F
///   in flight. Found: fill and answer. Absent: `404`. Error, timeout or over
///   F: `503`.
/// - Single flight: one fill per code at a time; lookups that arrive during
///   it wait for it. A fill shares a found link with every waiter. Its
///   absent or failed outcome may predate a waiter's arrival, since its read
///   may have begun before the code was created, so such a waiter goes back
///   to step 2 once: it joins the fill then in flight or leads a new one,
///   and takes that fill's outcome whatever it is, because that fill's read
///   began after the waiter arrived (NFR11). A lookup thus waits at most
///   twice, and a failed fill is followed by one more read per code, not one
///   per waiter (rfc.md, Why each tier: one store read per server per T).
///
/// See design-doc/url-shortener/rfc.md, Redirect Resolver.
pub struct Resolver<R, C> {
    reader: R,
    clock: C,
    config: ResolverConfig,
    state: Mutex<State>,
    consistent_in_flight: AtomicUsize,
    // Counters only: no other memory is published through them, so their
    // operations are `Relaxed`.
    cache_hits: AtomicU64,
    fill_joins: AtomicU64,
    consistent_rejected: AtomicU64,
}

impl<R, C> fmt::Debug for Resolver<R, C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Resolver")
            .field("config", &self.config)
            .field("stats", &self.counters())
            .finish_non_exhaustive()
    }
}

impl<R, C> Resolver<R, C> {
    fn counters(&self) -> ResolverStats {
        ResolverStats {
            cache_hits: self.cache_hits.load(Ordering::Relaxed),
            fill_joins: self.fill_joins.load(Ordering::Relaxed),
            consistent_rejected: self.consistent_rejected.load(Ordering::Relaxed),
        }
    }
}

/// What step 2 can answer from: found links only, so a local miss is never
/// cached (NFR11).
///
/// See design-doc/url-shortener/rfc.md, Redirect Resolver.
#[derive(Clone)]
enum Cached {
    Live(Arc<str>),
    Disabled,
}

impl Cached {
    fn of(link: Link) -> Self {
        if link.disabled {
            Cached::Disabled
        } else {
            Cached::Live(link.url)
        }
    }

    fn answer(&self) -> Resolution {
        match self {
            Cached::Live(url) => Resolution::Redirect(Arc::clone(url)),
            Cached::Disabled => Resolution::Gone,
        }
    }
}

struct Entry {
    cached: Cached,
    /// When the fill's leader passed step 2, before its reads began, so the
    /// entry's age bounds how old the store state it reflects is (rfc.md,
    /// Staleness).
    filled_at: Duration,
}

#[derive(Default)]
struct State {
    cache: HashMap<Code, Entry>,
    fills: HashMap<Code, Arc<Fill>>,
}

/// A fill in flight. Its outcome is `None` until the leader is done; each
/// waiter decides what of it to take.
#[derive(Default)]
struct Fill {
    outcome: Mutex<Option<Outcome>>,
    done: Condvar,
}

impl Fill {
    fn wait(&self) -> Outcome {
        let outcome = self
            .done
            .wait_while(lock(&self.outcome), |outcome| outcome.is_none())
            .unwrap_or_else(PoisonError::into_inner);
        outcome.clone().unwrap_or(Outcome::Failed)
    }
}

/// The leader's end of a fill. Dropping it, on a panic in the reader too,
/// caches what was found, removes the fill and wakes its waiters with the
/// outcome, so a failed leader never strands them. The fill is removed
/// before the waiters wake, so a fill a waiter finds afterwards began after
/// this one ended.
struct Leading<'a> {
    state: &'a Mutex<State>,
    code: Code,
    fill: Arc<Fill>,
    filled_at: Duration,
    // `Failed` until the reads return, which is what a panic leaves.
    outcome: Outcome,
}

impl Drop for Leading<'_> {
    fn drop(&mut self) {
        let outcome = std::mem::replace(&mut self.outcome, Outcome::Failed);
        {
            let mut state = lock(self.state);
            if let Outcome::Found(cached) = &outcome {
                let entry = Entry {
                    cached: cached.clone(),
                    filled_at: self.filled_at,
                };
                state.cache.insert(self.code, entry);
            }
            state.fills.remove(&self.code);
        }
        *lock(&self.fill.outcome) = Some(outcome);
        self.fill.done.notify_all();
    }
}

/// One of the F step 4 places, given back on drop.
struct Slot<'a>(&'a AtomicUsize);

impl Drop for Slot<'_> {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::Relaxed);
    }
}

/// Step 2's result: a cached answer, or the fill in flight for the code to
/// join, or a new fill to lead.
enum Role<'a> {
    Hit(Resolution),
    Join(Arc<Fill>),
    Lead(Leading<'a>),
}

/// What steps 3 and 4 found.
#[derive(Clone)]
enum Outcome {
    Found(Cached),
    Absent,
    Failed,
}

// Every critical section in this module is a few map or flag operations, so
// a panic elsewhere cannot leave the guarded state half-written.
fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(PoisonError::into_inner)
}

impl<R: LinkReader, C: Clock> Resolver<R, C> {
    /// A resolver with an empty cache, reading through `reader`.
    pub fn new(reader: R, clock: C, config: ResolverConfig) -> Self {
        Self {
            reader,
            clock,
            config,
            state: Mutex::default(),
            consistent_in_flight: AtomicUsize::new(0),
            cache_hits: AtomicU64::new(0),
            fill_joins: AtomicU64::new(0),
            consistent_rejected: AtomicU64::new(0),
        }
    }

    /// Answers `GET /{code}` for the path `code`.
    ///
    /// See design-doc/url-shortener/requirement.md, FR2, FR6, NFR4, NFR11.
    pub fn resolve(&self, code: &str) -> Resolution {
        let Some(code) = Code::parse(code) else {
            return Resolution::NotFound;
        };
        match self.step2(code) {
            Role::Hit(answer) => answer,
            Role::Lead(leading) => self.fill(leading),
            Role::Join(fill) => match fill.wait() {
                Outcome::Found(cached) => cached.answer(),
                // That fill's read may have begun before this code was
                // created, so its absent or failed outcome is not this
                // lookup's (NFR11). Whatever step 2 finds now began after
                // this lookup arrived.
                Outcome::Absent | Outcome::Failed => match self.step2(code) {
                    Role::Hit(answer) => answer,
                    Role::Lead(leading) => self.fill(leading),
                    Role::Join(fill) => answer(fill.wait()),
                },
            },
        }
    }

    /// The counters so far.
    pub fn stats(&self) -> ResolverStats {
        self.counters()
    }

    /// Step 2: a cached answer, else the fill in flight to join, else a new
    /// fill this lookup leads. The clock is read here, so a lookup back from
    /// a wait judges the cache at its own time and a fill it leads ages from
    /// the start of its read.
    fn step2(&self, code: Code) -> Role<'_> {
        let now = self.clock.now();
        let mut state = lock(&self.state);
        if let Some(entry) = state.cache.get(&code)
            && now.saturating_sub(entry.filled_at) < self.config.cache_ttl
        {
            self.cache_hits.fetch_add(1, Ordering::Relaxed);
            return Role::Hit(entry.cached.answer());
        }
        if let Some(fill) = state.fills.get(&code) {
            self.fill_joins.fetch_add(1, Ordering::Relaxed);
            return Role::Join(Arc::clone(fill));
        }
        let fill = Arc::<Fill>::default();
        state.fills.insert(code, Arc::clone(&fill));
        Role::Lead(Leading {
            state: &self.state,
            code,
            fill,
            filled_at: now,
            outcome: Outcome::Failed,
        })
    }

    /// Steps 3 and 4 for the fill `leading` owns; dropping it shares the
    /// outcome.
    fn fill(&self, mut leading: Leading<'_>) -> Resolution {
        let outcome = self.read_tiers(&leading.code);
        leading.outcome = outcome.clone();
        drop(leading);
        answer(outcome)
    }

    /// Steps 3 and 4. Any step 3 result but a found link goes to step 4.
    fn read_tiers(&self, code: &Code) -> Outcome {
        let local =
            self.reader
                .read_local(code, self.config.max_staleness, self.config.local_timeout);
        if let Ok(Some(link)) = local {
            return Outcome::Found(Cached::of(link));
        }
        let Some(_slot) = self.consistent_slot() else {
            self.consistent_rejected.fetch_add(1, Ordering::Relaxed);
            return Outcome::Failed;
        };
        match self
            .reader
            .read_consistent(code, self.config.consistent_timeout)
        {
            Ok(Some(link)) => Outcome::Found(Cached::of(link)),
            Ok(None) => Outcome::Absent,
            Err(_) => Outcome::Failed,
        }
    }

    fn consistent_slot(&self) -> Option<Slot<'_>> {
        let cap = self.config.max_consistent_in_flight;
        self.consistent_in_flight
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |n| {
                (n < cap).then_some(n + 1)
            })
            .ok()
            .map(|_| Slot(&self.consistent_in_flight))
    }
}

fn answer(outcome: Outcome) -> Resolution {
    match outcome {
        Outcome::Found(cached) => cached.answer(),
        Outcome::Absent => Resolution::NotFound,
        Outcome::Failed => Resolution::Unavailable,
    }
}

#[cfg(test)]
mod tests {
    use std::panic::{AssertUnwindSafe, catch_unwind};
    use std::sync::atomic::AtomicBool;

    use super::*;
    use crate::store::StoreError;

    struct Fixed(Duration);

    impl Clock for Fixed {
        fn now(&self) -> Duration {
            self.0
        }
    }

    /// Panics on its first local read, then finds every code.
    #[derive(Default)]
    struct PanicsOnce {
        panicked: AtomicBool,
    }

    impl LinkReader for PanicsOnce {
        fn read_local(
            &self,
            _code: &Code,
            _max_staleness: Duration,
            _timeout: Duration,
        ) -> Result<Option<Link>, StoreError> {
            if !self.panicked.swap(true, Ordering::SeqCst) {
                panic!("the reader failed");
            }
            Ok(Some(Link {
                url: "http://example.com/".into(),
                disabled: false,
            }))
        }

        fn read_consistent(
            &self,
            _code: &Code,
            _timeout: Duration,
        ) -> Result<Option<Link>, StoreError> {
            Ok(None)
        }
    }

    /// A leader that panics in the reader still removes its fill, so the
    /// next lookup reads instead of joining a fill that never ends.
    #[test]
    fn panicking_leader_leaves_no_fill_behind() {
        let resolver = Resolver::new(
            PanicsOnce::default(),
            Fixed(Duration::from_secs(1)),
            ResolverConfig::default(),
        );
        let first = catch_unwind(AssertUnwindSafe(|| resolver.resolve("panic001")));
        assert!(first.is_err());
        assert_eq!(
            resolver.resolve("panic001"),
            Resolution::Redirect("http://example.com/".into())
        );
        assert_eq!(resolver.stats().fill_joins, 0);
    }
}
