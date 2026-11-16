# Element Templates

Copy-paste JSON for every shape in `style-guide.md`. Each template is a complete JSON array whose elements go into the file's `elements` list. Change ids, positions, sizes, text, and colors; take colors from `color-palette.md`.

## Contents

- Rules for every template
- Text
- Shapes
- Grouping
- Connectors
- Sequence diagrams
- Legend

## Rules for every template

- Excalidraw fills any field you leave out with its default, so templates list only the fields that matter. Always set `type`, `id`, `x`, `y`, `width`, and `height`. Keep `roughness: 0` for the clean style.
- Elements draw in array order, so later elements cover earlier ones. Put boundaries first and labels last.
- **Text inside a shape:** the text has `"containerId": "<shape id>"`, and the shape's `boundElements` lists `{"id": "<text id>", "type": "text"}`. Only rectangles, ellipses, diamonds, and arrows can hold text; text bound to a `line` renders broken.
- **Center text in its shape yourself.** Excalidraw draws it at the text's own `x` and `y`: `x = shape.x + (shape.width - text.width) / 2` and `y = shape.y + (shape.height - text.height) / 2`, with `text.height = fontSize × 1.25 × lines`.
- **Arrow attached to shapes:** the arrow has `startBinding` and `endBinding` set to `{"elementId": "<shape id>", "focus": 0, "gap": 4}`, and each shape's `boundElements` lists `{"id": "<arrow id>", "type": "arrow"}`.
- **Arrow label:** the text has `"containerId": "<arrow id>"`, and the arrow's `boundElements` lists the text. Excalidraw centers the label on the arrow and cuts a gap in the line, so the label's `x` and `y` do not matter. Make the arrow at least 60 px longer than the label.
- **Shapes made of several parts** (cylinder, person, stack, cloud, hexagon, code block): give every part the same `groupIds`, so they move together in the editor.
- **Line shapes** (cloud, hexagon, parallelogram) are closed `line` elements: the last point equals the first. Arrows cannot attach to them, so place arrows by coordinates and leave out the bindings.
- **Text width:** use about `0.6 × fontSize × characters` for the longest line. Make boxes at least 40 px wider than their text.

---

## Text

### Title (free-floating)
For captions and notes, use `fontSize` 13 and color `#64748b`. For a boundary label, use `fontSize` 16 and color `#475569`.

```json
[
  {"type": "text", "id": "title", "x": 0, "y": 0, "width": 440, "height": 35,
   "text": "Checkout: placing an order", "fontSize": 28, "fontFamily": 9,
   "textAlign": "left", "verticalAlign": "top", "strokeColor": "#1e293b"}
]
```

---

## Shapes

### Component box, with a caption below
The name goes inside, and the technology, binary name, or count goes in a 13 px caption below. Same-kind components use the same size and colors.

```json
[
  {"type": "rectangle", "id": "orders", "x": 0, "y": 0, "width": 180, "height": 70,
   "strokeColor": "#2563eb", "backgroundColor": "#dbeafe", "fillStyle": "solid",
   "strokeWidth": 2, "roughness": 0, "roundness": {"type": 3},
   "boundElements": [{"id": "orders_label", "type": "text"}]},
  {"type": "text", "id": "orders_label", "x": 10, "y": 25, "width": 160, "height": 20,
   "text": "Order Service", "fontSize": 16, "fontFamily": 9,
   "textAlign": "center", "verticalAlign": "middle", "strokeColor": "#1e293b",
   "containerId": "orders"},
  {"type": "text", "id": "orders_caption", "x": 0, "y": 78, "width": 180, "height": 17,
   "text": "Go · orders-svc", "fontSize": 13, "fontFamily": 9,
   "textAlign": "center", "verticalAlign": "top", "strokeColor": "#64748b"}
]
```

### Database cylinder
Five parts plus a label. Attach arrows to the body (`orders_db`).

