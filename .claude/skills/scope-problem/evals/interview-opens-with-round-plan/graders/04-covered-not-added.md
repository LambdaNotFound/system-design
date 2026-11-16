---
type: regex
target:
  source: file
  path: design-doc/rate-limiter/scoping.md
pattern: "^Covered:"
flags: m
match: not_contains
weight: 1
---

scoping.md has no `Covered:` progress line: the interview has only begun.
