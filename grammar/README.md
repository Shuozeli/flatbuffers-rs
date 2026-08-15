# FlatBuffers Tree-sitter grammar

This directory contains the Tree-sitter grammar and Rust language binding for
FlatBuffers schema (`.fbs`) files.

Install the pinned CLI dependency and regenerate the checked-in parser files:

```sh
pnpm --dir grammar install --frozen-lockfile
pnpm --dir grammar generate
```

Run the corpus tests and the Rust binding test from the repository root:

```sh
pnpm --dir grammar test
cargo test -p flatc-rs-grammar --locked
```

Commit changes to `src/parser.c`, `src/grammar.json`, and
`src/node-types.json` whenever `grammar.js` changes.
