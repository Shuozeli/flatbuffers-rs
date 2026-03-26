# Code Quality Findings

*Last audit: 2026-03-26*

## 1. Duplication

### 1.1 Scalar size/alignment computed in three places -- DONE (previous audit)
- **Resolution:** Added `BaseType::byte_size() -> Option<u32>` as the canonical source. All call sites delegate to it.

### 1.2 `as_legacy` conversion is a large manual mirror -- SKIPPED (previous audit)
- **Reason:** Architectural refactor, deferred.

### 1.3 `ResolvedType` construction duplicated in `as_legacy` -- DONE (previous audit)
- **Resolution:** Added `ResolvedType::to_parsed()` method.

### 1.4 Duplicate `catch_unwind` panic-to-error conversion -- PARTIALLY DONE (previous audit)
- **Resolution:** `catch_codegen_panic` removed from codegen. WASM `catch` remains for boundary safety.

### 1.5 Triplicated `object_index` HashMap construction -- NEW, DONE
- **Location:** `annotator/src/binary_walker.rs:41-50`, `compiler/src/json/decoder.rs:71-80`, `compiler/src/json/encoder.rs:62-71`
- **Problem:** Three places build an identical `HashMap<&str, usize>` mapping object names (both FQN and short names) to schema indices. The loop body is copy-pasted verbatim in all three.
- **Fix:** Extract a shared `build_object_index` function in the `schema` crate's `resolved` module.
- **Resolution:** Added `ResolvedSchema::build_object_index()` method. All three call sites now delegate to it.

### 1.6 Trivial `get_base_type` wrapper function -- NEW, DONE
- **Location:** `codegen/src/type_map.rs:7-9`
- **Problem:** `get_base_type(ty) -> BaseType` is just `ty.base_type` -- a single field access wrapped in a function. Called 50+ times across the codegen crate. The wrapper adds indirection without value since `ResolvedType.base_type` is already a direct non-optional field.
- **Fix:** Remove `get_base_type` and use `field.type_.base_type` directly at call sites.
- **Resolution:** Removed `get_base_type` function. Updated all 50+ call sites to access `field.type_.base_type` directly.

### 1.7 Trivial `get_element_type` wrapper function -- NEW, DONE
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

### 2.5 `expect()` calls in `as_legacy` for RPC index conversion -- NEW, DONE
- **Location:** `schema/src/resolved.rs:488,491`
- **Problem:** `i32::try_from(c.request_index).expect("RPC request index overflow")` in non-test code. Although overflow is extremely unlikely (would need >2B objects), `expect` in production code violates fail-fast-with-error-propagation principle.
- **Fix:** Return a `ResolveError` instead of panicking.
- **Resolution:** Changed `as_legacy()` to return `Result<Schema, ResolveError>` and propagated errors. Updated the single call site in `compiler/src/main.rs` to handle the Result.

### 2.6 `expect()` in `ts_struct_gen.rs` for array fixed_length -- NEW, DOCUMENTED
- **Location:** `codegen/src/ts_struct_gen.rs:495`
- **Problem:** `.expect("BUG: array field has no fixed_length")` panics in production codegen code.
- **Fix:** Full conversion to `Result` would cascade through the entire TS struct codegen module. Instead, documented the invariant with a `# Panics` doc comment, consistent with the approach taken for `scalar_rust_type` (see 2.3).
- **Resolution:** Added `# Panics` documentation explaining the analyzer guarantee.

## 3. Dead Code / Unused Crates

### 3.1 `grammar` crate unused in workspace -- SKIPPED (previous audit)

### 3.2 Unused `std::env` import -- DONE (previous audit)

### 3.3 `#[allow(dead_code)]` in fbs-gen types -- DONE (previous audit)

### 3.4 `has_enum_index` function is misleading -- NEW, DONE
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

## 8. Style / Cosmetic

### 8.1 Inconsistent error handling -- DONE (previous audit)
### 8.2 Keyword check duplication -- SKIPPED (previous audit)
