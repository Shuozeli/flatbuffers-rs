# flatbuffers-rs

A pure Rust implementation of the [FlatBuffers](https://flatbuffers.dev/) compiler (`flatc`).
Drop-in replacement: same `.fbs` input, same generated code output, same binary wire format.

[![Live Visualizer](https://img.shields.io/badge/Visualizer-fbsviewer.shuozeli.com-blue?style=flat-square)](https://fbsviewer.shuozeli.com/)

## Features

- Full `.fbs` schema parsing via tree-sitter (incremental, error-tolerant)
- 8-step semantic analysis (type resolution, field layout, validation)
- Rust code generation (readers, builders, Object API with pack/unpack)
- TypeScript code generation (readers, builders, Object API)
- Serde Serialize/Deserialize support (`--rust-serialize`)
- Binary-compatible output verified against C++ `flatc`
- 550+ tests passing, including cross-compatibility with the C++ implementation

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
| `--gen-object-api` | Generate Object API (pack/unpack) |
| `--gen-name-strings` | Generate type name constants |
| `--gen-all` | Generate code for all included schemas |
| `--rust-serialize` | Add serde Serialize/Deserialize derives |
| `--rust-module-root-file` | Generate `mod.rs` instead of per-file modules |
| `--filename-suffix <s>` | Output filename suffix (default: `_generated`) |
| `--filename-ext <ext>` | Output file extension |
| `--file-names-only` | Print output filenames without writing |
| `--root-type <name>` | Override root type |
| `--require-explicit-ids` | Require `id:` on all table fields |
| `--dump-schema` | Dump compiled schema as JSON |
| `-I <dir>` | Include search path |
| `--no-warnings` | Suppress warnings |
| `--warnings-as-errors` | Treat warnings as errors |

## Architecture

```
grammar/       Tree-sitter grammar for .fbs IDL
schema/        Protobuf-based schema types (mirrors reflection.fbs)
parser/        .fbs -> unresolved Schema (tree-sitter based)
codegen/       Code generation logic (Rust, TypeScript)
compiler/      Analyzer + codegen CLI
test-utils/    Shared golden test framework
testdata/      Test schemas and expected outputs
```

Dependency chain: `grammar -> parser -> compiler`, with `schema` shared by `parser` and `compiler`.

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
