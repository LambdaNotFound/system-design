Covers: 0 amendments, 0 scope lines

# Implementation: URL Shortener

Brief: `design-doc/url-shortener/rfc.md` (approved by the design loop) and `design-doc/url-shortener/requirement.md` (`Status: signed-off 11/15/26`, no `## Amendment` sections). `impl-run.md` has no `Scope:` lines.

## Mechanisms

Three mechanisms, Work Breakdown Tasks 1 to 3, in dependency order. Tasks 4 to 7 (Abuse Limiter, Disables, Deployment, Metrics) are Follow-ups.

1. **Canonicalizer** (Task 1; FR4, FR7, FR9, FR10; `src/canon.rs`). The logic lives here because `canonicalize_v1`'s output is the `url_index` key: it is a data format frozen for ten years (rfc.md, Canonicalizer, Versioning), and Link Creation's NFR7 check re-runs it on stored rows. Steps, in order:
   1. **Convert (FR10).** Find the authority (after `scheme://`, up to the first `/`, `?` or `#`) and its host (after the last `@`, before `:port`, or a bracketed IP literal). A host containing non-ASCII goes through UTS #46 ToASCII, nontransitional; an IDNA error is `InvalidUrl`. The ToASCII call uses idna's WHATWG deny list (`AsciiDenyList::URL`: `%#/:<>?@[\]^|`, controls and space, checked after mapping), because without it a full-width `／`, `＠` or `：` maps to a delimiter and re-cuts the URL after it was read: under `AsciiDenyList::EMPTY`, `http://a／s.example/` was accepted with host `a` (R0, checked by flipping the list). A side effect: a non-ASCII host holding `%` is `InvalidUrl`. Every other non-ASCII character is percent-encoded as UTF-8 with uppercase hex. ASCII is never encoded, so a space or `<` stays and fails step 2. The result is the `SubmittedUrl`.
   2. **Validate (FR7).** A syntactically valid `scheme ":"` prefix is required (`InvalidUrl` without one); a scheme other than `http`/`https`, case-insensitive, is `BadScheme`; the whole string must match the RFC 3986 `URI` rule, IP literals included (`InvalidUrl`); then more than 2,048 characters is `TooLong`. Checked in that order.
   3. **Normalize (FR4).** In each component, decode percent-escapes of unreserved characters and uppercase the hex of the rest; then lowercase scheme and the host's characters outside the escapes that remain (so `%41` in a host ends as `a`, and `%2a` as `%2A`: RFC 3986 §3.2.2 and §6.2.2.1 keep percent-encodings uppercase in every component, read 11/16/26 at https://www.rfc-editor.org/rfc/rfc3986.txt); remove dot-segments (RFC 3986 §5.2.4) after the decode, so `%2E%2E` counts, in a URL with an authority only: a hostless path (D1) keeps them, since `http:/.//x/y` would become `http://x/y`, the form of another URL, and share its key (NFR7); drop the port when it is empty or its decimal value is 80 for http or 443 for https, so `:080` and `:0443` drop with it; an empty path with an authority becomes `/`. Query and fragment are kept, normalized only by the percent rules.

   Decision on ports: RFC 3986 §3.2.3 calls the port "an optional port number in decimal" and says to omit it when it is empty or the scheme's default (read 11/16/26; that the omit rule speaks of the port's *value* is from memory, since the fetch truncated the sentence). So the default is judged by value. No rule in §6.2.2 or §6.2.3 strips leading zeros from another port, so `:08080` keeps its digits and differs from `:8080` (FR4, "different otherwise"), and `:99999` is accepted as `*DIGIT` allows. A wrong call here costs a second code for one URL (FR3), never a misdirect (NFR7).
   4. **Reject (FR9), then hash.** Any `@` in the authority is `Userinfo`. The normalized host, minus one trailing dot for this check only, equal to `s.example` or ending in `.s.example` is `OwnDomain`. The key is SHA-256 of the canonical string.

   Decision the brief leaves to the literal text, the second of D1's class: FR9 judges the host after FR4, and an ASCII host is not converted (FR10), so a host of escapes that spell UTF-8, such as `%EF%BD%93.example` (U+FF53, full-width `s`), is judged as escaped and accepted. RFC 3986 §3.2.2 allows such a reg-name and says it is IDNA-transformed before name lookup (read 11/16/26), and a WHATWG parser decodes and maps it to `s.example` (from memory, not read this session), so it opens the service's own domain past FR9. The crate follows FR9's text and pins it in `canonicalize::percent_encoded_host_is_judged_as_escaped`. This is a brief correction for the user to decide, as D1 was; rejecting such hosts later is a step 4 rule and re-keys nothing.

   Decision the brief leaves to the literal text: FR7 says "valid under RFC 3986", and RFC 3986's grammar accepts `http:foo` (no authority) and `http:///x` (empty host). The plan follows the text and accepts both, and a test pins it. RFC 9110 §4.2.1 forbids an empty host in an `http` URI (cited from memory). The user accepted this at plan approval (impl-run.md, D1), with the brief correction left open: a WHATWG parser may open `http:///s.example/x` as `http://s.example/x`, past FR9's host check.

