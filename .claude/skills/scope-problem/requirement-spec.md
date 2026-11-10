# Requirement doc spec

## Contents

- Layout: header lines and section order
- Sections: what each of the four sections holds
- Rules: wording, citations, labels, conflicts, and IDs
- Scoping record: its template, progress lines and sections

The requirement doc, `design-doc/<slug>/requirement.md`, is what downstream work builds on. `/scope-problem` writes it, and `/new-design`, `/review-design` and `/implement-design` read it where it is. A signed-off doc that meets this spec can be treated as the user's own words. It is a quality bar, not a gate: a skill handed a draft, an informal doc, or no doc at all should still run, say so, and name the gaps rather than guess past them.

Dates are `MM/DD/YY`.

## Layout

```markdown
# Requirement doc: <title>

Status: draft
Slug: <slug>
Route: undecided
Scoping record: design-doc/<slug>/scoping.md

## Problem Statement
## Functional Requirements
## Non-functional Requirements
## Low-level Design Preferences
```

Header lines are plain `Key: value` text with no bold, so a resume can match them exactly.

- `Status:` reads `draft` while scoping moves, `signed-off <MM/DD/YY>` once the user approves, or `closed, routed to <next step> without full scoping` on the early exit. A downstream skill should not start on a draft; handed one, it runs anyway and says so.
- `Route:` is `design`, `code`, `no code`, or `decomposed` for a parent whose pieces carry their own routes. It reads `undecided` until decomposition, or until the early exit sets it.
- A piece doc, `design-doc/<slug>-<piece>/requirement.md`, has the piece slug on its `Slug:` line and adds `Parent: <parent slug>` and `Depends on: <piece slugs and external blockers, or none>`. Its `Scoping record:` points at the parent's record.
- Until sign-off, `/scope-problem` alone writes the doc. After sign-off the signed text is never edited. The doc changes only through dated `## Amendment` sections appended below it, on the user's explicit instruction, by whichever skill the user is working in.

## Sections

**Problem Statement** (required). A high-level debrief in one short paragraph: what hurts or is missing, for whom, why now, and why doing nothing loses. Problem-shaped: a requested solution appears only as context, and the requirement under it lands in the tables below. A deadline or a mandated team is stated as given. The last line draws the boundary, `Out of scope: <exclusion>; <exclusion>.` A missing boundary means the scope was never challenged.

**Functional Requirements** (required). A `| ID | Requirement |` table, `FR1` onward: what the system must do. One sentence each, testable as written, so a reviewer can mark it pass or fail without asking what it means. "The import endpoint returns the accepted and rejected row counts for each call" is a requirement; "improve imports" is not.

**Non-functional Requirements** (required). The same table, `NFR1` onward: the qualities the design must hold, each with a threshold or an observable behavior. Draw from scale, latency, failure behavior, the correctness bar (which of data loss, stale reads, and double processing is fatal), security, operability, and compatibility or migration. A row may lead with its quality in bold, as in `**Fault tolerance.**`

**Low-level Design Preferences** (optional). A `| ID | Preference |` table, `LLD1` onward: choices the user imposes rather than derives, such as a language, a library, a pattern, an interface to match, or a decision already made. Omit the section when nothing is imposed; a preference invented to fill it is worse than none.

## Rules

- **One testable sentence per row.** Every table row is one sentence with an ID, and the Problem Statement is one short paragraph. There is no word limit to measure. A row that needs a second sentence is two requirements, or belongs in another table.
- **One claim per sentence, no evidence dumps.** A load-bearing claim may carry one citation (`path:line`, a code permalink, or a page URL). Everything else goes to the scoping record.
- **Observed by default.** Every statement is a requirement the user signed or a fact read from a source. Anything inferred or assumed says so inline, `(inferred)` or `(assumed)`, so it cannot harden into fact downstream. Where the user's belief and the code disagreed, the doc records what the code does.
- **Everything traces.** Every FR traces to the Problem Statement, every NFR to a real constraint (cited, or named as imposed), and every decomposition piece to at least one ID. Anything that traces to nothing gets cut.
- **No hidden conflicts.** Two rows that cannot both hold, or a row the code or a cited page contradicts, is put to the user and decided before sign-off, never resolved by the author alone. Every conflict in the scoping record carries the user's decision.
- **No unowned questions.** Every open question in the scoping record has an owner and a default, or the doc is not signable.
- **IDs are stable.** Never renumber a signed doc; a withdrawn requirement keeps its ID and is withdrawn by amendment. A piece doc numbers its own rows from 1, and the scoping record maps them to the parent's IDs.
- **Design-ready wording.** The design doc links to this doc instead of repeating it, and its sections, review findings and tests cite rows by ID. Write each row so it reads correctly on its own.

