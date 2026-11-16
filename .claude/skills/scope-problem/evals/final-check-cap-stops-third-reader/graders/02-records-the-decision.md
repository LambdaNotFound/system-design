---
type: regex
target:
  source: file
  path: design-doc/rate-limiter/scoping.md
pattern: "^\\| C4 \\|[^|\\n]*\\|[^|\\n]*\\|[^|\\n]*\\| *a\\b"
flags: m
match: contains
weight: 1
---

The C4 row in the Conflicts table carries the user's decision, option a.
