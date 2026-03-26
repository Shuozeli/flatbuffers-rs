# Code Quality Findings

## 1. Duplication

### 1.1 Scalar size/alignment computed in three places -- DONE
- **Location:** `schema/src/lib.rs:75-86` (`BaseType::scalar_byte_size`), `parser/src/base_type.rs:25-52` (`base_type_size`), `compiler/src/analyzer.rs:394-404` (`scalar_size`)
- **Also at:** `codegen/src/rust_table_gen/helpers.rs:11-22` (`scalar_alignment_size`) -- nearly identical match arms
- **Problem:** Four functions compute the same scalar-type-to-byte-size mapping with minor signature differences (`usize` vs `Option<u32>` vs `u32`). Each must be kept in sync manually. A bug fix in one is easy to miss in the others.
- **Fix:** Consolidate into a single canonical method on `BaseType` in the `schema` crate (e.g., `BaseType::byte_size() -> Option<u32>`). Have all call sites delegate to it. The `parser::base_type_size` adds offset-type sizes (STRING=4, VECTOR=4, etc.) so it can wrap the canonical method and add those cases.
- **Resolution:** Added `BaseType::byte_size() -> Option<u32>` as the canonical source covering all types. Rewrote `scalar_byte_size()` to delegate to it. Made `parser::base_type_size`, `compiler::scalar_size`, and `codegen::scalar_alignment_size` all delegate to the canonical method.

### 1.2 `as_legacy` conversion is a large manual mirror of the schema structure -- SKIPPED
- **Location:** `schema/src/resolved.rs:393-526` (`ResolvedSchema::as_legacy`)
- **Problem:** The `as_legacy()` method manually reconstructs every parsed schema type from its resolved counterpart -- 130+ lines of field-by-field copying for `ResolvedType -> Type`, `ResolvedField -> Field`, `ResolvedEnumVal -> EnumVal`, etc. This is the inverse of `try_from_parsed` and is inherently fragile: adding a field to any schema type requires updating both directions. The method is documented as "lossy" and for "backward compatibility."
- **Fix:** Implement `From<&ResolvedField> for Field` (and similarly for other types) as trait impls alongside the existing `try_from_parsed` methods, keeping the two conversions co-located. Better yet, accelerate migration of consumers to accept `ResolvedSchema` directly and deprecate `as_legacy()`.
- **Skipped:** Large architectural refactor that involves migrating consumers and deprecating `as_legacy()`. The inline `ResolvedType` duplication was extracted (see 1.3), which addresses the most actionable part.

### 1.3 `ResolvedType` construction duplicated in `as_legacy` -- DONE
- **Location:** `schema/src/resolved.rs:403-413`, `452-461`, `468-476`
- **Problem:** The pattern to convert `ResolvedType` back to `Type` appears three times in `as_legacy()` (for field types, union types, and underlying types), each with the same 7-field struct literal.
- **Fix:** Extract a `ResolvedType::to_parsed(&self) -> super::Type` helper and call it in all three locations.
- **Resolution:** Added `ResolvedType::to_parsed()` method and replaced all three inline struct literals in `as_legacy()`.

### 1.4 Duplicate `catch_unwind` panic-to-error conversion -- PARTIALLY DONE
- **Location:** `codegen/src/lib.rs:203-216` (`catch_codegen_panic`), `wasm-api/src/lib.rs:5-22` (`catch`)
- **Problem:** Both functions catch panics and extract the message from the payload using identical `downcast_ref::<&str>` / `downcast_ref::<String>` logic.
- **Fix:** Extract a shared `panic_payload_to_string(payload: Box<dyn Any>) -> String` helper in a common location (e.g., the `schema` crate or a new `util` module in the compiler crate).
- **Resolution:** `catch_codegen_panic` was removed entirely from `codegen/src/lib.rs` because the TS codegen now returns `Result` (see 2.1). The wasm-api `catch` function remains as it serves a different purpose (catching panics across the WASM boundary).

## 2. Unsafe Patterns (unwrap/panic in non-test code)

### 2.1 `unwrap()` calls in TypeScript codegen (non-test code) -- DONE
- **Location:** `codegen/src/ts_enum_gen.rs:96`, `codegen/src/ts_enum_gen.rs:144`, `codegen/src/ts_enum_gen.rs:195`, `codegen/src/ts_enum_gen.rs:235`
- **Problem:** Four calls to `union_variant_type_index(val).unwrap()` in non-test production code. If a union variant lacks a type index (e.g., due to an analyzer bug), the program panics instead of returning an error. The Rust codegen (`enum_gen.rs`) correctly propagates these as `Result`, but the TS codegen does not.
- **Fix:** Change `ts_enum_gen::generate()` to return `Result<(), CodeGenError>` and propagate the error with `?`, matching the pattern already used in `enum_gen.rs`.
- **Resolution:** Changed `ts_enum_gen::generate` and all internal functions to return `Result<(), CodeGenError>`. Propagated `Result` through `ts_gen.rs` (`TsGenerator::generate` now returns `Result<String, CodeGenError>`). Removed the `catch_codegen_panic` wrapper from `generate_typescript`.

