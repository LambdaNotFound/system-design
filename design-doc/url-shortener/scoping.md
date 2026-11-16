# Scoping record: URL shortener

Mode: interview
Origin: scope-problem 11/15/26
Frame: 11/15/26
Grounded: 11/15/26
Exit check: 11/15/26, not offered
Covered: 11/15/26
Decomposed: 11/15/26
Reader test: 11/15/26
Swept: 11/15/26
Humanized: 11/15/26
Final check: 11/15/26, capped after two runs

## Questions asked

| # | Date | Question | Answer |
| --- | --- | --- | --- |
| F1 | 11/15/26 | What are we trying to change or build? | A short URL service like bit.ly: a user sends in a website's URL and the service returns a short URL; other users who open the short URL are redirected to the original website. |
| F2 | 11/15/26 | Who is it for, and who else is affected? | Anyone can use it; user authentication doesn't need to be designed first. |
| F3 | 11/15/26 | What does done look like: what would you check? | For /scope-problem, a requirement doc. |
| F4 | 11/15/26 | Anything that must or must not be used? | The same URL must map to the same short URL. |
| F5 | 11/15/26 | Any page, book chapter, file, or repo to read first? | References online may be read. |
| 1.1 | 11/15/26 | Should "same URL, same short URL" hold for everyone, one shared link per URL? Default: yes. | Global, one link (not "global now, per-user later"). |
| 1.2 | 11/15/26 | Custom aliases? Default: out of scope. | Out of scope. |
| 1.3 | 11/15/26 | Can links expire, or be edited or deleted by their creator? Default: permanent and immutable. | Permanent and immutable. |
| 1.4 | 11/15/26 | Click analytics? Default: out of scope. | Out of scope. |
| 1.5 | 11/15/26 | Must an operator be able to disable a link after creation? Default: yes, within a bound. | Yes, within a bound (no blocklist check at creation). |
| 1.6 | 11/15/26 | What counts as the same URL? Default: RFC 3986 syntax- and scheme-based normalization only. | RFC 3986 normalization. |
| 1.7 | 11/15/26 | Must two simultaneous submissions of a new URL get the same short URL? Default: yes. | Yes, always. |
| 1.8 | 11/15/26 | Which scale? Default: Xu's traffic and lifetime with Hello Interview's latency and availability. | Xu traffic plus Hello Interview targets. |
| 1.9 | 11/15/26 | Why does it exist? Default: a public bit.ly-style product for places where long URLs fail. | Public bit.ly-style product. |
| 1.10 | 11/15/26 | How soon must a disabled link stop redirecting, caches included? Default: within 5 minutes. | Within 5 minutes. |
| 1.11 | 11/15/26 | Is losing a link that was already handed out fatal? Default: yes. | Fatal. |
| 1.12 | 11/15/26 | During a partial outage, which must stay up? Default: redirects first. | Redirects first; creation may fail. |
| 2.1 | 11/15/26 | What peak over Xu's daily averages? Default: 10 times the averages. | 10 times the averages. |
| 2.2 | 11/15/26 | At which percentile is the 100 ms redirect target? Default: p99. | p99. |
| 2.3 | 11/15/26 | Creation latency and availability? Default: under 1 s at p99, 99.9%. | Under 1 s at p99, 99.9%. |
| 2.4 | 11/15/26 | Which long URLs can be submitted? Default: http and https, at most 2,048 characters. | http and https, at most 2,048 characters. |
| 2.5 | 11/15/26 | Must short codes be hard to guess? Default: yes, not enumerable. | Yes, not enumerable. |
| 2.6 | 11/15/26 | Rate-limit creation per client? Default: yes, per client IP, the number set in the design. | Yes, per client IP. |
| 2.7 | 11/15/26 | What must operators see and be alerted on? Default: SLOs with alerts, plus takedown time. | SLOs with alerts, plus takedown time. |
| 2.8 | 11/15/26 | Must the short URL format stay valid for the 10-year life? Default: fixed for 10 years. | Fixed for 10 years. |
| 2.9 | 11/15/26 | Conflict C1: which bar for guessability, given short codes at 365 billion links? Default: non-sequential codes plus rate-limited failed lookups. | Non-sequential codes plus rate-limited failed lookups; code length set in the design. |
| 2.10 | 11/15/26 | What would you check to call the system done, beyond this doc? Default: the RFC addresses every FR and NFR by ID and passes the bar-raiser. | The RFC addresses every FR and NFR by ID and passes the bar-raiser. |
| 2.11 | 11/15/26 | Any imposed constraint: deadline, language, cloud, database, tool? Default: none. | None. |
| 2.12 | 11/15/26 | Anything else before the write-up? | Nothing else. |
| S.1 | 11/15/26 | C2: what does a caller get when submitting a URL whose short link is disabled? Default: rejected with a reason. | Rejected with a reason. |
| S.2 | 11/15/26 | C3: does a repeat submission count toward the per-IP limit? Default: yes, and over the limit it is rejected. | Counts, rejected over the limit. |
| S.3 | 11/15/26 | G4: must a new short URL work everywhere as soon as it is returned? Default: yes. | Yes, immediately. |
| S.4 | 11/15/26 | G3: which caches does FR6's 5 minutes cover? Default: the service and every cache that honors its cache headers. | The service and caches that honor its headers. |
| S.5 | 11/15/26 | G1: which spelling does a shared short URL redirect to? Default: the URL as first submitted. | As first submitted. |
| S.6 | 11/15/26 | G2: only operators can disable links? Default: yes. | Only operators. |
| S.7 | 11/15/26 | G6: cap on short code length? Default: at most 8 characters. | At most 8 characters. |
| S.8 | 11/15/26 | G5: reject URLs that point at the service's own domain? Default: yes. | Reject own-domain URLs. |
| S.9 | 11/15/26 | G7: legal erasure of stored URLs? Default: out of scope. | Out of scope. |
| S.10 | 11/15/26 | G8: how soon does FR9 reject a URL whose link was disabled? Default: immediately. | Immediately. |
| S.11 | 11/15/26 | G9: does FR8's over-limit rejection give the reason? Default: yes. | Yes, with a reason. |
| S.12 | 11/15/26 | G10: reject URLs with a user or password part? Default: yes. | Reject them. |
| S.13 | 11/15/26 | G11: what happens to URLs with non-ASCII characters? Default: accept them, converted to a standard URL first. | Accept, converted first. |
| S.14 | 11/15/26 | C4: limiting only misses doesn't stop enumeration; which bar? Default: after too many misses, throttle every lookup from that IP for a cool-off, outside NFR4's count. | Throttle all lookups. |
| S.15 | 11/15/26 | G12: must a recorded disable be as durable as a link? Default: yes. | Yes, never lost. |

