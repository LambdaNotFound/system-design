---
type: regex
target:
  source: file
  path: design-doc/rate-limiter/rfc.md
pattern: "concepts/"
match: not_contains
weight: 1
---

rfc.md has no link to the concepts folder: the skill did not edit the design.
