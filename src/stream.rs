#[cfg(feature = "std")]
use std::io::BufRead;

use crate::{Decode, Error, arena::Arena};

/// A synchronous streaming deserializer for NDJSON (Newline Delimited JSON).
#[cfg(feature = "std")]
pub struct NdjsonStream<'a, R> {
    reader: R,
    buffer: alloc::vec::Vec<u8>,
    arena: &'a Arena,
}

#[cfg(feature = "std")]
impl<'a, R: BufRead> NdjsonStream<'a, R> {
    pub fn new(reader: R, arena: &'a Arena) -> Self {
        Self {
            reader,
            buffer: alloc::vec::Vec::with_capacity(8192),
            arena,
        }
    }

    /// Read and decode the next JSON object from the stream.
    pub fn next_obj<'de, T: Decode<'de>>(&'de mut self) -> Option<Result<T, Error>> {
        loop {
            self.buffer.clear();
            match self.reader.read_until(b'\n', &mut self.buffer) {
                Ok(0) => return None,
                Ok(_) => {
                    while let Some(&last) = self.buffer.last() {
                        if last == b'\n' || last == b'\r' {
                            self.buffer.pop();
                        } else {
                            break;
                        }
                    }
                    if self.buffer.is_empty() {
                        continue;
                    }
                    return Some(crate::json::decode(&mut self.buffer, self.arena));
                }
                Err(_) => return Some(Err(Error::Custom)),
            }
        }
    }
}

/// An asynchronous streaming deserializer for NDJSON (Newline Delimited JSON) using `tokio`.
#[cfg(feature = "tokio")]
pub struct AsyncNdjsonStream<'a, R> {
    reader: R,
    buffer: alloc::vec::Vec<u8>,
    arena: &'a Arena,
}

#[cfg(feature = "tokio")]
impl<'a, R: tokio::io::AsyncBufRead + core::marker::Unpin> AsyncNdjsonStream<'a, R> {
    pub fn new(reader: R, arena: &'a Arena) -> Self {
        Self {
            reader,
            buffer: alloc::vec::Vec::with_capacity(8192),
            arena,
        }
    }

    /// Asynchronously read and decode the next JSON object from the stream.
    pub async fn next_obj<'de, T: Decode<'de>>(&'de mut self) -> Option<Result<T, Error>> {
        use tokio::io::AsyncBufReadExt;
        loop {
            self.buffer.clear();
            match self.reader.read_until(b'\n', &mut self.buffer).await {
                Ok(0) => return None,
                Ok(_) => {
                    while let Some(&last) = self.buffer.last() {
                        if last == b'\n' || last == b'\r' {
                            self.buffer.pop();
                        } else {
                            break;
                        }
                    }
                    if self.buffer.is_empty() {
                        continue;
                    }
                    return Some(crate::json::decode(&mut self.buffer, self.arena));
                }
                Err(_) => return Some(Err(Error::Custom)),
            }
        }
    }
}
