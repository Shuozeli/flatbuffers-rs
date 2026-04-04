# flatc-rs Roadmap

Last updated: 2026-03-26

## Current State

Rust implementation of the FlatBuffers compiler (`flatc`). The core pipeline is complete:
parsing, semantic analysis, compiler orchestration, and code generation for Rust and
TypeScript. 612+ tests passing. Binary compatibility verified against the C++ `flatc` via
cross-compat tests. Production readiness audit completed 2026-02-28. All CRITICAL and HIGH
audit findings resolved. Code quality audit completed 2026-03-17 with all priority-1 and
priority-2 fixes applied.

| Component               | Status   | Tests |
|-------------------------|----------|-------|
| Parser (recursive descent)| Complete | 58  |
| Semantic Analyzer       | Complete | 105   |
| Compiler Orchestration  | Complete | 17    |
| Rust Code Generation    | Mature   | 168   |
| TypeScript Code Gen     | Mature   | 52    |
| Object API (pack/unpack)| Complete | 22    |
| Fuzz/Random Schema Gen  | Complete | 44    |
| Cross-compat (C++ flatc)| Verified | 17    |
| JSON/Binary Conversion  | Complete | 23    |
| Serde Integration       | Complete | 22    |
| Other                   |          | 84    |
| **Total**               |          |**612**|

---

## Phase A: Make It Usable in Build Systems -- COMPLETE

### A1: CLI alignment with C++ flatc + file output (#80)

**Priority:** P0 | **Effort:** Medium | **Status:** DONE

Implemented:
- `-o <dir>` writes generated files to disk (default: cwd)
- `--rust` / `-r` and `--ts` / `-T` (matching C++ flatc flag names)
- `--filename-suffix` (default `_generated`), `--filename-ext`
- `--gen-object-api`, `--gen-name-strings`, `--gen-all`
- `--gen-all` filtering: without flag, only direct input file types generated
- `--file-names-only`, `--root-type`, `--require-explicit-ids`
- `--rust-serialize`, `--rust-module-root-file` (accepted, warn not yet implemented)
- `--no-warnings`, `--warnings-as-errors`, `--version`

**Files changed:** `compiler/src/main.rs`, `codegen/src/lib.rs`,
`codegen/src/rust_gen.rs`, `codegen/src/ts_gen.rs`

---

## Phase B: Correctness Fixes -- COMPLETE

All P0 correctness bugs fixed. 364 tests passing (was 362).

### B1: Vector\<Enum\> codegen -- VERIFIED OK

Investigated: already working correctly. `field.type_.index` IS set on the outer type
for vectors of enums. Output matches C++ flatc.

### B2: Field ID validation -- DONE

Added `validate_field_ids()` to analyzer with three new error variants:
- `DuplicateFieldId` -- two fields share the same `id: N`
- `NonContiguousFieldIds` -- gap in 0..max_id
- `FieldIdOutOfRange` -- id > 65535
Golden tests: `error_duplicate_field_id`, `error_noncontiguous_field_id`.

### B3: Required field enforcement -- VERIFIED OK

Already implemented: `table_gen.rs:611-631` emits `fbb_.required()` calls.
Golden test `table_string_field.expected` covers this.

### B4: unwrap() panics replaced -- DONE

- Added `CodeGenError` enum and `catch_unwind` wrapper so `generate_rust()` /
  `generate_typescript()` return `Result<String, CodeGenError>` (never panic).
- Created `field_type()` / `field_type_index()` helpers with descriptive messages.
- Replaced ~24 bare `.unwrap()` calls across `struct_gen.rs`, `table_gen.rs`,
  `ts_table_gen.rs`, `ts_struct_gen.rs`.

### B5: Single-quoted string escapes -- DONE

Applied `unescape_string()` to single-quoted strings (was skipped). One-line fix
in `parser/src/parser.rs`.

### B6: Enum value overflow -- DONE

Replaced `next_value += 1` with `checked_add(1)` in `assign_enum_values()`.
New error variant `EnumValueOverflow` catches i64 overflow before validation step.

### B7: Array/object default values -- DONE

Non-empty array defaults (e.g. `= [1, 2, 3]`) now emit a parse error.
Empty `= []` is still accepted (matches C++ flatc behavior).

---

## Phase G: Production Readiness Audit Findings

Full audit completed 2026-02-28 across all 6 subsystems: parser, analyzer, Rust codegen,
TypeScript codegen, compiler orchestration, and schema representation.

**Verdict:** All CRITICAL (G1) and HIGH (G2) items resolved. 612 tests passing. Suitable
for production use where schemas are authored by the same team. Many MEDIUM (G3) items
have been fixed (see status below). Remaining open items are edge cases that can be
addressed incrementally.

