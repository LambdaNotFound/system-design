---
type: regex
target:
  source: file
  path: design-doc/rate-limiter/scoping.md
pattern: "^\\| rate-limiter \\|[^|\\n]*(LLD1|\\ball\\b)"
flags: m
match: contains
weight: 1
---

The Decomposition row for rate-limiter covers the accepted gap: its Serves column lists `LLD1` or reads `all`.
