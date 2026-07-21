<!-- agent-updated: 2026-07-21T21:04:00Z -->
# Rust Pluggable Buffer Design

Last updated: 2026-07-21

## Summary

Generated Rust FlatBuffers readers currently borrow a contiguous `&[u8]` through
the upstream `flatbuffers` crate. The write path already has a plug-in point via
`flatbuffers::Allocator`, but the read path is fixed to slice-backed
`flatbuffers::Table<'a>` and `flatbuffers::Vector<'a, T>`.

The goal is to make generated Rust bytes access replaceable in the same spirit
as Zig's allocator model: generated code should depend on small byte-buffer
capability interfaces, while callers choose the concrete storage policy.

This design proposes a staged change:

1. Add a local runtime adapter trait for readable FlatBuffer bytes.
2. Generate a generic reader mode that stores a buffer view instead of a raw
   upstream `Table<'a>`.
3. Keep the existing slice-backed generated API as the default compatibility
   mode.
4. Move verification/root helpers behind the same adapter so custom buffers are
   first-class, not wrapper-only.

## Current State

The generator already emits allocator-generic builders:

```rust
pub struct MonsterBuilder<'a: 'b, 'b, A: ::flatbuffers::Allocator + 'a> {
    fbb_: &'b mut ::flatbuffers::FlatBufferBuilder<'a, A>,
    start_: ::flatbuffers::WIPOffset<::flatbuffers::TableUnfinishedWIPOffset>,
}
```

This means write allocation can already be customized through the upstream
`flatbuffers::Allocator` trait.

The reader path is not pluggable:

```rust
pub struct Monster<'a> {
    pub _tab: ::flatbuffers::Table<'a>,
}

impl<'a> ::flatbuffers::Follow<'a> for Monster<'a> {
    type Inner = Monster<'a>;

    unsafe fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {
        Self { _tab: unsafe { ::flatbuffers::Table::new(buf, loc) } }
    }
}
```

Root helpers also take slices directly:

```rust
pub fn root_as_monster(buf: &[u8]) -> Result<Monster<'_>, ::flatbuffers::InvalidFlatbuffer> {
    ::flatbuffers::root::<Monster>(buf)
}
```

The upstream `flatbuffers` crate encodes the same assumption in core traits and
types:

- `Follow::follow(buf: &'buf [u8], loc: usize)`
- `Table<'a> { buf: &'a [u8], loc: usize }`
- `Vector<'a, T>(&'a [u8], usize, PhantomData<T>)`
- `Verifier::new(opts, data: &[u8])`

So a truly replaceable read buffer cannot be implemented only by changing
generated table signatures while still relying on upstream `Table`, `Vector`,
and `Follow` for all traversal.

## Goals

- Let callers choose the buffer storage/read policy used by generated readers.
- Make the read interface explicit at byte and range granularity.
- Treat the write interface as allocator-backed random access, not streaming IO.
- Preserve the current generated Rust API by default.
- Keep zero-copy access for normal slice-backed buffers.
- Support common plug-in cases:
  - `&[u8]`
  - `Vec<u8>` or `Arc<[u8]>` owned by an outer object
  - mmap-backed bytes
  - bounded arena/page-backed bytes that can expose contiguous ranges
- Keep generated code mostly mechanical and schema-driven.
- Make the unsafe boundary explicit and testable.

## Non-Goals

- Do not replace upstream `flatbuffers::Allocator` for writing in phase 1.
- Do not support non-contiguous random byte sources in phase 1 unless they can
  expose required ranges as slices.
- Do not fork every upstream runtime type immediately.
- Do not change FlatBuffers wire format or generated schema semantics.

## Proposed API

Add a small `flatc-rs-runtime` crate used by generated Rust code. The shared
byte-access traits live there, while schema-coupled glue such as `FollowIn` and
generic vector/table wrappers remains in generated output so generated table
types stay local to the crate that implements those glue traits.

Rust has `std::io::Read` and `std::io::Write`, but those are streaming traits.
They are not the right core interface for FlatBuffers because FlatBuffers access
is offset-based:

