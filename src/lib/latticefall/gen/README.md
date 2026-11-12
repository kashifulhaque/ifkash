# Generated content types

The files in this folder are generated from the Rust content crate. Don't edit them
by hand. The generated output is committed so the frontend builds without Rust.

- `*.ts`: TypeScript types from `lf-content` (`ts-rs`).
- `digest.json`: the content hash (as a string, because a `u64` doesn't fit a
  JavaScript number), the attribute curves, and the names and IDs of moves and
  weapons for the UI.

To regenerate after changing anything in `game/data/` or `game/crates/content/`, run
the following command from the repository root:

```sh
cd game && cargo run -p lf-tools -- gen-ts
```
