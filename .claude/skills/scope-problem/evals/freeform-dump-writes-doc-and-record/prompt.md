---
tags: [artifact]
max_turns: 40
timeout_seconds: 900
allowed_tools: [Read, Glob, Grep, Skill, Write, Edit, Bash]
---

/scope-problem slug: rate-limiter

Skip the interview, I'd rather just tell you everything. Treat this as freeform.

We need a per-client rate limiter in front of our public REST API. Today there is no limit at all, and last month one client's retry loop exhausted the shared Postgres connection pool and slowed every other client for 40 minutes. It's for the API clients, and for the on-call operators who got paged. Done means a client over its limit gets a 429 with Retry-After and nobody else's latency moves. We must use the Redis we already run, no new datastore. Peak is about 20k requests a second across roughly 5,000 clients, I think, I haven't measured it. If Redis is down, fail open and alert. Not in scope: billing by usage, and internal service-to-service traffic. There is nothing in this repo to read for it.

Your restatement is fine as long as it says the above, so don't wait for me to confirm it. Go as far as you can before you need a decision from me.
