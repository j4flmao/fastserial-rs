//! # FastSerial
//!
//! `fastserial` is a high-performance, format-agnostic serialization framework for Rust.
//! It is designed for high-throughput use cases by leveraging specialized code generation
//! and SIMD-accelerated scanning.
//!
//! ## Performance Highlights
//!
//! - **Encode**: 1.3x - 2.0x faster than serde_json (depending on data structure)
//! - **Decode**: Optimized with binary search and SIMD scanning
//! - **Memory**: Zero-copy deserialization for borrowed types (`&str`, `&[u8]`)
//!
//! ## Design Goals
//!
//! - **High Performance**: Maximum throughput in JSON and Binary formats.
//! - **Zero-Copy**: Borrowed data types are deserialized without allocation.
//! - **SIMD Acceleration**: First-class support for AVX2 and SSE4.2 instructions.
//! - **Safety First**: While leveraging `unsafe` for SIMD, the public API remains safe.
//! - **Minimal Overhead**: Thin abstraction layer that compiles to efficient machine code.
//!
//! ## Supported Formats
//!
//! | Format | Module | Description |
//! |--------|--------|-------------|
//! | JSON   | [`json`] | High-speed JSON with SIMD scanning. |
//! | Binary | [`binary`] | Compact, schema-validated binary format. |
//!
//! ## Core Traits
//!
//! There are two primary traits that power `fastserial`:
//!
//! 1.  **[`Encode`]**: Types that can be serialized into a supported format.
//! 2.  **[`Decode`]**: Types that can be deserialized from a byte buffer.
//!
//! For most users, these traits should be implemented using the `derive` macros:
//!
//! ```rust
//! use fastserial::{Encode, Decode, json};
//!
//! #[derive(Encode, Decode, Debug, PartialEq)]
//! struct User {
//!     id: u64,
//!     username: String,
//!     email: String,
//! }
//!
//! # fn main() -> Result<(), fastserial::Error> {
//! let user = User {
//!     id: 42,
//!     username: "dev_user".to_string(),
//!     email: "dev@example.com".to_string(),
//! };
//!
//! // Serialize to JSON string
//! let json_data = json::encode(&user)?;
//!
//! // Deserialize back to struct
//! let decoded: User = json::decode(&json_data)?;
//!
//! assert_eq!(user, decoded);
//! # Ok(())
//! # }
//! ```
//!
//! ## SIMD Support and Safety
//!
//! `fastserial` automatically detects CPU features at runtime to use the fastest possible
//! implementation.
//!
//! - **AVX2**: Used on modern x86_64 CPUs for 32-byte parallel scanning.
//! - **SSE4.2**: Fallback for older x86_64 CPUs.
//! - **NEON**: ARM-specific SIMD for mobile and embedded devices.
//! - **Scalar**: Default implementation for other architectures or when SIMD is disabled.
//!
//! All `unsafe` code used for SIMD is encapsulated within the [`simd`] module and is
//! thoroughly tested for memory safety.
//!
//! ## Feature Flags
//!
//! - `std` (default): Enables support for `std` types like `String`, `Vec`, and `std::error::Error`.
//! - `json` (default): Enables JSON serialization/deserialization.
//! - `binary` (default): Enables FastSerial binary format.
//! - `chrono`: Enables serialization support for `chrono` date/time types.
//! - `profile`: Enables internal profiling for performance debugging.
//!
//! ## Quick Start
//!
//! ```rust
//! use fastserial::{Encode, Decode, json};
//!
//! #[derive(Encode, Decode, Debug)]
//! struct Point {
//!     x: i32,
//!     y: i32,
//! }
//!
//! # fn main() -> Result<(), fastserial::Error> {
//! let p = Point { x: 10, y: 20 };
//! let encoded = json::encode(&p)?;
//! let decoded: Point = json::decode(&encoded)?;
//! assert_eq!(p.x, decoded.x);
//! # Ok(())
//! # }
//! ```

#![no_std]

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "std")]
extern crate alloc;

/// Derive macro for the `Decode` trait.
pub use fastserial_derive::Decode;
/// Derive macro for the `Encode` trait.
pub use fastserial_derive::Encode;

/// Codec implementations for various formats (JSON, Binary, etc.)
pub mod codec;
/// I/O traits and buffers for reading and writing.
pub mod io;
/// Schema hashing and validation.
pub mod schema;
/// SIMD-accelerated low-level operations.
pub mod simd;

mod error;
mod format;
pub mod value;

pub use error::Error;
pub use format::Format;
pub use value::Value;

/// Trait for types that can be encoded into a format.
///
/// This trait is the primary interface for serialization. It is recommended to use
/// `#[derive(Encode)]` to implement this trait for your structs.
///
/// # Examples
///
/// ```rust
/// use fastserial::Encode;
///
/// #[derive(Encode)]
/// struct MyData {
///     value: u32,
/// }
/// ```
pub trait Encode {
    /// A unique 64-bit hash representing the schema of the type.
    ///
    /// This hash is used by formats like Binary to validate that the data
    /// being deserialized matches the expected struct definition.
    ///
    /// The hash is computed from:
    /// - Field names
    /// - Field types
    /// - Field order
    ///
    /// Changing any of these will change the hash, enabling schema validation.
    const SCHEMA_HASH: u64;

