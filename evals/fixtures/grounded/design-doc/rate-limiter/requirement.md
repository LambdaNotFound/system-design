# Requirement doc: Rate limiter for the public API

Status: draft
Slug: rate-limiter
Scoping record: design-doc/rate-limiter/scoping.md

## Problem Statement

The public API has no per-client limit, so one misbehaving client can exhaust the shared database pool and slow every other client. Operators find out from latency alerts, after the damage. Doing nothing leaves each new integration able to cause an outage (assumed).

## Functional Requirements

| ID | Requirement |
| --- | --- |
| FR1 | The limiter rejects a request with HTTP 429 when the client has exceeded its configured requests per minute. |

## Non-functional Requirements

| ID | Requirement |
| --- | --- |
