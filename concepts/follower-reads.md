# Follower reads

## What it is
A follower read is a read served by a replica that is not the leader of its replication group. The replica answers from its own copy at a timestamp it can prove is complete, so the read skips the trip to the leader. The answer may be a few seconds old.

## Why it matters
In a leader-based store, a consistent read goes to the leader, because only the leader knows the latest committed write. With the leader in one region and a client in another, every read pays a cross-region round trip: tens of milliseconds, where a local read takes about one. The leader also carries all the read load, and a region that can't reach it can't read at all, even though a full copy of the data sits on a local disk.

Follower reads remove all three costs for reads that tolerate bounded staleness.

Used in: [url-shortener](../design-doc/url-shortener/)

## How it works
A follower can't serve an arbitrary read, because a write it hasn't applied yet may belong in the answer. The leader removes that doubt with a promise.

1. **The leader closes timestamps.** It continuously picks a timestamp a few seconds in the past and promises to accept no new write at or below it. A write that arrives below the closed timestamp is forced to a later timestamp, which can cost that transaction a retry [2].
2. **The promise travels with the log.** Each closed timestamp is sent in the replication stream, tied to a log position [2].
3. **A follower serves what is closed.** Once it has applied the log up to that position, it holds every write at or below the closed timestamp. A read at such a timestamp sees exactly what the leader would have shown at that moment [2].
4. **The read picks a timestamp.** Either the client names one (exact staleness) or the store picks the newest one the local replica can serve inside a bound (bounded staleness) [1][3].

**Worked example** (illustrative values). The leader is 80 ms away. At `12:00:10.0` the local follower's closed timestamp is `12:00:06.5`. Write A committed at `12:00:05`, write B at `12:00:08`.

| Read | Timestamp used | Served by | Latency | Sees |
|---|---|---|---|---|
| Exact, 4.2 s old | `12:00:05.8` | follower | 2 ms | A |
| Exact, 1 s old | `12:00:09.0` | leader: the follower hasn't closed it | 80 ms | A, B |
| Bounded, at most 10 s | `12:00:06.5`, the newest closed | follower | 2 ms | A |
| Strong | now | leader | 80 ms | A, B |

Then the link to the leader drops at `12:00:10`. The follower's closed timestamp freezes at `12:00:06.5`. A bounded read with a 30 s bound still succeeds at `12:00:30`, 23.5 s stale. At `12:00:40` it needs `12:00:10` or newer, the follower can't provide it, and the read fails or tries the unreachable leader.

## Trade-offs and variants
- **Session guarantees.** A client that writes and then reads through a follower inside the lag doesn't see its own write. Two reads that land on replicas with different closed timestamps can go backwards in time [7]. A read that must see the caller's own write goes to the leader.
- **Which data tolerates it.** The url-shortener design reads a redirect with a 30 s bound because a link's URL never changes: staleness can hide a new code or a disable for a while, but can't return a wrong target. A miss on the follower falls through to a consistent read at the leader [8]. An account balance read the same way would be wrong.
- **A lagging follower.** When a follower falls behind, an exact-staleness read that was local becomes a leader read [1]. Under load this moves a whole region's reads onto the leader at the moment replication is already struggling. The fallback needs a rate limit or a fail-fast mode.
- **The size of the lag.** Closing timestamps closer to now makes follower reads fresher and pushes more writes forward, so more transactions retry [2]. TiDB's interval for advancing its equivalent defaults to 20 s, and its docs warn that lowering it raises CPU and inter-node traffic [5].
- **Version retention.** A stale read needs the version it asks for. Spanner keeps versions for 1 hour by default and fails a read older than that [3].
- **Limits on bounded staleness.** The timestamp is negotiated per read, so it can't span statements. CockroachDB limits it to one statement reading one row with no index join [1]. Spanner limits it to single-use read-only transactions [3].

| Variant | Leader round trip per read | How stale | Pick it when |
|---|---|---|---|
| Exact staleness | none, if the timestamp is old enough | fixed by the caller | several reads must share one snapshot: reports, multi-statement read-only transactions |
| Bounded staleness | none | the freshest the replica has, within the bound | point lookups that must stay fast and available when the leader is far or unreachable |
| ReadIndex | one, for the commit index only | none | the leader is near but overloaded, and the read must be current |
| Strong follower reads | none | none | data is read everywhere and written rarely: every write gets slower [1] |

In the ReadIndex variant the follower asks the leader for its latest commit index, waits until it has applied that far, then answers [4]. It moves read work off the leader and keeps linearizability, but it still pays the round trip, so it doesn't help a far region.

## In real systems
- **CockroachDB:** offers both stale kinds. `follower_read_timestamp()` gives an exact timestamp at least 4.2 s old. `with_max_staleness('10s')` gives a bounded read, which keeps serving during a partition and grows staler; with `nearest_only` it errors when no nearby replica qualifies. `GLOBAL` tables serve current reads from any replica and slow their writes [1].
- **Spanner:** names the choices strong, bounded staleness and exact staleness. Exact is slightly faster because it skips the timestamp negotiation, bounded usually returns fresher data, and the docs advise at least 10 s of staleness to see a benefit [3].
- **TiDB:** has two separate features. Follower Read uses ReadIndex and stays strongly consistent, and its docs recommend it for large reads because every read costs a leader interaction [4]. Stale Read reads a historical version from a replica in the local data center [5].
- **MongoDB:** has no closed timestamp. The driver estimates each secondary's lag from periodic checks of last-write times, so `maxStalenessSeconds` can't be set below 90 [6].

## Check your understanding
1. A service inserts a row, then reads it back 500 ms later with an exact-staleness read 4.2 s in the past. What does the read return? Name the cheapest change that fixes this one request without giving up follower reads elsewhere.
2. A region loses its link to the leader for 45 seconds. Its reads use bounded staleness with a 30 s bound and no fallback. Describe what callers see across the 45 seconds. Then say what raising the bound to 120 s buys and what it costs.
3. Two deployments want reads off the leader. In the first, a follower sits in the leader's data center and the leader's CPU is saturated by large scans that must be current. In the second, a follower is 80 ms from the leader and serves point lookups with a 10 ms budget. Pick ReadIndex or bounded staleness for each, and say why the other choice fails.

## My notes
<!-- yours -->

## Sources
1. [Follower Reads](https://docs.cockroachlabs.com/docs/stable/follower-reads), Cockroach Labs, 2026. Read 11/16/26.
2. [Transaction Layer](https://docs.cockroachlabs.com/docs/stable/architecture/transaction-layer), Cockroach Labs, 2026. Read 11/16/26.
3. [Timestamp bounds](https://docs.cloud.google.com/spanner/docs/timestamp-bounds), Google Cloud, 2026. Read 11/16/26.
4. [Follower Read](https://docs.pingcap.com/tidb/stable/follower-read), PingCAP, 2026. Read 11/16/26.
5. [Usage Scenarios of Stale Read](https://docs.pingcap.com/tidb/stable/stale-read), PingCAP, 2026. Read 11/16/26.
6. [Read Preference maxStalenessSeconds](https://www.mongodb.com/docs/manual/core/read-preference-staleness/), MongoDB, 2026. Read 11/16/26.
7. *Designing Data-Intensive Applications*, ch. 5, "Replication", section "Problems with Replication Lag". From memory.
8. [design-doc/url-shortener/rfc.md](../design-doc/url-shortener/rfc.md), this repository, 2026. Read 11/16/26.