- readers jump to vtables, offsets, vectors, strings, and nested tables;
- builders write backwards, align values, and patch offsets;
- verified roots need range checks, not sequential reads.

The interface therefore needs random byte access. Streaming adapters can still
exist at the application boundary, but generated FlatBuffers readers/builders
should not depend on `Read`/`Write`.

```rust
pub unsafe trait FlatBufferRead {
    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn range(&self, start: usize, len: usize) -> Option<&[u8]>;

    fn read_byte(&self, index: usize) -> Option<u8> {
        self.range(index, 1).map(|bytes| bytes[0])
    }

    fn all_bytes(&self) -> Option<&[u8]> {
        self.range(0, self.len())
    }
}

pub struct SliceBuffer<'a> {
    bytes: &'a [u8],
}

unsafe impl<'a> FlatBufferRead for SliceBuffer<'a> {
    fn len(&self) -> usize {
        self.bytes.len()
    }

    fn range(&self, start: usize, len: usize) -> Option<&[u8]> {
        self.bytes.get(start..start.checked_add(len)?)
    }
}
```

The trait is `unsafe` because implementations must return one stable,
immutable byte sequence for the lifetime of any generated reader derived from
the buffer. `len`, `range`, and `all_bytes` must describe the same backing
bytes: if `all_bytes()` returns `Some(bytes)`, every later successful
`range(start, len)` must return the corresponding subrange of those same bytes.
Implementations must not synthesize temporary slices or expose bytes that can
change while a reader still exists. This is the boundary that lets
`root_as_<name>_in(&buffer)` verify the contiguous view and then safely read
through the buffer-specific `range` implementation.

The write-side interface should build on upstream `flatbuffers::Allocator`,
which is already the real Rust FlatBuffers plug-in point:

```rust
pub unsafe trait FlatBufferWrite: ::flatbuffers::Allocator {}
unsafe impl<T: ::flatbuffers::Allocator> FlatBufferWrite for T {}

pub fn write_byte<W: ?Sized + FlatBufferWrite>(
    buf: &mut W,
    index: usize,
    byte: u8,
) -> Option<()> {
    let bytes: &mut [u8] = core::ops::DerefMut::deref_mut(buf);
    *bytes.get_mut(index)? = byte;
    Some(())
}
```

`FlatBufferWrite` inherits the complete safety contract of
`flatbuffers::Allocator`: the dereferenced mutable byte slice must remain the
allocator's active backing allocation while generated builders perform checked
in-place writes. The blanket implementation adds no weaker or alternate
storage contract.

This is intentionally not a second, symmetric writer API. Any custom writer must
satisfy the same contract as `FlatBufferBuilder`: expose mutable bytes through
`DerefMut<Target = [u8]>` and support `grow_downwards()`. A simple byte-oriented
`Write` implementation is not enough for builder compatibility.

For ergonomic use with owned storage:

```rust
pub trait FlatBufferStorage {
    type Read<'a>: FlatBufferRead + 'a
    where
        Self: 'a;

    fn as_read(&self) -> Self::Read<'_>;
}
```

Examples:

- `&[u8]` returns `SliceBuffer<'_>`
- `Vec<u8>` returns `SliceBuffer<'_>`
- `Arc<[u8]>` returns `SliceBuffer<'_>`
- mmap wrappers return `SliceBuffer<'_>` over their mapped region

## Generated Reader Shape

Add an opt-in codegen mode that emits generic readers:

```rust
pub struct Monster<'buf, B: ?Sized + ::flatc_rs_runtime::FlatBufferRead> {
    _buf: &'buf B,
    _loc: usize,
}
```

The fields are private. Safe construction goes through verified root helpers.
Generated `init_from_buffer`/`init_from_table` constructors are `unsafe` because
callers must already hold the FlatBuffer validity invariant.

Accessors read through local helper functions instead of upstream `Table::get`:

