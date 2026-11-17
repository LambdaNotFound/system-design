---
type: regex
target:
  source: file
  path: drills/rate-limiter.md
pattern: "^Q1 \\| \\d{2}/\\d{2}/\\d{2} \\| open \\| .+ \\| .+ \\| -$"
flags: m
weight: 1
---

The record's first question line is `open`, with a date, where the answer is, the question, and `-` in the last field.