```json
[
  {"type": "ellipse", "id": "orders_db_bottom", "x": 0, "y": 82, "width": 140, "height": 28,
   "strokeColor": "#16a34a", "backgroundColor": "#dcfce7", "fillStyle": "solid",
   "strokeWidth": 2, "roughness": 0, "groupIds": ["orders_db_group"]},
  {"type": "rectangle", "id": "orders_db", "x": 0, "y": 14, "width": 140, "height": 82,
   "strokeColor": "transparent", "backgroundColor": "#dcfce7", "fillStyle": "solid",
   "roughness": 0, "groupIds": ["orders_db_group"],
   "boundElements": [{"id": "orders_db_label", "type": "text"}]},
  {"type": "line", "id": "orders_db_left", "x": 0, "y": 14, "width": 0, "height": 82,
   "points": [[0, 0], [0, 82]], "strokeColor": "#16a34a", "strokeWidth": 2,
   "roughness": 0, "groupIds": ["orders_db_group"]},
  {"type": "line", "id": "orders_db_right", "x": 140, "y": 14, "width": 0, "height": 82,
   "points": [[0, 0], [0, 82]], "strokeColor": "#16a34a", "strokeWidth": 2,
   "roughness": 0, "groupIds": ["orders_db_group"]},
  {"type": "ellipse", "id": "orders_db_top", "x": 0, "y": 0, "width": 140, "height": 28,
   "strokeColor": "#16a34a", "backgroundColor": "#dcfce7", "fillStyle": "solid",
   "strokeWidth": 2, "roughness": 0, "groupIds": ["orders_db_group"]},
  {"type": "text", "id": "orders_db_label", "x": 10, "y": 45, "width": 120, "height": 20,
   "text": "Postgres", "fontSize": 16, "fontFamily": 9,
   "textAlign": "center", "verticalAlign": "middle", "strokeColor": "#1e293b",
   "containerId": "orders_db", "groupIds": ["orders_db_group"]}
]
```


### Stacked replicas (many instances)
Two copies sit behind the front box, offset up and to the right. Attach arrows to the front box's left or bottom side; an arrow at the top or right side crosses the copies. If most arrows leave to the right, mirror the offsets so the copies stick out down and to the left. You can also put "×N" in the label.

```json
[
  {"type": "rectangle", "id": "workers_back2", "x": 16, "y": 0, "width": 180, "height": 70,
   "strokeColor": "#7c3aed", "backgroundColor": "#ede9fe", "fillStyle": "solid",
   "strokeWidth": 2, "roughness": 0, "roundness": {"type": 3}, "groupIds": ["workers_group"]},
  {"type": "rectangle", "id": "workers_back1", "x": 8, "y": 8, "width": 180, "height": 70,
   "strokeColor": "#7c3aed", "backgroundColor": "#ede9fe", "fillStyle": "solid",
   "strokeWidth": 2, "roughness": 0, "roundness": {"type": 3}, "groupIds": ["workers_group"]},
  {"type": "rectangle", "id": "workers", "x": 0, "y": 16, "width": 180, "height": 70,
   "strokeColor": "#7c3aed", "backgroundColor": "#ede9fe", "fillStyle": "solid",
   "strokeWidth": 2, "roughness": 0, "roundness": {"type": 3}, "groupIds": ["workers_group"],
   "boundElements": [{"id": "workers_label", "type": "text"}]},
  {"type": "text", "id": "workers_label", "x": 10, "y": 41, "width": 160, "height": 20,
   "text": "Worker ×3", "fontSize": 16, "fontFamily": 9,
   "textAlign": "center", "verticalAlign": "middle", "strokeColor": "#1e293b",
   "containerId": "workers", "groupIds": ["workers_group"]}
]
```


### Person (human user or actor)
Attach arrows to the body (`customer`).

```json
[
  {"type": "ellipse", "id": "customer_head", "x": 22, "y": 0, "width": 36, "height": 36,
   "strokeColor": "#475569", "backgroundColor": "#f8fafc", "fillStyle": "solid",
   "strokeWidth": 2, "roughness": 0, "groupIds": ["customer_group"]},
  {"type": "rectangle", "id": "customer", "x": 8, "y": 40, "width": 64, "height": 40,
   "strokeColor": "#475569", "backgroundColor": "#f8fafc", "fillStyle": "solid",
   "strokeWidth": 2, "roughness": 0, "roundness": {"type": 3}, "groupIds": ["customer_group"]},
  {"type": "text", "id": "customer_caption", "x": -10, "y": 88, "width": 100, "height": 20,
   "text": "Customer", "fontSize": 16, "fontFamily": 9,
   "textAlign": "center", "verticalAlign": "top", "strokeColor": "#1e293b",
   "groupIds": ["customer_group"]}
]
```


