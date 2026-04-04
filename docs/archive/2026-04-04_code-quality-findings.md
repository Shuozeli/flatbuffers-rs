# Code Quality Findings

*Last audit: 2026-04-04* (Phase 1 pipeline audit)

## 1. Duplication

### 1.1 Scalar size/alignment computed in three places -- DONE (previous audit)
- **Resolution:** Added `BaseType::byte_size() -> Option<u32>` as the canonical source. All call sites delegate to it.

### 1.2 `as_legacy` conversion is a large manual mirror -- SKIPPED (previous audit)
- **Reason:** Architectural refactor, deferred.

### 1.3 `ResolvedType` construction duplicated in `as_legacy` -- DONE (previous audit)
- **Resolution:** Added `ResolvedType::to_parsed()` method.

### 1.4 Duplicate `catch_unwind` panic-to-error conversion -- PARTIALLY DONE (previous audit)
- **Resolution:** `catch_codegen_panic` removed from codegen. WASM `catch` remains for boundary safety.

### 1.5 Triplicated `object_index` HashMap construction -- DONE (previous audit)
- **Location:** `annotator/src/binary_walker.rs:41-50`, `compiler/src/json/decoder.rs:71-80`, `compiler/src/json/encoder.rs:62-71`
- **Problem:** Three places build an identical `HashMap<&str, usize>` mapping object names (both FQN and short names) to schema indices. The loop body is copy-pasted verbatim in all three.
- **Fix:** Extract a shared `build_object_index` function in the `schema` crate's `resolved` module.
- **Resolution:** Added `ResolvedSchema::build_object_index()` method. All three call sites now delegate to it.

### 1.6 Trivial `get_base_type` wrapper function -- DONE (previous audit)
- **Location:** `codegen/src/type_map.rs:7-9`
- **Problem:** `get_base_type(ty) -> BaseType` is just `ty.base_type` -- a single field access wrapped in a function. Called 50+ times across the codegen crate. The wrapper adds indirection without value since `ResolvedType.base_type` is already a direct non-optional field.
- **Fix:** Remove `get_base_type` and use `field.type_.base_type` directly at call sites.
- **Resolution:** Removed `get_base_type` function. Updated all 50+ call sites to access `field.type_.base_type` directly.

### 1.7 Trivial `get_element_type` wrapper function -- DONE (previous audit)
- **Location:** `codegen/src/type_map.rs:12-14`
- **Problem:** Same as 1.6 -- `get_element_type(ty)` is `ty.element_type.unwrap_or(BASE_TYPE_NONE)`, a one-liner wrapper called 15+ times.
- **Fix:** Remove the wrapper. Call sites can use `field.type_.element_type.unwrap_or(BaseType::BASE_TYPE_NONE)` directly, or better, add a method to `ResolvedType`.
- **Resolution:** Added `ResolvedType::element_type_or_none()` method. Removed the wrapper and updated all call sites.

## 2. Unsafe Patterns (unwrap/panic in non-test code)

### 2.1 `unwrap()` calls in TS codegen -- DONE (previous audit)

### 2.2 `writeln!().unwrap()` in fbs-gen -- SKIPPED (previous audit)
- **Reason:** `fmt::Write` for `String` is infallible.

### 2.3 `panic!()` in type-map functions -- DONE (previous audit)
- **Resolution:** Documented with `# Panics` sections.

### 2.4 `eprintln!` warnings in compiler.rs -- SKIPPED (previous audit)

### 2.5 `expect()` calls in `as_legacy` for RPC index conversion -- DONE (previous audit)
- **Location:** `schema/src/resolved.rs:488,491`
- **Problem:** `i32::try_from(c.request_index).expect("RPC request index overflow")` in non-test code. Although overflow is extremely unlikely (would need >2B objects), `expect` in production code violates fail-fast-with-error-propagation principle.
- **Fix:** Return a `ResolveError` instead of panicking.
- **Resolution:** Changed `as_legacy()` to return `Result<Schema, ResolveError>` and propagated errors. Updated the single call site in `compiler/src/main.rs` to handle the Result.

### 2.6 `expect()` in `ts_struct_gen.rs` for array fixed_length -- DONE (previous audit)
- **Location:** `codegen/src/ts_struct_gen.rs:495`
- **Problem:** `.expect("BUG: array field has no fixed_length")` panics in production codegen code.
- **Fix:** Full conversion to `Result` would cascade through the entire TS struct codegen module. Instead, documented the invariant with a `# Panics` doc comment, consistent with the approach taken for `scalar_rust_type` (see 2.3).
- **Resolution:** Added `# Panics` documentation explaining the analyzer guarantee.