## Coverage

| Row | State | Where it landed |
| --- | --- | --- |
| Why now | assumed: no deadline or outside trigger | Problem Statement |
| Who it serves | settled: anonymous link creators, anyone who opens a link, operators who disable links | Problem Statement, FR1, FR2, FR6 |
| Done means | settled: the RFC addresses every FR and NFR by ID and passes the bar-raiser; each row is worded as its own check | every FR and NFR |
| Scale | settled | NFR1, NFR2, NFR3, NFR5 |
| Failure behavior | settled: redirects stay up while creation fails | NFR4, NFR5 |
| Correctness bar | settled: a wrong redirect and a lost link are both fatal, and a new link works at once | FR2, FR3, NFR6, NFR7, NFR11 |
| Existing state | collapsed: a new system, nothing live to migrate, backfill or roll back | |
| Blast radius | settled: every issued short URL depends on the service for 10 years | FR5, NFR10 |
| Operations | settled | NFR9 |
| Imposed constraints | settled: none | |
| The boundary | settled | Out of scope line, FR7, FR8, FR9, FR10, NFR12 |
| The null option | settled | Problem Statement |

## Evidence

| Claim | Source | Label |
| --- | --- | --- |
| This repo has no prior design doc, concept note or crate for a URL shortener; `url-shortener` appears only as an example slug. | `concepts/`, `design-doc/`, `src/` listings; grep of `*.md`, `*.rs`, `*.toml` | observed |
| Hello Interview treats deduplication as optional: "Optionally, we can check if this exact long URL was already shortened and return the existing short code." | "Design a URL Shortener like Bit.ly", https://www.hellointerview.com/learn/system-design/problem-breakdowns/bitly, accessed 11/15/26 | observed |
| Hello Interview: "most URL shorteners allow multiple short codes for the same long URL", so links can have separate expiration dates, independent analytics or different custom aliases. | Hello Interview, same page | observed |
| Hello Interview's requirements: redirect under 100 ms, 99.99% availability with availability over consistency, 1B shortened URLs, 100M DAU, 1000 reads per write, 100k new URLs a day. | Hello Interview, same page | observed |
| Hello Interview puts user accounts, click analytics, and spam and malicious-URL filtering out of scope. | Hello Interview, same page | observed |
| Hello Interview recommends a 302 redirect, to keep the ability to update or expire links and to count clicks; browsers typically cache a 301. | Hello Interview, same page | observed |
| Xu's base-62 design looks the long URL up first and returns the existing short URL when it finds one. | Xu, System Design Interview vol. 1, "Design a URL Shortener", https://bytebytego.com/courses/system-design-interview/design-a-url-shortener, accessed 11/15/26 | observed |
| Xu assumes "shortened URLs cannot be deleted or updated." | Xu, same page | observed |
| Xu's estimates: 100M new URLs a day, an assumed 10:1 read:write ratio, 10 years of service, 365 billion records, 100-byte URLs, 36.5 TB, 7 base-62 characters. | Xu, same page | observed |
| Xu: a 301 reduces server load because browsers cache it; a 302 is the better choice if analytics matter. | Xu, same page | observed |
| 301 and 308 responses are heuristically cacheable by default, other 3xx responses are not, and 307 and 308 forbid changing the request method. | RFC 9110 §15.1 and §15.4, https://www.rfc-editor.org/rfc/rfc9110#section-15.4, accessed 11/15/26 (read through a fetch summary, not verbatim) | observed |
| RFC 3986 equivalence: syntax-based normalization (scheme and host case, percent-encoding case, unreserved escapes, dot-segments) and scheme-based normalization (default port, empty path as "/") are the standard steps; protocol-based normalization risks false positives. | RFC 3986 §6 (fetch summary) and §2.1–2.3 (verbatim), https://www.rfc-editor.org/rfc/rfc3986#section-6, accessed 11/15/26 | observed |
| Bitly's v4 spec has an undocumented `force_new_link` boolean and does not say whether shortening an already-shortened URL returns the existing Bitlink, or whether reuse is per group. | Bitly v4 OpenAPI spec, https://dev.bitly.com/v4/v4.json, accessed 11/15/26 | observed |
| Bitly reuses an existing Bitlink by default: a search snippet quotes its reference as "Use force_new_link only when a new Bitlink should be created for a long URL that may already have an existing Bitlink." | Web search snippet of https://jentic.com/apis/bitly.com/bitly, not confirmed on dev.bitly.com | inferred |
| With no accounts, FR3's reuse is global, because nothing tells two submitters apart. | F2 and F4 | inferred |
| A redirect that browsers cached from a 301 cannot be withdrawn by the service afterwards. | Follows from RFC 9110's cacheability rule above | inferred |
| At 365 billion links, a random base-62 code is live 1 in 10 at 7 characters, 1 in 598 at 8, 1 in 37,088 at 9 and 1 in 2.3 million at 10; probing at the 116,000-a-second peak redirect rate finds about 12,000, 194, 3 and 0.05 live links a second. | Computed from NFR1 and NFR2 (python, 11/15/26) | inferred |
| Enumerating short URLs has exposed private cloud documents in real shorteners. | Georgiev and Shmatikov, "Gone in Six Characters: Short URLs Considered Harmful for Cloud Services" (2016), recalled, not read this session | assumed |
| 2,048 characters is a common practical URL length limit. | Recalled, not read this session | assumed |

