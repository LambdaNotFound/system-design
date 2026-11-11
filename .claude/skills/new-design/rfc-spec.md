# RFC Spec

## Contents

- File layout: title, header block, sections, loop-state marker
- Length: the word budget and the count command
- Header block
- Sections, in order: what each one holds
- Altitude
- Evidence
- Prose voice
- Brief to RFC: where each part of the requirement doc lands

This file is the contract for the RFC that `/new-design` produces: the
investigator shapes `design.md` to it, and the bar-raiser reviews
against it. The loop calls the signed requirement doc,
`design-doc/<slug>/requirement.md`, the brief. `rfc-exemplar.md`, beside this file,
shows every section in one fictional RFC; read it before drafting.
Where the two differ, this file wins. Where this file is silent,
follow the exemplar. A team with its own RFC template can edit this
file: the loop's agents read it fresh every round.

## File layout

1. Line 1 is the title, `# RFC: <Title>`.
2. The header block.
3. The sections below, in order.
4. A line reading exactly `<!-- loop state below: not part of the RFC -->`,
   then `## Evidence`, `## Revision log`, and `## Objection responses`.

The publishable RFC runs from line 1 down to that marker, and the loop
exports it as `rfc.md`. The loop sections stay in `design.md` as the
audit trail.

## Length

An RFC is a 20-minute read: at most 4,000 words from the header block
to the Appendix, or to References when there is no Appendix, unless
the brief sets `Word budget: <n>` in its header or in an amendment.
Code sketches and the verbatim requirement rows count; Mermaid
blocks, the Appendix, References, and everything below the loop-state
marker do not. Measure it, never estimate:

