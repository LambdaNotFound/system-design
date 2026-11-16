# Excalidraw Reference

How to write this skill's diagrams as Excalidraw JSON. Read `style-guide.md` first, and take colors from `color-palette.md`. Tested JSON for every shape is in `element-templates.md`; field values are in `json-schema.md`.

## Visual Vocabulary

| Thing | Shape | Template |
|-------|-------|----------|
| Service, process, component | Rounded rectangle, light fill | Component box |
| Database, cache, store | Cylinder | Database cylinder |
| Human user or actor | Person icon | Person |
| External system or SaaS | Gray hatched box | External system |
| The Internet, an external network | Cloud | Cloud |
| Proxy, load balancer, router | Hexagon | Hexagon |
| Decision | Diamond | Diamond |
| Input or output data | Parallelogram | Parallelogram |
| Many instances | 2 offset copies behind, or "×N" | Stacked replicas |
| Step in a flow | Numbered badge | Step badge |
| Code, payload, config | Dark block, monospace text | Code or JSON block |
| Network or trust zone | Dashed rounded rectangle, top-left label | Boundary |
| Layer, plane, or phase | Tinted band, no border | Band |
| Flat tiers | Dashed vertical divider | Lane divider |

## Text

- **Fonts:** `fontFamily: 9` (Liberation Sans) for every label. `fontFamily: 3` (monospace) only for literal text: commands, routes, code, JSON, identifiers. `fontFamily: 5` (hand-drawn) only for sketch style.
- **No bold or italic** exists in Excalidraw. Build hierarchy with size and color.
- **Sizes:** title 28, boundary label 16, component name 16, edge label 14, caption 13.
- **Two-tier labels:** the name goes inside the box, and the technology, binary name, or count goes in a 13 px caption below it.
- **The JSON `text` property contains only readable words.** No markup.
- **Fit the text.** Plan with about `0.6 × fontSize` px per character. Make boxes at least 40 px wider than their longest line. Excalidraw does not wrap text. The check script measures with the fonts' real widths and warns when a line does not fit.
- **Center text in its shape yourself.** Excalidraw draws it at its own `x`, `y`. Labels bound to arrows are the exception: Excalidraw centers them for you.

## Layout and Spacing

- Snap positions to a 20 px grid.
- Same-kind components get the same size. The default component box is 180×70.
- Leave at least 100 px between connected components, and enough that each arrow is at least 60 px longer than its label, or 100 px longer if it also carries a step badge. On a vertical arrow, the label's height is what lies along it. On an arrow with 4 points, the label sits on the middle segment, so that segment needs the room.
- Center each step badge on its arrow, 24 px from the arrow's start, so it never covers the label at the midpoint. For a numbered stage box, center the badge on the box's top-left corner.
- When 3 or more arrows leave one box, spread their attach points at least 40 px apart, or use a second side. Merge two same-direction arrows between the same boxes into one arrow with one label.
- Keep captions out of arrow paths: put each caption on a side of its box that no arrow uses.
- Boundaries get 30 to 40 px padding. Children start at least 44 px below a boundary's top edge.
- Nest boundaries at most 3 levels deep, with a different style per level.
- Use right-angle arrows in architecture diagrams, and curves for feedback, return, and skip paths.

## Style Modes

- **Clean (default):** `roughness: 0`, `fontFamily: 9`, `strokeWidth: 2`.
- **Sketch** (only when the user asks for hand-drawn): `roughness: 1`, `fontFamily: 5`. About 1 in 10 repo diagrams use this look (Traefik, LocalAI, HNSW).
- **Always `opacity: 100`.** Use color, size, and stroke width for hierarchy, not transparency.

## Large Diagrams

**Build a comprehensive or technical diagram one section at a time.** Do not write the whole file in one response. A response has an output limit of about 32,000 tokens, and a comprehensive diagram easily exceeds it. Section by section also gives a better layout.

1. **Create the base file** with the JSON wrapper (`type`, `version`, `appState`, `files`), the title, and the outer boundaries.
2. **Add one section per edit**, for example one boundary and its components. Think about the layout, the spacing, and how this section connects to what is already there.
3. **Use descriptive string IDs** (for example `"orders_db"`, `"api_to_queue"`), so references across sections are readable and the check script's messages name them.
4. **Add arrows with their labels and badges** once both ends exist. In the same edit, add each arrow's id to the `boundElements` of both shapes it connects.
5. **Add the legend and caption last.**

Do not hand the JSON to a coding agent: it lacks this skill's rules, and the handoff costs more than it saves. Do not write a Python generator script either: it adds a layer to debug. Hand-written JSON with descriptive ids is easier to fix.

## JSON Structure

```json
{
  "type": "excalidraw",
  "version": 2,
  "source": "https://excalidraw.com",
  "elements": [...],
  "appState": {
    "viewBackgroundColor": "#ffffff",
    "gridSize": 20
  },
  "files": {}
}
```

[element-templates.md](element-templates.md) has tested JSON for every shape in the vocabulary, plus the binding rules. Pull colors from [color-palette.md](color-palette.md) by each element's category.
