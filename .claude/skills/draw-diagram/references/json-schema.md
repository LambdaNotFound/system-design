# Excalidraw JSON Schema

Reference for the fields this skill uses, checked against Excalidraw 0.18.0. Excalidraw fills any field you leave out with its default.

## Element Types

| Type | Use For |
|------|---------|
| `rectangle` | Components, boundaries, bands, header boxes |
| `ellipse` | Cylinder caps, person heads, step badges |
| `diamond` | Decisions |
| `arrow` | Connections; sequence lifelines |
| `line` | Dividers and cylinder sides. A closed `line` (last point equals the first) draws a polygon: hexagon, parallelogram, cloud |
| `text` | All labels |
| `frame` | Named grouping container (rarely needed; prefer a boundary rectangle) |

## Common Properties

| Property | Type | Values |
|----------|------|--------|
| `id` | string | Unique in the file (the check script checks) |
| `type` | string | See above |
| `x`, `y` | number | Position in pixels |
| `width`, `height` | number | Size in pixels |
| `strokeColor` | string | Hex, or `"transparent"` |
| `backgroundColor` | string | Hex, or `"transparent"` |
| `fillStyle` | string | `"solid"`, `"hachure"`, `"cross-hatch"`, `"zigzag"` |
| `strokeWidth` | number | 1 (thin), 2 (standard), 4 (bold) |
| `strokeStyle` | string | `"solid"`, `"dashed"`, `"dotted"` |
| `roughness` | number | 0 (clean), 1 (hand-drawn), 2 (rough) |
| `opacity` | number | 0-100; this skill always uses 100 |
| `roundness` | object or null | `{"type": 3}` rounded rectangle corners; `{"type": 2}` curved lines and arrows; `null` sharp |
| `groupIds` | string[] | Same value on all parts of a composite shape |
| `boundElements` | object[] or null | Text and arrows attached to this element |

## Text Properties

| Property | Description |
|----------|-------------|
| `text` | The display text; `\n` starts a new line. Excalidraw does not wrap text for you |
| `originalText` | Optional; same as `text` |
| `fontSize` | Pixels: 28 title, 16 names and boundary labels, 14 edge labels, 13 captions |
| `fontFamily` | See the font table |
| `textAlign` | `"left"`, `"center"`, `"right"` |
| `verticalAlign` | `"top"`, `"middle"`, `"bottom"` |
| `containerId` | Id of the rectangle, ellipse, diamond, or arrow that holds this text |
| `lineHeight` | Optional; default 1.25 |

There is no bold or italic. Build hierarchy with size and color.

### Font Families

| Value | Font | Use |
|-------|------|-----|
| 9 | Liberation Sans | **Default** for all labels (an Arial-like sans) |
| 3 | Cascadia | Monospace: commands, routes, code, JSON, identifiers |
| 5 | Excalifont | Hand-drawn; only for sketch style |
| 6 | Nunito | Rounded sans; an alternative to 9 |
| 2 | Helvetica | Avoid: it uses the viewer's system font, so it looks different on each machine |
| 1, 7, 8 | Virgil, Lilita One, Comic Shanns | Not used by this skill |

## Arrow and Line Properties

| Property | Description |
|----------|-------------|
| `points` | Array of `[x, y]` offsets from the element's `x`, `y`. The first point is `[0, 0]` |
| `startBinding`, `endBinding` | `{"elementId": "<shape id>", "focus": 0, "gap": 4}`; only on arrows |
| `startArrowhead`, `endArrowhead` | `null`, `"arrow"`, `"triangle"`, `"triangle_outline"`, `"bar"`, `"dot"`, `"circle"`, `"circle_outline"`, `"diamond"`, `"diamond_outline"`, `"crowfoot_one"`, `"crowfoot_many"`, `"crowfoot_one_or_many"` |
| `elbowed` | `true` makes an elbow arrow that the editor re-routes when shapes move. This skill prefers plain points with `roundness: null` |

Arrows can attach to rectangles, ellipses, diamonds, and text, not to `line` shapes.

## Binding Format

```json
{
  "elementId": "shapeId",
  "focus": 0,
  "gap": 4
}
```

The shape must also list the arrow: `"boundElements": [{"id": "<arrow id>", "type": "arrow"}]`. See `element-templates.md` for the full binding rules.
