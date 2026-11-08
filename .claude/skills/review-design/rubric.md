# Review rubric

A map of what to look at, not a form to fill in. Skip what doesn't apply to this system.

## Requirements and scope
- Non-functional requirements are quantified: latency percentiles, availability, durability, and the consistency each operation needs.
- Scope fits the problem: core features in, everything else explicitly out.

## Estimates
- Math and units are right; peak is separated from average; the read:write ratio is stated.
- Each number drives a decision (storage → sharding, QPS → caching, bandwidth → CDN). Numbers nothing depends on are noise.

## API
- Retried writes are idempotent; lists paginate; errors are explicit.
- The API doesn't leak storage details or force chatty clients.

## Data model and storage
- The schema follows from the access patterns: keys, indexes, partition key.
- Skew is handled: celebrity users, viral items, time-ordered keys that all land on the newest partition.
- The storage choice is justified (B-tree vs LSM, row vs column, SQL vs KV vs document), not assumed.

## Scaling
- The hot path is identified, and so is how load spreads across it: partitioning, replicas, caching, CDN.
- Caching: what's cached, how it's invalidated, stampede protection, and the hit rate the design assumes.
- Fan-out on write vs on read, queues and back-pressure, batch vs stream.

## Consistency and correctness
- The consistency model is stated per operation, and the design actually delivers it.
- Races, reordering and duplicates are handled: idempotency keys, sequence numbers, no hidden clock-sync assumptions. "Exactly once" usually means effectively once; check how.
- Writes that span shards or services have a story (single-shard design, outbox, saga, 2PC), including its failure cases.

## Availability and failure modes
- No unacknowledged single points of failure. Each dependency has an answer for "slow" as well as "down".
- Timeouts, retries with backoff and jitter, circuit breakers, load shedding, graceful degradation.
- Replication and failover: data-loss window (RPO), recovery time (RTO), split-brain prevention.
- Multi-region, if the requirements call for it.

## Operations
- Metrics and alerts that would catch the failure modes above.
- Deploys, schema migrations, backfills and re-sharding without downtime.

## Security and abuse (when relevant)
- Authentication and authorization, rate limits, abuse vectors (enumeration, spam), PII handling.

## Trade-offs
- Alternatives are named, with why they lost and what would flip the decision.

## Implementation (if src/<slug>/ exists)
- The code implements what the doc says: algorithms, invariants, failure handling.
- The tests cover the edge cases the doc calls out.
