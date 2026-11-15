# src

Rust implementations, one crate per system or concept: `src/<slug>/` pairs with `design-doc/<slug>/` or `concepts/<slug>.md`.

The Cargo workspace (`src/Cargo.toml`) is created along with the first crate by `/implement-design <slug>`. Run cargo from this folder:

```sh
cargo test -p <slug>
cargo clippy --all-targets
cargo fmt
```

Each crate implements the core mechanism of its design, not the whole system. `/implement-design` builds it in two reviewed stages: the tests first, in `src/<slug>/tests/`, then the logic. The plan, the check records and the review ledger are beside the design, in `design-doc/<slug>/`, or in `concepts/<slug>/` for a concept. The tests are the place to start reading: each names the requirement it proves.
