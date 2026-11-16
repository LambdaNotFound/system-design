---
type: regex
target:
  source: file
  path: design-doc/rate-limiter/rfc.md
pattern: "^\\*Diagram:"
flags: m
match: contains
weight: 1
---

rfc.md still carries its `*Diagram:` line: the skill did not edit the RFC.
