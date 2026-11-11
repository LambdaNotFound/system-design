# Phases 5 and 6

## Contents

- Phase 5: Test the requirement doc on a reader who was not here
- Phase 6: Resolve conflicts, tighten, humanize, sign off (seven steps)
- Prose voice

Read at the start of phase 5. "The doc" below means each doc that will be handed off: the only doc when there is one piece, and every piece doc when there are several. The parent of a decomposed problem is never handed off, so it gets the conflict sweep and the stamp but no reader test.

## Phase 5: Test the requirement doc on a reader who was not here

The doc exists to be executed by an agent with none of this conversation, so the only test that means anything is a context-free read. Run it without involving the user:

1. Predict what an agent downstream would need to ask to start work: five to ten concrete questions.
2. Dispatch the reader with prompt C, sending only the doc's path and those questions. It must be a fresh agent, never a fork: a reader that saw the interview tests nothing. If it runs long, message it to stop reading and report from what it has.
3. Then send prompt C's follow-up, which asks separately for contradictions, ambiguities, and unstated assumptions.

**The gap between what comes back and what the user meant is the defect list.** A reader that would build the wrong thing means the doc is wrong, however well it reads to the two of you. Fix wording and ambiguity yourself, and re-test the sections that failed. A contradiction is not yours to fix: two requirements that cannot both hold, or a requirement the code or a cited page contradicts, goes to the conflict sweep in phase 6 for the user to decide.

Tell the user what the reader got right and what it missed. Record the result under Reader test, and add `Reader test: <date>` to the record's progress lines.

## Phase 6: Resolve conflicts, tighten, humanize, sign off

1. **Conflicts.** The conflicts raised during the interview are already in the Conflicts table with their decisions. This is the sweep over the whole doc. Find every place two things cannot both hold, and put each one to the user. Check, in this order:
   - each requirement row against every other row (FR, NFR, LLD) and against the `Out of scope:` line: can one system satisfy both? Typical clashes are a durability bar against a throughput bar, a mandated mechanism that cannot deliver a required guarantee, and a requirement that needs something the boundary excludes;
   - each requirement against the Evidence: code or a cited page that contradicts it, or shared code it would force to change, named with who else depends on that code;
   - each requirement against a decision already argued on a cited page or in another design in this repo;
   - accepted defaults that pull in opposite directions, and every contradiction the reader test returned.

   Present them as one numbered list, continuing the table's numbering (`C1` onward), most consequential first. Each entry names what clashes and between which IDs, with citations; the options, usually two or three, each with what it costs; and your recommendation with its reason. Shorthand answers work here too (`C1: a, C2: b`). Never resolve a conflict yourself, even one whose answer looks obvious: a resolution the user never saw is a requirement they never signed.

   **Gaps go to the user too.** The sweep changes the doc only through the user's decisions. A gap you find (something the user said that never reached the doc, such as a constraint in Questions asked that no LLD row carries, or a coverage row whose landing place is empty) is not yours to fill. Put it in the same list as `G1` onward, with the row you would add and where it lands. If the user accepts it, add the row, point the coverage row at it, and delete `Reader test:` and every progress line after it: the phase 5 reader never saw the new row, so the reader test, the sweep and the final check run again on the new text. Record each in the Conflicts table, apply the decisions to the doc, and scan again what the decisions changed, since a decision can open a new conflict. When a scan finds nothing left undecided, add `Swept: <date>` to the progress lines.

2. **Tighten.** Check the doc against the spec's Rules, including `doc only` pieces: every table row is one testable sentence with an ID, and the Problem Statement is one short paragraph. Cut words, not requirements: a row that needs a second sentence is two requirements, or belongs in another table.

3. **Humanize** the four sections: headings, prose, and table rows, not the header lines.
   1. If the session has a humanizer skill, invoke it. Tell it the text is a neutral technical requirements doc (no added opinions, first person, or personality), that the Prose voice list below is house style to keep, and that a writing sample, if the user or the repository supplied one, calibrates the voice. With no humanizer skill, make the pass yourself for common AI writing patterns, keeping the Prose voice list.
   2. A sample governs only the Problem Statement. Table rows stay one terse, testable sentence each.
   3. Apply the rewrite with `Edit`, one section at a time. Before each edit, check that every ID, number, unit, identifier, citation, `(inferred)` or `(assumed)` tag, and bounding word (not, only, never, unless, must) survives. A line whose meaning moved keeps its original wording; tell the user which.
   4. Log the date, the sections changed and the lines kept original under Humanizer, and add `Humanized: <date>` to the progress lines. A piece doc copied unchanged from a humanized parent needs no second pass; newly written text does.

4. **Final check.** The reader in phase 5 tested a draft, and steps 1 to 3 have rewritten it. For each doc those steps changed, dispatch a fresh reader with prompt C's final check. Fix wording and ambiguity yourself. A contradiction goes back to step 1 for the user to decide: if their decision changes a requirement, delete `Swept:` and `Humanized:` so the sweep, the pass and this check run again on the new text. Log what came back under Reader test, and add `Final check: <date>` to the progress lines, or `Final check: <date>, skipped, unchanged since the reader test` when steps 1 to 3 changed nothing.

5. **Walk it.** Show the user the requirement doc itself; it is short enough to read in full. Then walk the coverage list out loud: which rows are settled, which carry a default, which are assumed because the user said to stop asking, and which were collapsed. A row with no state at all was never asked: go back to phase 3 for it, and if the doc changes, delete `Covered:` and every progress line after it. Every open question in the record ends answered, defaulted with consent, or logged with an owner and a default.

6. **Say what signing commits them to.** Rank the defaults and assumptions by consequence: which of them, if wrong, make the work downstream build the wrong thing, and which are cosmetic. Give the three top-ranked risks, not a flat list of every open item.

7. **Sign off.** When the user approves:
   - Set `Status: signed-off <date>` in every requirement doc, the parent and each piece.
   - In the root README's Designs table, set the system's row to `Scoped` if it still reads `Scoping`. A row that is already further along stays as it is. With several pieces, add a `Scoped` row for each piece below the parent's row.
   - Offer to commit the docs, the record, `investigation.md` if there is one, and the README. Commit only when the user says so.

   From then on the signed text changes only through the spec's `## Amendment` sections. Then give the hand off, as SKILL.md describes.

## Prose voice

House style for the doc's four sections. A humanizing pass keeps these constructions: they are style, not AI patterns.

- Present tense for what exists ("Today's handler aborts the whole call on the first invalid row."), third person throughout. "We" appears only inside a walkthrough.
- Contractions are normal: "doesn't", "can't", "it's".
- Claims are bounded by contrast. ", not X" and "rather than" mark what the requirement deliberately is not ("decided now rather than deferred"). They carry scope; keep them.
- A colon introduces the elaboration of what precedes it.
- Bold lead-ins name list items and qualities: `**Fault tolerance.**`, `**Pros:**`. They are structure, not emphasis.
- Requirement IDs appear inline wherever a sentence serves one.
- Identifiers go in backticks.
- Zero em or en dashes.
- No hedging filler, no summary closers, no marketing adjectives.
