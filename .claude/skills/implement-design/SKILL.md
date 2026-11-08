---
name: implement-design
description: Scaffold a Rust implementation of a design's core mechanism under src/ (crate skeleton, types and failing tests), leaving the core logic as todo!() for the user, then review their implementation. Writes the full implementation only when asked. Use when the user wants to implement or code up a system or concept from this repo.
argument-hint: "[system or concept slug]"
---

# Implement design

The system or concept is in the skill's arguments or the conversation.

## 0. Check first

- Run `cargo --version`. If Rust isn't installed, stop and tell the user to install it with rustup (https://rustup.rs). Don't write Cargo files you can't build.
- The design doc (`design-doc/<slug>/README.md`) or concept note (`concepts/<slug>.md`) should be at least through its high-level design. If it isn't, suggest finishing it first.

## 1. Agree on the scope

Don't build the whole system. Propose the one to three mechanisms where the interesting logic lives, for example:

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

- Public types and signatures that use the design doc's vocabulary. Doc comments state each contract and point to the doc section (`/// See design-doc/<slug>/README.md §6.`).
- The core logic is `todo!("hint")`. The hint names the key idea without giving the solution away.
- Tests pin down the behavior the doc promises, including the edge cases it calls out: boundaries, concurrency, failure handling. They must compile, and fail only at the `todo!()`s.
- Std only by default. Add a dependency only when the mechanism needs it (e.g. `tokio` for async I/O), and say why.
- Check from `src/`: `cargo test -p <slug>` compiles and the tests fail at `todo!()`; `cargo clippy -p <slug> --all-targets` shows nothing beyond warnings caused by the stubs.

## 4. Link it up

- Design: in the doc header, set `**Code:** [src/<slug>](../../src/<slug>/)`, and fill in the Code column of the root README's Designs table.
- Concept: add a `**Code:** [src/<slug>](../src/<slug>/)` line under the note's title.

## 5. Review the user's implementation

When the user says they're done, run the tests, then review like a design reviewer. Does the code keep the doc's invariants? What breaks under concurrency or at the boundaries? Which test is missing? Ask before rewriting anything. Once the tests pass and the review is settled, set the design's status to **Implemented** in the doc header and the README.

If the user asks for the solution, write the full implementation, with comments on the non-obvious choices.
