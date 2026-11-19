---
name: implement-design
description: Builds the core mechanism of a design as a Rust crate under src/, using an implementation loop of fresh subagents. A coding expert plans the crate from the RFC, the requirement doc or a concept note, and the user approves the plan. It then writes the types and failing tests, which a bar-raiser reviews against the design, and only then the logic, which the bar-raiser reviews against the checks the lead ran. Each stage gets at most 2 review rounds. All loop state lives in files beside the design, so an interrupted loop resumes from them. Ends with a tested crate and its review ledger, ready for the user's manual review. Use when the user wants to implement, build or code up a system or concept from this repo, or to resume an implementation by slug. Not for scoping requirements (use scope-problem), writing the design (use new-design), or critiquing a design (use review-design).
argument-hint: "[system, piece or concept slug, or a slug to resume]"
---

# Implement design: the implementation loop

You are the lead of an implementation loop that turns an approved design into a tested Rust crate. Two roles do the work, each a fresh subagent:

- **Coding expert**: plans the crate, writes its tests, then its logic, and answers each review round.
- **Bar-raiser**: derives what a correct crate must contain, reviews the tests before any logic exists, then reviews the logic against your check record.

You write no plan and no code. The user's part is one approval, of the plan before any code is written, then the decisions the loop escalates and the manual review of the finished crate. That review is the backstop, not the excuse.

The slug is in the skill's arguments (the `ARGUMENTS:` line) or the conversation. If neither names one, ask.

**If this file was cut short.** A long session can be compacted, and that keeps only the start of this file. If you cannot see the "Close" section at its end, read this file again from disk before you go on.

Files that go with this skill. If one is missing, stop and tell the user where it should be:

- `${CLAUDE_SKILL_DIR}/roles/coding-expert.md` and `roles/bar-raiser.md`: each role's instructions. The subagents read them; you don't need to.
- `${CLAUDE_SKILL_DIR}/impl-spec.md`: the contract for the plan, the crate and the checks. Read its Checks section before your first check run.
- `${CLAUDE_SKILL_DIR}/review-rust.md`: the Rust checklist the bar-raiser reviews with.

## Files

`<dir>` is the folder beside the brief: `design-doc/<slug>/` for a system or a piece, `concepts/<slug>/` for a concept. Each file has one writer:

| File | Writer | Holds |
| --- | --- | --- |
| `impl-run.md` | you | plain `Key: value` lines: the run's facts, the user's approvals and decisions, the brief corrections, one line per pass |
| `impl.md` | coding expert | the plan, then the revision log and the objection responses |
| `impl-review.md` | bar-raiser | the append-only ledger: tests rounds, then review rounds |
| `impl-checks/` | you | the record of each check run and the tree manifest, one each per author pass, and the logs, which git ignores |
| `src/` | coding expert | the crate `src/<slug>/`, and the workspace's `Cargo.toml` and `Cargo.lock` |

These files and the crate are the whole loop state. Every agent keeps nothing between invocations and rereads them each time. The brief is read where it is and never copied; the spec says which documents it is. A path written into `impl-run.md` is relative to the repository root, because the file is committed to a public repository.

Dates are `MM/DD/YY` (`date +%m/%d/%y`).

## Dispatch

Send each role to a fresh `general-purpose` subagent. Never use a fork, or any agent that sees this conversation: the bar-raiser's derivation must not be anchored by it, and the files must carry everything. Do not substitute a named agent the session defines, and never pass a `name` on the Agent call: a named call can start a teammate instead of a subagent. Where the Agent tool offers a reasoning effort, run both roles at `high`. Go higher only when the user names a level: `xhigh` and `max` lengthen every run, and pay off only where a gain has been measured.

Fill every `<field>` with an absolute path or a value, and send:

```text
You are the <coding expert | bar-raiser> in an implementation loop.
Read <role file> in full before anything else, and follow it.
Loop directory: <dir>
Slug: <slug>
Task: <task>
Round: <A<n> in an amendment round, or none>
Brief: <paths>
Impl spec: <path of impl-spec.md>
Rust checklist: <path of review-rust.md>
```

The coding expert gets the lines down to `Impl spec:` only.

