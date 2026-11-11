---
type: regex
target:
  source: file
  path: design-doc/rate-limiter/requirement.md
pattern: "LLD1"
match: not_contains
weight: 1
---

The requirement doc still has no `LLD1` row: the Redis constraint from F4 that never reached the doc was not added during the sweep without the user's say.
