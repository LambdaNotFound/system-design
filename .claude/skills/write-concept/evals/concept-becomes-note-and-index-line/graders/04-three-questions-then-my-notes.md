---
type: regex
target:
  source: file
  path: concepts/consistent-hashing.md
pattern: "^## Check your understanding\\n+1\\. [\\s\\S]+?\\n2\\. [\\s\\S]+?\\n3\\. [\\s\\S]+?\\n+## My notes$"
flags: m
weight: 1
---

Check your understanding is a numbered list of at least three questions, directly followed by My notes.
