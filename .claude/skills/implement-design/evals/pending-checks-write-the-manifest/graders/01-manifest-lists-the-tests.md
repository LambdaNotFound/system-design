---
type: regex
target:
  source: file
  path: design-doc/rate-limiter/impl-checks/R0.sha256
pattern: "^[0-9a-f]{64}  src/rate-limiter/tests/sliding_window\\.rs$"
flags: m
weight: 1
---

The manifest has a SHA-256 line for the frozen test file, by its repository-relative path.
