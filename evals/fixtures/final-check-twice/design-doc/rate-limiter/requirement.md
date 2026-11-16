# Requirement doc: Rate limiter for the public API

Status: draft
Slug: rate-limiter
Route: design
Scoping record: design-doc/rate-limiter/scoping.md

## Problem Statement

The public API has no per-client limit, so one misbehaving client can exhaust the shared database pool and slow every other client. Operators find out from latency alerts, after the damage. Doing nothing leaves each new integration able to cause an outage. Out of scope: billing by usage; limits on internal service-to-service calls.

## Functional Requirements

| ID | Requirement |
| --- | --- |
| FR1 | The limiter rejects a request with HTTP 429 when it counts the client over its configured requests per minute. |
| FR2 | A rejected response carries a `Retry-After` header with the seconds until the client may retry. |
| FR3 | An operator can change one client's limit without a deploy. |

## Non-functional Requirements

| ID | Requirement |
| --- | --- |
| NFR1 | **Scale.** The limiter handles 20,000 requests per second across 5,000 clients (assumed). |
| NFR2 | **Latency.** The limiter adds at most 2 ms at p99 to a request. |
| NFR3 | **Failure behavior.** When the limiter's store is unreachable, requests are allowed through and an alert fires. |
| NFR4 | **Correctness bar.** In any minute during which the store stays reachable (NFR3), a client may exceed its limit by at most 5%. |

## Low-level Design Preferences

| ID | Preference |
| --- | --- |
| LLD1 | The limiter keeps its state in the existing Redis and adds no new datastore (F4). |
