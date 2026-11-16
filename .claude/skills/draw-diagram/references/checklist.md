# Quality Checklist

Run through this before you hand a diagram over. The section names are those of `SKILL.md`.

## Purpose
1. **Question:** The title states what the diagram shows.
2. **Kind:** The layout follows the recipe for its kind, with that kind's must-haves.
3. **Format:** Excalidraw or Mermaid, picked by the rules in "Pick the format".
4. **Concrete:** Real names, ports, routes, and payload examples appear where they matter.

## Structure
5. **Boundaries:** Components are grouped by real boundaries, each labeled, at most 3 levels deep.
6. **Consistency:** Same kind of thing has the same shape, size, and color.
7. **Color:** Color encodes one dimension, with at most 5 category colors; the rest is neutral.
8. **Outside:** External systems are gray and sit at the edge.
9. **Multiplicity:** Many instances are shown with a stack or "×N".

## Connections
10. **Labels:** Every arrow says what moves.
11. **Line style:** Solid, dashed, and thick each mean one thing, applied everywhere.
12. **Steps:** Flows are numbered in reading order; parallel branches use letters.
13. **Traceable:** One main direction and few crossing lines.

## Text and Legend
14. **Hierarchy:** Title, names, and captions differ in size and color.
15. **Legend:** Present when color or line style means something the labels do not say, and it covers every code used.
16. **Text clean:** Labels contain only readable words.

## Excalidraw
17. **Fonts:** Labels use `fontFamily: 9`; literals use `fontFamily: 3`.
18. **Roughness:** `roughness: 0` unless sketch style was requested.
19. **Opacity:** `opacity: 100` for all elements.
20. **Script:** The check script reports no errors, and every warning is fixed or explained.
21. **Read-through:** The checks the script cannot make, listed in `SKILL.md` under "Check before you deliver", pass.

## Mermaid
22. **Syntax:** No line matches the "Syntax that breaks" list.
23. **Ids and classes:** Every edge, `class`, and `linkStyle` refers to something that exists.
24. **Title and legend:** They sit outside the block, in Markdown.
