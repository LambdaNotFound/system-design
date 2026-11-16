# RFC: URL Shortener

**Status:** Draft · **Author:** @LambdaNotFound

| Name | Sign-off | Comments |
| --- | --- | --- |
| @TBD | - [ ] approved | |

## Background

People share links in text messages, print, QR codes and character-limited posts, where a long URL gets truncated or mistyped. Xu's reference shortener stores the long URL under a short code, redirects whoever opens it, and returns the existing code for a URL already shortened (FR3).

## Terminologies

| Term | Meaning |
| --- | --- |
| **Code** | The 8-character path of a short URL, `https://s.example/<code>`; `s.example` stands for the service's domain. |
| **Submitted URL** | The long URL after FR10's conversion, stored and redirected to. |
| **Canonical form** | The submitted URL after FR4's normalization; its SHA-256 is the **URL key**. |

## Problem Statement

A returned link must survive a region's loss (NFR6), forcing a cross-region write before the reply, and resolve in every region at once (NFR11), while redirects answer within 100 ms at p99 (NFR3) and outlive creation failures (NFR4), forbidding a cross-region round trip per redirect. Equivalent URLs share one code under a race (FR3), 8-character codes stay unguessable at 365 billion links (NFR2, NFR8, NFR12), and a disable takes effect within 5 minutes (FR6).

The forcing question is how to make a link durable across regions before it's returned, yet resolvable locally and consistently everywhere.

## Requirements

### Functional Requirements

