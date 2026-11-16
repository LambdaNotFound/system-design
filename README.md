# system-design

System-design learning notes: design docs, concept notes, and small Rust implementations of the interesting parts. Written with Claude Code as coach and co-author (see [CLAUDE.md](CLAUDE.md)).

## Designs

| System | Status | Started | Code |
|---|---|---|---|
| [URL Shortener](design-doc/url-shortener/rfc.md) | Reviewed | 11/15/26 | — |

**Status:** Scoping → Scoped (requirements settled) → Drafting (the design loop is running) → Reviewed (the bar-raiser approved, or you accepted what it left open) → Implemented (core mechanism built and tested)

Building blocks and theory live in **[concepts/](concepts/)**.

## Layout

```
.claude/skills/   Claude Code skills for the workflow below
.githooks/        commit message hooks, a pre-commit lint for the skills, and their sync to ~/.claude
concepts/         notes on building blocks and theory, linked from the designs
design-doc/       one folder per system: requirement.md, design.md, review.md, rfc.md + diagrams/
evals/            tests for the skills: fixtures and routing cases (see evals/README.md)
scripts/          check-skills lints the skills; eval-skills runs their eval cases; sync-skills installs them
src/              Rust workspace, one crate per system or concept
```

## Workflow

| Step | In Claude Code | What happens |
|---|---|---|
| Scope | `/scope-problem <system>` | Claude interviews you, then writes the requirement doc you sign off |
| Design | `/new-design <system>` | An agent loop writes a formal RFC from the requirement doc: an investigator drafts, a bar-raiser challenges it in rounds, an auditor tightens the prose |
| Review | `/review-design <system or path> [focus: …]` | On demand: a fresh bar-raiser critiques the design, or any design doc, with an optional focus area |
| Build | `/implement-design <system>` | An agent loop builds the core mechanism as a Rust crate: a coding expert plans it and you approve the plan, it writes failing tests that a bar-raiser reviews, then the logic, reviewed again. You read the result by hand |
| Draw | `/draw-diagram <what to draw, system or path> [format: …] [out: …]` | On demand: Claude writes a diagram as Excalidraw JSON or a Mermaid block, from a prompt, a design or a concept note. It renders nothing and never edits a design; you open the file and export the picture |

The skills have their own tests: `scripts/check-skills` lints them and `scripts/eval-skills` runs their eval cases. See [evals/](evals/).

Architecture diagrams are drawn by hand in Excalidraw, after the design is approved; sequence and flow diagrams are Mermaid inside the docs. See [design-doc/](design-doc/).

## Setup (once per clone)

```sh
git config core.hooksPath .githooks   # turn on the hooks
scripts/sync-skills                   # install the skills for every project; commits on main keep them in step
```

- **Rust:** install with [rustup](https://rustup.rs).
- **VS Code:** accept the recommended extensions (Excalidraw, rust-analyzer, Mermaid preview) when prompted.

## Commit messages

```
MM/DD/YY: <slug>: <what changed>      first line, max 72 chars; "<slug>: " is optional

Co-Authored-By: Name <email>          optional: a blank line, then only Key: value lines
```

Leave the date off and the hook fills in today's: `git commit -m "url-shortener: add base62 encoder"`.
