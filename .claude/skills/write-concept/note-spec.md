# Concept note spec

The layout and rules of a concept note, `concepts/<slug>.md`. `/write-concept` writes every note to this.

## Template

```markdown
# <Concept>

## What it is
Two or three sentences.

## Why it matters
The problem it solves.

Used in: [url-shortener](../design-doc/url-shortener/), …

## How it works
The mechanism, with a Mermaid diagram if it helps.

## Trade-offs and variants
When to use it, when not to, and the alternatives.

## In real systems
- **<System>:** what it does with the concept [1].

## Check your understanding
1. …
2. …
3. …

## My notes
<!-- yours -->

## Sources
1. [<Title>](<url>), <author or publisher>, <year>. Read <MM/DD/YY>.
2. *<Book>*, ch. <n>, "<chapter title>". From memory.
```

The section headings are fixed, in this order. `/implement-design` adds a `**Code:**` line under the title when the concept has a crate; nothing else goes there.

## Sections

- **What it is.** Two or three sentences that a reader could repeat: what the thing is and the one problem it exists for. No history.
- **Why it matters.** What goes wrong without it, as a concrete failure: a number, an outage, a cost. Then the `Used in:` line, on its own paragraph. It links every design in this repository that relies on the concept, by folder. With none, it reads `Used in: no design yet.`
- **How it works.** The mechanism, step by step, then one worked example with real values: 4 nodes and 8 keys, a 10-bit filter with 3 hash functions. Add a Mermaid block only when a picture carries the mechanism better than a paragraph does. For a larger drawing the user runs `/draw-diagram <slug>`.
- **Trade-offs and variants.** This section carries the most weight. Say what the concept costs, how it fails, and what changes at ten times the load. Name each variant with the case that picks it over the others. Use a table when three or more variants are compared on the same axes.
- **In real systems.** Two to four bullets. Each names a system, says what it does with the concept and where it departs from the textbook version, and cites a source by number. A claim you cannot source is left out.
- **Check your understanding.** Three numbered questions and no answers. Each asks the reader to apply the note: a failure to predict, a number to change, a choice between two variants. Each can be answered from the note's own sections, so the note is the answer key. None can be answered by copying one sentence.
- **My notes.** The line `<!-- yours -->` and nothing else. The section is the user's from then on.
- **Sources.** A numbered list. The text cites an entry as `[n]` wherever it takes a number, a definition or a claim about a real system from it.

## Source marks

Every entry under Sources ends with exactly one mark:

| Mark | When |
|---|---|
| `Read <MM/DD/YY>.` | You read the source itself in this session, on that date. |
| `From memory.` | You did not. This covers every book, and any page you could not reach. |

- A page gets its title as a link, its author or publisher, and its year.
- A book gets its title, the chapter number and the chapter title, so the user can find it on the shelf.
- A file in this repository is cited by its repository-relative path, as a link.
- Never mark a source `Read` on the strength of a search result's snippet, or of a page that quotes it.

## Length

A note is a ten-minute read: about 800 to 1,200 words from the title to `## My notes`. A draft well past that is usually two concepts, so split it and link the two.

## Prose voice

- Present tense, third person. No "we", no "you", no "let's".
- Plain, direct sentences. Say what the thing does, with the number where there is one.
- Contractions are normal: "doesn't", "can't", "it's".
- Identifiers, commands and literal values go in backticks.
- A bold lead-in may name a list item: `**Cassandra:**`. Bold is not used for emphasis.
- Zero em or en dashes.
- If the session has a skill named `humanizer`, load it and apply it to the prose you wrote, as neutral technical writing with no added opinions or personality. Where it and this list disagree, this list wins. Check after the pass that every number, source mark and `[n]` citation survived.