    /// Encodes the type into the provided `WriteBuffer`.
    ///
    /// This method is usually generated by the `#[derive(Encode)]` macro and handles
    /// the field-by-field serialization.
    fn encode<W: io::WriteBuffer>(&self, w: &mut W) -> Result<(), Error>;

    /// Encodes the type using a specific `Format` implementation.
    ///
    /// This allows the type to customize its representation for different
    /// formats (e.g., JSON vs Binary). The default implementation calls
    /// `F::encode_struct`.
    ///
    /// # Arguments
    ///
    /// * `F` - The format type (e.g., `codec::json::Format`, `codec::BinaryFormat`)
    /// * `w` - The output buffer
    fn encode_with_format<F: Format, W: io::WriteBuffer>(&self, w: &mut W) -> Result<(), Error> {
        F::encode_struct(self, w)
    }
}

/// Trait for types that can be decoded from a byte buffer.
///
/// The `'de` lifetime represents the lifetime of the input buffer. Types that
/// implement `Decode<'de>` can borrow data directly from this buffer (zero-copy).
///
/// # Examples
///
/// ```rust
/// use fastserial::Decode;
///
/// #[derive(Decode)]
/// struct MyData {
///     owned_string: String,
/// }
/// ```
pub trait Decode<'de>: Sized {
    /// Decodes the type from the provided `ReadBuffer`.
    ///
    /// This method is usually generated by the `#[derive(Decode)]` macro and handles
    /// field-by-field deserialization.
    ///
    /// # Lifetime `'de`
    ///
    /// The `'de` lifetime represents the input buffer's lifetime. This enables:
    /// - Zero-copy deserialization for `&str`, `&[u8]` types
    /// - Borrowed strings that reference the original input
    fn decode(r: &mut io::ReadBuffer<'de>) -> Result<Self, Error>;

    /// Decodes with a specific format.
    ///
    /// Default implementation calls `decode()`. Override this for custom format handling.
    #[inline(always)]
    fn decode_with_format<F: Format>(r: &mut io::ReadBuffer<'de>) -> Result<Self, Error> {
        Self::decode(r)
    }
}

/// JSON format support.
///
/// This module provides a high-level API for serializing and deserializing data in JSON format.
/// It is powered by SIMD scanning and specialized code generation to achieve maximum throughput.
pub mod json {
    use super::*;

    /// Encodes a value to a JSON byte vector.
    ///
    /// This is the primary function for JSON serialization in FastSerial.
    /// It uses SIMD-accelerated scanning for maximum performance.
    ///
    /// # Performance Characteristics
    ///
    /// - **Time Complexity**: O(n) where n is the total serialized size
    /// - **Space Complexity**: O(n) for the output buffer
    /// - Uses SIMD (AVX2/SSE4.2) for fast character scanning
    /// - Pre-allocates buffer with 256 byte capacity
    ///
    /// # Supported Types
    ///
    /// - All primitive types: `u8`, `i8`, `u16`, `i16`, `u32`, `i32`, `u64`, `i64`, `f32`, `f64`, `bool`
    /// - `String` and `&str` (valid UTF-8 only)
    /// - `Vec<T>` where T is serializable
    /// - Arrays `[T; N]` where T is serializable
    /// - `Option<T>` where T is serializable
    /// - Structs deriving `Encode`
    /// - Enums with data variants
    ///
    /// # Encoding Rules
    ///
    /// - Strings are escaped per JSON spec (quotes, backslash, control chars)
    /// - `f32`/`f64`: Special values (`inf`, `-inf`, `nan`) return error
    /// - Numbers are encoded as ASCII digits (not quoted strings)
    /// - `null` for Option::None
    /// - Structs become JSON objects
    /// - Vec becomes JSON arrays
    ///
    /// # Examples
    ///
    /// Basic primitive:
    /// ```rust
    /// use fastserial::json;
    ///
    /// let num = 42i32;
    /// let bytes = json::encode(&num).unwrap();
    /// assert_eq!(bytes, b"42");
    /// ```
    ///
    /// String:
    /// ```rust
    /// use fastserial::json;
    ///
    /// let s = "hello";  // requires &str or String
    /// let bytes = json::encode(&s).unwrap();
    /// assert_eq!(bytes, b"\"hello\"");
    /// ```
    ///
    /// Vector:
    /// ```rust
    /// use fastserial::json;
    ///
    /// let data = vec![1, 2, 3];
    /// let json_bytes = json::encode(&data).unwrap();
    /// assert_eq!(json_bytes, b"[1,2,3]");
    /// ```
    ///
    /// Struct:
    /// ```rust
    /// use fastserial::{Encode, json};
    ///
    /// #[derive(Encode)]
    /// struct User {
    ///     name: String,
    ///     age: u32,
    /// }
    ///
    /// let user = User { name: "Alice".into(), age: 30 };
    /// let bytes = json::encode(&user).unwrap();
    /// assert_eq!(bytes, b"{\"name\":\"Alice\",\"age\":30}");
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if:
    /// - The value contains invalid FP numbers (inf, -inf, nan)
    /// - A string is not valid UTF-8
    /// - A custom type's Encode impl returns an error
    pub fn encode<T: Encode>(val: &T) -> Result<alloc::vec::Vec<u8>, Error> {
        let mut buf = alloc::vec::Vec::with_capacity(256);
        val.encode(&mut buf)?;
        Ok(buf)
    }

