---
type: regex
target:
  source: file
  path: drills/rate-limiter.md
pattern: "^Q4 \\|"
flags: m
match: not_contains
weight: 1
---

No new question was added while a miss was waiting.