## Open questions

| Question | Owner | Default |
| --- | --- | --- |

## Decomposition

| Piece slug | Serves (parent IDs, mapped to piece IDs) | Route | Depends on | Handoff |
| --- | --- | --- | --- | --- |
| url-shortener | all: FR1–FR10, NFR1–NFR12 | design | none | `/new-design url-shortener` |

## Reader test

**11/15/26, phase 5.** A fresh reader got the doc's path and ten builder questions, then the follow-up.

- **Got right:**
  - separate creation and redirect paths, with redirects independent of creation;
  - FR3 enforced by a unique index on the normalized URL, including the simultaneous-submission race;
  - the FR4 examples: host case and an empty path are normalized, while http vs https, `www.` and query order stay different;
  - a disabled code is never reused;
  - caches bounded by FR6's 5 minutes, which rules out an uncached 301;
  - analytics out of scope;
  - the code-density risk behind NFR8.
- **Wording fixed (meaning unchanged):**
  - FR1: subject to FR7 and FR8.
  - FR2: excepts disabled links.
  - FR4: defines "same URL" for the whole doc, not just FR3. This was the reader's hard contradiction: FR3 against NFR7.
  - FR7: the 2,048 characters are counted as submitted.
  - NFR1: "an average of 10 redirects per link".
  - NFR2: "resolvable unless disabled".
  - NFR6: covers losing a whole region, from answer 1.11, whose rejected option was exactly that loss.
  - NFR7: "not the same, per FR4".
  - NFR8: the design sets the lookup limit.
  - NFR10: reworded to the option text chosen in 2.8.
  - Out of scope: names accounts for people who create or open links, so operators aren't excluded.