2. **Link Creation** (Task 2; FR1, FR3, FR5, FR9, NFR7, NFR8, NFR12; `src/create.rs`, `src/code.rs`). The logic lives in the handler, `LinkCreator::create`: Phase 1's `Disabled` and `Integrity` checks, the redraw on a code collision, the Phase 1 rerun on a lost key race, and the single retry of a timed-out transaction (rfc.md, Failure Modes, "The store is slow"). Departure from the RFC's `LinkStore::create`: with the four phases behind one store call, the in-memory store would hold all of the logic and the tests would test a test double. The trait is split into `find_by_key` (Phase 1's read) and `insert` (Phases 2 and 3, atomic: the transaction's only job). Phase 1 reading outside the insert's atomic step sees nothing a read inside it would miss: rows are insert-only and only `disabled_at` changes (FR5), and a disable committed before the read is seen. Retry counts as read: "retries, at most 5 times" is 5 redraws after the first draw, 6 inserts at most. The budget is per submission and every insert spends a draw, so a Phase 1 rerun after `KeyTaken` that finds the key absent draws from the same budget: a key race whose winner never becomes visible ends in `CodesExhausted` after 6 draws rather than looping (R1, R1-1; see Follow-ups). A code is drawn only after Phase 1 finds the key absent: drawing first, as the RFC's `create(key, url, fresh)` sketch does, would let a failing code source fail a repeat submission, which FR3 says gets its code. "Creation retries a timed-out transaction once" (Failure Modes) counts per submission: one retry, whether the find or the insert timed out, since both are one transaction. A find that fails is a store error, never an absent key. When the retry's Phase 1 finds the very code whose insert timed out, that insert committed, so the answer is `New` (the API's `201`), not `Existing`.

3. **Redirect Resolver** (Task 3; FR2, FR6, NFR4, NFR11; `src/resolve.rs`). Steps 2 to 4: the cache with T = 60 s, the local follower read with S = 30 s and a 20 ms timeout, the consistent read with a 100 ms timeout and at most F = 64 in flight per server, and the single-flight fill. Step 1's guard is reduced to "not a code: 404 with no store read"; the cool-off and miss limits are the Abuse Limiter's. The logic lives here because NFR11 and FR6's server bound are decided by what this code caches and what it shares. A refinement the RFC leaves implicit: **a single-flight fill shares a found link with every waiter, and its absent or failed outcome only with waiters that arrived before its read began.** A joiner that arrived after a code's creation returned, but while an older lookup's read was in flight, would otherwise take that read's `404` and break NFR11. So on absent, error or over the limit, a first-time waiter goes back to step 2 once: it joins the fill then in flight or leads a new one, and takes that fill's outcome whatever it is. That fill was registered after the old one was removed, which is after the waiter arrived, so its read is newer than the waiter's arrival and its `404` or `503` is valid for it. A lookup waits at most twice, and a failed fill costs one more read per code, not one per waiter, which keeps "one store read per server per T" (Why each tier) when the store is slow (R1, R1-2).

## Types

Files: `src/lib.rs` (re-exports), `src/canon.rs`, `src/code.rs`, `src/store.rs`, `src/create.rs`, `src/resolve.rs`.

```rust
// canon.rs: see rfc.md, Canonicalizer.
pub const SERVICE_DOMAIN: &str = "s.example";
pub struct SubmittedUrl(String);              // ASCII, FR7-valid; stored and sent in Location
impl SubmittedUrl { pub fn as_str(&self) -> &str; }
pub struct UrlKey([u8; 32]);                  // SHA-256 of the canonical form
impl UrlKey { pub fn as_bytes(&self) -> &[u8; 32]; }
pub enum Reject { BadScheme, InvalidUrl, TooLong, OwnDomain, Userinfo }
impl Reject { pub fn reason(&self) -> &'static str; } // "bad_scheme", ... (rfc.md, API)
/// Frozen: url_index keys depend on its output.
pub fn canonicalize_v1(raw: &str) -> Result<(SubmittedUrl, UrlKey), Reject>;
/// The same steps, returning the canonical string the key hashes.
pub fn canonical_form_v1(raw: &str) -> Result<(SubmittedUrl, String), Reject>;

// code.rs: see rfc.md, Link Creation; NFR8, NFR12.
pub struct Code([u8; 8]);                     // base62 ASCII
impl Code { pub fn parse(s: &str) -> Option<Code>; pub fn as_str(&self) -> &str; }
pub struct RngError;
pub trait CodeSource: Send + Sync { fn draw(&self) -> Result<Code, RngError>; }
/// byte < 248: ALPHABET[byte % 62]; byte >= 248: redrawn. 8 kept bytes.
pub fn code_from_bytes(next: impl FnMut() -> Result<u8, RngError>) -> Result<Code, RngError>;
pub struct OsCodes;                           // getrandom bytes through code_from_bytes
impl CodeSource for OsCodes { .. }

// store.rs: see rfc.md, Data model and storage; Link Creation; Redirect Resolver.
pub enum StoreError { Timeout, TooStale, Unavailable }
pub struct Link { pub url: Arc<str>, pub disabled: bool }
pub enum InsertError { CodeTaken, KeyTaken, Store(StoreError) }
pub trait LinkStore: Send + Sync {
    /// Phase 1: url_index[key], then links[code].
    fn find_by_key(&self, key: &UrlKey) -> Result<Option<(Code, Link)>, StoreError>;
    /// Phases 2 and 3, atomic: links[code] then url_index[key]; never overwrites (FR5).
    fn insert(&self, code: &Code, key: &UrlKey, url: &SubmittedUrl) -> Result<(), InsertError>;
}
pub trait LinkReader: Send + Sync {
    /// Follower read, nearest replica; TooStale past max_staleness.
    fn read_local(&self, code: &Code, max_staleness: Duration, timeout: Duration)
        -> Result<Option<Link>, StoreError>;
    /// Consistent read at the leaseholder.
    fn read_consistent(&self, code: &Code, timeout: Duration) -> Result<Option<Link>, StoreError>;
}
// Blanket impls of both traits for &T and Arc<T>, so one store serves a creator and a resolver.
pub struct MemStore { .. }                    // Mutex<HashMap>s; both reads see the latest state
impl MemStore { pub fn new() -> Self; pub fn disable(&self, code: &Code) -> bool; }
impl LinkStore for MemStore { .. }
impl LinkReader for MemStore { .. }

// create.rs: see rfc.md, Link Creation.
pub enum Created { New(Code), Existing(Code) }
impl Created { pub fn code(&self) -> &Code; }
pub enum CreateError { Rejected(Reject), Disabled, Integrity, CodesExhausted, Rng(RngError), Store(StoreError) }
pub struct LinkCreator<S, G> { .. }
impl<S: LinkStore, G: CodeSource> LinkCreator<S, G> {
    pub fn new(store: S, codes: G) -> Self;
    pub fn create(&self, raw: &str) -> Result<Created, CreateError>;
}

// resolve.rs: see rfc.md, Redirect Resolver.
pub trait Clock: Send + Sync { fn now(&self) -> Duration; } // monotonic, injected
pub struct ResolverConfig {
    pub cache_ttl: Duration,                  // T = 60 s
    pub max_staleness: Duration,              // S = 30 s
    pub local_timeout: Duration,              // 20 ms
    pub consistent_timeout: Duration,         // 100 ms
    pub max_consistent_in_flight: usize,      // F = 64
}
impl Default for ResolverConfig { .. }        // the RFC's values
pub enum Resolution { Redirect(Arc<str>), Gone, NotFound, Unavailable }
impl Resolution {
    pub fn status(&self) -> u16;              // 302, 410, 404, 503
    pub fn cache_control(&self) -> Option<&'static str>; // "private, max-age=60" | "no-store" | None
}
pub struct ResolverStats { pub cache_hits: u64, pub fill_joins: u64, pub consistent_rejected: u64 }
pub struct Resolver<R, C> { .. }
impl<R: LinkReader, C: Clock> Resolver<R, C> {
    pub fn new(reader: R, clock: C, config: ResolverConfig) -> Self;
    pub fn resolve(&self, code: &str) -> Resolution;
    pub fn stats(&self) -> ResolverStats;
}
```

