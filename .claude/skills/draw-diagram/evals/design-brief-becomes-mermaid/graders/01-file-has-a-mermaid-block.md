---
type: regex
target:
  source: file
  path: design-doc/rate-limiter/diagrams/request-path.md
pattern: "^```mermaid"
flags: m
match: contains
weight: 1
---

The file has a fenced block whose language is `mermaid`.
