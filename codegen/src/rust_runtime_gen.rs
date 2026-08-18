use codegen_core::CodeWriter;

const RUNTIME_TEMPLATE: &str = r#"
#[allow(dead_code, unused_imports)]
pub mod __flatc_rs_runtime {
    pub use ::flatc_rs_runtime::{write_byte, write_range, FlatBufferRead, FlatBufferWrite, SliceBuffer};

    use core::marker::PhantomData;
    use core::str;

    #[derive(Clone, Copy)]
    pub struct Table<'a, B: ?Sized + FlatBufferRead> {
        pub buf: &'a B,
        pub loc: usize,
    }

    pub struct Vector<'a, B: ?Sized + FlatBufferRead, T> {
        buf: &'a B,
        loc: usize,
        _marker: PhantomData<T>,
    }

    impl<'a, B: ?Sized + FlatBufferRead, T> Copy for Vector<'a, B, T> {}

    impl<'a, B: ?Sized + FlatBufferRead, T> Clone for Vector<'a, B, T> {
        fn clone(&self) -> Self { *self }
    }

    impl<'a, B: ?Sized + FlatBufferRead, T> core::fmt::Debug for Vector<'a, B, T> {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.debug_struct("Vector").field("len", &self.len()).finish()
        }
    }

    impl<'a, B: ?Sized + FlatBufferRead, T> Vector<'a, B, T> {
        #[inline]
        pub unsafe fn new(buf: &'a B, loc: usize) -> Self {
            Self { buf, loc, _marker: PhantomData }
        }

        #[inline]
        pub fn len(&self) -> usize { read_uoffset(self.buf, self.loc) as usize }

        #[inline]
        pub fn is_empty(&self) -> bool { self.len() == 0 }

        #[inline]
        pub fn bytes(&self) -> Option<&'a [u8]> {
            let len = self.len().checked_mul(core::mem::size_of::<T>())?;
            self.buf.range(checked_add(self.loc, ::flatbuffers::SIZE_UOFFSET), len)
        }
    }

    impl<'a, B, T> Vector<'a, B, T>
    where
        B: ?Sized + FlatBufferRead,
        T: FollowIn<'a, B>,
    {
        #[inline]
        pub fn get(&self, idx: usize) -> T::Inner {
            assert!(idx < self.len());
            let elem_off = core::mem::size_of::<T>().checked_mul(idx).expect("flatbuffer vector offset overflow");
            let loc = checked_add(checked_add(self.loc, ::flatbuffers::SIZE_UOFFSET), elem_off);
            unsafe { T::follow_in(self.buf, loc) }
        }

        #[inline]
        pub fn iter(&self) -> VectorIter<'a, B, T> {
            VectorIter { vector: *self, index: 0 }
        }
    }

    pub struct VectorIter<'a, B: ?Sized + FlatBufferRead, T> {
        vector: Vector<'a, B, T>,
        index: usize,
    }

    impl<'a, B, T> Iterator for VectorIter<'a, B, T>
    where
        B: ?Sized + FlatBufferRead,
        T: FollowIn<'a, B>,
    {
        type Item = T::Inner;

        #[inline]
        fn next(&mut self) -> Option<Self::Item> {
            if self.index >= self.vector.len() {
                return None;
            }
            let value = self.vector.get(self.index);
            self.index += 1;
            Some(value)
        }
    }

    impl<'a, B, T> IntoIterator for Vector<'a, B, T>
    where
        B: ?Sized + FlatBufferRead,
        T: FollowIn<'a, B>,
    {
        type Item = T::Inner;
        type IntoIter = VectorIter<'a, B, T>;

        #[inline]
        fn into_iter(self) -> Self::IntoIter {
            VectorIter { vector: self, index: 0 }
        }
    }

    pub unsafe trait FollowIn<'a, B: ?Sized + FlatBufferRead> {
        type Inner;
        unsafe fn follow_in(buf: &'a B, loc: usize) -> Self::Inner;
    }

    macro_rules! impl_scalar_follow_in {
        ($($ty:ty),* $(,)?) => {
            $(unsafe impl<'a, B: ?Sized + FlatBufferRead> FollowIn<'a, B> for $ty {
                type Inner = $ty;

                #[inline]
                unsafe fn follow_in(buf: &'a B, loc: usize) -> Self::Inner {
                    read_scalar::<$ty, B>(buf, loc)
                }
            })*
        };
    }

    impl_scalar_follow_in!(bool, i8, u8, i16, u16, i32, u32, i64, u64, f32, f64);

    unsafe impl<'a, B: ?Sized + FlatBufferRead> FollowIn<'a, B> for ::flatbuffers::ForwardsUOffset<&'a str> {
        type Inner = &'a str;

        #[inline]
        unsafe fn follow_in(buf: &'a B, loc: usize) -> Self::Inner {
            follow_string(buf, uoffset_target(buf, loc))
        }
    }

    #[inline]
    pub fn checked_add(lhs: usize, rhs: usize) -> usize {
        lhs.checked_add(rhs).expect("flatbuffer offset overflow")
    }

    #[inline]
    pub fn uoffset_target<B: ?Sized + FlatBufferRead>(buf: &B, loc: usize) -> usize {
        checked_add(loc, read_uoffset(buf, loc) as usize)
    }

    #[inline]
    pub fn read_scalar<T, B: ?Sized + FlatBufferRead>(buf: &B, loc: usize) -> T
    where
        T: ::flatbuffers::EndianScalar,
    {
        let bytes = buf.range(loc, core::mem::size_of::<T>()).expect("flatbuffer scalar out of bounds");
        unsafe { ::flatbuffers::read_scalar::<T>(bytes) }
    }

    #[inline]
    pub fn read_uoffset<B: ?Sized + FlatBufferRead>(buf: &B, loc: usize) -> ::flatbuffers::UOffsetT {
        read_scalar::<::flatbuffers::UOffsetT, B>(buf, loc)
    }

    #[inline]
    pub fn table_field_loc<B: ?Sized + FlatBufferRead>(buf: &B, table_loc: usize, slot: ::flatbuffers::VOffsetT) -> Option<usize> {
        let soff = read_scalar::<::flatbuffers::SOffsetT, B>(buf, table_loc);
        let vtable_loc = if soff >= 0 {
            table_loc.checked_sub(soff as usize)?
        } else {
            table_loc.checked_add(soff.unsigned_abs() as usize)?
        };
        let vtable_len = read_scalar::<::flatbuffers::VOffsetT, B>(buf, vtable_loc);
        if slot >= vtable_len {
            return None;
        }
        let field_off = read_scalar::<::flatbuffers::VOffsetT, B>(buf, checked_add(vtable_loc, slot as usize));
        if field_off == 0 {
            None
        } else {
            Some(checked_add(table_loc, field_off as usize))
        }
    }

    #[inline]
    pub unsafe fn table_get<T, B>(buf: &B, table_loc: usize, slot: ::flatbuffers::VOffsetT, default: Option<T>) -> Option<T>
    where
        T: ::flatbuffers::EndianScalar,
        B: ?Sized + FlatBufferRead,
    {
        table_field_loc(buf, table_loc, slot).map(|loc| read_scalar::<T, B>(buf, loc)).or(default)
    }

    #[inline]
    pub unsafe fn table_get_string<'a, B: ?Sized + FlatBufferRead>(buf: &'a B, table_loc: usize, slot: ::flatbuffers::VOffsetT, default: Option<&'a str>) -> Option<&'a str> {
        match table_field_loc(buf, table_loc, slot) {
            Some(loc) => Some(follow_string(buf, uoffset_target(buf, loc))),
            None => default,
        }
    }

    #[inline]
    pub unsafe fn follow_string<'a, B: ?Sized + FlatBufferRead>(buf: &'a B, loc: usize) -> &'a str {
        let len = read_uoffset(buf, loc) as usize;
        let bytes = buf.range(checked_add(loc, ::flatbuffers::SIZE_UOFFSET), len).expect("flatbuffer string out of bounds");
        str::from_utf8_unchecked(bytes)
    }

    #[inline]
    pub unsafe fn table_get_struct<'a, S, B: ?Sized + FlatBufferRead>(buf: &'a B, table_loc: usize, slot: ::flatbuffers::VOffsetT) -> Option<&'a S> {
        let loc = table_field_loc(buf, table_loc, slot)?;
        follow_struct(buf, loc)
    }

    #[inline]
    pub unsafe fn follow_struct<'a, S, B: ?Sized + FlatBufferRead>(buf: &'a B, loc: usize) -> Option<&'a S> {
        if core::mem::align_of::<S>() != 1 {
            return None;
        }
        let bytes = buf.range(loc, core::mem::size_of::<S>())?;
        Some(&*(bytes.as_ptr() as *const S))
    }

    #[inline]
    pub unsafe fn table_get_vector<'a, B, T>(buf: &'a B, table_loc: usize, slot: ::flatbuffers::VOffsetT) -> Option<Vector<'a, B, T>>
    where
        B: ?Sized + FlatBufferRead,
    {
        let loc = table_field_loc(buf, table_loc, slot)?;
        Some(Vector::new(buf, uoffset_target(buf, loc)))
    }

    #[inline]
    pub fn root_loc<B: ?Sized + FlatBufferRead>(buf: &B) -> usize {
        read_uoffset(buf, 0) as usize
    }

    #[inline]
    pub fn size_prefixed_root_loc<B: ?Sized + FlatBufferRead>(buf: &B) -> usize {
        ::flatbuffers::SIZE_SIZEPREFIX + read_uoffset(buf, ::flatbuffers::SIZE_SIZEPREFIX) as usize
    }
}
"#;

pub(crate) fn generate(w: &mut CodeWriter) {
    for line in RUNTIME_TEMPLATE.lines() {
        w.line(line);
    }
}
