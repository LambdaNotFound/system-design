---
name: new-design
description: Writes a formal RFC for a system from its signed-off requirement doc, using a design-review loop of fresh subagents. An investigator derives and writes the design; a bar-raiser derives its own answer from the requirements, checks the design's evidence and challenges it for up to 5 rounds until it approves or escalates; a prose auditor then gives the approved RFC one editorial pass. All loop state lives in files under design-doc/, so an interrupted loop resumes from them. Ends with the approved design, the review ledger as its audit trail, and an exported rfc.md. Use when the user wants a system designed or an RFC written from a requirement doc, e.g. "design the rate limiter", or wants to resume a design by slug. Not for scoping requirements (use scope-problem), critiquing an existing design (use review-design), or implementing one.
argument-hint: "[system slug, or a slug to resume]"
---

# New design: the design-review loop

You are the lead of a design-review loop that turns a signed-off requirement doc into a formal RFC. Three roles do the work, each a fresh subagent:

- **Investigator**: researches, writes `design.md`, answers each review round, and adopts the editorial pass.
- **Bar-raiser**: derives its own answer from the requirements, challenges the design for up to 5 rounds, and writes each verdict to `review.md`.
- **Auditor**: one editorial pass over the approved document.

You write no design yourself. The user's part comes before the loop (the requirements, in `/scope-problem`) and after it: the decisions the loop escalates, the architecture drawing, and `/review-design` deep dives.

The system is in the skill's arguments (the `ARGUMENTS:` line) or the conversation. If neither names one, ask.

**If this file was cut short.** A long session can be compacted, and that keeps only the start of this file. If you cannot see the "Close" section at its end, read this file again from disk before you go on.

Files that go with this skill. If one is missing, stop and tell the user where it should be:

- `${CLAUDE_SKILL_DIR}/waiting.md`: what to do when the loop waits for the user. Read it when it does.
- `${CLAUDE_SKILL_DIR}/roles/investigator.md`, `roles/bar-raiser.md` and `roles/auditor.md`: each role's instructions. The subagents read them; you don't need to.
- `${CLAUDE_SKILL_DIR}/rfc-spec.md`: the RFC contract, with `rfc-exemplar.md` beside it.
- `${CLAUDE_SKILL_DIR}/../scope-problem/requirement-spec.md`: the requirement doc contract.
- `${CLAUDE_SKILL_DIR}/../review-design/rubric.md`: the system-design map the bar-raiser reviews with.

## Files

`<dir>` is `design-doc/<slug>/`. Each file has one writer:

| File | Writer | Holds |
| --- | --- | --- |
| `requirement.md` | `/scope-problem`; after sign-off, you append amendments on the user's instruction | the requirements, read in place and never copied |
| `design.md` | investigator | the RFC, then the loop state below its marker |
| `review.md` | bar-raiser | the append-only ledger: rounds, checks, and `/review-design` reviews |
| `loop.md` | you | the append-only log of user decisions and failed audits |
| `design.rewritten.md` | auditor | the editorial draft, deleted at close |
| `design.pre-A<n>.md` | you | `design.md` as it stood before amendment `A<n>`, deleted when it closes |
| `rfc.md` | you | the exported RFC, rewritten at every close |
| `diagrams/` | the user | the architecture drawing, made after approval |

These files are the whole loop state. The investigator and the bar-raiser keep nothing between invocations: they re-read the files every time. Both take the requirements from `requirement.md` plus its amendments, the latest winning on conflict, never from your prompt or from the design's restatement of them. The doc's `Scoping record:` line names `scoping.md`; both agents may read it for evidence and open questions. It binds nothing, and no loop agent writes it.

Dates are `MM/DD/YY` (`date +%m/%d/%y`).

## Dispatch

Send each role to a fresh `general-purpose` subagent. Never use a fork, or any agent that sees this conversation: the bar-raiser's derivation must not be anchored by it, and the files must carry everything. Do not substitute a named agent the session defines: its own instructions may write to other paths. Where the Agent tool offers a reasoning effort, run the investigator and the bar-raiser at the highest one.

Fill every `<field>` with an absolute path or a value, and send:

```text
You are the <investigator | bar-raiser | auditor> in a design-review loop.
Read <role file> in full before anything else, and follow it.
Loop directory: <dir>
Slug: <slug>
Task: <task>
RFC spec: <path of rfc-spec.md>
Round budget: <M>
Reopened by: <objection IDs or an amendment, or none>
Rubric: <path of rubric.md>
RFC exemplar: <path of rfc-exemplar.md>
Requirement spec: <path of requirement-spec.md>
```

The auditor gets the lines down to `RFC spec:` only.

The tasks are `round 0`, `respond to round <N>`, `review round <N>`, `audit design.md`, `adopt the editorial pass`, `editorial check <k>: <sections and corrections>`, `respond to editorial check <k>`, `amendment A<n> pass <p>`, `amendment A<n> check <p>`, and `embed the diagram`.

Relay only slugs, round numbers, verdicts, and tasks. The substance travels through the files: a paraphrased objection drifts. The one exception is the editorial check, which carries the sections that changed.

