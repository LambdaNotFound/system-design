# Concepts

Notes on building blocks and theory: consistent hashing, replication, CAP, LSM trees, and so on. Design docs link here instead of re-explaining them. Claude writes the notes; annotate them freely under **My notes**.

## Index

_None yet. Notes get added as designs need them, each with a line here._

## Note format

One file per concept, `concepts/<slug>.md`, with this outline:

```markdown
# <Concept>

## What it is
Two or three sentences.

## Why it matters
The problem it solves. Used in: [url-shortener](../design-doc/url-shortener/), …

## How it works
The mechanism, with a Mermaid diagram if it helps.

## Trade-offs and variants
When to use it, when not to, and the alternatives.

## In real systems
Where it shows up in production (e.g. Dynamo, Cassandra, Kafka).

## Check your understanding
1. …
2. …
3. …

## My notes
<!-- yours -->

## Sources
- Book chapters, papers, engineering posts.
```