### G1: CRITICAL -- Crash and Wrong Output

| # | Location | Description | Status |
|---|----------|-------------|--------|
| G1.1 | `struct_layout.rs` | `i32 as usize` unchecked cast panics on negative `Type.index`. | **DONE** -- added `checked_obj_index()` helper, `type_size_align` returns `Result`, new `InvalidTypeIndex` error variant |
| G1.2 | `ts_table_gen.rs:1328` | Missing `)` in TS `for...of` loop. | **DONE** -- one-char fix |
| G1.3 | `schema/src/lib.rs:9-31` | `BaseType` enum discriminants don't match official `reflection.fbs`. | **Resolved** -- `#[repr(u8)]` with explicit discriminants, `to_reflection_byte()` method |
| G1.4 | `schema/src/lib.rs:365-381` | `AdvancedFeatures` modeled as struct instead of bit_flags enum. | **Deferred to Phase F** -- latent, only needed for binary schema output |
| G1.5 | `analyzer.rs` | Struct array element type not validated. `[string:4]` in struct accepted. | **DONE** -- validates element_type in `validate_struct_field_type`, golden test added |

### G2: HIGH -- Silent Wrong Behavior

| # | Location | Description | Status |
|---|----------|-------------|--------|
| G2.1 | `struct_layout.rs` (absent) | `force_align` validated in analyzer but never applied in `compute_struct_layouts()`. Binary-incompatible for any struct using `force_align`. | **DONE** -- `get_force_align()` helper, applied in layout computation |
| G2.2 | `struct_gen.rs` (entire) | `escape_keyword()` exists in `type_map.rs` but never called by struct codegen. Fields named `type`, `match`, `struct` produce invalid Rust. | **DONE** -- wrapped all 9 `to_snake_case()` calls with `escape_keyword()` |
| G2.3 | `ts_type_map.rs:210-269` | `escape_ts_keyword()` defined but never wired up. Fields named `class`, `static` produce invalid TypeScript. | **DONE** -- wrapped all `to_camel_case()`/`to_pascal_case()` calls in `ts_struct_gen.rs` and `ts_table_gen.rs` |
| G2.4 | `compiler.rs:198-223` | Include path traversal. `include "../../etc/passwd"` reads arbitrary files. No sandboxing on `find_include()`. | **DONE** -- reject absolute paths, validate resolved path stays within search root, `PathTraversal`/`AbsoluteIncludePath` errors |
| G2.5 | `ts_table_gen.rs:1157,1482` | TS Union Object API broken. Union fields typed as `null`, `unpack()` returns raw value without type wrapping. Cannot round-trip. | **DONE** -- proper discriminated union types, `unionToXxx` dispatch for unpack |
| G2.6 | `ts_table_gen.rs:1341-1343` | TS vector-of-struct `pack()` not implemented. Falls to catch-all, emits `const field = 0; // TODO`. Silent data loss. | **DONE** -- inline struct packing in reverse order |
| G2.7 | `ts_table_gen.rs:495-509` | TS vector-of-union element accessor not generated. `BASE_TYPE_UNION` falls to `_ => {}`. Only `fieldLength()` emitted. | **DONE** -- `gen_union_vector_accessor()` with `__union` dispatch |
| G2.8 | `parser.rs:82-91` | No `root.has_error()` check after tree-sitter parse. ERROR nodes inside declarations silently dropped. Invalid schemas accepted. | **DONE** -- added `check_children_for_errors()` in type_decl/enum_decl visitors, `SyntaxError` variant, top-level ERROR handling already present |
| G2.9 | `grammar.js:387` / `parser.rs:769-805` | Grammar accepts `\NNN` octal escapes, parser rejects them as `InvalidEscape`. Mismatch. | **DONE** -- full `\0`-`\377` octal escape handling in `unescape_string()` |
| G2.10 | `schema/src/lib.rs` | 8 serde field name mismatches vs official schema (e.g., `element_type` vs `element`, `is_deprecated` vs `deprecated`). Missing `#[serde(rename)]`. Breaks JSON schema compat. | **DONE** -- added 8 `#[serde(rename)]` annotations, golden files regenerated |
| G2.11 | `main.rs:110,185` | `.expect()` on `file_stem()` panics on pathological paths. `.unwrap()` on `serde_json` panics on NaN defaults in `--dump-schema`. | **DONE** -- replaced with proper error handling |
| G2.12 | `enum_gen.rs:57-58` | `bit_flags` shift: `1u64 << bit_pos` panics in debug if `bit_pos >= 64` or negative. No analyzer validation of bit_flags value ranges. | **DONE** -- added `validate_bitflags_values()` with `BitFlagsValueOutOfRange` error |

