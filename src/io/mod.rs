use crate::Error;

/// A buffer for writing encoded data.
///
/// This trait abstracts over different types of output buffers (e.g., `Vec<u8>`, `&mut [u8]`).
/// It is designed for maximum performance by minimizing checks and allowing for pre-allocation.
pub trait WriteBuffer {
    /// Writes a single byte to the buffer.
    ///
    /// # Performance
    /// For implementations like `Vec<u8>`, this may trigger a re-allocation if the
    /// capacity is exceeded. Use [`Self::reserve`] to minimize this.
    fn write_byte(&mut self, b: u8) -> Result<(), Error>;

    /// Writes a slice of bytes to the buffer.
    ///
    /// # Performance
    /// This is typically implemented using `copy_from_slice` for maximum throughput.
    fn write_bytes(&mut self, bs: &[u8]) -> Result<(), Error>;

    /// Writes a slice of bytes from a pointer (unsafe, for zero-copy scenarios).
    ///
    /// Default implementation uses write_bytes.
    unsafe fn write_bytes_ptr(&mut self, ptr: *const u8, len: usize) -> Result<(), Error> {
        if len > 0 {
            let slice = unsafe { core::slice::from_raw_parts(ptr, len) };
            self.write_bytes(slice)?;
        }
        Ok(())
    }

    /// Returns the current length of the buffer.
    /// Default implementation returns 0 for cases where length tracking is not needed.
    fn len(&self) -> usize {
        0
    }

    /// Returns if the buffer is empty.
    #[inline]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Reserves space for at least `hint` more bytes to be written.
    ///
    /// This is a performance hint and may be a no-op for some implementations.
    #[inline]
    fn reserve(&mut self, _hint: usize) {}
}

impl WriteBuffer for alloc::vec::Vec<u8> {
    #[inline]
    fn write_byte(&mut self, b: u8) -> Result<(), Error> {
        self.push(b);
        Ok(())
    }

    #[inline]
    fn write_bytes(&mut self, bs: &[u8]) -> Result<(), Error> {
        self.extend_from_slice(bs);
        Ok(())
    }

    #[inline]
    fn len(&self) -> usize {
        alloc::vec::Vec::len(self)
    }

    #[inline]
    fn reserve(&mut self, hint: usize) {
        alloc::vec::Vec::reserve(self, hint);
    }
}

impl WriteBuffer for &mut [u8] {
    #[inline]
    fn write_byte(&mut self, b: u8) -> Result<(), Error> {
        if self.is_empty() {
            return Err(Error::UnexpectedEof);
        }
        self[0] = b;
        *self = &mut std::mem::take(self)[1..];
        Ok(())
    }

    #[inline]
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), Error> {
        if (&**self).len() < bytes.len() {
            return Err(Error::UnexpectedEof);
        }
        self[..bytes.len()].copy_from_slice(bytes);
        *self = &mut std::mem::take(self)[bytes.len()..];
        Ok(())
    }

    #[inline]
    fn len(&self) -> usize {
        (&**self).len()
    }
}

/// A buffer for reading data during deserialization.
///
/// `ReadBuffer` keeps track of the current reading position and provides safe
/// methods for peeking, reading, and validating data from a byte slice.
///
/// # Lifetime
/// The `'de` lifetime ensures that any data borrowed from this buffer (e.g., strings)
/// cannot outlive the buffer itself.
pub struct ReadBuffer<'de> {
    pub(crate) data: &'de [u8],
    pub(crate) pos: usize,
}

impl<'de> ReadBuffer<'de> {
    /// Creates a new `ReadBuffer` from a byte slice.
    #[inline]
    pub fn new(data: &'de [u8]) -> Self {
        Self { data, pos: 0 }
    }