Where the crate departs from the RFC's sketches, and why:

- `LinkStore` is split into `find_by_key` and `insert` (Mechanisms, 2). The RFC's `CodeTaken` and `KeyTaken` move to `InsertError`, since the handler retries them and no caller sees them. `CreateError` gains `Rejected`, `CodesExhausted` and `Rng`.
- `LinkReader`'s methods take a `timeout`. A blocking std call can't be cut off by its caller, so the deadline belongs to the store client, the way a statement timeout does.
- `canonical_form_v1` is added beside the frozen signature. The goldens must show which FR4 rule failed, and hex keys can't. A few literal hex keys still pin the hash.
- `Cached` stays crate-private: no caller needs the cache's entry type.
- `Link` carries `disabled: bool`, not `disabled_at` or `created_at`: the timestamps serve the Disables mechanism (NFR9), which this crate leaves out.
- `ResolverStats` exists because the single-flight test must see that a lookup joined a fill rather than read the store, and the RFC lists the cache hit rate and the step 4 rejections over F among its metrics. Its counters are those metrics.
- `ResolverConfig` exists because Open Question 2 marks T, S, F and the timeouts as assumed, and the concurrency tests need a small F. Its `Default` holds the RFC's values.
- `CodeSource` is a trait so a test can force `CodeTaken`. `Clock` is a trait so tests set time (impl-spec, Scope).
- `code_from_bytes` is public so a deterministic test can check the byte-to-symbol mapping and its rejection sampling, the crate's own share of NFR8; `OsCodes` only feeds it `getrandom` bytes.

## Test plan

Integration tests in `src/url-shortener/tests/`: `canonicalize.rs`, `create.rs`, `resolve.rs`, `failed_fill.rs` (added in R1), with the scripted doubles (`ManualClock`, gated and failing stores and readers, scripted code sources) in `tests/common/mod.rs`.

