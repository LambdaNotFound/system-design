---
type: regex
target:
  source: file
  path: design-doc/rate-limiter/impl-run.md
pattern: "^Plan approved: \\d\\d/\\d\\d/\\d\\d, 0 amendments, 0 scope lines\\s*$"
flags: m
weight: 1
---

impl-run.md records the plan approval with the counts from impl.md's first line.
