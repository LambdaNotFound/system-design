---
type: regex
target:
  source: file
  path: design-doc/rate-limiter/review.md
pattern: "^## Review 1\\s*$[\\s\\S]*^Focus:.*Redis"
flags: m
weight: 1
---

The ledger has a `## Review 1` section with a `Focus:` line about Redis.
