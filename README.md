# system-design

System-design learning notes: design docs, concept notes, and small Rust implementations of the interesting parts. Written with Claude Code as coach and co-author (see [CLAUDE.md](CLAUDE.md)).

## Designs

| System | Status | Started | Code |
|---|---|---|---|

_None yet. Start one in Claude Code with `/new-design <system>`._

**Status:** Drafting → Reviewed (no blocking review findings left) → Implemented (core mechanism built and tested)

Building blocks and theory live in **[concepts/](concepts/)**.

## Layout

```
.claude/skills/   Claude Code skills for the workflow below
.githooks/        commit message hooks
concepts/         notes on building blocks and theory, linked from the designs
design-doc/       one folder per system: README.md (the doc) + diagrams/
src/              Rust workspace, one crate per system or concept
```

## Workflow

| Step | In Claude Code | What happens |
|---|---|---|
| Design | `/new-design <system>` | Claude coaches you through each section, then writes it up |
| Review | `/review-design <system>` | Staff-level critique, logged in the doc's Review log |
| Build | `/implement-design <system>` | Rust skeleton and failing tests; you write the `todo!()`s |

Architecture diagrams are drawn by hand in Excalidraw; sequence and flow diagrams are Mermaid inside the docs. See [design-doc/](design-doc/).

## Setup (once per clone)

```sh
git config core.hooksPath .githooks   # turn on the commit message hooks
```

- **Rust:** install with [rustup](https://rustup.rs).
- **VS Code:** accept the recommended extensions (Excalidraw, rust-analyzer, Mermaid preview) when prompted.

## Commit messages

```
MM/DD/YY: <slug>: <what changed>      first line, max 72 chars; "<slug>: " is optional

Co-Authored-By: Name <email>          optional: a blank line, then only Key: value lines
```

Leave the date off and the hook fills in today's: `git commit -m "url-shortener: add base62 encoder"`.