## Scoping record

`scoping.md`, beside the requirement doc, holds scoping's working state. `/scope-problem` alone writes it. Downstream skills may read it for context and open questions; nothing in it binds them. Requirements bind only from the doc.

Create it from this template. Keep the section headings and the table columns exactly as they are here.

```markdown
# Scoping record: <title>

Mode: interview
Origin: scope-problem <MM/DD/YY>

## Questions asked

| # | Date | Question | Answer |
| --- | --- | --- | --- |
| F1 | 11/09/26 | What are we trying to change or build? | A rate limiter in front of the public API. |
| 2.3 | 11/09/26 | Is the gap the retry count, or that nobody is told after the third? Default: the second. | 3: yes |

## Coverage

| Row | State | Where it landed |
| --- | --- | --- |
| Scale | settled | NFR1 |
| Operations | collapsed: single process, no external callers | |

## Evidence

| Claim | Source | Label |
| --- | --- | --- |
| The scheduler retries three times. | `scheduler.rs:141` | observed |

## Open questions

| Question | Owner | Default |
| --- | --- | --- |

## Decomposition

| Piece slug | Serves (parent IDs, mapped to piece IDs) | Route | Depends on | Handoff |
| --- | --- | --- | --- | --- |

## Reader test

## Conflicts

| # | Between | Clash | Options | Decision | Date |
| --- | --- | --- | --- | --- | --- |

## Humanizer
```

The rows shown are examples of the format; a new record starts with empty tables. Question numbers are `F1` to `F5` for the frame and `<round>.<number>` after that.

### Header lines

`Mode:` is `interview` or `freeform`. `Origin:` is `scope-problem <date>`, or `investigation <date>` when the record was created from an investigator's draft.

Below those two go the **progress lines**. Each one is added when its phase finishes, in this order, and a resume starts at the first one missing:

| Line | Added when |
| --- | --- |
| `Frame: <date>` | Phase 1 is done: the restatement is confirmed and both files exist. |
| `Grounded: <date>` | Phase 2 is done, even if there was nothing to read. |
| `Exit check: <date>, <not offered, declined or taken>` | The early exit was considered. |
| `Covered: <date>` | Every coverage row has a state. |
| `Decomposed: <date>` | The Decomposition table is confirmed and any piece docs are written. |
| `Reader test: <date>` | The phase 5 read is done and its wording fixes are in. |
| `Swept: <date>` | The phase 6 conflict sweep found nothing left undecided. |
| `Humanized: <date>` | The tighten and humanize passes are done. |
| `Final check: <date>` | The final read is done, or was skipped because nothing changed. |

When work goes back to an earlier phase and the doc changes, that phase's line and every line after it are deleted, so the later checks run again.

### Sections

- **Questions asked:** every question and its answer, logged as soon as it is answered, the frame questions included.
- **Coverage:** one row per line of the coverage list, with its state: `settled`, `defaulted`, `assumed`, or `collapsed` with the reason.
- **Evidence:** one row per claim, written as it is read. The label is `observed`, `inferred` or `assumed`.
- **Open questions:** each with an owner and a default.
- **Decomposition:** one row per piece. Handoff is the command, the concept note's path, or `doc only`.
- **Reader test:** a dated entry for the phase 5 read (what the reader got right, what it missed, what was fixed), then one for each final check.
- **Conflicts:** every conflict put to the user, in the interview or in the sweep, numbered `C1` onward, with their decision.
- **Humanizer:** the date, the sections changed, and the lines kept original.

When an artifact investigation ran, a third file sits beside the doc and the record: `investigation.md`, with its draft requirement doc, evidence, and open questions, one dated section per run. The investigating agent alone writes that file, and `/scope-problem` folds what holds into the doc and the record.