| Test | Proves | Kind |
| --- | --- | --- |
| `canonicalize::golden_canonical_forms_v1`: a table of input, submitted and canonical strings, one or more rows per step 3 rule: scheme case, host case, hex case in the path and in the host (`A%2ab` to `a%2Ab`), unreserved decode (`%7E`, `%41`), reserved escape kept (`%2F`), dot-segments incl. `%2E%2E`, ports 80/443 dropped, `:080` and `:0443` dropped by value, 8080, `:08080` and `:99999` kept, empty port dropped, empty path `/`, query and fragment kept, nontransitional `faß` to `xn--fa-hia` | FR4, FR10 | behavior |
| `canonicalize::golden_url_keys_v1`: four literal SHA-256 hex keys, computed outside the crate, `faß` among them | FR4 | behavior |
| `canonicalize::equal_urls_share_a_key_and_different_ones_do_not`: the RFC's Example pair, `%2a` vs `%2A` in a host, `:080` vs none and `:0443` vs none on https are equal; `/A` vs `/a`, `%2F` vs `/`, `/a` vs `/a/`, http vs https, `:8080` vs none, `:08080` vs `:8080`, `faß` vs `fass`, query differing are not | FR4 | boundary |
| `canonicalize::submitted_url_is_a_fixed_point`: for every golden input, canonicalizing the submitted URL returns the same submitted URL and key, which Phase 1's NFR7 re-check depends on | FR3, NFR7 | behavior |
| `canonicalize::submitted_url_keeps_the_callers_spelling`: `HTTPS://Example.com:443/a/../b` is submitted unchanged; an ASCII host keeps its case | FR2, FR10 | behavior |
| `canonicalize::non_ascii_host_goes_through_idna`: `Bücher.example` becomes `xn--bcher-kva.example`; composed and decomposed `ü` give the same host; `faß.example` becomes `xn--fa-hia.example`, not `fass.example`; full-width `ｓ.example` maps to the own domain and is `OwnDomain` | FR10, FR9 | behavior |
| `canonicalize::non_ascii_elsewhere_is_percent_encoded_as_utf8`: `/café?q=ü#ß` becomes `/caf%C3%A9?q=%C3%BC#%C3%9F` | FR10 | behavior |
| `canonicalize::scheme_must_be_http_or_https`: `HtTpS` accepted; `ftp`, `javascript`, `mailto`, `example.com:80/x` are `BadScheme`; no scheme is `InvalidUrl` | FR7 | boundary |
| `canonicalize::invalid_rfc3986_is_rejected`: space, `<`, `%zz`, trailing `%4`, unclosed `[`, `[::g]`, a non-digit port, an IDNA-invalid host are all `InvalidUrl` | FR7 | failure |
| `canonicalize::ip_literals_are_hosts`: `[::1]:8080`, `[2001:DB8::1]` (lowercased in the canonical form), `127.0.0.1` accepted | FR7, FR4 | behavior |
| `canonicalize::length_is_judged_after_conversion`: 2,048 characters accepted, 2,049 `TooLong`; a URL under 2,048 characters raw whose non-ASCII expands past it is `TooLong` | FR7, FR10 | boundary |
| `canonicalize::own_domain_and_subdomains_are_rejected`: `s.example`, `S.EXAMPLE`, `s.example.`, `a.b.s.example`, `%73.example` are `OwnDomain`; `nots.example`, `s.example.com`, `s.example..` are accepted | FR9 | boundary |
| `canonicalize::percent_encoded_host_is_judged_as_escaped`: `%ef%bd%93.example` is accepted with canonical host `%EF%BD%93.example`; `%c3%bc` and `%C3%BC` hosts share a key, which differs from `ü.example`'s; `%73.example` is `OwnDomain` (Mechanisms, 1, the FR9 decision) | FR9, FR4 | boundary |
| `canonicalize::any_userinfo_is_rejected`: `user@`, `user:pw@`, `:@`, a bare `@`, `http://s.example@evil.com/` are `Userinfo` | FR9 | boundary |
| `canonicalize::hostless_urls_follow_rfc3986_literally`: `HTTP:foo`, `http:///x` and `http:///s.example/x` are accepted (Mechanisms, 1; impl-run.md, D1) | FR7 | boundary |
| `create::first_submission_creates_and_repeats_return_it`: `New(c)`, then `Existing(c)` for an FR4-equal different spelling; the store keeps the first spelling | FR1, FR3 | behavior |
| `create::concurrent_first_submissions_get_one_code`: 8 threads, FR4-equal different spellings, a store that holds every `find_by_key` at a barrier until all 8 have read absent; exactly one `New`, every code equal, and the code resolves to the `New` caller's spelling | FR3, FR2 | concurrency |
| `create::code_collision_redraws_and_never_overwrites`: the code source first returns a taken code; the result is `New` with the second code, and the taken code still resolves to its own URL | FR5, NFR7 | failure |
| `create::five_redraws_then_gives_up`: 5 collisions then a free code is `New` after 6 draws; 6 collisions are `CodesExhausted` after 6 draws | FR1 | boundary |
| `create::disabled_url_is_rejected_on_resubmission`: create, disable, resubmit an FR4-equal spelling: `Disabled` | FR9 | behavior |
| `create::index_to_a_foreign_url_is_an_integrity_error`: `url_index[key(A)]` pointing at a row holding B; submitting A is `Integrity`, never `Existing` | NFR7 | failure |
| `create::timed_out_transaction_is_retried_once`: one `Timeout` then success is `New`; two are `Store(Timeout)`; `Unavailable` is not retried | FR1 | failure |
| `create::timed_out_insert_that_committed_returns_its_code`: an insert that commits and then reports `Timeout`; the retry returns the committed code and writes no second code | FR3, FR1 | failure |
| `create::failed_lookup_is_retried_once_and_never_read_as_absent`: one find `Timeout` then `New`; two are `Store(Timeout)` with no insert; a find `Timeout` and an insert `Timeout` are `Store(Timeout)`, the one retry spent; a find `Unavailable` is `Store(Unavailable)` with no draw and no insert, also against a key that exists | FR1, FR3 | failure |
| `create::rng_failure_blocks_only_new_links`: with an empty code source a new URL is `Rng(RngError)` with no insert, and an FR4-equal spelling of a stored URL is still `Existing`, with one draw in all | FR3, FR1 | failure |
| `create::rejected_url_never_reaches_the_store`: an invalid URL is `Rejected` with its reason and the store records no call | FR7 | failure |
| `create::drawn_codes_are_eight_base62_characters`: 10,000 `OsCodes` draws are 8 characters from the base62 alphabet, round-trip through `Code::parse`, and are distinct | NFR12, NFR8 | boundary |
| `create::code_bytes_map_to_base62_with_rejection`: scripted bytes through `code_from_bytes`: 0 and 61 give `0` and `z`, 62 and 124 give `0`, 247 gives `z`, 248 and 255 are redrawn, 10 bytes taken for 8 kept; a source error is `RngError` | NFR8, NFR12 | boundary |
| `create::code_parse_accepts_only_eight_base62`: 7 and 9 characters, `-`, `/`, non-ASCII, `v1/links` are `None` | NFR12 | boundary |
| `resolve::redirects_to_the_exact_submitted_spelling`: create, then resolve: `302`, `Location` the submitted spelling, `private, max-age=60` | FR2, FR6 | behavior |
| `resolve::local_miss_falls_through_to_a_consistent_read`: local absent, consistent found: `302`; again at T minus 1 ms, a cache hit with no read on either tier | NFR11, NFR1 | behavior |
| `resolve::expired_entry_is_not_served_while_the_store_fails`: filled live at t; at t + T, local `TooStale` and consistent `Timeout` give `503`; with F = 1 held by another code, local `Timeout` gives `503` and `consistent_rejected` 1 | FR6, NFR11 | failure |
| `resolve::local_error_falls_through_to_a_consistent_read`: local `TooStale`, then `Timeout`: each `302` from the consistent read | NFR11 | failure |
| `resolve::absent_code_is_never_cached`: absent everywhere at t is `404` `no-store`; the code is created; at the same t it is `302` | NFR11 | boundary |
| `resolve::consistent_failure_is_503_and_not_cached`: local absent, consistent `Timeout`: `503`; the next lookup reads again | NFR4, NFR11 | failure |
| `resolve::not_a_code_is_404_without_a_store_read`: `abc`, `v1/links`, 9 characters: `404`, zero reader calls | NFR12 | boundary |
| `resolve::reads_carry_the_designs_bounds`: the reader sees `max_staleness` 30 s and timeout 20 ms locally, timeout 100 ms consistently | FR6, NFR3 | behavior |
| `resolve::cache_serves_until_ttl_then_rereads`: filled at 0; a hit at T minus 1 ms with no read; a read at T | FR6 | boundary |
| `resolve::disabled_is_cached_as_gone`: a disabled link is `410` `no-store`, served from cache with no read within T | FR6 | behavior |
| `resolve::disable_takes_effect_within_staleness_plus_ttl`: disabled at 0; the local replica shows it live until S; filled at S minus 1 ms as live, `302` at S + T minus 2 ms, `410` at S + T minus 1 ms, within the 90 s server budget | FR6 | boundary |
| `resolve::viral_code_is_read_once_per_fill`: the first lookup's local read blocks; 7 more lookups of the code wait until `stats().fill_joins == 7` or the reader counts a second call (fail); on release all 8 get `302` and the reader counted 1 read | NFR1 | concurrency |
| `resolve::joiners_never_get_another_codes_url`: two codes' fills blocked together, joiners on each; each thread gets its own code's URL | NFR7 | concurrency |
| `resolve::negative_fill_is_not_shared`: the leader's consistent read snapshots absent and blocks; the code is created; a joiner then joins; on release the leader gets `404` and the joiner `302` | NFR11 | concurrency |
| `resolve::consistent_reads_are_capped_per_server`: F = 2; two lookups block in step 4 (the reader counts 2 in flight); a third is `503` at once and `consistent_rejected` is 1; on release both are `302`, and a fourth lookup proceeds | NFR4 | concurrency |
| `resolve::step4_slot_is_released_on_error`: F = 1; a consistent `Timeout` gives `503`; the next lookup gets a slot and `302` | NFR4 | failure |
| `failed_fill::waiters_of_a_failed_fill_share_one_refill` (R1): the leader's step 3 and step 4 time out while 7 lookups wait; the second fill's local read is held until the 6 others join it; the leader is `503`, all 7 waiters `302`, 2 local reads and 1 consistent | NFR1, rfc.md Why each tier | concurrency |
| `failed_fill::waiters_take_the_refills_failure` (R1): the same, with the second fill failing at both tiers: all 7 waiters `503`, 2 local and 2 consistent reads | NFR4, rfc.md Failure Modes | concurrency |

