---
type: regex
target:
  source: file
  path: drills/rate-limiter.md
pattern: "^Q3 \\| \\d{2}/\\d{2}/\\d{2} \\| hit \\| Fail Open \\| .+ \\| complete$"
flags: m
weight: 1
---

Q3, which the user answered in full, is now a `hit` with `complete` in its last field.
