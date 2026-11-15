---
type: regex
target:
  source: file
  path: design-doc/rate-limiter/impl-run.md
pattern: "^Closed: \\d\\d/\\d\\d/\\d\\d, approve\\s*$"
flags: m
weight: 1
---

impl-run.md has a `Closed:` line with a date and the verdict `approve`.
