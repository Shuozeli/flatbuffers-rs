# flatbuffers-rs

A pure Rust implementation of the [FlatBuffers](https://flatbuffers.dev/) compiler (`flatc`).
Drop-in replacement: same `.fbs` input, same generated code output, same binary wire format.

[![Live Visualizer](https://img.shields.io/badge/Visualizer-fbsviewer.shuozeli.com-blue?style=flat-square)](https://fbsviewer.shuozeli.com/)

## Features

- Full `.fbs` schema parsing via hand-written recursive-descent parser
- 8-step semantic analysis (type resolution, field layout, validation)
- Rust code generation (readers, builders, Object API with pack/unpack)
- TypeScript code generation (readers, builders, Object API)
- Serde Serialize/Deserialize support (`--rust-serialize`)
- Binary schema (.bfbs) output and JSON/binary conversion
- Schema backwards-compatibility checking (`--conform`)
- Binary annotation (`--annotate`) for debugging FlatBuffer data
- Binary-compatible output verified against C++ `flatc`
- Optional gRPC service stub generation (compile with `--features grpc`) via [pure-grpc-rs](https://github.com/shuozeli/pure-grpc-rs)
- WASM compilation support for browser-based use
- 612+ tests passing, including cross-compatibility with the C++ implementation

## Quick Start

```bash
# Build
cargo build --workspace

# Generate Rust code
cargo run -- --rust -o out/ schema.fbs

# Generate Rust with Object API
cargo run -- --rust --gen-object-api -o out/ schema.fbs

# Generate TypeScript
cargo run -- --ts -o out/ schema.fbs

# Generate both
cargo run -- --rust --ts -o out/ schema.fbs
```

## CLI Flags

| Flag | Description |
|------|-------------|
| `--rust` / `-r` | Generate Rust code |
| `--ts` / `-T` | Generate TypeScript code |
| `-o <dir>` | Output directory (default: cwd) |
| `-I <dir>` | Include search path |
| `--gen-object-api` | Generate Object API (pack/unpack) |
| `--gen-name-strings` | Generate type name constants |
| `--gen-all` | Generate code for all included schemas |
| `--gen-mutable` | Generate mutate methods for scalar fields (TS) |
| `--rust-serialize` | Add serde Serialize/Deserialize derives |
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
codegen/       Code generation logic (Rust, TypeScript, gRPC)
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

```bash
# Run all tests
cargo test --workspace

# Regenerate golden files after intentional output changes
UPDATE_GOLDEN=1 cargo test --workspace
```

## License

Apache-2.0
