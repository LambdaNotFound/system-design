---
type: regex
target:
  source: file
  path: drills/rate-limiter.md
pattern: "^Q1 \\| 11/10/26 \\| hit \\| Sliding Window Counter \\| A client sends its full limit in the last second of one minute and again in the first second of the next\\. What does a fixed window allow, and what does the design do about it\\? \\| complete$"
flags: m
weight: 1
---

Q1, a hit, reads exactly as it did.