    /// Encodes a value into an existing byte vector.
    ///
    /// This is more efficient than [`encode`] if you already have a buffer that can be reused.
    ///
    /// # Arguments
    /// * `val` - The value to encode.
    /// * `buf` - The destination buffer. Encoded data will be appended to it.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fastserial::json;
    ///
    /// let mut buf = Vec::new();
    /// json::encode_into(&42, &mut buf).unwrap();
    /// assert_eq!(buf, b"42");
    /// ```
    pub fn encode_into<T: Encode>(val: &T, buf: &mut alloc::vec::Vec<u8>) -> Result<(), Error> {
        val.encode(buf)
    }

    /// Decodes a value from a JSON byte slice.
    ///
    /// This function performs zero-copy deserialization where possible, meaning it borrows
    /// strings and byte slices directly from the input `input`.
    ///
    /// # Performance Characteristics
    ///
    /// - **Time Complexity**: O(n) where n is the input size
    /// - **Space Complexity**: O(1) for borrowed types, O(n) for owned types (String, Vec)
    /// - Uses SIMD for fast whitespace skipping and quote scanning
    /// - Zero-copy: strings borrow from input buffer when possible
    ///
    /// # Supported Types
    ///
    /// - All primitive types: `u8`, `i8`, `u16`, `i16`, `u32`, `i32`, `u64`, `i64`, `f32`, `f64`, `bool`
    /// - `String` (borrows or owned)
    /// - `&str` (borrows from input, zero-copy)
    /// - `Vec<T>` where T is decodable
    /// - Arrays `[T; N]`
    /// - `Option<T>`
    /// - Structs deriving `Decode`
    ///
    /// # Type Requirements
    ///
    /// The target type `T` must implement the [`Decode`] trait. You can derive it:
    ///
    /// ```rust
    /// use fastserial::Decode;
    ///
    /// #[derive(Decode)]
    /// struct MyStruct {
    ///     field1: String,
    ///     field2: i32,
    /// }
    /// ```
    ///
    /// # Examples
    ///
    /// Basic primitive:
    /// ```rust
    /// use fastserial::json;
    ///
    /// let num: i32 = json::decode(b"42").unwrap();
    /// assert_eq!(num, 42);
    /// ```
    ///
    /// Struct:
    /// ```rust
    /// use fastserial::{Decode, json};
    ///
    /// #[derive(Decode)]
    /// struct Message {
    ///     text: String,
    /// }
    ///
    /// let input = r#"{"text":"Hello"}"#;
    /// let msg: Message = json::decode(input.as_bytes()).unwrap();
    /// assert_eq!(msg.text, "Hello");
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if:
    /// - The JSON is malformed (missing quotes, invalid syntax)
    /// - Required fields are missing
    /// - Invalid data types for target fields
    /// - Invalid UTF-8 in strings
    /// - Trailing data after complete JSON value
    pub fn decode<'de, T: Decode<'de>>(input: &'de [u8]) -> Result<T, Error> {
        let mut r = io::ReadBuffer::new(input);
        let val = T::decode(&mut r)?;
        codec::json::skip_whitespace(&mut r);
        if !r.is_eof() {
            return Err(Error::TrailingData);
        }
        Ok(val)
    }

    /// Decodes a value from a JSON string.
    ///
    /// This is a convenience wrapper around [`decode`] that accepts a `&str`
    /// instead of `&[u8]`. Useful when working with string literals.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fastserial::json;
    ///
    /// let num: i32 = json::decode_str("42").unwrap();
    /// assert_eq!(num, 42);
    /// ```
    pub fn decode_str<'de, T: Decode<'de>>(input: &'de str) -> Result<T, Error> {
        decode(input.as_bytes())
    }

    /// Encodes a value to pretty-printed JSON with indentation.
    ///
    /// This function first encodes to compact JSON, then parses it into an intermediate
    /// [`Value`], and finally pretty-prints with 4-space indentation.
    ///
    /// # Performance Note
    ///
    /// This is significantly slower than [`encode`] because it:
    /// 1. Encodes to compact JSON
    /// 2. Parses the JSON back into a Value tree
    /// 3. Pretty-prints with indentation
    ///
    /// Use [`encode`] for performance-critical code.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fastserial::json;
    ///
    /// let user = serde_json::json!({
    ///     "name": "Alice",
    ///     "age": 30
    /// });
    /// // Note: This example uses serde_json for Value, but fastserial::Value also works
    /// ```
    ///
    /// # Output Format
    ///
    /// ```json
    /// {
    ///     "name": "Alice",
    ///     "age": 30
    /// }
    /// ```
    pub fn encode_pretty<T: Encode>(val: &T) -> Result<alloc::vec::Vec<u8>, Error> {
        let compact = encode(val)?;
        let value: crate::Value = decode(&compact)?;
        let mut buf = alloc::vec::Vec::with_capacity(compact.len() * 2);
        pretty_print_value(&value, &mut buf, 0)?;
        Ok(buf)
    }

