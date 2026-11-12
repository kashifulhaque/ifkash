# Latticefall game workspace

A Cargo workspace, separate from `api/`. Run everything through `mise`:

- `mise run game:test`: run `cargo test --workspace`.
- `mise run game:lint`: run clippy with `-D warnings`, then `cargo fmt --check`.
- `mise run game:validate`: run `lf-tools validate`.
- `mise run game:bench`: run `lf-tools bench`.

The `game:*` tasks set `RUSTUP_TOOLCHAIN` to the version in `rust-toolchain.toml`.
See `mise.toml` for why.