- **Sent to the phase 6 sweep:**
  - resubmitting a disabled URL (FR3 against FR6);
  - an over-limit caller resubmitting a known URL (FR3 against FR8);
  - whether new links must redirect at once;
  - which caches FR6 and NFR9 can cover;
  - the spelling a redirect goes to;
  - restricting disabling to operators;
  - a number for "short";
  - URLs that point back at the service;
  - legal erasure.
- **Left to the design, not requirements:**
  - how the client IP is found, behind proxies and in IPv6 blocks, and the rate-limit window;
  - the redirect status code, and what disabled and unknown codes return;
  - the API's shape;
  - alphabet and case sensitivity;
  - the measurement windows for the availability targets;
  - hot links;
  - the domain, DNS and TLS;
  - the cost budget and average URL size;
  - single region or multi-region.
- **Re-test:** folded into the phase 6 final check, because the sweep changes the same rows.

**11/15/26, phase 5 again**, after the sweep added FR9, NFR11 and NFR12. A fresh reader got nine questions aimed at the changed rows, then the follow-up.

- **Got right:** all nine answers, matching decisions C2, C3 and G1 to G7: a disabled URL is rejected on resubmission; repeats count toward FR8; a shared link redirects to the creating request's spelling; only operators disable, within 5 minutes for the service and caches that honor its headers; a new link works at once; codes are at most 8 characters, which fills 0.17% of the base-62 space at 365 billion links; no one erases or changes a stored URL.
- **Wording fixed (meaning unchanged):**
  - FR2: "the request that created the short URL", rather than "first submitted", which was ambiguous under simultaneous submissions.
  - FR3: lists FR7 among its exceptions. FR1 already did; a submission over 2,048 characters could otherwise normalize to a known URL and be owed a link.
  - NFR10: "resolves at them unless disabled (FR6)", matching NFR2.
  - NFR11: "every server that is serving redirects", so a failed region no longer breaks the row; "no replication delay" carries the intent; plus the FR6 exception.
- **Sent to the sweep:**
  - how soon FR9's rejection of a disabled URL applies;
  - whether FR8's rejection carries a reason;
  - URLs with userinfo (`user@host`), which can disguise the real host.
- **Left to the design:**
  - the alphabet, which needs at least 28 symbols for 8 characters at 365 billion links;
  - normalizer versioning;
  - what counts as "own domain" in FR9;
  - the redirect status code;
  - measurement windows;
  - how client IPs are found;
  - hot links and growth;
  - storage cost, about 748 TB of raw URLs at the 2,048-character worst case.

**11/15/26, phase 5, third run**, after G8 to G10 changed FR8 and FR9. A fresh reader got seven questions, then the follow-up.

