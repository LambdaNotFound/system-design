---
type: regex
target:
  source: file
  path: design-doc/rate-limiter/impl-run.md
pattern: "^D1 \\| \\d\\d/\\d\\d/\\d\\d \\| C1\\b[^|]*\\| as written \\|"
flags: m
weight: 1
---

impl-run.md has a `D1` line that answers C1 with the decision `as written`.
