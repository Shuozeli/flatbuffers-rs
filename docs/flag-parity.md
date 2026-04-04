# CLI Flag Parity: C++ flatc vs Rust flatc

Last updated: 2026-03-26

This document tracks every C++ `flatc` flag (excluding language backends we don't
plan to support) and its status in the Rust implementation. Language-specific flags
for unsupported languages (C++, Java, C#, Go, Python, Dart, Kotlin, Swift, Lobster,
Lua, Nim, PHP) are excluded entirely.

## Legend

- **DONE**: Fully implemented
- **STUB**: Accepted but not yet wired up (warns/ignores)
- **TODO**: Not yet implemented
- **N/A**: Not applicable to Rust/TS (language-specific to unsupported backends)
- **WONT**: Deliberately not planned

---

## Core Flags

| Flag | Status | Notes |
|------|--------|-------|
| `-o PATH` | DONE | Output directory prefix |
| `-I PATH` | DONE | Include search paths |
| `-r, --rust` | DONE | Rust codegen |
| `-T, --ts` | DONE | TypeScript codegen |
| `-b, --schema` | DONE | BFBS binary schema output |
| `-t, --json` | DONE | Binary -> JSON conversion |
| `--` (data files) | DONE | Data file separator for JSON/binary conversion |
| `--root-type T` | DONE | Override root_type |
| `--gen-all` | DONE | Generate for all included files |
| `--gen-object-api` | DONE | Object API (owned types with pack/unpack) |
| `--gen-name-strings` | DONE | Type name constants |
| `--filename-suffix` | DONE | Output filename suffix (default `_generated`) |
| `--filename-ext` | DONE | Output filename extension override |
| `--file-names-only` | DONE | Print output filenames without writing |
| `--no-warnings` | DONE | Suppress warnings |
| `--warnings-as-errors` | DONE | Treat warnings as errors |
| `-h, --help` | DONE | Help text |
| `--version` | DONE | Version number |
| `--dump-schema` | DONE | **Rust-only extension**: dump resolved schema as JSON |

## Rust-Specific Flags

| Flag | Status | Notes |
|------|--------|-------|
| `--rust-serialize` | DONE | `serde::Serialize` on generated types |
| `--rust-module-root-file` | STUB | Accepted, warns not implemented |
| `--require-explicit-ids` | STUB | Accepted, warns not implemented |
| `--no-leak-private-annotation` | DONE | Controls `pub` vs `pub(crate)` in Rust codegen. Types with `(private)` attribute get `pub(crate)` visibility when this flag is set. |

## TypeScript-Specific Flags

| Flag | Status | Notes |
|------|--------|-------|
| `--gen-mutable` | DONE | Generate `mutate_*` methods for scalar fields in TS. Gated behind flag; off by default. |

## JSON / Binary Conversion Flags

These flags control the `flatc -t` (binary -> JSON) and `flatc -b ... -- data.json`
(JSON -> binary) pipelines. They do NOT affect code generation.

| Flag | Status | What it does |
|------|--------|-------------|
| `--strict-json` | DONE | Quoted field names, no trailing commas. Always true with serde_json. |
| `--defaults-json` | DONE | Output fields with default values in JSON. |
| `--force-defaults` | DONE | When encoding JSON -> binary, emit fields even when they equal the default. Accepted as CLI flag; our encoder already writes all JSON-present fields. |
| `--unknown-json` | DONE | When parsing JSON input, silently skip fields not in the schema instead of erroring. Default is strict (error on unknown); flag enables lenient mode. |
| `--natural-utf8` | TODO | Output UTF-8 strings as-is instead of escaping to `\uXXXX`. serde_json already outputs natural UTF-8 by default, so this may already be our behavior. **Priority: LOW** -- may be a no-op for us. |
| `--allow-non-utf8` | TODO | Allow non-UTF-8 byte sequences in string fields during parsing; emit `\xNN` escapes. For legacy data with broken encoding. **Priority: LOW** -- niche. |
| `--json-nested-bytes` | TODO | Allow `nested_flatbuffer` fields to be parsed as a JSON byte array `[1,2,3,...]` instead of the nested schema's object representation. **Priority: LOW** -- backwards compat for legacy JSON formats. |
| `--raw-binary` | TODO | Skip file_identifier check when reading binary input. Allows reading binaries that lack an identifier, at the risk of crashes on mismatched schemas. **Priority: LOW** -- debugging aid. |
| `--size-prefixed` | DONE | Treat input binary files as size-prefixed (4-byte length header before the flatbuffer). Skips 4 bytes before reading root offset. |
| `--flexbuffers` | WONT | Use FlexBuffers (schema-less) format instead of FlatBuffers. Completely different binary format and parser. Not in scope for this compiler. |

## BFBS (Binary Schema) Flags

These control details of the `--schema` / `-b` output.

