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
        if self.len() < bytes.len() {
            return Err(Error::UnexpectedEof);
        }
        self[..bytes.len()].copy_from_slice(bytes);
        *self = &mut std::mem::take(self)[bytes.len()..];
        Ok(())
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

    /// Peeks at the next byte without advancing the position.
    #[inline]
    pub fn peek(&self) -> u8 {
        self.data.get(self.pos).copied().unwrap_or(0)
    }

    /// Returns the current reading position.
    #[inline]
    pub fn get_pos(&self) -> usize {
        self.pos
    }

    /// Checks if the buffer has reached the end.
    #[inline]
    pub fn is_eof(&self) -> bool {
        self.pos >= self.data.len()
    }

    /// Advances the reading position by `n` bytes.
    #[inline]
    pub fn advance(&mut self, n: usize) {
        self.pos += n;
    }

    /// Reads the next byte and advances the position.
    #[inline]
    pub fn next_byte(&mut self) -> Result<u8, Error> {
        self.data
            .get(self.pos)
            .copied()
            .inspect(|_| {
                self.pos += 1;
            })
            .ok_or(Error::UnexpectedEof)
    }

    /// Expects the next byte to be `expected`. Advances if it matches, returns an error otherwise.
    #[inline]
    pub fn expect_byte(&mut self, expected: u8) -> Result<(), Error> {
        let got = self.next_byte()?;
        if got != expected {
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
                got,
                offset: self.pos - 1,
            })
        } else {
            Ok(())
        }
    }

    /// Expects the next sequence of bytes to match `expected`.
    pub fn expect_bytes(&mut self, expected: &[u8]) -> Result<(), Error> {
        for &b in expected.iter() {
            self.expect_byte(b)?;
        }
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
