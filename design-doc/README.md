# Design docs

One folder per system, named by its slug:

```
design-doc/url-shortener/
├── README.md                         the design doc
└── diagrams/
    └── architecture.excalidraw.svg   drawn in Excalidraw
```

Start one in Claude Code with `/new-design <system>`. It creates the folder from the [template](../.claude/skills/new-design/template.md) and adds the system to the [index](../README.md#designs).

## Drawing in Excalidraw

1. Install the recommended VS Code extensions (VS Code offers them when you open the repo).
2. Create an empty `diagrams/<name>.excalidraw.svg` and open it. VS Code opens it in Excalidraw.
3. Draw and save. That one file is both the editable drawing and the image GitHub shows, so there's nothing to export.

The background is kept solid white (`.vscode/settings.json`). Don't turn it off: with a transparent background, the black lines disappear in GitHub's dark mode.

Sequence and flow diagrams aren't drawn here. They're written as Mermaid inside the doc.
