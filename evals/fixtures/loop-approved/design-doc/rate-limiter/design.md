# RFC: Rate Limiter for the Public API

**Status:** Draft · **Author:** @author-example

| Name | Sign-off | Comments |
| --- | --- | --- |
| @TBD | - [ ] approved | |

## Background

The public API serves every client from one pool of API nodes and one shared database pool. Nothing limits how many requests a single client sends, so a client stuck in a retry loop can take the database pool from everyone else. Operators learn of it from latency alerts, after other clients are already slow.

## Problem Statement

One misbehaving client can cause an outage for all clients, and each new integration adds another client able to do it. The forcing question is how every API node can agree on one client's request count closely enough to enforce a limit, without making each request wait on a shared store.

## Requirements

### Functional Requirements

| ID | Requirement |
| --- | --- |
| FR1 | The limiter rejects a request with HTTP 429 when the client has exceeded its configured requests per minute. |
| FR2 | A rejected response carries a `Retry-After` header with the seconds until the client may retry. |
| FR3 | An operator can change one client's limit without a deploy, and the change takes effect within 60 seconds. |

### Non-functional Requirements

| ID | Requirement |
| --- | --- |
| NFR1 | **Scale.** The limiter handles 20,000 requests per second across 5,000 clients (assumed). |
| NFR2 | **Latency.** The limiter adds at most 2 ms at p99 to a request, including when its store is slow or down. |
| NFR3 | **Failure behavior.** When the limiter's store does not answer within 1 ms, the request is allowed through and an alert fires. |
| NFR4 | **Correctness bar.** While the store is reachable, a client exceeds its limit by at most 5% in any minute. |

### Out of Scope

- Billing by usage. The limiter counts requests to protect the API, not to price them.
- Limits on internal service-to-service calls. Only the public API is in scope.

## Estimates

Both numbers start from NFR1, whose inputs are assumed, so both are assumed.

- **Store calls.** 20,000 requests/s means 20,000 Redis calls/s, one per request. A single Redis call per request is the budget; a design needing two would double it.
- **Counter memory.** 5,000 clients × 2 window counters × 100 bytes a key (assumed) = 1 MB. Memory does not drive the design.

## Proposed Solution

### Overview

This RFC adds a rate-limit check to the API's request path: one Redis call per request that increments the client's counter for the current minute and returns it with the previous minute's count. A request over the limit gets a 429. When Redis does not answer in 1 ms, the request is allowed through.

*Diagram: the API node's request path calling the limit check, the check's single call to Redis, the limit table with its 30-second refresh, and the fail-open path to the alert.*

### API

No new public endpoint. A rejected request returns `429 Too Many Requests` with `Retry-After: <seconds>` (FR1, FR2). Operators change a limit with `PUT /admin/clients/{id}/limit`, which is idempotent: setting the same limit twice changes nothing (FR3).

### Data model and storage

Two access patterns: increment and read one client's counters on every request, and read one client's limit. Counters live in the existing Redis (LLD1) under `rl:{client_id}:{minute}`, each with a 120-second expiry. Limits live in the existing `clients` table and are cached in each API node for 30 seconds.

### Sliding Window Counter

A fixed one-minute window lets a client send its full limit at the end of one minute and again at the start of the next, which is twice the limit inside 60 seconds and breaks NFR4. The check weights the previous minute's count by how much of it still overlaps the last 60 seconds:

```text
estimate = previous_count * (1 - elapsed_fraction_of_current_minute) + current_count
reject when estimate > limit
```

One Lua script increments the current counter and returns both counts, so the check is one round trip. The estimate assumes the previous minute's requests were evenly spread; the overshoot that assumption allows is the cost.

**Pros:** one call, two small keys a client. **Cons:** the count is an estimate, not exact, and a burst late in the previous minute is undercounted.

### Fail Open

The Redis call has a 1 ms timeout. When it fires, the request is allowed and a `limiter_store_timeout_total` counter is incremented, which the alert watches (NFR3). The check's worst case is then 1 ms, inside NFR2's 2 ms.

### Limit Refresh

Each API node reloads a client's limit from the `clients` table when its cached copy is older than 30 seconds, so a change reaches every node within 30 seconds, inside FR3's 60.

## Alternatives Considered

- **Fixed window counter.** Pro: exact counts and the simplest script. Con: the boundary burst reaches twice the limit, far past NFR4's 5%.
- **Sliding window log.** Pro: exact over any 60 seconds. Con: it stores one entry per request, which at 20,000 requests/s is 1.2 million entries a minute in Redis for a bound NFR4 does not ask for.
- **Token bucket in each API node.** Pro: no store call at all, so no added latency. Con: each node counts alone, so a client's real limit becomes the limit times the number of nodes.

## Failure Modes and Observability

Two failure modes matter for this design.

- **Redis is slow or down.** Every check times out at 1 ms and every request is allowed: the API stays up and the limit is not enforced until Redis answers again.
- **A limit change does not arrive.** A node that cannot read the `clients` table keeps its cached limit past 30 seconds.

Metrics that track:

- `limiter_store_timeout_total`, alerted on any sustained rate.
- `limiter_rejected_total` per client.
- The age of the oldest cached limit on each node.

## Open Questions

1. **Is the 5% bound met when a client's traffic is bursty?** Current design: assumed the previous minute's requests are evenly spread. If that is wrong, a client that bursts at the end of a minute can exceed its limit by more than 5%.

## Work Breakdown

| # | Task | Description | Depends on |
| --- | --- | --- | --- |
| 1 | Sliding window check | The Lua script and the estimate (FR1, FR2, NFR4). | none |
| 2 | Fail open | The 1 ms timeout, the allow path, and the alert (NFR2, NFR3). | Task 1 |
| 3 | Limit refresh | The 30-second limit cache and the admin endpoint (FR3). | none |

## References

- [Redis: EVAL](https://redis.io/docs/latest/commands/eval/), the scripting command the check relies on.

<!-- loop state below: not part of the RFC -->

## Evidence

| Claim | Source | Label |
| --- | --- | --- |
| 20,000 Redis calls a second | NFR1: one call per request | assumed |
| A Lua script runs atomically on one Redis node | https://redis.io/docs/latest/commands/eval/, read 11/09/26 | observed |
| Counter memory is about 1 MB | 5,000 clients × 2 keys × 100 B; the key size is none yet, measure with `MEMORY USAGE` | assumed |

## Revision log

- R0: first draft.
- R1: fixed window replaced by a sliding window counter (R1-1); the Redis call gained a 1 ms timeout (R1-2).
- editorial: adopted

## Objection responses

- R1-1 | accepted | The fixed window allowed twice the limit across a boundary; the check now weights the previous minute.
- R1-2 | accepted | The call had no timeout; it now times out at 1 ms and fails open.
