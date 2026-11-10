# CLAUDE.md

System-design learning notes: design docs, concept notes, and small Rust implementations.

- `design-doc/<slug>/` holds one system: `requirement.md` (signed-off requirements), `scoping.md` (its working record), `README.md` (the design doc).
- `concepts/<slug>.md` are notes on building blocks. `src/<slug>/` is one Rust crate per system.
- Each system has one kebab-case slug, used unchanged in folder names. No number prefixes.
- Dates are `MM/DD/YY`.
- Commit only when the user asks. Messages read `MM/DD/YY: <slug>: <what changed>`.