```rust
pub fn hp(&self) -> i16 {
    unsafe {
        ::flatc_rs_runtime::table_get_scalar::<i16, _>(
            self._buf,
            self._loc,
            Self::VT_HP,
            Some(100),
        )
        .unwrap()
    }
}
```

Tables and unions return the same buffer type:

```rust
pub fn pos(&self) -> Option<&'buf Vec3> {
    unsafe {
        ::flatc_rs_runtime::table_get_struct::<Vec3, _>(
            self._buf,
            self._loc,
            Self::VT_POS,
        )
    }
}

pub fn weapon(&self) -> Option<Weapon<'buf, B>> {
    unsafe {
        ::flatc_rs_runtime::table_get_table::<Weapon<'buf, B>, B>(
            self._buf,
            self._loc,
            Self::VT_WEAPON,
        )
    }
}
```

Vectors need a generated/runtime vector view that is generic over `B`:

```rust
pub struct FbVector<'buf, B: ?Sized, T> {
    buf: &'buf B,
    loc: usize,
    _marker: core::marker::PhantomData<T>,
}
```

The generated API can expose:

```rust
pub fn inventory(&self) -> Option<FbVector<'buf, B, u8>>;
```

For scalar vectors backed by contiguous data, `FbVector::bytes()` returns
`Option<&[u8]>`. It returns `None` if a future buffer implementation cannot
expose the complete vector as one contiguous slice.

## Compatibility Mode

The default generated Rust should remain unchanged:

```rust
pub struct Monster<'a> {
    pub _tab: ::flatbuffers::Table<'a>,
}
```

This avoids a large breaking change for existing users.

Introduce a codegen flag after the runtime helpers exist:

```text
--rust-pluggable-buffer
```

In that mode, generate generic readers and adapter root helpers:

```rust
pub fn root_as_monster_in<'buf, B>(
    buf: &'buf B,
) -> Result<Monster<'buf, B>, ::flatbuffers::InvalidFlatbuffer>
where
    B: ?Sized + ::flatc_rs_runtime::FlatBufferRead,
{
    ::flatc_rs_runtime::root::<Monster<'buf, B>, B>(buf)
}
```

For slice users in pluggable mode, also generate the familiar name:

```rust
pub fn root_as_monster(buf: &[u8]) -> Result<Monster<'_, SliceBuffer<'_>>, InvalidFlatbuffer>;
```

## Verification

Phase 1 can require `all_bytes()` for verified roots:

```rust
pub fn root<T, B>(buf: &B) -> Result<T, InvalidFlatbuffer>
where
    B: ?Sized + FlatBufferRead,
{
    let bytes = buf.all_bytes().ok_or(InvalidFlatbuffer::MissingRequiredField)?;
    // Use upstream verifier initially.
}
```

This keeps verification behavior aligned with upstream while still allowing
owned/mmap/arena-backed buffers that expose a contiguous region.

Phase 2 should port verifier reads to `FlatBufferRead::range` so verification
does not require a whole-buffer slice.

## Implementation Plan

### Phase 0: Design and Fixtures

- Add this design document.
- Add generated golden fixtures for the current Rust output before changing
  generation.
- Add schemas that cover:
  - scalars and defaults
  - optional scalars
  - strings
  - structs
  - tables
  - unions
  - vectors of scalars
  - vectors of strings
  - vectors of tables
  - nested flatbuffers
  - file identifiers
  - size-prefixed roots
  - object API pack/unpack
  - serde generation

### Phase 1: Runtime Read Adapter

- Add `FlatBufferRead`, `FlatBufferWrite`, `SliceBuffer`, and basic checked
  byte/range helpers.
- Implement scalar, offset, vtable, string, table, struct, and vector helpers.
- Keep all helpers independent of generated schema types.
- Add unit tests with fake buffers and intentionally short ranges.

### Phase 2: Opt-In Generic Reader Generation

- Add `CodeGenOptions::rust_pluggable_buffer`.
- Add CLI flag `--rust-pluggable-buffer`.
- Generate generic table readers when enabled.
- Preserve existing writer/builder signatures using upstream
  `flatbuffers::FlatBufferBuilder<'a, A>`.
