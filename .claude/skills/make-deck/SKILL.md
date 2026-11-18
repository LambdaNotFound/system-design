---
name: make-deck
description: Makes a slide deck for a software engineering talk, as Markdown source and a 16:9 PDF, from a design under design-doc/ (its exported RFC), a concept note, or any doc by path. It plans a storyline for the audience and the time, writes one claim per slide with the doc's own numbers and requirement IDs, reuses the doc's Mermaid diagrams and the architecture drawing, and keeps the detail in speaker notes and backup slides. The design language is minimal on purpose. A stdlib script checks the deck against its source, so it states nothing the doc does not, and a second script prints the PDF with a local Chrome and reports any slide whose content overflows. It never edits the source doc. Use when the user wants slides, a deck or a presentation from a design, an RFC, a concept note or another doc, for a design review, a team walkthrough or an interview. Not for writing or changing a design (use new-design), drawing a single diagram (use draw-diagram), or a PowerPoint file.
argument-hint: "[slug, or a path to a doc] [minutes: n] [audience: review, team or interview] [out: folder]"
---

# Make deck

Turn a doc into a talk: a Markdown deck the user can edit and diff, and the PDF they present. The look is plain on purpose: white slides, one sans-serif face, a title rule, page numbers. The work goes into what each slide says.

What to present, the time and the audience are in the skill's arguments (the `ARGUMENTS:` line) or the conversation. Defaults: 20 minutes, a design review. If nothing names what to present, list the designs in `design-doc/` that have an `rfc.md`, ask which one, and write nothing until it is answered.

**Read [references/deck-spec.md](references/deck-spec.md) before you write a slide.** It holds the deck format, the storyline for each audience, and the slide rules the check script enforces. If it is missing, stop and tell the user where it should be.

**If this file was cut short.** A long session can be compacted, and that keeps only the start of this file. If you cannot see the "Hand it over" section at its end, read this file again from disk before you go on.

## What to present from, and where it goes

Take the first row that matches. A folder the user names with `out:` outranks the last column.

| The user gave | Present from | Write to |
| --- | --- | --- |
| A slug whose `design-doc/<slug>/` holds `rfc.md` | `rfc.md` | `design-doc/<slug>/deck/` |
| A slug whose folder holds `design.md` and no `rfc.md` | `design.md` down to its loop-state marker | `design-doc/<slug>/deck/` |
| A slug with a note at `concepts/<slug>.md` | the note | `concepts/<slug>/deck/` |
| A file inside this repository | that file | a `deck/` folder beside it |
| A file outside this repository | that file | the folder the user names, else a fresh `mktemp -d` directory |

The deck is `deck.md` in that folder, and the PDF prints beside it as `deck.pdf`.

- **No design yet.** A slug whose folder holds only `requirement.md` has nothing to present. Say so, point the user to `/new-design <slug>`, and stop.
- **An open loop.** A `design.md` with no `rfc.md` beside it is still in review and may change. Say so when you hand the deck over.
- **The source belongs to its writers.** Never edit the doc you present from, and never write `design.md`, `rfc.md`, `review.md` or `requirement.md`. A doc that is wrong is the user's to fix, through `/review-design` for a design.
- **A deck exists.** If `deck.md` is already there, read it first and revise it in place rather than starting over: the user may have edited it. Say what you changed.
- This repository is public. Write repository-relative paths into the deck, and write nothing about a doc from outside it into the repository unless the user says where.
- Commit only when the user asks. `deck.pdf` is a build output: suggest committing `deck.md` and leaving the PDF out unless the user wants it in git.

## Steps

1. **Read the source in full,** and for a design, the last section of `review.md` too: an `approve-with-risks` verdict's residual risks belong on the risk slide. Read the architecture drawing's path if the RFC embeds one.
2. **Plan the storyline** from deck-spec's outline for the audience, inside the slide budget for the time. Write the slide titles first, as claims, and read them in order on their own: they should tell the talk. Cut any slide the talk survives without, or move it to Backup.
3. **Write `deck.md`** to deck-spec's format: front matter, a title slide, one `## ` claim per slide, the doc's own numbers and IDs, and speaker notes that name the section each slide comes from.
4. **Check it.** Run `python3 ${CLAUDE_SKILL_DIR}/scripts/check_deck.py <deck.md>` and fix every error, then run it again, until its last line reads `0 errors`. Never edit the source to make a number pass: a number the source doesn't have comes out of the deck.
5. **Render it.** Run `python3 ${CLAUDE_SKILL_DIR}/scripts/render_deck.py <deck.md> --notes`. It finds a local Chrome or Chromium, or takes one from `CHROME_PATH`. Exit 1 names the slides whose content overflows: cut or split them, check again, and render again. Exit 2 means no PDF: with no browser, say so, give the command, and hand over the Markdown.
6. **Look at it.** When `pdftoppm` is available, turn the pages into images (`pdftoppm -r 50 -png deck.pdf <scratch dir>/page`) and read them: a clean exit does not prove a slide reads well. Fix what looks wrong. Without it, say you did not see the slides.

## Hand it over

Give the paths of `deck.md`, `deck.pdf` and `deck.notes.pdf`, the slide count against the budget, and the one-line storyline: the claims of the first and the last main slide. Then:

- **Diagrams.** Name each Mermaid diagram that printed as source because `mmdc` (mermaid-cli) was missing, and say that installing it and rendering again draws them.
- **What you left out.** Name the doc sections with no slide, so the user can ask for one.
- **What you saw.** Say whether you looked at the rendered pages, and name every warning from either script you left standing, and why.
- **Editing.** The user edits `deck.md` and renders again with the same command; the check script keeps the edits honest against the source.
