# Distribution and releases

The workspace has two crates.io packages:

| Crate | Purpose |
|---|---|
| `flatc-rs-schema` | Shared FlatBuffers schema model |
| `flatc-rs-runtime` | Runtime traits and adapters used by generated Rust readers |

Both crates have no first-party path dependencies, so their package archives
can be built and verified before a release:

```sh
cargo package --locked -p flatc-rs-runtime
cargo package --locked -p flatc-rs-schema
```

All other workspace crates are explicitly source-only with `publish = false`.
The compiler and code generators depend on `codegen-infra` crates that are not
available on crates.io. Those git dependencies use immutable revisions, and
`Cargo.lock` is committed because the repository ships the `flatc` binary.
The optional `pure-grpc-rs` dependency is also revision-pinned; locked builds
therefore retain exact commits for its transitive git dependencies.

Install the CLI directly from a reviewed repository revision:

```sh
cargo install \
  --git https://github.com/Shuozeli/flatbuffers-rs.git \
  --rev <flatbuffers-rs-commit> \
  --locked \
  flatc-rs-compiler
```

The WASM crate is delivered as a generated JavaScript/WASM package rather than
a crates.io library. See [`wasm-api/README.md`](../wasm-api/README.md) for its
locked release build and Node verification commands.

Before publishing either public crate, run the full workspace CI suite and
confirm the package job passes from a clean checkout. Publish using the same
workspace version, then tag the exact tested commit.