| Flag | Status | What it does |
|------|--------|-------------|
| `--bfbs-filenames PATH` | DONE | Set root path for computing relative filenames in BFBS `declaration_file` fields. Strips the given prefix from absolute paths. |
| `--bfbs-absolute-paths` | DONE | Use absolute paths instead of relative paths in BFBS output. Modifier for `--bfbs-filenames`. |
| `--bfbs-comments` | DONE | Include doc comments in BFBS output. Accepted as CLI flag; our BFBS serializer always includes documentation fields. |
| `--bfbs-builtins` | TODO | Include built-in attributes (`id`, `deprecated`, `required`, etc.) in the BFBS attribute list, not just user-defined ones. **Priority: LOW** -- niche introspection. |
| `--bfbs-gen-embed` | N/A | Generate a C++ header with the BFBS as an embedded byte array. C++ specific. |

## Code Generation Options

| Flag | Status | What it does | Relevance |
|------|--------|-------------|-----------|
| `--no-includes` | DONE | Don't generate `use super::*;` statements in Rust codegen. | Rust |
| `--include-prefix PATH` | TODO | Prefix added to generated import paths. C++ flatc's Rust generator prepends this to `use crate::...` module paths. **Priority: LOW** -- monorepo concern. | Rust |
| `--gen-mutable` | DONE | (See TypeScript section above) | TS only |
| `--gen-onefile` | N/A | Force single output file. Rust/TS already produce one file per schema. | -- |
| `--gen-compare` | N/A | Generate `operator==`. Rust already derives `PartialEq` on Object API types. | -- |
| `--gen-nullable` | N/A | Add nullability annotations. Rust uses `Option<>` natively. | -- |
| `--gen-json-emit` | N/A | Generate JSON encoding in types. Swift-only feature. | -- |
| `--keep-prefix` | N/A | Keep include path prefix. C++ specific behavior. | -- |
| `--force-empty` | N/A | Force empty strings/vectors in Object API Pack. C++ Object API specific. | -- |
| `--force-empty-vectors` | N/A | Same as above, vectors only. | -- |
| `--reflect-types` | N/A | Compile-time type reflection tables. C++ specific. | -- |
| `--reflect-names` | N/A | Type reflection with names. C++ specific. | -- |
| `-M` | TODO | Print Makefile dependency rules instead of generating code. Language-agnostic. **Priority: LOW** -- build system integration. | Generic |

## Schema Conformance / Validation

| Flag | Status | What it does |
|------|--------|-------------|
| `--conform FILE` | DONE | Check that the input schema is a backwards-compatible evolution of a base schema. Validates: field IDs unchanged, defaults unchanged, types unchanged, no field deletions, enum values stable. |
| `--conform-includes PATH` | DONE | Include search paths for the `--conform` base schema. Falls back to `-I` paths if not specified. |
| `--annotate` | DONE | Annotate binary files with human-readable field names/types using a schema. Produces `.afb` text dump showing each byte's meaning. Usage: `flatc --annotate schema.fbs -- data.bin` |
| `--annotate-sparse-vectors` | TODO | When annotating, skip individual vector elements. Modifier for `--annotate`. |

## Enum Flags (C++ specific, N/A)

| Flag | Status | Notes |
|------|--------|-------|
| `--no-prefix` | N/A | Don't prefix enum values with type name. Rust/TS use scoped enums natively. |
| `--scoped-enums` | N/A | Use `enum class` in C++. Rust/TS always scoped. |
| `--no-emit-min-max-enum-values` | N/A | Skip MIN/MAX enum sentinels. C++ specific. |
| `--object-prefix PREFIX` | N/A | Object API naming prefix. Hardcoded in Rust (`""`). |

## gRPC

gRPC service stub generation is enabled via the `features = ["grpc"]` Cargo feature on the `flatc-rs-codegen` crate (compile-time), not a runtime CLI flag. It uses `grpc-codegen` from [pure-grpc-rs](https://github.com/shuozeli/pure-grpc-rs) to generate server traits and client stubs from `rpc_service` declarations. This is intentionally not a CLI flag since it requires a compile-time dependency.

## Proto Conversion

| Flag | Status | Notes |
|------|--------|-------|
| `--proto` | TODO | Convert `.proto` to `.fbs`. Placeholder exists at `proto2fbs/`. |
| `--oneof-union` | TODO | Map proto `oneof` to FlatBuffer unions during conversion. |
| `--keep-proto-id` | TODO | Preserve protobuf field IDs in generated `.fbs`. |
| `--proto-id-gap` | TODO | Action on gaps between protobuf field IDs (nop/warn/error). |
| `--proto-namespace-suffix` | TODO | Namespace suffix for proto-to-fbs conversion. |

---

## Priority Summary

### Must Have (HIGH) -- ALL DONE

| Flag | Category | Status |
|------|----------|--------|
| `--no-leak-private-annotation` | Rust codegen | DONE |
| `--conform` | Validation | DONE |

### Should Have (MEDIUM) -- ALL DONE

| Flag | Category | Status |
|------|----------|--------|
| `--force-defaults` | JSON/Binary | DONE |
| `--unknown-json` | JSON/Binary | DONE |
| `--size-prefixed` | JSON/Binary | DONE |
| `--no-includes` | Rust codegen | DONE |
| `--gen-mutable` | TS codegen | DONE |
| `--bfbs-filenames` | BFBS | DONE |
| `--bfbs-comments` | BFBS | DONE |

### Nice to Have (LOW)

Everything else -- niche, debugging aids, or covered by existing tools (visualizer).
