# Design docs

One folder per system, named by its slug:

```
design-doc/url-shortener/
├── requirement.md                    the signed-off requirements
├── scoping.md                        the working record behind them
├── design.md                         the RFC, with the design loop's state below its marker
├── review.md                         the review ledger: loop rounds and on-demand reviews
├── loop.md                           your decisions during the loop, when there were any
├── rfc.md                            the exported RFC: the one to read and share
└── diagrams/
    └── architecture.excalidraw.svg   drawn in Excalidraw
```

Start one in Claude Code with `/scope-problem <system>`. It creates the folder, interviews you, and writes `requirement.md` to this [spec](../.claude/skills/scope-problem/requirement-spec.md), and adds the system to the [index](../README.md#designs). Then `/new-design <system>` runs the design loop on it and writes the RFC to this [spec](../.claude/skills/new-design/rfc-spec.md); an [exemplar](../.claude/skills/new-design/rfc-exemplar.md) shows every section. `/review-design <system>` adds an on-demand critique to `review.md`.

A third file, `investigation.md`, sits beside `scoping.md` only when the requirements were drafted from an existing system's code.

A problem that splits into several pieces gets one folder per piece, named `<slug>-<piece>`. Each piece's `requirement.md` names its parent on a `Parent:` line.

## Drawing in Excalidraw

Draw once the design is approved: the RFC's Overview carries a line starting `*Diagram:` that says what to show.

1. Install the recommended VS Code extensions (VS Code offers them when you open the repo).
2. Create an empty `diagrams/<name>.excalidraw.svg` and open it. VS Code opens it in Excalidraw.
3. Draw and save. That one file is both the editable drawing and the image GitHub shows, so there's nothing to export.
4. Save the architecture drawing as `diagrams/architecture.excalidraw.svg` and run `/new-design <system>` again. Claude reviews it against the design and has it embedded in the RFC.

The background is kept solid white (`.vscode/settings.json`). Don't turn it off: with a transparent background, the black lines disappear in GitHub's dark mode.

Sequence and flow diagrams aren't drawn here. They're written as Mermaid inside the doc.