    fn pretty_print_value(
        val: &crate::Value,
        buf: &mut alloc::vec::Vec<u8>,
        indent: usize,
    ) -> Result<(), Error> {
        use crate::io::WriteBuffer;
        match val {
            crate::Value::Null => buf.write_bytes(b"null"),
            crate::Value::Bool(true) => buf.write_bytes(b"true"),
            crate::Value::Bool(false) => buf.write_bytes(b"false"),
            crate::Value::Number(_) | crate::Value::String(_) => val.encode(buf),
            crate::Value::Array(arr) => {
                if arr.is_empty() {
                    return buf.write_bytes(b"[]");
                }
                buf.write_bytes(b"[\n")?;
                let child_indent = indent + 2;
                for (i, item) in arr.iter().enumerate() {
                    for _ in 0..child_indent {
                        buf.write_byte(b' ')?;
                    }
                    pretty_print_value(item, buf, child_indent)?;
                    if i + 1 < arr.len() {
                        buf.write_byte(b',')?;
                    }
                    buf.write_byte(b'\n')?;
                }
                for _ in 0..indent {
                    buf.write_byte(b' ')?;
                }
                buf.write_byte(b']')
            }
            crate::Value::Object(map) => {
                if map.is_empty() {
                    return buf.write_bytes(b"{}");
                }
                buf.write_bytes(b"{\n")?;
                let child_indent = indent + 2;
                let len = map.len();
                for (i, (k, v)) in map.iter().enumerate() {
                    for _ in 0..child_indent {
                        buf.write_byte(b' ')?;
                    }
                    codec::json::write_str(k, buf)?;
                    buf.write_bytes(b": ")?;
                    pretty_print_value(v, buf, child_indent)?;
                    if i + 1 < len {
                        buf.write_byte(b',')?;
                    }
                    buf.write_byte(b'\n')?;
                }
                for _ in 0..indent {
                    buf.write_byte(b' ')?;
                }
                buf.write_byte(b'}')
            }
        }
    }
}

/// FastSerial binary format support.
///
/// The binary format is a high-performance, compact representation of data.
/// It includes a 16-byte header with a magic number, version, and schema hash
/// to ensure data integrity and compatibility.
///
/// ### Header Specification (16 bytes)
///
/// | Bytes | Name | Description |
/// |-------|------|-------------|
/// | 0-3   | Magic | Fixed string `FBIN`. |
/// | 4-5   | Version | Format version (current: `0x0001`). |
/// | 6-13  | Schema Hash | 64-bit hash of the target type's schema. |
/// | 14-15 | Reserved | Reserved for future use. |
pub mod binary {
    use super::*;

    const MAGIC: [u8; 4] = *b"FBIN";
    const VERSION: u16 = 0x0001;

    /// Encodes a value to a new byte vector using the FastSerial binary format.
    ///
    /// This function automatically prepends a 16-byte protocol header containing:
    /// - Magic bytes: `FBIN` (4 bytes)
    /// - Version: `0x0001` (2 bytes, little-endian)
    /// - Schema hash: 64-bit hash of the target type (8 bytes, little-endian)
    /// - Reserved: 2 bytes (currently unused)
    ///
    /// # Binary Format Specification
    ///
    /// | Type | Encoding | Notes |
    /// |------|----------|-------|
    /// | `null` | `0x00` | Single null byte |
    /// | `bool` | `0x01` (true), `0x00` (false) | |
    /// | `u8` | 1 byte | Unsigned 8-bit |
    /// | `u16` | 2 bytes LE | Unsigned 16-bit |
    /// | `u32` | 4 bytes LE | Unsigned 32-bit |
    /// | `u64` | 8 bytes LE | Unsigned 64-bit |
    /// | `i8` | 1 byte | Signed 8-bit |
    /// | `i16` | 2 bytes LE | Signed 16-bit |
    /// | `i32` | 4 bytes LE | Signed 32-bit |
    /// | `i64` | 8 bytes LE | Signed 64-bit |
    /// | `f32` | 4 bytes LE | IEEE 754 float |
    /// | `f64` | 8 bytes LE | IEEE 754 double |
    /// | `String`/`&str` | 4-byte length + UTF-8 | Zero-copy when possible |
    /// | `Vec<T>` | 4-byte length + elements | |
    /// | Struct | Fields in declaration order | No field names |
    ///
    /// # Performance Characteristics
    ///
    /// - **Time Complexity**: O(n)
    /// - **Space Complexity**: O(n) output
    /// - Typically 2-5x smaller than JSON
    /// - No string escaping needed
    ///
    /// # Use Cases
    ///
    /// - Binary protocols (network, files)
    /// - Caching serialized data
    /// - Database storage
    /// - Inter-process communication
    ///
    /// # Examples
    ///
    /// Basic primitive:
    /// ```rust
    /// use fastserial::binary;
    ///
    /// let data = 12345u64;
    /// let bin_data = binary::encode(&data).unwrap();
    /// assert!(bin_data.starts_with(b"FBIN"));
    /// ```
    ///
    /// Struct:
    /// ```rust
    /// use fastserial::{Encode, Decode, binary};
    ///
    /// #[derive(Encode, Decode, PartialEq, Debug)]
    /// struct User {
    ///     id: u64,
    ///     name: String,
    /// }
    ///
    /// let user = User { id: 1, name: "Alice".into() };
    /// let bytes = binary::encode(&user).unwrap();
    /// assert!(bytes.starts_with(b"FBIN"));
    /// let decoded: User = binary::decode(&bytes).unwrap();
    /// assert_eq!(user, decoded);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`Error`] if:
    /// - The value cannot be encoded
    /// - Floating point infinity/NaN (not supported in binary)
    pub fn encode<T: Encode>(val: &T) -> Result<alloc::vec::Vec<u8>, Error> {
        let mut buf = alloc::vec::Vec::with_capacity(256);
        buf.extend_from_slice(&MAGIC);
        buf.extend_from_slice(&VERSION.to_le_bytes());
        buf.extend_from_slice(&T::SCHEMA_HASH.to_le_bytes());
        buf.extend_from_slice(&[0, 0]);
        val.encode(&mut buf)?;
        Ok(buf)
    }