### External system (out of scope)
Gray hatch fill and gray text. Place it at the canvas edge.

```json
[
  {"type": "rectangle", "id": "stripe", "x": 0, "y": 0, "width": 180, "height": 70,
   "strokeColor": "#94a3b8", "backgroundColor": "#cbd5e1", "fillStyle": "hachure",
   "strokeWidth": 1, "roughness": 0, "roundness": {"type": 3},
   "boundElements": [{"id": "stripe_label", "type": "text"}]},
  {"type": "text", "id": "stripe_label", "x": 10, "y": 25, "width": 160, "height": 20,
   "text": "Stripe API", "fontSize": 16, "fontFamily": 9,
   "textAlign": "center", "verticalAlign": "middle", "strokeColor": "#64748b",
   "containerId": "stripe"}
]
```

### Cloud (the Internet, an external network)

```json
[
  {"type": "line", "id": "internet", "x": 0, "y": 0, "width": 200, "height": 115,
   "points": [[30, 110], [100, 112], [170, 110], [192, 92], [188, 66], [168, 54], [160, 28],
              [132, 18], [112, 30], [98, 8], [66, 6], [52, 30], [30, 28], [12, 50], [14, 82], [30, 110]],
   "strokeColor": "#64748b", "backgroundColor": "#f1f5f9", "fillStyle": "solid",
   "strokeWidth": 2, "roughness": 0, "roundness": {"type": 2}, "groupIds": ["internet_group"]},
  {"type": "text", "id": "internet_label", "x": 50, "y": 52, "width": 100, "height": 20,
   "text": "Internet", "fontSize": 16, "fontFamily": 9,
   "textAlign": "center", "verticalAlign": "top", "strokeColor": "#475569",
   "groupIds": ["internet_group"]}
]
```

### Hexagon (proxy, load balancer, router)

```json
[
  {"type": "line", "id": "envoy", "x": 0, "y": 0, "width": 160, "height": 70,
   "points": [[160, 35], [130, 70], [30, 70], [0, 35], [30, 0], [130, 0], [160, 35]],
   "strokeColor": "#7c3aed", "backgroundColor": "#ede9fe", "fillStyle": "solid",
   "strokeWidth": 2, "roughness": 0, "roundness": null, "groupIds": ["envoy_group"]},
  {"type": "text", "id": "envoy_label", "x": 20, "y": 25, "width": 120, "height": 20,
   "text": "Envoy", "fontSize": 16, "fontFamily": 9,
   "textAlign": "center", "verticalAlign": "top", "strokeColor": "#1e293b",
   "groupIds": ["envoy_group"]}
]
```

### Diamond (decision)

```json
[
  {"type": "diamond", "id": "cache_hit", "x": 0, "y": 0, "width": 160, "height": 100,
   "strokeColor": "#475569", "backgroundColor": "#f8fafc", "fillStyle": "solid",
   "strokeWidth": 2, "roughness": 0,
   "boundElements": [{"id": "cache_hit_label", "type": "text"}]},
  {"type": "text", "id": "cache_hit_label", "x": 30, "y": 40, "width": 100, "height": 20,
   "text": "Cache hit?", "fontSize": 16, "fontFamily": 9,
   "textAlign": "center", "verticalAlign": "middle", "strokeColor": "#1e293b",
   "containerId": "cache_hit"}
]
```

### Parallelogram (input or output data)

```json
[
  {"type": "line", "id": "query_input", "x": 0, "y": 0, "width": 180, "height": 70,
   "points": [[24, 0], [180, 0], [156, 70], [0, 70], [24, 0]],
   "strokeColor": "#475569", "backgroundColor": "#f8fafc", "fillStyle": "solid",
   "strokeWidth": 2, "roughness": 0, "roundness": null, "groupIds": ["query_input_group"]},
  {"type": "text", "id": "query_input_label", "x": 20, "y": 25, "width": 140, "height": 20,
   "text": "User question", "fontSize": 16, "fontFamily": 9,
   "textAlign": "center", "verticalAlign": "top", "strokeColor": "#1e293b",
   "groupIds": ["query_input_group"]}
]
```