- **Got right:** all seven answers, with no contradiction between any two requirements: what each rejection covers and that it carries a reason; a disabled URL is rejected at once while its redirects stop within 5 minutes; `http://paypal.com@evil.example` and own-domain URLs are rejected; a shared link redirects to the creating request's spelling; what holds from the moment a link is returned; the scale numbers; redirects keep running while creation fails.
- **Wording fixed (meaning unchanged):**
  - Out of scope: "scanning or classifying" for spam or malware, so FR9's lookup of disabled URLs doesn't read as that excluded check.
  - FR5: "during the service's 10-year life", matching NFR2 and NFR10.
  - FR6: the 5 minutes start when the disable is recorded, matching FR9.
  - FR9: the own-domain check runs on the normalized host and covers subdomains, which closes the case-change bypass that defeats G5; an empty user part counts.
  - NFR8: "only" lookups of codes that do not exist are limited, so a throttled IP still gets working redirects, as decision C1 intended.
- **Sent to the sweep:** non-ASCII URLs. FR2's "exactly as submitted" can't go into a `Location` header unchanged when the input isn't a strict RFC 3986 URI.
- **Left to the design:**
  - the normalizer's library and its edge cases within RFC 3986 §6.2;
  - how a disable is recorded across regions;
  - the redirect status code and what a disabled link returns;
  - the quota window and how client IPs are found;
  - whether a consistent fallback read on a miss fits NFR3;
  - CDN edges under NFR11;
  - availability measurement windows;
  - regions;
  - the submission rate including repeats;
  - storage;
  - the API's shape.
- **Residual risks for sign-off:** a disable blocks only one URL and its FR4 equivalents, so `?x=2` gets around a disable of `?x=1`; per-IP limits assume one IP is roughly one caller; operators are the only abuse control.
- **Correction:** the NFR8 "only" change above was not a wording fix. It defeated NFR8's purpose and went to the user as C4.

**11/15/26, final check (phase 6, step 4)**, a fresh reader with the doc alone, standing in for a fourth phase 5 run after G11 added FR10. The skill's rule (an accepted gap reruns phase 5) would otherwise loop; the final check is the same context-free read.