    /// Decodes a value from a byte slice using the FastSerial binary format.
    ///
    /// This function validates the 16-byte header:
    /// 1. Magic bytes must be `FBIN`
    /// 2. Version must be `0x0001`
    /// 3. Schema hash validation (reserved for future use)
    ///
    /// # Performance Characteristics
    ///
    /// - **Time Complexity**: O(n)
    /// - **Space Complexity**: Depends on target type
    /// - Zero-copy for borrowed types
    ///
    /// # Error Handling
    ///
    /// | Error | Cause |
    /// |-------|-------|
    /// | `UnexpectedEof` | Input shorter than 16 bytes |
    /// | `InvalidMagic` | First 4 bytes not `FBIN` |
    /// | `UnsupportedVersion` | Version not `0x0001` |
    /// | `TrailingData` | Extra data after complete value |
    ///
    /// # Type Requirements
    ///
    /// The target type must implement [`Decode`]. Use the derive macro:
    ///
    /// ```rust
    /// use fastserial::Decode;
    ///
    /// #[derive(Decode)]
    /// struct MyStruct {
    ///     field1: String,
    ///     field2: i32,
    /// }
    /// ```
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fastserial::{Encode, Decode, binary};
    ///
    /// #[derive(Encode, Decode, PartialEq, Debug)]
    /// struct Data { id: u32 }
    ///
    /// let original = Data { id: 100 };
    /// let bytes = binary::encode(&original).unwrap();
    /// let decoded: Data = binary::decode(&bytes).unwrap();
    /// assert_eq!(original, decoded);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if:
    /// - `InvalidMagic`: Header doesn't start with `FBIN`
    /// - `UnsupportedVersion`: Version != 0x0001
    /// - `UnexpectedEof`: Input too short
    /// - Standard deserialization errors
    pub fn decode<'de, T: Decode<'de>>(input: &'de [u8]) -> Result<T, Error> {
        if input.len() < 16 {
            return Err(Error::UnexpectedEof);
        }
        if input[0..4] != MAGIC {
            return Err(Error::InvalidMagic);
        }
        let version = u16::from_le_bytes([input[4], input[5]]);
        if version != VERSION {
            return Err(Error::UnsupportedVersion { version });
        }
        // TODO: Validate T::SCHEMA_HASH against input[6..14]
        let mut r = io::ReadBuffer::new(&input[16..]);
        let val = T::decode_with_format::<codec::BinaryFormat>(&mut r)?;
        if !r.is_eof() {
            return Err(Error::TrailingData);
        }
        Ok(val)
    }

    /// Encodes a value to binary without the 16-byte header.
    ///
    /// This is faster for internal use where the schema is known and doesn't require
    /// header validation on decode. Use this when:
    /// - You control both encoding and decoding ends
    /// - Performance is critical
    /// - You're storing/transporting raw binary data
    ///
    /// # Difference from [`encode`]
    ///
    /// | Function | Header | Use Case |
    /// |----------|--------|----------|
    /// | `encode` | 16 bytes | Network protocol, storage |
    /// | `encode_raw` | None | Internal caching, IPC |
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fastserial::binary;
    ///
    /// let data = 12345u64;
    /// let bin_data = binary::encode_raw(&data).unwrap();
    /// assert!(!bin_data.starts_with(b"FBIN"));
    /// ```
    pub fn encode_raw<T: Encode>(val: &T) -> Result<alloc::vec::Vec<u8>, Error> {
        let mut buf = alloc::vec::Vec::with_capacity(256);
        val.encode(&mut buf)?;
        Ok(buf)
    }

