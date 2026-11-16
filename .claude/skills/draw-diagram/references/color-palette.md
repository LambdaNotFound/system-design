# Color Palette & Brand Style

**This is the single source of truth for all colors.** To match your brand, change the hex values here and keep the structure. Everything else in the skill refers to these roles, not to hex values.

How color works in this skill (see `style-guide.md`, section 4): each diagram uses color for **one** dimension only, such as component role, owner, layer, or flow. List that dimension's categories, give each category one hue from the table below in order, and keep everything else neutral.

---

## Neutrals

| Use | Color |
|-----|-------|
| Canvas background | `#ffffff` |
| Main text: title, component names | `#1e293b` |
| Boundary labels, lane labels, edge labels | `#475569` |
| Captions, subtitles, notes | `#64748b` |
| Default arrows and lines | `#475569` |
| Neutral component (no category): fill / stroke | `#f8fafc` / `#475569` |
| Boundary (dashed zone): stroke / fill | `#94a3b8` / `#f8fafc` |
| Band (layer, plane, phase): fill, no stroke | `#f1f5f9` |
| Lane divider line | `#cbd5e1` |

---

## Category Hues

Assign these in order, one per category of the dimension you encode. Use 2-5 of them.

| # | Hue | Fill (light) | Stroke | Text on light fill |
|---|-----|--------------|--------|--------------------|
| 1 | Blue | `#dbeafe` | `#2563eb` | `#1e293b` |
| 2 | Green | `#dcfce7` | `#16a34a` | `#1e293b` |
| 3 | Purple | `#ede9fe` | `#7c3aed` | `#1e293b` |
| 4 | Teal | `#ccfbf1` | `#0d9488` | `#1e293b` |
| 5 | Pink | `#fce7f3` | `#db2777` | `#1e293b` |

- **Default:** light fill, darker stroke of the same hue, main-text color inside.
- **Strong fill:** use the stroke color as the fill and `#ffffff` text. Use it only for the main path or 1-3 key components.
- **Color by flow or actor:** color the arrows with the stroke hue. Give each arrow's label and step badge the same color, and keep the boxes neutral.
- **More than 5 categories:** merge categories, or keep the extras neutral and name them in labels.

---

## Reserved Colors

Never use these for categories.

| Meaning | Fill | Stroke | Text |
|---------|------|--------|------|
| External or out of scope | `#cbd5e1` with `fillStyle: "hachure"`, or `#f1f5f9` solid | `#94a3b8` | `#64748b` |
| Step badge | `#fbbf24` | `#d97706` | `#1e293b` |
| Error or failure path | `#fee2e2` | `#dc2626` | `#991b1b` |
| Changed or new part (before/after) | hue 1 (blue), with unchanged parts neutral | | |

---

## Evidence Artifact Colors

Used for code, payloads, and config shown inside the diagram.

| Artifact | Background | Text |
|----------|------------|------|
| Code or JSON block | `#1e293b` | `#e2e8f0` (code), `#86efac` (JSON values), monospace |
| Literal text in a light box or on an arrow | the box's own fill | `#1e293b`, monospace |

---

## Line Styles

Default meanings. If a diagram uses a style for anything else, define it in a legend.

| Style | Meaning |
|-------|---------|
| Solid, `strokeWidth: 2` | Request, call, or data flow |
| `strokeStyle: "dashed"` | Async event, control or config, remote call, or "can be disabled" |
| `strokeStyle: "dotted"` | Polling, optional input, or a secondary link |
| `strokeWidth: 4` | The main or critical path |
| `strokeWidth: 1` | A background or secondary link |
