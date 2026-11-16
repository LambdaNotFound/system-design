---
type: regex
target:
  source: file
  path: design-doc/rate-limiter/scoping.md
pattern: "^\\| *\\d+ *\\|[^|\\n]*[Tt]rade-?offs[^|\\n]*\\|[^|\\n]*\\|\\n\\| *\\d+ *\\|[^|\\n]*[Ww]alk-?back"
flags: m
match: contains
weight: 1
---

The Round plan table has a trade-offs round followed directly by a walk-back round.
