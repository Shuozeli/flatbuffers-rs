/// Random-access bytes used by generated FlatBuffers readers.
///
/// # Safety
///
/// Implementors must expose one immutable byte sequence for the full lifetime
/// of any reader built from `&self`. `len`, `range`, and `all_bytes` must be
/// mutually consistent: every successful `range(start, len)` must return bytes
/// from the same backing sequence described by `all_bytes`, and those bytes must
/// not change while a generated reader borrowing this buffer is alive.
pub unsafe trait FlatBufferRead {
    fn len(&self) -> usize;

    #[inline]
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

unsafe impl FlatBufferRead for [u8] {
    #[inline]
    fn len(&self) -> usize {
        <[u8]>::len(self)
    }

    #[inline]
    fn range(&self, start: usize, len: usize) -> Option<&[u8]> {
        self.get(start..start.checked_add(len)?)
    }
}

/// Mutable contiguous storage used by generated FlatBuffers builders.
///
/// # Safety
///
/// Implementors must uphold every safety invariant of
/// [`flatbuffers::Allocator`]. In particular, its dereferenced byte slice must
/// remain the same allocation described by the allocator while generated
/// builder code performs checked in-place writes.
pub unsafe trait FlatBufferWrite: flatbuffers::Allocator {}
unsafe impl<T: flatbuffers::Allocator> FlatBufferWrite for T {}

#[inline]
pub fn write_byte<W: ?Sized + FlatBufferWrite>(buf: &mut W, index: usize, byte: u8) -> Option<()> {
    let bytes: &mut [u8] = core::ops::DerefMut::deref_mut(buf);
    *bytes.get_mut(index)? = byte;
    Some(())
}

#[inline]
pub fn write_range<W: ?Sized + FlatBufferWrite>(
    buf: &mut W,
    start: usize,
    src: &[u8],
) -> Option<()> {
    let bytes: &mut [u8] = core::ops::DerefMut::deref_mut(buf);
    bytes
        .get_mut(start..start.checked_add(src.len())?)?
        .copy_from_slice(src);
    Some(())
}

#[derive(Clone, Copy)]
pub struct SliceBuffer<'a> {
    bytes: &'a [u8],
}

impl<'a> SliceBuffer<'a> {
    #[inline]
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes }
    }
}

unsafe impl<'a> FlatBufferRead for SliceBuffer<'a> {
    #[inline]
    fn len(&self) -> usize {
        self.bytes.len()
    }

    #[inline]
    fn range(&self, start: usize, len: usize) -> Option<&[u8]> {
        self.bytes.get(start..start.checked_add(len)?)
    }
}

#[cfg(test)]
mod tests {
    use super::{FlatBufferRead, SliceBuffer};

    #[test]
    fn slice_buffer_reports_an_empty_backing_sequence() {
        // Arrange
        let buffer = SliceBuffer::new(&[]);

        // Act
        let is_empty = buffer.is_empty();

        // Assert
        assert!(is_empty);
    }

    #[test]
    fn byte_slice_reports_a_non_empty_backing_sequence() {
        // Arrange
        let bytes = [7_u8];

        // Act
        let is_empty = FlatBufferRead::is_empty(bytes.as_slice());

        // Assert
        assert!(!is_empty);
    }
}