### Step badge
Center the badge on its arrow, 24 px from the arrow's first point, and put it after the arrow in the array so it draws on top. The arrow's label sits at the arrow's midpoint, so an arrow with a badge must be at least its label width + 100 px long (at least 120 px if vertical). Use letters for parallel branches ("5B"); widen the badge to 36 px for 2 characters.

When the steps are boxes (a pipeline of stages), center the badge on each box's top-left corner instead: badge `x = box.x - 14`, `y = box.y - 14`. Inside a boundary with a label, start such a box at least 60 px below the boundary's top, so the badge clears the label.

```json
[
  {"type": "ellipse", "id": "step1", "x": 0, "y": 0, "width": 28, "height": 28,
   "strokeColor": "#d97706", "backgroundColor": "#fbbf24", "fillStyle": "solid",
   "strokeWidth": 1, "roughness": 0,
   "boundElements": [{"id": "step1_label", "type": "text"}]},
  {"type": "text", "id": "step1_label", "x": 4, "y": 5, "width": 20, "height": 18,
   "text": "1", "fontSize": 14, "fontFamily": 9,
   "textAlign": "center", "verticalAlign": "middle", "strokeColor": "#1e293b",
   "containerId": "step1"}
]
```

### Code or JSON block
A dark block with monospace text. The text is free-floating, not bound, so it stays left-aligned.

```json
[
  {"type": "rectangle", "id": "payload", "x": 0, "y": 0, "width": 260, "height": 96,
   "strokeColor": "#1e293b", "backgroundColor": "#1e293b", "fillStyle": "solid",
   "strokeWidth": 1, "roughness": 0, "roundness": {"type": 3}, "groupIds": ["payload_group"]},
  {"type": "text", "id": "payload_text", "x": 16, "y": 14, "width": 200, "height": 65,
   "text": "{\n  \"order_id\": \"ord_7Hk2\",\n  \"total\": 42.50\n}", "fontSize": 13, "fontFamily": 3,
   "textAlign": "left", "verticalAlign": "top", "strokeColor": "#86efac",
   "groupIds": ["payload_group"]}
]
```

---

## Grouping

### Boundary (network, trust, or deployment zone)
Draw it before its children. Put children at least 44 px below the top edge and 30 px from the sides.

```json
[
  {"type": "rectangle", "id": "main_region", "x": 0, "y": 0, "width": 460, "height": 200,
   "strokeColor": "#94a3b8", "backgroundColor": "#f8fafc", "fillStyle": "solid",
   "strokeWidth": 2, "strokeStyle": "dashed", "roughness": 0, "roundness": {"type": 3}},
  {"type": "text", "id": "main_region_label", "x": 16, "y": 12, "width": 200, "height": 20,
   "text": "Main Region", "fontSize": 16, "fontFamily": 9,
   "textAlign": "left", "verticalAlign": "top", "strokeColor": "#475569"}
]
```

For a product or system boundary, use `"strokeStyle": "solid"`. For a deeper level, change the style, for example a solid inner box inside a dashed zone.

### Band (layer, plane, or phase)
No border, only a tint.

```json
[
  {"type": "rectangle", "id": "control_plane", "x": 0, "y": 0, "width": 700, "height": 120,
   "strokeColor": "transparent", "backgroundColor": "#f1f5f9", "fillStyle": "solid",
   "roughness": 0, "roundness": {"type": 3}},
  {"type": "text", "id": "control_plane_label", "x": 16, "y": 12, "width": 200, "height": 20,
   "text": "Control plane", "fontSize": 16, "fontFamily": 9,
   "textAlign": "left", "verticalAlign": "top", "strokeColor": "#475569"}
]
```

### Lane divider (flat tiers)
A dashed vertical line between lanes, with each lane's title centered at the top of the lane.

```json
[
  {"type": "text", "id": "lane_web_title", "x": 0, "y": 0, "width": 200, "height": 20,
   "text": "Presentation", "fontSize": 16, "fontFamily": 9,
   "textAlign": "center", "verticalAlign": "top", "strokeColor": "#475569"},
  {"type": "line", "id": "lane_divider", "x": 220, "y": 0, "width": 0, "height": 400,
   "points": [[0, 0], [0, 400]], "strokeColor": "#cbd5e1", "strokeWidth": 2,
   "strokeStyle": "dashed", "roughness": 0},
  {"type": "text", "id": "lane_api_title", "x": 240, "y": 0, "width": 200, "height": 20,
   "text": "Application logic", "fontSize": 16, "fontFamily": 9,
   "textAlign": "center", "verticalAlign": "top", "strokeColor": "#475569"}
]
```