- Generate compatibility root helpers for `&[u8]`.

### Phase 3: Vector and Nested FlatBuffer Coverage

- Replace generated vector return types with runtime `FbVector<'buf, B, T>`.
- Add typed nested flatbuffer helpers that use the same buffer abstraction.
- Add iterators and key lookup for vector-of-table parity.

### Phase 4: Verification Without Whole-Slice Requirement

- Port enough verifier logic to `FlatBufferRead::range`.
- Remove the phase 1 `all_bytes()` requirement for verified roots.
- Keep an escape hatch for `root_unchecked_in` for trusted sources.

## Test Matrix

Tests should follow the existing Rust generated-code coverage style and use
Arrange-Act-Assert comments.

### Runtime Unit Tests

- `SliceBuffer::range` returns expected slices.
- Out-of-bounds ranges return `None`.
- Overflowing `start + len` returns `None`.
- Scalar reads decode little-endian values.
- Table vtable lookup returns defaults for missing slots.
- String reads validate range boundaries.
- Vector length and element reads work.
- Vector reads reject truncated buffers.

### Generated Compile Tests

Run generated pluggable-buffer code through `cargo check` for schemas covering:

- scalar-only table
- all scalar types
- proto-style optional scalar equivalents
- strings with required/default/missing fields
- structs and nested structs
- table fields
- all supported vector element kinds
- unions
- nested flatbuffers
- file identifiers
- size-prefixed roots
- object API enabled
- serde enabled
- direct safe reader construction is rejected by privacy
- vectors of structs, strings, and tables compile in pluggable mode
- union table and struct variants compile in pluggable mode

### Runtime E2E Tests

For each representative schema:

1. Build bytes with upstream `FlatBufferBuilder`.
2. Read with default generated API.
3. Read with pluggable generated API over `SliceBuffer`.
4. Assert identical values.

Add at least one custom buffer implementation used only in tests:

```rust
struct CountingBuffer<'a> {
    bytes: &'a [u8],
    reads: core::cell::Cell<usize>,
}
```

Use it to assert generated readers go through `FlatBufferRead`, not directly
through `&[u8]`.

Add ownership and safety tests that compile or intentionally fail to compile:

- a generated reader can borrow a custom buffer and read scalar, string, and
  vector fields;
- verified roots reject a buffer that cannot expose a full contiguous view in
  phase 1;
- a reader cannot be returned with a `'static` lifetime when it borrows bytes
  owned by a local `FlatBufferBuilder`;
- an owned buffer cannot be mutably changed while a generated reader still
  borrows it;
- the owned buffer can be mutated after the reader's scope ends;
- generated builders accept a custom `flatbuffers::Allocator`, proving write
  ownership stays on the allocator path.

### Regression Tests

- Existing generated Rust output stays byte-for-byte unchanged without
  `--rust-pluggable-buffer`.
- Existing compile/runtime/serde tests continue to pass.
- `--rust-pluggable-buffer` output has its own golden file.
- `--rust-pluggable-buffer` and `--rust-serialize` compile together.
- `--rust-pluggable-buffer` and `--gen-object-api` compile together.

## Open Questions

- Should the runtime adapter live in a separate published crate or inside the
  generated output as a private module?
- Should phase 1 support only contiguous buffers, or should it immediately
  model page-backed buffers with copied scratch ranges?
- Should generated vector APIs return `Option<&[u8]>` for scalar byte vectors,
  or should they always return `FbVector` to avoid API branching?
- How much upstream verifier code should be copied versus wrapped?
- Should pluggable mode eventually become the default after compatibility has
  been proven?

## Recommendation

Do not try to retrofit upstream `flatbuffers::Table<'a>` with a custom buffer.
That type owns the `&[u8]` assumption too deeply. Keep current generated Rust as
the default, add an opt-in generic reader mode, and introduce a small local
runtime abstraction that generated code controls.

This gives us the Zig allocator-style plug-in point without breaking existing
users, and it gives us a clean path to deeper runtime replacement later.
