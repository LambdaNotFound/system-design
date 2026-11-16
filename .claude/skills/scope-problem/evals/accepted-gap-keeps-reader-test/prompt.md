---
tags: [mid-state, regression]
max_turns: 25
timeout_seconds: 600
allowed_tools: [Read, Glob, Grep, Skill, Write, Edit, Bash]
---

/scope-problem rate-limiter

I've decided the two open items from the conflict sweep. The clash between NFR3 (requests are allowed through when the store is unreachable) and NFR4 (a client exceeds its limit by at most 5%): keep NFR3, and NFR4 holds only while the store is reachable. The Redis constraint from F4 that never reached the doc: accept it, as an LLD row. Apply both and stop after the sweep; I'll do the rest later.