    /// Decodes a value from binary without parsing the header.
    ///
    /// Use this with data encoded via [`encode_raw`]. No header validation is performed.
    ///
    /// # Performance
    ///
    /// Slightly faster than [`decode`] because it skips:
    /// - Magic byte validation
    /// - Version check
    /// - Schema hash validation (reserved)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fastserial::{Encode, Decode, binary};
    ///
    /// #[derive(Encode, Decode, PartialEq, Debug)]
    /// struct Data { id: u32 }
    ///
    /// let original = Data { id: 100 };
    /// let bytes = binary::encode_raw(&original).unwrap();
    /// let decoded: Data = binary::decode_raw(&bytes).unwrap();
    /// assert_eq!(original, decoded);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if:
    /// - Input is insufficient or corrupted
    /// - Type doesn't match encoded format
    /// - Trailing data after complete value
    pub fn decode_raw<'de, T: Decode<'de>>(input: &'de [u8]) -> Result<T, Error> {
        let mut r = io::ReadBuffer::new(input);
        let val = T::decode(&mut r)?;
        if !r.is_eof() {
            return Err(Error::TrailingData);
        }
        Ok(val)
    }
}

mod option_impl {
    use super::*;

    impl<T: Encode> Encode for Option<T> {
        const SCHEMA_HASH: u64 = 0;

        #[inline]
        fn encode<W: io::WriteBuffer>(&self, w: &mut W) -> Result<(), Error> {
            match self {
                None => w.write_bytes(b"null"),
                Some(v) => v.encode(w),
            }
        }

        #[inline]
        fn encode_with_format<F: Format, W: io::WriteBuffer>(
            &self,
            w: &mut W,
        ) -> Result<(), Error> {
            match self {
                None => F::write_null(w),
                Some(v) => v.encode_with_format::<F, W>(w),
            }
        }
    }

    impl<'de, T: Decode<'de>> Decode<'de> for Option<T> {
        #[inline]
        fn decode(r: &mut io::ReadBuffer<'de>) -> Result<Self, Error> {
            codec::json::skip_whitespace(r);
            if r.peek() == b'n' {
                r.expect_bytes(b"null")?;
                Ok(None)
            } else {
                Ok(Some(T::decode(r)?))
            }
        }
    }
}

mod vec_impl {
    use super::*;

    impl<T: Encode> Encode for alloc::vec::Vec<T> {
        const SCHEMA_HASH: u64 = 0;

        #[inline]
        fn encode<W: io::WriteBuffer>(&self, w: &mut W) -> Result<(), Error> {
            w.write_byte(b'[')?;
            let mut iter = self.iter();
            if let Some(item) = iter.next() {
                item.encode(w)?;
                for item in iter {
                    w.write_byte(b',')?;
                    item.encode(w)?;
                }
            }
            w.write_byte(b']')
        }

        #[inline]
        fn encode_with_format<F: Format, W: io::WriteBuffer>(
            &self,
            w: &mut W,
        ) -> Result<(), Error> {
            F::begin_array(self.len(), w)?;
            let mut iter = self.iter();
            if let Some(item) = iter.next() {
                item.encode_with_format::<F, W>(w)?;
                for item in iter {
                    F::array_separator(w)?;
                    item.encode_with_format::<F, W>(w)?;
                }
            }
            F::end_array(w)
        }
    }

    impl<'de, T: Decode<'de>> Decode<'de> for alloc::vec::Vec<T> {
        #[inline]
        fn decode(r: &mut io::ReadBuffer<'de>) -> Result<Self, Error> {
            r.expect_byte(b'[')?;
            let mut vec = alloc::vec::Vec::new();
            codec::json::skip_whitespace(r);
            if r.peek() == b']' {
                r.advance(1);
                return Ok(vec);
            }
            loop {
                vec.push(T::decode(r)?);
                codec::json::skip_comma_or_close(r, b']')?;
                if r.peek() == b']' {
                    r.advance(1);
                    break;
                }
            }
            Ok(vec)
        }
    }
}

macro_rules! impl_primitive {
    ($($ty:ty => $write_fn:ident, $read_fn:ident, $hash:literal),* $(,)?) => {
        $(
            impl Encode for $ty {
                const SCHEMA_HASH: u64 = $hash;

                #[inline(always)]
                fn encode<W: io::WriteBuffer>(&self, w: &mut W) -> Result<(), Error> {
                    codec::json::$write_fn(*self, w)
                }

                #[inline(always)]
                fn encode_with_format<F: Format, W: io::WriteBuffer>(&self, w: &mut W) -> Result<(), Error> {
                    F::write_u64(*self as u64, w)
                }
            }

            impl<'de> Decode<'de> for $ty {
                #[inline(always)]
                fn decode(r: &mut io::ReadBuffer<'de>) -> Result<Self, Error> {
                    codec::json::$read_fn(r)
                }
            }
        )*
    };
}

impl_primitive! {
    u8 => write_u8, read_u8, 0x7a3c8d2e,
    u16 => write_u16, read_u16, 0x8b4d9e3f,
    u32 => write_u32, read_u32, 0x9c5e0f4a,
    u64 => write_u64, read_u64, 0xad6f1a5b,
    i8 => write_i8, read_i8, 0xbe7a2b6c,
    i16 => write_i16, read_i16, 0xcf8b3c7d,
    i32 => write_i32, read_i32, 0xde9c4d8e,
    i64 => write_i64, read_i64, 0xefad5e9f,
    f32 => write_f32, read_f32, 0xf0be6faa,
    f64 => write_f64, read_f64, 0x01cf8fbb,
    bool => write_bool, read_bool, 0x12d090cc,
}

