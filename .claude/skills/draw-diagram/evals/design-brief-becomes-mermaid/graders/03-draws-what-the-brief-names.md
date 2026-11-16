---
type: regex
target:
  source: file
  path: design-doc/rate-limiter/diagrams/request-path.md
pattern: "redis"
flags: i
match: contains
weight: 1
---

The diagram shows Redis, which the RFC's `*Diagram:` line names.
