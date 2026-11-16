---
type: regex
target:
  source: file
  path: design-doc/rate-limiter/requirement.md
pattern: "^\\| LLD1 \\|"
flags: m
match: contains
weight: 1
---

The requirement doc has an `LLD1` row: the accepted Redis gap was applied.
