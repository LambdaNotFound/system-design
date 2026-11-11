---
type: regex
target:
  source: file
  path: design-doc/rate-limiter/requirement.md
pattern: "^\\| FR1 \\|[\\s\\S]*^\\| NFR1 \\|"
flags: m
weight: 1
---

The requirement rows are table rows with IDs: an `FR1` row, and after it an `NFR1` row.