### 2.7 NEW: Misleading `expect()` message in data-gen -- DONE
- **Location:** `data-gen/src/lib.rs:97`
- **Problem:** `serde_json::to_string_pretty(&value).expect("serde_json::to_string_pretty cannot fail")` -- the message claims the operation "cannot fail" but it CAN fail if serialization encounters an error.
- **Fix:** Added `DataGenError::JsonSerialization` variant and use `map_err` to convert the serde_json error properly.

## 3. Dead Code / Unused Crates

### 3.1 `grammar` crate unused in workspace -- SKIPPED (previous audit)

### 3.2 Unused `std::env` import -- DONE (previous audit)

### 3.3 `#[allow(dead_code)]` in fbs-gen types -- DONE (previous audit)

### 3.4 `has_enum_index` function is misleading -- DONE (previous audit)
- **Location:** `codegen/src/type_map.rs:18-20`
- **Problem:** `has_enum_index` checks `field.type_.index.map(|i| i >= 0).unwrap_or(false)`. Since `index` is `Option<i32>`, this is equivalent to `field.type_.index.is_some()` because the analyzer never produces negative indices. The name suggests it checks for enums specifically but it actually checks for any type reference.
- **Fix:** Rename to `has_type_index` to match what it actually checks, and simplify to `field.type_.index.is_some()`.
- **Resolution:** Renamed to `has_type_index` and simplified the implementation. Updated all call sites.

## 4. Missing Abstractions

### 4.1 Attribute lookup helpers -- DONE (previous audit)

### 4.2 No structured constructors -- SKIPPED (previous audit)

## 5. Noise / Redundant Comments

### 5.1 Section divider comments -- SKIPPED (previous audit)

## 6. Architectural Concerns

### 6.1 `as_legacy()` bypass -- SKIPPED (previous audit)
### 6.2 TS codegen panics -- DONE (previous audit)
### 6.3 data-gen uses legacy Schema -- SKIPPED (previous audit)

## 7. Potential Correctness Issues

### 7.1 `usize as i32` cast in `as_legacy()` -- DONE (previous audit)
### 7.2 `saturating_sub(1)` for union companion -- DONE (previous audit)
### 7.3 Unknown top-level tokens silently skipped -- DONE (previous audit)

### 7.4 NEW: Silent skipping of out-of-bounds in binary walker -- DONE
- **Location:** `annotator/src/binary_walker.rs:915-918`
- **Problem:**
```rust
for i in byte_range.clone() {
    if i < self.annotated.len() {
        self.annotated[i] = true;
    }
}
```
This silently skips any byte indices that are out of bounds instead of returning an error. If `byte_range` contains invalid indices, this data corruption goes undetected.
- **Fix:** Added `WalkError::ByteRangeOutOfBounds` variant. Changed `add_region` to return `Result<usize, WalkError>` and validate the byte_range before processing. Updated all call sites to use `?`.

## 8. Style / Cosmetic

### 8.1 Inconsistent error handling -- DONE (previous audit)
### 8.2 Keyword check duplication -- SKIPPED (previous audit)

## 9. Duplication (Dart vs TypeScript)

### 9.1 Duplicate `to_camel_case` and `to_pascal_case` functions -- DONE (previous audit)
- **Location:** `codegen/src/ts_type_map.rs:184-216`, `codegen/src/dart_type_map.rs:65-96`
- **Problem:** Nearly identical implementations in both files.
- **Fix:** Extract into a shared helper in `codegen/src/type_map.rs` or a new `common_type_map.rs`.
- **Resolution:** Moved `to_camel_case` and `to_pascal_case` to `type_map.rs` and re-exported from both `ts_type_map.rs` and `dart_type_map.rs`.

### 9.2 Duplicate `gen_doc_comment` functions -- SKIPPED (previous audit)
- **Location:** `codegen/src/ts_type_map.rs:281-295`, `codegen/src/dart_type_map.rs:208-220`
- **Problem:** Both generate documentation comments with only minor formatting differences (JSDoc `/** */` vs DartDoc `///`).
- **Fix:** Extract shared logic, keep language-specific formatting in each module.
- **Reason:** The functions are simple enough and language-specific formatting makes extraction complex.

### 9.3 Duplicate `build_fqn` functions -- DONE (previous audit)
- **Location:** `codegen/src/ts_type_map.rs:298-306`, `codegen/src/dart_type_map.rs:223-231`
- **Problem:** Identical implementations.
- **Fix:** Move to shared `type_map.rs`.
- **Resolution:** Moved `build_fqn` to `type_map.rs` and re-exported from both modules.

