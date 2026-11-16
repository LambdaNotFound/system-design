---
type: regex
target:
  source: file
  path: order-write.excalidraw
pattern: "\"type\":\\s*\"excalidraw\""
flags: m
match: contains
weight: 1
---

The file is an Excalidraw scene: it has `"type": "excalidraw"`.
