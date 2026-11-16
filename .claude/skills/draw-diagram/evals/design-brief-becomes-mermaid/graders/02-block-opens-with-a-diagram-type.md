---
type: regex
target:
  source: file
  path: design-doc/rate-limiter/diagrams/request-path.md
pattern: "^\\s*(flowchart|sequenceDiagram|stateDiagram-v2)\\b"
flags: m
match: contains
weight: 1
---

A line of the file opens a Mermaid diagram with its type.