```sh
awk 'NR==1{next} /^<!-- loop state below: not part of the RFC -->$/ || /^#+ (Appendix|References)/{exit} /^```mermaid/{m=1} m{if(/^```[ \t]*$/)m=0; next} /^\|[-: |]+\|$/{next} {gsub(/\|/," "); n+=NF} END{print n+0}' <loop dir>/design.md
```

Fit the budget by saying each thing once, not by thinning the design:

- State a fact in one section and point to it from the others. The
  Overview previews the components, so a component section doesn't
  preview itself again; Key differences and Error handling don't
  re-derive what a component section showed; the Example walks the
  mechanism without explaining it again.
- No paragraph restates the table or sketch beside it. A rationale is
  one or two sentences. An optional subsection earns its words or
  goes.
- Material a reader consults rather than reads moves to the Appendix:
  a component's second and later sketches, lifecycle tables, full
  config listings, wire layouts, the arithmetic behind a bound, and
  minor variants of a rival, one line each (the steel-man rule binds
  the alternatives in the body). The Appendix carries no prose beyond
  one lead-in line per item, and the body never repeats it.
- Cut repetition before depth: what the riskiest component needs to be
  proven stays in the body.

## Header block

```markdown
**Status:** Draft · **Author:** @<author> · **Related:** [<RFC title>](<url>)

| Name | Sign-off | Comments |
| --- | --- | --- |
| @<reviewer> | - [ ] approved | |
```

The loop writes `Status: Draft`; later values such as `In Review` are
the user's to set. The author is the user (`git config user.name`).
`Related` links the parent RFC named in the brief or its scoping
record; drop it when there is none. Reviewers come from the brief or
the user, otherwise one `@TBD` row. Never invent a name.

## Sections, in order

Top-level sections use `##` and their subsections `###`, or the whole
document shifts up one level. Pick once and hold it. A `---` rule
between top-level sections is optional.

| # | Section | Required |
| --- | --- | --- |
| 1 | Background | yes |
| 2 | Terminologies | when the RFC uses terms a reader needs defined first |
| 3 | Problem Statement | yes |
| 4 | Requirements | yes |
| 5 | Out of Scope | yes |
| 6 | Use cases | optional |
| 7 | Estimates | yes |
| 8 | Proposed Solution | yes |
| 9 | Alternatives Considered | yes |
| 10 | Failure Modes and Observability | yes |
| 11 | Open Questions | when anything is open |
| 12 | Work Breakdown | yes |
| 13 | Appendix | optional |
| 14 | References | yes |

Out of Scope and Use cases may sit inside Requirements as its
subsections or beside it at the same level; the order holds either
way. Omitting an optional section that does not apply is fine.
Omitting a required one, or an optional one this problem leans on, is
a review objection.

**Background.** What exists today and how data or control flows
through it, in present tense, ending where the gap begins. One to
three paragraphs. For a system that does not exist yet, this is the
situation the brief describes: who has the problem and what they do
about it now. When existing options fail in measurably
different ways, a comparison table carries the evidence.

**Terminologies.** A `| Term | Meaning |` table with the term in
bold, holding only the terms a reader needs before the Problem
Statement makes sense.

**Problem Statement.** Why the system in Background cannot proceed
without this change: what is blocked, for whom, and which constraints
make it hard. One or two paragraphs, often closing on the forcing
question ("The forcing question is how to..."). It expands the
brief's Problem Statement and never contradicts it.

**Requirements.** `### Functional Requirements` and
`### Non-functional Requirements`, each a `| ID | Requirement |`
table. The brief's FR and NFR rows carry over verbatim: same IDs,
same wording, same order, their `(assumed)` and `(inferred)` tags
included. The RFC adds no requirement and drops none; a needed change
is a brief correction for the user, never an edit here.

**Out of Scope.** Bullets. Each names the exclusion and either its
reason or where it lives instead: a later task, or a separate RFC or
project, linked. A partial exclusion says what stays in scope and
cites the ID. Every exclusion on the brief's `Out of scope:` line
appears here; the RFC may add one only where no FR or NFR needs the
excluded thing.

**Use cases.** A `**Use cases:**` lead-in line, then one bullet per
concrete flow, tagged with the IDs it exercises.

**Estimates.** Back-of-the-envelope sizing, as bullets. Each one
starts from a requirement row or a cited source, shows every step of
its arithmetic with units, separates peak from average, and ends with
the decision it drives ("so one multi-row insert per event is enough").
Typical subjects are request rate, read to write ratio, storage over
the retention period, and bandwidth. Keep only the numbers that drive
a decision: a number nothing depends on is noise. An input the brief
tags `(assumed)` makes the result assumed, and the bullet says so.

**Proposed Solution.** Opens with an Overview, as a titled subsection
or as untitled opening paragraphs. Its first sentence says what the
RFC adds and its shape ("This RFC adds durable retries to Notifier:
one `deliveries` table that..."), or what the mechanism exists to do.
Then what changes when it ships, then the architecture diagram. The
user draws it after the design is approved, so the investigator
leaves one line that starts `*Diagram:` and says what the drawing
must show: the boxes, the arrows and what flows over them, and the
labels that matter. The loop later replaces that line with the image.
A one-sentence roadmap of the subsections is house style.

Two subsections follow the Overview, before the components:

- **API.** The endpoints or RPCs, each with its key parameters, what a
  retried call does (idempotency), and how a list paginates. Internal
  interfaces belong to their component, not here.
- **Data model and storage.** The access patterns first, since the
  rest follows from them. Then the entities and who owns each, the
  schema with its keys, indexes and partition key, and the storage
  choice with the requirement that forces it.

A request flow that crosses components is a Mermaid `sequenceDiagram`,
placed where the flow is explained.

Then one subsection per component, in data-flow order, each covering:

- Today's behavior when something exists today ("Today's
  `send_webhook` keeps the delivery only in memory..."), the gap, and
  the proposal, with requirement IDs inline ("(FR1)", "FR3
  forbids..."). For a new system, start at the responsibility the
  component holds and the requirement that forces it.
- A sketch wherever an interface, type, or state is proposed: a
  lead-in line ending in a colon ("Sketch of the row and its
  states:"), then a fenced block in the target language, a schema
  language, or `text`. Signatures, fields, enums, and state types
  only, with comments on the semantics a reader cannot infer. No
  function bodies. Use the real types from source where they exist.
- The rationale.
- Tradeoffs, as a `**Pros:**` / `**Cons:**` pair or one
  `**Pros and cons.**` paragraph. The cons are real costs.

Use these where they fit:

- Options inline, then `**MVP choice: <option>.**` and its reason,
  when a sub-decision has rivals. The rejected options also go to
  Alternatives Considered.
- A `Key differences from <predecessor>` subsection with
  `**Difference:**`, `**Proposed change:**`, and `Unchanged:`.
- A `| State | Meaning |` table for anything with a lifecycle.
- An `Error handling` subsection for when the component's own tasks
  fail.
- Named phases or numbered steps for concurrency and algorithms.
- A short `Example` walking the proposal through the motivating case,
  rarely the right thing to omit.

**Alternatives Considered.** Grouped by the component each relates
to, one subsection per component or a bold label per sub-decision.
Each alternative is one bullet:
`**<Name>.** Pro: <its real advantage>. Con: <the specific failure or cost that kills it>.`
The con names a mechanism (a race, a missing dependency, a cost) and
cites the requirement it breaks where one applies; "may not scale"
names nothing. A design this RFC replaces gets a short paragraph
instead: what it was, what it bought, why it lost.

**Failure Modes and Observability.** A lead sentence that counts the
modes ("Three failure modes matter for this design; each below names
the mode, then what to measure for it."), one bullet per mode with
the mode in bold: what fails, what it takes down with it, and how the
system recovers. Every dependency has an answer for slow as well as
for down. Then `Metrics that track:` bullets and, where an operator
needs them, `Debug endpoints that:` bullets. A mode nothing detects
is stated as a gap.

**Open Questions.** Numbered. Each is a bold question, the detail,
then `Current design: assumed <X>.` and what changes if that is
wrong. Every open question in the brief's scoping record that the
design did not close lands here.

**Work Breakdown.** A `| # | Task | Description | Depends on |` table,
one row per task an implementer could finish and test on its own,
riskiest first. Each description ends with the IDs it serves in
parens. Depends on names `Task <n>`, a sibling RFC's task by name, or
an external blocker in bold: `**<system or team>** (external, <why>)`.
`/implement-design` picks the mechanisms worth building from this
table.

**Appendix.** Material too detailed for the body, such as a
wire-format layout in a `text` block.

**References.** Bullets of `[<title>](<url>), <its relationship to this RFC>`,
such as "the parent RFC". Pick one link form per kind of source and
hold it. Code links are permalinks pinned to a commit. A book is
cited by title and chapter, without a link.

## Altitude

An RFC goes as deep as interfaces, types, state machines, concurrency
phases, and algorithm steps, all at sketch level. Function bodies,
line-by-line control flow, and test plans belong in the code and its
PRs. Spend depth where the risk is: the riskiest component gets the
sketches and the phases, and a thin wrapper gets a paragraph.

## Evidence

The body carries no `path:line` citations and no
observed/inferred/assumed labels inline. The evidence lives in the
`## Evidence` table below the marker: `| Claim | Source | Label |`,
one row per load-bearing claim. The source is one of these, the
strongest that applies:

- a requirement ID from the brief (`NFR1`);
- a `path:line` in this repository, or a permalink pinned to a commit;
- a URL with its title and access date (`read 11/10/26`), for official
  documentation, a paper, or upstream source;
- shown arithmetic, naming the rows it starts from;
- a command and its output;
- a book and chapter, or `none yet` with the measurement that would
  settle it.

The label is observed (read or run this session), inferred (it
follows from observed facts or requirement rows by shown steps), or
assumed (everything else, books and memory included). In the body an
inferred or assumed claim still reads as one ("Current design:
assumed..."), never as fact. A number appears only with a source:
required, derived, measured, or cited.

## Prose voice

- Present tense for what exists ("Today's `send_webhook` makes one
  HTTP POST per receiver"), "This RFC adds" or "defines" for the
  proposal, third person throughout. "We" appears only inside a
  walkthrough.
- Contractions are normal: "doesn't", "can't", "it's".
- Claims are bounded by contrast. ", not X" and "rather than" mark
  what the design deliberately is not ("a receiver outage costs
  latency, not events"). They carry scope; keep them.
- A colon introduces the elaboration of what precedes it.
- Bold lead-ins name list items and decisions: `**Replay.**`,
  `**MVP choice: ...**`, `**Pros:**`. They are structure, not
  emphasis.
- A one-sentence roadmap may open the Overview or a large component
  section ("This section covers the three parts in data-flow
  order..."). It orients; it is not filler.
- Requirement IDs appear inline wherever a sentence serves one.
- Identifiers go in backticks.
- Zero em or en dashes.
- No hedging filler, no summary closers, no marketing adjectives.

## Brief to RFC

| Requirement doc (`requirement.md`) | RFC |
| --- | --- |
| Problem Statement | Background and Problem Statement, expanded with what the investigation found |
| The `Out of scope:` line | Out of Scope bullets, each with its reason or destination |
| Functional Requirements | Functional Requirements, verbatim |
| Non-functional Requirements | Non-functional Requirements, verbatim |
| Low-level Design Preferences | Honored in Proposed Solution, each `LLD<n>` cited where it is honored; never a section of its own |
| The scale and growth rows among the NFRs | The inputs to Estimates |
| Scoping record: Open questions | Open Questions, unless the design closed them |
| Scoping record: Evidence | The Evidence table, and References where a reader would follow the link |
| `Depends on:` header line | Work Breakdown dependencies |
