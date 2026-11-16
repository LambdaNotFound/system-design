---
name: draw-diagram
description: Draws a software engineering diagram as an Excalidraw file (.excalidraw JSON) or as a Mermaid block in Markdown, from a description in the prompt, a design under design-doc/, a concept note, or any doc by path. Covers system architectures, request and message flows, data pipelines, sequences, layered stacks, state machines, comparisons and concepts, in the style of the curated diagrams in terrastruct/awesome-diagrams. It writes the source only and renders no image; a stdlib script checks an Excalidraw file's ids, bindings, text fit and layout. It never edits a design doc or an RFC: a diagram of a design is written beside it, under diagrams/. Use when the user wants to draw, diagram or visualize a system, flow or concept, or asks for an Excalidraw or Mermaid diagram. Not for writing or changing a design (use new-design), critiquing one (use review-design), slide decks, or charts and plots of data.
argument-hint: "[what to draw, or a slug or a path to a doc] [format: excalidraw or mermaid] [out: path]"
---

# Draw diagram

Write a diagram as source the user opens: an `.excalidraw` file, or a Mermaid block in Markdown. Both formats follow one style: real boundaries, one consistent shape vocabulary, labeled arrows, numbered steps, and color that means one thing.

The style comes from studying all 79 diagrams in [terrastruct/awesome-diagrams](https://github.com/terrastruct/awesome-diagrams). **Read [references/style-guide.md](references/style-guide.md) before you design.** It holds the full rules, with the repo diagrams that show each one.

**This skill renders nothing.** It needs no browser, package, or network to write a diagram. The user opens the result and exports a PNG or SVG. You will not see the picture, so the checks in "Check before you deliver" stand in for looking at it.

What to draw, the format and the output path are in the skill's arguments (the `ARGUMENTS:` line) or the conversation. If nothing says what to draw, ask.

**If this file was cut short.** A long session can be compacted, and that keeps only the start of this file. If you cannot see the "Hand it over" section at its end, read this file again from disk before you go on.

## What to draw from, and where it goes

Take the first row that matches. A path the user names with `out:` outranks the last column.

| The user gave | Draw from | Write to |
| --- | --- | --- |
| A slug whose `design-doc/<slug>/` holds `rfc.md` or `design.md` | `rfc.md`; without one, `design.md` down to its loop-state marker | `design-doc/<slug>/diagrams/` |
| A slug with a note at `concepts/<slug>.md` | the note | Mermaid: inline in the note, under How it works. Excalidraw: `concepts/<slug>/diagrams/` |
| A file inside this repository | that file | a `diagrams/` folder beside it |
| A file outside this repository | that file | the path the user names, else a fresh `mktemp -d` directory |
| A description, and no doc | the description | the current directory |

Name the file for what it shows, in kebab-case: `request-path.excalidraw`, or `request-path.md` for Mermaid.

- **A design has its own writers.** Never write `design.md`, `rfc.md`, `review.md` or `requirement.md`: a diagram of a design goes in its `diagrams/` folder, Mermaid included. Never write `diagrams/architecture.excalidraw.svg` either. That file is the user's own drawing, and `/new-design` embeds it.
- **The `*Diagram:` line.** A design that `/new-design` wrote carries one line in its Overview that starts `*Diagram:` and says what the architecture drawing must show. When the user asks for the architecture, or names no diagram, draw what that line says.
- **No design yet.** A slug whose folder holds only `requirement.md` has nothing to draw. Say so, point the user to `/new-design <slug>`, and stop.
- **An open loop.** A `design.md` with no `rfc.md` beside it is still in review and may change. Say so when you hand the diagram over.
- **Use the doc's own names** for its components, stores and flows, so the diagram and the doc can be read side by side.
- This repository is public. Write repository-relative paths into any file you write, and write nothing about a doc from outside it into the repository unless the user says where.
- Commit only when the user asks.

## Pick the format

| Format | Pick it when | Deliver |
|--------|--------------|---------|
| Excalidraw | The user asks for Excalidraw, or for a picture to edit by hand. Or the diagram needs what Mermaid cannot place: exact positions, nested boundaries with labels at the top-left, step badges, stacked replicas, code blocks, or a hand-tuned layout. | `<name>.excalidraw` |
| Mermaid | The user asks for Mermaid or Markdown. Or the diagram goes into a Markdown file (README, concept note, PR description, notes) and should diff as text. Mermaid lays out the nodes itself. | A `mermaid` fenced block in the target file, or in `<name>.md` |

If the request does not say, ask one question and recommend by destination: Mermaid for a Markdown doc, Excalidraw for anything else. Write nothing until it is answered.

## Reference files

| File | Format | Read it when |
|------|--------|--------------|
| [references/style-guide.md](references/style-guide.md) | Both | Designing: diagram kinds, rules, and repo examples |
| [references/color-palette.md](references/color-palette.md) | Both | Choosing any color or line style |
| [references/mermaid.md](references/mermaid.md) | Mermaid | Writing Mermaid: types, shapes, styles, and the syntax that breaks |
| [references/excalidraw.md](references/excalidraw.md) | Excalidraw | Before writing any JSON: the shape vocabulary, text, spacing, style modes, and how to build a large file |
| [references/element-templates.md](references/element-templates.md) | Excalidraw | Writing JSON: tested templates for every shape |
| [references/json-schema.md](references/json-schema.md) | Excalidraw | Looking up a field, font, fill, or arrowhead value |
| [references/checklist.md](references/checklist.md) | Both | Before you hand the diagram over |

If one is missing, stop and tell the user where it should be.

**All colors live in one file:** [references/color-palette.md](references/color-palette.md). Read it before any diagram and take every color from it. To match a brand, edit that file; the rest of this skill refers to color roles, not hex values.

## Core rules

These hold for every diagram in both formats. The style guide explains each one with examples. Mermaid cannot do all of them directly; [references/mermaid.md](references/mermaid.md) gives the substitute for each.

1. **Answer one question.** Decide what the reader should learn, for example "what happens when a user sends a message". Put it in the title. Leave out anything that does not help answer it.
2. **Pick the diagram kind first.** Each kind has its own layout (see the table below).
3. **Draw real boundaries.** Group components by where they run or who owns them: cluster, region, VPC, process, team, or trust zone. Label each boundary.
4. **Same kind, same look.** All databases are cylinders and all services are the same box. Color encodes exactly one thing: role, owner, layer, or flow. Use 2 to 5 colors and keep everything else neutral.
5. **Label every arrow** with what moves: an action ("publish order"), a payload ("chunks"), a protocol ("gRPC :50051"), or a tool ("Debezium CDC").
6. **Number the steps of a flow.** In a request flow, each arrow carries its step number. In a pipeline where each step is a box, number the boxes instead. Use letters for parallel branches (5B, 5C).
7. **Use line style for one more meaning.** Solid means request or data, dashed means async, control, or config, and thick means the main path. Define any other meaning in a legend.
8. **Show many with one.** Stack 2 offset copies behind a box, or write "×N". Do not draw 10 boxes.
9. **Push the outside to the edge.** External and out-of-scope systems go at the edge, in gray.
10. **Be concrete.** Use real names, ports, routes, topics, and payload examples. Show a code or JSON block when the format matters.
11. **Keep it traceable.** Use one main direction and avoid crossing lines. If lines cross a lot, re-layout or split the diagram.

## Diagram kinds

| Kind | Answers | Layout | Must have | Mermaid type |
|------|---------|--------|-----------|--------------|
| System architecture | What runs where, and what talks to what? | Nested boundaries; clients left or top, stores right or bottom | Labeled boundaries, role colors, protocol labels | `flowchart` with subgraphs |
| Request or message flow | What happens, in order, when X occurs? | Architecture skeleton plus numbered steps | Step numbers, action labels, a color per actor if 2+ | `sequenceDiagram`, or `flowchart` with numbered labels |
| Data pipeline | How does data move and change? | One direction, usually left to right | Payload or tool on every arrow | `flowchart` |
| Sequence | In what order do a few actors exchange messages? | Lifelines; time flows down | Numbered messages, a label on each, legend if colored | `sequenceDiagram` |
| Layered stack | What is built on what? | Horizontal bands, top to bottom | Color per layer, band titles | `flowchart TB`, a subgraph per layer |
| State machine or workflow | What states exist, and how do they change? | Main path in one row, side states below | Main path filled, conditions on transitions | `stateDiagram-v2` |
| Comparison or before/after | What is the difference? | Same canvas twice, or a grid of small panels | Change only what differs; highlight it in one color | `flowchart LR`, a subgraph per side |
| Concept or algorithm | How does an idea work? | A structure that mirrors the idea | Concrete example values | `flowchart` |

Style guide section 1 has the full recipe and repo examples for each kind.

## Design process

Do this before writing any source.

### Step 0: Research (technical diagrams)

When you draw from a doc, read all of it first: its components, names and flows are the diagram's content.

If you diagram a real protocol, API, or system, look up the actual specs first: real component names, event names, endpoints, ports, and payload formats. Concrete details are what make a diagram teach.

Bad: "Protocol" → "Frontend"
Good: "AG-UI streams events (RUN_STARTED, STATE_DELTA)" → "CopilotKit renders via createA2UIMessageRenderer()"

### Step 1: Write the question, then pick the kind and the format

Write one sentence: "This diagram shows ___ for ___." It becomes the title or the caption. Then pick the kind from the table above, and the format from "Pick the format".

Choose the depth for the audience. The repo has good diagrams with 6 shapes (Lambda architecture) and with 45 or more (Lichess). Dense diagrams need boundaries, a legend, and labels to stay readable. A Mermaid diagram gets hard to follow sooner, because you cannot place its nodes; split it before it sprawls.

### Step 2: List the parts

- Components, grouped by boundary.
- Connections: from, to, label, line style, and step number.
- The one dimension that color encodes, and its 2 to 5 categories.
- What is external or out of scope (gray).
- Concrete details to show: ports, routes, payload samples, counts.

### Step 3: Plan the layout

- **Excalidraw:** pick the main direction and a 20 px grid. Place boundaries first, then components inside them. Estimate the size and keep the long side at most about 2000 px; if it must be bigger, plan a series (style guide section 13). Place the title (top-left), the legend (a corner), and the caption (bottom).
- **Mermaid:** pick a flowchart's direction by its longest chain of edges: `LR` for at most 4 nodes, `TB` for more. In `LR` each labeled hop adds about 250 px, and the page shrinks a wide diagram, text included ([references/mermaid.md](references/mermaid.md) section 2). Mermaid places nodes itself; you control the direction, the subgraphs, and which way each edge points. Point edges the way the reader should read, for example from clients toward stores.

### Step 4: Write the source

- **Excalidraw:** read [references/excalidraw.md](references/excalidraw.md), then copy shapes from [references/element-templates.md](references/element-templates.md). Build a large diagram one section at a time, as that file's "Large Diagrams" section says. Write the JSON yourself: do not hand it to a subagent, which lacks this skill's rules, and do not write a generator script, which adds a layer to debug.
- **Mermaid:** follow [references/mermaid.md](references/mermaid.md): its type mapping, shapes, edges, and class styles. Put the title in a Markdown heading or sentence above the block, and the caption and legend in a short list below it. Show a payload or command as its own fenced code block next to the diagram, not inside a node.

### Step 5: Check

Run the checks in the next section, fix what they find, and check again. This step is never skipped.

## Check before you deliver

### Excalidraw

The check script needs only `python3`:

```bash
python3 ${CLAUDE_SKILL_DIR}/scripts/check_excalidraw.py <path-to-file.excalidraw>
```

- **Errors** (exit 1): the file is not Excalidraw JSON, an id repeats, a binding is listed on one end only, or text bound to a shape sits outside it or is too tall. Fix every one and run it again.
- **Warnings** (exit 0): a line wider than its shape leaves, text bound to a shape more than 6 px off its center, or an arrow label with too little arrow under it or on a sharp corner. Then the layout, read from the coordinates and the fonts' real widths:
  - two shapes overlap and neither holds the other;
  - an arrow end sits more than 12 px from the edge of the shape it binds;
  - an arrow passes through a shape that is not its end, or through free text or another arrow's label;
  - text runs across a shape's outline, or over other text;
  - a step badge on an arrow sits more than 8 px off its spot, 24 px from the start;
  - a shape starts less than 44 px below the top of a boundary that has a label.

  Each warning names the ids and the fix. Fix each, or tell the user why it stands.
- **Bounds:** it prints `Diagram bounds: x=… y=… w=… h=…`. If `w` or `h` is over about 2000, split the diagram or plan a series.

Do not write your own coordinate checks; the script makes them. It cannot judge these, so read the JSON for them:

- Same-kind components share one size and one color.
- Few arrows cross each other. The script checks arrows against shapes and text, not against other arrows.
- A curved arrow bulges past the straight lines between its points, and the script checks only those lines. Leave room around the curve.

### Mermaid

Read the block line by line against the "Syntax that breaks" list in [references/mermaid.md](references/mermaid.md). Then check:

- In a `flowchart LR`, the longest chain of edges has at most 4 nodes.
- Every edge names an id you declared. Mermaid silently draws a new node for a misspelled id.
- Every class used in `:::` or `class` has a `classDef`, and every `classDef` sets `fill`, `stroke`, and `color`.
- Each `linkStyle` index counts edges from 0, in the order they are written.
- Subgraph ids differ from node ids.

### Both

Go through [references/checklist.md](references/checklist.md) once.

## Hand it over

Give the path, and say in one sentence what the diagram shows.

- **Excalidraw:** the user opens the file in VS Code with the Excalidraw extension this repository recommends, or at excalidraw.com, then exports a PNG or SVG.
- **Mermaid:** GitHub renders the block in Markdown files, issues, PRs, and wikis, and VS Code's Markdown preview does with the Mermaid extension. Excalidraw's Mermaid to Excalidraw tool turns a flowchart, sequence, or class diagram into editable shapes; depending on the Excalidraw version, other types arrive as an image.
- **From a design:** say which file you drew from. The drawing the RFC embeds is still the user's own, `diagrams/architecture.excalidraw.svg`; yours is a separate file they can start from.

Say that you did not see a render, and name every warning you left standing and why. Ask the user to send back anything that looks wrong when it opens.