---

## Connectors

### Labeled arrow between two boxes
This shows the bindings on both ends. The route label is a literal, so it uses the monospace font.

```json
[
  {"type": "rectangle", "id": "checkout", "x": 0, "y": 0, "width": 180, "height": 70,
   "strokeColor": "#2563eb", "backgroundColor": "#dbeafe", "fillStyle": "solid",
   "strokeWidth": 2, "roughness": 0, "roundness": {"type": 3},
   "boundElements": [{"id": "checkout_label", "type": "text"}, {"id": "checkout_to_payments", "type": "arrow"}]},
  {"type": "text", "id": "checkout_label", "x": 10, "y": 25, "width": 160, "height": 20,
   "text": "Checkout", "fontSize": 16, "fontFamily": 9,
   "textAlign": "center", "verticalAlign": "middle", "strokeColor": "#1e293b",
   "containerId": "checkout"},
  {"type": "rectangle", "id": "payments", "x": 400, "y": 0, "width": 180, "height": 70,
   "strokeColor": "#2563eb", "backgroundColor": "#dbeafe", "fillStyle": "solid",
   "strokeWidth": 2, "roughness": 0, "roundness": {"type": 3},
   "boundElements": [{"id": "payments_label", "type": "text"}, {"id": "checkout_to_payments", "type": "arrow"}]},
  {"type": "text", "id": "payments_label", "x": 410, "y": 25, "width": 160, "height": 20,
   "text": "Payments", "fontSize": 16, "fontFamily": 9,
   "textAlign": "center", "verticalAlign": "middle", "strokeColor": "#1e293b",
   "containerId": "payments"},
  {"type": "arrow", "id": "checkout_to_payments", "x": 184, "y": 35, "width": 212, "height": 0,
   "points": [[0, 0], [212, 0]], "strokeColor": "#475569", "strokeWidth": 2, "roughness": 0,
   "startArrowhead": null, "endArrowhead": "triangle",
   "startBinding": {"elementId": "checkout", "focus": 0, "gap": 4},
   "endBinding": {"elementId": "payments", "focus": 0, "gap": 4},
   "boundElements": [{"id": "checkout_to_payments_label", "type": "text"}]},
  {"type": "text", "id": "checkout_to_payments_label", "x": 240, "y": 26, "width": 101, "height": 18,
   "text": "POST /charge", "fontSize": 14, "fontFamily": 3,
   "textAlign": "center", "verticalAlign": "middle", "strokeColor": "#475569",
   "containerId": "checkout_to_payments"}
]
```

### Right-angle arrow
More than 2 points with `"roundness": null` draws sharp corners. Excalidraw also has `"elbowed": true` arrows, which the editor re-routes when shapes move. Plain points are more predictable when you write JSON by hand.

```json
[
  {"type": "rectangle", "id": "api", "x": 0, "y": 0, "width": 180, "height": 70,
   "strokeColor": "#2563eb", "backgroundColor": "#dbeafe", "fillStyle": "solid",
   "strokeWidth": 2, "roughness": 0, "roundness": {"type": 3},
   "boundElements": [{"id": "api_label", "type": "text"}, {"id": "api_to_queue", "type": "arrow"}]},
  {"type": "text", "id": "api_label", "x": 10, "y": 25, "width": 160, "height": 20,
   "text": "API", "fontSize": 16, "fontFamily": 9,
   "textAlign": "center", "verticalAlign": "middle", "strokeColor": "#1e293b",
   "containerId": "api"},
  {"type": "rectangle", "id": "queue", "x": 300, "y": 160, "width": 180, "height": 70,
   "strokeColor": "#0d9488", "backgroundColor": "#ccfbf1", "fillStyle": "solid",
   "strokeWidth": 2, "roughness": 0, "roundness": {"type": 3},
   "boundElements": [{"id": "queue_label", "type": "text"}, {"id": "api_to_queue", "type": "arrow"}]},
  {"type": "text", "id": "queue_label", "x": 310, "y": 185, "width": 160, "height": 20,
   "text": "jobs queue", "fontSize": 16, "fontFamily": 9,
   "textAlign": "center", "verticalAlign": "middle", "strokeColor": "#1e293b",
   "containerId": "queue"},
  {"type": "arrow", "id": "api_to_queue", "x": 90, "y": 74, "width": 206, "height": 121,
   "points": [[0, 0], [0, 121], [206, 121]], "strokeColor": "#475569", "strokeWidth": 2,
   "roughness": 0, "roundness": null, "startArrowhead": null, "endArrowhead": "triangle",
   "startBinding": {"elementId": "api", "focus": 0, "gap": 4},
   "endBinding": {"elementId": "queue", "focus": 0, "gap": 4}}
]
```