No test sleeps or reads the wall clock: time is a `ManualClock`, and the concurrency tests wait on a channel or a `thread::yield_now` loop over an observable counter with an iteration cap, so a wrong implementation fails instead of hanging. The rendezvous in `create::concurrent_first_submissions_get_one_code` is such a loop, not a `Barrier`, for the same reason. A held reader tier is released when its guard drops, so a failing assertion does not leave threads blocked inside `thread::scope`.

## Not proven here

- **NFR1, NFR2.** 11,600 creations a second and 365 billion stored links are properties of the cluster; Task 2's peak load test proves them. The crate proves only that a viral code costs one read per fill.
- **NFR3, NFR5.** p99 latencies need production load. The crate shows only that each read carries the RFC's timeout.
- **NFR4, NFR5 availability.** The percentages need production. That redirects outlive creation failures is structural: the `Resolver` holds only a `LinkReader`.
- **NFR6.** Cross-region durability is CockroachDB's 2+2+1 replication; `MemStore` is one process.
- **FR3 against the real store.** The crate proves the handler's response to `KeyTaken`. That CockroachDB raises a duplicate-key or serialization error for two concurrent inserts of one key is the RFC's assumed row (design.md, Evidence, "assumed"), for a test against the cluster.
- **NFR8.** Unpredictability rests on the OS CSPRNG through `getrandom`, which no in-process test can check. The crate's own share, the uniform byte-to-symbol mapping with rejection sampling, is tested through `code_from_bytes`. The cool-off half belongs to the Abuse Limiter, which this crate leaves out.
- **FR6, operator-only and end to end.** The Admin API and the Disable Tracker are not built. The browser's M = 60 s is only the header's value, and the S bound is the store's, so the crate proves the server's S + T given a replica within S.
- **NFR11 across servers.** The crate proves one server's rule: a local miss is never final, never cached and never shared. "Every server" also needs the leaseholder read to be linearizable, which is the store's.
- **FR5 over ten years.** Structural: neither trait has an update or delete, and `insert` never overwrites (tested).
- **FR1's "no account"** and the query string being ignored belong to the HTTP layer, which is not built.
- **FR8, NFR9, NFR10.** No chosen mechanism serves them (Follow-ups).

## Dependencies

