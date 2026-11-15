---
type: regex
target:
  source: file
  path: README.md
pattern: "^\\|.*Rate Limiter.*\\| Implemented \\|.*\\| \\[src/rate-limiter\\]\\(src/rate-limiter/?\\) \\|\\s*$"
flags: m
weight: 1
---

The Rate Limiter row in the Designs table reads `Implemented`, and its Code column links src/rate-limiter.
