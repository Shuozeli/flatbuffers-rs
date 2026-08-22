<!-- agent-updated: 2026-08-22T04:42:32Z -->
# flatbuffers-rs

A pure Rust implementation of the [FlatBuffers](https://flatbuffers.dev/) compiler (`flatc`).
Drop-in replacement: same `.fbs` input, same generated code output, same binary wire format.

[![Live Visualizer](https://img.shields.io/badge/Visualizer-fbsviewer.shuozeli.com-blue?style=flat-square)](https://fbsviewer.shuozeli.com/)

## Features

- Full `.fbs` schema parsing via hand-written recursive-descent parser
- 8-step semantic analysis (type resolution, field layout, validation)
- Rust code generation (readers, builders, Object API with pack/unpack)
- Opt-in Rust pluggable buffer readers (`--rust-pluggable-buffer`)
- TypeScript/Node.js code generation (readers, builders, Object API)
- Python model code generation (dataclasses and IntEnum)
- Serde Serialize/Deserialize support (`--rust-serialize`)
- Binary schema (.bfbs) output and JSON/binary conversion
- Schema backwards-compatibility checking (`--conform`)
- Binary annotation (`--annotate`) for debugging FlatBuffer data
- Binary-compatible output verified against C++ `flatc`
- Optional unary gRPC codecs plus server/client stubs via [pure-grpc-rs](https://github.com/Shuozeli/pure-grpc-rs)
- WASM compilation support for browser-based use
- Default and all-feature release-mode test gates, including generated-code and C++ cross-compatibility coverage

## Quick Start

```bash
# Build compiler libraries
cargo build --release --workspace

# Build or install the CLI (CLI dependencies are opt-in)
cargo build --release -p flatc-rs-compiler --features cli --bin flatc
cargo install --path compiler --features cli --bin flatc

# Generate Rust code
cargo run --release -p flatc-rs-compiler --features cli -- --rust -o out/ schema.fbs

# Generate Rust with Object API
cargo run --release -p flatc-rs-compiler --features cli -- --rust --gen-object-api -o out/ schema.fbs

# Generate unary FlatBuffers gRPC codecs and stubs
cargo run --release -p flatc-rs-compiler --features cli,grpc -- \
  --rust --gen-object-api -o out/ schema.fbs

# Generate Rust readers over a pluggable byte-buffer abstraction
cargo run --release -p flatc-rs-compiler --features cli -- \
  --rust --rust-pluggable-buffer -o out/ schema.fbs

# Generate TypeScript
cargo run --release -p flatc-rs-compiler --features cli -- --ts -o out/ schema.fbs

# Generate TypeScript for Node.js projects
cargo run --release -p flatc-rs-compiler --features cli -- --nodejs -o out/ schema.fbs

# Generate Python model code
cargo run --release -p flatc-rs-compiler --features cli -- --python -o out/ schema.fbs

# Generate multiple language outputs in one invocation
cargo run --release -p flatc-rs-compiler --features cli -- \
  --rust --ts --python -o out/ schema.fbs
```

## Cargo Build Scripts

Use `flatc-rs-build` when generated Rust should be produced automatically by
Cargo. The helper discovers all transitive `.fbs` includes, emits precise
`cargo::rerun-if-changed` directives, generates into `OUT_DIR`, and does not
rewrite an output whose contents are unchanged.

```toml
[build-dependencies]
flatc-rs-build = { git = "https://github.com/Shuozeli/flatbuffers-rs.git", rev = "<reviewed-revision>" }
```

```rust
// build.rs
fn main() -> Result<(), Box<dyn std::error::Error>> {
    flatc_rs_build::Builder::new()
        .schema("schemas/game.fbs")
        .include_dir("schemas")
        .gen_all()
        .compile()?;
    Ok(())
}
```

```rust
// src/lib.rs
include!(concat!(env!("OUT_DIR"), "/game_generated.rs"));
```

With unchanged schemas, Cargo skips the build script and `flatc-rs` entirely.
Each direct schema produces `{schema_stem}_generated.rs`; duplicate output
names are rejected instead of being overwritten. Use `gen_all()` for one
self-contained output that includes declarations from transitive schemas, or
register those schemas as direct inputs when they should be separate generated
modules. See [Cargo build integration](docs/cargo-build-integration.md) for the
complete option and incremental-rebuild contract.

## Language Codegen Usage

`flatc-rs` can generate several language targets from the same resolved `.fbs` schema.

| Flag | Output | Runtime dependency | Notes |
|------|--------|--------------------|-------|
| `--rust` / `-r` | `schema_generated.rs` | `flatbuffers` crate | Full FlatBuffers readers, builders, verification, optional Object API, and opt-in pluggable buffer readers with `--rust-pluggable-buffer` |
| `--ts` / `-T` | `schema_generated.ts` | `flatbuffers` npm package | TypeScript readers, builders, Object API, namespaces, unions, vectors, and mutate methods with `--gen-mutable` |
| `--nodejs` | `schema_generated.ts` | `flatbuffers` npm package | Alias for `--ts`; useful when build scripts name the Node.js target explicitly |
| `--python` / `-p` | `schema_generated.py` | Python standard library | Python `dataclass(slots=True)` models and `IntEnum` enums |

The Rust and TypeScript/Node.js backends generate FlatBuffers reader/builder code. The Python backend currently generates typed model code for application and tooling use; it preserves table/struct fields, scalar defaults, optional fields, vectors, namespaces, unions, enum defaults, and keyword-safe names, but it does not include binary encode/decode helpers.

Rust generated code is slice-backed by default for compatibility with upstream `flatbuffers` APIs. Add `--rust-pluggable-buffer` to generate readers over the `flatc-rs-runtime::FlatBufferRead` abstraction, including `root_as_<name>_in(&buffer)` helpers for custom byte providers such as mmap or arena-backed buffers that expose one stable immutable byte sequence through `all_bytes()` and `range()`. Builders still use the upstream `flatbuffers::Allocator` path.

Generated Rust tables use the upstream-style associated constructor
`Type::create(&mut builder, &TypeArgs { ... })`; the former standalone
camel-case `createType(...)` helper is not emitted. Deprecated fields retain
their reader accessors for old-data compatibility, but builders, Args, Object
API, `Debug`, and serde traversal omit them. Together, these rules make
deprecated fields read-only and keep normal public-module consumers
warning-clean without suppressing Rust lints. The coordinated SchemaHub
integration passes its 591-test release workspace with generated-code crates
using `#![deny(warnings)]`, plus all seven real-world usage codelabs.
Union accessors for struct variants also compile without importing
`flatbuffers::Follow`: generated code uses the fully qualified trait and the
public `Table::buf()`/`Table::loc()` API.

The `flatc` binary is gated by the `cli` Cargo feature so compiler-library and
WASM consumers do not compile command-line dependencies. Build or install the
binary with `--features cli`. When `flatc-rs-compiler` is also built with its
`grpc` Cargo feature, Rust output for
`rpc_service` declarations also contains `FlatBufferGrpcMessage` codecs and
pure-grpc server/client modules. RPC messages use the owned Object API `*T`
types, so `--gen-object-api` is required. Unary methods are supported;
server-, client-, and bidirectional-streaming declarations fail generation
with an explicit error until their transport contract is production-tested.
The pure-grpc code generator is pinned to an immutable revision and its
FlatBuffers adapter feature stays disabled, avoiding a dependency back to this
repository.

Output names follow C++ `flatc` conventions: `{input_stem}{suffix}.{ext}`. The default suffix is `_generated`; override it with `--filename-suffix`, and override the extension with `--filename-ext`.

## CLI Flags

| Flag | Description |
|------|-------------|
| `--rust` / `-r` | Generate Rust code |
| `--ts` / `-T` | Generate TypeScript code |
| `--nodejs` | Generate TypeScript code for Node.js projects (alias for `--ts`) |
| `--python` / `-p` | Generate Python model code |
| `-o <dir>` | Output directory (default: cwd) |
| `-I <dir>` | Include search path |
| `--gen-object-api` | Generate Object API (pack/unpack) |
| `--gen-name-strings` | Generate type name constants |
| `--gen-all` | Generate code for all included schemas |
| `--gen-mutable` | Generate mutate methods for scalar fields (TS) |
| `--rust-serialize` | Add serde Serialize/Deserialize derives |
| `--rust-pluggable-buffer` | Generate Rust readers over a replaceable `FlatBufferRead` byte-buffer abstraction |
| `--rust-module-root-file` | Generate `mod.rs` instead of per-file modules |
| `--no-includes` | Don't generate include statements |
| `--no-leak-private-annotation` | Enforce `pub(crate)` for private types |
| `--filename-suffix <s>` | Output filename suffix (default: `_generated`) |
| `--filename-ext <ext>` | Output file extension |
| `--file-names-only` | Print output filenames without writing |
| `--root-type <name>` | Override root type |
| `--require-explicit-ids` | Require `id:` on all table fields |
| `-b` / `--schema` | Generate binary schema (.bfbs) output |
| `-t` / `--json` | Convert FlatBuffer binary to JSON |
| `--conform <file>` | Check backwards compatibility against a base schema |
| `--annotate` | Annotate a binary with schema field names |
| `--dump-schema` | Dump compiled schema as JSON |
| `--no-warnings` | Suppress warnings |
| `--warnings-as-errors` | Treat warnings as errors |

For the full list of flags (including JSON/BFBS options), see [docs/flag-parity.md](docs/flag-parity.md).

## Architecture

```
schema/        Schema type definitions (mirrors reflection.fbs)
parser/        .fbs -> unresolved Schema (hand-written recursive descent)
codegen/       Code generation logic (Rust, TypeScript/Node.js, Python, gRPC)
compiler/      Analyzer, include resolver, JSON/BFBS tools, CLI binary
annotator/     Binary annotation engine (.afb output)
fbs-gen/       Random schema generator for fuzz testing
data-gen/      Random JSON data generator for testing
test-utils/    Shared golden test framework
wasm-api/      WASM bindings for browser-based compilation
grammar/       Tree-sitter grammar for .fbs IDL (editor integration only)
testdata/      Test schemas and expected outputs
```

Dependency chain: `schema -> parser -> compiler`, `schema -> codegen -> compiler`, `schema -> annotator -> compiler`.

## Visualizer

An interactive binary visualizer built on this compiler is available at
[Shuozeli/fbsviewer-lib](https://github.com/Shuozeli/fbsviewer-lib).

**Try it now: [fbsviewer.shuozeli.com](https://fbsviewer.shuozeli.com/)**

## Testing

The workspace commits `Cargo.lock` because it ships the `flatc` binary. CI uses
that lock for both default and all-feature release suites, plus strict default
and all-feature production-target Clippy contracts. The CLI and code-generation
crates are currently installed from source because their pinned infrastructure
dependencies are not published on crates.io. See
[docs/releasing.md](docs/releasing.md) for the public crate boundary and
reproducible install/package commands.

```bash
# Run compiler-core tests without CLI dependencies
CARGO_INCREMENTAL=0 cargo test --release --workspace --locked

# Run the complete suite, including CLI and optional backends
CARGO_INCREMENTAL=0 cargo test --release --workspace --all-features --locked

# Exercise optional gRPC generation and compile an isolated downstream crate
CARGO_INCREMENTAL=0 cargo test --release --locked \
  -p flatc-rs-compiler --features cli,grpc --test grpc_codegen_compile_test

# Regenerate golden files after intentional output changes
UPDATE_GOLDEN=1 CARGO_INCREMENTAL=0 cargo test --release --workspace --locked
```

## License

Apache-2.0