### G3: MEDIUM -- Edge Cases and Robustness

| # | Location | Description | Status |
|---|----------|-------------|--------|
| G3.1 | `analyzer.rs` | Union field `id: 0` causes collision with companion `_type` field. | **DONE** -- `UnionFieldIdZero` validation error |
| G3.2 | `analyzer.rs` | Empty enums not validated (C++ flatc rejects them). | Open |
| G3.3 | `analyzer.rs` | Union NONE variant position/value not validated. | Open |
| G3.4 | `analyzer.rs` | `key` attribute not validated (single key per table, must be scalar/string, not deprecated). | Open |
| G3.5 | `analyzer.rs` | Invalid `force_align` values silently ignored when `val.parse::<i64>()` fails. | Open |
| G3.6 | `type_index.rs` | No parent namespace walking. | **DONE** -- `TypeIndex.resolve()` walks parent namespaces |
| G3.7 | `struct_layout.rs` | Unbounded recursion in topological sort. | **DONE** -- depth limit (256) + `StructDepthLimitExceeded` error |
| G3.8 | `compiler.rs` | No include depth limit. | **DONE** -- 64-level depth limit, 1000 file limit |
| G3.9 | `table_gen.rs` | Unhandled `BaseType` variants in Rust codegen emit `// TODO` comments instead of errors. | Partially addressed (warnings added) |
| G3.10 | `table_gen.rs` | Vector-of-unknown-type silently generates `Vector<'a, u8>` fallback. | Open |
| G3.11 | `compiler.rs` | Conflicting `file_identifier`/`root_type` across includes: last-one-wins with no warning. | Open |
| G3.12 | `main.rs` | `fs::canonicalize` failure silently drops files from `--gen-all` filter. | Open |
| G3.13 | `main.rs` | Non-atomic file writes. | **DONE** -- uses temp file + rename for atomic output |
| G3.14 | `codegen/*.rs` | ~40 instances of `.unwrap_or(0) as usize`. | **Partially addressed** -- codegen instances fixed; ~27 benign instances remain in decoder/encoder/annotator/data-gen for fields where the analyzer guarantees presence |
| G3.15 | `schema/src/lib.rs` | `root_table` deep clone goes stale after layout computation. | **DONE** -- `root_table_index` added, refreshed after layout |
| G3.16 | `schema/src/lib.rs` | `objects`/`enums` arrays not sorted. Binary schema consumers expect alphabetical order. | Open |
| G3.17 | `ts_type_map.rs` | `panic!()` calls behind `catch_unwind`. | **Partially fixed** -- Rust codegen uses `Result`; TS codegen still uses `catch_unwind` as safety net |
| G3.18 | `parser.rs` | Fixed array length `i64 as u32` silent truncation for values > `u32::MAX`. | Open |
| G3.19 | `parser.rs` | No parse timeout. Pathological input could stall indefinitely. | **DONE** -- 10-second parse timeout added |
| G3.20 | `compiler.rs` | No limit on schema size. Millions of fields cause OOM with no bound. | **DONE** -- 10K objects, 10K enums, 100K fields, 100K enum values, 1K files |
| G3.21 | `struct_gen.rs` | Object API silently skips structs with array fields (documented as deferred). | Open (warning added) |
| G3.22 | `analyzer.rs` | `ulong` enum/default range capped at `i64::MAX` (matches C++ flatc limitation). | Open (by design) |
| G3.23 | `parser.rs` | `_other` catch-all silently ignores unrecognized top-level node types. | Open |
| G3.24 | `grammar.js` | `object` grammar rule passes wrong arg count to `comma_separate`. Effectively broken. | Open |

### G4: Recommended Fix Order

1. ~~**G1.2** (TS for-of syntax)~~ -- **DONE**
2. ~~**G2.2 + G2.3** (keyword escaping)~~ -- **DONE**
3. ~~**G1.1** (bounds-check i32 casts)~~ -- **DONE**
4. ~~**G1.5** (struct array element validation)~~ -- **DONE**
5. ~~**G2.1** (apply force_align)~~ -- **DONE**
6. ~~**G2.8** (has_error check)~~ -- **DONE**
7. ~~**G2.11** (CLI unwraps)~~ -- **DONE**
8. ~~**G2.12** (bit_flags validation)~~ -- **DONE**
9. ~~**G2.5 + G2.6 + G2.7** (TS Object API gaps)~~ -- **DONE**
10. ~~**G2.9** (octal escapes)~~ -- **DONE**
11. ~~**G2.10** (serde renames)~~ -- **DONE**
12. ~~**G2.4** (include path traversal)~~ -- **DONE**
13. ~~**G3.1** (union field ID validation)~~ -- **DONE**
14. ~~**G3.14** (unwrap_or(0) elimination)~~ -- **DONE**

