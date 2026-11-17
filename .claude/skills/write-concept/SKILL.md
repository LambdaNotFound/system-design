---
name: write-concept
description: Writes a concept note, a short sourced explanation of one building block or piece of theory such as consistent hashing, a Bloom filter, quorum replication or LSM trees, saved as concepts/SLUG.md and added to the concepts index. The note says what the concept is, why it matters, how it works, its trade-offs and variants, and where real systems use it, and ends with check-your-understanding questions and a sources list. It reads primary sources online in the session and marks everything it cites from memory. It also revises a note that exists and never touches the user's own annotations. Use when the user asks to write, add, update or revise a concept note, or wants a building block written up to keep in this repository. Not for answering a quick question about a concept (answer it directly), designing a system (use new-design), building a concept as a crate (use implement-design), or drawing a diagram of one (use draw-diagram).
argument-hint: "[concept to write up, or the slug of a note to revise] [sources: urls or paths]"
---

# Write concept

You write one concept note: `concepts/<slug>.md`, a short explanation of one building block or piece of theory that the designs in this repository link to instead of explaining it again. The user studies from it, so a wrong fact in it costs more than a missing one.

You write the note yourself, in the main session. No subagent is dispatched.

The concept is in the skill's arguments (the `ARGUMENTS:` line) or the conversation. If neither names one, ask. Given several concepts, write them one at a time and finish each note before you start the next.

**If this file was cut short.** A long session can be compacted, and that keeps only the start of this file. If you cannot see the "Hand it over" section at its end, read this file again from disk before you go on.

## Supporting file

- [note-spec.md](note-spec.md): the note's layout, what each section holds, how sources are marked, and the prose voice. Read it before you write or edit a note. If it is missing, stop and tell the user where it should be.

## Files

| File | Who writes it |
|---|---|
| `concepts/<slug>.md` | you, except its `## My notes` section, which is the user's |
| `concepts/README.md` | you: one line per note under `## Index` |
| `concepts/<slug>/` | `/implement-design` and `/draw-diagram`. Never you |
| `design-doc/**` | their own skills. Never you |

## Before anything else

1. **Settle the slug:** kebab-case, no number prefix, named for the concept and not for a product (`consistent-hashing`, not `dynamo-ring`).
2. **Look for the note.** Read `concepts/README.md` and list `concepts/`. The concept may be there under another name, or as a section of a wider note. If `concepts/<slug>.md` exists, go to Revising a note.
3. **Check the slug is free.** A new slug must not match a folder in `design-doc/` or `src/`: `/implement-design <slug>` looks there first.
4. **Check the size.** A note covers one mechanism or one idea, small enough for a design to link in a sentence. If the name covers several (caching, replication), name the two or three notes it splits into and ask which to write.

## Research

Work the sources in this order. Never assert what you have not read without marking it.

1. **This repository.** Search `design-doc/` and `src/` for the concept, under its name and its common synonyms. A design that relies on it goes on the note's `Used in:` line, and what that design decided about it is an example worth citing. Read the other notes in `concepts/` so this one links to them instead of repeating them.
2. **What the user named** with `sources:` or in the conversation. Read every one. When one cannot be reached, say so and ask for a paste. Never guess what it says.
3. **Primary sources online,** read this session with the web tools the session has: the paper that introduced the concept, the official documentation of each system you will name under In real systems, and engineering posts by the people who run those systems. Prefer these over a summary of them. Search for the concept, not for this repository: put nothing from the user's designs into a query. Save a long page to a file in a fresh `mktemp -d` directory, never into the repository.
4. **Books and your own memory.** Name the matching chapter of Alex Xu's *System Design Interview*, *Designing Data-Intensive Applications* or a Hello Interview guide when one exists. You have not read them this session, so each is marked `From memory`.

With no web tool, or when a page cannot be reached, go on from memory. Mark what that leaves unread, and say so when you hand the note over.

## Write the note

Read [note-spec.md](note-spec.md), then write `concepts/<slug>.md` to it, every section in the spec's order.

- **Pitch it at the reader.** Follow the coaching style in the repository's `CLAUDE.md` when it has one. Without one, write for an engineer who knows the fundamentals: spend the words on the mechanism, the trade-offs, the failure modes and what changes at scale.
- **Explain in your own words.** Quote at most one sentence from any source. This repository is public.
- **`## My notes` holds only the spec's placeholder.** It is the user's section.
- **Link, don't repeat.** Where the note leans on another concept that has a note, link it: `[quorum](quorum.md)`. Do not link a note that does not exist.
- **Never edit a design to link the note.** `design.md`, `rfc.md`, `review.md` and `requirement.md` have their own writers. The link from the note to the design is yours; the link back is for that design's next amendment.
- Write repository-relative links only, never an absolute path.

Then add the note to the index. In `concepts/README.md`, under `## Index`, add one line in alphabetical order:

```markdown
- [<Concept>](<slug>.md): <what it is, in one clause>
```

Delete the placeholder line that starts `_None yet` when the first note goes in. Without a `concepts/README.md`, create one with a `# Concepts` title and an `## Index` section.

## Check before you hand it over

Read the note once more against the spec, and fix what fails:

1. Every section is there, in order, and My notes holds only the placeholder.
2. Each Check-your-understanding question can be answered from the note's own sections, and none by copying one sentence out of them.
3. Every entry under Sources ends with its mark, and every claim under In real systems cites an entry.
4. Every relative link points to a file that exists.
5. The index has exactly one line for the note.

## Revising a note

`concepts/<slug>.md` exists. The files outrank the arguments: never start a second note for the same concept.

- **`## My notes` is the user's.** From that heading down to the next `## ` heading, change nothing: not a character, not the whitespace. Make every change with `Edit` on the other sections. Never rewrite the whole file.
- **The `Code:` line.** A bold `Code:` line under the title was added by `/implement-design`, and the note is the brief its crate was built from. Keep the line. When How it works or Trade-offs and variants changed, say that the crate may now differ from the note.
- **A note that stops before `## Sources`** was cut off. Finish it from where it stops, keeping what is there, then add its index line if that is missing.
- **The user said what to change:** research that change as above and edit only the sections it touches. A source you read now gets today's date. If What it is changed, bring the index line in step.
- **The user named the note and no change:** say the note exists and what you found that it does not yet cover: a design that uses the concept and is missing from `Used in:`, sources still marked `From memory`, a missing section. Ask which of those to do. Write nothing until they answer.

## Hand it over

Give the note's path, and say in one sentence what it covers. Then:

- Which sources you read this session, which are cited from memory, and any you could not reach.
- The designs that use the concept but do not link the note.
- The concepts the note leans on that have no note yet.
- The questions under Check your understanding are for the user to work. Do not answer them in your reply.

Offer to commit the note and `concepts/README.md`. Commit only when the user says so.
