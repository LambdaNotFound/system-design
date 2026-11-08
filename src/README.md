# src

Rust implementations, one crate per system or concept: `src/<slug>/` pairs with `design-doc/<slug>/` or `concepts/<slug>.md`.

The Cargo workspace (`src/Cargo.toml`) is created along with the first crate by `/implement-design <slug>`. Run cargo from this folder:

```sh
cargo test -p <slug>
cargo clippy --all-targets
cargo fmt
```

Each crate implements the core mechanism of its design, not the whole system. Claude writes the skeleton and the failing tests; you fill in the `todo!()`s.