### 2.2 `writeln!().unwrap()` in fbs-gen -- SKIPPED
- **Location:** `fbs-gen/src/gen_meta.rs:30`, `38`, `45`, `59`, `61`, `110`, `112`, `115`
- **Problem:** Writing to a `String` buffer via `writeln!` cannot fail, so `.unwrap()` is technically safe here. However, the function signature should document this assumption. If the output type ever changes to `io::Write`, these become real panics.
- **Fix:** Low priority. These are acceptable since `write!` to `String` is infallible. Optionally, use the `write!` macro without error handling (Rust guarantees `fmt::Write` for `String` never fails).
- **Skipped:** Low priority as acknowledged in the finding. `fmt::Write` for `String` is guaranteed infallible.

### 2.3 `panic!()` in `scalar_rust_type` and TS type-map functions -- DONE
- **Location:** `codegen/src/type_map.rs:37` (`scalar_rust_type`), `codegen/src/ts_type_map.rs:21`, `38`, `55`, `72`, `89`, `106`, `123`
- **Problem:** Eight functions in the codegen type maps use `panic!()` as their catch-all arm for unrecognized `BaseType` variants. After analyzer validation these should be unreachable, but a panic provides no diagnostic and crashes instead of producing an error.
- **Fix:** Change these functions to return `Result<&'static str, CodeGenError>` and propagate errors. Alternatively, keep the panic but add a comment documenting the invariant (that the analyzer guarantees only valid types reach codegen).
- **Resolution:** Added `# Panics` documentation to all 8 functions explaining the invariant that the analyzer guarantees only valid scalar types reach these call sites.

### 2.4 `eprintln!` warnings in `compiler.rs` swallowed during schema merging -- SKIPPED
- **Location:** `compiler/src/compiler.rs:338-342`, `351-354`, `366-370`
- **Problem:** Conflicting `file_identifier`, `file_extension`, and `root_type` across included files are warned via `eprintln!` but processing continues with a last-wins policy. These warnings are not captured or testable, and cannot be escalated to errors with `--warnings-as-errors` since `merge_schemas` has no access to CLI flags.
- **Fix:** Return warnings as a structured `Vec<Warning>` from `merge_schemas` so the caller can decide to print them or promote them to errors.
- **Skipped:** Medium-priority refactor that touches the compiler API surface. Requires introducing a `Warning` type and changing `merge_schemas` return type. Deferred for a focused PR.

## 3. Dead Code / Unused Crates

### 3.1 `grammar` crate is unused in the workspace -- SKIPPED
- **Location:** `Cargo.toml:10` (workspace member `"grammar"`), `grammar/Cargo.toml`
- **Problem:** The `grammar` crate (tree-sitter FlatBuffers parser) is listed as a workspace member but no other crate in the workspace depends on it. It compiles C code and adds build time.
- **Fix:** If the grammar crate is intended for external consumers or future use, document that in the README. If unused, remove it from the workspace members list.
- **Skipped:** The grammar crate may be intended for external consumers (e.g., editor integration). Removing it requires a product decision, not a code quality fix.

### 3.2 Unused `std::env` import in `main.rs` -- DONE
- **Location:** `compiler/src/main.rs:2`
- **Problem:** `use std::env;` is imported but never used. Line 18 uses `env!("CARGO_PKG_VERSION")` which is a compiler built-in macro, not the `std::env` module.
- **Fix:** Remove `use std::env;`.
- **Resolution:** Removed the unused import.

### 3.3 `#[allow(dead_code)]` suppressing warnings in fbs-gen types -- DONE
- **Location:** `fbs-gen/src/types.rs:27` (`EnumInfo.underlying`), `fbs-gen/src/types.rs:45` (`UnionInfo.variant_tables`)
- **Problem:** Two struct fields are annotated with `#[allow(dead_code)]`, meaning they are populated but never read. This is either truly dead code or a sign that planned functionality was never implemented.
- **Fix:** Either use these fields in the generators (e.g., for validation or smarter generation) or remove them. Suppressing the warning hides the fact that the code is unused.
- **Resolution:** Removed both dead fields (`EnumInfo.underlying` and `UnionInfo.variant_tables`) and their `#[allow(dead_code)]` annotations. Updated construction sites in `gen_enum.rs` and `gen_union.rs`.

