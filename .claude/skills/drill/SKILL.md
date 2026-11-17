---
name: drill
description: Questions the user on a concept note or a design from this repository, to practise recall. Asks one question at a time about a failure to predict, a number to change, a choice between variants or a decision to defend, grades the user's answer against the doc, and asks one follow-up where the answer was thin. Keeps a private record per subject under drills/, which git ignores, so the next session starts with what the user missed. It writes no note and no design, and never drills from memory when the repository has no doc on the subject. Use when the user asks to be quizzed, drilled or tested on a concept or a design, or wants to practise or check what they remember. Not for explaining a concept or answering a question about one (answer it directly), writing a concept note (use write-concept), or critiquing a design (use review-design).
argument-hint: "[slug of a concept note or a design] [n: number of questions]"
---

# Drill

You question the user on one doc they have already read, a concept note or a design, so they practise recalling it and applying it. You are their coach: you ask, they answer in their own words, you grade against the doc.

This runs in the main session, because the conversation is the work. No subagent is dispatched.

The subject is in the skill's arguments (the `ARGUMENTS:` line) or the conversation.

**If this file was cut short.** A long session can be compacted, and that keeps only the start of this file. If you cannot see the "End the session" section at its end, read this file again from disk before you go on.

## The subject

Take the first row that matches.

| The user gave | Drill from | Record |
|---|---|---|
| A slug with a note at `concepts/<slug>.md` | the note | `drills/<slug>.md` |
| A slug whose `design-doc/<slug>/` holds `rfc.md` or `design.md` | `rfc.md`; without one, `design.md` down to its loop-state marker | `drills/<slug>.md` |
| A concept with no note | nothing | none |
| A slug whose folder holds only `requirement.md` | nothing | none |
| No subject | see below | |

- **The doc is the answer key.** Every question is answerable from the subject's doc, and every answer is graded against it. Never drill from your own memory.
- **No note:** say so, offer `/write-concept <concept>`, and stop. Ask no question and write no record.
- **No design yet:** say so, point the user to `/new-design <slug>`, and stop.
- **An open loop.** A `design.md` with no `rfc.md` beside it is still in review and may change. Say so before the first question.
- **No subject:** list `drills/`. Take the subject whose record holds the oldest `miss`. With no record, list the notes in `concepts/` and the designs in `design-doc/`, and ask which one.
- Read the whole doc before the first question. For a design, also read its `review.md`: an objection the bar-raiser raised and the design answered is a question worth asking.
- You only read `concepts/` and `design-doc/`. You never write there.

## The record

`drills/<slug>.md` is the whole state of the drill. It is private to the user.

```markdown
# Drill record: <slug>

Subject: <repository-relative path of the doc>

Q1 | <MM/DD/YY> | hit | <where the answer is> | <the question> | complete
Q2 | <MM/DD/YY> | miss | <where the answer is> | <the question> | <what was missing>
Q3 | <MM/DD/YY> | open | <where the answer is> | <the question> | -
```

One line per question, numbered in the order first asked. A question asked again keeps its line and its number: only the date, the state and the last field change.

| Field | Holds |
|---|---|
| Date | the day the line last changed |
| State | `open`: asked, not yet graded. `hit`, `partial` or `miss`: its last grade |
| Where the answer is | the doc's section heading, or a requirement ID |
| Question | one line, with no `\|` in it |
| Last field | `-` while open, `complete` for a hit, otherwise what the answer lacked, in one clause |

- **Git must ignore it.** Before any write under `drills/`, read `.gitignore`. Without a line that reads `/drills/`, add one, creating the file if there is none, and tell the user.
- Never commit the record, and never copy a line of it into a file that git tracks.
- The record holds the question and what was missing. It never holds the user's answer.

## Questions

Write a question only after you know where the doc answers it. Each one asks the user to apply the doc:

- **Predict a failure:** a dependency is down or slow, a node leaves, a burst arrives. What happens, and how would an operator see it?
- **Change a number:** ten times the load, half the nodes, a tighter bound. What changes, and what does it now cost?
- **Choose:** two variants, or the design and an alternative it rejected, and a situation. Which one, and why?
- **Defend a decision:** why this and not the alternative, and which requirement would have to change for the alternative to win?
- **Trace:** follow one request or one key through the mechanism, with real values.

Do not ask for a definition or a list. No multiple choice, no hint in the question, and never the answer alongside it.

Where the questions come from:

- **A concept note:** its Check your understanding questions first, each once. Then a question the user left under My notes. Then new ones, mostly from Trade-offs and variants and How it works.
- **A design:** Alternatives Considered, Failure Modes and Observability, Estimates, the requirement rows and the sections that serve them, and the resolved objections in `review.md`.

## A session

A session is five questions, or the number the user gives with `n:`. One question per message.

**Order.** An `open` line first. Then every `miss`, then every `partial`, oldest date first, asked in its recorded wording. Then new questions. A `hit` is asked again only when the user asks for all of them.

**Each question:**

1. Write its line in the record, with today's date and the state `open`. For a recorded question, change its line.
2. Ask it, and say where the session stands ("2 of 5"). Then stop and wait for the answer.
3. Grade the answer against the doc:

   | Grade | The answer |
   |---|---|
   | `hit` | has the outcome and the reason the doc gives, with its conditions |
   | `partial` | has the outcome, and lacks the reason, a condition or a number that the doc gives |
   | `miss` | is wrong, or the user does not know |

4. **A thin answer gets one follow-up.** When the answer would grade `partial`, ask one question that points at the gap without filling it, and wait. Grade the two answers together. Never ask a second follow-up. A `miss` and a `hit` get none.
5. Update the line: the state, and the last field.
6. Tell the user the grade, what was missing in a sentence or two, and the section to read again. Do not recite the whole section. Then go to the next question.

Grade honestly. An answer that is roughly right is `partial`. Do not pad the feedback with praise.

- **The user disputes a grade:** read the section again. When the doc supports them, change the grade and say so.
- **The answer contradicts the doc and may be right:** say that the doc may be wrong, and where. Grade against what is correct, and do not edit the doc. The user fixes a note with `/write-concept <slug>` and questions a design with `/review-design <slug>`.
- **The user says stop:** end the session. The record is already current.

## Resuming

The record outranks the arguments. When `drills/<slug>.md` holds an `open` line:

- The user's message carries an answer: grade it against that question, from step 3.
- It carries none: ask the question again in its recorded wording, with today's date.

Then go on in the session's order. The question count starts again with each session.

## End the session

After the last question, or when the user stops:

- Give the count of hits, partials and misses, and the sections worth reading again.
- Say that the next session starts with what was missed.
- Say that the record is at `drills/<slug>.md` and that git ignores it.

Change no doc and commit nothing.
