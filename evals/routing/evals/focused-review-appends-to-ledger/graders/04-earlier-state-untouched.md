---
type: regex
target:
  source: file
  path: design-doc/rate-limiter/review.md
pattern: "^## Round 1\\s*$[\\s\\S]*^Verdict: revise\\s*$[\\s\\S]*^## Round 2\\s*$[\\s\\S]*^Verdict: approve\\s*$[\\s\\S]*^## Review 1\\s*$"
flags: m
weight: 1
---

The loop's two rounds and their verdicts are still there, and the review comes after them.
