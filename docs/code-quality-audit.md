# flatbuffers-rs Code Quality Audit

**Date:** 2026-03-17
**Scope:** Full codebase review across all 10 crates (~26,000 lines of Rust)
**Verdict:** This codebase exhibits strong indicators of AI-generated code and contains
numerous architectural flaws, code quality issues, and known unfixed bugs that make it
unsuitable for production use.

---

## Fixes Applied (2026-03-17)

The following issues from this audit have been **fixed** with all 612 tests passing:

| # | Fix | Status |
|---|-----|--------|
| 1.3 | Replaced `catch_unwind` with proper `Result` returns for Rust codegen (9 panic helpers converted, `try_block` added to CodeWriter) | DONE |
| 1.4 | Simplified `float_to_json` dead logic (identical if/else branches) | DONE |
| 2.2 | Fixed `root_table` stale clone: added `root_table_index`, refreshed after layout computation | DONE |
| 2.4 | Replaced all `Option<bool>` flags with `bool` + `#[serde(default)]` (7 fields, ~60 call sites) | DONE |
| 3.1 | Extracted shared `BufReader` into `schema/src/buf_reader.rs` (eliminated 124-line duplication) | DONE |
| 3.2 | Deduplicated `is_scalar()`/`scalar_byte_size()` into `BaseType` methods (3 copies consolidated) | DONE |
| 4.1 | Converted 9 panic-based codegen helpers to return `Result<T, CodeGenError>` | DONE |
| 4.2 | Fixed ~24 `unwrap_or(0) as usize` in decoder, encoder, and annotator with proper error handling | DONE |
| 5.1 | Added `eprintln!` warnings for silently skipped struct Object API and unknown base types | DONE |
| 6.1 | Fixed unsafe `as char` byte casts in type_map, formatter, gen_meta, names | DONE |
| 7.1 | Removed 7 unused function parameters in codegen | DONE |
| 7.2 | Replaced blanket `clippy::all` suppression with specific auditable lint list | DONE |
| 8 (G3.7) | Fixed struct recursion depth limit to return `StructDepthLimitExceeded` (was incorrectly `CircularStruct`) | DONE |
| 8 (G3.19) | Added parse timeout (10s deadline) and input size limit (10 MB) to parser | DONE |
| 8 (G3.20) | Added schema size limits: 10K objects, 10K enums, 100K fields, 100K enum values, 1K included files | DONE |
| 9.16 | Replaced linear O(n) type lookups with HashMap O(1) in decoder, encoder, and annotator | DONE |
| 9.10 | Fixed inconsistent hardcoded indentation in enum codegen (4 instances converted to CodeWriter blocks) | DONE |

---

## Table of Contents