The tasks are `map`, `write the tests`, `respond to tests round <N>`, `implement`, `fix the failed checks: <record path>`, `respond to round <N>`, `review tests round <N>`, and `review round <N>`.

Relay only the slug, round numbers, verdicts, tasks and record paths. The substance travels through the files and the crate: a paraphrased objection drifts.

After every pass, confirm it wrote its file: `impl.md` has the new plan or log entry, or the ledger has the new section. A pass that wrote nothing has stalled: dispatch it once more, and stop on a second miss, telling the user which role stalled and on what. A ledger section also needs its `Checked:` line above the `Verdict:` line. One without it is unfinished: dispatch the bar-raiser for that round again.

## Before the map

1. **Slug.** The one given, else a short kebab-case slug, with no number prefix.
2. **Resume?** If `design-doc/<slug>/impl-run.md` or `concepts/<slug>/impl-run.md` exists, go to Resuming. The files outrank the arguments.
3. **The brief.** Read `design-doc/<slug>/requirement.md` if it exists, and go by its header lines:
   - `Route: design`: the brief is `rfc.md` with the requirement doc. If `rfc.md` doesn't exist, the design loop hasn't closed: point the user to `/new-design <slug>` and stop.
   - `Route: code`: the brief is the requirement doc alone. That holds for a signed-off doc and for the three-line doc of a `Status: closed` early exit.
   - `Route: decomposed`: the code belongs to the pieces. List the piece slugs the Decomposition table in `scoping.md` routes to `design` or `code`, and ask which one to build.
   - `Route: no code`: there is nothing to build. Point the user to the piece's handoff.
   - `Status: draft`, whatever the route: scoping is unfinished. Offer `/scope-problem <slug>`, and run on the draft only at the user's explicit word.

   With no requirement doc, a `concepts/<slug>.md` written at least through How it works is the brief. With neither, stop and offer `/scope-problem <slug>`. Never invent the requirements, and never edit a brief.
