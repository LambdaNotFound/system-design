# Phase 4: Decompose

Read at the start of phase 4. When the Decomposition table is confirmed and any piece docs are written, SKILL.md says which progress line to add.

Break the problem into independently deliverable pieces sized for their route. The routes and their handoffs are fixed in this repo:

| Route | For | Handoff |
| --- | --- | --- |
| `design` | A piece that needs a reviewed design | `/new-design <piece slug>` |
| `code` | A well-understood mechanism that needs no design doc of its own | `/implement-design <piece slug>` |
| `no code` | A notes or process outcome | A concept note, `concepts/<piece slug>.md`, or `doc only` |

The usual shape is one design piece. `/implement-design` later picks the mechanisms worth building out of the finished design, so do not split those out as code pieces here. Split into several design pieces only when parts can be designed and reviewed independently. Order by dependency and risk, riskiest first, so a failed assumption surfaces early.

Confirm the pieces, routes and handoffs with the user: they are cheap to fix now and expensive after sign-off. Then write the record's Decomposition table. A piece without a slug, a route, and a handoff is not decomposed yet.

- **One piece:** its slug is the bare `<slug>`. Set the doc's `Route:` to its route.
- **Several pieces:** each piece is a sub-system in its own right, with the slug `<slug>-<piece>`. A piece slug must not match a slug already used in `design-doc/`, `concepts/` or `src/`. Set the parent doc's `Route:` to `decomposed`, and write each piece's doc now, `design-doc/<slug>-<piece>/requirement.md`, with the piece-doc header lines the spec defines and the rows it serves, numbered from 1. A piece doc stands alone: an agent downstream never opens the parent to understand its job. If the decomposition changes later, remove the docs of pieces that no longer exist.