    /// Returns a slice from current position to end.
    #[inline]
    pub fn remaining_slice(&self) -> &'de [u8] {
        &self.data[self.pos..]
    }

    /// Peeks at the next byte without advancing the position.
    #[inline]
    pub fn peek(&self) -> u8 {
        self.data.get(self.pos).copied().unwrap_or(0)
    }

    /// Peeks at offset without advancing.
    #[inline]
    pub fn peek_at(&self, offset: usize) -> u8 {
        self.data.get(self.pos + offset).copied().unwrap_or(0)
    }

    /// Returns the current reading position.
    #[inline]
    pub fn get_pos(&self) -> usize {
        self.pos
    }

    /// Sets the position directly.
    #[inline]
    pub fn set_pos(&mut self, pos: usize) {
        self.pos = pos;
    }

    /// Checks if the buffer has reached the end.
    #[inline]
    pub fn is_eof(&self) -> bool {
        self.pos >= self.data.len()
    }

    /// Checks if there are at least n bytes remaining.
    #[inline]
    pub fn has_remaining(&self, n: usize) -> bool {
        self.pos + n <= self.data.len()
    }

    /// Advances the reading position by `n` bytes.
    #[inline]
    pub fn advance(&mut self, n: usize) {
        self.pos += n;
    }

    /// Reads the next byte and advances the position.
    #[inline]
    pub fn next_byte(&mut self) -> Result<u8, Error> {
        let b = self
            .data
            .get(self.pos)
            .copied()
            .ok_or(Error::UnexpectedEof)?;
        self.pos += 1;
        Ok(b)
    }

    /// Reads n bytes without advancing.
    #[inline]
    pub fn read(&mut self, n: usize) -> Result<&'de [u8], Error> {
        let end = self.pos + n;
        if end > self.data.len() {
            return Err(Error::UnexpectedEof);
        }
        let result = &self.data[self.pos..end];
        self.pos = end;
        Ok(result)
    }

    /// Reads n bytes without advancing.
    #[inline]
    pub fn peek_n(&self, n: usize) -> &'de [u8] {
        let end = (self.pos + n).min(self.data.len());
        &self.data[self.pos..end]
    }

    /// Fast skip - advances without bounds check (caller must ensure valid).
    #[inline]
    pub unsafe fn unsafe_advance(&mut self, n: usize) {
        self.pos += n;
    }

    /// Expects the next byte to be `expected`. Advances if it matches, returns an error otherwise.
    #[inline]
    pub fn expect_byte(&mut self, expected: u8) -> Result<(), Error> {
        let b = self
            .data
            .get(self.pos)
            .copied()
            .ok_or(Error::UnexpectedEof)?;
        self.pos += 1;
        if b != expected {
            Err(Error::UnexpectedByte {
                expected: match expected {
                    b'{' => "opening '{'",
                    b'}' => "closing '}'",
                    b'[' => "opening '['",
                    b']' => "closing ']'",
                    b':' => "':'",
                    b',' => "','",
                    b'"' => "quote",
                    _ => "byte",
                },
                got: b,
                offset: self.pos - 1,
            })
        } else {
            Ok(())
        }
    }

    /// Expects the next sequence of bytes to match `expected`.
    #[inline]
    pub fn expect_bytes(&mut self, expected: &[u8]) -> Result<(), Error> {
        let end = self.pos + expected.len();
        if end > self.data.len() {
            return Err(Error::UnexpectedEof);
        }
        if &self.data[self.pos..end] != expected {
            return Err(Error::UnexpectedByte {
                expected: "expected bytes",
                got: self.data.get(self.pos).copied().unwrap_or(0),
                offset: self.pos,
            });
        }
        self.pos = end;
        Ok(())
    }

    /// Checks if the byte at `offset` matches `expected`. Does not advance the position.
    #[inline]
    pub fn expect_at(&self, offset: usize, expected: u8) -> Result<(), Error> {
        let got = self.data.get(offset).copied().ok_or(Error::UnexpectedEof)?;
        if got != expected {
            Err(Error::UnexpectedByte {
                expected: "byte",
                got,
                offset,
            })
        } else {
            Ok(())
        }
    }

    #[inline]
    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.pos)
    }

    #[inline]
    pub fn slice_from(&self, start: usize) -> &'de [u8] {
        &self.data[start..self.pos]
    }

    #[inline]
    pub fn peek_slice(&self, len: usize) -> &'de [u8] {
        let end = (self.pos + len).min(self.data.len());
        &self.data[self.pos..end]
    }

    #[inline]
    pub fn skip(&mut self, n: usize) {
        self.pos = (self.pos + n).min(self.data.len());
    }
}

#[cfg(feature = "std")]
pub struct IoWriter<W: std::io::Write>(pub W);

#[cfg(feature = "std")]
impl<W: std::io::Write> WriteBuffer for IoWriter<W> {
    fn write_byte(&mut self, b: u8) -> Result<(), Error> {
        self.0.write_all(&[b]).map_err(Error::from)
    }

    fn write_bytes(&mut self, bs: &[u8]) -> Result<(), Error> {
        self.0.write_all(bs).map_err(Error::from)
    }
}