## 4. Missing Abstractions

### 4.1 Attribute lookup is repeated everywhere as inline closures -- DONE
- **Location:** `codegen/src/enum_gen.rs:10-15` (`has_attribute`), `codegen/src/rust_table_gen/helpers.rs:266-273` (`has_key_attribute`), `codegen/src/type_map.rs:230-238` (`is_bitflags_enum`), `compiler/src/struct_layout.rs:109-119` (`get_force_align`), `codegen/src/lib.rs:119-132` (`type_visibility`)
- **Problem:** Attribute lookup follows the same pattern every time: navigate `attributes.as_ref() -> entries.iter().find/any(|e| e.key.as_deref() == Some("xxx"))`. This is repeated in at least 5 locations with slight variations. Easy to get wrong, hard to refactor.
- **Fix:** Add helper methods to the `Attributes` type in the `schema` crate: `Attributes::has(&self, key: &str) -> bool` and `Attributes::get(&self, key: &str) -> Option<&str>`. Then `Option<Attributes>` calls become `attrs.as_ref().is_some_and(|a| a.has("key"))`.
- **Resolution:** Added `Attributes::has()` and `Attributes::get()` methods. Updated all 5 call sites to use the new helpers: `has_attribute`, `has_key_attribute`, `is_bitflags_enum`, `get_force_align`, and `type_visibility`.

### 4.2 No structured `new()` constructors -- many types use `Default` with field-by-field assignment -- SKIPPED
- **Location:** `schema/src/lib.rs` -- all 11 schema types (Span, Type, KeyValue, Field, Object, Enum, etc.)
- **Problem:** Every schema type has `fn new() -> Self { Self::default() }` followed by manual field assignment at each call site. This is verbose and error-prone. For example, creating a `Field` requires setting `name`, `type_`, `id` separately.
- **Fix:** Low priority cosmetic issue. The `Default` pattern is intentional for builder-style schema construction during parsing. No action needed unless readability becomes a problem.
- **Skipped:** Low priority as acknowledged in the finding.

## 5. Noise / Redundant Comments

### 5.1 Section divider comments throughout `schema/src/lib.rs` and `resolved.rs` -- SKIPPED
- **Location:** `schema/src/lib.rs` (lines 7-8, 117-119, 138-139, 152-154, 188-190, etc.), `schema/src/resolved.rs` (similar pattern)
- **Problem:** Heavy ASCII-art section dividers (`// ---------------------------------------------------------------------------`) appear before every type definition. In a file that is essentially a flat list of struct definitions, these add visual noise without conveying information beyond what the type name already provides.
- **Fix:** Low priority. Remove the dividers or replace with single-line `// --- Type ---` separators. The doc comments on each type already serve as section headers.
- **Skipped:** Low priority cosmetic preference. The dividers are consistent and do not affect correctness.

## 6. Architectural Concerns

### 6.1 `ResolvedSchema` bypass via `as_legacy()` defeats the purpose of the resolved layer -- SKIPPED
- **Location:** `compiler/src/main.rs:351` (`schema.as_legacy()`), `compiler/src/bfbs.rs` (serialization uses `ResolvedSchema` directly), `codegen/src/lib.rs` (codegen uses `ResolvedSchema` directly)
- **Problem:** The `as_legacy()` method exists to convert back to the parsed `Schema` type for code that hasn't been migrated. It is currently used in `--dump-schema` (main.rs:351). This creates a maintenance burden and defeats the non-optional field guarantees of `ResolvedSchema`.
- **Fix:** Implement serde `Serialize` directly on `ResolvedSchema` (or a dedicated JSON-schema output type) so `--dump-schema` doesn't need the lossy roundtrip.
- **Skipped:** Architectural refactor requiring design work on the serialization format. Deferred.

### 6.2 TypeScript codegen uses panics instead of Results -- DONE
- **Location:** `codegen/src/lib.rs:190-198` (`generate_typescript` wrapped in `catch_codegen_panic`)
- **Problem:** The TS codegen path returns `String` (not `Result`) and relies on `catch_unwind` as a safety net. This masks bugs -- a panic in codegen is caught and converted to `CodeGenError::Internal`, losing the stack trace. The Rust codegen path correctly uses `Result` throughout.
- **Fix:** Refactor the TS codegen functions (`ts_enum_gen::generate`, `ts_table_gen`, `ts_struct_gen`, `ts_gen`) to return `Result<(), CodeGenError>`, matching the Rust codegen convention. Remove the `catch_codegen_panic` wrapper once all paths return `Result`.
- **Resolution:** Converted `ts_enum_gen` and `ts_gen` to return `Result`. Removed `catch_codegen_panic`. `ts_table_gen` and `ts_struct_gen` still use panics internally but these are now documented as analyzer-guaranteed invariants (see 2.3). Full conversion of those modules deferred.