Where a bound label lands on a multi-segment arrow: with 3 points it sits on the corner and splits it, which looks broken. With 4 points it sits in the middle of the middle segment, which looks fine. For a 3-point arrow, put free-floating text beside the long segment instead.

### Curved arrow (feedback, return, or skip path)
Three or more points with `"roundness": {"type": 2}` draws a smooth curve.

```json
[
  {"type": "arrow", "id": "retry", "x": 0, "y": 0, "width": 300, "height": 60,
   "points": [[0, 0], [150, 60], [300, 0]], "strokeColor": "#475569", "strokeWidth": 2,
   "strokeStyle": "dashed", "roughness": 0, "roundness": {"type": 2},
   "startArrowhead": null, "endArrowhead": "triangle"}
]
```

### Arrow variants
Change these fields on any arrow:

| Variant | Fields |
|---|---|
| Async, control, or config | `"strokeStyle": "dashed"` |
| Polling or optional | `"strokeStyle": "dotted"` |
| Main or critical path | `"strokeWidth": 4` |
| Request and response, or a long-lived connection | `"startArrowhead": "triangle"` (keep `endArrowhead`) |
| Mark the side that opens the connection | `"startArrowhead": "dot"` |
| Undirected link, such as a bus | `"endArrowhead": null` |
| Data model (ER) cardinality | `"crowfoot_one"`, `"crowfoot_many"`, `"crowfoot_one_or_many"` |

---

## Sequence diagrams

### Lifelines, a message, and a collapsed sub-flow
Each lifeline is a header box plus a gray arrow pointing down. Messages are arrows between lifelines, colored by the sender and numbered in the label. A faint dashed two-way bar stands for a sub-flow you skip. If a bar crosses another lifeline, its label lands on that lifeline, so put free-floating text above the bar, in a gap between two lifelines.

```json
[
  {"type": "rectangle", "id": "browser_hdr", "x": 0, "y": 0, "width": 160, "height": 50,
   "strokeColor": "#475569", "backgroundColor": "#f8fafc", "fillStyle": "solid",
   "strokeWidth": 2, "roughness": 0, "roundness": {"type": 3},
   "boundElements": [{"id": "browser_hdr_label", "type": "text"}]},
  {"type": "text", "id": "browser_hdr_label", "x": 10, "y": 15, "width": 140, "height": 20,
   "text": "Browser", "fontSize": 16, "fontFamily": 9,
   "textAlign": "center", "verticalAlign": "middle", "strokeColor": "#1e293b",
   "containerId": "browser_hdr"},
  {"type": "arrow", "id": "browser_life", "x": 80, "y": 50, "width": 0, "height": 300,
   "points": [[0, 0], [0, 300]], "strokeColor": "#94a3b8", "strokeWidth": 2,
   "roughness": 0, "startArrowhead": null, "endArrowhead": "triangle"},
  {"type": "rectangle", "id": "app_hdr", "x": 300, "y": 0, "width": 160, "height": 50,
   "strokeColor": "#475569", "backgroundColor": "#f8fafc", "fillStyle": "solid",
   "strokeWidth": 2, "roughness": 0, "roundness": {"type": 3},
   "boundElements": [{"id": "app_hdr_label", "type": "text"}]},
  {"type": "text", "id": "app_hdr_label", "x": 310, "y": 15, "width": 140, "height": 20,
   "text": "Your App", "fontSize": 16, "fontFamily": 9,
   "textAlign": "center", "verticalAlign": "middle", "strokeColor": "#1e293b",
   "containerId": "app_hdr"},
  {"type": "arrow", "id": "app_life", "x": 380, "y": 50, "width": 0, "height": 300,
   "points": [[0, 0], [0, 300]], "strokeColor": "#94a3b8", "strokeWidth": 2,
   "roughness": 0, "startArrowhead": null, "endArrowhead": "triangle"},
  {"type": "arrow", "id": "msg1", "x": 82, "y": 110, "width": 296, "height": 0,
   "points": [[0, 0], [296, 0]], "strokeColor": "#db2777", "strokeWidth": 2,
   "roughness": 0, "startArrowhead": null, "endArrowhead": "triangle",
   "boundElements": [{"id": "msg1_label", "type": "text"}]},
  {"type": "text", "id": "msg1_label", "x": 0, "y": 0, "width": 110, "height": 18,
   "text": "1. GET /login", "fontSize": 14, "fontFamily": 9,
   "textAlign": "center", "verticalAlign": "middle", "strokeColor": "#db2777",
   "containerId": "msg1"},
  {"type": "arrow", "id": "auth_subflow", "x": 82, "y": 200, "width": 296, "height": 0,
   "points": [[0, 0], [296, 0]], "strokeColor": "#cbd5e1", "strokeWidth": 2,
   "strokeStyle": "dashed", "roughness": 0, "startArrowhead": "triangle", "endArrowhead": "triangle",
   "boundElements": [{"id": "auth_subflow_label", "type": "text"}]},
  {"type": "text", "id": "auth_subflow_label", "x": 0, "y": 0, "width": 140, "height": 18,
   "text": "Authenticate user", "fontSize": 14, "fontFamily": 9,
   "textAlign": "center", "verticalAlign": "middle", "strokeColor": "#94a3b8",
   "containerId": "auth_subflow"}
]
```

