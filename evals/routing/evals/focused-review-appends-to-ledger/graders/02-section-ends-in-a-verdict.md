---
type: regex
target:
  source: file
  path: design-doc/rate-limiter/review.md
pattern: "^## Review 1\\s*$[\\s\\S]*^Verdict: (holds|holds-with-risks|does-not-hold)\\s*$"
flags: m
weight: 1
---

The `## Review 1` section has a verdict line with one of the on-demand review's three values.