---

## Phase C: Production Rust Features

Features needed for real-world Rust projects to adopt flatc-rs.

### C1: Serde Serialize/Deserialize support (#77)

**Priority:** P1 | **Effort:** Medium | **Status:** DONE

Implemented:
- `--rust-serialize` CLI flag
- Regular enums: manual Serialize/Deserialize using variant names
- Bitflags enums: manual Serialize/Deserialize using numeric bits
- Structs: derived Serialize/Deserialize for Object API, manual Serialize for reader
- Tables (Object API `*T` types): derived Serialize/Deserialize
- Union `*T` enums: derived Serialize/Deserialize with discriminant
- Integrated with `flatbuffers` crate `serialize` feature

**Files:** `enum_gen.rs`, `struct_gen.rs`, `table_gen.rs`, `rust_gen.rs`, `main.rs`

### C2: Native type support (#81)

**Priority:** P1 | **Effort:** Medium | **Status:** Not started

`(native_type: "SomeCppType")` attribute on structs tells codegen to use an external
type instead of generating one. Used for Rust/C++ interop patterns.

**Files:** `struct_gen.rs`, `table_gen.rs`, `analyzer.rs`

---

## Phase D: Error Quality

Make flatc-rs pleasant to use when schemas have errors.

### D1: Error source locations (#P1.4)

**Priority:** P1 | **Effort:** Medium | **Status:** DONE

Implemented:
- `Span { file: Option<String>, line: u32, col: u32 }` added to `Schema` types
- `FbsParser` now extracts position info from parser nodes
- `AnalyzeError` variants include `Span` for precise error reporting
- `IncludeResolver` passes file paths to the parser for multi-file span support

**Files:** `schema/src/lib.rs`, `parser/src/parser.rs`, `compiler/src/error.rs`, `compiler/src/analyzer.rs`

---

## Phase E: TypeScript Gaps -- COMPLETE

### E1: TS vector-of-struct packing in Object API (#82)

**Priority:** P2 | **Effort:** Small | **Status:** DONE (fixed as G2.6)

### E2: TS union field type signatures (#83)

**Priority:** P2 | **Effort:** Small | **Status:** DONE (fixed as G2.5 + G2.7)

---

## Phase F: Advanced Features

### F1: Binary schema (.bfbs) output (#84)

**Priority:** P3 | **Effort:** Large | **Status:** DONE

Implemented in `compiler/src/bfbs.rs`. CLI flag: `-b`/`--schema`. Serializes the
schema as a FlatBuffer using `reflection.fbs`. Supporting flags: `--bfbs-comments`,
`--bfbs-filenames`, `--bfbs-absolute-paths`.

**Prerequisite G1.3 (BaseType discriminants):** Resolved -- `BaseType` now has
`#[repr(u8)]` with explicit discriminants matching official `reflection.fbs` and
a `to_reflection_byte()` method for wire-format serialization.

**Prerequisite G1.4 (AdvancedFeatures):** Deferred -- latent, only needed if advanced
feature flags are used.

### F2: gRPC service stub generation (#85)

**Priority:** P3 | **Effort:** Large | **Status:** DONE (feature-gated)

Implemented in `codegen/src/service_gen.rs`. Compile with `--features grpc` on the
`flatc-rs-codegen` crate. Uses `grpc-codegen` from [pure-grpc-rs](https://github.com/shuozeli/pure-grpc-rs).
Generates server traits and client stubs from `rpc_service` declarations.

### F3: Additional language backends (#86)

**Priority:** P3 | **Effort:** Large per language | **Status:** Not started

Go, C++, Python, Java codegen. Architecture supports it -- each backend is a separate
`*_gen.rs` module under `codegen/src/`.

---

## Execution Order

```
Phase A (CLI) --> Phase B (correctness) --> Phase G (audit fixes: G1-G2 first)
                                        --> Phase C (production features)
                                        --> Phase D (error quality)
                                        --> Phase E (TS gaps, partially covered by G2.5-G2.7)
                                                    |
                                                    v
                                              Phase F (advanced, requires G1.3/G1.4 for .bfbs)
```

Phase G (audit fixes) should be addressed before Phases C-F. G1 (CRITICAL) and G2 (HIGH)
items are prerequisites for production use. G3 (MEDIUM) items can be interleaved with
other phases.