4. **Say what you are about to run:** a crate of one to three mechanisms, built in two stages, the tests first and then the logic, each reviewed for at most 2 rounds; one stop, for their approval of the plan; and the cost: up to 9 deep subagent runs when every round is used.
5. **Commits.** Ask once per session whether to commit `<dir>` and `src/` after each pass, and wait for the answer. The two checks below come after it, so this opening is the same on every machine. CLAUDE.md has the message format. Push only if the user asks.
6. **Rust.** Run `cargo --version`. If it fails, stop and tell the user what is missing, and that rustup (https://rustup.rs) installs it. Never install or configure a toolchain yourself.
7. **The tree.** `git status --porcelain -- src/` prints nothing, and `src/<slug>/` doesn't exist. Otherwise show the user what is there and stop. Never stash, reset or clean their work.
8. **Write `<dir>/impl-run.md`:** `Slug:`, `Brief:` (the paths), `Crate: src/<slug>`, `Base:` (the full SHA of `HEAD`), and `Started:` (the date and time).

## The loop

1. **Map.** Dispatch the coding expert with `map`. It writes the plan in `impl.md` and no code. `blocked` or `split proposed` waits for the user.
2. **Plan gate.** Show the user the plan in short, with the file's path: the mechanisms, the test plan, what the crate will not prove, and any dependency. Put every open brief correction to them with it (Loop rules). On a yes, append `Plan approved: <date>, <N> amendments, <M> scope lines` to `impl-run.md`, with the counts from `impl.md`'s first line. A change they ask for goes into `impl-run.md` in their own words as a `Scope:` line, and the map runs again. A change to a requirement is not yours to make: point them to `/new-design <slug>`, which amends the requirement doc. No code is written without an approval line whose counts match the files.
3. **Tests stage, first pass.** Dispatch the coding expert with `write the tests`. It creates the workspace if needed, then the crate: types, signatures, `todo!()` bodies, and every test in the plan. It logs `T0:`.
4. **Checks, after every author pass.** Write the tree manifest first, `impl-checks/<label>.sha256`, with the spec's command. Then run each command in `impl.md`'s `## Checks` block once, from `src/`, and write the record as the spec's Checks section says: `impl-checks/<label>.md`, with `<label>` the pass's log key. In the tests stage a pass means the crate compiles and every failing test fails at a `todo!()` (in an amendment round, the spec's An amendment round says what passes). In the implementation stage it means everything is green. On `Result: fail`, dispatch the coding expert with `fix the failed checks: <record path>`, then run them again; a second failure waits for the user. Then, if the user agreed to commits, commit `<dir>` and `src/`. Append `Pass: <label> <commit SHA>` to `impl-run.md`, or `Pass: <label> uncommitted`.
5. **Tests review, round N.** Dispatch the bar-raiser with `review tests round N`. It appends `## Tests round N` to `impl-review.md`, ending in a `Checked:` line and a `Verdict:` line:
   - `approve`: step 7.
   - `approve-with-risks`: step 7, unless the section has a should-fix objection and this is not the stage's last round. Then treat it as `revise`.
   - `revise`: step 6. In the stage's last round, treat it as `escalate`.
   - `escalate`: the loop waits for the user.
6. **Respond.** Dispatch the coding expert with `respond to tests round N`. It answers every objection by ID, revises the tests, and logs `T<N>:`. Run step 4, then step 5 as round N+1.
7. **Implementation stage, first pass.** Dispatch the coding expert with `implement`. It fills in the `todo!()`s and logs `R0:`. The approved tests are frozen: one it changes is named in the log with its reason. Run step 4.
8. **Review, round N.** Dispatch the bar-raiser with `review round N`. It appends `## Round N`. The verdicts route as in step 5, with `approve` going to Close and `revise` to step 9.
9. **Respond.** Dispatch the coding expert with `respond to round N`. It logs `R<N>:`. Run step 4, then step 8 as round N+1.
10. **The budget is a deadline, not a target.** Each stage has 2 rounds, or the highest `rounds through <M>` on a `Budget:` line for it in `impl-run.md`. Converging in one round because it was thorough is the success case.

## Loop rules

- Never write `impl.md`, `impl-review.md`, or anything under `src/`. Never edit the brief.
- No agent plays another's role. Disagreement between the author and the reviewer is signal: surface it, never smooth it.
- The checks run once per author pass, by you. The author runs the tests it needs while it works, and the reviewer runs probes for its own questions.
- After a review pass, the spec's manifest command prints exactly the newest manifest, and with commits on, `git status --porcelain -- src/` prints nothing. Otherwise the reviewer changed the tree: stop and show the user.
- **Brief corrections.** When an agent's return reports that the brief asserts something the code or a source contradicts, and no `C<n>` line has it yet, append it to `impl-run.md` in the agent's words: `C<n> | <date> | <pass or objection ID> | <brief claim> | <what is true>`. Put each one without a decision line to the user with the evidence at the next stop (the plan gate, or any wait below), or else at Close, and record the answer as a `D<n>` line. The fix is an amendment through `/new-design <slug>`, or a design fix through `/review-design <slug>`.
- An amendment to the requirement doc, or a new `Scope:` line, while the loop is open sends it through the map and the plan gate again. The loop then resumes where the files say it was, and the bar-raiser's next round re-derives against the change.
- Keep the user posted between passes, one line each: which pass finished, its result or verdict, and what runs next. The full report comes at the plan gate, at the end, or when the loop waits for them. The objections stay in the files until then.

## When the loop waits for the user

Put the matter to them in one message: the author's question for `blocked`, the proposed split, the failing check record, or the ledger's escalation paragraph. Record the answer as one line in `impl-run.md`, so a resume knows it was given:

`D<n> | <date> | <what it answers> | continue or accept or stop | <note>`

- `continue`: on an escalation, the verdict counts as `revise` and the stage gets one more round: append `Budget: <tests or review> rounds through <N+1>` to `impl-run.md`. On a block or a failed check, dispatch the author again once the user has settled the question, with any instruction recorded as a `Scope:` line.
- `accept`, on an escalation only: the user approves over the bar-raiser. The verdict counts as `approve-with-risks`, and the escalation paragraph travels as a residual risk.
- `stop`: the loop ends where it is. Append `Closed: <date>, stopped`, leave the README row as it is, and report.

A brief correction is answered with its `C<n>` as what the line answers, and one of three decisions. `as written`: the crate follows the brief's text, and the correction stays open for the amendment. `amend`: the loop pauses while the user amends through `/new-design <slug>`; the amendment then sends it through the map again. `stop`, as above. `continue` and `accept` answer the waits above, never a correction.

## Amending after close

An `## Amendment ... (after approval)` section that `impl.md`'s `Covers:` line doesn't count reopens a closed crate as round `A<n>`, the doc's n-th `## Amendment` section; the oldest goes first. The spec's An amendment round defines its labels, stages and checks.

1. **The design comes first.** With an RFC in the brief, `review.md`'s last `## Amendment A<n> check` must read `approve` or `approve-with-risks`. Otherwise point the user to `/new-design <slug>` and stop.
2. **Say what runs:** the map for A<n>, the plan gate, both stages at 2 rounds each, up to 9 deep subagent runs. Ask the commit question. `git status --porcelain -- src/` must print nothing, or show the user and stop.
3. **Reopen.** Append `Reopened: <date>, A<n>` to `impl-run.md` and run the loop from step 1, with `Round: A<n>` in every dispatch. Its `Plan approved:` line follows the `Reopened:` line, and Close writes `Closed: <date>, <verdict>, A<n>`.

## Resuming

`<dir>` and the crate are the checkpoint; no conversation context is needed. Read every file in `<dir>` and announce the state you resumed into, by name, so the user can correct it, and name each `C<n>` line that has no decision line yet. Ask the commit question from Before the map before the session's first dispatch. Take the first state that matches. In an amendment round, the states read only what follows its `Reopened:` line, and the entries, records and sections with its prefix.

1. `impl-run.md`'s last `Closed:` or `Reopened:` line is a `Closed:` line. With an amendment the `Covers:` line doesn't count, run Amending after close. Otherwise the loop is over: report its result again, and never restart it. More work on the crate, beyond an amendment, is the user's to do, or to ask for directly.
2. The ledger's last section has no `Verdict:` line, or no `Checked:` line: it is unfinished. Dispatch the bar-raiser for that same round.
3. No `impl.md`, or its `Covers:` line counts fewer amendments or `Scope:` lines than the files have: the map is pending.
4. No `Plan approved:` line with the counts on `impl.md`'s first line: the plan gate is pending. Show the plan and wait.
5. No `T0:` entry: dispatch `write the tests`.
6. The newest log entry has no check record, or its record reads `Result: fail`: step 4 is pending for that pass.
7. The loop waits for the user (above) and `impl-run.md` has no `D<n>` line for this occurrence: put it to them. With a decision line, do what it says, and read the states below with the verdict counted as that line makes it count.
8. No `## Tests round` section: tests round 1.
9. The last tests round's verdict is `revise`, or one step 5 treats as `revise`: with no `T<N>:` entry, the response is pending; with one, tests round N+1.
10. The tests are approved and there is no `R0:` entry: dispatch `implement`.
11. No `## Round` section: review round 1.
12. The last round's verdict is `revise`, or one treated as `revise`: with no `R<N>:` entry, the response is pending; with one, round N+1.
13. Otherwise the last round approved: run Close.

## Close

Check that the newest check record reads `Result: pass`. Then:

1. In the root `README.md` Designs table, set the system's row to `Implemented` and its Code column to `[src/<slug>](src/<slug>/)`. For a concept, add `**Code:** [src/<slug>](../src/<slug>/)` under the note's title instead.
2. Append `Closed: <date>, <verdict>` to `impl-run.md`. It goes last, so a resume that finds it knows the README is done.
3. Commit, only if the user agreed to commits.

Report the verdict; the crate's path and the command that runs its tests; the rows the tests prove and the rows under Not proven here; the residual risks with their triggers, the follow-ups, and each brief correction that is undecided or kept `as written`; and the paths of `impl.md` and `impl-review.md`. Tell the user the crate is theirs to read by hand now, and that the frozen tests in `src/<slug>/tests/` are the place to start, since each names the row it proves. Name `/review-design <slug>` for any doubt about the design that the code exposed.
