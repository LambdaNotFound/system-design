---
type: regex
target:
  source: file
  path: design-doc/rate-limiter/scoping.md
pattern: "^Reader test: 11/09/26"
flags: m
match: contains
weight: 1
---

scoping.md still has its `Reader test: 11/09/26` progress line: accepting a gap in the sweep does not send scoping back to phase 5.