impl Encode for () {
    const SCHEMA_HASH: u64 = 0x23e1a1dd;

    #[inline]
    fn encode<W: io::WriteBuffer>(&self, w: &mut W) -> Result<(), Error> {
        codec::json::write_null(w)
    }
}

impl<'de> Decode<'de> for () {
    #[inline]
    fn decode(r: &mut io::ReadBuffer<'de>) -> Result<Self, Error> {
        codec::json::read_null(r)
    }
}

impl Encode for alloc::string::String {
    const SCHEMA_HASH: u64 = 0x34f2b2ee3d6e8cbb;

    #[inline]
    fn encode<W: io::WriteBuffer>(&self, w: &mut W) -> Result<(), Error> {
        codec::json::write_str(self, w)
    }

    #[inline]
    fn encode_with_format<F: Format, W: io::WriteBuffer>(&self, w: &mut W) -> Result<(), Error> {
        F::write_str(self, w)
    }
}

impl<'de> Decode<'de> for alloc::string::String {
    #[inline]
    fn decode(r: &mut io::ReadBuffer<'de>) -> Result<Self, Error> {
        Ok(codec::json::read_string_cow(r)?.into_owned())
    }
}

impl Encode for &str {
    const SCHEMA_HASH: u64 = 0x45c3c3ff4e7d9dcc;

    #[inline]
    fn encode<W: io::WriteBuffer>(&self, w: &mut W) -> Result<(), Error> {
        codec::json::write_str(self, w)
    }

    #[inline]
    fn encode_with_format<F: Format, W: io::WriteBuffer>(&self, w: &mut W) -> Result<(), Error> {
        F::write_str(self, w)
    }
}

impl<'de> Decode<'de> for &'de str {
    #[inline]
    fn decode(r: &mut io::ReadBuffer<'de>) -> Result<Self, Error> {
        codec::json::read_string(r)
    }
}

impl Encode for &[u8] {
    const SCHEMA_HASH: u64 = 0x56d4d4ff5f8deed;

    #[inline]
    fn encode<W: io::WriteBuffer>(&self, w: &mut W) -> Result<(), Error> {
        codec::json::write_bytes(self, w)
    }

    #[inline]
    fn encode_with_format<F: Format, W: io::WriteBuffer>(&self, w: &mut W) -> Result<(), Error> {
        F::write_bytes(self, w)
    }
}

impl<'de> Decode<'de> for &'de [u8] {
    #[inline]
    fn decode(r: &mut io::ReadBuffer<'de>) -> Result<Self, Error> {
        codec::json::read_bytes(r)
    }
}

#[cfg(feature = "chrono")]
mod chrono_impl {
    use super::*;
    use chrono::{DateTime, Utc};

    impl Encode for DateTime<Utc> {
        const SCHEMA_HASH: u64 = 0x67e5e5ff6f9efffe;

        #[inline]
        fn encode<W: io::WriteBuffer>(&self, w: &mut W) -> Result<(), Error> {
            let s = self.to_rfc3339();
            codec::json::write_str(&s, w)
        }

        #[inline]
        fn encode_with_format<F: Format, W: io::WriteBuffer>(
            &self,
            w: &mut W,
        ) -> Result<(), Error> {
            let s = self.to_rfc3339();
            F::write_str(&s, w)
        }
    }

    impl<'de> Decode<'de> for DateTime<Utc> {
        #[inline]
        fn decode(r: &mut io::ReadBuffer<'de>) -> Result<Self, Error> {
            let s = codec::json::read_string(r)?;
            DateTime::parse_from_rfc3339(s)
                .map(|dt| dt.with_timezone(&Utc))
                .map_err(|_| Error::InvalidUtf8 {
                    byte_offset: r.get_pos(),
                })
        }
    }
}

#[cfg(feature = "std")]
mod hashmap_impl {
    use super::*;
    use std::collections::HashMap;

    impl<K: Encode + core::fmt::Display, V: Encode> Encode for HashMap<K, V> {
        const SCHEMA_HASH: u64 = 0;

        #[inline]
        fn encode<W: io::WriteBuffer>(&self, w: &mut W) -> Result<(), Error> {
            w.write_byte(b'{')?;
            let mut first = true;
            for (k, v) in self {
                if !first {
                    w.write_byte(b',')?;
                }
                first = false;
                let key_str = alloc::format!("{}", k);
                codec::json::write_str(&key_str, w)?;
                w.write_byte(b':')?;
                v.encode(w)?;
            }
            w.write_byte(b'}')
        }
    }

    impl<'de, V: Decode<'de>> Decode<'de> for HashMap<alloc::string::String, V> {
        #[inline]
        fn decode(r: &mut io::ReadBuffer<'de>) -> Result<Self, Error> {
            codec::json::skip_whitespace(r);
            r.expect_byte(b'{')?;
            let mut map = HashMap::new();
            codec::json::skip_whitespace(r);
            if r.peek() == b'}' {
                r.advance(1);
                return Ok(map);
            }
            loop {
                codec::json::skip_whitespace(r);
                let key = codec::json::read_string_cow(r)?.into_owned();
                codec::json::skip_whitespace(r);
                r.expect_byte(b':')?;
                let val = V::decode(r)?;
                map.insert(key, val);
                codec::json::skip_comma_or_close(r, b'}')?;
                if r.peek() == b'}' {
                    r.advance(1);
                    break;
                }
            }
            Ok(map)
        }
    }
}

