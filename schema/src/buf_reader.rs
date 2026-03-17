/// A safe buffer reader for FlatBuffers binary data.
///
/// All read methods return `Result<T, BoundsError>`. Callers convert
/// `BoundsError` into their own error type via the `From` trait and `?`.

/// Error returned when a read would exceed the buffer bounds.
#[derive(Debug, Clone)]
pub struct BoundsError {
    pub offset: usize,
    pub size: usize,
    pub buf_len: usize,
}

impl std::fmt::Display for BoundsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "buffer read out of bounds: offset {}, size {}, buffer length {}",
            self.offset, self.size, self.buf_len
        )
    }
}

impl std::error::Error for BoundsError {}

pub struct BufReader<'a> {
    buf: &'a [u8],
}

impl<'a> BufReader<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Self { buf }
    }

    pub fn len(&self) -> usize {
        self.buf.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    pub fn check_bounds(&self, offset: usize, size: usize) -> Result<(), BoundsError> {
        if offset
            .checked_add(size)
            .is_none_or(|end| end > self.buf.len())
        {
            return Err(BoundsError {
                offset,
                size,
                buf_len: self.buf.len(),
            });
        }
        Ok(())
    }

    pub fn read_u8(&self, offset: usize) -> Result<u8, BoundsError> {
        self.check_bounds(offset, 1)?;
        Ok(self.buf[offset])
    }

    pub fn read_i8(&self, offset: usize) -> Result<i8, BoundsError> {
        self.check_bounds(offset, 1)?;
        Ok(self.buf[offset] as i8)
    }

    pub fn read_u16_le(&self, offset: usize) -> Result<u16, BoundsError> {
        self.check_bounds(offset, 2)?;
        Ok(u16::from_le_bytes([self.buf[offset], self.buf[offset + 1]]))
    }

    pub fn read_i16_le(&self, offset: usize) -> Result<i16, BoundsError> {
        self.check_bounds(offset, 2)?;
        Ok(i16::from_le_bytes([self.buf[offset], self.buf[offset + 1]]))
    }

    pub fn read_u32_le(&self, offset: usize) -> Result<u32, BoundsError> {
        self.check_bounds(offset, 4)?;
        Ok(u32::from_le_bytes([
            self.buf[offset],
            self.buf[offset + 1],
            self.buf[offset + 2],
            self.buf[offset + 3],
        ]))
    }

    pub fn read_i32_le(&self, offset: usize) -> Result<i32, BoundsError> {
        self.check_bounds(offset, 4)?;
        Ok(i32::from_le_bytes([
            self.buf[offset],
            self.buf[offset + 1],
            self.buf[offset + 2],
            self.buf[offset + 3],
        ]))
    }

    pub fn read_u64_le(&self, offset: usize) -> Result<u64, BoundsError> {
        self.check_bounds(offset, 8)?;
        Ok(u64::from_le_bytes([
            self.buf[offset],
            self.buf[offset + 1],
            self.buf[offset + 2],
            self.buf[offset + 3],
            self.buf[offset + 4],
            self.buf[offset + 5],
            self.buf[offset + 6],
            self.buf[offset + 7],
        ]))
    }

    pub fn read_i64_le(&self, offset: usize) -> Result<i64, BoundsError> {
        self.check_bounds(offset, 8)?;
        Ok(i64::from_le_bytes([
            self.buf[offset],
            self.buf[offset + 1],
            self.buf[offset + 2],
            self.buf[offset + 3],
            self.buf[offset + 4],
            self.buf[offset + 5],
            self.buf[offset + 6],
            self.buf[offset + 7],
        ]))
    }

    pub fn read_f32_le(&self, offset: usize) -> Result<f32, BoundsError> {
        self.check_bounds(offset, 4)?;
        Ok(f32::from_le_bytes([
            self.buf[offset],
            self.buf[offset + 1],
            self.buf[offset + 2],
            self.buf[offset + 3],
        ]))
    }

    pub fn read_f64_le(&self, offset: usize) -> Result<f64, BoundsError> {
        self.check_bounds(offset, 8)?;
        Ok(f64::from_le_bytes([
            self.buf[offset],
            self.buf[offset + 1],
            self.buf[offset + 2],
            self.buf[offset + 3],
            self.buf[offset + 4],
            self.buf[offset + 5],
            self.buf[offset + 6],
            self.buf[offset + 7],
        ]))
    }

    pub fn read_bytes(&self, offset: usize, len: usize) -> Result<&'a [u8], BoundsError> {
        self.check_bounds(offset, len)?;
        Ok(&self.buf[offset..offset + len])
    }
}
