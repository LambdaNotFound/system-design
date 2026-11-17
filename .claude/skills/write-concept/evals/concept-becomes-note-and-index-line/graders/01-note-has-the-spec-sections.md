---
type: regex
target:
  source: file
  path: concepts/consistent-hashing.md
pattern: "^# .+[\\s\\S]*^## What it is$[\\s\\S]*^## Why it matters$[\\s\\S]*^## How it works$[\\s\\S]*^## Trade-offs and variants$[\\s\\S]*^## In real systems$[\\s\\S]*^## Check your understanding$[\\s\\S]*^## My notes$[\\s\\S]*^## Sources$"
flags: m
weight: 1
---

The note has a title and the spec's eight sections, in the spec's order.
