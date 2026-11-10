---
name: new-design
description: Start or resume a system design in this repo. Scaffolds the system's folder under design-doc/ from the template and adds it to the README index, then coaches the user through it one section at a time and writes up each section from their decisions. Use when the user wants to design a system, e.g. "let's design a URL shortener".
argument-hint: "[system name]"
---

# New design

The system to design is in the skill's arguments (the `ARGUMENTS:` line) or the conversation. If neither names one, ask.

## 1. Set up the doc

1. Choose the slug: kebab-case, no numbers (`URL Shortener` → `url-shortener`). Check it with the user if it isn't obvious.
2. Read `design-doc/<slug>/requirement.md` if it exists. `/scope-problem` writes it, to [this spec](../scope-problem/requirement-spec.md). Go by its header lines:
   - `Status: signed-off` and `Route: design`: the requirements are settled. Don't ask for them again.
   - `Route: decomposed`: the designs belong to the pieces. List the piece slugs from the Decomposition table in `scoping.md` and ask which one to start.
   - `Route: code` or `no code`: this piece needs no design doc. Point the user to its handoff instead.
   - `Status: draft`, `Status: closed`, or no file: suggest `/scope-problem <slug>` first. If the user wants to go ahead anyway, do, and say that the requirements are informal.
3. If `design-doc/<slug>/README.md` exists, don't overwrite it. Read it, summarize where it stands, and resume at the first unfinished section.
4. Otherwise:
   - Create `design-doc/<slug>/README.md` from [template.md](template.md). Fill in the system name and today's date (`date +%m/%d/%y`).
   - Create the empty folder `design-doc/<slug>/diagrams/`.
   - In the Designs table in the root `README.md`, set the system's row to `Drafting`. If `/scope-problem` didn't add one, add it, and remove the "None yet" line if it's there:
     `| [System Name](design-doc/<slug>/) | Drafting | MM/DD/YY | — |`

## 2. Coach, then write up

Work through the sections in order. For each one:

1. **Ask first.** One to three pointed questions that make the user commit to a position. Skip the basics and go to what's actually hard about this system.
2. **Push back** when an answer is vague or risky: ask for the number, the failure scenario, or the alternative they rejected. One or two rounds; it's a design review, not an interrogation.
3. **Write it up** once the user has decided: their decisions and reasoning, in clean prose and tables. If they missed something important, add it and point it out. Delete the section's guidance comment.
4. **Move on.** If the user says "just write it" (for one section or the whole doc), author it directly, then ask them to challenge what you wrote.

Section-specific:

- **Requirements:** with a signed-off requirement doc, section 1 is one line, `Signed off MM/DD/YY: [requirement.md](requirement.md)`, and its subsections go. Later sections cite the rows they serve by ID (`FR2`, `NFR1`). If a decision shows that a requirement is wrong or missing, don't edit the signed doc: tell the user, and on their instruction append a dated `## Amendment` to it, as the spec says. Without a signed-off doc, coach section 1 like any other.
- **Estimates:** the user states the assumptions; you can do the arithmetic. Keep only numbers that drive a decision.
- **High-level design:** ask the user to draw `diagrams/architecture.excalidraw.svg`. Put a short brief in the section first: the boxes, the arrows and what flows over them, and the labels that matter. When the file exists, replace the brief with `![Architecture](diagrams/architecture.excalidraw.svg)` and review the drawing against the design.
- **Request flows:** write these yourself, as Mermaid sequence diagrams.
- **Building blocks:** when the design relies on one (consistent hashing, leader election, LSM trees, …), link `../../concepts/<slug>.md`. If the note doesn't exist yet, offer to write it (format in `concepts/README.md`).

## 3. Finish

When every section is written, write the summary at the top of the doc and suggest `/review-design <slug>`. Don't commit unless the user asks.
