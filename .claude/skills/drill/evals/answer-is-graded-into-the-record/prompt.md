---
tags: [mid-state, artifact]
max_turns: 20
timeout_seconds: 420
allowed_tools: [Read, Glob, Grep, Skill, Write, Edit, Bash]
---

/drill rate-limiter

It fails open. After the 1 ms timeout the request is allowed through, and the limiter_store_timeout_total counter goes up, which is what the alert watches.
