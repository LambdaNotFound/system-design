---
type: regex
target:
  source: file
  path: design-doc/rate-limiter/scoping.md
pattern: "^Final check: .*capped after two runs"
flags: m
match: contains
weight: 1
---

scoping.md has a `Final check:` progress line that says the check was capped after two runs.