| ID | Requirement |
| --- | --- |
| FR1 | Any caller, with no account, can submit a long URL and receive a short URL for it, within the limits of FR7, FR8 and FR9. |
| FR2 | Opening a short URL that is not disabled (FR6), from a client IP that is not in an NFR8 cool-off, redirects the caller to the long URL exactly as it was submitted (FR10) in the request that created the short URL. |
| FR3 | Submitting a long URL that was already shortened returns the existing short URL to every caller, including when the first submissions of that URL arrive at the same moment, unless FR7, FR8 or FR9 rejects the submission. |
| FR4 | Two long URLs count as the same URL throughout this doc when they are equal after RFC 3986 syntax-based and scheme-based normalization (https://www.rfc-editor.org/rfc/rfc3986#section-6.2), and as different otherwise. |
| FR5 | A short URL never expires during the service's 10-year life and never changes its target, and no caller can edit or delete it. |
| FR6 | Only an operator can disable a short URL, and within 5 minutes of the disable being recorded the service stops redirecting it, and so does every cache that honors the service's cache headers. |
| FR7 | Only http and https URLs (the scheme in any case) that are valid under RFC 3986 and at most 2,048 characters long, both judged on the submitted URL as FR10 defines it, are accepted, and any other submission is rejected with the reason (assumed: 2,048 is a common practical limit, not a sourced one). |
| FR8 | Each client IP can submit only a limited number of URLs per hour, repeats of already-shortened URLs included, and submissions over that limit are rejected with the reason (assumed: the design sets the number). |
| FR9 | A submission is rejected with the reason when its host, after FR4 normalization, is the service's own domain or a subdomain of it, when it contains a user or password part (even an empty one), or when it is the same URL (FR4) as one whose short URL is disabled, from the moment the disable is recorded. |
| FR10 | A submitted URL containing non-ASCII characters is converted to a standard URL, with a punycode host and every other non-ASCII character percent-encoded as UTF-8, before any other rule applies, and the converted form counts as the submitted URL throughout this doc. |

### Non-functional Requirements

| ID | Requirement |
| --- | --- |
| NFR1 | **Scale.** The service absorbs 100 million new short URLs a day and an average of 10 redirects per link created, about 1,160 creations and 11,600 redirects a second (https://bytebytego.com/courses/system-design-interview/design-a-url-shortener), with peaks of 10 times those rates. |
| NFR2 | **Capacity.** Every short URL created over the service's 10-year life, about 365 billion, stays stored, and resolvable unless disabled (FR6). |
| NFR3 | **Redirect latency.** A redirect is answered within 100 ms at the 99th percentile, measured at the service (https://www.hellointerview.com/learn/system-design/problem-breakdowns/bitly). |
| NFR4 | **Redirect availability.** Redirects are available 99.99% of the time, with lookups rejected under NFR8 counted as available, and keep working while creating new short URLs is failing. |
| NFR5 | **Creation.** Creating a short URL completes within 1 second at the 99th percentile and is available 99.9% of the time. |
| NFR6 | **Durability.** A short URL that has been returned to a caller, and a disable that has been recorded, are never lost, even when a whole region fails. |
| NFR7 | **Correctness.** A short URL never redirects to a long URL that is not the same, per FR4, as the one it was created for. |
| NFR8 | **Security.** A short code cannot be predicted from any other short code, and a client IP that looks up too many codes that do not exist within a window has every lookup rejected for a cool-off period (assumed: the design sets the window, the limit and the cool-off). |
| NFR9 | **Operability.** Operators can see redirect and creation latency and error rates against NFR3, NFR4 and NFR5, are alerted when a target is breached, and can see how long each disable took to take effect across the service. |
| NFR10 | **Compatibility.** A short URL, once issued, keeps the same domain and code for the service's whole 10-year life, and resolves at them unless disabled (FR6). |
| NFR11 | **Read-after-create.** From the moment a short URL is returned to the caller, every server that is serving redirects resolves it with no replication delay, unless it is disabled (FR6). |
| NFR12 | **Length.** Every short code is at most 8 characters long. |

### Out of Scope

- Accounts for callers, so FR3's reuse is global; operators still authenticate (Open Question 3).
- Custom aliases: every code is random (NFR8).
- Expiry, and editing or deleting by callers: FR5 forbids them.
- Click analytics: redirects record NFR9's metrics only.
- Scanning URLs for spam or malware: operators are the only abuse control.
- Legal erasure: a disable stops redirects but keeps the row.

## Estimates

- **Rates.** NFR1: 100,000,000 / 86,400 s = 1,157 creations and 11,574 redirects a second, 11,600 and 116,000 at peak. A time-ordered key would put every insert on one range, so keys are random.
- **Code space.** 62^8 = 2.18 × 10^14 codes, 0.167% full at NFR2's 365 × 10^9 links: one live code in 598, so inserts rarely retry. Base36 would be 12.9% full (Appendix), a live link every 8 probes (NFR8), so base62 is forced.
- **Storage (assumed).** At Xu's assumed 100 B URL, rows of 124 B and 40 B total 299 TB with 5 replicas, 3.85 PB at FR7's maximum (Appendix), so the store is range-sharded and the index keys on a 32 B hash.
- **Disable budget.** FR6 allows 300 s. Replica staleness S + cache TTL T = 30 + 60 = 90 s for servers, plus browser `max-age` M = 60 s = 150 s, so expiry alone meets FR6.
- **Headroom and step 4 load (assumed).** A drain moves a region's third onto two, raising each by half, so each region is sized for 116,000 / 2 = 58,000 redirects a second; at an 80% hit rate, 11,600 reach the store, against 1,280 that one server's step 4 confirms (Appendix). Until Task 3 sizes that fleet, a slow replica drains the region.

## Proposed Solution

### Overview

This RFC defines a three-region service that writes each link in one cross-region transaction before returning it, and answers each redirect from a server cache or the local replica, reading consistently only when that replica lacks the code. The store is CockroachDB, set to survive a region failure; codes are 8 random base62 characters. NFR3, NFR4 and NFR5 rest on figures assumed until load-tested; a wide botnet can fail fresh codes, and a cool-off cuts off a whole shared address or prefix for an hour (Failure Modes).

*Diagram: three regions, each with an L4 load balancer, a stateless fleet (Redirect Resolver with cache, Create Handler, Admin API), a counter store and CockroachDB nodes of one 2+2+1 cluster. Arrows: DNS to a healthy region; redirect to cache, local replica, then dashed to leaseholders; create committed across two regions; the Disable Tracker probing servers. Label S, T and M.*

### API

- `POST /v1/links` `{"url"}` returns `201 {"code", "short_url"}`, or `200` with the existing code (FR3), so a retry is safe though charged to FR8 again. Rejections: `400` with `bad_scheme`, `invalid_url`, `too_long`, `own_domain`, `userinfo` or `disabled`; `429 rate_limited` with `Retry-After`; `503`.
- `GET /{code}` returns `302` with `Location` and `Cache-Control: private, max-age=60` (FR2). Disabled is `410`, unknown `404`, both `no-store`; cool-off `429` (NFR8); unconfirmable or shed-limited `503`. Other paths are unknown codes; query strings are ignored.
- `POST /admin/v1/links/{code}/disable` `{"reason"}`, on the internal admin host only, returns `disabled_at`, the first one on a repeat.
- `GET /admin/v1/disables/{code}` returns `disabled_at` and `server_effective_at` (NFR9).

`/v1/links` can't collide with a code, which has no second `/`. Nothing paginates.

### Data model and storage

Access patterns: a point read by code per redirect (116,000 a second at peak, mostly cached); per creation (11,600), a point read by URL key, then a read or two inserts atomic across ranges (FR3); rare disables, tracker scans and cool-off writes, read by each server every second.

```sql
CREATE TABLE links (
  code        STRING(8) PRIMARY KEY,  -- random base62 (NFR8, NFR12)
  url         STRING NOT NULL,        -- submitted URL, never updated (FR5)
  created_at  TIMESTAMPTZ NOT NULL,
  disabled_at TIMESTAMPTZ NULL        -- set once by an operator (FR6)
);
CREATE TABLE url_index (
  url_key BYTES PRIMARY KEY,          -- SHA-256 of canonicalize_v1 output
  code    STRING(8) NOT NULL
);
CREATE TABLE disables (
  code STRING(8) PRIMARY KEY, disabled_at TIMESTAMPTZ NOT NULL,
  operator STRING NOT NULL, reason STRING NOT NULL,
  server_effective_at TIMESTAMPTZ NULL  -- set by the Disable Tracker (NFR9)
);
CREATE TABLE cooloffs (                 -- LOCALITY GLOBAL (Abuse Limiter, NFR8)
  identity BYTES PRIMARY KEY,           -- IPv4 address, or IPv6 prefix and length
  until TIMESTAMPTZ NOT NULL, INDEX (until)
);
```

**MVP choice: CockroachDB in three regions, `SURVIVE REGION FAILURE`, `REGIONAL BY TABLE`,** because:

- NFR6: 5 replicas placed 2+2+1, so 3-replica commits outlive any one region.
- FR3: a transaction makes the two inserts atomic.
- NFR3, NFR4: bounded-staleness follower reads serve a row from the nearest replica, even partitioned.
- NFR11: a consistent read at the leaseholder settles a local miss.

**Pros:** the fatal invariants (NFR6, NFR7, FR3) are the database's job. **Cons:** a self-operated cluster, a cross-region round trip on every write, and 5× storage.

### Edge and client identity

DNS latency records send clients to the nearest healthy region, with a 60 s TTL and health checks every 10 s, 3 failures to trip, so NFR4 holds 78 drains a year (Appendix). The L4 load balancer passes the peer address by PROXY protocol as FR8 and NFR8's client IP, ignoring `X-Forwarded-For`.

No single IPv6 prefix is one caller: RIPE-690 gives a home a /56 and a business a /48, and a phone gets a /64 per connection (RFC 6459). So an IPv4 address is one **identity**, and an IPv6 address three, its /64, /56 and /48, limited at 1, 4 and 16 times the base. A /48 holder gets 16 times an IPv4 caller's quota, not 65,536. The cost is IPv6's carrier-grade NAT: one prober's cool-off on a carrier's /56 or /48 rejects up to 256 or 65,536 phones' /64s for an hour, everywhere (Failure Modes).

### Canonicalizer

```rust
pub struct SubmittedUrl(String); // ASCII, FR7-valid; stored and sent in Location
pub struct UrlKey([u8; 32]);     // SHA-256 of the canonical form
pub enum Reject { BadScheme, InvalidUrl, TooLong, OwnDomain, Userinfo }

/// Frozen: url_index keys depend on its output.
pub fn canonicalize_v1(raw: &str) -> Result<(SubmittedUrl, UrlKey), Reject>;
```

1. **Convert (FR10).** Only a host with non-ASCII goes through UTS #46 ToASCII, nontransitional, which maps case and applies NFC; other non-ASCII characters are percent-encoded as UTF-8. ASCII is untouched, so an ASCII host keeps its case and a space stays invalid.
2. **Validate (FR7).** Scheme `http` or `https` in any case, RFC 3986 grammar, at most 2,048 characters.
3. **Normalize (FR4).** Lowercase scheme and host, uppercase percent-encoding hex, decode unreserved escapes, remove dot-segments, drop the default port, and make an empty path `/`.
4. **Reject (FR9), then hash.** Any `@` in the authority rejects. The normalized host, minus one trailing dot for this check only, must not be `s.example` or end in `.s.example`; the canonical form is then hashed.

**Versioning.** Any change to step 3, a bug fix included, re-keys old URLs and gives them second codes (FR3), so v1 is frozen behind golden tests; a change is a v2 with a backfill re-keying `url_index`. **Cons:** the parser and IDNA tables become part of the data format for ten years.

### Link Creation

The Create Handler charges FR8 first, so rejections count, then canonicalizes and runs one transaction:

```rust
pub struct Code([u8; 8]); // base62 ASCII from a CSPRNG
pub enum Created { New(Code), Existing(Code) }
pub enum CreateError { Disabled, Integrity, CodeTaken, KeyTaken, Store(StoreError) }

pub trait LinkStore {
    fn create(&self, key: &UrlKey, url: &SubmittedUrl, fresh: Code) -> Result<Created, CreateError>;
}
```

```text
Phase 1, look up: read url_index[key]. Found code c: read links[c].
                  disabled_at set: Disabled (FR9).
                  canonicalize_v1(links.url) yields another key: Integrity,
                  alert, never return c (NFR7). Otherwise Existing(c) (FR3).
Phase 2, insert:  insert links[fresh]. Duplicate code (p <= 0.00167): CodeTaken;
                  the handler draws a new code and retries, at most 5 times.
Phase 3, claim:   insert url_index[key] = fresh. Duplicate key or a serialization
                  conflict: a concurrent first submission won; KeyTaken, and the
                  handler reruns Phase 1 (FR3).
Phase 4, commit:  3 of 5 replicas across two regions (NFR6), then New(fresh).
```

An aborted transaction leaves no rows, and the stored spelling is the winner's, FR2's creating request.

```mermaid
sequenceDiagram
    participant C as Client
    participant H as Create Handler
    participant L as Abuse Limiter
    participant DB as CockroachDB
    C->>H: POST /v1/links {url}
    H->>L: charge client IP (FR8)
    H->>H: canonicalize_v1
    H->>DB: BEGIN; read url_index[key]
    alt key found
        DB-->>H: code c, links[c]
        H-->>C: 200 {c}, or 400 disabled
    else key absent
        H->>DB: insert links[fresh], url_index[key]; COMMIT
        DB-->>H: committed on 3 of 5 replicas
        H-->>C: 201 {fresh}
    end
```

**Pros:** FR3, NFR6 and NFR7 hold by construction, and codes reveal nothing about others (NFR8). **Cons:** each creation pays cross-region round trips; NFR5 is assumed until Task 2's load test.

### Redirect Resolver

```rust
pub enum Cached { Live(Arc<str>), Disabled } // positive entries only (NFR11)

pub trait LinkReader {
    /// Follower read, nearest replica only; errors past max_staleness.
    fn read_local(&self, code: &Code, max_staleness: Duration) -> Result<Option<Link>, StoreError>;
    /// Consistent read at the leaseholder.
    fn read_consistent(&self, code: &Code) -> Result<Option<Link>, StoreError>;
}
```

```text
Step 1, guard:   not a code: 404, a miss. An identity in cool-off or past its
                 regional miss limit: 429 (NFR8); past its shed limit: 503.
Step 2, cache:   entry younger than T = 60 s: 302 or 410.
Step 3, local:   read_local(code, S = 30 s), 20 ms timeout. Found: fill, answer.
Step 4, confirm: absent, error or timeout: read_consistent(code), 100 ms
                 timeout, at most F = 64 in flight per server, of which IPs
                 with a miss in their window share 16. Found: fill, answer.
                 Absent: 404, a miss. Error, timeout or over the limit: 503,
                 a shed if step 3 found the code absent.
```

**Why each tier.** Single-flight fill limits a viral code to one store read per server per T. Step 3 keeps the common path in-region (NFR3) and off the write quorum (NFR4). Step 4 exists for NFR11: a code returned a moment ago may not be on this replica yet, so a local miss is never final or cached. It crosses regions, so NFR3 assumes steps 2 and 3 answer 99% of redirects.

**Staleness (FR6).** An entry filled at t reflects the store at t − S or later and serves until t + T, and a browser keeps the `302` M more (Estimates). `private` keeps shared caches out, and `302`, unlike `301`, isn't cached heuristically.

**Draining.** Each server's health endpoint fails when a step 3 read of a sentinel code fails or passes 20 ms, or over 5% of step 3 reads in 10 s time out, so the region drains rather than serve a disabled or unseen code (FR6, NFR11). A drain adds half to each survivor's load, within its headroom (Estimates); if survivors trip anyway, Route 53 treats all records as healthy once none is, the undrained state.

```mermaid
sequenceDiagram
    participant C as Client
    participant R as Redirect Resolver
    participant LR as Local replica
    participant LH as Leaseholder
    C->>R: GET /{code}
    R->>R: cool-off and miss-count check, cache lookup
    alt cache miss
        R->>LR: follower read, max staleness 30 s
        alt absent or too stale
            R->>LH: consistent read
            LH-->>R: link or none
        else found
            LR-->>R: link
        end
    end
    R-->>C: 302, 410, 404, 429 or 503
```

**Pros:** staleness can only hide a code or a disable, never misdirect, because a URL never changes (FR5). **Cons:** a fresh code's first lookups cross regions, and enumeration reaches the leaseholders, up to F per server.

### Abuse Limiter

A regional in-memory counter store charges each identity per submission, `(identity, hour)`, rejected past 100 (FR8), and per miss, `(identity, 10 minutes)`, each limit times the identity's multiple. Step 1 reads the IP's counts in one round trip and rejects once any passes its limit, overshooting by the lookups then in flight (Appendix); the server whose charge crosses it writes a 1-hour cool-off to `cooloffs`, a `GLOBAL` table every region reads consistently and locally. Servers read new rows every second, overlapping 5 s for commit order, so a cool-off holds everywhere within seconds (NFR8).

**Sheds.** A shed of a locally absent code leaks what a `404` does, so it counts, apart from misses since the code may be fresh; past the limit, step 1 answers `503` regionally until the window ends, which NFR4 counts, with no cool-off (Appendix).

Counting stays regional: a global counter puts a cross-region write on every miss (NFR3, NFR4); cool-offs start at most once per identity an hour, suiting `GLOBAL`'s slow writes; a client in all three regions gets three times each limit (Appendix). If the counter store is down or past a 5 ms timeout, step 1 passes and servers count in process and alert; a failed cool-off write holds locally and retries, and a failed read keeps the last set. **Cons:** a counter read on every lookup, up to 5 ms of NFR3; shared addresses and prefixes share limits, a risk the scoping record accepted; a botnet multiplies them.

### Disables

One transaction sets `links.disabled_at` and inserts the `disables` row (FR6), committed like a creation (NFR6); the next creation for that URL reads it in Phase 1 and is rejected (FR9).

**Disable Tracker (NFR9).** Every 5 s, for each disable lacking `server_effective_at`, a singleton job calls every serving server's debug lookup and records the time when all answer `410`; the browser tail M is computed, not observed. A disable not in effect within 90 s pages an operator.

### Example

At 09:00:00 a caller in region B submits `HTTPS://Example.com:443/a/../b`, stored as is with canonical form `https://example.com/b`; code `q7Xk2PzA` commits in regions A and B. At 09:00:01 region C's replica lacks it, so step 4 finds it and redirects to the stored spelling. Disabled at 10:00:00, it stops redirecting everywhere by 10:01:30; a resubmission at 10:00:05 fails (FR9).

## Alternatives Considered

### Code generation

- **A counter in base62 (Xu).** Pro: never collides. Con: a code predicts its neighbors (NFR8), and every insert lands on the newest range, a hotspot at 11,600 a second.
- **A counter through a keyed permutation of 62^8.** Pro: no collisions, random-looking codes. Con: in place of one retry in 598, a durable counter that survives region loss and a ten-year secret whose leak inverts every code under that key (NFR8).
- **A truncated hash of the canonical form.** Pro: reuse without an index. Con: at 47.6 bits, n² / 2N = 3.05 × 10^8 URL pairs collide, so a collision path is needed anyway, and anyone can compute a URL's code (NFR8).

### Store

- **One primary region, asynchronous replicas.** Pro: local writes, simplest operations. Con: a region loss drops unreplicated writes, so a returned link can vanish (NFR6).
- **DynamoDB global tables, multi-Region strong consistency.** Pro: managed, RPO zero across three regions, and conditional puts give FR3 without transactions: code item first, then URL key, so a crash or lost race leaves only an unreturned code for an FR4-equal URL (NFR7), which the winner's disable misses. Con: strongly consistent reads cross regions, a round trip per uncached redirect (NFR3); eventually consistent reads have no staleness bound or lag metric, so a lagging replica can serve a disabled code past FR6's 5 minutes.
- **CockroachDB `GLOBAL` tables.** Pro: consistent local reads everywhere, so step 4 disappears. Con: writes pay a commit-wait, which the docs aim at rarely updated data, not 11,600 inserts a second (NFR5).

### Read path

- **A CDN caching redirects.** Pro: absorbs viral links and cuts latency. Con: edge hits skip the service, so an IP in cool-off still gets redirects (NFR8).

## Failure Modes and Observability

Five failure modes matter; metrics follow.

- **A region fails.** Each range keeps 3 of 5 replicas, so reads and writes continue (NFR6); DNS drains the region onto the survivors' headroom in about 90 s, and its leases move.
- **A region is partitioned.** Past S it reads neither locally nor consistently, so uncached lookups get `503` until it drains, about 120 s (Appendix).
- **Two regions fail.** Beyond NFR6: creation and disables stop, and redirects fail once caches expire and local reads pass S. The break-glass is a logged operator flag lifting S, suspending FR6 and NFR11.
- **The store is slow.** A local replica past step 3's 20 ms drains its region (Draining). A slow leaseholder fails fresh and unknown codes with `503`, and a client shed past the limit loses that region until its window ends (Abuse Limiter). Creation retries a timed-out transaction once, safe under FR3.
- **A botnet enumerates.** Clean IPs share step 4's other 48 slots with fresh codes. Past 960 first-time misses a second on a server, about 576,000 distinct IPv4 addresses or IPv6 /48s per 10 minutes (Appendix), fresh codes there get `503` and count as sheds: a residual risk to NFR4 and NFR11, shown by step 4 rejections.

Metrics that track:

- Redirect and creation latency by region and status, and availability (non-`5xx` over all), with burn-rate alerts at 99.99% and 99.9% (NFR3, NFR4, NFR5, NFR9).
- Follower staleness, step 3 timeouts, step 4 rate and rejections over F, cache hit rate.
- Misses, sheds, shed-limit `503`s, active cool-offs and counter-store fallback per region; lookups rejected per cool-off, paging when a /56 or /48 cool-off rejects over 16 /64s without misses, which NFR4's count hides.
- `server_effective_at − disabled_at` per disable (NFR9); the tracker's heartbeat age, paging past 1 minute; `Integrity` aborts, which should be zero.

Debug endpoints that:

- Resolve one code through one server's steps 2 to 4, cache untouched: `/internal/lookup/{code}`.

## Open Questions

1. **Which three regions?** Creation and step 4 latency scale with their round trips. Current design: assumed one continent. If wider, measure creation p99 first.
2. **Are the limits and load figures right?** Current design: assumed 100 creations an hour, 50 misses or sheds in 10 minutes, multiples of 4 and 16 at /56 and /48, a 1-hour cool-off, F = 64 with 16 for IPs with misses, an 80% cache hit rate, 50 ms per step 4 read, regions sized for half the peak. If they block real users, raise them; the Appendix prices each.
3. **How do operators authenticate?** Current design: assumed existing single sign-on fronts the admin host. If none exists, the Admin API needs its own.
4. **Does one cluster sustain 11,600 two-range transactions a second plus step 4?** Current design: assumed yes, scaled by nodes. If not, revisit the two-insert transaction first.

## Work Breakdown

| # | Task | Description | Depends on |
| --- | --- | --- | --- |
| 1 | Canonicalizer | `canonicalize_v1` and its frozen golden tests (FR4, FR7, FR9, FR10). | none |
| 2 | Link creation | Schema, the four-phase transaction, codes, a peak load test (FR1, FR3, FR5, NFR1, NFR2, NFR5, NFR6, NFR7, NFR8, NFR12). | Task 1, **CockroachDB cluster** (external, three regions) |
| 3 | Redirect Resolver | Tiers, single-flight cache, bounded fallback, headers (FR2, FR6, NFR3, NFR4, NFR11). | Task 2 |
| 4 | Abuse Limiter | Identities, counters, shed limit, `cooloffs` and its pull, step 4 slot split, fallback (FR8, NFR8). | Task 3 |
| 5 | Disables | Admin API, disable transaction, internal lookup, tracker (FR6, FR9, NFR9). | Task 3 |
| 6 | Deployment | DNS health checks, the health endpoint, PROXY protocol, the fixed domain (NFR4, NFR10). | Task 3 |
| 7 | Metrics and alerts | The metrics and alerts above (NFR9). | Tasks 3, 4, 5 |

## Appendix

**Alphabet against density** at 365 × 10^9 links of 8 characters:

```text
alphabet  space           fraction live  one live code per
base32    1.10 × 10^12    33.2%          3 probes
base36    2.82 × 10^12    12.9%          8 probes
base62    2.18 × 10^14    0.167%         598 probes
```

**Minor rivals.** No server cache: removes T from FR6's budget, but a viral code's reads concentrate on one range's replicas. Push invalidation: takedown in seconds, but a pub/sub fleet that falls back to expiry, which already meets FR6.

**Availability per region drain.** NFR4's 99.99%: 0.0001 × 31,536,000 s = 3,154 s of failed lookups a year, counted per lookup as NFR4's NFR8 clause implies. A partitioned region: up to S = 30 s before its reads fail, then 10 s × 3 health checks to detect, then a 60 s DNS TTL to drain = 120 s; a hard region failure skips S, 90 s. At a third of the traffic, 120 / 3 = 40 s an event, 3,154 / 40 = 78 a year (assumed: regions split traffic evenly and clients honor the TTL).

**Enumeration under the limits.** Step 1 rejects an identity once its regional count passes the limit, so a region answers 50 of its misses per cycle plus the lookups in flight when the 50th is charged, at the end of step 4. That overshoot is min(r × ℓ, 16n + r × 20 ms) on the n servers a prober reaches, since a dirty identity holds at most 16 step 4 slots a server and the rest wait in step 3. At an assumed r = 1,000 probes a second and ℓ = 20 ms (step 3) + 50 ms (step 4) = 70 ms, it is 70 once n ≥ 4, rising with r and the fleet. A fast prober's cycle is the 1-hour cool-off: 3 × (50 + 70) = 360 probes an hour, 8,640 a day, 14.4 live codes a day at 1 in 598 for an IPv4 address or a lone /64. A /48 holder at 16 times: 3 × (800 + 70) × 24 = 62,640 probes, 104.7 live codes a day. Through a step 4 outage in one region, the shed limit binds there instead, with the 10-minute window as the cycle and ℓ up to 20 + 100 ms (step 4's timeout): 50 + 120 = 170 probes a window, 1,020 an hour, 1.7 live codes an hour for the outage's length.

**Step 4 capacity (assumed).** At an assumed 50 ms a consistent read, F = 64 slots confirm 64 / 0.05 = 1,280 a second per server.

**Step 4 under a wide botnet.** An IP whose identities have no miss in a region's window takes a clean slot, and one miss from a /48 dirties every address in it: 48 clean slots / 0.05 s = 960 a second per server, × 600 s = 576,000 distinct IPv4 addresses or IPv6 /48s per server per 10 minutes.

**Storage rows.** `links`: 8 B code + 100 B URL + 16 B timestamps = 124 B; `url_index`: 32 B + 8 B = 40 B. Worst case: (2,072 + 40) B × 365 × 10^9 × 5 = 3.85 PB. Hash saving: (100 − 32) B × 365 × 10^9 × 5 = 124 TB.

## References

- *System Design Interview*, vol. 1, Alex Xu, "Design a URL Shortener" ([course page](https://bytebytego.com/courses/system-design-interview/design-a-url-shortener)), NFR1's rates and the counter rival.
- [Hello Interview: Design a URL Shortener like Bit.ly](https://www.hellointerview.com/learn/system-design/problem-breakdowns/bitly), NFR3 and NFR4's targets.
- [CockroachDB: Multi-Region Survival Goals](https://docs.cockroachlabs.com/docs/stable/multiregion-survival-goals), the 2+2+1 placement behind NFR6.
- [CockroachDB: Follower Reads](https://docs.cockroachlabs.com/docs/stable/follower-reads), the bounded-staleness reads of step 3.
- [CockroachDB: Global Tables](https://docs.cockroachlabs.com/docs/stable/global-tables), the locality of `cooloffs`, rejected for `links`.
- [Amazon DynamoDB: How global tables work](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/V2globaltables_HowItWorks.html), the managed rival.
- [Amazon Route 53: How Route 53 chooses records when health checking is configured](https://docs.aws.amazon.com/Route53/latest/DeveloperGuide/health-checks-how-route-53-chooses-records.html), DNS behavior when regions are unhealthy.
- [Amazon Route 53: How Route 53 determines whether a health check is healthy](https://docs.aws.amazon.com/Route53/latest/DeveloperGuide/dns-failover-determining-health-of-endpoints.html), the 10 s interval and failure threshold behind the drain time.
- [RFC 9111: HTTP Caching](https://www.rfc-editor.org/rfc/rfc9111), `max-age`, `private` and `no-store`.
- [RFC 3986: URI Generic Syntax](https://www.rfc-editor.org/rfc/rfc3986), FR4's normalization and the userinfo grammar.
- [UTS #46: Unicode IDNA Compatibility Processing](https://www.unicode.org/reports/tr46/), FR10's host conversion.
- [RIPE-690: IPv6 prefix assignment for end-users](https://www.ripe.net/publications/docs/ripe-690/), the /56 and /48 behind the IPv6 identities.
- [RFC 6459: IPv6 in 3GPP Evolved Packet System](https://www.rfc-editor.org/rfc/rfc6459), a phone's /64 per connection.
- *Designing Data-Intensive Applications*, Martin Kleppmann, chapter 5 (reading your own writes) and chapter 9 (linearizability), the theory behind step 4.

<!-- loop state below: not part of the RFC -->

## Evidence

| Claim | Source | Label |
| --- | --- | --- |
| The repo has no prior design, concept note or crate for a URL shortener | `design-doc/url-shortener/scoping.md:85`; `concepts/`, `src/` listings this session (only README.md in each) | observed |
| Xu looks the long URL up first and returns the existing code; assumes 100-byte URLs and a base62 counter | `design-doc/url-shortener/scoping.md:91`, `:93` (Xu, accessed 11/15/26 by scoping) | observed |
| Hello Interview treats reuse as optional and recommends 302 | `design-doc/url-shortener/scoping.md:86`, `:90` | observed |
| 1,157 creations/s, 11,574 redirects/s average; 10× at peak | NFR1: 100,000,000 / 86,400; python run this session | inferred |
| 62^8 = 218,340,105,584,896; 0.167% full; 1 in 598; base36 12.9%; base32 33.2% | NFR2, NFR12; python run this session | inferred |
| 59.9 TB logical, 299 TB with 5 replicas; 3.85 PB worst case; 124 TB saved by hashing | NFR2, FR7, Xu's 100 B URL (assumed), row sizes (assumed); python run this session | assumed |
| SHA-256 collision probability across 365 × 10^9 keys is about 5.8 × 10^-55 | n² / 2^257; python run this session | inferred |
| Truncated 47.6-bit hash gives about 3.05 × 10^8 colliding pairs | n² / 2N with N = 62^8 | inferred |
| SURVIVE REGION FAILURE needs 3 regions, places 5 replicas 2+2+1, stays available for reads and writes through a region outage, write latency rises by at least the RTT to the nearest region | CockroachDB, "Multi-Region Survival Goals", https://docs.cockroachlabs.com/docs/stable/multiregion-survival-goals, read 11/15/26 | observed |
| Every commit has a replica outside any one region | 3-of-5 quorum with at most 2 replicas per region (above) | inferred |
| Bounded-staleness follower reads serve from local replicas under partitions, become increasingly stale, `nearest_only` errors rather than leave the region, and require a single-row primary or covering index read in an implicit transaction | CockroachDB, "Follower Reads", https://docs.cockroachlabs.com/docs/stable/follower-reads, read 11/15/26 | observed |
| GLOBAL tables read a consistent local replica, writes incur commit-wait, suited to read-mostly rarely updated data; lead override guidance of 800 ms and up when RTT exceeds 150 ms | CockroachDB, "Global Tables", https://docs.cockroachlabs.com/docs/stable/global-tables, read 11/15/26 | observed |
| GLOBAL table writes risk NFR5 at 11,600/s | from the commit-wait note above; no latency number published | assumed |
| DynamoDB MRSC: exactly three regions, RPO zero, conditional writes on the latest version, strongly consistent reads on any replica, transactions not supported | AWS, "How DynamoDB global tables work", https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/V2globaltables_HowItWorks.html, read 11/15/26 and 11/16/26 | observed |
| MRSC writes and strongly consistent reads require cross-Region communication; MRSC publishes no `ReplicationLatency` metric; eventually consistent reads are permitted (FIS note) and the page states no staleness bound for them | same page, read 11/16/26 | observed |
| A crash or lost race in the code-first, then URL-key conditional-put protocol leaves only an unreturned code whose URL is FR4-equal to the winner's | from the two conditional puts and FR4 keying (R1-5) | inferred |
| A lagging MRSC replica can serve a disabled code past FR6's 5 minutes under eventually consistent reads | follows from the absence of any documented staleness bound; not tested | inferred |
| A response with `max-age` may be stored whatever its status; `private` forbids shared caches; `no-store` forbids all caches | RFC 9111 §3, §5.2.2.1, §5.2.2.5, §5.2.2.7, https://www.rfc-editor.org/rfc/rfc9111, read 11/15/26 (fetch summary with quotes) | observed |
| 301 and 308 are heuristically cacheable; 302 is not | RFC 9110 §15.1, §15.4 via `design-doc/url-shortener/scoping.md:95` | observed |
| RFC 3986 normalization steps (case, percent-encoding, dot-segments, default port, empty path) | `design-doc/url-shortener/scoping.md:96` (RFC 3986 §6, accessed 11/15/26 by scoping) | observed |
| Userinfo grammar, `user:password` deprecated; a DNS FQDN may end in one trailing dot | RFC 3986 §3.2.1, §3.2.2, https://www.rfc-editor.org/rfc/rfc3986, read 11/15/26 | observed |
| UTS #46 maps case, applies NFC, and recommends nontransitional processing | UTS #46 v18.0.0, https://www.unicode.org/reports/tr46/, read 11/15/26 | observed |
| FR6 worst case 150 s with S = 30, T = 60, M = 60 | FR6; the three bounds are design choices; arithmetic in Estimates | inferred |
| Normal follower-read lag is a few seconds, well under S = 30 s | Follower Reads page suggests reads at least 4.2 s in the past; per-range lag not measured | assumed; measure follower staleness per server after Task 3 |
| Steps 2 and 3 serve at least 99% of redirects within 100 ms | none yet; measure with the latency histogram and cache hit rate under load after Task 3 | assumed |
| Creation transaction completes well under 1 s at p99 across three same-continent regions | none yet; load test in Task 2 | assumed |
| One cluster sustains 11,600 two-range transactions a second | none yet; load test in Task 2 | assumed |
| Step 1's read of the regional counts caps a region's answered misses, and separately its sheds, per identity at the limit plus the lookups in flight when it is crossed; a shed of a locally absent code counts apart from misses, so shedding can't hide probes, and past the limit it draws a regional `503` until the window ends, not a cool-off | from step 1, step 4 and Sheds as specified; each count charged when its lookup ends | inferred |
| The overshoot is min(r × ℓ, 16n + r × 20 ms), since a dirty identity holds at most 16 step 4 slots on each of n servers: 70 at r = 1,000 probes a second, ℓ = 20 ms (step 3 timeout) + 50 ms (step 4 read) and n ≥ 4, up to 120 for sheds at step 4's 100 ms timeout; it grows with r and n | step 3 and step 4 as specified; r, the 50 ms read and n are assumed; measure step 1 to charge latency after Task 3 | assumed |
| Enumeration yield of 14.4 live codes a day for an IPv4 address or lone /64, 104.7 for a /48 holder, and 1.7 an hour in a region with no working step 4 | Appendix arithmetic: 3 × (50 + 70) × 24 / 598, 3 × (800 + 70) × 24 / 598, and 6 × (50 + 120) / 598, a fast prober's cycle being the 1-hour cool-off, or the 10-minute window under the shed limit; python run this session; assumed limits and NFR2 density | assumed |
| RIPE-690 recommends a /48 per business and a /56 per residential customer, strongly discourages assigning longer than /56, says a /64 or longer does not conform to IPv6 standards, and names a single /64 per PDP context for cellular phones | RIPE NCC, "RIPE-690: Best Current Operational Practice for Operators: IPv6 prefix assignment for end-users", https://www.ripe.net/publications/docs/ripe-690/, §1, §4.2.2 to §4.2.4, read 11/16/26 | observed |
| A 3GPP network gives each UE's default bearer a unique /64 prefix | RFC 6459, "IPv6 in 3GPP EPS", §3.2, §4.2, §5.2, https://www.rfc-editor.org/rfc/rfc6459, read 11/16/26 | observed |
| Counting IPv6 at /64, /56 and /48 with 1, 4 and 16 times the base gives one /48 holder 16 times an IPv4 caller's quota, against 65,536 times at /64 alone (2^(64 − 48)) | the two rows above; multiples are design choices | inferred |
| A cool-off on a /56 or /48 rejects up to 2^(64 − 56) = 256 or 2^(64 − 48) = 65,536 /64s, each a phone's whole prefix under RFC 6459, for the 1-hour cool-off in every region, and NFR4 counts those rejections as available | RFC 6459 row above, the identity rule and NFR4's NFR8 clause | inferred |
| Carriers place many phones' /64s inside one /56 or /48, so they share the wider prefix's quota and cool-off | RFC 6459 (row above) gives a /64 per UE; how carriers lay those out within a /56 or /48 was not read this session; measure from traffic per prefix after Task 4 | assumed |
| DNS latency records with health checks route to a healthy region, and treat all records as healthy when none is, the floor of a drain cascade | AWS, "How Amazon Route 53 chooses records when health checking is configured", https://docs.aws.amazon.com/Route53/latest/DeveloperGuide/health-checks-how-route-53-chooses-records.html, read 11/15/26 | observed |
| Limits and timeouts: 100 creations/h, 50 misses per 10 min, multiples of 4 and 16 at /56 and /48, 1 h cool-off, F = 64 with 16 slots for IPs with misses, step 3 timeout 20 ms, step 4 timeout 100 ms, a shed limit equal to the miss limit, a page past 16 /64s without misses rejected under one /56 or /48 cool-off, counter store timeout 5 ms, health endpoint failing on a sentinel read past 20 ms or at 5% step 3 timeouts in 10 s, 5 s cool-off read overlap, tracker heartbeat page at 1 min | design choices under FR8 and NFR8's (assumed) tags; none yet, tune from Task 3 and 4 load tests | assumed |
| Two concurrent transactions inserting the same primary key end with one commit and the other a duplicate-key or retryable serialization error | memory of CockroachDB's transaction model, not read this session; settle with a concurrency test in Task 2 | assumed |
| Codes are case-sensitive base62 in the URL path, so a mis-cased code is a different code | NFR12 and the code-space bullet; consequence for print and QR readers is accepted | inferred |
| Route 53 health checks run every 10 s or 30 s; the failure threshold is the number of consecutive checks that flips status; an endpoint is unhealthy when 18% or fewer checkers report healthy | AWS, "How Amazon Route 53 determines whether a health check is healthy", https://docs.aws.amazon.com/Route53/latest/DeveloperGuide/dns-failover-determining-health-of-endpoints.html, and "Values that you specify when you create or update health checks", https://docs.aws.amazon.com/Route53/latest/DeveloperGuide/health-checks-creating-values.html, read 11/16/26 | observed |
| Drain time 90 s for a failed region, 120 s for a partitioned one; 40 s of NFR4 budget an event; 78 events a year | Appendix arithmetic from NFR4, S, the 10 s interval (observed option), a threshold of 3 and a 60 s TTL (design choices); even traffic split and clients honoring the TTL | assumed; measure drain time in a region-isolation drill after Task 6 |
| NFR4 is counted per lookup, not per wall-clock second | NFR4's clause counting NFR8-rejected lookups as available | inferred |
| Each region sized for half the peak, 116,000 / 2 = 58,000 redirects a second, 1.5 times its 38,667 share, so one drain leaves survivors within capacity; at an 80% hit rate 11,600 a second reach the store | NFR1; the hit rate and the sizing are assumed; load-test a region at 58,000 a second in Task 3 | assumed |
| The counter store serves a read on every lookup, up to 58,000 a second per region, within a 5 ms timeout | none yet; measure in Task 4's load test | assumed |
| A consistent read takes about 50 ms, so F = 64 confirms 1,280 a second per server, and 48 clean slots 960 | none yet; measure step 4 latency per region pair after Task 3 | assumed |
| A wide botnet needs about 576,000 distinct IPv4 addresses or IPv6 /48s per server per 10 minutes to saturate the clean step 4 pool | 960 × 600, one clean probe per identity per region window, a /48's first miss dirtying every address in it; inputs assumed above | assumed |
| `GLOBAL` table writes are slow but rare enough for cool-off starts, at most one per identity an hour | Global Tables row above; the 1-hour cool-off | inferred |
| A cool-off reaches every region within seconds of its write | GLOBAL tables read a consistent local replica (row above) plus a 1 s poll; commit-wait duration not measured | assumed; measure cool-off propagation after Task 4 |
| UTS #46 ToASCII runs only on hosts containing non-ASCII, matching FR10's trigger, so an ASCII host keeps its submitted case | FR10, FR2 | inferred |

## Revision log

- R0: first draft.
- R1: cool-off made service-wide through a `GLOBAL` `cooloffs` table; step 4 slots split; health endpoint drains on slow replicas; availability and step 4 arithmetic added; DynamoDB and keyed-permutation rivals restated; ToASCII scoped to non-ASCII hosts; repetition cut; tracker heartbeat metric.
- R2: step 1 rejects on the regional miss count, and step 4 sheds of locally absent codes count as misses; IPv6 counted at /64, /56 and /48 with multiples 1, 4, 16 (RIPE-690, RFC 6459); regions sized for half the peak, sentinel read timed, Route 53 fail-open named as the cascade floor; Appendix yield and botnet figures redone; Estimates availability bullet cut to a pointer; push invalidation moved to Appendix minor rivals; trims to fit 4,000 words.
- R3: overshoot input ℓ taken from steps 3 and 4 (70 ms) and capped at min(r × ℓ, 16n + r × 20 ms), yields redone (14.4 and 104.7 a day); step 4 given a 100 ms timeout; sheds counted apart from misses, with a regional `503` until the window ends instead of a cool-off; prefix cool-off blast radius stated with a paging metric; step 1 wording fixed to "its limit"; Estimates availability pointer moved to Edge, canonicalizer-drift bullet cut as a repeat of Versioning, trims across Background, Problem Statement, Overview, Link Creation, Abuse Limiter and Failure Modes to fit 4,000 words.

## Objection responses

### Round 1

- R1-1 | accepted. Counting stays regional, but a regional count past 50 writes the cool-off to `cooloffs`, a `GLOBAL` table each server reads locally every second, so an IP cooled off in one region is rejected in all three within seconds (Abuse Limiter, Data model). Kill reason restated: a global counter costs a write per miss, while cool-off starts are rare enough for `GLOBAL`'s commit-wait. The in-process fallback still writes cool-offs; a failed write holds on that server and retries, a failed read keeps the last set. Appendix yield updated to 150 probes per 70 minutes, 5.2 live codes a day per IP. No brief correction needed.
- R1-2 | accepted. Step 4 reserves 16 of F's 64 slots for IPs whose regional miss count is nonzero, read from the counter store before step 4, so a repeating prober can't crowd out fresh codes. What remains is a botnet whose IPs probe once per window: past 960 first-time misses a second per server (about 576,000 distinct IPs per server per 10 minutes at an assumed 50 ms read), fresh codes there get `503`, stated as a residual risk against NFR4 and NFR11 (Failure Modes, Appendix).
- R1-3 | accepted. Estimates sizes the uncached share at an assumed 80% hit rate: 7,733 lookups a second per region at peak against 1,280 a second per server in step 4. Because absorbing that depends on an unfixed fleet size, the health endpoint also fails when over 5% of a server's step 3 reads in 10 s time out, so a slow local replica drains its region like a partition (Redirect Resolver, Failure Modes).
- R1-4 | accepted. Health checks every 10 s with a threshold of 3 and a 60 s TTL give a 90 s drain for a failed region and 120 s for a partitioned one (S first). Counted per lookup, as NFR4's NFR8 clause implies, an event costs 40 s of the 3,154 s yearly budget, so it holds 78 a year, assumed until a region-isolation drill measures it (Estimates, Appendix, Evidence).
- R1-5 | accepted. The rival now carries its steel-manned FR3 protocol (code item first, then URL key, both conditional; an orphan is unreturned and FR4-equal, though the winner's disable misses it). The deciding con is the read path, from the AWS page read 11/16/26: MRSC strongly consistent reads require cross-Region communication (NFR3), and eventually consistent reads have no stated staleness bound and MRSC publishes no replication-lag metric, so FR6's bound can't be enforced on a local read.
- R1-6 | accepted. The con now names the rival's real costs, a durable region-safe counter and a ten-year secret whose leak lets anyone invert codes (NFR8), set against one retry in 598.
- R1-7 | accepted. Canonicalizer step 1 runs ToASCII only on a host containing non-ASCII, matching FR10's trigger; an ASCII host keeps its submitted case, and step 3 lowercases it for the key only, so the Example and FR2 agree.
- R1-8 | accepted. Staleness now points to the Estimates arithmetic, the Overview's restatements of Disables, Partitions and Latency are cut to one sentence of caveats, the Latency paragraph is folded into Why each tier, and drain and step 4 arithmetic live in the Appendix. Word count 3,998 of 4,000 after the additions.
- R1-9 | accepted. Metrics now include the tracker's heartbeat age, paging past 1 minute, so a stopped tracker can't silently drop NFR9's timings.

### Round 2

- R2-1 | accepted. Step 1 now reads the IP's regional miss counts in the same round trip step 4 already used, and answers `429` once any identity passes its limit, so a region stops answering an identity at its 50th miss plus the lookups in flight at that moment, whatever its probe rate or the `GLOBAL` propagation gap; `cooloffs` then only carries the hour-long cool-off to the other regions. A step 4 error, timeout or shed after step 3 found the code absent now counts as a miss, so saturating the 16 shared slots no longer hides probes (Redirect Resolver step 1 and step 4, Abuse Limiter). Appendix restated with its assumed probe rate: overshoot r × ℓ = 10 at 1,000 probes a second and 10 ms, a 60-minute cycle for a fast prober, 180 probes an hour and 7.2 live codes a day per IPv4 address or lone /64. Costs stated: a counter read on every lookup, up to 5 ms of NFR3, failing open; and a legitimate client shed during a botnet is charged a miss (Failure Modes).
- R2-2 | accepted. The /64 premise was wrong: RIPE-690 (read 11/16/26) recommends a /56 per home and a /48 per business and says a /64 or longer does not conform, while RFC 6459 (read 11/16/26) gives a phone a /64 per connection, so no single prefix length is one caller. An IPv6 address now counts as three identities, its /64, /56 and /48, limited at 1, 4 and 16 times the base for both FR8 and the miss count, and a limit or cool-off on any of them applies (Edge and client identity, Abuse Limiter, `cooloffs.identity`). A /48 holder gets 16 times an IPv4 caller's quota instead of 65,536 times; the shared-quota cost is that phones a carrier packs into one /56 share 4 times one caller's quota, stated as IPv6's carrier-grade NAT and left assumed in Evidence. Appendix: a /48 holder's yield is 97.5 live codes a day; the wide-botnet threshold is now 576,000 distinct IPv4 addresses or IPv6 /48s, since a /48's first miss dirties every address in it. No brief correction: FR8 and NFR8 say "client IP", and scoping left IPv6 handling to the design.
- R2-3 | accepted. Estimates now sizes each region for half the peak, 58,000 redirects a second, 1.5 times its share, so one drain leaves the survivors within capacity (assumed until Task 3's load test). The sentinel read now fails past the same 20 ms, so a slow replica stays drained with no traffic, while a load-induced trip clears when its load leaves. The cascade floor is named: if survivors trip anyway, Route 53 treats every record as healthy once none is (Evidence, observed 11/15/26), which is the undrained state (Redirect Resolver, Draining; Failure Modes).
- R2-4 | accepted. The Estimates availability bullet now carries only the result and a pointer, with the arithmetic in the Appendix alone. Further cuts to fit R2-1 to R2-3: push invalidation moved to the Appendix's minor rivals, the partition sentence folded into its Failure Modes bullet, and tightening across Estimates, Overview and Alternatives. Word count 4,000 of 4,000.

### Round 3

- R3-1 | accepted. ℓ now comes from the path a miss takes before it is charged: step 3's 20 ms timeout plus an assumed 50 ms step 4 read, 70 ms; step 4 now has a stated 100 ms timeout, so a shed's ℓ is at most 120 ms. The overshoot is min(r × ℓ, 16n + r × 20 ms), slightly tighter than the suggested min(r × ℓ, 16n), since lookups still in step 3 aren't held by step 4's slots: 70 at r = 1,000 once n ≥ 4, and it grows with r and the fleet. Yields redone: 3 × (50 + 70) × 24 / 598 = 14.4 live codes a day for an IPv4 address or lone /64, 3 × (800 + 70) × 24 / 598 = 104.7 for a /48 holder (Appendix, Evidence). "Whatever the probe rate" is gone from the Abuse Limiter.
- R3-2 | accepted, narrowed rather than only stated. A shed of a locally absent code still counts, since a `503` leaks "absent locally" as a `404` does, but it counts apart from misses: past the limit in sheds, step 1 answers `503` in that region until the 10-minute window ends, and no cool-off starts (Abuse Limiter, Sheds; step 1 sketch). A legitimate client opening fresh codes through a slow leaseholder, a pre-S partition or a botnet loses at most one window of that region's redirects, counted against NFR4 so burn-rate alerts see it, rather than an hour everywhere that NFR4 hides; the slow-store and botnet bullets now say so, and "costs fresh and unknown codes only" is gone. The probe bound holds: R2-1's prober that fills the 16 shared slots is stopped at the shed limit, and in a region with no working step 4 its cycle shrinks to the window, 6 × (50 + 120) / 598 = 1.7 live codes an hour for the outage's length (Appendix). The bar-raiser's other narrowing, charging sheds only after a confirmed miss, was rejected: a prober that never gets a `404` (every step 4 read shed during an outage) would then probe step 3 uncounted for the outage's length, which reopens R2-1. Shed-limit `503`s are a metric.
- R3-3 | accepted, as a stated cost with a trigger, not a narrowing. Edge and client identity now states the blast radius: one prober's cool-off on a carrier's /56 or /48 rejects up to 256 or 65,536 phones' /64s for an hour, in every region; the Overview caveat carries it. Metrics count lookups rejected per cool-off and page when a /56 or /48 cool-off rejects over 16 /64s without misses of their own, since NFR4's count hides them. Narrowing was weighed and rejected: if a wider-prefix cool-off spared /64s with no misses, a /48 holder rotating /64s would get each fresh /64's own limit, up to 65,536 × 50 misses an hour against the 800 the /48 limit allows, which gives up the rotation bound R2-2 added. The page threshold of 16 is assumed (Evidence limits row).
- R3-4 | accepted. The Abuse Limiter now reads "rejects once any passes its limit".