## Before round 0

1. **Slug.** The one given, else a short kebab-case slug from the system's name, with no number prefix.
2. **Resume?** If `<dir>` holds `design.md` or `review.md`, go to Resuming. The files outrank the arguments on any disagreement.
3. **Requirement doc.** Read `<dir>/requirement.md` and go by its header lines:
   - `Status: signed-off` and `Route: design`: this is the loop's input.
   - `Status: draft`: scoping is unfinished. Offer `/scope-problem <slug>`, which picks the draft up where it stopped. Run on the draft only at the user's explicit word; until then, stop here.
   - `Route: decomposed`: the designs belong to the pieces. List the piece slugs the Decomposition table in `scoping.md` routes to `design`, and ask which one to start.
   - Any other route or status: say where the doc is routed, and stop.
   - No file: stop and offer `/scope-problem <slug>`. Never invent the requirements, and never edit the doc or its status: it belongs to `/scope-problem` and the user.
4. **Say what you are about to run:** an RFC to the RFC spec, its word budget (the doc's `Word budget:` line, else 4,000 words), and the cost: up to 13 deep subagent runs when every round is used.
5. **Commits.** Ask once per session whether to commit `<dir>` after each round, and wait for the answer before any dispatch. CLAUDE.md has the message format. Push only if the user asks.

## The loop

1. **Round 0: author.** Set the system's row in the root `README.md` Designs table to `Drafting`, then dispatch the investigator with `round 0`. It writes `design.md`, with an `## Evidence` table, a `## Revision log` and an `## Objection responses` section below the loop-state marker. If it returns `not answerable`, the loop waits for the user.
2. **Round N: challenge.** Dispatch the bar-raiser with `review round N`. It appends `## Round N` to `review.md` and returns a verdict:
   - `approve` or `approve-with-risks`: step 4. The investigator is not invoked again for what an `approve-with-risks` round leaves open: the round's residual-risk and unaddressed lists are their disposition, and they travel with the deliverable.
   - `revise`: step 3.
   - `reject-approach`: step 3, and the investigator restarts the design core instead of patching it. It costs a round.
   - `escalate`: the loop waits for the user. Do not loop around an irreconcilable core.
3. **Respond and revise.** Dispatch the investigator with `respond to round N`. It answers every objection by ID (accepted, rebutted, or deferred), revises `design.md` in place, and logs `R<N>:`. Then step 2 as round N+1.
4. **Editorial pass, after approval only.** Dispatch the auditor with `audit design.md`. It writes `design.rewritten.md` and returns a claim-inventory report.
   - Clean inventory: dispatch the investigator with `adopt the editorial pass`. Then verify the adoption yourself, because the inventory was run on the rewrite and not on what was adopted:

     ```sh
     diff <(awk '/^<!-- loop state below: not part of the RFC -->$/{exit} {print}' <dir>/design.rewritten.md) <(awk '/^<!-- loop state below: not part of the RFC -->$/{exit} {print}' <dir>/design.md)
     ```

     No difference and `Adopted`: the loop closes. Any difference, or `Adopted with corrections`: dispatch the bar-raiser with `editorial check 1`, naming the sections that differ and the investigator's corrections. `approve` closes the loop; any other verdict waits for the user.
   - Failed rewrite (a lost claim, a changed section): keep `design.md` as approved, append `audit | <date> | failed | <reason>` to `loop.md`, and close. Tell the user the audit failed and why. Run the auditor again only on the user's word.

   Delete `design.rewritten.md` when the loop closes.
5. **The round budget is a deadline, not a target.** It is 5 rounds, or the highest `rounds through <M>` in `loop.md`. The bar-raiser lands approve, approve-with-risks, or escalate by the last round. A `revise` or `reject-approach` there is out of contract: treat it as `escalate`, and put the round's open blocking objections to the user. Converging in 2 rounds because round 1 was thorough is the success case.

## Loop rules

- Never edit `design.md` or `review.md`. Change `requirement.md` only by appending a dated `## Amendment` section, on the user's instruction; its signed text is never rewritten. An amendment is revised material: it allows new blocking objections without a moving-goalposts penalty.
- No agent plays another's role. The auditor touches prose only: technical verdicts never route through it, and its pass never reopens the review.
- The editorial pass runs once per approved design, at the end. Prose churn between rounds breaks the bar-raiser's spot-checks for no quality gain.
- **Brief corrections.** The agents call the requirement doc the brief. When one reports that it asserts something a source contradicts, put that to the user with the evidence and offer the `## Amendment` that fixes it, at the next stop or at the end. A factual correction does not pause or reopen the design. Never amend on your own. One the user accepts after approval runs as Amending after approval.
- If the user agreed to commits, commit `<dir>` after each round, pass, and check. The agents write files and never commit.
- A round with no visible change (no revision-log entry, no new review section) means the loop is stuck: stop and tell the user which role stalled, and on what.
- Report to the user at the end, or when the loop waits for them, or on a stall: the verdict, the file paths, and the residual risks. No play-by-play of rounds.

## When the loop waits for the user

Four things stop the loop for a decision: `R0: not answerable`, a round's `escalate`, an editorial check that is not approved, and an amendment or review fix that reopens the design. Read [waiting.md](waiting.md) now: it has what to put to the user, their options, and the `D<n>` line to record in `loop.md`.

## Amending after approval

Once a round approves, an amendment the user makes gets a scoped edit, not a reopened loop. Append it to `requirement.md` headed `## Amendment <date> (after approval)`. It is `A<n>`, the doc's n-th `## Amendment` section.

1. **Snapshot.** Copy `design.md` to `design.pre-A<n>.md`. The bar-raiser checks the pass against it.
2. **Pass.** Dispatch the investigator with `amendment A<n> pass 1`. It edits only what the amendment touches and logs `A<n>.1:`, naming every section it edited. If honoring the amendment needs more, it edits nothing and logs `A<n>.1: reopens the design`: the loop waits for the user.
3. **Check.** Dispatch the bar-raiser with `amendment A<n> check 1`. It appends `## Amendment A<n> check 1` to `review.md`, ending in a `Verdict:` line. On `revise`, the investigator answers in pass 2 and the bar-raiser checks again; the second check lands approve, approve-with-risks, or escalate.
4. **Close.** Run Close and delete the snapshot. An edit this small gets no second editorial pass.

## Resuming

`<dir>` is the checkpoint; no conversation context is needed. Read every file in `<dir>` and announce the state you resumed into, so the user can correct it. Ask the commit question from Before round 0 before the session's first dispatch. Then take the first state that matches:

1. `/review-design` owns the open work: the last section of `review.md` is a `## Review <k>` or `## Review <k> check <p>` with no `Verdict:` line, or a `design.pre-V<k>.md` snapshot exists. Point the user to `/review-design <slug>`, which finishes it, and stop. Never dispatch a review or a review-fix check from here: those prompts are `/review-design`'s.
2. The last section of `review.md` is a loop section (`## Round <N>`, `## Editorial check <k>` or `## Amendment A<n> check <p>`) with no `Verdict:` line: it is unfinished. Dispatch the bar-raiser for that same round or check, and say the section is unfinished. This comes before the states below because they read verdicts.
3. No `design.md`, or no `R0:` entry in its revision log: round 0 is pending. Dispatch the investigator with `round 0`.
4. The loop waits for the user ([waiting.md](waiting.md)) and `loop.md` has no decision line for this occurrence: put it to them. With a decision line, do what its column says, and read the states below with the verdict counted as that line makes it count.
5. The last `## Round N` verdict is `revise` or `reject-approach`, and the revision log has no `R<N>:` entry: dispatch the investigator with `respond to round N`.
6. Same verdict, and the `R<N>:` entry exists: dispatch the bar-raiser with `review round N+1`.
7. The latest `(after approval)` amendment, `A<n>`, has no `A<n>.1:` entry: make its snapshot if it is missing, then dispatch the investigator with `amendment A<n> pass 1`.
8. `A<n>` has more `A<n>.<p>:` entries than `## Amendment A<n> check` sections: dispatch the bar-raiser for the next check.
9. `A<n>`'s last check verdict is `revise`: dispatch the investigator for the next pass.
10. The last round approved, and the revision log has no `editorial:` entry after its last `R<N>:` entry:
    - the latest `audit` line in `loop.md` reads `failed`, with no `continue` decision after it: the loop is closed. Run Close.
    - otherwise delete any `design.rewritten.md`, because the report that says whether it is clean was lost with the session, and dispatch the auditor.
11. That `editorial:` entry exists, `design.rewritten.md` is still there, and `review.md` has no `## Editorial check`: run the adoption diff from step 4 of the loop, and dispatch the bar-raiser with `editorial check 1` if anything differs or the entry reads `adopted with corrections`.
12. The loop is closed, `design.md` still has a line starting `*Diagram:` above the marker, and `diagrams/architecture.excalidraw.svg` exists: look at the drawing (CLAUDE.md says how) and review it against the design. When it matches, or the user says to embed it anyway, dispatch the investigator with `embed the diagram`, then run Close.
13. Otherwise the loop is closed. Delete a leftover `design.rewritten.md` and the snapshot of any amendment whose last check approved, then run Close.

## Close

The deliverable is the approved, audited `design.md`, the full `review.md` as its audit trail, and the exported RFC. Export the publishable part, line 1 down to the loop-state marker:

```sh
awk '/^<!-- loop state below: not part of the RFC -->$/{exit} {print}' <dir>/design.md > <dir>/rfc.md
```

Then, in the root `README.md` Designs table, set the system's row to `Reviewed` and point its link at `design-doc/<slug>/rfc.md`. A row that is already further along keeps its status.

Report the verdict, the paths of `rfc.md`, `design.md` and `review.md`, the residual risks with their triggers, and any brief correction still open. If the RFC still carries a `*Diagram:` line, give the user its brief: what to draw, and to save it as `diagrams/architecture.excalidraw.svg` and run `/new-design <slug>` again to have it embedded. Name the next steps: `/review-design <slug>` for a deep dive on any area, and `/implement-design <slug>` to build the core mechanism. Commit only if the user agreed to commits.