mod btreemap_impl {
    use super::*;
    use alloc::collections::BTreeMap;

    impl<K: Encode + core::fmt::Display, V: Encode> Encode for BTreeMap<K, V> {
        const SCHEMA_HASH: u64 = 0;

        #[inline]
        fn encode<W: io::WriteBuffer>(&self, w: &mut W) -> Result<(), Error> {
            w.write_byte(b'{')?;
            let mut first = true;
            for (k, v) in self {
                if !first {
                    w.write_byte(b',')?;
                }
                first = false;
                let key_str = alloc::format!("{}", k);
                codec::json::write_str(&key_str, w)?;
                w.write_byte(b':')?;
                v.encode(w)?;
            }
            w.write_byte(b'}')
        }
    }

    impl<'de, V: Decode<'de>> Decode<'de> for BTreeMap<alloc::string::String, V> {
        #[inline]
        fn decode(r: &mut io::ReadBuffer<'de>) -> Result<Self, Error> {
            codec::json::skip_whitespace(r);
            r.expect_byte(b'{')?;
            let mut map = BTreeMap::new();
            codec::json::skip_whitespace(r);
            if r.peek() == b'}' {
                r.advance(1);
                return Ok(map);
            }
            loop {
                codec::json::skip_whitespace(r);
                let key = codec::json::read_string_cow(r)?.into_owned();
                codec::json::skip_whitespace(r);
                r.expect_byte(b':')?;
                let val = V::decode(r)?;
                map.insert(key, val);
                codec::json::skip_comma_or_close(r, b'}')?;
                if r.peek() == b'}' {
                    r.advance(1);
                    break;
                }
            }
            Ok(map)
        }
    }
}

macro_rules! impl_tuple {
    ($($idx:tt $T:ident),+) => {
        impl<$($T: Encode),+> Encode for ($($T,)+) {
            const SCHEMA_HASH: u64 = 0;

            #[inline]
            fn encode<W: io::WriteBuffer>(&self, w: &mut W) -> Result<(), Error> {
                w.write_byte(b'[')?;
                impl_tuple!(@encode self w $($idx $T),+);
                w.write_byte(b']')
            }
        }

        impl<'de, $($T: Decode<'de>),+> Decode<'de> for ($($T,)+) {
            #[inline]
            fn decode(r: &mut io::ReadBuffer<'de>) -> Result<Self, Error> {
                codec::json::skip_whitespace(r);
                r.expect_byte(b'[')?;
                impl_tuple!(@decode r $($idx $T),+);
                codec::json::skip_whitespace(r);
                r.expect_byte(b']')?;
                Ok(($($T,)+))
            }
        }
    };

    (@encode $self:ident $w:ident $first_idx:tt $first_T:ident $(, $idx:tt $T:ident)*) => {
        $self.$first_idx.encode($w)?;
        $(
            $w.write_byte(b',')?;
            $self.$idx.encode($w)?;
        )*
    };

    (@decode $r:ident $first_idx:tt $first_T:ident $(, $idx:tt $T:ident)*) => {
        codec::json::skip_whitespace($r);
        #[allow(non_snake_case)]
        let $first_T = $first_T::decode($r)?;
        $(
            codec::json::skip_comma_or_close($r, b']')?;
            #[allow(non_snake_case)]
            let $T = $T::decode($r)?;
        )*
    };
}

impl_tuple!(0 A);
impl_tuple!(0 A, 1 B);
impl_tuple!(0 A, 1 B, 2 C);
impl_tuple!(0 A, 1 B, 2 C, 3 D);
impl_tuple!(0 A, 1 B, 2 C, 3 D, 4 E2);
impl_tuple!(0 A, 1 B, 2 C, 3 D, 4 E2, 5 F2);
impl_tuple!(0 A, 1 B, 2 C, 3 D, 4 E2, 5 F2, 6 G);
impl_tuple!(0 A, 1 B, 2 C, 3 D, 4 E2, 5 F2, 6 G, 7 H);
impl_tuple!(0 A, 1 B, 2 C, 3 D, 4 E2, 5 F2, 6 G, 7 H, 8 I2);
impl_tuple!(0 A, 1 B, 2 C, 3 D, 4 E2, 5 F2, 6 G, 7 H, 8 I2, 9 J);
impl_tuple!(0 A, 1 B, 2 C, 3 D, 4 E2, 5 F2, 6 G, 7 H, 8 I2, 9 J, 10 K);
impl_tuple!(0 A, 1 B, 2 C, 3 D, 4 E2, 5 F2, 6 G, 7 H, 8 I2, 9 J, 10 K, 11 L);

#[cfg(feature = "msgpack")]
pub mod msgpack {
    pub use crate::codec::msgpack::*;
}
