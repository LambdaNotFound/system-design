---
name: review-design
description: Critiques a design doc the way a bar-raiser would. A fresh subagent derives its own answer from the requirements, then challenges the design on necessity, sufficiency, failure modes, reversibility and internal consistency, and checks its evidence. Takes an optional focus for a deep dive on one area. Works on a design under design-doc/ or on any design doc by path, including one this repo did not produce. Findings land in a review ledger with IDs, and a design that new-design produced is changed only on the user's word. Use when the user asks to review, critique, deep-dive into or poke holes in a design. Not for writing a design (use new-design) or scoping requirements (use scope-problem).
argument-hint: "[slug, or path to a design doc] [focus: area or question]"
---

# Review design

You run one on-demand design review. A fresh bar-raiser subagent does the reviewing; you find the design, dispatch it, and take its findings to the user. Never review the design in its place: you have seen this conversation, and the bar-raiser's derivation must not be anchored by it.

The design and the focus are in the skill's arguments (the `ARGUMENTS:` line) or the conversation.

Files that go with this skill. If one is missing, stop and tell the user where it should be:

- `${CLAUDE_SKILL_DIR}/../new-design/roles/bar-raiser.md`: the bar-raiser's instructions, shared with `/new-design`.
- `${CLAUDE_SKILL_DIR}/../new-design/roles/investigator.md`: the investigator's, for a fix.
- `${CLAUDE_SKILL_DIR}/../new-design/rfc-spec.md`, with `rfc-exemplar.md` beside it, and `${CLAUDE_SKILL_DIR}/../scope-problem/requirement-spec.md`.
- `${CLAUDE_SKILL_DIR}/rubric.md`: the system-design map the bar-raiser reviews with.

## 1. Find the design

| The user named | Design | Requirements | Ledger |
| --- | --- | --- | --- |
| A slug whose folder holds `design.md` | `design-doc/<slug>/design.md` | `design-doc/<slug>/requirement.md` | `design-doc/<slug>/review.md` |
| A file inside this repository | that file | a `requirement.md` in its folder, else `in the design doc` | `review.md` in its folder |
| A file outside this repository | that file | the requirement doc the user names, else `in the design doc` | a path the user names, else `review.md` in a fresh `mktemp -d` directory |

- Nothing named, or more than one match: list `design-doc/` and ask.
- A slug whose folder has no `design.md`: there is no design to review yet. Point the user to `/new-design <slug>` and stop.
- This repository is public. Write nothing about a doc from outside it into the repository unless the user says where.
- **The loop is still open.** A design under `design-doc/` whose `/new-design` loop has not closed is not reviewed here: a review appended mid-loop would be read as loop state. The loop is open when `review.md`'s last section is a loop section (`## Round`, `## Editorial check` or `## Amendment ... check`) with no `Verdict:` line, when its last `## Round` verdict is `revise`, `reject-approach` or `escalate` with no decision for it in `loop.md`, or when `rfc.md` does not exist yet. Say which, point the user to `/new-design <slug>`, and stop.
- **An earlier review was cut off.** If the ledger's last section is a `## Review <k>` with no `Verdict:` line, finish that review before starting another: dispatch step 3 with that same `<k>` and the focus its `Focus:` line records, and tell the user. A focus the user gave this time waits for the next review.
- **A fix is in flight.** If the folder holds a `design.pre-V<k>.md` snapshot, an earlier review's fix was interrupted. Finish it first: go to step 5 and take the first that matches. The revision log has `V<k>.1: reopens the design`: delete the snapshot and tell the user. It has no `V<k>.1:` entry: dispatch the pass. It has more `V<k>.<p>:` entries than the ledger has `## Review <k> check` sections: dispatch the check. The last check's verdict is `revise`: dispatch the next pass. Otherwise close the fix.

## 2. Take the focus

The focus is the user's own words for the area or question they want dug into ("what happens when Redis is down", "the sharding key"). Pass it verbatim: a paraphrase narrows or widens what they asked. With none, the review covers the whole design.

## 3. Dispatch the bar-raiser

`<k>` is one more than the number of `## Review <k>` sections in the ledger, not counting `check` sections. Send this to a fresh `general-purpose` subagent, at the highest reasoning effort the Agent tool offers. Never use a fork, or a named agent the session defines. Fill every `<field>` with an absolute path or a value:

```text
You are the bar-raiser, running an on-demand design review.
Read <path of bar-raiser.md> in full before anything else, and follow it.
Task: on-demand review <k>
Design: <path>
Requirements: <path, or: in the design doc>
Ledger: <path>
Focus: <the user's words, or none>
RFC spec: <path of rfc-spec.md>
Rubric: <path of rubric.md>
```

Relay nothing else. If it returns and the ledger's last section has no `Verdict:` line, the review is unfinished: dispatch the same task again.

## 4. Report and discuss

Report the verdict, the ledger path, the objections with their IDs, blocking first, and the `Strong:` line. Quote the objections as the ledger has them; don't strengthen or soften one.

Then go through the blocking objections with the user, one at a time, the way CLAUDE.md's coaching style asks: what failure sequence does it describe, would they rebut it, and what would have to be true for the design to hold. The review is theirs to think with. Nothing in the design changes at this step.

## 5. Fix, on the user's word

Only a design that `/new-design` produced can be fixed here: its `design.md` carries the loop-state marker and belongs to the investigator. Any other doc is comment-only. Never edit a design yourself.

The user names the objection IDs they want fixed. An objection whose fix is a change to a requirement is an amendment, not a fix: point the user to `/new-design <slug>`, which runs it as Amending after approval.

1. **Snapshot.** Copy `design.md` to `design.pre-V<k>.md`.
2. **Pass.** Dispatch the investigator with the task `review V<k> pass 1: <objection IDs>`. It answers those objections by ID, edits only what resolving them touches, and logs `V<k>.1:`. If it logs `V<k>.1: reopens the design`, the fix is too large for a scoped edit: delete the snapshot, tell the user why, and point them to `/new-design <slug>`, which puts the reopening to them.
3. **Check.** Dispatch the bar-raiser with the task `review V<k> check 1`. It appends `## Review <k> check 1` to the ledger, ending in a `Verdict:` line. On `revise`, the investigator answers in pass 2 and the bar-raiser checks again; the second check lands approve, approve-with-risks, or escalate. Put an escalation to the user.
4. **Close the fix.** Export the RFC again, delete the snapshot, and report what changed and the verdict:

   ```sh
   awk '/^<!-- loop state below: not part of the RFC -->$/{exit} {print}' <dir>/design.md > <dir>/rfc.md
   ```

The pass and the check go to fresh `general-purpose` subagents in the loop's own format, with `<dir>` as `design-doc/<slug>/`:

```text
You are the <investigator | bar-raiser> in a design-review loop.
Read <role file> in full before anything else, and follow it.
Loop directory: <dir>
Slug: <slug>
Task: <task>
RFC spec: <path of rfc-spec.md>
Rubric: <path of rubric.md>
RFC exemplar: <path of rfc-exemplar.md>
Requirement spec: <path of requirement-spec.md>
```

Commit only when the user asks.
