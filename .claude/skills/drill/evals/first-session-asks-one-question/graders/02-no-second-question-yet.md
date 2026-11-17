---
type: regex
target:
  source: file
  path: drills/rate-limiter.md
pattern: "^Q2 \\|"
flags: m
match: not_contains
weight: 1
---

The record has no second question: one is asked at a time.