## 10. Silent Failures (Dart Codegen)

### 10.1 Union accessor always returns null -- PARTIALLY DOCUMENTED
- **Location:** `codegen/src/dart_gen.rs:376-388`
- **Problem:** `return null;` is hardcoded - the union accessor never returns actual data.
- **Fix:** Implement proper union type resolution and return the correct type.

### 10.2 Wildcard match arms silently ignore unhandled BaseTypes -- DONE (previous audit)
- **Location:** `codegen/src/dart_gen.rs:389`, `dart_gen.rs:439`, `dart_gen.rs:507`, `dart_gen.rs:596`
- **Problem:** `_ => {}` patterns mean new BaseType variants silently produce no code.
- **Fix:** Replace with `unimplemented!()` or proper error handling.
- **Resolution:** Replaced with `unreachable!("unexpected ... in ...")` to catch analyzer bugs.

## 11. Incorrect Write Methods (Dart Type Map)

### 11.1 U_BYTE uses writeInt16 instead of writeUint8 -- DONE (previous audit)
- **Location:** `codegen/src/dart_type_map.rs:53`
- **Problem:** `BaseType::BASE_TYPE_U_BYTE | BaseType::BASE_TYPE_U_TYPE => "writeInt16"` should be `writeUint8`.
- **Fix:** Change to `"writeUint8"`.
- **Resolution:** Fixed.

### 11.2 U_SHORT uses writeInt16 instead of writeUint16 -- DONE (previous audit)
- **Location:** `codegen/src/dart_type_map.rs:53`
- **Problem:** `BaseType::BASE_TYPE_U_SHORT => "writeInt16"` should be `writeUint16`.
- **Fix:** Change to `"writeUint16"`.
- **Resolution:** Fixed.

## 12. Missing Abstractions

### 12.1 Repetitive field accessor naming pattern -- SKIPPED (previous audit)
- **Location:** Across `dart_gen.rs`, `ts_struct_gen.rs`, `dart_struct_gen.rs`
- **Problem:** `dart_type_map::escape_dart_keyword(&dart_type_map::to_camel_case(&field.name))` repeated dozens of times.
- **Fix:** Add a helper function like `field_name(field: &ResolvedField) -> String`.

## 13. Low-Priority Issues

### 13.1 CLI uses String error type instead of proper error enum -- SKIPPED (previous audit)
- **Location:** `compiler/src/main.rs:187-219`
- **Problem:** `fn output_file_path(...) -> Result<PathBuf, String>` should use a proper `CliError` enum.
- **Fix:** Define a `CliError` enum and use it consistently.

### 13.2 NEW: Redundant `new()` constructors in schema types -- DONE
- **Location:** `schema/src/lib.rs:173,210,229,245,275,306,346,423,466,515,549,570,597,640`
- **Problem:** 14 `pub fn new()` implementations that all just call `Self::default()`. These provide no value since `Default` is already implemented.
- **Fix:** Removed all 14 redundant `new()` methods. Updated all callers in parser and compiler crates to use `Default::default()` instead.

### 13.3 NEW: Unreachable with unhelpful messages -- DONE
- **Location:** `fbs-gen/src/gen_table.rs:208,268`
- **Problem:** `unreachable!("pick_weighted returned invalid category")` doesn't provide useful debugging information.
- **Fix:** Updated both unreachable messages to include the category value and weights array: `unreachable!("pick_weighted returned invalid category {category} with weights {:?}")`

### 13.4 NEW: `#[allow(clippy::too_many_arguments)]` suppressions -- DONE
- **Locations:**
  - `annotator/src/binary_walker.rs:332`
  - `codegen/src/rust_table_gen/reader.rs:192`
- **Problem:** Functions with 9+ parameters trigger clippy warnings. Instead of suppressing, consider refactoring to use a context struct.
- **Fix:** Created `WalkFieldContext` struct for `walk_field` and `GenScalarAccessorContext` struct for `gen_scalar_accessor`. Removed the `#[allow(clippy::too_many_arguments)]` attributes since parameter counts are now within limits.

## Summary (Issues Found in This Audit)

| Priority | Count | Status |
|----------|-------|--------|
| Critical (Silent failures) | 1 | Done |
| High (Correctness) | 1 | Done |
| Medium | 2 | Done |
| Low | 3 | Done |
| Previously documented | 25 | Various |
