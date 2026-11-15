---
type: regex
target:
  source: file
  path: design-doc/rate-limiter/impl-review.md
pattern: "^## Tests round 1\\s*$[\\s\\S]*^Verdict: approve\\s*$[\\s\\S]*^## Round 1\\s*$[\\s\\S]*^Verdict: approve\\s*$(?![\\s\\S]*^## )"
flags: m
weight: 1
---

The ledger still holds its two sections and ends with `Verdict: approve`.
