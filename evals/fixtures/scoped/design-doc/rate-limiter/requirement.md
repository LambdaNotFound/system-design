# Requirement doc: Rate limiter for the public API

Status: signed-off 11/09/26
Slug: rate-limiter
Route: design
Scoping record: design-doc/rate-limiter/scoping.md

## Problem Statement

The public API has no per-client limit, so one misbehaving client can exhaust the shared database pool and slow every other client. Operators find out from latency alerts, after the damage. Doing nothing leaves each new integration able to cause an outage. Out of scope: billing by usage; limits on internal service-to-service calls.

## Functional Requirements

| ID | Requirement |
| --- | --- |
| FR1 | The limiter rejects a request with HTTP 429 when the client has exceeded its configured requests per minute. |
| FR2 | A rejected response carries a `Retry-After` header with the seconds until the client may retry. |
| FR3 | An operator can change one client's limit without a deploy, and the change takes effect within 60 seconds. |

## Non-functional Requirements

| ID | Requirement |
| --- | --- |
| NFR1 | **Scale.** The limiter handles 20,000 requests per second across 5,000 clients (assumed). |
| NFR2 | **Latency.** The limiter adds at most 2 ms at p99 to a request, including when its store is slow or down. |
| NFR3 | **Failure behavior.** When the limiter's store does not answer within 1 ms, the request is allowed through and an alert fires. |
| NFR4 | **Correctness bar.** While the store is reachable, a client exceeds its limit by at most 5% in any minute. |

## Low-level Design Preferences

| ID | Preference |
| --- | --- |
| LLD1 | The limiter's store is the Redis deployment that already exists, not a new datastore. |
