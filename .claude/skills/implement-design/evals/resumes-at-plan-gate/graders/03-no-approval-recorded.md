---
type: regex
target:
  source: file
  path: design-doc/rate-limiter/impl-run.md
pattern: "^Plan approved:"
flags: m
match: not_contains
weight: 1
---

impl-run.md has no `Plan approved:` line: the skill recorded no approval the user did not give.