---

## Legend
Use swatches for colors and short line samples for line styles. Cover every code used in the diagram.

```json
[
  {"type": "rectangle", "id": "legend", "x": 0, "y": 0, "width": 230, "height": 170,
   "strokeColor": "#cbd5e1", "backgroundColor": "#ffffff", "fillStyle": "solid",
   "strokeWidth": 1, "roughness": 0, "roundness": {"type": 3}},
  {"type": "text", "id": "legend_title", "x": 16, "y": 12, "width": 80, "height": 18,
   "text": "Legend", "fontSize": 14, "fontFamily": 9, "strokeColor": "#475569"},
  {"type": "rectangle", "id": "legend_sw1", "x": 16, "y": 44, "width": 28, "height": 18,
   "strokeColor": "#2563eb", "backgroundColor": "#dbeafe", "fillStyle": "solid",
   "strokeWidth": 2, "roughness": 0, "roundness": {"type": 3}},
  {"type": "text", "id": "legend_sw1_text", "x": 56, "y": 44, "width": 120, "height": 18,
   "text": "Our service", "fontSize": 14, "fontFamily": 9, "strokeColor": "#1e293b"},
  {"type": "rectangle", "id": "legend_sw2", "x": 16, "y": 72, "width": 28, "height": 18,
   "strokeColor": "#16a34a", "backgroundColor": "#dcfce7", "fillStyle": "solid",
   "strokeWidth": 2, "roughness": 0, "roundness": {"type": 3}},
  {"type": "text", "id": "legend_sw2_text", "x": 56, "y": 72, "width": 120, "height": 18,
   "text": "Data store", "fontSize": 14, "fontFamily": 9, "strokeColor": "#1e293b"},
  {"type": "line", "id": "legend_ls1", "x": 16, "y": 121, "width": 28, "height": 0,
   "points": [[0, 0], [28, 0]], "strokeColor": "#475569", "strokeWidth": 2, "roughness": 0},
  {"type": "text", "id": "legend_ls1_text", "x": 56, "y": 112, "width": 120, "height": 18,
   "text": "Request", "fontSize": 14, "fontFamily": 9, "strokeColor": "#1e293b"},
  {"type": "line", "id": "legend_ls2", "x": 16, "y": 147, "width": 28, "height": 0,
   "points": [[0, 0], [28, 0]], "strokeColor": "#475569", "strokeWidth": 2,
   "strokeStyle": "dashed", "roughness": 0},
  {"type": "text", "id": "legend_ls2_text", "x": 56, "y": 138, "width": 120, "height": 18,
   "text": "Async event", "fontSize": 14, "fontFamily": 9, "strokeColor": "#1e293b"}
]
```