1. [Evidence of AI Generation](#1-evidence-of-ai-generation)
2. [Architectural Flaws](#2-architectural-flaws)
3. [Code Duplication](#3-code-duplication)
4. [Error Handling Failures](#4-error-handling-failures)
5. [Silent Failures and Data Loss](#5-silent-failures-and-data-loss)
6. [Unsafe Code Patterns](#6-unsafe-code-patterns)
7. [Dead Code and Unused Parameters](#7-dead-code-and-unused-parameters)
8. [Known Unfixed Bugs (from ROADMAP.md)](#8-known-unfixed-bugs-from-roadmapmd)
9. [Recommended Improvements](#9-recommended-improvements)

---

## 1. Evidence of AI Generation

### 1.1 Monolithic Commit History

The entire codebase was created in only **8 commits**. The initial commit (`9d1ee99`)
dropped the full project at once. No human develops a 26,000-line compiler in a single
commit. Subsequent commits are labeled "sync: update from private repo" and contain
massive multi-thousand-line diffs with no iterative development history.

```
9d1ee99 flatbuffers-rs: Pure Rust FlatBuffers compiler          <-- entire codebase
f5cae8d feat: add random schema/data generation, clippy fixes, CI workflow
2a0039e refactor: replace tree-sitter parser with hand-written recursive descent
9dfb7f8 feat: add llms.txt, docs, and CLI flags implementation
ff224dd fix: resolve serde rlib ambiguity in compile_test
bf255cc feat: add --annotate CLI flag and extract annotator crate
214859c sync: update from private repo 2026-03-08
879e555 sync: update from private repo 2026-03-09
```

**Improvement:** Any future development should follow standard software engineering
practices with incremental, reviewable commits that each address a single concern.

### 1.2 Self-Audit ROADMAP

`ROADMAP.md` contains a structured audit of the codebase's own bugs, categorized by
severity (CRITICAL, HIGH, MEDIUM). This is not a typical project roadmap -- it reads as
an AI self-reviewing its own output and cataloging what it got wrong. The audit lists 24
MEDIUM-severity known bugs that remain unfixed, including stack overflows, OOM
conditions, and silent data loss scenarios.

**Improvement:** Convert the ROADMAP into a proper issue tracker. Each G3 item should
become a tracked issue with an owner and timeline.

### 1.3 `catch_unwind` as Error Handling Strategy

**Location:** `codegen/src/lib.rs:226-239`

```rust
fn catch_codegen_panic<F: FnOnce() -> String + panic::UnwindSafe>(
    f: F,
) -> Result<String, CodeGenError> {
    panic::catch_unwind(f).map_err(|payload| {
        let msg = if let Some(s) = payload.downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = payload.downcast_ref::<String>() {
            s.clone()
        } else {
            "unknown codegen error".to_string()
        };
        CodeGenError::Internal(msg)
    })
}
```

The entire code generation pipeline is wrapped in `panic::catch_unwind()` because the
internal helper functions (`field_type`, `field_type_index`, `type_index`, etc.) all use
`panic!()` for error cases. This is a band-aid over missing proper error propagation.

**Problems:**
- `catch_unwind` does not work with `panic = "abort"` compilation profiles, which are
  common in release builds for smaller binaries. The code will crash instead of returning
  an error. This is documented as known bug G3.17 but unfixed.
- Stack unwinding has significant performance overhead compared to `Result<T, E>`.
- Panic messages are lossy -- the original error context (file, line, field name) is
  reduced to a string.

**Improvement:** Replace all `panic!()` calls in `codegen/src/lib.rs:34-124` and
`codegen/src/ts_type_map.rs` with `Result<T, CodeGenError>` returns. Thread the
`Result` through the call chain. Remove the `catch_unwind` wrapper entirely. This
requires changing approximately 10 functions and their ~50 call sites.

### 1.4 `float_to_json` Contains Dead Logic

**Location:** `compiler/src/json/decoder.rs:844-854`

```rust
fn float_to_json(v: f64) -> Value {
    if v.is_nan() || v.is_infinite() {
        Value::Null
    } else if v == (v as i64) as f64 && v.abs() < (1i64 << 53) as f64 {
        json!(v)    // <-- returns json!(v)
    } else {
        json!(v)    // <-- also returns json!(v)  -- IDENTICAL to the if branch
    }
}
```

The `if`/`else` branches are identical. The integer range check serves no purpose. This
is a classic AI artifact -- the model generated plausible-looking logic that does nothing.

**Improvement:** Simplify to:

```rust
fn float_to_json(v: f64) -> Value {
    if v.is_nan() || v.is_infinite() {
        Value::Null
    } else {
        json!(v)
    }
}
```

---

## 2. Architectural Flaws

### 2.1 No Separation Between Parsed and Resolved Schemas -- FIXED

**Status:** FIXED in commit `8791685` (refactor: separate parsed and resolved schema types)

The parsed types in `schema/src/lib.rs` (all `Option<T>`) are now only used during
parsing and analysis. After semantic analysis, the analyzer converts to `ResolvedSchema`
(`schema/src/resolved.rs`) where required fields like `name`, `type_`, `value` are
non-optional. Codegen and other downstream consumers now use `ResolvedSchema`,
`ResolvedObject`, `ResolvedField`, etc.

The `ResolvedSchema.as_legacy()` method provides backward compatibility for code that
hasn't been migrated yet.

**Remaining work:** Some fields that are always populated post-analysis are still
`Option<T>` in the resolved types (e.g., `id: Option<u32>`, `offset: Option<u32>`).
These could be made non-optional for stronger guarantees.

### 2.2 `root_table` Is a Deep Clone That Goes Stale

**Location:** `schema/src/lib.rs:528`

```rust
pub struct Schema {
    pub objects: Vec<Object>,
    // ...
    pub root_table: Option<Object>,  // deep clone of one object in `objects`
}
```

`root_table` is a full deep clone of the root table `Object`. After
`compute_struct_layouts()` modifies field offsets on objects in the `objects` vec, the
`root_table` clone still contains the old, pre-layout offsets. Any code that reads
`root_table.fields[i].offset` gets stale data.

This is documented as known bug **G3.15** but remains unfixed.

**Improvement:** Replace `root_table: Option<Object>` with `root_table_index: Option<usize>`
that indexes into `schema.objects`. All consumers dereference through the index. This
eliminates the stale data problem and reduces memory usage.

### 2.3 Linear Object/Enum Lookup

**Location:** `compiler/src/json/decoder.rs:223-260`, `annotator/src/binary_walker.rs:212-231`

Object lookup is performed by linear scan over `schema.objects`:

```rust
fn find_object_index(&self, name: &str) -> Result<usize, ...> {
    for (i, obj) in self.schema.objects.iter().enumerate() {
        if let Some(ref obj_name) = obj.name {
            if obj_name == name {
                return Ok(i);
            }
        }
    }
    // ... second linear scan for short name match
}
```

For schemas with hundreds of types, this is O(n) per lookup.

**Improvement:** Build a `HashMap<String, usize>` name-to-index map once at the start
of decoding/walking. This makes all lookups O(1).

### 2.4 `Option<bool>` for Boolean Flags

**Location:** `schema/src/lib.rs:301-323`

Multiple boolean attributes are modeled as `Option<bool>`, creating a three-state flag
(`None`, `Some(false)`, `Some(true)`) where only two states are meaningful:

```rust
pub is_deprecated: Option<bool>,
pub is_required: Option<bool>,
pub is_key: Option<bool>,
pub is_optional: Option<bool>,
pub is_offset_64: Option<bool>,
```

Throughout the codebase, these are checked as `field.is_required == Some(true)` instead
of just `field.is_required`. This is verbose and error-prone -- forgetting `Some()`
silently evaluates to `false`.

**Improvement:** Use `#[serde(default)]` and plain `bool` fields:

```rust
#[serde(default)]
pub is_deprecated: bool,
#[serde(default)]
pub is_required: bool,
```

---

## 3. Code Duplication

### 3.1 BufReader Duplicated Verbatim (124 lines x 2)

**Locations:**
- `compiler/src/json/decoder.rs:58-181`
- `annotator/src/binary_walker.rs:12-135`

These are **character-for-character identical** implementations of a safe binary buffer
reader with methods: `new`, `len`, `check_bounds`, `read_u8`, `read_i8`, `read_u16_le`,
`read_i16_le`, `read_u32_le`, `read_i32_le`, `read_u64_le`, `read_i64_le`,
`read_f32_le`, `read_f64_le`, `read_bytes`. The only difference is the error type
(`JsonError` vs `WalkError`).

**Improvement:** Extract into a shared crate (e.g., `flatc-rs-binary-reader`) or into
the `schema` crate. Make the error type generic:

```rust
pub struct BufReader<'a, E: From<BoundsError>> {
    buf: &'a [u8],
    _phantom: PhantomData<E>,
}
```

Or use a trait for error conversion. This eliminates 124 lines of duplicated code.

### 3.2 `is_scalar()` and `scalar_byte_size()` Duplicated (3 copies)

**Locations:**
- `codegen/src/type_map.rs:48-64` (`is_scalar`)
- `compiler/src/json/error.rs` (`is_scalar`, `scalar_byte_size`)
- `annotator/src/binary_walker.rs:1179-1194` (`is_scalar`, `scalar_byte_size`)

Three independent implementations of the same BaseType classification logic.

**Improvement:** Move `is_scalar()` and `scalar_byte_size()` to `schema/src/lib.rs` as
methods on `BaseType`:

```rust
impl BaseType {
    pub fn is_scalar(self) -> bool { ... }
    pub fn byte_size(self) -> usize { ... }
}
```

### 3.3 `read_scalar_value` vs `decode_field` Duplication

**Location:** `compiler/src/json/decoder.rs:396-502` and `decoder.rs:710-767`

`decode_field` contains a full scalar decoding match block for all 12 scalar types.
`read_scalar_value` contains the **same match block** with slightly different return
types. Both functions exist in the same file and do the same thing.

**Improvement:** Have `decode_field` delegate to `read_scalar_value` for scalar types
instead of duplicating the logic:

```rust
bt if is_scalar(bt) => {
    let val = self.read_scalar_value(offset, bt, ty)?;
    Ok(Some(val))
}
```

### 3.4 `find_object_index` Duplicated (2 copies)

**Locations:**
- `compiler/src/json/decoder.rs:223-260`
- `annotator/src/binary_walker.rs:212-231`

Nearly identical linear search logic for finding an object by name.

**Improvement:** Move to a shared utility or make it a method on `Schema`.

### 3.5 Union Type Pre-scan Duplicated (2 copies)

**Locations:**
- `compiler/src/json/decoder.rs:308-339` (first-pass union type scanning)
- `annotator/src/binary_walker.rs:311-340` (identical first-pass union type scanning)

Both files implement the same two-pass table decode: first scan for `_type`
discriminator fields, then decode all fields.

**Improvement:** Extract the union type pre-scan into a shared helper.

---

## 4. Error Handling Failures

### 4.1 Panic-Based Internal APIs (10 functions)

**Location:** `codegen/src/lib.rs:34-124`

Ten functions use `panic!()` with "BUG:" messages for error conditions:

| Function | Line | What It Panics On |
|----------|------|-------------------|
| `field_type()` | 34 | Field has no type descriptor |
| `field_type_index()` | 44 | Type has no index |
| `type_index()` | 55 | Type has no index in context |
| `union_variant_type_index()` | 64 | Union variant has no type index |
| `obj_byte_size()` | 77 | Object has no byte_size |
| `obj_min_align()` | 87 | Object has no min_align |
| `field_offset()` | 97 | Field has no offset |
| `field_id()` | 107 | Field has no id |
| `enum_val_value()` | 117 | Enum value has no value |

Additionally, `codegen/src/ts_type_map.rs` contains 8 more `panic!()` calls for
unhandled base types.

**Improvement:** Convert all to `Result<T, CodeGenError>`:

```rust
fn field_type(field: &schema::Field) -> Result<&schema::Type, CodeGenError> {
    field.type_.as_ref().ok_or_else(|| {
        CodeGenError::Internal(format!(
            "field '{}' has no type descriptor",
            field.name.as_deref().unwrap_or("<unknown>")
        ))
    })
}
```

Thread `Result` through all callers using `?`. Remove `catch_unwind`.

### 4.2 `unwrap_or(0) as usize` Silent Wrong-Type References (~40 instances)

**Locations:** Throughout `compiler/src/json/decoder.rs`, `annotator/src/binary_walker.rs`,
and codegen files.

Examples:
```rust
let obj_idx = ty.index.unwrap_or(0) as usize;   // decoder.rs:465,472,534,582,632,674
let enum_idx = ty.index.unwrap_or(0) as usize;   // decoder.rs:447,674
```

When `ty.index` is `None` (no type reference), this silently maps to **index 0** --
the first object or enum in the schema. This produces silently wrong behavior: wrong
type is decoded, wrong fields are read, wrong code is generated.

This is documented as known bug **G3.14** but remains unfixed.

**Improvement:** Replace with explicit error handling:

```rust
let obj_idx = ty.index
    .ok_or(JsonError::MissingTypeIndex { field: fname.to_string() })?
    as usize;
```

### 4.3 String-Based Error Handling in CLI

**Location:** `compiler/src/main.rs:186-213`

Helper functions return `Result<_, String>` instead of proper error types:

```rust
fn output_file_path(...) -> Result<PathBuf, String> { ... }
fn write_output(path: &Path, content: &str) -> Result<(), String> { ... }
```

**Improvement:** Define a `CliError` enum or use the existing `CompilerError` type.

---

## 5. Silent Failures and Data Loss

### 5.1 Object API Silently Skips Structs with Array Fields

**Location:** `codegen/src/struct_gen.rs:532-535`

```rust
if has_array_fields(obj) {
    return;  // No warning, no error
}
```

When `--gen-object-api` is used, structs containing fixed-length array fields
silently produce no Object API code. The user gets incomplete generated code with no
indication that anything was skipped.

**Improvement:** Emit a warning to stderr:

```rust
if has_array_fields(obj) {
    eprintln!("warning: Object API not generated for struct '{}' (contains array fields)", name);
    return;
}
```

Or implement array support in the Object API.

### 5.2 Parser Silently Drops Unrecognized Tokens

**Location:** `parser/src/parser.rs:111` -- `_ => { self.advance(); }`

The hand-written recursive-descent parser (which replaced the tree-sitter-based parser
in commit `2a0039e`) silently skips unrecognized top-level tokens. Invalid schema
constructs produce no error -- they are simply dropped.

**Improvement:** Emit a parse warning or error for unrecognized top-level tokens.

### 5.3 Unhandled BaseType Variants Produce `// TODO` Comments

**Location:** Documented as G3.9

When the Rust codegen encounters an unhandled `BaseType` variant, some code paths
emit `// TODO` comments in the generated output instead of erroring. The generated
Rust code compiles but produces incorrect behavior.

**Improvement:** Replace all `// TODO` fallbacks with `panic!()` or `Result::Err()`
so the issue is caught at generation time rather than silently producing broken code.

### 5.4 Vector-of-Unknown-Type Silently Falls Back to `u8`

**Location:** Documented as G3.10

When the codegen encounters a vector of an unknown element type, it silently generates
`Vector<'a, u8>` instead of reporting an error.

**Improvement:** Report an error instead of generating incorrect type signatures.

---

## 6. Unsafe Code Patterns

### 6.1 Raw Byte-to-Char Casts Without UTF-8 Validation

**Locations:**
- `codegen/src/type_map.rs:79,83`
- `annotator/src/formatter.rs:233,286`
- `fbs-gen/src/gen_meta.rs:36`
- `fbs-gen/src/names.rs:89`

```rust
// type_map.rs:79
let prev = name.as_bytes()[i - 1] as char;
// type_map.rs:83
let next = name.as_bytes()[i + 1] as char;
```

Casting `u8` to `char` via `as` bypasses Rust's UTF-8 safety guarantees. While these
happen to work for ASCII identifiers (which is the expected input), the code will
produce silently wrong results for any non-ASCII input.

**Improvement:** Use `char::from` with explicit validation, or work with bytes directly:

```rust
let prev = name.as_bytes()[i - 1];
if prev.is_ascii_lowercase() || prev.is_ascii_digit() {
    result.push('_');
}
```

### 6.2 `i32 as usize` Unchecked Casts

**Locations:** Throughout codegen and decoder

`Type.index` is `Option<i32>`, but it is used as a `usize` index into arrays:

```rust
ty.index.unwrap_or(0) as usize  // negative i32 becomes a huge usize
```

While G1.1 added a `checked_obj_index()` in `struct_layout.rs`, the same pattern
persists in decoder.rs and binary_walker.rs without protection.

**Improvement:** Add bounds checking:

```rust
let idx = ty.index.filter(|&i| i >= 0).map(|i| i as usize)
    .ok_or(Error::InvalidTypeIndex)?;
if idx >= schema.objects.len() {
    return Err(Error::ObjectIndexOutOfRange { index: idx, count: schema.objects.len() });
}
```

---

## 7. Dead Code and Unused Parameters

### 7.1 Unused Function Parameters

| File | Line | Parameter |
|------|------|-----------|
| `codegen/src/rust_table_gen/reader.rs` | 91 | `_obj: &schema::Object` |
| `codegen/src/rust_table_gen/reader.rs` | 93 | `_field_idx: usize` |
| `codegen/src/rust_table_gen/builder.rs` | 82 | `_field_idx: usize` |
| `codegen/src/rust_table_gen/builder.rs` | 247 | `_schema: &schema::Schema` |
| `codegen/src/rust_table_gen/builder.rs` | 250 | `_current_ns: &str` |
| `codegen/src/rust_table_gen/reader.rs` | 565 | `_schema: &schema::Schema` |
| `codegen/src/rust_table_gen/reader.rs` | 617-620 | `_schema`, `_current_ns` |

**Improvement:** Remove unused parameters or implement the functionality they were
intended for. The underscore prefix suggests these were planned for future use but
never implemented.

### 7.2 Blanket Lint Suppression on Reflection Module

**Location:** `compiler/src/lib.rs:7-16`

```rust
#[allow(
    unused_imports, dead_code, clippy::all,
    non_camel_case_types, non_snake_case,
    unused_variables, unused_mut, deprecated
)]
pub mod reflection;
```

This suppresses **ALL** clippy and rustc warnings for the entire reflection module,
hiding any code quality issues.

**Improvement:** Remove the blanket suppression. Fix or individually suppress specific
warnings with explanatory comments. If the module is generated, document the generation
source and add `// @generated` markers.

---

## 8. Known Bugs (from ROADMAP.md) -- Status Update

Many bugs listed in the original ROADMAP have been fixed. Updated status:

| ID | Category | Description | Status |
|----|----------|-------------|--------|
| G3.7 | Crash | Unbounded recursion in struct topological sort. | **FIXED** -- depth limit (256) + proper `StructDepthLimitExceeded` error |
| G3.8 | Crash | No include depth limit. | **FIXED** -- 64-level depth limit, 1000 file limit |
| G3.14 | Wrong output | ~40 instances of `unwrap_or(0) as usize`. | **FIXED** -- proper error handling throughout decoder/encoder/annotator |
| G3.17 | Crash | `panic!()` calls in Rust codegen behind `catch_unwind`. | **FIXED** -- Rust codegen converted to `Result<T, CodeGenError>`. TS codegen still uses `catch_unwind` as safety net. |
| G3.9 | Wrong output | Unhandled BaseType emits `// TODO` in generated code. | Partially addressed -- warnings added |
| G3.10 | Wrong output | Vector-of-unknown-type silently generates `Vector<u8>`. | Still open |
| G3.15 | Wrong output | `root_table` deep clone goes stale. | **FIXED** -- `root_table_index` added, refreshed after layout computation |
| G3.19 | DoS | No parser timeout. | **FIXED** -- 10-second parse timeout added |
| G3.20 | DoS | No schema size limit. | **FIXED** -- 10K objects, 10K enums, 100K fields, 100K enum values, 1K files |
| G3.1 | Wrong output | Union field `id: 0` collision. | **FIXED** -- `UnionFieldIdZero` validation error |
| G3.6 | Wrong output | No parent namespace walking. | **FIXED** -- `TypeIndex.resolve()` walks parent namespaces |

**Remaining open items:** G3.9, G3.10 (silent fallbacks in codegen), G3.23 (parser
silently drops unrecognized tokens).

---

## 9. Recommended Improvements

### Priority 1: Correctness -- ALL DONE

1. ~~Replace `unwrap_or(0) as usize` with proper error handling (G3.14)~~ -- DONE
2. ~~Replace panic-based codegen APIs with `Result` (G3.17)~~ -- DONE (Rust codegen)
3. ~~Fix `root_table` stale clone (G3.15)~~ -- DONE
4. ~~Add recursion depth limit to struct topological sort (G3.7)~~ -- DONE

### Priority 2: Architecture -- MOSTLY DONE

5. ~~Separate parsed and resolved schema types~~ -- DONE (`schema/src/resolved.rs`)
6. ~~Extract shared `BufReader` into a common crate~~ -- DONE (`schema/src/buf_reader.rs`)
7. ~~Deduplicate `is_scalar()`, `scalar_byte_size()`~~ -- DONE (methods on `BaseType`)
8. ~~Replace `Option<bool>` flags with `bool`~~ -- DONE

### Priority 3: Code Quality (Nice to Fix)

9. **Remove unused function parameters**
   - 7 instances of `_`-prefixed unused parameters
   - Estimated effort: 30 minutes

10. **Fix inconsistent indentation in code generator**
    - Replace hardcoded `"    "` strings with `CodeWriter` indentation
    - Estimated effort: 1-2 hours

11. **Add warnings for silently skipped features**
    - Object API skipping array structs (struct_gen.rs:532)
    - Parser dropping unrecognized nodes (G3.23)
    - Estimated effort: 1-2 hours

12. **Remove blanket lint suppression on reflection module**
    - Fix or individually suppress each warning
    - Estimated effort: 1-2 hours

13. **Simplify `float_to_json` dead logic**
    - Remove the identical if/else branches
    - Estimated effort: 5 minutes

### Priority 4: Robustness (For Production)

14. **Add input size limits** (G3.20)
    - Cap maximum number of types, fields, and include depth
    - Estimated effort: 2-4 hours

15. **Add tree-sitter parse timeout** (G3.19)
    - Prevent pathological input from stalling the compiler
    - Estimated effort: 1-2 hours

16. **Build name-to-index HashMap for O(1) type lookup**
    - Replace linear scans in decoder and annotator
    - Estimated effort: 1-2 hours

---

## Appendix: File Reference

Key files mentioned in this audit:

| File | Lines | Role |
|------|-------|------|
| `schema/src/lib.rs` | 545 | Schema type definitions (all-Optional pattern) |
| `codegen/src/lib.rs` | 240 | Codegen entry point (panic-based helpers, catch_unwind) |
| `codegen/src/rust_table_gen/reader.rs` | 746 | Rust table reader generation |
| `codegen/src/rust_table_gen/builder.rs` | 324 | Rust table builder generation |
| `codegen/src/rust_table_gen/object_api.rs` | 579 | Rust Object API generation |
| `codegen/src/enum_gen.rs` | 536 | Rust enum/bitflags generation |
| `codegen/src/struct_gen.rs` | 653 | Rust struct generation |
| `codegen/src/type_map.rs` | 365 | Type mapping and name conversion |
| `compiler/src/main.rs` | 700 | CLI entry point |
| `compiler/src/analyzer.rs` | 1310 | 8-step semantic analysis pipeline |
| `compiler/src/compiler.rs` | 449 | Include resolution and schema merging |
| `compiler/src/json/decoder.rs` | 854 | FlatBuffer binary to JSON conversion |
| `compiler/src/json/encoder.rs` | 926 | JSON to FlatBuffer binary conversion |
| `compiler/src/lib.rs` | 32 | Compiler crate root (blanket lint suppression) |
| `annotator/src/binary_walker.rs` | 1194 | Binary annotation engine |
| `ROADMAP.md` | 288 | Self-audit with 24 known unfixed bugs |
