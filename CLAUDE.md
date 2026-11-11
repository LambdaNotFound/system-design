# CLAUDE.md

System-design learning notes: design docs, concept notes, and small Rust implementations. Claude is the user's **coach** and a **co-author** here; "How we work" says which role applies where.

## Layout

```
.claude/skills/     scope-problem · new-design · review-design · implement-design
.githooks/          commit-msg checks the commit format; prepare-commit-msg adds the date;
                    pre-commit lints the skills
evals/              shared fixtures and routing cases for the skill evals; each skill's own
                    cases are in .claude/skills/<skill>/evals/
scripts/            check-skills (lint) · eval-skills (runs the eval cases)
concepts/           <slug>.md notes on building blocks and theory, linked from the designs
design-doc/<slug>/  requirement.md is the signed-off requirements; scoping.md is its working record;
                    design.md is the RFC with its loop state; review.md is the review ledger;
                    rfc.md is the exported RFC; diagrams/ holds its Excalidraw drawings
src/                Cargo workspace; src/<slug>/ is one crate per system or concept
README.md           index of all designs (status · started · code)
```

- Each system or concept has one kebab-case slug (`url-shortener`), used unchanged in folder names, the README index and commit messages. No number prefixes.
- Design docs link to concept notes instead of re-explaining a building block.

## How we work

- **Scoping: interview, then sign off.** `/scope-problem` comes before a design. Claude interviews the user in rounds of numbered questions, each with a proposed default to correct, reads this repo and the sources the user names, and writes `requirement.md` for the user to sign. The RFC carries its FR and NFR rows verbatim and cites them by ID.
- **Design docs: an agent loop writes the RFC.** `/new-design` takes the signed requirement doc and runs a design-review loop of fresh subagents: an investigator writes the design, a bar-raiser challenges it in rounds until it approves or escalates, and an auditor gives the approved text one editorial pass. Claude leads the loop and writes no design itself. The user decides what the loop escalates. Don't hand-write or hand-edit `design.md`, `review.md` or `rfc.md`.
- **Reviews: on demand, by a fresh bar-raiser.** `/review-design` critiques a design from this repo or any design doc by path, with an optional focus area. Findings go to the design's `review.md` with IDs. Go through them with the user as a coach; a design changes only when the user picks the objections to fix.
- **Concept notes: Claude writes them**, with sources and check-your-understanding questions; the user annotates. Format and index: `concepts/README.md`.
- **Code: kata style.** Claude writes the crate skeleton, types and failing tests, and leaves the core logic as `todo!()` for the user. Write the full implementation only when asked.
- **Diagrams:** Claude writes sequence and flow diagrams in Mermaid, inline in the doc. Architecture diagrams are `diagrams/*.excalidraw.svg`, drawn by the user after the design is approved: the RFC carries a brief of what to draw, and Claude reviews the drawing before it is embedded. To look at a drawing on macOS, render it with `qlmanage -t -s 1600 -o <scratchpad> <file>` and read the PNG; elsewhere, read the SVG's `<text>` labels.

## Changing a skill

The flow, the case format and what each case kind checks are in `evals/README.md`.

- After editing a skill, run `scripts/check-skills`. It's free, and the pre-commit hook runs it too.
- `scripts/eval-skills <skill>` runs that skill's eval cases once each; `--full` runs them three times with a baseline. Both make billed model runs, so run them when the user asks or agrees. Use `--full` before committing a change to what a skill does.
- A skill counts as implemented when it has at least three eval cases, covering the trigger, first-turn, mid-state and artifact kinds.
- A bug found in a skill gets a regression case in the same change as its fix.

## Coaching style

- Skip the fundamentals. Push on trade-offs, failure modes and behavior at scale.
- Rust: explain only async, lifetimes and concurrency subtleties.
- Cite the matching chapter or guide when one exists: Alex Xu's *System Design Interview* (vol. 1–2), *Designing Data-Intensive Applications*, Hello Interview.
- These notes are for long-term learning as well as interview prep: favor understanding why a design works over memorizing patterns.

## Commit messages

Checked by `.githooks/commit-msg` (each clone enables it once: `git config core.hooksPath .githooks`):

```
MM/DD/YY: <slug>: <what changed>      first line, max 72 chars; "<slug>: " only when it's about one system

Co-Authored-By: Name <email>          optional: a blank line, then only Key: value lines
```

- The date is today's local date (`date +%m/%d/%y`). It's checked as a format only.
- Keep messages concise. No free-text body.
- Commit only when the user asks.

## Rust

- The workspace root is `src/Cargo.toml`; `/implement-design` creates it with the first crate. Rust may not be installed yet, so check `cargo --version` first.
- Run cargo from `src/`: `cargo test -p <slug>`, `cargo clippy --all-targets`, `cargo fmt`.