### 6.3 `data-gen` crate uses legacy `Schema` instead of `ResolvedSchema` -- SKIPPED
- **Location:** `data-gen/src/lib.rs:9` (`use flatc_rs_schema::Schema`)
- **Problem:** The `data-gen` crate operates on the legacy `Schema` type where all fields are `Option<T>`. This means every field access requires `.unwrap()` or `.as_ref()?.` checks that should be unnecessary after analysis. The rest of the codebase (codegen, bfbs, annotator, json) has been migrated to `ResolvedSchema`.
- **Fix:** Migrate `data-gen` to accept `ResolvedSchema` and remove the optional-field gymnastics.
- **Skipped:** Significant migration effort. Deferred for a focused PR.

## 7. Potential Correctness Issues

### 7.1 `usize as i32` cast in `as_legacy()` can truncate on 64-bit -- DONE
- **Location:** `schema/src/resolved.rs:496-497` (`c.request_index as i32`, `c.response_index as i32`)
- **Problem:** `ResolvedRpcCall` stores request/response indices as `usize`, but the legacy `RpcCall` type uses `Option<i32>`. The cast `usize as i32` silently truncates if the index exceeds `i32::MAX` (unlikely but undefined behavior in a safety-critical context).
- **Fix:** Use `i32::try_from(c.request_index).expect("index overflow")` or return `Result` to surface the error.
- **Resolution:** Replaced `as i32` casts with `i32::try_from(...).expect("RPC request/response index overflow")`.

### 7.2 `saturating_sub(1)` for union companion field ID can silently produce wrong IDs -- DONE
- **Location:** `compiler/src/analyzer.rs:283` (`type_field.id = Some(uid.saturating_sub(1));`)
- **Problem:** If a union field has `id: 0`, `saturating_sub(1)` produces `id: 0` for the companion `_type` field, causing it to collide with the union field. The validator (`UnionFieldIdZero`) checks for this later, but the insertion code silently creates a broken state.
- **Fix:** Check for `uid == 0` at insertion time and skip companion field insertion (or error immediately) rather than relying on a downstream validator to catch the invalid state.
- **Resolution:** Added early `uid == 0` check in `insert_union_type_fields` that returns `AnalyzeError::UnionFieldIdZero` immediately. Replaced `saturating_sub(1)` with plain `uid - 1` (safe because `uid == 0` is now rejected above). Changed function return type to `Result<()>`.

### 7.3 Unknown top-level tokens silently skipped during parsing -- DONE
- **Location:** `parser/src/parser.rs:110-113`
- **Problem:** Unknown tokens at the top level of a `.fbs` file are silently skipped (`self.advance()`). This means typos in keywords (e.g., `tabel Monster {}`) produce no error -- the entire declaration is just ignored.
- **Fix:** Emit a warning or error for unknown top-level identifiers. At minimum, log a warning so users know their declaration was not parsed.
- **Resolution:** Changed the catch-all arm to return `ParseError::UnexpectedContent` with context "unknown top-level declaration", so typos are now caught at parse time.

## 8. Style / Cosmetic (Low Priority)

### 8.1 Inconsistent error handling between Rust and TS codegen -- DONE
- **Location:** `codegen/src/enum_gen.rs` (returns `Result<(), CodeGenError>`), `codegen/src/ts_enum_gen.rs` (returns `()` or panics)
- **Problem:** The Rust codegen uses `Result` throughout while the TS codegen uses panics. This inconsistency makes the codebase harder to navigate.
- **Fix:** Addressed in finding 6.2 above.
- **Resolution:** See 6.2 -- `ts_enum_gen` and `ts_gen` now return `Result`.

### 8.2 `is_rust_keyword` and `is_ts_keyword` could use a shared keyword-check pattern -- SKIPPED
- **Location:** `codegen/src/type_map.rs:136-149`, `codegen/src/ts_type_map.rs:196-246`
- **Problem:** Both functions use `matches!()` with long lists of keywords. They serve different languages so unification isn't strictly needed, but the `escape_keyword` / `escape_ts_keyword` pattern is duplicated.
- **Fix:** Low priority. The duplication is intentional since the keyword lists differ by language.
- **Skipped:** Low priority as acknowledged in the finding. Language-specific keyword lists are inherently different.
