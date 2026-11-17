---
type: regex
target:
  source: file
  path: design-doc/rate-limiter/impl-checks/R0.sha256
pattern: "^[0-9a-f]{64}  (/|\\./)"
flags: m
match: not_contains
weight: 1
---

No manifest line carries an absolute or dot-prefixed path.