- **Got right:** the whole build, matching every decision.
- **Wording fixed:** FR7 judges validity and length on the converted URL (FR10), as the G11 option chosen said, and the scheme is matched in any case.
- **Sent back to the user:** C4 (NFR8 limited only misses, which can't stop enumeration) and G12 (durability of disables). Both decisions changed requirements, so the sweep, the pass and the final check run again.
- **Second final check, 11/15/26,** after C4 and G12, with a fresh reader and the doc alone.
  - **Got right:** the whole build.
  - **Contradiction:** FR2 promised every non-disabled link redirects, while C4 rejects every lookup from a throttled IP. This was resolved by the user's own C4 decision, so FR2 now carries the NFR8 exception rather than a new choice.
  - **Wording fixed:**
    - FR2 points to FR10's definition of "as submitted".
    - FR10 percent-encodes only non-ASCII characters, so spaces stay invalid under FR7.
    - NFR4 counts NFR8 rejections as available, which keeps the target time-based.
    - The Problem Statement reads "a long URL" rather than "any", since FR7 and FR9 restrict input.
  - **Left to the design:** whether disabling shares the creation write path (if it does, disables fail while creation fails); IDNA variant and Unicode normalization before conversion; and the items below.
- **Cap, 11/15/26.** The second final check returned a contradiction (C5), so no third reader was dispatched. The rows changed after it (the Problem Statement, FR2, FR10 and NFR4) were shown to the user to read for themselves in the sign-off walk.
- **Left to the design:** URL parser and IDNA version, a trailing-dot host under FR9, the redirect status code, what a disabled code returns, the code alphabet, codes colliding with service routes, how NFR11 is met during a partition (redirect servers that can't confirm freshness leave service), negative caching under NFR11, measurement windows, how client IPs are found, interfaces, and requests to a short URL with a query string or a trailing slash.

## Conflicts

| # | Between | Clash | Options | Decision | Date |
| --- | --- | --- | --- | --- | --- |
| C1 | Problem Statement (short links) and answer 2.5 (codes not enumerable), at NFR2's 365 billion links | Short codes fill the code space densely: a random 7-character code is live 1 in 10, so short codes alone cannot stop probing. | Non-sequential codes plus rate-limited failed lookups; non-sequential codes only; secret-strength codes of about 22 characters | Non-sequential codes plus rate-limited failed lookups, code length set in the design (NFR8); later capped at 8 characters by G6 | 11/15/26 |
| C2 | FR3 and FR6 | Resubmitting a URL whose short link is disabled: FR3 returns the existing link, FR6 says it no longer redirects. | Reject with a reason; return the dead link; issue a new code | Reject with a reason (FR9, FR3) | 11/15/26 |
| C3 | FR3 and FR8 | A caller over the creation limit resubmits a known URL: FR3 returns the link, FR8 rejects the request. | Every submission counts and is rejected over the limit; repeats are exempt | Every submission counts (FR8, FR3) | 11/15/26 |
| G1 | FR2 and FR4 | Equivalent spellings share one short URL, but no row said which spelling it redirects to. | As first submitted; the normalized form | As first submitted (FR2) | 11/15/26 |
| G2 | FR6 | No row said only operators can disable a link. | Only operators; leave it to the design | Only operators (FR6) | 11/15/26 |
| G3 | FR6 and NFR9 | "Everywhere, caches included" can't be met or measured for caches that ignore the service's headers. | The service plus caches that honor its headers; the service only; keep "everywhere" | The service plus caches that honor its headers (FR6, NFR9) | 11/15/26 |
| G4 | FR2 | No row said a new short URL must work at once in every region. | Immediately; up to 5 seconds of lag | Immediately (NFR11) | 11/15/26 |
| G5 | FR7 | Nothing stopped short URLs pointing at the service itself, which chains or loops redirects. | Reject own-domain URLs; allow them | Reject own-domain URLs (FR9) | 11/15/26 |
| G6 | Problem Statement | "Short" had no number. | At most 8 characters; at most 10; no cap | At most 8 characters (NFR12) | 11/15/26 |
| G7 | FR5 and NFR2 | Nothing covered a legal demand to erase a stored URL. | Out of scope; operators can erase | Out of scope (Problem Statement) | 11/15/26 |
| G8 | FR6 and FR9 | No row said how soon a disabled URL's resubmission is rejected. | Immediately; within 5 minutes | Immediately, from the moment the disable is recorded (FR9) | 11/15/26 |
| G9 | FR8, against FR7 and FR9 | The over-limit rejection gave no reason, unlike the other rejections. | With a reason; no requirement | With a reason (FR8) | 11/15/26 |
| G10 | FR7 and the Out of scope line | A user part (`http://trusted@evil.example`) disguises a link's real host and passed FR7. | Reject them; allow them | Reject them (FR9) | 11/15/26 |
| G11 | FR2 and FR7 | Non-ASCII input isn't a strict RFC 3986 URL and can't go into a redirect header exactly as typed. | Accept and convert first; reject | Accept, converted to a standard URL first (FR10) | 11/15/26 |
| C4 | NFR8 and NFR4 | Limiting only an IP's misses can't stop enumeration, because the limited answer itself marks a miss; limiting every lookup hurts users sharing an IP. The "only" came from a phase 5 fix that was wrongly logged as wording. | Throttle every lookup after too many misses; delay responses; no throttle | Throttle every lookup from that IP for a cool-off, not counted against NFR4 (NFR8, NFR4) | 11/15/26 |
| G12 | NFR6 and FR6 | A lost disable silently re-enables a link, and NFR6 covered only links. | Never lost; best effort | Never lost (NFR6) | 11/15/26 |
| C5 | FR2 and NFR8 | Found by the second final check: FR2 promised that every link that isn't disabled redirects, while C4's cool-off rejects every lookup from a throttled IP. | FR2 excepts the cool-off, as C4 decided; no other option is consistent with C4 | Carried from C4 rather than asked separately: FR2 excepts an IP in an NFR8 cool-off (FR2); shown in the sign-off walk | 11/15/26 |

## Humanizer

**11/15/26.** No humanizer skill in this session, so the scoping lead made the pass over all four sections. No section changed for style: every row was already one terse sentence, with zero dashes and no filler words. Lines kept original: all.