- `idna` 1.1.0 (`Cargo.lock`; `idna_adapter` 1.2.2): UTS #46 ToASCII, nontransitional, for FR10's host conversion. The mapping and normalization tables can't reasonably be hand-written, and they are part of the frozen v1 format. Read in the 1.1.0 source from the crates.io registry on 11/16/26: `src/uts46.rs` line 20 says Transitional_Processing is "Always _false_", so every entry point is nontransitional; `Uts46::to_ascii` takes an `AsciiDenyList`, `Hyphens` and `DnsLength` (line 527); `domain_to_ascii_cow` in `src/lib.rs` uses `Hyphens::Allow` and `DnsLength::Ignore`. The docs page for that version is https://docs.rs/idna/1.1.0/idna/ (not fetched). The `url` crate is not used: it implements the WHATWG URL Standard, whose parsing and normalization are not RFC 3986 §6's (from memory).
- `sha2` 0.11.1: SHA-256 for `UrlKey`; std has no hash of that kind.
- `getrandom` 0.4.3: the OS CSPRNG behind `OsCodes` (NFR8); std has no CSPRNG. `getrandom::fill(&mut [u8])` in `src/lib.rs` line 87 of the 0.4.3 source.

No async runtime: the mechanisms are tier and retry logic over blocking traits, and real threads with barriers force the interleavings deterministically.

## Checks

```sh
cargo fmt -p url-shortener -- --check
cargo clippy -p url-shortener --all-targets -- -D warnings
cargo test -p url-shortener --no-fail-fast
```

## Follow-ups

- **Abuse Limiter (Task 4).** The step 1 guard, identities, counters, cool-offs, the shed limit and the 48/16 step 4 slot split. With it, `Resolution` needs a miss/shed classification, which step 4's text defines.
- **Disables (Task 5).** The Admin API, the `disables` row with `disabled_at`, the Disable Tracker, and `/internal/lookup/{code}` with the cache untouched.
- **Deployment and metrics (Tasks 6, 7).** The health endpoint and draining, PROXY protocol, the alerts. `ResolverStats` carries only three counters.
- **Cache bound.** The RFC sets no size limit. This cache keeps one entry for every live code looked up and refills expired entries without purging them, so a server's memory grows with the distinct codes it has served. Production needs eviction.
- **`KeyTaken` reruns share the draw budget.** The RFC bounds code redraws but not Phase 1 reruns. The crate bounds both with one per-submission budget of 6 draws (Mechanisms, 2), so a key race that never settles, as when a CockroachDB serialization conflict aborts both racers each time, ends in `CodesExhausted`, the same answer as a code space that keeps colliding. If the metrics need to tell contention from code exhaustion, the fix is a separate rerun bound with its own error; the RFC would have to name the bound.
- **Percent-encoded hosts.** `http://%EF%BD%93.example/` is accepted per FR9's text (Mechanisms, 1). The brief correction stays open for the user: amend FR9 to judge the host after decoding and ToASCII, or reject hosts whose escapes decode to non-ASCII. Either is a step 4 rule and re-keys nothing.
- **Non-ASCII hosts holding `%`.** The WHATWG deny list makes `http://ü%41.example/` `InvalidUrl` while the ASCII `http://u%41.example/` is accepted (Mechanisms, 1). Allowing `%` would need a deny list without it plus a check that no mapped character is a delimiter; either way it is a step 1 rule, frozen with v1.
- **Hostless http(s) URLs.** Accepted per FR7's literal text, as the user decided at approval (impl-run.md, D1). The brief correction stays open: amend FR7 through `/new-design url-shortener`; rejecting them later re-keys nothing.

## Revision log

