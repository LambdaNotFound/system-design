---
type: regex
target:
  source: file
  path: design-doc/rate-limiter/scoping.md
pattern: "^\\| *9 *\\|"
flags: m
match: not_contains
weight: 1
---

The Round plan table in scoping.md has no ninth round.
