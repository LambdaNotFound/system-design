---
name: implement-design
description: Scaffold a Rust implementation of a design's core mechanism under src/ (crate skeleton, types and failing tests), leaving the core logic as todo!() for the user, then review their implementation. Writes the full implementation only when asked. Use when the user wants to implement or code up a system or concept from this repo.
argument-hint: "[system or concept slug]"
---

# Implement design

The system or concept is in the skill's arguments or the conversation.

## 0. Check first

- Run `cargo --version`. If Rust isn't installed, stop and tell the user to install it with rustup (https://rustup.rs). Don't write Cargo files you can't build.
- Read `design-doc/<slug>/requirement.md` if it exists (`/scope-problem` writes it), and go by its `Route:` line:
  - `code`: the piece has no design doc, and the requirement doc is the basis. That holds for a signed-off doc and for the three-line doc of a `Status: closed` early exit.
  - `design`: build from the RFC that `/new-design` wrote, as below.
  - `decomposed`: the code belongs to the pieces. List the piece slugs from the Decomposition table in `scoping.md` and ask which one to build.
  - `no code`: there is nothing to build. Point the user to the piece's handoff instead.
  - `Status: draft`, whatever the route: suggest finishing `/scope-problem <slug>` first. If the user wants to go ahead anyway, do, and say that the requirements are informal.
- On the `design` route, read `design-doc/<slug>/rfc.md`, the exported RFC. If it doesn't exist, the design loop hasn't closed: suggest `/new-design <slug>` first. Never edit `design.md`, `review.md` or `rfc.md`; they belong to the design loop.
- For a concept, the note (`concepts/<slug>.md`) should be at least through How it works. If it isn't, suggest finishing it first.

## 1. Agree on the scope

Don't build the whole system. Propose the one to three mechanisms where the interesting logic lives, taken from the RFC's Work Breakdown when there is one, for example:

- URL shortener: short-code allocation and collision handling
- Rate limiter: token bucket and sliding-window log
- Chat: per-conversation message ordering and delivery acks

Keep it in-process, with no real network or infrastructure, unless the network behavior is the point. Confirm the scope with the user before writing code.

## 2. Create the workspace (first time only)

If `src/Cargo.toml` doesn't exist, create it:

```toml
[workspace]
resolver = "3"
members = []

[workspace.package]
edition = "2024"
```

Then, from `src/`, run `cargo new --lib <slug>`. Check that `<slug>` was added to `members`, and that the crate's `Cargo.toml` has `edition.workspace = true`.

## 3. Scaffold, kata style

- Public types and signatures that use the design doc's vocabulary. Doc comments state each contract and point to the RFC section by name (`/// See design-doc/<slug>/rfc.md, Retry Scheduler.`), or to the requirement row when the piece has no design doc (`/// See design-doc/<slug>/requirement.md, FR2.`).
- The core logic is `todo!("hint")`. The hint names the key idea without giving the solution away.
- Tests pin down the behavior the doc promises, including the edge cases it calls out: boundaries, concurrency, failure handling. Where a test proves a row of the requirement doc, name the row in its doc comment (`/// Proves FR2.`). They must compile, and fail only at the `todo!()`s.
- Std only by default. Add a dependency only when the mechanism needs it (e.g. `tokio` for async I/O), and say why.
- Check from `src/`: `cargo test -p <slug>` compiles and the tests fail at `todo!()`; `cargo clippy -p <slug> --all-targets` shows nothing beyond warnings caused by the stubs.

## 4. Link it up

- Design: fill in the Code column of the root README's Designs table.
- Concept: add a `**Code:** [src/<slug>](../src/<slug>/)` line under the note's title.
- A `code` piece with no design doc: fill in the Code column of its row in the root README's Designs table.

## 5. Review the user's implementation

When the user says they're done, run the tests, then review like a design reviewer. Does the code keep the doc's invariants? What breaks under concurrency or at the boundaries? Which test is missing? Ask before rewriting anything. Once the tests pass and the review is settled, set the status to **Implemented** in the root README's Designs table.

If the user asks for the solution, write the full implementation, with comments on the non-obvious choices.