- T0: 11/16/26. Created `src/Cargo.toml` (workspace) and `src/url-shortener/` with `cargo new --lib`, added `idna`, `sha2` and `getrandom` with `cargo add` (`src/Cargo.lock`). Stubs in `src/url-shortener/src/{lib,canon,code,store,create,resolve}.rs` with every logic body `todo!()`. Real bodies, each because no test can be written without it or because it holds no logic: `SubmittedUrl::as_str`, `UrlKey::as_bytes` and `Code::as_str` (the only way to read those values), `Debug for Code` (prints the string, for failure messages), `MemStore::new` and its `Default`, `LinkCreator::new`, `Resolver::new`, `ResolverConfig::default` (the RFC's constants), and the forwarding impls of `LinkStore` and `LinkReader` for `&T` and `Arc<T>`. Tests: `tests/canonicalize.rs` (14), `tests/create.rs` (11), `tests/resolve.rs` (15), doubles in `tests/common/mod.rs`. Against the plan: added `create::timed_out_insert_that_committed_returns_its_code` (a lost commit reply is the case where a retry could write a second code); `resolve::negative_fill_is_not_shared` creates the code before the joiner joins, the order Mechanisms, 3 describes, since a joiner that arrived before the creation may legally get `404`; the rendezvous is a capped spin, not a `Barrier`. `timed_out_transaction_is_retried_once` does not fix which code the retry uses. No signature changed. Golden SHA-256 keys computed with `printf '%s' <form> | shasum -a 256`; the punycode and the IDNA-invalid hosts checked against idna 1.1.0 in a scratch crate outside the repository. Ran from `src/`: `cargo fmt -p url-shortener -- --check` exit 0; `cargo clippy -p url-shortener --all-targets` exit 0, warnings only for the stubs' unused parameters and fields; `cargo test -p url-shortener --no-fail-fast` exit 101: 40 run, 0 passed, 40 failed, all 40 panics `not yet implemented`. Most create and resolve tests reach their first `todo!()` in setup, at `Code::parse` or `canonicalize_v1` on the main thread, so the doubles and the wait loops run for the first time in the implementation stage. The lead's record is `impl-checks/T0.md`.
- T1: 11/16/26. Answered tests round 1 (Objection responses). `src/url-shortener/src/code.rs`: `code_from_bytes` added with a `todo!()` body, `OsCodes`' doc points to it; `src/lib.rs` re-exports it; `src/create.rs`: doc line on `CreateError::Rng`, `LinkCreator`'s doc states the draw order and the per-submission retry. `tests/common/mod.rs`: `TestStore::fail_next_find`. `tests/canonicalize.rs`: golden rows for host hex case, ports with leading zeros and out of range, and `faß`; a fourth literal key; new `same_key` pairs; new test `percent_encoded_host_is_judged_as_escaped`. `tests/create.rs`: new `failed_lookup_is_retried_once_and_never_read_as_absent`, `rng_failure_blocks_only_new_links`, `code_bytes_map_to_base62_with_rejection`. `tests/resolve.rs`: new `expired_entry_is_not_served_while_the_store_fails`; `local_miss_falls_through_to_a_consistent_read` extended. The new key and punycode checked with idna 1.1.0 and sha2 0.11.1 in a scratch crate outside the repository, and with `printf '%s' http://xn--fa-hia.example/ | shasum -a 256`. Ran from `src/`: `cargo fmt -p url-shortener -- --check` exit 0; `cargo clippy -p url-shortener --all-targets` exit 0, warnings only for the stubs' unused parameters and fields; `cargo test -p url-shortener --no-fail-fast`: 45 run, 0 passed, 45 failed, all 45 panics `not yet implemented`.

- R0: 11/16/26. Filled every `todo!()` in `src/url-shortener/src/{canon,code,store,create,resolve}.rs`; no file under `tests/` changed (their timestamps predate the pass, and `cargo fmt -p url-shortener -- --check` exits 0 on them). `canon.rs`: `split` and `split_authority` cut the components; `convert` (FR10, ToASCII with `AsciiDenyList::URL`); `validate` (FR7: scheme prefix, scheme, `follows_uri_rule` with the RFC 3986 character classes, `is_ipv6`, `is_ipv4`, `IPvFuture`, then length); `normalize_and_judge` (FR4, then FR9's userinfo and own-domain checks); `remove_dot_segments` per §5.2.4. Two choices beyond the plan, both written into Mechanisms, 1: the deny list, and no dot-segment removal in a hostless path. `code.rs`: `Code::parse`, `code_from_bytes`, `OsCodes::draw` over a 16-byte `getrandom::fill` buffer. `store.rs`: `MemStore` under one `Mutex`, poison recovered. `create.rs`: the Phase 1 / insert loop with one timeout retry per submission and at most 6 draws per submission; `New` when the retry finds its own timed-out code (Mechanisms, 2); `CreateError::Rng`'s doc now says a failed draw is not retried (T2-1). `resolve.rs`: cache and fill map under one `Mutex`, a `Leading` guard whose drop caches a found link, removes the fill and wakes the joiners, also on a reader panic; joiners on an unshared outcome run steps 3 and 4 themselves; step 4 slots through `AtomicUsize::try_update` (rustc 1.99.0 deprecates `fetch_update` under that name); an entry's age counts from the start of the lookup that filled it. Unit tests added: `canon::tests::{hostless_path_never_turns_into_an_authority, mapped_delimiters_in_a_host_are_invalid, ip_literal_grammar, remove_dot_segments_matches_the_rfc_examples}`, `create::tests::own_commit_found_on_retry_is_new`, `resolve::tests::panicking_leader_leaves_no_fill_behind`. `mapped_delimiters_in_a_host_are_invalid` fails with `AsciiDenyList::EMPTY` in place of `URL` (run once, then reverted). Ran from `src/`: `cargo fmt -p url-shortener -- --check` exit 0; `cargo clippy -p url-shortener --all-targets -- -D warnings` exit 0; `cargo test -p url-shortener --no-fail-fast` exit 0, 51 passed (6 unit, 15 canonicalize, 14 create, 16 resolve); `cargo test -q -p url-shortener --test resolve --test create` 200 times in a loop, 0 failing runs. The lead's record is `impl-checks/R0.md`.
- R1: 11/16/26. Answered review round 1 (Objection responses). `src/url-shortener/src/create.rs`: `CodesExhausted`, `MAX_DRAWS` and `LinkCreator` docs say the draw budget also bounds `KeyTaken` reruns; doc on `LinkCreator::new`; a manual `Debug` for `LinkCreator` that prints no field; unit test `create::tests::key_race_that_never_settles_exhausts_the_draws`. `src/url-shortener/src/resolve.rs`: `Fill` now carries the full `Outcome`; `Leading` holds an `Outcome` (`Failed` until the reads return); `resolve` split into `step2` (cache, join or lead, reading the clock) and `fill`; a waiter on a non-found outcome goes through `step2` once more and takes the next fill's outcome, replacing the per-joiner `read_tiers`; `Resolver`'s and `ResolverStats::fill_joins`' docs updated; `Entry::filled_at` doc; doc on `Resolver::new`; a manual `Debug` for `Resolver` printing `config` and the counters. `src/url-shortener/src/store.rs`: the `tables` comment rewritten (R1-3); docs on `Link`'s fields, `InsertError::Store`, `MemStore::new`. New integration test file `src/url-shortener/tests/failed_fill.rs` (2 tests, using `tests/common/mod.rs` unchanged). No frozen test changed: `tests/{canonicalize,create,resolve}.rs` and `tests/common/mod.rs` keep their 22:46 to 22:48 timestamps. Kill check: in a copy under the scratchpad with the waiter's second round replaced by the R0 behavior (`answer(self.read_tiers(&code))`), both `failed_fill` tests fail at the "waiters of a failed fill read the store on their own" assertion. Ran from `src/`: `cargo fmt -p url-shortener -- --check` exit 0; `cargo clippy -p url-shortener --all-targets -- -D warnings` finished with no warnings; `cargo test -p url-shortener --no-fail-fast` all passed: 7 unit, 15 canonicalize, 14 create, 2 failed_fill, 16 resolve; `cargo test -q -p url-shortener --test resolve --test create --test failed_fill` 200 times in a loop, 0 failing runs. The lead's record is `impl-checks/R1.md`.

## Objection responses

- T1-1: accepted. `tests/resolve.rs`, `expired_entry_is_not_served_while_the_store_fails`: an entry T old with step 3 `TooStale` and step 4 `Timeout` is `503`; with F = 1 held by another code's read, step 3 `Timeout` is `503` and `consistent_rejected` is 1.
- T1-2: accepted. `tests/resolve.rs`, `local_miss_falls_through_to_a_consistent_read` now resolves again at T minus 1 ms and asserts one read per tier and `cache_hits == 1`.
- T1-3: accepted. `tests/common/mod.rs`, `TestStore::fail_next_find`; `tests/create.rs`, `failed_lookup_is_retried_once_and_never_read_as_absent`, including a find `Unavailable` against a key that exists. The retry budget is one per submission across find and insert (Mechanisms, 2; `LinkCreator`'s doc in `src/create.rs`), and the test pins it.
- T1-4: accepted. `src/code.rs`, `code_from_bytes` (re-exported in `src/lib.rs`), with `OsCodes`' doc pointing to it; `tests/create.rs`, `code_bytes_map_to_base62_with_rejection`; Not proven here's NFR8 line narrowed to the CSPRNG.
- T1-5: accepted. `tests/canonicalize.rs`: (a) golden row, literal key `e52ad9f0…e4f5` and an assertion in `non_ascii_host_goes_through_idna` for `faß` to `xn--fa-hia.example`, checked against idna 1.1.0 in a scratch crate and with `shasum`, plus `faß` vs `fass` as different keys; (b) golden row `A%2ab` to `a%2Ab` and `same_key` for `%2a` vs `%2A` in a host, with step 3 reworded in Mechanisms, 1 so that hex stays uppercase in the host; (c) golden rows for `:080`, `:0443` on https and on http, `:08080`, `:99999`, and `same_key` for `:080` and `:0443` against no port, `:08080` against `:8080` different. The port rule and its sources are in Mechanisms, 1.
- T1-6: accepted. Pinned as FR9's text reads, accepted, in `tests/canonicalize.rs`, `percent_encoded_host_is_judged_as_escaped`, with the `%c3%bc` vs `%C3%BC` host `same_key` here so a later change to this decision touches one test. Reported as a brief correction for the user, and logged under Follow-ups.
- T1-7: accepted by stating the order as a contract, with its reason: `LinkCreator`'s doc in `src/create.rs` and Mechanisms, 2 say a code is drawn only after Phase 1 finds the key absent, since drawing first lets a failing code source fail a repeat submission (FR3). `tests/create.rs`, `rng_failure_blocks_only_new_links`, pins that consequence, so `first_submission_creates_and_repeats_return_it` stays as written.
- T1-8: accepted. `src/create.rs`, a doc line on `CreateError::Rng`; `tests/create.rs`, `rng_failure_blocks_only_new_links`.
- T2-1: accepted by contract, the way T1-7 was: `CreateError::Rng`'s doc in `src/create.rs` says a failed draw is not retried, so `rng_failure_blocks_only_new_links`'s `draws() == 1` pins a stated promise. The frozen test is unchanged.
- R1-1: accepted, keeping the shared budget. `src/create.rs`: `MAX_DRAWS`' doc, `CreateError::CodesExhausted`'s doc (both causes: taken codes and a key race whose winner the rerun does not find), and `LinkCreator`'s doc bullet on Phases 2 and 3; test `create::tests::key_race_that_never_settles_exhausts_the_draws` (the probe-2 store: `find_by_key` always `None`, `insert` always `KeyTaken`) asserts `CodesExhausted` after exactly 6 draws. `impl.md`: Mechanisms, 2 and the Follow-ups line rewritten to match the code. A separate rerun bound was not chosen because the RFC names no number for it; the trade-off (one answer for contention and exhaustion) is logged under Follow-ups.
- R1-2: accepted. `src/resolve.rs`, `Resolver::resolve`, `step2` and `fill`: a waiter on an absent or failed outcome goes back through step 2 once and takes the next fill's outcome, which is valid for it because that fill was registered after the old one was removed (`Leading::drop` removes the fill before it wakes the waiters). `tests/failed_fill.rs`, `waiters_of_a_failed_fill_share_one_refill` (7 waiters, 2 local reads, not 8) and `waiters_take_the_refills_failure` (the refill fails too: 2 local and 2 consistent reads, all `503`); both fail against the R0 behavior (Revision log, R1). `negative_fill_is_not_shared` still passes: its joiner, back at step 2, finds no fill and leads its own. Mechanisms, 3 rewritten.
- R1-3: accepted. `src/store.rs`, the comment on `MemStore::tables` now says `insert` writes `links` before `url_index`, so a panic between them leaves an unindexed code, never an index entry without its row.
- R1-4: accepted. Doc comments on `MemStore::new`, `LinkCreator::new`, `Resolver::new`, `InsertError::Store` and `Link`'s two fields (`src/store.rs`, `src/create.rs`, `src/resolve.rs`). `LinkCreator` and `Resolver` get manual `Debug` impls with no bounds on their type parameters, which print no store, clock or code source: `LinkCreator` prints nothing but its name, `Resolver` its `config` and counters.
